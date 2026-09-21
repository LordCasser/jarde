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
            declaring_class: declared.and_then(|declaration| declaration.declaring_class.clone()),
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
    pub(crate) declaring_class: Option<String>,
    /// Whether that class is an interface, when the caller stated a class at all.
    pub(crate) interface: Option<bool>,
    /// The member's own flags, exactly as the class declared them.
    pub(crate) member_flags: u16,
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
pub(crate) fn plan(method: &MethodFacts) -> Plan {
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
    let form = match method.name() {
        "<init>" => Some(DeclarationForm::Constructor),
        "<clinit>" => Some(DeclarationForm::StaticInitializer),
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
    Plan {
        declaration: Some(Declaration {
            form,
            declaring_class: method
                .declaring_class()
                .map(|class| class.name().to_string()),
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
        );
        assert_eq!(
            in_interface.declaration().map(|d| d.form),
            Some(DeclarationForm::DefaultMethod)
        );
        let in_class = plan(
            &MethodFacts::new("run", "()V", 1)
                .with_access_flags(flags)
                .with_declaring_class(class("p/Shape", ACC_PUBLIC)),
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
        );
        assert_eq!(
            abstract_method.declaration().map(|d| d.form),
            Some(DeclarationForm::AbstractInterfaceMethod)
        );
        let static_method = plan(
            &MethodFacts::new("run", "()V", 0)
                .with_access_flags(ACC_PUBLIC | ACC_STATIC)
                .with_declaring_class(class("p/Shape", interface)),
        );
        assert_eq!(
            static_method.declaration().map(|d| d.form),
            Some(DeclarationForm::StaticInterfaceMethod)
        );
        // A constructor is decided without the class's own flags: the form does not depend on them.
        let constructor = plan(&MethodFacts::new("<init>", "()V", 1).with_access_flags(0));
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
        let without_class = plan(&MethodFacts::new("run", "()V", 1).with_access_flags(ACC_PUBLIC));
        assert_eq!(
            record(&without_class)
                .and_then(|record| record.refusal)
                .map(|r| r.code),
            Some("jre_declaration_class_not_in_run")
        );
        assert!(!record(&without_class).is_some_and(|record| record.presented()));
    }
}
