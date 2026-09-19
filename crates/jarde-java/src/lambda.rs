//! The verified `LambdaMetafactory` shape: which `invokedynamic` call sites this layer may present
//! as a lambda or a method reference, and which it must leave as bytecode (P3 2.1, acceptance A04).
//!
//! # The one thing this module exists for
//!
//! An `invokedynamic` site says almost nothing about itself: it names a bootstrap entry, and that
//! entry says *how* the site is resolved. Every lambda, every string-concatenation site, every
//! `ConstantBootstraps` use and every other dynamic-language call site in a class file has the same
//! instruction behind it. So "this site is a lambda" is not a fact this layer may assume from the
//! opcode, from the site's descriptor, from the presence of a `MethodHandle` in the pool, or from
//! anything else but the **bootstrap the site itself names**:
//!
//! ```text
//! invokedynamic #N ──▶ CP entry N: InvokeDynamic { bootstrap_method_attr_index = K, name, descriptor }
//!                                                     │
//!                       class attribute `BootstrapMethods` entry K
//!                                                     │
//!                    ┌────────────────────────────────┴───────────────────────────────┐
//!                    ▼                                                                ▼
//!        method handle: the factory                                 static arguments
//!        (must be java/lang/invoke/LambdaMetafactory.metafactory / altMetafactory)
//!                    │                                                                │
//!                    │                                    0: SAM method type (MethodType)
//!                    │                                    1: implementation handle (MethodHandle)
//!                    │                                    2: instantiated method type (MethodType)
//!                    ▼                                                                ▼
//!        handle kind decides the call shape                          descriptors decide the arity
//! ```
//!
//! Only that chain, read out of **this run's** bootstrap table and pool
//! ([`jarde_jvm::method_ir::MethodIr::bootstrap_methods`],
//! [`jarde_jvm::method_ir::MethodIr::constant_pool`]), produces a [`Plan`]. Anything else produces a
//! [`Refusal`] that states which link failed — a refusal is a *stated* outcome with a diagnostic
//! code, never a quieter presentation.
//!
//! # What is checked, and what a refusal means
//!
//! 1. the profile admits the `lambda@1` rule at all (its output is a Java 8 construct);
//! 2. the class's bootstrap table states the entry the site names;
//! 3. the entry's method handle is a method handle, and its member is
//!    `java/lang/invoke/LambdaMetafactory.metafactory` or `.altMetafactory` — **the A04 check**: a
//!    site whose bootstrap is anything else is refused here and never presented as a lambda;
//! 4. the factory's static arguments are the ones the contract names, by count and by constant kind
//!    (`altMetafactory`'s flag word must be zero: markers, bridges and serializable sites need a
//!    presentation this layer does not have);
//! 5. the site's descriptor, the SAM method type and the instantiated method type all parse;
//! 6. the implementation handle is an invocation (a field handle is not one this layer presents);
//! 7. **the arity lines up**: the operands the handle is given — the site's captures followed by the
//!    SAM's own parameters — are exactly its receiver plus its parameters, and each aligned position
//!    has the same descriptor shape. This is the check that keeps a site whose implementation
//!    disagrees with its SAM from being printed as a lambda that would not mean the same thing.
//!
//! What is *not* checked, and is therefore not claimed: adaptation. The JVM lets an implementation
//! method be adapted to the instantiated type (subtyping, boxing, widening) and this layer compares
//! reference types to reference types without naming them, and primitive shapes to the same
//! primitive shape. A site that needs any other adaptation is refused (`jre_lambda_sam_types`)
//! rather than presented with a conversion nobody wrote.
//!
//! # The two presentations, and what decides which is written
//!
//! Both writings mean the same function; which one is *readable* depends on what the implementation
//! handle is:
//!
//! * a **method reference** (`Qualifier::name`, or `Type::new`) when the site's captures are exactly
//!   what the handle's own receiver needs and nothing else — a bound receiver or nothing at all.
//!   Java's `::` has no form for bound *arguments*, so this is the only case where it can be written
//!   at all;
//! * a **lambda** (`(params) -> body`) otherwise, with the body the handle's own invocation:
//!   `impl(captures…, params…)` for a static handle, `capture0.impl(captures[1..]…, params…)` for an
//!   instance one, `new Owner(captures…, params…)` for a constructor.
//!
//! When the handle is a method the compiler generated for this very site — named with the marker
//! both javac and ECJ use for lambda bodies, `lambda$…` — the lambda writing is chosen even where a
//! `::` reference would have been possible, because the handle is the site's *own body* and not a
//! member the source named. The marker decides between two equivalent writings and **never** whether
//! a site is a lambda at all: that is decided by the bootstrap chain above. A class that happens to
//! hold a member named `lambda$…` therefore gets a lambda written over its call, which is still the
//! same function.
//!
//! # Capture order is not a detail
//!
//! The captured operands are read off the site's own value flow in the order the instruction reads
//! them off the stack ([`crate::build`] renders them), which is the order the implementation handle
//! receives them in — before the SAM's parameters, always. The segment table records each capture's
//! own BCI beside the site's, so "which original instruction produced this argument" survives into
//! the artifact instead of being answered by the impl handle's descriptor alone.

use serde::Serialize;

use jarde_reader::classfile::{BootstrapMethodFacts, CpEntryFacts, CpEntryKind, cp_entry};

use crate::ast::Type;
use crate::facts::DynamicSite;
use crate::pass::{IrTable, LAMBDA, Pass, Precondition, RecoveryProfile, RuleVersion};

/// The class that owns the factories this layer verifies.
const FACTORY_OWNER: &str = "java/lang/invoke/LambdaMetafactory";

/// The factory methods this layer verifies, as the class file spells them.
const FACTORY_METHODS: [&str; 2] = ["metafactory", "altMetafactory"];

/// The marker both javac and ECJ put in front of the body method they generate for a lambda site.
const BODY_MARKER: &str = "lambda$";

/// The method-handle kind of the static factory call this layer expects.
const REF_INVOKE_STATIC: u8 = 6;

/// The handle kinds whose invocation supplies the receiver first.
const REF_RECEIVER_KINDS: [u8; 3] = [5, 7, 9];

/// The handle kind of a constructor (`REF_newInvokeSpecial`).
const REF_NEW_INVOKE_SPECIAL: u8 = 8;

/// What one `invokedynamic` site was decided to be.
pub(crate) struct Verdict {
    /// What the class states about the site, resolved as far as it resolves: kept for the record
    /// even when the site is refused, because "which bootstrap was it" is the answer A04 asks for.
    pub(crate) evidence: Evidence,
    /// The presentation, or the reason there is none.
    pub(crate) outcome: Result<Plan, Refusal>,
}

/// Everything this layer read about one site, in the vocabulary a report reads back.
pub(crate) struct Evidence {
    /// The factory handle as `owner.name (kind)`, when the table states the entry and it resolves.
    pub(crate) bootstrap: Option<String>,
    /// How many static arguments that entry states.
    pub(crate) bootstrap_arguments: usize,
    /// The SAM method type the bootstrap states, when it states one.
    pub(crate) sam_method_type: Option<String>,
    /// The instantiated method type the bootstrap states, when it states one.
    pub(crate) instantiated_method_type: Option<String>,
    /// The implementation handle as `owner.name(descriptor) (kind)`, when it resolves.
    pub(crate) implementation: Option<String>,
}

/// How one verified site is written.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LambdaForm {
    /// `(params) -> body`, with the implementation called inside the body.
    Lambda,
    /// `Qualifier::name` — a reference to the implementation member itself.
    MethodReference,
}

/// The implementation handle of one verified site, resolved.
pub(crate) struct Member {
    kind: u8,
    owner: String,
    name: String,
    descriptor: String,
}

impl Member {
    /// Whether the handle's invocation supplies a receiver before its arguments.
    fn takes_receiver(&self) -> bool {
        REF_RECEIVER_KINDS.contains(&self.kind)
    }

    /// Whether the handle constructs a new instance (`REF_newInvokeSpecial`).
    fn constructs(&self) -> bool {
        self.kind == REF_NEW_INVOKE_SPECIAL
    }

    /// Whether the handle names an invocation the presentation can write as a call — as opposed to
    /// one of the four field-access kinds, which name no method this layer may call.
    fn is_invocation(&self) -> bool {
        REF_INVOKE_STATIC == self.kind || self.takes_receiver() || self.constructs()
    }

    /// Whether the handle is the body method a compiler generated for a lambda site.
    fn is_generated_body(&self) -> bool {
        self.name.starts_with(BODY_MARKER)
    }

    /// How the member is reached, which decides the call the presentation writes.
    pub(crate) fn reach(&self) -> Reach {
        if self.constructs() {
            Reach::Constructor
        } else if self.takes_receiver() {
            Reach::Receiver
        } else {
            Reach::Static
        }
    }

    /// The owner's internal name (`java/lang/Runnable`), as the class states it.
    pub(crate) fn owner(&self) -> &str {
        &self.owner
    }

    /// The member's name.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

/// How one implementation member is reached — which is what the body of a lambda calls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Reach {
    /// A static member: `Owner.name(args…)`.
    Static,
    /// An instance member: `receiver.name(args…)`, the first bound operand being the receiver.
    Receiver,
    /// A constructor: `new Owner(args…)`.
    Constructor,
}

/// One verified site, ready to be written.
pub(crate) struct Plan {
    /// Which of the two writings.
    pub(crate) form: LambdaForm,
    /// The parameters the lambda takes, in the order the SAM states them.
    pub(crate) params: Vec<Type>,
    /// The implementation member.
    pub(crate) implementation: Member,
    /// How that member is reached.
    pub(crate) reach: Reach,
    /// How many of the site's operands are the handle's leading arguments.
    pub(crate) captures: usize,
}

/// Why one site was not presented.
pub(crate) struct Refusal {
    code: &'static str,
    requirement: Option<Precondition>,
    message: String,
}

impl Refusal {
    /// A refusal of a site that simply is not the verified shape.
    pub(crate) fn shape(code: &'static str, message: String) -> Self {
        Self {
            code,
            requirement: None,
            message,
        }
    }

    /// A refusal under a **declared** precondition of the `lambda@1` rule.
    ///
    /// The debug assertion is the same one [`crate::region`]'s refusals carry: a requirement that is
    /// checked without being declared — or declared and never checked — fails this build's tests
    /// instead of drifting away from the declaration a reader consults.
    pub(crate) fn unmet(pass: &'static Pass, requirement: Precondition, message: String) -> Self {
        debug_assert!(
            pass.requires(requirement),
            "{} states no {requirement:?} precondition",
            pass.rule()
        );
        Self {
            code: requirement_code(requirement),
            requirement: Some(requirement),
            message,
        }
    }

    /// The diagnostic code this refusal is reported under.
    pub(crate) fn code(&self) -> &'static str {
        self.code
    }

    /// The declared requirement that fell short, when the refusal is one of those.
    pub(crate) fn requirement(&self) -> Option<Precondition> {
        self.requirement
    }

    /// What the refusal says, in one sentence. The caller states the BCI: the site is the caller's.
    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

/// The code a refusal under one declared requirement is reported with.
fn requirement_code(requirement: Precondition) -> &'static str {
    match requirement {
        Precondition::IrTable(IrTable::BootstrapMethods) => "jre_lambda_no_bootstrap",
        Precondition::Replayable => "jre_lambda_capture_not_replayable",
        _ => "jre_lambda_unmet_precondition",
    }
}

/// The pass answerable for every verdict of this module — the one that presents the shape or
/// refuses it.
pub(crate) const RULE: RuleVersion = LAMBDA.rule();

/// What one `invokedynamic` site of a body was presented as, or why it was not (P3 2.1).
///
/// This is the record A04 asks for, and it is the reason a refusal is as important as a
/// presentation: it states the **bootstrap** the class really named (the factory handle with its
/// method-handle kind, the number of static arguments, the two method types), the **use site** (the
/// BCI of the `invokedynamic` and the constant-pool index of its own entry), the SAM the site
/// presents (its name and the descriptor), the implementation handle, and every **captured
/// argument** in the order the site reads it — each with the BCI it came from. Nothing here is
/// re-derived from the produced text: it is the evidence the presentation was decided from.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LambdaRecord {
    /// The BCI of the `invokedynamic` instruction: the use-site evidence.
    pub use_site: u32,
    /// The constant-pool index of the site's own `InvokeDynamic` entry.
    pub site_cp: u16,
    /// The `BootstrapMethods` entry the site names.
    pub bootstrap_index: u16,
    /// The factory handle as `owner.name (kind)`, when the table states the entry and it resolves.
    pub bootstrap: Option<String>,
    /// How many static arguments that entry states.
    pub bootstrap_arguments: usize,
    /// The name the site presents — for a lambda site, the SAM method's name.
    pub sam_name: String,
    /// The descriptor the site presents: the captured values' types, then the functional interface.
    pub sam_descriptor: String,
    /// The SAM method type the factory states, when it states one.
    pub sam_method_type: Option<String>,
    /// The instantiated method type the factory states, when it states one.
    pub instantiated_method_type: Option<String>,
    /// The implementation handle as `owner.name(descriptor) (kind)`.
    pub implementation: Option<String>,
    /// Every value the site captures, in the order the instruction reads it off the stack.
    pub captures: Vec<LambdaCapture>,
    /// Which writing the site took; `None` when it was refused.
    pub form: Option<LambdaForm>,
    /// Why the site was not presented, when it was not.
    pub refusal: Option<LambdaRefusal>,
}

impl LambdaRecord {
    /// Whether this site was presented (as opposed to refused and left as bytecode).
    pub fn presented(&self) -> bool {
        self.form.is_some()
    }

    /// The rule this record is answerable to: the one that presented the site or refused it.
    pub fn rule(&self) -> RuleVersion {
        RULE
    }
}

/// One captured argument of a verified site: where the value it binds came from.
///
/// The text of the argument is answered by the artifact's segment table under this BCI
/// ([`crate::report::RecoveryReport::text_of_bci`]) — one mechanism for text, not two — while the
/// record states the *order* and the *number* of the bindings, which is what the SAM's descriptor
/// and the implementation's own signature have to agree on.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LambdaCapture {
    /// The BCI the captured value was produced at — the instruction whose text stands for it, or
    /// `None` when the value flowed in from a frame entry and no instruction produced it (its text
    /// is then written where the site is).
    pub bci: Option<u32>,
}

/// Why one dynamic site was refused.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LambdaRefusal {
    /// The diagnostic code, in the recovery layer's own vocabulary: one `jre_lambda_*` per link of
    /// the chain that can fail.
    pub code: &'static str,
    /// The rule that refused the site.
    pub rule: RuleVersion,
    /// The declared requirement that fell short, when the refusal is one of the rule's own
    /// preconditions rather than a shape that simply is not this one.
    pub requirement: Option<String>,
    /// One sentence stating which link failed, with the site's BCI in it.
    pub message: String,
}

impl LambdaRefusal {
    /// The refusal of one site, as the report records it.
    pub(crate) fn of(refusal: &Refusal, use_site: u32) -> Self {
        Self {
            code: refusal.code(),
            rule: RULE,
            requirement: refusal.requirement().map(Precondition::describe),
            message: format!(
                "the dynamic site at BCI {use_site} was not presented: {}",
                refusal.message()
            ),
        }
    }
}

/// Decides what one dynamic site is.
///
/// `captures` are the values the site reads, in the order it reads them: each one's BCI (as far as
/// the run states one) and the type the frames state for it — `None` when this run states no type.
/// The types are what the arity and shape checks align against the implementation handle, so a site
/// whose arguments are not the shape the handle needs is refused here rather than written, and a
/// site whose captures are not *readable from this run's facts* is refused as well.
pub(crate) fn plan(
    site: &DynamicSite,
    table: &[BootstrapMethodFacts],
    pool: &[CpEntryFacts],
    captures: &[(Option<u32>, Option<Type>)],
    profile: &RecoveryProfile,
) -> Verdict {
    let mut evidence = Evidence {
        bootstrap: None,
        bootstrap_arguments: 0,
        sam_method_type: None,
        instantiated_method_type: None,
        implementation: None,
    };
    if !LAMBDA.admits(profile) {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_rule_not_admitted",
                format!(
                    "the {RULE} rule presents an `invokedynamic` call site, which is Java {} and later, and this run's profile presents the artifact as Java {}",
                    LAMBDA.required_release().unwrap_or(8),
                    profile.java_release,
                ),
            )),
        };
    }
    let Some(entry) = table.get(usize::from(site.bootstrap_index())) else {
        return Verdict {
            evidence,
            outcome: Err(Refusal::unmet(
                &LAMBDA,
                Precondition::IrTable(IrTable::BootstrapMethods),
                format!(
                    "the site names bootstrap entry {}, and this class's bootstrap table states {} entry(s): no factory is named, so no lambda shape is claimed",
                    site.bootstrap_index(),
                    table.len(),
                ),
            )),
        };
    };
    let Some(handle) = handle_of(pool, entry.method_ref) else {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_bootstrap",
                format!(
                    "the bootstrap entry {} does not name a method handle this layer can resolve",
                    site.bootstrap_index(),
                ),
            )),
        };
    };
    evidence.bootstrap = Some(describe(&handle));
    evidence.bootstrap_arguments = entry.arguments.len();
    if handle.owner != FACTORY_OWNER || !FACTORY_METHODS.contains(&handle.name.as_str()) {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_bootstrap",
                format!(
                    "the bootstrap is {}, not a `{FACTORY_OWNER}.metafactory`/`altMetafactory` factory method: an arbitrary or unknown bootstrap never takes this shape (A04)",
                    describe(&handle),
                ),
            )),
        };
    }
    if handle.kind != REF_INVOKE_STATIC {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_bootstrap",
                format!(
                    "the factory `{}` is named by a {} handle, and the factory the contract states is a static one",
                    handle.name,
                    kind_word(handle.kind),
                ),
            )),
        };
    }
    // The static arguments: the SAM method type, the implementation handle and the instantiated
    // method type, plus — for `altMetafactory` — a flag word this layer requires to be zero.
    let (sam_index, implementation_index, instantiated_index) = match arguments_of(entry, pool) {
        Ok(indices) => indices,
        Err(message) => {
            return Verdict {
                evidence,
                outcome: Err(Refusal::shape("jre_lambda_bootstrap_arguments", message)),
            };
        }
    };
    let read_type = |index: u16| -> Option<String> {
        match cp_entry(pool, index).map(|entry| &entry.kind) {
            Ok(CpEntryKind::MethodType { descriptor, .. }) => {
                Some(String::from_utf8_lossy(&descriptor.0).into_owned())
            }
            _ => None,
        }
    };
    evidence.sam_method_type = read_type(sam_index);
    evidence.instantiated_method_type = read_type(instantiated_index);
    let Some(implementation) = handle_of(pool, implementation_index) else {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_implementation",
                format!(
                    "the factory's implementation argument (index {implementation_index}) does not name a method handle this layer can resolve"
                ),
            )),
        };
    };
    evidence.implementation = Some(format!(
        "{}.{}{} ({})",
        source_name(&implementation.owner),
        implementation.name,
        implementation.descriptor,
        kind_word(implementation.kind)
    ));
    if !implementation.is_invocation() {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_implementation",
                format!(
                    "the implementation is {}, and only an invocation or a constructor handle presents a lambda: a field handle is not a method this layer may call",
                    kind_word(implementation.kind)
                ),
            )),
        };
    }
    let (Some(sam), Some(instantiated)) = (
        evidence.sam_method_type.clone(),
        evidence.instantiated_method_type.clone(),
    ) else {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_sam_descriptor",
                "the factory's SAM argument or its instantiated-method-type argument is not a `MethodType`, so the SAM's own signature is not stated".to_string(),
            )),
        };
    };
    let Some((site_params, site_returns)) = parse_method(site.descriptor()) else {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_descriptor",
                format!(
                    "the site's descriptor `{}` is not one this layer reads",
                    site.descriptor()
                ),
            )),
        };
    };
    let (Some((sam_params, _)), Some((instantiated_params, _))) =
        (parse_method(&sam), parse_method(&instantiated))
    else {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_descriptor",
                format!(
                    "the factory's method types (`{sam}`, `{instantiated}`) are not descriptors this layer reads"
                ),
            )),
        };
    };
    if sam_params.len() != instantiated_params.len() {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_sam_descriptor",
                format!(
                    "the SAM method type `{sam}` takes {} parameter(s) and the instantiated method type `{instantiated}` takes {}: the SAM's own signature is not one signature",
                    sam_params.len(),
                    instantiated_params.len(),
                ),
            )),
        };
    }
    if site_params.len() != captures.len() {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_descriptor",
                format!(
                    "the site's descriptor `{}` names {} captured value(s) and the value flow states {}: the site is not read from half its facts",
                    site.descriptor(),
                    site_params.len(),
                    captures.len(),
                ),
            )),
        };
    }
    let Some(interface) = site_returns else {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_descriptor",
                format!(
                    "the site's descriptor `{}` returns no type, so no functional interface is named",
                    site.descriptor()
                ),
            )),
        };
    };
    debug_assert!(matches!(interface, Type::Reference(_)));
    let Some((implementation_params, _)) = parse_method(&implementation.descriptor) else {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_implementation",
                format!(
                    "the implementation's descriptor `{}` is not one this layer reads",
                    implementation.descriptor
                ),
            )),
        };
    };
    debug_assert_eq!(instantiated_params.len(), sam_params.len());
    // Every captured value's type has to be stated by this run: the alignment below compares shapes,
    // and a shape nobody stated cannot be compared. The names table is where that fact lives, so the
    // requirement is stated as that table's — and a site is refused, never guessed at.
    let Some(capture_types) = captures
        .iter()
        .map(|(_, ty)| ty.clone())
        .collect::<Option<Vec<Type>>>()
    else {
        let at = captures
            .iter()
            .find(|(_, ty)| ty.is_none())
            .and_then(|(at, _)| *at);
        return Verdict {
            evidence,
            outcome: Err(Refusal::unmet(
                &LAMBDA,
                Precondition::IrTable(IrTable::Ssa),
                format!(
                    "the value flow states no type for the value captured at {}, so the site's arguments are not read from this run's facts",
                    at.map_or_else(
                        || "an instruction this run does not state".to_string(),
                        |at| format!("BCI {at}")
                    )
                ),
            )),
        };
    };
    let receiver = usize::from(implementation.takes_receiver());
    // The operands the handle is *given*: the values the site captures, then the SAM's parameters.
    // They have to be exactly the receiver the handle takes plus the parameters it declares.
    let given = captures.len() + sam_params.len();
    if given != implementation_params.len() + receiver {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_sam_arity",
                format!(
                    "the implementation `{}.{}{}` takes {} parameter(s){} and the site would give it {given} ({} captured value(s) plus the SAM's {})",
                    source_name(&implementation.owner),
                    implementation.name,
                    implementation.descriptor,
                    implementation_params.len(),
                    if receiver == 1 {
                        " after its receiver"
                    } else {
                        ""
                    },
                    captures.len(),
                    sam_params.len(),
                ),
            )),
        };
    }
    // Each aligned position has to be the same shape. What is deliberately *not* compared is which
    // reference type a reference is: the JVM lets the implementation's parameters be adapted to the
    // instantiated type, and deciding that adaptation is a typing question this layer does not
    // answer — so two references line up, and a reference against a primitive never does.
    let mut bound: Vec<Type> = capture_types;
    bound.extend(instantiated_params.iter().cloned());
    let mut expected: Vec<Type> = Vec::with_capacity(given);
    if receiver == 1 {
        expected.push(Type::Reference(source_name(&implementation.owner)));
    }
    expected.extend(implementation_params.iter().cloned());
    if let Some((position, (bound, expected))) = bound
        .iter()
        .zip(expected.iter())
        .enumerate()
        .find(|(_, (bound, expected))| shape(bound) != shape(expected))
    {
        return Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_sam_types",
                format!(
                    "the implementation `{}.{}{}` takes `{}` at parameter {position} and the site binds `{}` there, which is not the same shape",
                    source_name(&implementation.owner),
                    implementation.name,
                    implementation.descriptor,
                    expected.spell(),
                    bound.spell(),
                ),
            )),
        };
    }
    // The parameters the lambda takes are the SAM's own, as the *instantiated* method type states
    // them: the erased form (`Object`) would throw away what the class really said, and the
    // implementation's parameter types are only required to be *adaptable* to these, not equal to
    // them. Their count was checked against the SAM method type above.
    let params: Vec<Type> = instantiated_params;
    let form = if receiver == captures.len() && !implementation.is_generated_body() {
        LambdaForm::MethodReference
    } else {
        LambdaForm::Lambda
    };
    Verdict {
        evidence,
        outcome: Ok(Plan {
            form,
            params,
            reach: implementation.reach(),
            implementation,
            captures: captures.len(),
        }),
    }
}

/// The method handle one pool index names, resolved to its member.
fn handle_of(pool: &[CpEntryFacts], index: u16) -> Option<Member> {
    let Ok(CpEntryKind::MethodHandle {
        reference_kind,
        reference_index,
    }) = cp_entry(pool, index).map(|entry| &entry.kind)
    else {
        return None;
    };
    match cp_entry(pool, *reference_index).map(|entry| &entry.kind) {
        Ok(CpEntryKind::MethodRef {
            owner,
            name,
            descriptor,
            ..
        })
        | Ok(CpEntryKind::InterfaceMethodRef {
            owner,
            name,
            descriptor,
            ..
        }) => Some(Member {
            kind: *reference_kind,
            owner: String::from_utf8_lossy(&owner.0).into_owned(),
            name: String::from_utf8_lossy(&name.0).into_owned(),
            descriptor: String::from_utf8_lossy(&descriptor.0).into_owned(),
        }),
        // A field handle names a field, and a handle whose reference does not resolve is not a
        // member this layer may state.
        _ => None,
    }
}

/// The three argument indexes the contract names, checked by constant kind and count.
fn arguments_of(
    entry: &BootstrapMethodFacts,
    pool: &[CpEntryFacts],
) -> Result<(u16, u16, u16), String> {
    let kinds: Vec<&'static str> = entry
        .arguments
        .iter()
        .map(
            |index| match cp_entry(pool, *index).map(|entry| &entry.kind) {
                Ok(CpEntryKind::MethodType { .. }) => "MethodType",
                Ok(CpEntryKind::MethodHandle { .. }) => "MethodHandle",
                Ok(CpEntryKind::Integer { .. }) => "Integer",
                Ok(CpEntryKind::String { .. }) => "String",
                Ok(CpEntryKind::Class { .. }) => "Class",
                Ok(CpEntryKind::Long { .. }) => "Long",
                Ok(CpEntryKind::Float { .. }) => "Float",
                Ok(CpEntryKind::Double { .. }) => "Double",
                Ok(CpEntryKind::Dynamic { .. }) => "Dynamic",
                _ => "unresolved",
            },
        )
        .collect();
    let named = |index: usize| -> String {
        format!(
            "{} ({})",
            entry.arguments.get(index).copied().unwrap_or(0),
            kinds.get(index).copied().unwrap_or("none")
        )
    };
    let wrong = |detail: String| {
        format!(
            "the factory's static arguments are [{}], and the contract states a SAM method type, an implementation handle and an instantiated method type: {detail}",
            kinds.join(", ")
        )
    };
    if kinds.len() < 3 {
        return Err(wrong(format!(
            "only {} argument(s) are stated",
            kinds.len()
        )));
    }
    if kinds[0] != "MethodType" || kinds[1] != "MethodHandle" || kinds[2] != "MethodType" {
        return Err(wrong(format!(
            "the first three are argument {}, argument {} and argument {}",
            named(0),
            named(1),
            named(2)
        )));
    }
    if entry.arguments.len() > 3 {
        // `altMetafactory` states one more argument: the flag word. Markers, bridges and
        // serializable sites would need the arrays behind those flags and a presentation this layer
        // does not have, so only the flagless form is one it presents.
        if entry.arguments.len() != 4 || kinds[3] != "Integer" {
            return Err(wrong(format!(
                "a fourth argument is stated as argument {}, and the only fourth argument the contract adds is the `altMetafactory` flag word",
                named(3)
            )));
        }
        match cp_entry(pool, entry.arguments[3]).map(|entry| &entry.kind) {
            Ok(CpEntryKind::Integer { value: 0 }) => {}
            Ok(CpEntryKind::Integer { value }) => {
                return Err(wrong(format!(
                    "`altMetafactory` states the flag word {value}: markers, bridges or a serializable site would change what the site means, and this layer presents only the flagless form"
                )));
            }
            _ => unreachable!("the fourth argument's kind was checked above"),
        }
    }
    Ok((entry.arguments[0], entry.arguments[1], entry.arguments[2]))
}

/// How one handle is stated in a record: `owner.name (kind)`.
fn describe(handle: &Member) -> String {
    format!(
        "{}.{} ({})",
        source_name(&handle.owner),
        handle.name,
        kind_word(handle.kind)
    )
}

/// The JVMS name of one method-handle reference kind.
fn kind_word(kind: u8) -> &'static str {
    match kind {
        1 => "REF_getField",
        2 => "REF_getStatic",
        3 => "REF_putField",
        4 => "REF_putStatic",
        5 => "REF_invokeVirtual",
        6 => "REF_invokeStatic",
        7 => "REF_invokeSpecial",
        8 => "REF_newInvokeSpecial",
        9 => "REF_invokeInterface",
        _ => "an unknown reference kind",
    }
}

/// The shape one type has on the operand stack, for the alignment check.
///
/// The int-shaped primitives are one shape: the bytecode holds a `boolean`, a `byte`, a `char`, a
/// `short` and an `int` in the same slot and verifies them alike, so a descriptor that says `Z` and
/// a frame that says `int` are not a disagreement. References are one shape whatever they name —
/// which reference is compared is a typing question this layer deliberately does not answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Shape {
    Int,
    Long,
    Float,
    Double,
    Reference,
}

/// The stack shape of one type.
fn shape(ty: &Type) -> Shape {
    match ty {
        Type::Boolean | Type::Byte | Type::Char | Type::Short | Type::Int => Shape::Int,
        Type::Long => Shape::Long,
        Type::Float => Shape::Float,
        Type::Double => Shape::Double,
        Type::Reference(_) => Shape::Reference,
    }
}

/// The Java source spelling of an internal name.
fn source_name(internal: &str) -> String {
    internal.replace('/', ".")
}

/// One descriptor's parameters and return type, as this layer reads them.
///
/// `None` for the return type is `V`. Only the `(…)…` form is read: a field descriptor reaching here
/// is one this layer does not state, and `None` says so instead of splitting it wrongly.
fn parse_method(descriptor: &str) -> Option<(Vec<Type>, Option<Type>)> {
    let bytes = descriptor.as_bytes();
    if bytes.first() != Some(&b'(') {
        return None;
    }
    let mut at = 1;
    let mut params = Vec::new();
    loop {
        match bytes.get(at)? {
            b')' => break,
            _ => {
                let (ty, next) = parse_type(bytes, at)?;
                params.push(ty);
                at = next;
            }
        }
    }
    at += 1;
    if bytes.get(at) == Some(&b'V') {
        return (at + 1 == bytes.len()).then_some((params, None));
    }
    let (returns, next) = parse_type(bytes, at)?;
    (next == bytes.len()).then_some((params, Some(returns)))
}

/// One type of a descriptor, and where the next one starts.
fn parse_type(bytes: &[u8], at: usize) -> Option<(Type, usize)> {
    match bytes.get(at)? {
        b'Z' => Some((Type::Boolean, at + 1)),
        b'B' => Some((Type::Byte, at + 1)),
        b'C' => Some((Type::Char, at + 1)),
        b'S' => Some((Type::Short, at + 1)),
        b'I' => Some((Type::Int, at + 1)),
        b'J' => Some((Type::Long, at + 1)),
        b'F' => Some((Type::Float, at + 1)),
        b'D' => Some((Type::Double, at + 1)),
        b'L' => {
            let end = bytes[at + 1..].iter().position(|byte| *byte == b';')? + at + 1;
            let name = std::str::from_utf8(&bytes[at + 1..end]).ok()?;
            Some((Type::Reference(source_name(name)), end + 1))
        }
        b'[' => {
            let mut dimensions = 0;
            let mut probe = at;
            while bytes.get(probe) == Some(&b'[') {
                dimensions += 1;
                probe += 1;
            }
            let (element, next) = parse_type(bytes, probe)?;
            Some((
                Type::Reference(format!("{}{}", element.spell(), "[]".repeat(dimensions))),
                next,
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_method_descriptor_is_read_into_parameters_and_a_return_type() {
        let (params, returns) =
            parse_method("(I)Ljava/lang/Runnable;").expect("a method descriptor");
        assert_eq!(params, vec![Type::Int]);
        assert_eq!(
            returns,
            Some(Type::Reference("java.lang.Runnable".to_string()))
        );

        let (params, returns) = parse_method("()V").expect("a void descriptor");
        assert!(params.is_empty());
        assert_eq!(returns, None, "`V` is no type");

        let (params, returns) =
            parse_method("([[Ljava/lang/String;JZ)Ljava/util/function/IntUnaryOperator;")
                .expect("a descriptor with an array, a long and a boolean");
        assert_eq!(
            params,
            vec![
                Type::Reference("java.lang.String[][]".to_string()),
                Type::Long,
                Type::Boolean,
            ]
        );
        assert_eq!(
            returns,
            Some(Type::Reference(
                "java.util.function.IntUnaryOperator".to_string()
            ))
        );

        let (params, returns) = parse_method("([I)[Ljava/lang/Object;").expect("arrays of both");
        assert_eq!(params, vec![Type::Reference("int[]".to_string())]);
        assert_eq!(
            returns,
            Some(Type::Reference("java.lang.Object[]".to_string()))
        );
    }

    #[test]
    fn a_descriptor_this_layer_does_not_read_is_refused_rather_than_split() {
        assert_eq!(
            parse_method("I"),
            None,
            "a field descriptor is not a method"
        );
        assert_eq!(parse_method("(I"), None, "an unterminated parameter list");
        assert_eq!(parse_method("(I)V)"), None, "trailing bytes");
        assert_eq!(parse_method("(Q)V"), None, "an unknown type code");
        assert_eq!(parse_method("(Ljava/lang/String)V"), None, "no terminator");
    }

    #[test]
    fn the_int_shaped_primitives_are_one_shape_and_a_reference_is_not_a_primitive() {
        assert_eq!(shape(&Type::Boolean), shape(&Type::Int));
        assert_eq!(shape(&Type::Byte), shape(&Type::Char));
        assert_eq!(shape(&Type::Short), Shape::Int);
        assert_ne!(shape(&Type::Int), shape(&Type::Long));
        assert_ne!(
            shape(&Type::Reference("java.lang.String".to_string())),
            shape(&Type::Int)
        );
        assert_eq!(
            shape(&Type::Reference("java.lang.String".to_string())),
            shape(&Type::Reference("java.lang.Object".to_string())),
            "which reference a reference is stays a typing question"
        );
    }

    #[test]
    fn the_generated_body_marker_is_the_only_thing_that_decides_the_lambda_writing() {
        let body = Member {
            kind: 6,
            owner: "Test".to_string(),
            name: "lambda$method$0".to_string(),
            descriptor: "(I)V".to_string(),
        };
        assert!(body.is_generated_body());
        let named = Member {
            name: "valueOf".to_string(),
            ..body
        };
        assert!(!named.is_generated_body());
    }
}
