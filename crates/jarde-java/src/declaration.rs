//! What the class file declares the presented member to be (P3 2.3, rule `declaration@1`).
//!
//! # Why this needs a caller-stated fact, and which one
//!
//! The artifact of this layer is one method **body**: an envelope comment naming the member, then its
//! statements. The member's own declaration — is it a `default` method of an interface? a static
//! method of an interface? a constructor? — is therefore written into that envelope, and the two
//! facts it needs arrive here as [`crate::facts::MethodFacts`], which the entry point fills from the
//! run's **own** read rather than from a second look at the class: the member's own flags since P3
//! 3.1, and the class that declares it — its `this_class` and its own access flags — since the
//! declaring-class handoff carried those two into the payload's declaration. This rule is written
//! against the fact type and not against the payload, because a library caller may still be the one
//! that read the class header; the facts are optional here, and a caller (or a run) that states
//! neither refuses rather than guessing. P3 2.2 set the same precedent for a member's flags, which
//! the bridge rule reads the same way.
//!
//! The **member's** own flags were the first of the two to stop being the caller's problem: P3 3.1
//! put the driver method's declaration — its `access_flags`, descriptor, parameter slots and
//! identity — into the payload, read in the same header pass that read the body, and the entry point
//! fills [`crate::facts::MethodFacts`] from there. So the flags this rule reads reach it from the run
//! rather than from a second reading, and the field stays on the facts type because the library's
//! callers may still be the ones that read it.
//!
//! So this rule reads exactly two declaration facts, both optional:
//!
//! * the member's own `access_flags` ([`crate::facts::MethodFacts::access_flags`]) — stated by the
//!   caller, and by the entry point from the run's own declaration;
//! * the class that declares it, with that class's own flags ([`crate::facts::DeclaringClass`]) —
//!   stated by the caller, and by the entry point from the two facts the declaring-class handoff
//!   carried: the header read's `this_class` and the class's `access_flags`. It stays optional here
//!   for the same reason as the flags above — a caller that states no class, or a run that published
//!   no member declaration and therefore has no class facts to hand over, still gets the refusal
//!   below instead of a guess.
//!
//! # What each combination means, and what a refusal means
//!
//! A `default` method is not a flag of its own: JVMS 4.6 gives an interface's methods `public`,
//! `static`, `abstract` and the rest, and the `default` keyword is exactly "an interface's method
//! that is neither `static` nor `abstract`". That is why the class's own `ACC_INTERFACE` is needed
//! and why this rule refuses when either fact is missing instead of guessing one from the other: a
//! `public` method that is not abstract is a default method **in an interface** and an ordinary
//! method **in a class**, and the two spell differently. A refusal is recorded with the requirement
//! it fell short of and the run writes no declaration line — it does not write a weaker one.

use serde::Serialize;

use crate::evidence::Publication;
use crate::facts::{ACC_ABSTRACT, ACC_STATIC, DeclaringClass, MethodFacts};
use crate::pass::{DECLARATION, Precondition, RuleVersion};
use crate::refusal::{Gap, Refusal};

// Source-unit declarations consume selected facts; they never read artifacts or commit reports.
use crate::facts::{ACC_ANNOTATION, ACC_INTERFACE, ACC_PRIVATE, ACC_PUBLIC};
use crate::names::SourceClassNames;
use crate::{LocalVariable, NameTable, RecoveryFacts, SlotEvidence, type_of_component};
use jarde_reader::classfile::{AttributeShell, DescriptorKind, MemberHeader, descriptor_facts};
use jarde_reader::model::{JvmBytes, JvmString};
const ACC_PROTECTED: u16 = 0x0004;
const ACC_FINAL: u16 = 0x0010;
const ACC_NATIVE: u16 = 0x0100;
const ACC_SYNCHRONIZED: u16 = 0x0020;
const ACC_STRICT: u16 = 0x0800;
const ACC_VARARGS: u16 = 0x0080;

/// The pass answerable for every verdict of this module.
pub(crate) const RULE: RuleVersion = DECLARATION.rule();

/// The member's own declaration facts.
const MEMBER_FLAGS: Precondition = Precondition::Metadata {
    attribute: "access_flags",
};

/// The class that declares the member.
const DECLARING_CLASS: Precondition = Precondition::Metadata {
    attribute: "declaring_class",
};

/// The form the class file declares one member in.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeclarationForm {
    /// An instance initializer: the `<init>` of JVMS 2.9.
    Constructor,
    /// The class initializer: `<clinit>`.
    StaticInitializer,
    /// An interface's method that is neither `static` nor `abstract`: the `default` keyword.
    DefaultMethod,
    /// An interface's `static` method (Java 8 and later).
    StaticInterfaceMethod,
    /// An interface's `abstract` method: the declaration the implementor has to write.
    AbstractInterfaceMethod,
    /// A class's `static` method.
    StaticMethod,
    /// A class's instance method.
    InstanceMethod,
}

impl DeclarationForm {
    /// The phrase the artifact's envelope states this form with.
    pub(crate) fn spell(self) -> &'static str {
        match self {
            Self::Constructor => "a constructor",
            Self::StaticInitializer => "a static initializer",
            Self::DefaultMethod => "an interface's default method",
            Self::StaticInterfaceMethod => "an interface's static method",
            Self::AbstractInterfaceMethod => "an interface's abstract method",
            Self::StaticMethod => "a static method",
            Self::InstanceMethod => "an instance method",
        }
    }

    /// The same phrase, under the name a report reads the form back by.
    pub fn name(self) -> &'static str {
        match self {
            Self::Constructor => "constructor",
            Self::StaticInitializer => "static_initializer",
            Self::DefaultMethod => "default_method",
            Self::StaticInterfaceMethod => "static_interface_method",
            Self::AbstractInterfaceMethod => "abstract_interface_method",
            Self::StaticMethod => "static_method",
            Self::InstanceMethod => "instance_method",
        }
    }
}

/// The declaration one run could read, the decision behind it — from which the owning record is
/// materialized after the artifact is committed — and the refusal as the report states it in every
/// selection.
pub(crate) struct Plan {
    declaration: Option<Declaration>,
    /// The member the record is about, as the request named it (`name` plus descriptor).
    identity: String,
    /// Which of the two declaration facts was missing, when one was.
    refusal: Option<Refusal>,
}

impl Plan {
    /// The declaration the artifact's envelope states, when the run could read one.
    pub(crate) fn declaration(&self) -> Option<&Declaration> {
        self.declaration.as_ref()
    }

    /// Why the run stated no declaration, as the gap every selection carries.
    pub(crate) fn refusal(&self) -> Option<Gap> {
        self.refusal.as_ref().map(|refusal| {
            let gap = DeclarationRefusal::of(refusal, &self.identity);
            Gap::whole(gap.code, gap.message)
        })
    }

    /// The declaration's own record, when the request selected rule records and the phase can pay
    /// for it: the one owning record of this rule, built after the artifact was committed.
    ///
    /// The declaration states no driver position of its own — it is a verdict about the member — so
    /// a driver range neither selects nor drops it.
    pub(crate) fn materialize(
        &self,
        publication: Publication,
        phase: &mut crate::evidence::EvidencePhase,
        budget: &mut jarde_reader::budget::Budget,
    ) -> (Option<DeclarationRecord>, crate::evidence::Materialized) {
        let methods = self.identity.clone();
        let selected = publication.publishes(&[]).then_some(());
        let (records, reached) = phase.materialize(budget, selected, |()| {
            crate::demand_counts::record_built(crate::evidence::RecoveryEvidenceKind::RuleDetails);
            self.record(&methods)
        });
        (records.into_iter().next(), reached)
    }

    /// The record one decision publishes, built from the plan this holds.
    fn record(&self, identity: &str) -> DeclarationRecord {
        let declared = self.declaration.as_ref();
        DeclarationRecord {
            method: identity.to_string(),
            member_flags: declared.map(|declaration| declaration.member_flags),
            declaring_class: declared.and_then(|declaration| {
                declaration
                    .declaring_class
                    .as_ref()
                    .map(|class| class.name().to_owned())
            }),
            interface: declared.and_then(|declaration| declaration.interface),
            form: declared.map(|declaration| declaration.form),
            presented: declared.is_some(),
            refusal: self
                .refusal
                .as_ref()
                .map(|refusal| DeclarationRefusal::of(refusal, identity)),
        }
    }
}

/// One member's declaration, as far as the caller-stated facts go.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Declaration {
    /// The form the flags make.
    pub(crate) form: DeclarationForm,
    /// The class that declares the member, when the caller stated it. A constructor or a static
    /// initializer is decided without it: neither form depends on what kind of class declares it.
    pub(crate) declaring_class: Option<DeclaringClass>,
    /// Whether that class is an interface, when the caller stated a class at all.
    pub(crate) interface: Option<bool>,
    /// The member's own flags, exactly as the class declared them.
    pub(crate) member_flags: u16,
}

impl Declaration {
    pub(crate) fn source_names(&self) -> crate::names::SourceClassNames<'_> {
        self.declaring_class
            .as_ref()
            .map(DeclaringClass::source_names)
            .unwrap_or_default()
    }
}

/// The name plus descriptor the request spells one member with.
fn identity_of(method: &MethodFacts) -> String {
    format!("{}{}", method.name(), method.descriptor())
}

/// Reads one member's declaration from the facts the caller stated.
///
/// The decision is taken for every selection — the envelope's declaration line is written from
/// [`Plan::declaration`] whether the caller asked for rule records or not — and this function builds
/// no owning record: the decision stays in the [`Plan`] and [`Plan::materialize`] writes the record
/// from it after the artifact is committed.
pub(crate) fn plan(method: &MethodFacts, source_target: crate::SourceTarget) -> Plan {
    let method_identity = identity_of(method);
    /// States one refusal as the plan's decision: the gap every selection carries, and the evidence
    /// the record a selected run materializes is written from.
    macro_rules! refuse {
        ($refusal:expr) => {{
            Plan {
                declaration: None,
                identity: method_identity,
                refusal: Some($refusal),
            }
        }};
    }
    let Some(flags) = method.access_flags() else {
        return refuse!(Refusal::unmet(
            &DECLARATION,
            MEMBER_FLAGS,
            "this run was not told which access flags the class declares for this member, so neither `static` nor `abstract` — which is what tells an interface's `default` method from its others — can be read".to_string(),
        ));
    };
    // The two initializers are decided from the member itself: neither is a `default` or a `static`
    // method question, and a class file names them in its own way (`<init>`/`<clinit>`).
    let form = match method.raw_name().0.as_slice() {
        b"<init>" => Some(DeclarationForm::Constructor),
        b"<clinit>" => Some(DeclarationForm::StaticInitializer),
        _ => method.declaring_class().map(|class| {
            if class.is_interface() {
                if flags & ACC_STATIC != 0 {
                    DeclarationForm::StaticInterfaceMethod
                } else if flags & ACC_ABSTRACT != 0 {
                    DeclarationForm::AbstractInterfaceMethod
                } else {
                    DeclarationForm::DefaultMethod
                }
            } else if flags & ACC_STATIC != 0 {
                DeclarationForm::StaticMethod
            } else {
                DeclarationForm::InstanceMethod
            }
        }),
    };
    let Some(form) = form else {
        return refuse!(Refusal::unmet(
            &DECLARATION,
            DECLARING_CLASS,
            "this run was not told which class declares this member, so whether the member is an interface's `default` method or a class's ordinary one cannot be read".to_string(),
        ));
    };
    if source_target.java_release() < 8
        && matches!(
            form,
            DeclarationForm::DefaultMethod | DeclarationForm::StaticInterfaceMethod
        )
    {
        return refuse!(Refusal::shape(
            "jre_declaration_source_target",
            "interface default and static method declarations require Java source target 8"
                .to_owned()
        ));
    }
    Plan {
        declaration: Some(Declaration {
            form,
            declaring_class: method.declaring_class().cloned(),
            interface: method.declaring_class().map(DeclaringClass::is_interface),
            member_flags: flags,
        }),
        identity: method_identity,
        refusal: None,
    }
}

/// What one run read of the presented member's declaration (P3 2.3).
///
/// `form` is the answer the acceptance asks for — a `default` method of an interface is
/// [`DeclarationForm::DefaultMethod`] and its record says which class and which flags made it one —
/// and a refusal says which of the two declaration facts was missing instead.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeclarationRecord {
    /// The member the record is about, as the request named it (`name` plus descriptor).
    pub method: String,
    /// The flags the class declares for the member, when the caller stated them.
    pub member_flags: Option<u16>,
    /// The class that declares the member, in internal form, when the caller stated it.
    pub declaring_class: Option<String>,
    /// Whether that class is an interface, when the caller stated a class at all.
    pub interface: Option<bool>,
    /// The form the run read, when it could read one.
    pub form: Option<DeclarationForm>,
    /// Whether the artifact's envelope states the declaration.
    pub presented: bool,
    /// Why it does not, when it does not.
    pub refusal: Option<DeclarationRefusal>,
}

impl DeclarationRecord {
    /// Whether the declaration was stated in the artifact.
    pub fn presented(&self) -> bool {
        self.presented
    }

    /// The rule this record is answerable to.
    pub fn rule(&self) -> RuleVersion {
        RULE
    }
}

/// Why one member's declaration was not stated.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeclarationRefusal {
    /// The diagnostic code, `jre_declaration_*`.
    pub code: &'static str,
    /// The rule that refused.
    pub rule: RuleVersion,
    /// The declared requirement that fell short.
    pub requirement: Option<String>,
    /// One sentence stating what could not be read.
    pub message: String,
}

impl DeclarationRefusal {
    fn of(refusal: &Refusal, method: &str) -> Self {
        Self {
            code: refusal.code(),
            rule: RULE,
            requirement: refusal.requirement().map(Precondition::describe),
            message: format!(
                "the declaration of {method} was not stated: {}",
                refusal.message()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{ACC_INTERFACE, ACC_PUBLIC};

    fn class(name: &str, flags: u16) -> DeclaringClass {
        DeclaringClass::new(name, flags)
    }

    /// The one record one plan materializes under the full selection, built the way the evidence
    /// phase builds it: after the decision, one charge for the record.
    fn record(plan: &Plan) -> Option<DeclarationRecord> {
        let mut budget = jarde_reader::budget::Budget::new(jarde_reader::budget::Limits {
            ir_items: 1 << 20,
            elapsed_millis: u64::MAX,
            ..jarde_reader::budget::Limits::default()
        });
        let mut phase = crate::evidence::EvidencePhase::new();
        let (record, _) = plan.materialize(
            Publication::of(&crate::RecoveryEvidenceRequest::all()),
            &mut phase,
            &mut budget,
        );
        record
    }

    #[test]
    fn an_interfaces_non_abstract_method_is_a_default_method_and_a_classs_is_not() {
        // The same member flags, two classes: what decides the form is the class, which is exactly
        // why the fact has to be stated instead of guessed from the member alone.
        let flags = ACC_PUBLIC;
        let in_interface = plan(
            &MethodFacts::new("run", "()V", 1)
                .with_access_flags(flags)
                .with_declaring_class(class("p/Shape", ACC_INTERFACE | ACC_PUBLIC)),
            crate::SourceTarget::JAVA_8,
        );
        assert_eq!(
            in_interface.declaration().map(|d| d.form),
            Some(DeclarationForm::DefaultMethod)
        );
        let in_class = plan(
            &MethodFacts::new("run", "()V", 1)
                .with_access_flags(flags)
                .with_declaring_class(class("p/Shape", ACC_PUBLIC)),
            crate::SourceTarget::JAVA_8,
        );
        assert_eq!(
            in_class.declaration().map(|d| d.form),
            Some(DeclarationForm::InstanceMethod)
        );
        assert_eq!(DeclarationForm::DefaultMethod.name(), "default_method");
        assert!(DeclarationForm::DefaultMethod.spell().contains("default"));
    }

    #[test]
    fn an_interface_method_without_either_flag_is_abstract_and_a_static_one_is_static() {
        let interface = ACC_INTERFACE | ACC_PUBLIC;
        let abstract_method = plan(
            &MethodFacts::new("run", "()V", 1)
                .with_access_flags(ACC_PUBLIC | ACC_ABSTRACT)
                .with_declaring_class(class("p/Shape", interface)),
            crate::SourceTarget::JAVA_8,
        );
        assert_eq!(
            abstract_method.declaration().map(|d| d.form),
            Some(DeclarationForm::AbstractInterfaceMethod)
        );
        let static_method = plan(
            &MethodFacts::new("run", "()V", 0)
                .with_access_flags(ACC_PUBLIC | ACC_STATIC)
                .with_declaring_class(class("p/Shape", interface)),
            crate::SourceTarget::JAVA_8,
        );
        assert_eq!(
            static_method.declaration().map(|d| d.form),
            Some(DeclarationForm::StaticInterfaceMethod)
        );
        // A constructor is decided without the class's own flags: the form does not depend on them.
        let constructor = plan(
            &MethodFacts::new("<init>", "()V", 1).with_access_flags(0),
            crate::SourceTarget::JAVA_8,
        );
        assert_eq!(
            constructor.declaration().map(|d| d.form),
            Some(DeclarationForm::Constructor)
        );
        assert!(
            constructor
                .declaration()
                .is_some_and(|d| d.declaring_class.is_none())
        );
    }

    #[test]
    fn a_missing_declaration_fact_is_refused_with_its_own_requirement_named() {
        let without_flags = plan(
            &MethodFacts::new("run", "()V", 1)
                .with_declaring_class(class("p/Shape", ACC_INTERFACE | ACC_PUBLIC)),
            crate::SourceTarget::JAVA_8,
        );
        assert!(without_flags.declaration().is_none());
        assert_eq!(
            record(&without_flags)
                .and_then(|record| record.refusal)
                .map(|r| r.code),
            Some("jre_declaration_flags_missing")
        );
        assert!(
            record(&without_flags)
                .and_then(|record| record.refusal)
                .is_some_and(|r| r.requirement.as_deref() == Some("the `access_flags` attribute"))
        );
        // The same refusal is what every selection carries, as the gap the report states.
        assert_eq!(
            without_flags.refusal().map(|gap| gap.code()),
            Some("jre_declaration_flags_missing")
        );
        let without_class = plan(
            &MethodFacts::new("run", "()V", 1).with_access_flags(ACC_PUBLIC),
            crate::SourceTarget::JAVA_8,
        );
        assert_eq!(
            record(&without_class)
                .and_then(|record| record.refusal)
                .map(|r| r.code),
            Some("jre_declaration_class_not_in_run")
        );
        assert!(!record(&without_class).is_some_and(|record| record.presented()));
    }
}

/// One method descriptor as this presentation writes its declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Signature {
    /// The parameters in declaration order: the type as Java spells it, and the **local slot** the
    /// parameter occupies, with the receiver counted.
    ///
    /// Array dimensions come from the descriptor via [`crate::type_of_component`].
    /// The argument list applies [`Self::varargs`] only to its last parameter.
    ///
    /// The slot is the JVM layer's derivation from the descriptor's own facts
    /// ([`jarde_jvm::method_ir::parameter_positions`]): `this` holds slot 0 for an instance member; each
    /// parameter starts where the one before it ended — a `long`/`double` filling two slots and
    /// **an array of either filling one**, like every other array.
    pub parameters: Vec<(String, u16)>,
    /// Whether the declaration's **last** parameter is written `T...`: true exactly when the
    /// member's own `ACC_VARARGS` is set and the descriptor's last parameter is an array, which is
    /// the one shape the flag can apply to (see [`method_descriptor`]).
    ///
    /// It is stated here and not spelled into [`Self::parameters`] because it is a fact about the
    /// parameter **list** — the last position, and only the last — while each entry above is the
    /// type its own component names.
    pub varargs: bool,
    /// The return type, or `None` for `V`.
    pub returns: Option<String>,
    /// How many slots the parameters occupy together, the receiver included: the first slot a body
    /// of this member may declare a local in.
    pub slots: u16,
}

/// One method descriptor as Java source spells it, or `None` when the bytes are not that production
/// or state a type this presentation cannot write.
///
/// The descriptor is read **once**, by the reader's own facts, and everything the declaration needs
/// comes from that one reading: the types are the recovery layer's spelling of the reader's
/// components, and the slots are the JVM layer's derivation from those same components. So a
/// signature's parameter names and the body's references to them are one reading of one descriptor
/// — never two parses that could place a parameter in two different slots.
///
/// `varargs` is the member's own `ACC_VARARGS` (0x0080), the one fact of the declaration that the
/// descriptor does not state. It reaches the **last** parameter and only when that parameter is an
/// array: `[I` is written `int...` and `[[B` is written `byte[]...` — the element type, then the
/// `[]` it still has, then the dots the last `[]` became. A member whose flag is set and whose last
/// parameter is not an array states no `...` at all: the flag is carried by no position, and the
/// parameters are written as the descriptor states them.
pub fn method_descriptor(descriptor: &[u8], is_static: bool, varargs: bool) -> Option<Signature> {
    let facts = jarde_reader::classfile::descriptor_facts(
        descriptor,
        jarde_reader::classfile::DescriptorKind::Method,
    )
    .ok()?;
    let positions = jarde_jvm::method_ir::parameter_positions(&facts, is_static)?;
    let parameters = facts
        .parameters()
        .iter()
        .zip(positions)
        .map(|(component, slot)| {
            Some((
                crate::type_of_component(component)?.spell().to_owned(),
                slot,
            ))
        })
        .collect::<Option<Vec<(String, u16)>>>()?;
    // The two facts meet here, and neither alone states a `...`: the member's own flag states that
    // its last parameter is the variable-arity one, and the descriptor's own last component states
    // whether there is an array for that to mean. An array parameter of a member without the flag
    // keeps its brackets, and a flag on a member whose last parameter is not an array is written as
    // the descriptor states it.
    let varargs = varargs
        && facts
            .parameters()
            .last()
            .is_some_and(jarde_reader::classfile::DescriptorComponent::is_array);
    let returns = match facts.result() {
        Some(component) => Some(crate::type_of_component(component)?.spell().to_owned()),
        None => None,
    };
    let slots = u16::from(!is_static).checked_add(facts.parameter_slots()?)?;
    Some(Signature {
        parameters,
        varargs,
        returns,
        slots,
    })
}

/// Spell the final array parameter as variable arity when its descriptor permits it.
pub fn varargs_type(ty: &str, varargs: bool) -> String {
    match ty.strip_suffix("[]") {
        Some(element) if varargs => format!("{element}..."),
        _ => ty.to_owned(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvedMethodParameter {
    pub slot: u16,
    pub name: JvmBytes,
    pub is_final: bool,
}

/// The names the parameters of one member are written with, in **slot** order: index 0 is slot 0,
/// which is the receiver of a member that is not `static` and the first parameter of one that is.
///
/// The names are the recovery layer's own — [`NameTable`] over the debug names the member's run
/// read — so the signature and the body spell one parameter the same way: a slot the debug table
/// names once takes that name (aliased when Java cannot spell it), a slot it names over several
/// records takes the first variable's name, and a slot it does not name takes the ordinal name the
/// naming table invents (`arg<slot>`), which is exactly what the body's statements say.
///
/// A member no run was performed for — one that declares no body, one that could not be spelled —
/// carries no debug names at all, so its parameters take those ordinal names. That is a rule of this
/// presentation's, stated here, and not a claim about the source: the class file does not state
/// parameter names, and the debug table is the only place one may come from.
fn parameter_names(facts: Option<&RecoveryFacts>, slots: u16) -> Vec<String> {
    let mut records: Vec<Vec<JvmBytes>> = vec![Vec::new(); usize::from(slots)];
    for record in facts.into_iter().flat_map(|facts| facts.debug_locals()) {
        if let Some(names) = records.get_mut(usize::from(record.slot())) {
            names.push(record.name().to_owned());
        }
    }
    let evidence: Vec<SlotEvidence> = records
        .into_iter()
        .map(|names| match names.len() {
            0 => SlotEvidence::Unnamed,
            1 => SlotEvidence::Whole(names[0].clone()),
            _ => SlotEvidence::Split(names.into_iter().map(Some).collect()),
        })
        .collect();
    let table = if facts.is_some_and(|facts| facts.method().has_receiver()) {
        NameTable::build_with_receiver(slots, slots, &evidence)
    } else {
        NameTable::build(slots, slots, &evidence)
    };
    (0..slots)
        .map(|slot| {
            table
                .name(LocalVariable::new(slot, 0))
                .map(|name| name.text().to_owned())
                .unwrap_or_else(|| format!("arg{slot}"))
        })
        .collect()
}

/// Whether this presentation can write a declaration for a member whose raw descriptor these bytes
/// are: a method descriptor (JVMS 4.3.3) whose every type is one Java source spells.
///
/// A member this answers `false` for has no writable source declaration; the caller
/// retains its physical record without starting a recovery whose artifact it cannot place.
pub fn spellable_method_descriptor(descriptor: &[u8]) -> bool {
    match descriptor_facts(descriptor, DescriptorKind::Method) {
        Ok(facts) => facts
            .components()
            .all(|component| type_of_component(component).is_some()),
        Err(_) => false,
    }
}

/// The visibility Java source spells, or nothing at all for a package-private declaration.
pub fn visibility(access_flags: u16) -> Option<&'static str> {
    if access_flags & ACC_PUBLIC != 0 {
        Some("public")
    } else if access_flags & ACC_PROTECTED != 0 {
        Some("protected")
    } else if access_flags & ACC_PRIVATE != 0 {
        Some("private")
    } else {
        None
    }
}

/// 直接 Override 提示的子类源码种类准入；在门面首次读取子定义之前调用。
/// 只判断已有 flags，保留原 interface/annotation/enum/module 和 abstract-final 边界。
pub fn direct_override_child_class_flags_supported(flags: u16) -> bool {
    flags & (ACC_INTERFACE | ACC_ANNOTATION | 0x4000 | 0x8000) == 0
        && flags & (0x0010 | 0x0400) != (0x0010 | 0x0400)
}

/// 直接 Override 的现有窄域不消费泛型类作用域；只检查同次读取的属性 shells。
pub fn direct_override_scope_supported(attributes: &[AttributeShell]) -> bool {
    !attributes.iter().any(is_signature_attribute)
}

/// 门面完成父定义的物理身份和完整性核对后，判断父类的源码资格。
pub fn direct_override_parent_class_supported(flags: u16, attributes: &[AttributeShell]) -> bool {
    flags & (ACC_INTERFACE | ACC_ANNOTATION | 0x0010 | 0x4000 | 0x8000) == 0
        && direct_override_scope_supported(attributes)
}

/// 子方法源码资格在父方法表匹配/计费前判断，不选择或读取父定义。
/// 门面已绑定 report index、raw name/descriptor 与 flags；此处消费相同 member。
/// 保持 flags/name、Signature、已有声明/注解/正文、descriptor 的原短路顺序。
pub fn direct_override_child_method_supported(
    method: &MemberHeader,
    has_declaration: bool,
    annotations: &[String],
    text: &str,
) -> bool {
    ordinary_override_method(method, false)
        && direct_override_scope_supported(&method.attributes)
        && has_declaration
        && !annotations
            .iter()
            .any(|use_line| use_line == "@Override" || use_line == "@java.lang.Override")
        && !text.is_empty()
        && descriptor_facts(&method.descriptor.raw().0, DescriptorKind::Method).is_ok()
}

/// 只在门面完成 raw name/descriptor 的父子唯一配对和原计费后调用。
/// 返回源码资格，不构造可信来源或提交 Override；native/abstract 等边界沿用原规则。
pub fn direct_override_parent_method_supported(
    child: &MemberHeader,
    parent: &MemberHeader,
) -> bool {
    ordinary_override_method(parent, true)
        && direct_override_scope_supported(&parent.attributes)
        && visibility_rank(child.access_flags) >= visibility_rank(parent.access_flags)
}

fn is_signature_attribute(attribute: &AttributeShell) -> bool {
    attribute.name.raw().0 == b"Signature"
}

fn visibility_rank(flags: u16) -> u8 {
    if flags & 0x0001 != 0 {
        2
    } else if flags & 0x0004 != 0 {
        1
    } else {
        0
    }
}

fn ordinary_override_method(method: &MemberHeader, parent: bool) -> bool {
    let flags = method.access_flags;
    let name = &method.name.raw().0;
    let access = flags & (0x0001 | 0x0002 | 0x0004);
    let invalid_abstract = flags & 0x0400 != 0 && flags & (0x0010 | 0x0020 | 0x0100 | 0x0800) != 0;
    (access == 0 || access == 0x0001 || access == 0x0004)
        && flags
            & !(0x0001
                | 0x0002
                | 0x0004
                | 0x0008
                | 0x0010
                | 0x0020
                | 0x0040
                | 0x0080
                | 0x0100
                | 0x0400
                | 0x0800
                | 0x1000)
            == 0
        && !invalid_abstract
        && flags & (0x0002 | 0x0008 | 0x0040 | 0x1000) == 0
        && (!parent || flags & 0x0010 == 0)
        && name != b"<init>"
        && name != b"<clinit>"
        && crate::names::source_method_name(method.name.raw()).is_some()
}

/// The `throws` clause one member's own `Exceptions` attribute is written as, after its parameter
/// list, or nothing at all when the member declares no exception.
///
/// The clause is one line of the declaration and it states exactly the types the attribute states,
/// in the attribute's own order; an empty list is no clause, because
/// `throws` with nothing after it is not a clause Java writes.
fn throws_clause(throws: &[String]) -> String {
    if throws.is_empty() {
        return String::new();
    }
    format!(" throws {}", throws.join(", "))
}

/// Whether an interface's selected physical member is spelled with `default`.
pub fn is_default_method(class_flags: u16, flags: u16, has_code: bool) -> bool {
    class_flags & ACC_INTERFACE != 0
        && flags & ACC_STATIC == 0
        && flags & ACC_ABSTRACT == 0
        && has_code
}

/// Refuse a whole source unit whose selected declarations require Java 8.
/// The caller lends existing report facts; no physical member or recovery result is changed.
/// The method and annotation accessors only borrow facts and perform no reads or analysis.
pub fn source_unit_target_refusal<F, M>(
    source_target: crate::SourceTarget,
    class_name: &JvmString,
    class_flags: u16,
    fields: &[F],
    methods: &[M],
    method_body_facts: impl Fn(&M) -> (u16, bool),
    field_has_type_annotations: impl Fn(&F) -> bool,
    method_has_type_annotations: impl Fn(&M) -> bool,
) -> Option<String> {
    if source_target.java_release() < 8 {
        let interface_body = class_flags & ACC_INTERFACE != 0
            && methods.iter().any(|method| {
                let (flags, has_code) = method_body_facts(method);
                is_default_method(ACC_INTERFACE, flags, has_code)
                    || (flags & ACC_STATIC != 0 && has_code)
            });
        let type_uses = fields.iter().any(field_has_type_annotations)
            || methods.iter().any(method_has_type_annotations);
        if interface_body || type_uses {
            return Some(format!(
                "// jarde: source_target_refused: `{}` requires Java 8 {}. Physical declarations, annotations and method recoveries remain in this report.\n",
                crate::emit::comment_text(&class_name.escaped()),
                if interface_body {
                    "interface method bodies"
                } else {
                    "type annotations"
                },
            ));
        }
    }
    None
}

/// Source-path eligibility is evaluated before annotation spelling by the caller.
pub fn method_signature_source_paths_valid(
    signature: &Signature,
    names: SourceClassNames<'_>,
) -> bool {
    signature
        .parameters
        .iter()
        .all(|(ty, _)| names.reference(ty).is_some())
        && signature
            .returns
            .as_deref()
            .is_none_or(|ty| names.reference(ty).is_some())
}

/// The argument list one declaration is written with: every parameter of the descriptor, in
/// declaration order, each with its type and its name.
///
/// A parameter's slot is the JVM layer's own derivation ([`Signature::parameters`]) — `this` holds
/// slot 0 of a member that is not `static`, a `long`/`double` fills two slots and an **array of
/// either fills one** — and the name is the one the body's own statements use for that slot
/// ([`parameter_names`]), so the declaration and the body cannot name two different slots for one
/// parameter.
///
/// The one parameter whose type is not written as the descriptor states it is the last one of a
/// member whose own `ACC_VARARGS` applies ([`Signature::varargs`]): it is written `T...`, the array
/// it is with the last `[]` spelled as the dots (see [`varargs_type`]). No other position is
/// touched, and no call site is read: the declaration says the member takes a variable number of
/// arguments, and the body still reads the slot as the array the descriptor states.
pub fn method_arguments(
    facts: Option<&RecoveryFacts>,
    recovered_parameter_names: Option<&[Option<String>]>,
    signature: &Signature,
    source_names: SourceClassNames<'_>,
    parameter_annotations: &[Vec<String>],
    type_annotations: &[Vec<String>],
    proved: Option<&[ProvedMethodParameter]>,
) -> String {
    let mut names = parameter_names(facts, signature.slots);
    if let Some(recovered) = recovered_parameter_names {
        for (name, chosen) in names.iter_mut().zip(recovered) {
            if let Some(chosen) = chosen {
                *name = chosen.clone();
            }
        }
    }
    let proved = proved.filter(|parameters| {
        facts.is_some_and(|facts| {
            facts.debug_locals().len() == parameters.len()
                && parameters
                    .iter()
                    .zip(facts.debug_locals())
                    .all(|(parameter, local)| {
                        local.slot() == parameter.slot
                            && local.name() == &parameter.name
                            && local.range().is_none()
                    })
        })
    });
    let written: Vec<String> = signature
        .parameters
        .iter()
        .enumerate()
        .map(|(position, (ty, slot))| {
            let name = names
                .get(usize::from(*slot))
                .cloned()
                .unwrap_or_else(|| format!("arg{slot}"));
            let varargs = signature.varargs && position + 1 == signature.parameters.len();
            let ty = source_names.reference(ty).unwrap_or_else(|| ty.to_owned());
            let ty = type_annotations
                .get(position)
                .filter(|items| !items.is_empty())
                .and_then(|items| decorate_qualified_type(&ty, items))
                .unwrap_or(ty);
            let ty = varargs_type(&ty, varargs);
            let annotations = parameter_annotations
                .get(position)
                .into_iter()
                .flatten()
                .map(|annotation| format!("{annotation} "))
                .collect::<String>();
            let final_word = if proved
                .and_then(|parameters| parameters.get(position))
                .is_some_and(|parameter| parameter.slot == *slot && parameter.is_final)
            {
                "final "
            } else {
                ""
            };
            format!("{annotations}{final_word}{ty} {name}")
        })
        .collect();
    format!("({})", written.join(", "))
}

/// Decorate the selected qualified, non-array source type with its type-use annotations.
pub fn decorate_qualified_type(ty: &str, annotations: &[String]) -> Option<String> {
    let (package, simple) = ty.rsplit_once('.')?;
    if package.is_empty() || simple.contains('$') || ty.contains("[]") {
        return None;
    }
    Some(format!("{package}.{} {simple}", annotations.join(" ")))
}

/// Complete ordinary member declaration policy over supplied same-run facts.
/// The caller keeps physical reads, annotation facts, output charging and publication.
#[allow(clippy::too_many_arguments)]
pub fn spell_method_declaration(
    raw_name: &JvmString,
    descriptor: &JvmString,
    flags: u16,
    has_code: bool,
    facts: Option<&RecoveryFacts>,
    recovered_parameter_names: Option<&[Option<String>]>,
    class: &str,
    source_names: SourceClassNames<'_>,
    class_flags: u16,
    throws: &[String],
    default: Option<&crate::metadata_literal::MemberDefault>,
    member_annotations: &mut crate::report::MemberAnnotationUses,
    parameter_annotations: &mut crate::report::ParameterAnnotationUses,
    return_type_annotations: &[String],
    parameter_type_annotations: &[Vec<String>],
    proved: Option<&[ProvedMethodParameter]>,
    signature_present: bool,
    source_paths_valid: bool,
) -> (Option<String>, Option<String>) {
    if raw_name.raw().0 == b"<clinit>" {
        crate::metadata_literal::refuse_uses(
            member_annotations,
            "class initializer blocks have no Java method declaration to annotate",
        );
    }
    let (mut declaration, mut marker) = (|| {
        let raw = &raw_name.raw().0;
        if raw == b"<clinit>" {
            return (Some("static".to_owned()), None);
        }
        let Some(signature) = method_descriptor(
            &descriptor.raw().0,
            flags & ACC_STATIC != 0,
            flags & ACC_VARARGS != 0,
        ) else {
            return (
                None,
                Some(format!(
                    "// jarde: not spelled: the descriptor `{}` of the member `{}` is not a method descriptor, so this presentation writes no declaration for it and performs no run for it",
                    crate::comment_text(&descriptor.escaped()),
                    crate::comment_text(&raw_name.escaped())
                )),
            );
        };
        if raw == b"<init>" {
            let mut declaration = String::new();
            if let Some(word) = visibility(flags) {
                declaration.push_str(word);
                declaration.push(' ');
            }
            declaration.push_str(class);
            declaration.push_str(&method_arguments(
                facts,
                recovered_parameter_names,
                &signature,
                source_names,
                &parameter_annotations.uses_by_position,
                parameter_type_annotations,
                proved,
            ));
            declaration.push_str(&throws_clause(throws));
            return (Some(declaration), None);
        }
        let (name, aliased) = crate::names::method_declaration_name(raw_name.raw());
        let mut words: Vec<&str> = Vec::new();
        words.extend(visibility(flags));
        if flags & ACC_STATIC != 0 {
            words.push("static");
        }
        if flags & ACC_FINAL != 0 {
            words.push("final");
        }
        if flags & ACC_ABSTRACT != 0 {
            words.push("abstract");
        }
        if flags & ACC_NATIVE != 0 {
            words.push("native");
        }
        if flags & ACC_SYNCHRONIZED != 0 {
            words.push("synchronized");
        }
        if flags & ACC_STRICT != 0 {
            words.push("strictfp");
        }
        if is_default_method(class_flags, flags, has_code) {
            words.push("default");
        }
        let returns = signature.returns.as_deref().unwrap_or("void");
        let returns = source_names
            .reference(returns)
            .unwrap_or_else(|| returns.to_owned());
        let returns = if return_type_annotations.is_empty() {
            returns
        } else {
            decorate_qualified_type(&returns, return_type_annotations).unwrap_or(returns)
        };
        let mut declaration = String::new();
        for word in words {
            declaration.push_str(word);
            declaration.push(' ');
        }
        declaration.push_str(&returns);
        declaration.push(' ');
        declaration.push_str(&name);
        declaration.push_str(&method_arguments(
            facts,
            recovered_parameter_names,
            &signature,
            source_names,
            &parameter_annotations.uses_by_position,
            parameter_type_annotations,
            proved,
        ));
        declaration.push_str(&throws_clause(throws));
        if let Some(default) = default {
            declaration.push_str(" default ");
            declaration.push_str(&default.spelling());
        }
        (Some(declaration), aliased.then(|| format!(
            "// jarde: the member's raw name `{}` is not a Java identifier; it is written as `{}`",
            crate::comment_text(&raw_name.escaped()), crate::names::written_name(raw).0
        )))
    })();
    if signature_present && !source_paths_valid {
        declaration = None;
        marker = Some(format!(
            "// jarde: not spelled: member `{}` has no unambiguous Java type path in the selected class's nesting facts",
            crate::comment_text(&raw_name.escaped()),
        ));
    }
    if declaration.is_none() {
        crate::metadata_literal::refuse_uses(
            member_annotations,
            "method descriptor cannot be spelled as a Java declaration",
        );
        crate::metadata_literal::refuse_parameter_uses(
            parameter_annotations,
            "method descriptor cannot establish parameter positions",
        );
    }
    (declaration, marker)
}

const ACC_ENUM: u16 = 0x4000;
const ACC_VOLATILE: u16 = 0x0040;
const ACC_TRANSIENT: u16 = 0x0080;

/// Class declaration name and ordinary header, over the selected physical header facts.
pub fn class_source_declaration(
    this_class: &JvmString,
    flags: u16,
    super_class: Option<&JvmString>,
    interfaces: &[JvmString],
) -> jarde_reader::error::Result<(String, String)> {
    let raw = &this_class.raw().0;
    let refused = || {
        jarde_reader::error::Error::unsupported(
            "class_source_name_unspellable",
            "class or parent name has no faithful Java source spelling",
        )
    };
    if raw.contains(&b'/') && crate::names::class_package_name(raw).is_none() {
        return Err(refused());
    }
    let name = crate::names::class_simple_name(raw)
        .or_else(|| {
            raw.ends_with(b"/package-info")
                .then(|| "package-info".to_owned())
        })
        .ok_or_else(refused)?;
    let declaration = class_declaration_with_types(
        &name,
        this_class,
        flags,
        super_class,
        interfaces,
        None,
        None,
        None,
    )
    .ok_or_else(refused)?;
    Ok((name, declaration))
}

/// Nested declarations cannot use the root's physical package-info placeholder.
pub fn nested_member_class_header(
    name: &str,
    this_class: &JvmString,
    flags: u16,
    super_class: Option<&JvmString>,
    interfaces: &[JvmString],
) -> Option<String> {
    if crate::names::source_class_name(&JvmBytes::from_unicode(name)).as_deref() != Some(name) {
        return None;
    }
    class_declaration_with_types(
        name,
        this_class,
        flags,
        super_class,
        interfaces,
        None,
        None,
        None,
    )
}

/// Complete class/interface/enum/annotation declaration spelling; no physical read or publication.
#[allow(clippy::too_many_arguments)]
pub fn class_declaration_with_types(
    name: &str,
    this_class: &JvmString,
    flags: u16,
    super_class: Option<&JvmString>,
    interfaces: &[JvmString],
    parameters: Option<&str>,
    generic_superclass: Option<&str>,
    generic_interfaces: Option<&[String]>,
) -> Option<String> {
    // The package-info path carries a physical metadata placeholder until the facade's dedicated
    // package certificate replaces it. Ordinary and nested type declarations require a source name.
    let package_metadata = name == "package-info" && this_class.raw().0.ends_with(b"/package-info");
    if !package_metadata
        && crate::names::source_class_name(&JvmBytes::from_unicode(name)).as_deref() != Some(name)
    {
        return None;
    }
    let interface = flags & ACC_INTERFACE != 0;
    let canonical_annotation = flags & (ACC_ANNOTATION | ACC_INTERFACE)
        == (ACC_ANNOTATION | ACC_INTERFACE)
        && interfaces.len() == 1
        && interfaces[0].raw().0 == b"java/lang/annotation/Annotation";
    let enumeration = !interface && flags & ACC_ENUM != 0;
    let mut words: Vec<&str> = Vec::new();
    words.extend(visibility(flags));
    if !interface {
        if flags & ACC_ABSTRACT != 0 && !enumeration {
            words.push("abstract");
        }
        if flags & ACC_FINAL != 0 && !enumeration {
            words.push("final");
        }
        if flags & ACC_STRICT != 0 {
            words.push("strictfp");
        }
    }
    let kind = if flags & ACC_ANNOTATION != 0 {
        "@interface"
    } else if interface {
        "interface"
    } else if enumeration {
        "enum"
    } else {
        "class"
    };
    let mut line = String::new();
    for word in words {
        line.push_str(word);
        line.push(' ');
    }
    line.push_str(kind);
    line.push(' ');
    line.push_str(name);
    if let Some(parameters) = parameters {
        line.push('<');
        line.push_str(parameters);
        line.push('>');
    }
    if canonical_annotation {
        return Some(line);
    }
    let supers = if let Some(interfaces) = generic_interfaces {
        interfaces.to_vec()
    } else {
        interfaces
            .iter()
            .map(|interface| crate::names::source_class_name(interface.raw()))
            .collect::<Option<Vec<_>>>()?
    };
    if interface {
        if !supers.is_empty() {
            line.push_str(" extends ");
            line.push_str(&supers.join(", "));
        }
        return Some(line);
    }
    if !enumeration && let Some(super_class) = super_class {
        line.push_str(" extends ");
        if let Some(generic_superclass) = generic_superclass {
            line.push_str(generic_superclass);
        } else {
            line.push_str(&crate::names::source_class_name(super_class.raw())?);
        }
    }
    if !supers.is_empty() {
        line.push_str(" implements ");
        line.push_str(&supers.join(", "));
    }
    Some(line)
}

/// A field descriptor's faithful Java type, using the shared descriptor and source type readers.
pub fn field_type(descriptor: &[u8]) -> Option<String> {
    let facts = descriptor_facts(descriptor, DescriptorKind::Field).ok()?;
    Some(type_of_component(facts.single()?)?.spell().to_owned())
}

/// Complete field declaration and its existing alias/descriptor marker.
pub fn spell_field_declaration(
    raw_name: &JvmString,
    descriptor: &JvmString,
    flags: u16,
    source_names: SourceClassNames<'_>,
    constant: Option<&crate::metadata_literal::MemberDefault>,
) -> (Option<String>, Option<String>) {
    let Some(ty) = field_type(&descriptor.raw().0).and_then(|ty| source_names.reference(&ty))
    else {
        return (
            None,
            Some(format!(
                "// jarde: not spelled: the descriptor `{}` of the field `{}` is not a field descriptor, so this presentation writes no declaration for it",
                crate::comment_text(&descriptor.escaped()),
                crate::comment_text(&raw_name.escaped())
            )),
        );
    };
    let (name, aliased) = crate::names::written_name(&raw_name.raw().0);
    let declaration = field_declaration_with_type(raw_name, flags, &ty, constant);
    (Some(declaration), aliased.then(|| format!(
        "// jarde: the field's raw name `{}` is not a Java identifier; it is written as `{name}`",
        crate::comment_text(&raw_name.escaped())
    )))
}

/// Ordinary and Signature-projected fields share the same flags/name/initializer spelling.
pub fn field_declaration_with_type(
    raw_name: &JvmString,
    flags: u16,
    ty: &str,
    constant: Option<&crate::metadata_literal::MemberDefault>,
) -> String {
    let (name, _) = crate::names::written_name(&raw_name.raw().0);
    let mut words: Vec<&str> = Vec::new();
    words.extend(visibility(flags));
    if flags & ACC_STATIC != 0 {
        words.push("static");
    }
    if flags & ACC_FINAL != 0 {
        words.push("final");
    }
    if flags & ACC_VOLATILE != 0 {
        words.push("volatile");
    }
    if flags & ACC_TRANSIENT != 0 {
        words.push("transient");
    }
    let mut declaration = String::new();
    for word in words {
        declaration.push_str(word);
        declaration.push(' ');
    }
    declaration.push_str(ty);
    declaration.push(' ');
    declaration.push_str(&name);
    if let Some(constant) = constant {
        declaration.push_str(" = ");
        declaration.push_str(&constant.spelling());
    }
    declaration
}

/// Place field type annotations and refuse declaration annotations without a source position.
/// Consumes only supplied facts; the caller owns the physical field and report transaction.
pub fn place_field_annotations(
    declaration: &mut Option<String>,
    descriptor: &[u8],
    source_names: SourceClassNames<'_>,
    type_annotations: &[String],
    member_annotations: &mut crate::report::MemberAnnotationUses,
) {
    if let Some(text) = declaration.as_ref()
        && let (Some(ty), Some(decorated)) = (
            field_type(descriptor).map(|ty| source_names.reference(&ty).unwrap_or(ty)),
            field_type(descriptor).and_then(|ty| {
                decorate_qualified_type(source_names.reference(&ty)?.as_str(), type_annotations)
            }),
        )
        && !type_annotations.is_empty()
    {
        *declaration = Some(replace_first(text, &ty, &decorated));
    }
    if declaration.is_none() {
        crate::metadata_literal::refuse_uses(
            member_annotations,
            "field descriptor cannot be spelled as a Java declaration",
        );
    }
}

fn replace_first(text: &str, from: &str, to: &str) -> String {
    text.find(from).map_or_else(
        || text.to_owned(),
        |at| format!("{}{}{}", &text[..at], to, &text[at + from.len()..]),
    )
}
