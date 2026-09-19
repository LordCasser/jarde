//! The one shape a pattern rule states its verdict in: a *refusal* names the link that failed, the
//! rule that refused and — when a declared precondition fell short — the requirement itself.
//!
//! P3 2.1 wrote this shape for the lambda rule; 2.2's three rules (`concat@1`, `bridge@1`,
//! `accessor@1`) use the same one rather than a parallel mechanism, because the property P3
//! decision 1 asks for is the same for all four: a shape that was *not* presented is as readable
//! as one that was, and a requirement that was checked without being declared fails this build's
//! tests instead of drifting away from the declaration a reader consults.
//!
//! What is per-rule is only the **diagnostic code** a refusal under a given requirement is reported
//! under ([`requirement_code`]), because a code is a rule's own vocabulary: `jre_lambda_no_bootstrap`
//! and `jre_accessor_members_missing` both say "a table this rule needs is not in this run", and a
//! reader of the artifact wants to know which rule said it.

use crate::pass::{IrTable, Pass, Precondition};

/// Why one shape was not presented.
#[derive(Clone)]
pub(crate) struct Refusal {
    code: &'static str,
    requirement: Option<Precondition>,
    message: String,
}

impl Refusal {
    /// A refusal of a shape that simply is not the one the rule verifies.
    pub(crate) fn shape(code: &'static str, message: String) -> Self {
        Self {
            code,
            requirement: None,
            message,
        }
    }

    /// A refusal under a **declared** precondition of the rule that consulted it.
    ///
    /// The debug assertion is the one every rule of this layer carries: a requirement that is
    /// checked without being declared — or declared and never checked — fails this build's tests
    /// instead of drifting away from the declaration a reader consults.
    pub(crate) fn unmet(pass: &'static Pass, requirement: Precondition, message: String) -> Self {
        debug_assert!(
            pass.requires(requirement),
            "{} states no {requirement:?} precondition",
            pass.rule()
        );
        Self {
            code: requirement_code(pass.rule().rule(), requirement),
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

/// The code one rule's refusal under one declared requirement is reported with.
pub(crate) fn requirement_code(rule: &str, requirement: Precondition) -> &'static str {
    match (rule, requirement) {
        ("lambda", Precondition::IrTable(IrTable::BootstrapMethods)) => "jre_lambda_no_bootstrap",
        ("lambda", Precondition::Replayable) => "jre_lambda_capture_not_replayable",
        ("lambda", _) => "jre_lambda_unmet_precondition",
        ("concat", Precondition::StatementFree) => "jre_concat_interleaved_effect",
        ("concat", _) => "jre_concat_unmet_precondition",
        ("accessor", Precondition::IrTable(IrTable::Members)) => "jre_accessor_members_missing",
        ("accessor", _) => "jre_accessor_unmet_precondition",
        ("bridge", Precondition::Metadata { .. }) => "jre_bridge_flags_missing",
        ("bridge", _) => "jre_bridge_unmet_precondition",
        // P3 2.3's four rules. `new@1` states the effect requirement the concatenation states (its
        // arguments are written inside the `new` expression); `field@1` and `init@1` state the one
        // declaration fact they compare a class name against; `enumswitch@1` states no requirement of
        // its own — a read either is the dispatch-table shape or it is not.
        ("new", Precondition::StatementFree) => "jre_new_interleaved_effect",
        ("new", _) => "jre_new_unmet_precondition",
        ("field", Precondition::Metadata { .. }) => "jre_field_declaring_class_missing",
        ("field", _) => "jre_field_unmet_precondition",
        ("enumswitch", _) => "jre_enumswitch_unmet_precondition",
        ("init", Precondition::Metadata { .. }) => "jre_init_class_not_in_run",
        ("init", _) => "jre_init_unmet_precondition",
        (
            "declaration",
            Precondition::Metadata {
                attribute: "access_flags",
            },
        ) => "jre_declaration_flags_missing",
        ("declaration", Precondition::Metadata { .. }) => "jre_declaration_class_not_in_run",
        ("declaration", _) => "jre_declaration_unmet_precondition",
        _ => "jre_unmet_precondition",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pass::{ACCESSOR, BRIDGE, CONCAT, LAMBDA};

    #[test]
    fn a_refusal_under_a_requirement_carries_the_rules_own_code() {
        let message = "the shape was not claimed".to_string();
        let lambda = Refusal::unmet(
            &LAMBDA,
            Precondition::IrTable(IrTable::BootstrapMethods),
            message.clone(),
        );
        assert_eq!(lambda.code(), "jre_lambda_no_bootstrap");
        assert_eq!(
            lambda.requirement(),
            Some(Precondition::IrTable(IrTable::BootstrapMethods))
        );
        assert_eq!(lambda.message(), "the shape was not claimed");

        assert_eq!(
            Refusal::unmet(&CONCAT, Precondition::StatementFree, message.clone()).code(),
            "jre_concat_interleaved_effect"
        );
        assert_eq!(
            Refusal::unmet(
                &ACCESSOR,
                Precondition::IrTable(IrTable::Members),
                message.clone()
            )
            .code(),
            "jre_accessor_members_missing"
        );
        assert_eq!(
            Refusal::unmet(
                &BRIDGE,
                Precondition::Metadata {
                    attribute: "access_flags"
                },
                message
            )
            .code(),
            "jre_bridge_flags_missing"
        );
        let shape = Refusal::shape("jre_concat_shape", "not this shape".to_string());
        assert_eq!(shape.requirement(), None);
        assert_eq!(shape.code(), "jre_concat_shape");
    }

    #[test]
    #[should_panic(expected = "states no")]
    fn a_requirement_a_rule_never_declared_cannot_be_refused_under() {
        // The declaration and the check site cannot drift: refusing under a requirement the rule
        // does not state fails here (in a debug build) instead of being reported as if declared.
        let _ = Refusal::unmet(
            &CONCAT,
            Precondition::Replayable,
            "not declared".to_string(),
        );
    }
}
