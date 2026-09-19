//! What the class file declares the presented member to be (P3 2.3, rule `declaration@1`).
//!
//! # Why this needs a caller-stated fact, and which one
//!
//! The artifact of this layer is one method **body**: an envelope comment naming the member, then its
//! statements. The member's own declaration — is it a `default` method of an interface? a static
//! method of an interface? a constructor? — is therefore written into that envelope, and it cannot
//! be read from the payload: one run's payload publishes that body's graph, frames, names, decode and
//! pool, and **no class-level fact at all** (P3 2.3 §3 measured this against `MethodIr`'s own
//! fields: no `this_class`, no `InnerClasses`). The **class's** flags are therefore still the
//! caller's to state, and P3 2.2 already let the caller state a member's flags for the bridge rule.
//!
//! The **member's** own flags have since stopped being one of them: P3 3.1 put the driver method's
//! declaration — its `access_flags`, descriptor, parameter slots and identity — into the payload,
//! read in the same header pass that read the body, and the entry point fills
//! [`crate::facts::MethodFacts`] from there. So the flag on the left below now reaches this rule
//! from the run rather than from a second reading, and the field stays on the facts type because the
//! library's callers may still be the ones that read it.
//!
//! So this rule reads exactly two declaration facts, both optional:
//!
//! * the member's own `access_flags` ([`crate::facts::MethodFacts::access_flags`]) — stated by the
//!   caller, and by the entry point from the run's own declaration;
//! * the class that declares it, with that class's own flags ([`crate::facts::DeclaringClass`]) —
//!   stated by the caller, because the payload carries the declaring class's identity but not its
//!   flags.
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

use crate::facts::{ACC_ABSTRACT, ACC_STATIC, DeclaringClass, MethodFacts};
use crate::pass::{DECLARATION, Precondition, RuleVersion};
use crate::refusal::Refusal;

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

/// The declaration one run could read, and the record of how it read it.
pub(crate) struct Plan {
    declaration: Option<Declaration>,
    record: DeclarationRecord,
}

impl Plan {
    /// The declaration the artifact's envelope states, when the run could read one.
    pub(crate) fn declaration(&self) -> Option<&Declaration> {
        self.declaration.as_ref()
    }

    /// The record, whether the declaration was read or refused.
    pub(crate) fn record(&self) -> &DeclarationRecord {
        &self.record
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

/// Reads one member's declaration from the facts the caller stated.
pub(crate) fn plan(method: &MethodFacts) -> Plan {
    let method_identity = format!("{}{}", method.name(), method.descriptor());
    let mut record = DeclarationRecord {
        method: method_identity,
        member_flags: method.access_flags(),
        declaring_class: method
            .declaring_class()
            .map(|class| class.name().to_string()),
        interface: method.declaring_class().map(DeclaringClass::is_interface),
        form: None,
        presented: false,
        refusal: None,
    };
    let Some(flags) = method.access_flags() else {
        record.refusal = Some(DeclarationRefusal::of(
            &Refusal::unmet(
                &DECLARATION,
                MEMBER_FLAGS,
                "this run was not told which access flags the class declares for this member, so neither `static` nor `abstract` — which is what tells an interface's `default` method from its others — can be read".to_string(),
            ),
            &record.method,
        ));
        return Plan {
            declaration: None,
            record,
        };
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
        record.refusal = Some(DeclarationRefusal::of(
            &Refusal::unmet(
                &DECLARATION,
                DECLARING_CLASS,
                "this run was not told which class declares this member, so whether the member is an interface's `default` method or a class's ordinary one cannot be read".to_string(),
            ),
            &record.method,
        ));
        return Plan {
            declaration: None,
            record,
        };
    };
    record.form = Some(form);
    record.presented = true;
    Plan {
        declaration: Some(Declaration {
            form,
            declaring_class: record.declaring_class.clone(),
            interface: record.interface,
            member_flags: flags,
        }),
        record,
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
            without_flags.record().refusal.as_ref().map(|r| r.code),
            Some("jre_declaration_flags_missing")
        );
        assert!(
            without_flags
                .record()
                .refusal
                .as_ref()
                .is_some_and(|r| r.requirement.as_deref() == Some("the `access_flags` attribute"))
        );
        let without_class = plan(&MethodFacts::new("run", "()V", 1).with_access_flags(ACC_PUBLIC));
        assert_eq!(
            without_class.record().refusal.as_ref().map(|r| r.code),
            Some("jre_declaration_class_not_in_run")
        );
        assert!(!without_class.record().presented());
    }
}
