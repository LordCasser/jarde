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
//!    SAM's own parameters — are exactly its receiver plus its parameters. Captures must match the
//!    frame, site and implementation types exactly; each SAM parameter then proves both descriptor
//!    edges independently. This is the check that keeps an unproved implementation/SAM pairing
//!    from being printed as a lambda that would not mean the same thing.
//!
//! The plan retains the erased SAM, instantiated and implementation parameter types separately.
//! It proves each edge from those descriptors: identity, an `Object` check/upcast, or boxing and
//! unboxing between one primitive and its unique wrapper. Parameters flow SAM → instantiated →
//! implementation; returns flow implementation → instantiated → SAM. Unknown reference relations
//! and other conversions are refused.
//!
//! # The two presentations, and what decides which is written
//!
//! Both writings mean the same function when the raw target can select the implementation the
//! bootstrap names; parameter adaptations decide whether a method reference can express it:
//!
//! * a **method reference** (`Qualifier::name`, or `Type::new`) when the site's captures are exactly
//!   what the handle's own receiver needs and nothing else — a bound receiver or nothing at all —
//!   and every SAM parameter reaches the implementation unchanged. Java's `::` has no form for
//!   bound *arguments* or explicit parameter adaptations, so the raw target must already select the
//!   handle the bootstrap names;
//! * a **lambda** (`(params) -> body`) when explicit parameter adaptations or captured leading
//!   arguments need to be written, with the body the handle's own invocation:
//!   `impl(captures…, params…)` for a static handle, `capture0.impl(captures[1..]…, params…)` for an
//!   instance one, `new Owner(captures…, params…)` for a constructor.
//!
//! When the handle is a method the compiler generated for this very site — named with the marker
//! both javac and ECJ use for lambda bodies, `lambda$…` — the lambda writing is chosen even where a
//! `::` reference would otherwise preserve the same call, because the handle is the site's *own
//! body* and not a member the source named. The marker decides between writings and **never** whether
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

use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{
    Base, BaseType, BootstrapMethodFacts, CpEntryFacts, CpEntryKind, DescriptorComponent,
    DescriptorCursor, DescriptorKind, MethodCodeFacts, cp_entry, descriptor_facts,
};
use jarde_reader::model::{ExecutionReport, JvmBytes};

use crate::ast::Type;
use crate::decode::Operations;
use crate::facts::{ACC_PRIVATE, ACC_STATIC, ACC_SYNTHETIC, ClassMembers, DynamicSite, Operation};
use crate::pass::{IrTable, LAMBDA, Precondition, RecoveryProfile, RuleVersion};
use crate::stop::{StopReason, charge, poll};

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
///
/// This is the *plan* side of the record: the walk keeps it in [`crate::build::LambdaSite`], and the
/// owning record is materialized from it after the artifact is committed.
#[derive(Clone, Debug, Eq, PartialEq)]
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

/// One same-run BSM reference to a candidate javac array-constructor helper.
///
/// This only justifies reading the named member body. [`prove_array_constructor`] remains the
/// authority that decides whether the body can be presented as an array constructor reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrayHelperCandidate {
    /// The `invokedynamic` instruction whose BSM names the helper.
    pub call_site: u32,
    /// The owner stated by the BSM's implementation handle.
    pub owner: JvmBytes,
    /// The exact helper name stated by that handle.
    pub name: JvmBytes,
    /// The exact helper descriptor stated by that handle.
    pub descriptor: JvmBytes,
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
    /// The descriptor-derived parameter and return adaptations consumed by the writer.
    pub(crate) adaptation: AdaptationPlan,
    /// The functional interface the invokedynamic descriptor returns.
    pub(crate) target_type: Type,
    /// The implementation member.
    pub(crate) implementation: Member,
    /// How that member is reached.
    pub(crate) reach: Reach,
    /// How many of the site's operands are the handle's leading arguments.
    pub(crate) captures: usize,
    /// Exact same-class proof for a synthetic one-dimensional array constructor helper, when the
    /// class member table states the helper's complete body.
    #[allow(
        dead_code,
        reason = "task 2.3 consumes the proved target during emission"
    )]
    pub(crate) array_constructor: Option<ArrayConstructorProof>,
}

/// The exact array result proved from a selected synthetic helper's complete `Code` attribute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArrayConstructorProof {
    pub(crate) array_type: Type,
}

/// The bounded conversions proved from one site's bootstrap descriptors.
///
/// Each SAM argument names the erased declaration, instantiated type and implementation argument,
/// plus both conversions in order. Returns record both conversions in the reverse direction. No
/// class hierarchy or generic signature is inferred.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdaptationPlan {
    pub(crate) parameters: Vec<ParameterAdaptation>,
    pub(crate) returns: ReturnAdaptation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParameterAdaptation {
    /// Parameter type declared by the erased SAM descriptor.
    pub(crate) sam: Type,
    /// Parameter type the instantiated method type states for the call.
    pub(crate) dynamic: Type,
    /// The implementation parameter (or receiver) reached by this SAM argument.
    pub(crate) implementation: Type,
    pub(crate) sam_to_dynamic: TypeConversion,
    pub(crate) dynamic_to_implementation: TypeConversion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TypeConversion {
    Identity,
    /// A runtime reference cast from `Object` to the stated reference/array type.
    CheckCast,
    /// A statically known reference upcast to `Object`.
    WidenToObject,
    /// Java's primitive-to-corresponding-wrapper conversion.
    BoxPrimitive,
    /// Java's corresponding-wrapper-to-primitive conversion.
    UnboxPrimitive,
    /// The implementation result is discarded by a void instantiated/SAM return.
    DropToVoid,
    /// Both ends of this edge are void.
    Void,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReturnAdaptation {
    /// Erased SAM return type; `None` means `void`.
    pub(crate) sam: Option<Type>,
    /// Instantiated return type, retained as the middle point of the return proof.
    pub(crate) dynamic: Option<Type>,
    /// The implementation return type; constructors produce their owner despite descriptor `V`.
    pub(crate) implementation: Option<Type>,
    /// Proven implementation → instantiated conversion.
    pub(crate) implementation_to_dynamic: TypeConversion,
    /// Proven instantiated → erased SAM conversion.
    pub(crate) dynamic_to_sam: TypeConversion,
}

/// Why one site was not presented.
///
/// The shape itself is shared with the other pattern rules of this layer ([`crate::refusal`]): what
/// this module contributes is which codes the `lambda@1` rule's refusals are reported under.
pub(crate) use crate::refusal::Refusal;

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
/// Capture types must agree exactly with the site and implementation descriptors. A site whose
/// captures are not *readable from this run's facts* is refused as well.
#[allow(
    clippy::too_many_arguments,
    reason = "the one-site decision consumes its class, bootstrap, member, value-flow, profile and budget facts"
)]
pub(crate) fn plan(
    site: &DynamicSite,
    table: &[BootstrapMethodFacts],
    pool: &[CpEntryFacts],
    members: Option<&ClassMembers>,
    captures: &[(Option<u32>, Option<Type>)],
    profile: &RecoveryProfile,
    budget: &mut Budget,
    at: u32,
) -> Result<Verdict, StopReason> {
    let mut evidence = Evidence {
        bootstrap: None,
        bootstrap_arguments: 0,
        sam_method_type: None,
        instantiated_method_type: None,
        implementation: None,
    };
    if !LAMBDA.admits(profile) {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_rule_not_admitted",
                format!(
                    "the {RULE} rule presents an `invokedynamic` call site, which is Java {} and later, and this run's profile presents the artifact as Java {}",
                    LAMBDA.required_release().unwrap_or(8),
                    profile.java_release,
                ),
            )),
        });
    }
    let Some(entry) = table.get(usize::from(site.bootstrap_index())) else {
        return Ok(Verdict {
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
        });
    };
    let Some(handle) = handle_of(pool, entry.method_ref) else {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_bootstrap",
                format!(
                    "the bootstrap entry {} does not name a method handle this layer can resolve",
                    site.bootstrap_index(),
                ),
            )),
        });
    };
    evidence.bootstrap = Some(describe(&handle));
    evidence.bootstrap_arguments = entry.arguments.len();
    if handle.owner != FACTORY_OWNER || !FACTORY_METHODS.contains(&handle.name.as_str()) {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_bootstrap",
                format!(
                    "the bootstrap is {}, not a `{FACTORY_OWNER}.metafactory`/`altMetafactory` factory method: an arbitrary or unknown bootstrap never takes this shape (A04)",
                    describe(&handle),
                ),
            )),
        });
    }
    if handle.kind != REF_INVOKE_STATIC {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_bootstrap",
                format!(
                    "the factory `{}` is named by a {} handle, and the factory the contract states is a static one",
                    handle.name,
                    kind_word(handle.kind),
                ),
            )),
        });
    }
    // The static arguments: the SAM method type, the implementation handle and the instantiated
    // method type, plus — for `altMetafactory` — a flag word this layer requires to be zero.
    let (sam_index, implementation_index, instantiated_index) = match arguments_of(entry, pool) {
        Ok(indices) => indices,
        Err(message) => {
            return Ok(Verdict {
                evidence,
                outcome: Err(Refusal::shape("jre_lambda_bootstrap_arguments", message)),
            });
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
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_implementation",
                format!(
                    "the factory's implementation argument (index {implementation_index}) does not name a method handle this layer can resolve"
                ),
            )),
        });
    };
    evidence.implementation = Some(format!(
        "{}.{}{} ({})",
        source_name(&implementation.owner),
        implementation.name,
        implementation.descriptor,
        kind_word(implementation.kind)
    ));
    if !implementation.is_invocation() {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_implementation",
                format!(
                    "the implementation is {}, and only an invocation or a constructor handle presents a lambda: a field handle is not a method this layer may call",
                    kind_word(implementation.kind)
                ),
            )),
        });
    }
    let (Some(sam), Some(instantiated)) = (
        evidence.sam_method_type.clone(),
        evidence.instantiated_method_type.clone(),
    ) else {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_sam_descriptor",
                "the factory's SAM argument or its instantiated-method-type argument is not a `MethodType`, so the SAM's own signature is not stated".to_string(),
            )),
        });
    };
    let Some((site_params, site_returns)) = parse_method_input(site.descriptor(), budget, at)?
    else {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_descriptor",
                format!(
                    "the site's descriptor `{}` is not one this layer reads",
                    site.descriptor()
                ),
            )),
        });
    };
    let sam_signature = parse_method_input(&sam, budget, at)?;
    let instantiated_signature = parse_method_input(&instantiated, budget, at)?;
    let (Some((sam_params, sam_return)), Some((instantiated_params, instantiated_return))) =
        (sam_signature, instantiated_signature)
    else {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_descriptor",
                format!(
                    "the factory's method types (`{sam}`, `{instantiated}`) are not descriptors this layer reads"
                ),
            )),
        });
    };
    if sam_params.len() != instantiated_params.len() {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_sam_descriptor",
                format!(
                    "the SAM method type `{sam}` takes {} parameter(s) and the instantiated method type `{instantiated}` takes {}: the SAM's own signature is not one signature",
                    sam_params.len(),
                    instantiated_params.len(),
                ),
            )),
        });
    }
    if site_params.len() != captures.len() {
        return Ok(Verdict {
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
        });
    }
    let Some(interface) = site_returns else {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_descriptor",
                format!(
                    "the site's descriptor `{}` returns no type, so no functional interface is named",
                    site.descriptor()
                ),
            )),
        });
    };
    debug_assert!(matches!(interface, Type::Reference(_)));
    let Some((implementation_params, implementation_return)) =
        parse_method_input(&implementation.descriptor, budget, at)?
    else {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_implementation",
                format!(
                    "the implementation's descriptor `{}` is not one this layer reads",
                    implementation.descriptor
                ),
            )),
        });
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
        return Ok(Verdict {
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
        });
    };
    let receiver = usize::from(implementation.takes_receiver());
    // The operands the handle is *given*: the values the site captures, then the SAM's parameters.
    // They have to be exactly the receiver the handle takes plus the parameters it declares.
    let given = captures.len() + sam_params.len();
    if !implementation_arity_matches(given, implementation_params.len(), receiver) {
        return Ok(Verdict {
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
        });
    }
    // A capture runs while the functional value is created, before any SAM argument. The site
    // descriptor and implementation operand must state the same exact type, and the current frame
    // must state that same type too. Shape equality alone would silently infer a conversion at the
    // capture boundary (or move it into a lambda body), neither of which this plan proves.
    let mut expected: Vec<Type> = Vec::with_capacity(given);
    if receiver == 1 {
        expected.push(Type::Reference(source_name(&implementation.owner)));
    }
    expected.extend(implementation_params.iter().cloned());
    for position in 0..captures.len() {
        let frame_type = &capture_types[position];
        let site_type = &site_params[position];
        let implementation_type = &expected[position];
        if frame_type != site_type || site_type != implementation_type {
            return Ok(Verdict {
                evidence,
                outcome: Err(Refusal::shape(
                    "jre_lambda_sam_types",
                    format!(
                        "captured operand {position} is `{}` in the frame, `{}` in the site descriptor and `{}` in the implementation: the capture conversion and its creation-time effects are not proven",
                        frame_type.spell(),
                        site_type.spell(),
                        implementation_type.spell(),
                    ),
                )),
            });
        }
    }
    let adaptation = match adaptation_plan(
        &sam_params,
        &instantiated_params,
        (sam_return, instantiated_return),
        &implementation_params,
        implementation_return,
        (&implementation, captures.len()),
        || charge(budget, CountedBudgetDimension::AnalysisSteps, 1, Some(at)),
    ) {
        Ok(adaptation) => adaptation,
        Err(AdaptationFailure::Stop(stop)) => return Err(stop),
        Err(AdaptationFailure::Refusal) => {
            return Ok(Verdict {
                evidence,
                outcome: Err(Refusal::shape(
                    "jre_lambda_sam_types",
                    format!(
                        "the erased SAM `{sam}`, instantiated type `{instantiated}` and implementation `{}.{}{}` require a parameter or return conversion outside identity, Object check/upcast, or the matching primitive-wrapper pair",
                        source_name(&implementation.owner),
                        implementation.name,
                        implementation.descriptor,
                    ),
                )),
            });
        }
    };
    let array_constructor = if captures.is_empty() && sam_params.len() == 1 {
        prove_array_constructor(&implementation, pool, members, budget, at)?
    } else {
        None
    };
    // A method reference has no expression slot where an erased SAM argument can receive its
    // conversions. Keep it only when both stages are identity. A bound receiver also carries a
    // creation-time null check; converting that reference to a lambda would defer the check until
    // invocation, so refuse that adaptation shape.
    let parameter_adaptation = adaptation.parameters.iter().any(|parameter| {
        parameter.sam_to_dynamic != TypeConversion::Identity
            || parameter.dynamic_to_implementation != TypeConversion::Identity
    });
    let reference_shape = receiver == captures.len() && !implementation.is_generated_body();
    if reference_shape
        && parameter_adaptation
        && implementation.reach() == Reach::Receiver
        && captures.len() == 1
    {
        return Ok(Verdict {
            evidence,
            outcome: Err(Refusal::shape(
                "jre_lambda_sam_types",
                "adapting this bound receiver would move its null failure from functional-value creation to invocation".to_string(),
            )),
        });
    }
    // A proved compiler helper is the bytecode implementation of a source-level array
    // constructor reference. Its parameter adaptation (for example Integer -> int) is expressed
    // by the functional target type, just as it is for `int[]::new` in Java source.
    let form = if array_constructor.is_some() || (reference_shape && !parameter_adaptation) {
        LambdaForm::MethodReference
    } else {
        LambdaForm::Lambda
    };
    Ok(Verdict {
        evidence,
        outcome: Ok(Plan {
            form,
            adaptation,
            target_type: interface,
            reach: implementation.reach(),
            implementation,
            captures: captures.len(),
            array_constructor,
        }),
    })
}

/// Proves the narrow compiler helper used to implement an array-constructor method reference.
///
/// The name and flags only select a candidate. The proof comes from the exact member's descriptor
/// and complete, handler-free instruction sequence in this class's own member table.
fn prove_array_constructor(
    implementation: &Member,
    pool: &[CpEntryFacts],
    members: Option<&ClassMembers>,
    budget: &mut Budget,
    at: u32,
) -> Result<Option<ArrayConstructorProof>, StopReason> {
    if implementation.kind != REF_INVOKE_STATIC || !implementation.is_generated_body() {
        return Ok(None);
    }
    let Some(members) = members else {
        return Ok(None);
    };
    if members.owner() != implementation.owner {
        return Ok(None);
    }

    let mut selected = None;
    for member in members.members() {
        poll(budget, Some(at))?;
        charge(budget, CountedBudgetDimension::AnalysisSteps, 1, Some(at))?;
        if member.owner() == members.owner()
            && member.name() == implementation.name
            && member.descriptor() == implementation.descriptor
        {
            if selected.is_some() {
                return Ok(None);
            }
            selected = Some(member);
        }
    }
    let Some(member) = selected else {
        return Ok(None);
    };
    let flags = member.access_flags();
    if flags & (ACC_PRIVATE | ACC_STATIC | ACC_SYNTHETIC)
        != (ACC_PRIVATE | ACC_STATIC | ACC_SYNTHETIC)
    {
        return Ok(None);
    }
    let Some(code) = member.code() else {
        return Ok(None);
    };
    prove_array_constructor_code(&implementation.descriptor, code, pool, budget, at)
}

fn prove_array_constructor_code(
    descriptor: &str,
    code: &MethodCodeFacts,
    pool: &[CpEntryFacts],
    budget: &mut Budget,
    at: u32,
) -> Result<Option<ArrayConstructorProof>, StopReason> {
    poll(budget, Some(at))?;
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(descriptor.len()).unwrap_or(u64::MAX),
        Some(at),
    )?;
    let Ok(method) = descriptor_facts(descriptor.as_bytes(), DescriptorKind::Method) else {
        return Ok(None);
    };
    let [length] = method.parameters() else {
        return Ok(None);
    };
    if length.dimensions() != 0 || !matches!(length.base(), Base::Primitive(BaseType::Int)) {
        return Ok(None);
    }
    let Some(result) = method.result() else {
        return Ok(None);
    };
    if result.dimensions() != 1 {
        return Ok(None);
    }
    let Some(array_type) = type_of_component(result) else {
        return Ok(None);
    };
    if !matches!(&code.execution, ExecutionReport::Complete { .. })
        || code.stopped_at.is_some()
        || !code.exception_handlers.is_empty()
        || code.exception_handler_count != 0
        || code.max_locals < 1
        || code.max_stack < 1
        || code.instructions.len() != 3
        || code.operands().len() != 3
    {
        return Ok(None);
    }

    let mut cursor = 0u64;
    for fact in &code.instructions {
        poll(budget, Some(at))?;
        charge(budget, CountedBudgetDimension::AnalysisSteps, 1, Some(at))?;
        if u64::from(fact.bci) != cursor {
            return Ok(None);
        }
        cursor = cursor.saturating_add(u64::from(fact.width));
    }
    if cursor != code.code_span.length {
        return Ok(None);
    }

    let [_, _, ret] = code.instructions.as_slice() else {
        return Ok(None);
    };
    let [load_operands, allocation_operands, return_operands] = code.operands() else {
        return Ok(None);
    };
    if !matches!(load_operands.effective_opcode, 0x15 | 0x1a)
        || load_operands.local.map(|local| local.index) != Some(0)
        || allocation_operands.effective_opcode != 0xbc
            && allocation_operands.effective_opcode != 0xbd
        || ret.opcode != 0xb0
    {
        return Ok(None);
    }
    // No other typed operand may smuggle in an effect or a second value source.
    if load_operands.constant_pool_index.is_some()
        || load_operands.immediate.is_some()
        || allocation_operands.local.is_some()
        || allocation_operands.immediate.is_some()
        || return_operands.local.is_some()
        || return_operands.constant_pool_index.is_some()
        || return_operands.immediate.is_some()
        || return_operands.effective_opcode != 0xb0
    {
        return Ok(None);
    }
    if !array_allocation_matches(result, allocation_operands, pool) {
        return Ok(None);
    }
    Ok(Some(ArrayConstructorProof { array_type }))
}

fn array_allocation_matches(
    result: &DescriptorComponent,
    allocation: &jarde_reader::classfile::InstructionOperands,
    pool: &[CpEntryFacts],
) -> bool {
    match (result.base(), allocation.effective_opcode) {
        (Base::Primitive(base), 0xbc) => {
            let expected = match base {
                BaseType::Boolean => 4,
                BaseType::Char => 5,
                BaseType::Float => 6,
                BaseType::Double => 7,
                BaseType::Byte => 8,
                BaseType::Short => 9,
                BaseType::Int => 10,
                BaseType::Long => 11,
            };
            result.dimensions() == 1
                && allocation.atype == Some(expected)
                && allocation.constant_pool_index.is_none()
        }
        (Base::Object(object), 0xbd) => {
            let Some(index) = allocation.constant_pool_index else {
                return false;
            };
            let Ok(CpEntryKind::Class { name, .. }) =
                cp_entry(pool, index).map(|entry| &entry.kind)
            else {
                return false;
            };
            result.dimensions() == 1 && name == object && allocation.atype.is_none()
        }
        _ => false,
    }
}

type ParsedMethodDescriptor = (Vec<Type>, Option<Type>);

fn parse_method_input(
    descriptor: &str,
    budget: &mut Budget,
    at: u32,
) -> Result<Option<ParsedMethodDescriptor>, StopReason> {
    poll(budget, Some(at))?;
    let bytes = u64::try_from(descriptor.len()).unwrap_or(u64::MAX);
    if bytes != 0 {
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            bytes,
            Some(at),
        )?;
    }
    Ok(parse_method(descriptor))
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

/// Finds the narrow same-run BSM references whose member Code could prove an array constructor.
///
/// The returned handles are only read candidates. They are derived from the exact
/// `invokedynamic` → LambdaMetafactory bootstrap → implementation MethodHandle chain in this IR,
/// restricted to a same-class static `lambda$` helper with `(int) array` descriptor shape. The
/// member table and its complete Code still have to pass [`prove_array_constructor`].
pub fn array_helper_candidates(ir: &jarde_jvm::method_ir::MethodIr) -> Vec<ArrayHelperCandidate> {
    let (Some(code), Some(declaration)) = (ir.code(), ir.declaration()) else {
        return Vec::new();
    };
    let pool = ir.constant_pool();
    let owner = declaration.class_name();
    let operations = Operations::of(code, pool);
    let mut candidates = Vec::new();
    for instruction in &code.instructions {
        let Some(Operation::InvokeDynamic(site)) = operations.get(instruction.bci) else {
            continue;
        };
        let Some(entry) = ir
            .bootstrap_methods()
            .get(usize::from(site.bootstrap_index()))
        else {
            continue;
        };
        let Some((factory_kind, factory_owner, factory_name, _)) =
            method_handle_member(pool, entry.method_ref)
        else {
            continue;
        };
        if factory_kind != REF_INVOKE_STATIC
            || factory_owner.0.as_slice() != FACTORY_OWNER.as_bytes()
            || !FACTORY_METHODS
                .iter()
                .any(|name| factory_name.0.as_slice() == name.as_bytes())
        {
            continue;
        }
        let Ok((_, implementation_index, _)) = arguments_of(entry, pool) else {
            continue;
        };
        let Some((implementation_kind, implementation_owner, implementation_name, descriptor)) =
            method_handle_member(pool, implementation_index)
        else {
            continue;
        };
        if implementation_kind != REF_INVOKE_STATIC
            || implementation_owner != owner
            || !implementation_name.0.starts_with(BODY_MARKER.as_bytes())
        {
            continue;
        }
        let Ok(descriptor_text) = std::str::from_utf8(&descriptor.0) else {
            continue;
        };
        let Some((parameters, Some(Type::Reference(array_type)))) = parse_method(descriptor_text)
        else {
            continue;
        };
        if parameters.as_slice() != [Type::Int] || !array_type.ends_with("[]") {
            continue;
        }
        candidates.push(ArrayHelperCandidate {
            call_site: instruction.bci,
            owner: implementation_owner.clone(),
            name: implementation_name.clone(),
            descriptor: descriptor.clone(),
        });
    }
    candidates
}

/// Resolves only the raw member bytes and kind of one method-handle pool entry.
fn method_handle_member(
    pool: &[CpEntryFacts],
    index: u16,
) -> Option<(u8, &JvmBytes, &JvmBytes, &JvmBytes)> {
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
        }) => Some((*reference_kind, owner, name, descriptor)),
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

/// Prove only conversions whose direction and type follow from the descriptors themselves.
///
/// The `implementation` vector includes the receiver as its first operand when the handle takes
/// one. Site captures occupy the prefix before SAM arguments, so each SAM parameter maps to the
/// implementation operand at `captures + index`.
#[derive(Debug)]
enum AdaptationFailure {
    Refusal,
    Stop(StopReason),
}

fn adaptation_plan(
    sam_parameters: &[Type],
    dynamic_parameters: &[Type],
    returns: (Option<Type>, Option<Type>),
    implementation_parameters: &[Type],
    implementation_return: Option<Type>,
    implementation_and_captures: (&Member, usize),
    mut bill_parameter: impl FnMut() -> Result<(), StopReason>,
) -> Result<AdaptationPlan, AdaptationFailure> {
    let (sam_return, dynamic_return) = returns;
    let (implementation, captures) = implementation_and_captures;
    if sam_parameters.len() != dynamic_parameters.len() {
        return Err(AdaptationFailure::Refusal);
    }
    let takes_receiver = implementation.takes_receiver();
    let operand_count = usize::from(takes_receiver) + implementation_parameters.len();
    let mut parameters = Vec::new();
    for (index, (sam, dynamic)) in sam_parameters.iter().zip(dynamic_parameters).enumerate() {
        bill_parameter().map_err(AdaptationFailure::Stop)?;
        let operand = captures + index;
        let implementation_type = if takes_receiver && operand == 0 {
            Type::Reference(source_name(&implementation.owner))
        } else {
            let parameter_index = operand
                .checked_sub(usize::from(takes_receiver))
                .ok_or(AdaptationFailure::Refusal)?;
            implementation_parameters
                .get(parameter_index)
                .ok_or(AdaptationFailure::Refusal)?
                .clone()
        };
        if operand >= operand_count {
            return Err(AdaptationFailure::Refusal);
        }

        let sam_to_dynamic = type_conversion(sam, dynamic).ok_or(AdaptationFailure::Refusal)?;
        let dynamic_to_implementation =
            type_conversion(dynamic, &implementation_type).ok_or(AdaptationFailure::Refusal)?;
        parameters.push(ParameterAdaptation {
            sam: sam.clone(),
            dynamic: dynamic.clone(),
            implementation: implementation_type,
            sam_to_dynamic,
            dynamic_to_implementation,
        });
    }

    let implementation_return = if implementation.kind == REF_NEW_INVOKE_SPECIAL {
        Some(Type::Reference(source_name(&implementation.owner)))
    } else {
        implementation_return
    };
    let implementation_to_dynamic = match (&implementation_return, &dynamic_return) {
        (None, None) => TypeConversion::Void,
        (Some(_), None) if sam_return.is_none() => TypeConversion::DropToVoid,
        (Some(implementation), Some(dynamic)) => {
            type_conversion(implementation, dynamic).ok_or(AdaptationFailure::Refusal)?
        }
        _ => return Err(AdaptationFailure::Refusal),
    };
    let dynamic_to_sam = match (&dynamic_return, &sam_return) {
        (None, None) => TypeConversion::Void,
        (Some(_), None) => TypeConversion::DropToVoid,
        (Some(dynamic), Some(sam)) => {
            type_conversion(dynamic, sam).ok_or(AdaptationFailure::Refusal)?
        }
        _ => return Err(AdaptationFailure::Refusal),
    };

    Ok(AdaptationPlan {
        parameters,
        returns: ReturnAdaptation {
            sam: sam_return,
            dynamic: dynamic_return,
            implementation: implementation_return,
            implementation_to_dynamic,
            dynamic_to_sam,
        },
    })
}

/// Prove one descriptor-to-descriptor edge without resolving arbitrary class inheritance.
fn type_conversion(from: &Type, to: &Type) -> Option<TypeConversion> {
    if from == to {
        Some(TypeConversion::Identity)
    } else if is_object(from) && is_reference(to) {
        Some(TypeConversion::CheckCast)
    } else if is_reference(from) && is_object(to) {
        Some(TypeConversion::WidenToObject)
    } else if primitive_wrapper(from, to) {
        Some(TypeConversion::BoxPrimitive)
    } else if primitive_wrapper(to, from) {
        Some(TypeConversion::UnboxPrimitive)
    } else {
        None
    }
}

fn implementation_arity_matches(given: usize, parameters: usize, receiver: usize) -> bool {
    parameters.checked_add(receiver) == Some(given)
}

/// Whether the arguments are one primitive and its unique Java wrapper, in either direction.
fn primitive_wrapper(primitive: &Type, wrapper: &Type) -> bool {
    let (Type::Reference(wrapper), Some(name)) = (wrapper, primitive_wrapper_name(primitive))
    else {
        return false;
    };
    wrapper == name
}

fn primitive_wrapper_name(ty: &Type) -> Option<&'static str> {
    Some(match ty {
        Type::Boolean => "java.lang.Boolean",
        Type::Byte => "java.lang.Byte",
        Type::Char => "java.lang.Character",
        Type::Short => "java.lang.Short",
        Type::Int => "java.lang.Integer",
        Type::Long => "java.lang.Long",
        Type::Float => "java.lang.Float",
        Type::Double => "java.lang.Double",
        Type::Reference(_) => return None,
    })
}

fn is_reference(ty: &Type) -> bool {
    matches!(ty, Type::Reference(_))
}

fn is_object(ty: &Type) -> bool {
    matches!(ty, Type::Reference(name) if name == "java.lang.Object")
}

/// The Java source spelling of an internal name.
fn source_name(internal: &str) -> String {
    internal.replace('/', ".")
}

/// The Java type one descriptor component names (JVMS 4.3.2), or `None` when it names no type this
/// layer can write.
///
/// This is the repository's one descriptor → Java type spelling. The production itself belongs to
/// the reader ([`DescriptorComponent`]: base type, object name, array dimensions), so nothing here
/// reads bytes; what this function owns is the *type the text writes* — a primitive's own name, an
/// array spelled from its element outwards with one `[]` per dimension, and an object type's
/// internal name spelled with [`source_name`]. A name that is not UTF-8 states no Java type, so it
/// is refused here instead of being written lossily into a type position.
pub fn type_of_component(component: &DescriptorComponent) -> Option<Type> {
    let base = type_of_base(component.base())?;
    if component.dimensions() == 0 {
        return Some(base);
    }
    Some(Type::Reference(format!(
        "{}{}",
        base.spell(),
        "[]".repeat(usize::try_from(component.dimensions()).ok()?)
    )))
}

/// The Java type one descriptor component's **base** names, with none of its dimensions: what an
/// array's element is, and what a component with no `[` is whole.
///
/// Published to the crate because two readings need the base on its own: [`type_of_component`],
/// which appends the `[]`s the component states, and `crate::decode`'s array creation, where the
/// element type is the component of the array class the instruction's pool entry names
/// (`multianewarray`'s `[[I` is a creation of `int`s). A class type with no name is no type at all
/// (JVMS 4.2: a binary name is not empty), and the reader refuses one before it becomes a
/// component; the same rule is stated here, where the type is written.
pub(crate) fn type_of_base(base: &Base) -> Option<Type> {
    Some(match base {
        Base::Primitive(BaseType::Boolean) => Type::Boolean,
        Base::Primitive(BaseType::Byte) => Type::Byte,
        Base::Primitive(BaseType::Char) => Type::Char,
        Base::Primitive(BaseType::Short) => Type::Short,
        Base::Primitive(BaseType::Int) => Type::Int,
        Base::Primitive(BaseType::Long) => Type::Long,
        Base::Primitive(BaseType::Float) => Type::Float,
        Base::Primitive(BaseType::Double) => Type::Double,
        Base::Object(name) => {
            let name = std::str::from_utf8(&name.0).ok()?;
            if name.is_empty() {
                return None;
            }
            Type::Reference(source_name(name))
        }
    })
}

/// One descriptor's parameters and return type, as this layer reads them.
///
/// `None` for the return type is `V`. Only the `(…)…` form is read: a field descriptor reaching here
/// is one this layer does not state, and `None` says so instead of splitting it wrongly.
///
/// Shared with the pattern rules of P3 2.2, which read the same descriptors for their own purposes
/// (the `append` overloads a concatenation calls, the erased signature a bridge forwards, the
/// declaration of an accessor): one reading of the production, and the reader's own — this function
/// only turns its facts into the types this layer writes.
pub(crate) fn parse_method(descriptor: &str) -> Option<(Vec<Type>, Option<Type>)> {
    let facts = descriptor_facts(descriptor.as_bytes(), DescriptorKind::Method).ok()?;
    let parameters = facts
        .parameters()
        .iter()
        .map(type_of_component)
        .collect::<Option<Vec<Type>>>()?;
    let returns = match facts.result() {
        Some(component) => Some(type_of_component(component)?),
        None => None,
    };
    Some((parameters, returns))
}

/// One type of a descriptor, and where the next one starts.
///
/// Published to the crate because this is the repository's one descriptor→Java type spelling:
/// [`crate::build::spell_reference`] reads the frames' own array descriptors through it, so a
/// declaration and a lambda parameter cannot spell `[[Ljava/lang/String;` two different ways. The
/// bytes are the reader's reading of the production ([`DescriptorCursor`]) and the spelling is
/// [`type_of_component`]'s, so both entries above read one descriptor the one way.
pub(crate) fn parse_type(bytes: &[u8], at: usize) -> Option<(Type, usize)> {
    let mut cursor = DescriptorCursor::at_offset(bytes, at);
    let component = cursor.field_type().ok()?;
    Some((type_of_component(&component)?, cursor.position()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{ClassMembers, MemberBody};
    use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension, Limits};
    use jarde_reader::classfile::{
        CpEntryFacts, CpEntryKind, InstructionFact, InstructionOperands, LocalOperand,
        MethodCodeFacts,
    };
    use jarde_reader::model::{
        ByteSpan, ClassBytesId, Digest, ExecutionReport, JvmBytes, PhysicalClassLocation,
        PhysicalDefinitionId, PhysicalMethodId, PhysicalVariant, SnapshotId,
    };

    fn adaptation_plan_for_test(
        sam_parameters: &[Type],
        dynamic_parameters: &[Type],
        returns: (Option<Type>, Option<Type>),
        implementation_parameters: &[Type],
        implementation_return: Option<Type>,
        implementation: &Member,
        captures: usize,
    ) -> Result<AdaptationPlan, AdaptationFailure> {
        super::adaptation_plan(
            sam_parameters,
            dynamic_parameters,
            returns,
            implementation_parameters,
            implementation_return,
            (implementation, captures),
            || Ok(()),
        )
    }

    fn cp(index: u16, kind: CpEntryKind) -> CpEntryFacts {
        CpEntryFacts {
            index,
            span: ByteSpan::new(0, 0),
            kind,
        }
    }

    fn unlimited_budget() -> Budget {
        Budget::new(Limits {
            analysis_steps: u64::MAX,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        })
    }

    fn array_helper_code(atype: u8) -> MethodCodeFacts {
        let load = InstructionOperands {
            effective_opcode: 0x15,
            local: Some(LocalOperand {
                index: 0,
                wide: false,
            }),
            ..InstructionOperands::default()
        };
        let allocation = InstructionOperands {
            effective_opcode: 0xbc,
            atype: Some(atype),
            ..InstructionOperands::default()
        };
        let ret = InstructionOperands {
            effective_opcode: 0xb0,
            ..InstructionOperands::default()
        };
        let fact = |bci, opcode, width| InstructionFact {
            bci,
            opcode,
            width,
            span: ByteSpan::new(u64::from(bci), u64::from(width)),
            operands_span: ByteSpan::new(u64::from(bci), u64::from(width)),
            constant_pool_index: None,
        };
        MethodCodeFacts::from_parts(
            1,
            1,
            ByteSpan::new(0, 4),
            vec![
                (fact(0, 0x1a, 1), load),
                (fact(1, 0xbc, 2), allocation),
                (fact(3, 0xb0, 1), ret),
            ],
            vec![],
            0,
            ExecutionReport::Complete {
                usage: Default::default(),
            },
            None,
            jarde_reader::classfile::LocalDebugTable::Absent,
        )
    }

    fn class_members_for_helper(code: Option<MethodCodeFacts>, flags: u16) -> ClassMembers {
        class_members_for("lambda$arrayCtor$0", "(I)[I", code, flags)
    }

    fn class_members_for(
        name: &str,
        descriptor: &str,
        code: Option<MethodCodeFacts>,
        flags: u16,
    ) -> ClassMembers {
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: SnapshotId("test-snapshot".to_string()),
            },
            class_bytes: ClassBytesId {
                digest: Digest("test-digest".to_string()),
                length: 0,
            },
            variant: PhysicalVariant::Base,
        };
        let identity = PhysicalMethodId {
            owner: definition,
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        };
        let member = match code {
            Some(code) => MemberBody::new("test/Target", identity, flags, code),
            None => MemberBody::without_body("test/Target", identity, flags),
        };
        ClassMembers::new("test/Target", vec![member])
    }

    #[test]
    fn array_constructor_proof_requires_exact_member_and_complete_effect_free_code() {
        let implementation = Member {
            kind: REF_INVOKE_STATIC,
            owner: "test/Target".to_string(),
            name: "lambda$arrayCtor$0".to_string(),
            descriptor: "(I)[I".to_string(),
        };
        let flags = ACC_PRIVATE | ACC_STATIC | ACC_SYNTHETIC;
        let pool = [];
        let mut budget = unlimited_budget();
        let members = class_members_for_helper(Some(array_helper_code(10)), flags);
        let proof =
            prove_array_constructor(&implementation, &pool, Some(&members), &mut budget, 19)
                .expect("exact member facts stay within budget")
                .expect("the helper has the exact int-array allocation sequence");
        assert_eq!(proof.array_type, Type::Reference("int[]".to_string()));

        let mut bounded = Budget::new(Limits {
            analysis_steps: 0,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        assert!(matches!(
            prove_array_constructor(&implementation, &pool, Some(&members), &mut bounded, 19),
            Err(StopReason::Budget {
                dimension: CountedBudgetDimension::AnalysisSteps,
                written: 0,
                limit: 0,
                at: Some(19),
            })
        ));
        assert_eq!(bounded.usage().analysis_steps, 0);

        let planned = plan_for_named_test(
            "()Ljava/util/function/Function;",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            REF_INVOKE_STATIC,
            "(I)[I",
            "(Ljava/lang/Integer;)[I",
            &[],
            "lambda$arrayCtor$0",
            Some(&members),
        );
        let plan = planned.outcome.expect("the SAM adaptation remains proved");
        assert_eq!(plan.array_constructor, Some(proof.clone()));
        assert_eq!(
            plan.form,
            LambdaForm::MethodReference,
            "a capture-free unary SAM may retain the verified array constructor reference"
        );

        // Method-only requests retain the generic lambda plan but cannot claim an array target.
        let method_only = plan_for_named_test(
            "()Ljava/util/function/Function;",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            REF_INVOKE_STATIC,
            "(I)[I",
            "(Ljava/lang/Integer;)[I",
            &[],
            "lambda$arrayCtor$0",
            None,
        );
        let method_only = method_only
            .outcome
            .expect("descriptor adaptation is independent");
        assert!(method_only.array_constructor.is_none());

        let mut budget = unlimited_budget();
        assert!(
            prove_array_constructor(&implementation, &pool, None, &mut budget, 19,)
                .expect("missing members is a conservative non-proof")
                .is_none()
        );

        let mut budget = unlimited_budget();
        let name_only = class_members_for_helper(None, flags);
        assert!(
            prove_array_constructor(&implementation, &pool, Some(&name_only), &mut budget, 19,)
                .expect("a missing body is not a parse failure")
                .is_none()
        );

        let mut budget = unlimited_budget();
        let wrong_flags =
            class_members_for_helper(Some(array_helper_code(10)), ACC_PRIVATE | ACC_STATIC);
        assert!(
            prove_array_constructor(&implementation, &pool, Some(&wrong_flags), &mut budget, 19,)
                .expect("a non-synthetic member cannot be claimed as the helper")
                .is_none()
        );

        let mut budget = unlimited_budget();
        let wrong_type = class_members_for_helper(Some(array_helper_code(8)), flags);
        assert!(
            prove_array_constructor(&implementation, &pool, Some(&wrong_type), &mut budget, 19,)
                .expect("mismatched primitive atype is a failed proof")
                .is_none()
        );

        let mut extra_effect = array_helper_code(10);
        let mut instructions = extra_effect
            .instructions
            .iter()
            .cloned()
            .zip(extra_effect.operands().iter().cloned())
            .collect::<Vec<_>>();
        instructions.push((
            InstructionFact {
                bci: 4,
                opcode: 0x00,
                width: 1,
                span: ByteSpan::new(4, 1),
                operands_span: ByteSpan::new(4, 1),
                constant_pool_index: None,
            },
            InstructionOperands {
                effective_opcode: 0x00,
                ..InstructionOperands::default()
            },
        ));
        extra_effect = MethodCodeFacts::from_parts(
            1,
            1,
            ByteSpan::new(0, 5),
            instructions,
            vec![],
            0,
            ExecutionReport::Complete {
                usage: Default::default(),
            },
            None,
            jarde_reader::classfile::LocalDebugTable::Absent,
        );
        let mut budget = unlimited_budget();
        let extra_effect = class_members_for_helper(Some(extra_effect), flags);
        assert!(
            prove_array_constructor(&implementation, &pool, Some(&extra_effect), &mut budget, 19,)
                .expect("an extra opcode is a failed proof")
                .is_none()
        );

        let mut budget = unlimited_budget();
        let wrong_parameter = class_members_for(
            "lambda$arrayCtor$0",
            "(J)[I",
            Some(array_helper_code(10)),
            flags,
        );
        assert!(
            prove_array_constructor(
                &implementation,
                &pool,
                Some(&wrong_parameter),
                &mut budget,
                19
            )
            .expect("wrong helper parameter is a non-proof")
            .is_none()
        );

        let mut budget = unlimited_budget();
        let mut handler_code = array_helper_code(10);
        handler_code = MethodCodeFacts::from_parts(
            handler_code.max_stack,
            handler_code.max_locals,
            handler_code.code_span.clone(),
            handler_code
                .instructions
                .iter()
                .cloned()
                .zip(handler_code.operands().iter().cloned())
                .collect(),
            vec![jarde_reader::classfile::ExceptionHandlerFact {
                ordinal: 0,
                start_bci: 0,
                end_bci: 1,
                handler_bci: 3,
                catch_type_index: None,
            }],
            1,
            handler_code.execution.clone(),
            None,
            jarde_reader::classfile::LocalDebugTable::Absent,
        );
        let handler_members = class_members_for_helper(Some(handler_code), flags);
        assert!(
            prove_array_constructor(
                &implementation,
                &pool,
                Some(&handler_members),
                &mut budget,
                19
            )
            .expect("handlers are a non-proof, never inferred over")
            .is_none()
        );
    }

    #[test]
    fn captured_array_length_does_not_receive_an_array_constructor_proof() {
        let flags = ACC_PRIVATE | ACC_STATIC | ACC_SYNTHETIC;
        let members = class_members_for_helper(Some(array_helper_code(10)), flags);
        let captures = [(Some(2), Some(Type::Int))];
        let planned = plan_for_named_test(
            "(I)Ljava/util/function/Supplier;",
            "()Ljava/lang/Object;",
            REF_INVOKE_STATIC,
            "(I)[I",
            "()[I",
            &captures,
            "lambda$arrayCtor$0",
            Some(&members),
        );
        let plan = planned
            .outcome
            .expect("the captured helper remains a valid zero-argument Supplier lambda");
        assert_eq!(plan.captures, 1);
        assert_eq!(plan.form, LambdaForm::Lambda);
        assert!(
            plan.array_constructor.is_none(),
            "a captured length cannot be represented by a no-capture `int[]::new` reference"
        );
    }

    fn method_ref(owner: &str, name: &str, descriptor: &str) -> CpEntryKind {
        CpEntryKind::MethodRef {
            class_index: 0,
            name_and_type_index: 0,
            owner: JvmBytes(owner.as_bytes().to_vec()),
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        }
    }

    fn plan_for_test(
        site_descriptor: &str,
        sam_descriptor: &str,
        implementation_kind: u8,
        implementation_descriptor: &str,
        instantiated_descriptor: &str,
        captures: &[(Option<u32>, Option<Type>)],
    ) -> Verdict {
        plan_for_named_test(
            site_descriptor,
            sam_descriptor,
            implementation_kind,
            implementation_descriptor,
            instantiated_descriptor,
            captures,
            "apply",
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn plan_for_named_test(
        site_descriptor: &str,
        sam_descriptor: &str,
        implementation_kind: u8,
        implementation_descriptor: &str,
        instantiated_descriptor: &str,
        captures: &[(Option<u32>, Option<Type>)],
        implementation_name: &str,
        members: Option<&ClassMembers>,
    ) -> Verdict {
        let pool = vec![
            cp(
                1,
                CpEntryKind::MethodHandle {
                    reference_kind: REF_INVOKE_STATIC,
                    reference_index: 2,
                },
            ),
            cp(2, method_ref(FACTORY_OWNER, "metafactory", "()V")),
            cp(
                3,
                CpEntryKind::MethodType {
                    descriptor_index: 0,
                    descriptor: JvmBytes(sam_descriptor.as_bytes().to_vec()),
                },
            ),
            cp(
                4,
                CpEntryKind::MethodHandle {
                    reference_kind: implementation_kind,
                    reference_index: 5,
                },
            ),
            cp(
                5,
                method_ref(
                    "test/Target",
                    implementation_name,
                    implementation_descriptor,
                ),
            ),
            cp(
                6,
                CpEntryKind::MethodType {
                    descriptor_index: 0,
                    descriptor: JvmBytes(instantiated_descriptor.as_bytes().to_vec()),
                },
            ),
        ];
        let bootstraps = [BootstrapMethodFacts {
            method_ref: 1,
            arguments: vec![3, 4, 6],
        }];
        let site = DynamicSite::new(7, 0, "apply", site_descriptor);
        let mut budget = Budget::new(Limits {
            analysis_steps: u64::MAX,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        plan(
            &site,
            &bootstraps,
            &pool,
            members,
            captures,
            &crate::pass::JAVA_8,
            &mut budget,
            7,
        )
        .expect("default test budget is ample for one lambda site")
    }

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
        assert_eq!(parse_method("(L;)V"), None, "a class type with no name");
    }

    #[test]
    fn the_declaration_spelling_reuses_this_parser_and_the_two_agree() {
        // `spell_reference` reads the frames' array descriptors through `parse_type`, so the two
        // spellings of "descriptor → Java type" are one implementation: what this parser reads is
        // what a declaration is written with, for every shape an array can take.
        for descriptor in [
            "[B",
            "[C",
            "[D",
            "[F",
            "[I",
            "[J",
            "[S",
            "[Z",
            "[Ljava/lang/String;",
            "[[I",
            "[[Ljava/lang/String;",
            "[[[B",
            "[[[[Ljava/lang/Object;",
        ] {
            let (ty, end) =
                parse_type(descriptor.as_bytes(), 0).unwrap_or_else(|| panic!("{descriptor}"));
            assert_eq!(end, descriptor.len(), "`{descriptor}` is read whole");
            assert_eq!(
                crate::build::spell_reference(descriptor).as_deref(),
                Some(ty.spell()),
                "`{descriptor}` is spelled the same way by both entries"
            );
        }
        // The forms this parser does not read are the forms the declaration spelling refuses: an
        // array with no element type, a `void` element, an unterminated class element, and a class
        // type with no name.
        for descriptor in ["[", "[V", "[Lfoo", "[L;", "[[", "[[L;"] {
            assert_eq!(
                crate::build::spell_reference(descriptor),
                None,
                "`{descriptor}` states no Java type"
            );
        }
    }

    fn method_types(descriptor: &str) -> (Vec<Type>, Option<Type>) {
        parse_method(descriptor).unwrap_or_else(|| panic!("invalid test descriptor: {descriptor}"))
    }

    fn static_member(descriptor: &str) -> Member {
        Member {
            kind: REF_INVOKE_STATIC,
            owner: "test/Target".to_string(),
            name: "accept".to_string(),
            descriptor: descriptor.to_string(),
        }
    }

    #[test]
    fn lambda_adapter_parameter_work_stops_through_the_shared_analysis_budget() {
        let site_descriptor = "(I)Ljava/lang/Runnable;";
        let sam_descriptor = "(I)V";
        let instantiated_descriptor = "(I)V";
        let implementation_descriptor = "(II)V";
        let descriptor_work = [
            site_descriptor,
            sam_descriptor,
            instantiated_descriptor,
            implementation_descriptor,
        ]
        .into_iter()
        .map(|descriptor| descriptor.len() as u64)
        .sum();
        let pool = vec![
            cp(
                1,
                CpEntryKind::MethodHandle {
                    reference_kind: REF_INVOKE_STATIC,
                    reference_index: 2,
                },
            ),
            cp(2, method_ref(FACTORY_OWNER, "metafactory", "()V")),
            cp(
                3,
                CpEntryKind::MethodType {
                    descriptor_index: 0,
                    descriptor: JvmBytes(sam_descriptor.as_bytes().to_vec()),
                },
            ),
            cp(
                4,
                CpEntryKind::MethodHandle {
                    reference_kind: REF_INVOKE_STATIC,
                    reference_index: 5,
                },
            ),
            cp(
                5,
                method_ref("test/Target", "apply", implementation_descriptor),
            ),
            cp(
                6,
                CpEntryKind::MethodType {
                    descriptor_index: 0,
                    descriptor: JvmBytes(instantiated_descriptor.as_bytes().to_vec()),
                },
            ),
        ];
        let bootstraps = [BootstrapMethodFacts {
            method_ref: 1,
            arguments: vec![3, 4, 6],
        }];
        let site = DynamicSite::new(7, 0, "apply", site_descriptor);
        let captures = [(Some(9), Some(Type::Int))];
        let mut budget = Budget::new(Limits {
            analysis_steps: descriptor_work,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });

        let stop = match plan(
            &site,
            &bootstraps,
            &pool,
            None,
            &captures,
            &crate::pass::JAVA_8,
            &mut budget,
            42,
        ) {
            Err(stop) => stop,
            Ok(_) => panic!(
                "the first adapter parameter should exceed the exact descriptor-work allowance"
            ),
        };

        assert_eq!(
            stop,
            StopReason::Budget {
                dimension: CountedBudgetDimension::AnalysisSteps,
                written: 0,
                limit: descriptor_work,
                at: Some(42),
            }
        );
        assert_eq!(budget.usage().analysis_steps, descriptor_work);

        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(Limits::default(), token);
        let cancelled_stop = match plan(
            &site,
            &bootstraps,
            &pool,
            None,
            &captures,
            &crate::pass::JAVA_8,
            &mut cancelled,
            42,
        ) {
            Err(stop) => stop,
            Ok(_) => panic!("a cancelled adapter plan cannot return a refusal or produced plan"),
        };
        assert_eq!(cancelled_stop, StopReason::Cancelled { at: Some(42) });
        assert_eq!(cancelled.usage().analysis_steps, 0);
    }

    #[test]
    fn adaptation_plan_preserves_object_to_reference_and_array_checks() {
        let (sam_params, sam_return) = method_types("(Ljava/lang/Object;)V");
        let (dynamic_params, dynamic_return) = method_types("(Ljava/lang/String;)V");
        let (implementation_params, implementation_return) = method_types("(Ljava/lang/String;)V");
        let plan = adaptation_plan_for_test(
            &sam_params,
            &dynamic_params,
            (sam_return.clone(), dynamic_return.clone()),
            &implementation_params,
            implementation_return,
            &static_member("(Ljava/lang/String;)V"),
            0,
        )
        .expect("Object to String is a dynamic check, then an exact implementation argument");
        assert_eq!(
            plan.parameters[0].sam,
            Type::Reference("java.lang.Object".to_string())
        );
        assert_eq!(
            plan.parameters[0].dynamic,
            Type::Reference("java.lang.String".to_string())
        );
        assert_eq!(plan.parameters[0].sam_to_dynamic, TypeConversion::CheckCast);
        assert_eq!(
            plan.parameters[0].dynamic_to_implementation,
            TypeConversion::Identity
        );

        let (dynamic_params, dynamic_return) = method_types("([Ljava/lang/String;)V");
        let (implementation_params, implementation_return) = method_types("([Ljava/lang/String;)V");
        let array_plan = adaptation_plan_for_test(
            &sam_params,
            &dynamic_params,
            (sam_return.clone(), dynamic_return),
            &implementation_params,
            implementation_return,
            &static_member("([Ljava/lang/String;)V"),
            0,
        )
        .expect("Object to String[] is a dynamic reference check");
        assert_eq!(
            array_plan.parameters[0].dynamic,
            Type::Reference("java.lang.String[]".to_string())
        );
        assert_eq!(
            array_plan.parameters[0].sam_to_dynamic,
            TypeConversion::CheckCast
        );
    }

    #[test]
    fn adaptation_plan_proves_boxed_sam_parameter_and_return_edges() {
        let (sam_params, sam_return) = method_types("()Ljava/lang/Object;");
        let (dynamic_params, dynamic_return) = method_types("()Ljava/lang/Integer;");
        let (implementation_params, implementation_return) = method_types("()I");
        let supplier = adaptation_plan_for_test(
            &sam_params,
            &dynamic_params,
            (sam_return, dynamic_return),
            &implementation_params,
            implementation_return,
            &static_member("()I"),
            0,
        )
        .expect("primitive implementation result boxes to instantiated Integer, then Object");
        assert_eq!(
            supplier.returns.implementation_to_dynamic,
            TypeConversion::BoxPrimitive
        );
        assert_eq!(
            supplier.returns.dynamic_to_sam,
            TypeConversion::WidenToObject
        );

        let (sam_params, sam_return) = method_types("(Ljava/lang/Object;)Ljava/lang/Object;");
        let (dynamic_params, dynamic_return) = method_types("(Ljava/lang/Integer;)[I");
        let (implementation_params, implementation_return) = method_types("(I)[I");
        let array_function = adaptation_plan_for_test(
            &sam_params,
            &dynamic_params,
            (sam_return, dynamic_return),
            &implementation_params,
            implementation_return,
            &static_member("(I)[I"),
            0,
        )
        .expect("Object checks to Integer, Integer unboxes to int");
        assert_eq!(
            array_function.parameters[0].sam_to_dynamic,
            TypeConversion::CheckCast
        );
        assert_eq!(
            array_function.parameters[0].dynamic_to_implementation,
            TypeConversion::UnboxPrimitive
        );
        assert_eq!(
            array_function.returns.implementation_to_dynamic,
            TypeConversion::Identity
        );
        assert_eq!(
            array_function.returns.dynamic_to_sam,
            TypeConversion::WidenToObject
        );

        let (sam_params, sam_return) = method_types("(Ljava/lang/Object;)Ljava/lang/Object;");
        let (dynamic_params, dynamic_return) =
            method_types("(Ljava/lang/String;)Ljava/lang/Integer;");
        let (implementation_params, implementation_return) = method_types("(Ljava/lang/String;)I");
        let function = adaptation_plan_for_test(
            &sam_params,
            &dynamic_params,
            (sam_return, dynamic_return),
            &implementation_params,
            implementation_return,
            &static_member("(Ljava/lang/String;)I"),
            0,
        )
        .expect("erased Function argument checks to String and primitive result boxes");
        assert_eq!(
            function.parameters[0].sam_to_dynamic,
            TypeConversion::CheckCast
        );
        assert_eq!(
            function.parameters[0].dynamic_to_implementation,
            TypeConversion::Identity
        );
        assert_eq!(
            function.returns.implementation_to_dynamic,
            TypeConversion::BoxPrimitive
        );
        assert_eq!(
            function.returns.dynamic_to_sam,
            TypeConversion::WidenToObject
        );
    }

    #[test]
    fn plan_accepts_boxed_array_length_but_keeps_instance_handle_arity_and_captures_exact() {
        let verdict = plan_for_test(
            "()Ljava/util/function/Function;",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            REF_INVOKE_STATIC,
            "(I)[I",
            "(Ljava/lang/Integer;)[I",
            &[],
        );
        let plan = match verdict.outcome {
            Ok(plan) => plan,
            Err(refusal) => panic!("boxed array length adaptation refused: {refusal:?}"),
        };
        assert_eq!(plan.form, LambdaForm::Lambda);
        assert_eq!(
            plan.adaptation.parameters[0].sam_to_dynamic,
            TypeConversion::CheckCast
        );
        assert_eq!(
            plan.adaptation.parameters[0].dynamic_to_implementation,
            TypeConversion::UnboxPrimitive
        );

        let receiver_capture = [(Some(2), Some(Type::Reference("test.Target".to_string())))];
        let captured_receiver = plan_for_test(
            "(Ltest/Target;)Ljava/util/function/Supplier;",
            "()Ljava/lang/Object;",
            5,
            "()I",
            "()Ljava/lang/Integer;",
            &receiver_capture,
        );
        let captured_receiver = match captured_receiver.outcome {
            Ok(plan) => plan,
            Err(refusal) => panic!("exact bound receiver remains accepted: {refusal:?}"),
        };
        assert_eq!(captured_receiver.captures, 1);
        assert_eq!(captured_receiver.form, LambdaForm::MethodReference);

        let mismatch = plan_for_test(
            "()Ljava/util/function/Function;",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            5,
            "(I)[I",
            "(Ljava/lang/Integer;)[I",
            &[],
        );
        let mismatch_refusal = match mismatch.outcome {
            Err(refusal) => refusal,
            Ok(_) => panic!("receiver handle needs an extra operand"),
        };
        assert_eq!(mismatch_refusal.code(), "jre_lambda_sam_arity");

        let captures = [(
            Some(3),
            Some(Type::Reference("java.lang.Integer".to_string())),
        )];
        let capture_mismatch = plan_for_test(
            "(Ljava/lang/String;)Ljava/util/function/Function;",
            "()Ljava/lang/Object;",
            REF_INVOKE_STATIC,
            "(Ljava/lang/String;)Ljava/lang/String;",
            "()Ljava/lang/String;",
            &captures,
        );
        let capture_refusal = match capture_mismatch.outcome {
            Err(refusal) => refusal,
            Ok(_) => panic!("capture frame/site/impl types must be exact"),
        };
        assert_eq!(capture_refusal.code(), "jre_lambda_sam_types");
    }

    #[test]
    fn boxed_adaptation_pairs_are_exact_and_arity_keeps_receiver_staticness() {
        for (primitive, wrapper) in [
            (Type::Boolean, "java.lang.Boolean"),
            (Type::Byte, "java.lang.Byte"),
            (Type::Char, "java.lang.Character"),
            (Type::Short, "java.lang.Short"),
            (Type::Int, "java.lang.Integer"),
            (Type::Long, "java.lang.Long"),
            (Type::Float, "java.lang.Float"),
            (Type::Double, "java.lang.Double"),
        ] {
            let wrapper_type = Type::Reference(wrapper.to_string());
            assert_eq!(
                type_conversion(&primitive, &wrapper_type),
                Some(TypeConversion::BoxPrimitive)
            );
            assert_eq!(
                type_conversion(&wrapper_type, &primitive),
                Some(TypeConversion::UnboxPrimitive)
            );
        }

        assert_eq!(
            type_conversion(&Type::Int, &Type::Reference("java.lang.Long".to_string())),
            None,
            "only the unique matching wrapper is accepted"
        );
        assert_eq!(
            type_conversion(&Type::Reference("java.lang.String".to_string()), &Type::Int),
            None,
            "an unrelated reference cannot unbox to a primitive"
        );
        assert!(
            adaptation_plan_for_test(
                &[Type::Reference("java.lang.Object".to_string())],
                &[Type::Reference("java.lang.Long".to_string())],
                (None, None),
                &[Type::Int],
                None,
                &static_member("(I)V"),
                0,
            )
            .is_err(),
            "Integer-to-int cannot be inferred through Long"
        );
        assert!(
            adaptation_plan_for_test(
                &[Type::Reference("java.lang.Object".to_string())],
                &[],
                (None, None),
                &[Type::Int],
                None,
                &static_member("(I)V"),
                0,
            )
            .is_err(),
            "different SAM and instantiated arities are refused"
        );
        assert!(
            adaptation_plan_for_test(
                &[Type::Reference("java.lang.String".to_string())],
                &[Type::Reference("java.lang.String".to_string())],
                (None, None),
                &[Type::Int],
                None,
                &static_member("(I)V"),
                0,
            )
            .is_err(),
            "String cannot be adapted to int"
        );
        assert!(!implementation_arity_matches(1, 0, 0));
        assert!(implementation_arity_matches(1, 0, 1));
        assert!(!implementation_arity_matches(0, 0, 1));
        assert!(implementation_arity_matches(0, 0, 0));
    }

    #[test]
    fn adaptation_plan_preserves_identity_and_proven_object_conversions() {
        let (sam_params, sam_return) = method_types("(Ljava/lang/String;)Ljava/lang/Object;");
        let (dynamic_params, dynamic_return) =
            method_types("(Ljava/lang/String;)Ljava/lang/String;");
        let (implementation_params, implementation_return) =
            method_types("(Ljava/lang/String;)Ljava/lang/String;");
        let identity = adaptation_plan_for_test(
            &sam_params,
            &dynamic_params,
            (sam_return.clone(), dynamic_return.clone()),
            &implementation_params,
            implementation_return,
            &static_member("(Ljava/lang/String;)Ljava/lang/String;"),
            0,
        )
        .expect("same parameter type and String-to-Object return upcast");
        assert_eq!(
            identity.parameters[0].sam_to_dynamic,
            TypeConversion::Identity
        );
        assert_eq!(
            identity.parameters[0].dynamic_to_implementation,
            TypeConversion::Identity
        );
        assert_eq!(
            identity.returns.implementation_to_dynamic,
            TypeConversion::Identity
        );
        assert_eq!(
            identity.returns.dynamic_to_sam,
            TypeConversion::WidenToObject
        );
        assert_eq!(identity.returns.dynamic, dynamic_return);

        let (sam_params, sam_return) = method_types("(Ljava/lang/Object;)V");
        let (dynamic_params, dynamic_return) = method_types("(Ljava/lang/String;)V");
        let (implementation_params, implementation_return) = method_types("(Ljava/lang/Object;)V");
        let upcast = adaptation_plan_for_test(
            &sam_params,
            &dynamic_params,
            (sam_return, dynamic_return),
            &implementation_params,
            implementation_return,
            &static_member("(Ljava/lang/Object;)V"),
            0,
        )
        .expect("String dynamically checked, then widened to implementation Object");
        assert_eq!(
            upcast.parameters[0].sam_to_dynamic,
            TypeConversion::CheckCast
        );
        assert_eq!(
            upcast.parameters[0].dynamic_to_implementation,
            TypeConversion::WidenToObject
        );
    }

    #[test]
    fn adaptation_plan_rejects_unknown_reference_relations_and_unsupported_returns() {
        let (sam_params, sam_return) = method_types("(Ljava/lang/Number;)V");
        let (dynamic_params, dynamic_return) = method_types("(Ljava/lang/Integer;)V");
        let (implementation_params, implementation_return) = method_types("(Ljava/lang/Integer;)V");
        assert!(
            adaptation_plan_for_test(
                &sam_params,
                &dynamic_params,
                (sam_return, dynamic_return),
                &implementation_params,
                implementation_return,
                &static_member("(Ljava/lang/Integer;)V"),
                0,
            )
            .is_err(),
            "this layer has no Number/Integer inheritance proof"
        );

        let (sam_params, sam_return) = method_types("()Ljava/lang/String;");
        let (dynamic_params, dynamic_return) = method_types("()Ljava/lang/Integer;");
        let (implementation_params, implementation_return) = method_types("()Ljava/lang/Number;");
        assert!(
            adaptation_plan_for_test(
                &sam_params,
                &dynamic_params,
                (sam_return, dynamic_return),
                &implementation_params,
                implementation_return,
                &static_member("()Ljava/lang/Number;"),
                0,
            )
            .is_err(),
            "this layer does not infer arbitrary Number-to-Integer or Integer-to-String relations"
        );
    }

    #[test]
    fn instantiated_return_is_retained_and_both_edges_are_proved() {
        let (sam_params, sam_return) = method_types("()Ljava/lang/Object;");
        let (dynamic_params, dynamic_return) = method_types("()Ljava/lang/String;");
        let (implementation_params, implementation_return) = method_types("()Ljava/lang/Object;");
        let plan = adaptation_plan_for_test(
            &sam_params,
            &dynamic_params,
            (sam_return, dynamic_return),
            &implementation_params,
            implementation_return,
            &static_member("()Ljava/lang/Object;"),
            0,
        )
        .expect("implementation and erased SAM both return Object");
        assert_eq!(
            plan.returns.sam,
            Some(Type::Reference("java.lang.Object".to_string()))
        );
        assert_eq!(
            plan.returns.dynamic,
            Some(Type::Reference("java.lang.String".to_string()))
        );
        assert_eq!(
            plan.returns.implementation,
            Some(Type::Reference("java.lang.Object".to_string()))
        );
        assert_eq!(
            plan.returns.implementation_to_dynamic,
            TypeConversion::CheckCast
        );
        assert_eq!(plan.returns.dynamic_to_sam, TypeConversion::WidenToObject);
    }

    #[test]
    fn implementation_result_can_be_discarded_only_for_a_void_sam() {
        let (sam_params, sam_return) = method_types("()V");
        let (dynamic_params, dynamic_return) = method_types("()V");
        let (implementation_params, implementation_return) = method_types("()I");
        let plan = adaptation_plan_for_test(
            &sam_params,
            &dynamic_params,
            (sam_return, dynamic_return),
            &implementation_params,
            implementation_return,
            &static_member("()I"),
            0,
        )
        .expect("a void SAM may discard an implementation result");
        assert_eq!(plan.returns.sam, None);
        assert_eq!(plan.returns.dynamic, None);
        assert_eq!(plan.returns.implementation, Some(Type::Int));
        assert_eq!(
            plan.returns.implementation_to_dynamic,
            TypeConversion::DropToVoid
        );
        assert_eq!(plan.returns.dynamic_to_sam, TypeConversion::Void);

        let (sam_params, sam_return) = method_types("()I");
        let (dynamic_params, dynamic_return) = method_types("()I");
        let (implementation_params, implementation_return) = method_types("()V");
        assert!(
            adaptation_plan_for_test(
                &sam_params,
                &dynamic_params,
                (sam_return, dynamic_return),
                &implementation_params,
                implementation_return,
                &static_member("()V"),
                0,
            )
            .is_err(),
            "a void implementation cannot supply a non-void SAM result"
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
