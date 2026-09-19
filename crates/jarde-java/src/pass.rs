//! The rule book of the recovery layer: which rule produced a shape, which version of it, what it
//! requires before it may claim one, and which profile admits it (P3 decision 1).
//!
//! # Registration is compile-time, and there is no trait
//!
//! P3 decision 1 asks every pattern pass to *declare* the IR/effect/metadata it reads, the nodes it
//! produces, its rule version and its failure fallback. The declaration is this module's table: a
//! `const` per pass, in one `const` list, readable by the report and by the diagnostic that states
//! a refusal. There is no `trait Backend`, no dynamic registration and no pass registry object —
//! P3's design defers those to the second consumer, and the shapes below are `const` data a reader
//! can check against the code that consults them.
//!
//! # The recovery profile is the fact layer's own type
//!
//! The P3 spec's sentence is "在 Java 8 RuntimeProfile 下" — it *names* an existing type, and that
//! type already carries the release a gate needs. So [`RecoveryProfile`] is a re-export of
//! [`jarde_reader::view::RuntimeProfile`], not a second enum: a `RecoveryProfile::Java8` beside
//! `RuntimeProfile { java_release: 8, … }` would be two spellings of one fact, and the two would
//! drift the first time either moved. What the recovery layer adds is the *use* — the release a pass
//! requires, and the predicate [`Pass::admits`] the gate reads — not the vocabulary.
//!
//! A pass's `required_release` is `None` when the rule is independent of what the profile presents
//! the artifact as (every structure rule in this slice: a `while`, an `if` and a `switch` are the
//! same Java in every release). It is `Some(release)` for a rule whose output only exists from that
//! release on — the Java 8 presentation rules of 2.x (lambda, string concatenation, try-with-
//! resources) are the first passes that will carry `Some(8)`, and until one exists no profile can
//! be refused a pass. That is recorded rather than faked: this slice ships the gate, the
//! classification and its contract, not a made-up pass to trip it.
//!
//! # Preconditions fail as fallbacks; missing tables stop the run
//!
//! A declared [`Precondition`] that is not met **at the point a pass would claim a shape** is a
//! fallback: the shape is quoted as bytecode, and the report and the diagnostics state which rule
//! was not applied and what it required (see [`crate::region::FallbackReason`]). A table the run
//! never had is not a fallback but a stop ([`crate::stop::StopReason::IrTableMissing`]): with no
//! graph, no SSA names and no decode there is nothing to quote either, and a run that cannot
//! present *anything* must not look like a run that presented nothing.

use std::fmt;

use serde::Serialize;

/// The recovery profile a run is presented under.
///
/// This is [`jarde_reader::view::RuntimeProfile`] — the same type the request's environment states
/// as its runtime view. It is re-exported under the name the recovery side's own vocabulary uses
/// for it, so that a reader of a recovery report does not have to know that the profile it names
/// also decides multi-release selection and layout.
pub use jarde_reader::view::RuntimeProfile as RecoveryProfile;

/// The profile this build's promise is written for: the Java 8 target of P3.
///
/// A caller that has a runtime view states its own profile (the entry point hands the request's
/// environment profile to the gate); this constant is what "the Java 8 recovery profile" means
/// where a test or a documentation sentence has to name one value.
pub const JAVA_8: RecoveryProfile = RecoveryProfile {
    java_release: 8,
    multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
    layout: jarde_reader::view::LayoutMode::Generic,
};

/// One table of the same run's payload that a pass reads.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IrTable {
    /// The canonical graph: blocks, edges, throw sites and handler rows.
    Canonical,
    /// The frames of the same run.
    Frames,
    /// The SSA names and effects of the same run.
    Ssa,
    /// The decode of the same run: instructions, typed operands and the exception table.
    Code,
    /// The constant pool read with the same header.
    ConstantPool,
}

impl IrTable {
    /// The name this table is stated by, in the vocabulary the stop reason already uses.
    pub fn name(self) -> &'static str {
        match self {
            Self::Canonical => "canonical",
            Self::Frames => "frames",
            Self::Ssa => "ssa",
            Self::Code => "code",
            Self::ConstantPool => "constant pool",
        }
    }
}

/// One thing a pass needs before it may claim a shape.
///
/// The three families are P3 decision 1's own list — the IR it reads, the effects its output has to
/// preserve, and the metadata it would have to name — typed so that a refusal can state which one
/// fell short instead of failing generically.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Precondition {
    /// **IR**: the pass reads this table of the run's payload.
    ///
    /// Checked once per run, before the walk: a payload that does not publish the table stops the
    /// run with [`crate::stop::StopReason::IrTableMissing`], because nothing can be presented at
    /// all without it.
    IrTable(IrTable),
    /// **Effect**: every instruction of the block a pass is about to write *inside* the structure it
    /// decides must be part of a value expression — a push, a load or an arithmetic — so that
    /// writing it in the structure's condition runs it exactly as often, and in the same order, as
    /// the bytecode ran it. A store, an increment, a call, a return or an operation this subset does
    /// not model is an effect with no place in the shape, and the shape is quoted instead.
    StatementFree,
    /// **Metadata**: an attribute-derived fact the shape needs (a debug name, a line number) before
    /// it could be written.
    ///
    /// **No rule in this slice requires metadata**, and that is a decision rather than an omission:
    /// a body compiled without debug information is presented with deterministic ordinal names
    /// (A10) instead of being refused, and the emitter never writes a source position beyond the BCI
    /// an origin carries, so no shape's precondition is a name or a line. The variant exists because
    /// the family is one of the three decision 1 names, and a rule that *did* require a
    /// `LineNumberTable` would have to state it here rather than assume it.
    Metadata { attribute: &'static str },
}

impl Precondition {
    /// What this requirement asks for, in one phrase, for a diagnostic's message.
    pub fn describe(self) -> String {
        match self {
            Self::IrTable(table) => format!("the `{}` table of this run", table.name()),
            Self::StatementFree => {
                "a test block whose every instruction is part of a value expression".to_string()
            }
            Self::Metadata { attribute } => format!("the `{attribute}` attribute"),
        }
    }
}

/// The version of one recovery rule.
///
/// A rule's *name* says which shape it presents; its *version* says which judgement about that
/// shape the output came from, so that a report or a diagnostic can be read back to the rule that
/// produced it. The version is a rule's own promise and changes when the rule's output changes —
/// not when the code around it does.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RuleVersion {
    rule: &'static str,
    version: &'static str,
}

impl RuleVersion {
    /// The version of one rule.
    pub const fn new(rule: &'static str, version: &'static str) -> Self {
        Self { rule, version }
    }

    /// The rule's name.
    pub const fn rule(self) -> &'static str {
        self.rule
    }

    /// The rule's version.
    pub const fn version(self) -> &'static str {
        self.version
    }

    /// `rule@version`: the one spelling a report, a diagnostic and a comment share.
    pub fn citation(self) -> String {
        format!("{}@{}", self.rule, self.version)
    }
}

impl fmt::Display for RuleVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}@{}", self.rule, self.version)
    }
}

/// One recovery pass as this build registers it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pass {
    rule: RuleVersion,
    required_release: Option<u16>,
    preconditions: &'static [Precondition],
}

impl Pass {
    /// One pass's declaration: its rule version, the release its output needs (if any) and the
    /// preconditions it states.
    pub const fn new(
        rule: RuleVersion,
        required_release: Option<u16>,
        preconditions: &'static [Precondition],
    ) -> Self {
        Self {
            rule,
            required_release,
            preconditions,
        }
    }

    /// The rule and version this pass presents shapes under.
    pub const fn rule(self) -> RuleVersion {
        self.rule
    }

    /// The release this pass's output needs, or `None` when the rule is release-independent.
    pub const fn required_release(self) -> Option<u16> {
        self.required_release
    }

    /// What this pass needs before it may claim a shape.
    pub const fn preconditions(self) -> &'static [Precondition] {
        self.preconditions
    }

    /// Whether this profile admits the pass.
    ///
    /// Every registered pass of this slice is release-independent (see the module documentation), so
    /// this predicate has no refusing instance in production yet; it is the gate the Java 8
    /// presentation rules of 2.x will be admitted through, and its contract is stated by this
    /// module's own tests rather than by a fabricated pass.
    pub fn admits(self, profile: &RecoveryProfile) -> bool {
        match self.required_release {
            None => true,
            Some(release) => profile.java_release >= release,
        }
    }

    /// Whether this pass states this precondition.
    pub fn requires(self, requirement: Precondition) -> bool {
        self.preconditions.contains(&requirement)
    }
}

/// The pass that presents a straight run of blocks.
pub const STRAIGHT: Pass = Pass::new(
    RuleVersion::new("straight", "1"),
    None,
    &[
        Precondition::IrTable(IrTable::Canonical),
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
    ],
);

/// The pass that presents a two-way branch as `if`/`else`.
pub const IF: Pass = Pass::new(
    RuleVersion::new("if", "1"),
    None,
    &[
        Precondition::IrTable(IrTable::Canonical),
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
    ],
);

/// The pass that presents a natural loop as `while`/`do … while`.
pub const LOOP: Pass = Pass::new(
    RuleVersion::new("loop", "1"),
    None,
    &[
        Precondition::IrTable(IrTable::Canonical),
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
        Precondition::StatementFree,
    ],
);

/// The pass that presents a `tableswitch`/`lookupswitch` as a `switch`.
pub const SWITCH: Pass = Pass::new(
    RuleVersion::new("switch", "1"),
    None,
    &[
        Precondition::IrTable(IrTable::Canonical),
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
    ],
);

/// Every pass this build registers, in the order the design lists them.
pub const PASSES: [Pass; 4] = [STRAIGHT, IF, LOOP, SWITCH];

/// The registered pass with this rule name, when there is one.
pub fn pass(rule: &str) -> Option<Pass> {
    PASSES
        .iter()
        .copied()
        .find(|pass| pass.rule().rule() == rule)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_profile_is_the_fact_layers_own_type_and_the_gate_reads_its_release() {
        // The one thing this module must not do is spell the profile twice. `RecoveryProfile` *is*
        // `RuntimeProfile`: a value built as one is the other, so the gate cannot end up reading a
        // release the request did not state.
        let profile: RecoveryProfile = JAVA_8;
        assert_eq!(profile.java_release, 8);
        let as_fact_layer: jarde_reader::view::RuntimeProfile = profile;
        assert_eq!(as_fact_layer.java_release, 8);

        let java8_only = Pass::new(RuleVersion::new("lambda", "1"), Some(8), &[]);
        assert!(java8_only.admits(&JAVA_8), "release 8 admits a Java 8 rule");
        assert!(
            java8_only.admits(&RecoveryProfile {
                java_release: 9,
                ..JAVA_8
            }),
            "a later release presents Java 8 shapes too"
        );
        assert!(
            !java8_only.admits(&RecoveryProfile {
                java_release: 7,
                ..JAVA_8
            }),
            "a profile that presents Java 7 does not admit a rule whose output is Java 8"
        );
        assert!(
            STRAIGHT.admits(&RecoveryProfile {
                java_release: 7,
                ..JAVA_8
            }),
            "a structure rule is release-independent"
        );
    }

    #[test]
    fn the_registered_table_states_each_rule_and_its_preconditions() {
        assert_eq!(PASSES.len(), 4);
        assert_eq!(
            PASSES
                .iter()
                .map(|pass| pass.rule().citation())
                .collect::<Vec<_>>(),
            vec![
                "straight@1".to_string(),
                "if@1".to_string(),
                "loop@1".to_string(),
                "switch@1".to_string()
            ]
        );
        // Every registered rule of this slice is release-independent: the Java 8 presentation rules
        // are 2.x's, and a `Some(8)` entry here would be a pass that does not exist.
        for pass in PASSES {
            assert_eq!(pass.required_release(), None, "{pass:?}");
            assert!(
                !pass.requires(Precondition::Metadata {
                    attribute: "LocalVariableTable"
                }),
                "{pass:?} does not require debug metadata: a body without it is named, not refused"
            );
        }
        // The loop pass is the one that states the effect precondition, and it is the precondition
        // this slice checks at the pass site (`region`'s test-block rule).
        assert!(LOOP.requires(Precondition::StatementFree));
        assert!(!IF.requires(Precondition::StatementFree));
        assert!(!SWITCH.requires(Precondition::StatementFree));
        assert!(!STRAIGHT.requires(Precondition::StatementFree));
        // The IR preconditions a structure pass states are exactly the tables the run checks before
        // the walk: a pass that read a table the run never demanded would be a declaration the run
        // does not honour.
        for pass in PASSES {
            for requirement in pass.preconditions() {
                if let Precondition::IrTable(table) = requirement {
                    assert!(matches!(
                        table,
                        IrTable::Canonical | IrTable::Ssa | IrTable::Code
                    ));
                }
            }
        }
        assert_eq!(pass("loop"), Some(LOOP));
        assert_eq!(pass("lambda"), None);
    }

    #[test]
    fn a_requirement_says_what_it_asks_for_and_a_rule_version_says_who_asked() {
        assert_eq!(
            Precondition::IrTable(IrTable::Ssa).describe(),
            "the `ssa` table of this run"
        );
        assert!(
            Precondition::StatementFree
                .describe()
                .contains("value expression")
        );
        assert_eq!(
            Precondition::Metadata {
                attribute: "LineNumberTable"
            }
            .describe(),
            "the `LineNumberTable` attribute"
        );
        assert_eq!(LOOP.rule().citation(), "loop@1");
        assert_eq!(LOOP.rule().to_string(), "loop@1");
        assert_eq!(LOOP.rule().rule(), "loop");
        assert_eq!(LOOP.rule().version(), "1");
    }
}
