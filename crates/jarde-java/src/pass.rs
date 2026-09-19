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
//! the artifact as (every *structure* rule in this crate: a `while`, an `if` and a `switch` are the
//! same Java in every release). It is `Some(release)` for a rule whose output only exists from that
//! release on — and as of P3 2.1 there is one: [`LAMBDA`] presents `invokedynamic` call sites, a
//! Java 8 construct, so a profile that presents the artifact as Java 7 is refused it.[^lambda]
//!
//! [^lambda]: Until that slice the recorded gap was that `admits`'s refusing branch had no
//! production instance; `lambda@1` is the first rule that has one, and its tests check it through the
//! gate the run really reads (`crate::lambda` refuses a site the profile does not admit).
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
    /// The class's `BootstrapMethods` table, read with the same header.
    ///
    /// This is the table an `invokedynamic`'s bootstrap index resolves against, and it is the one
    /// fact from which "this site is a `LambdaMetafactory` call" can be read at all (A04). Unlike
    /// the two tables above it is *optional for a run*: a class that declares no such attribute
    /// hands over an empty table, and a site whose index is not in it is a refused *site*, not a
    /// refused run — nothing about a missing bootstrap stops the rest of a body from being
    /// presented. So the requirement is checked where a rule would claim a shape, through
    /// [`crate::region::FallbackReason::unmet`]'s own path, rather than by the run-level gate.
    BootstrapMethods,
    /// The class's **other members**, with their declarations and their decoded bodies (P3 2.2).
    ///
    /// A synthetic accessor is a different member of the same class, so the run that presents a
    /// caller's body can only decide what a call to one means if the caller hands that member over.
    /// Like the bootstrap table it is optional for a run and checked where a rule would claim a
    /// shape — a call site naming a member this table does not hold is a refused *site*
    /// ([`crate::accessor`]), not a refused run, and a body that calls nothing of the sort never
    /// consults it at all.
    Members,
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
            Self::BootstrapMethods => "bootstrap methods",
            Self::Members => "class members",
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
    /// Where the requirement is *checked* depends on what the table is needed for, and the
    /// declaration says which. A table nothing can be presented without — the canonical graph, the
    /// names, the decode — is checked once per run, before the walk: a payload that does not publish
    /// one stops the run with [`crate::stop::StopReason::IrTableMissing`]. A table only some rules
    /// need is checked **where the rule would claim a shape**, because a run does not stop being
    /// presentable just because the class happens to declare no such table: the class's
    /// `BootstrapMethods` table is the live instance ([`LAMBDA`]), and a site whose entry that table
    /// does not state is a refused *site* with the requirement named in its refusal
    /// ([`crate::lambda::LambdaRefusal::requirement`]).
    IrTable(IrTable),
    /// **Effect**: every instruction of the block a pass is about to write *inside* the structure it
    /// decides must be part of a value expression — a push, a load or an arithmetic — so that
    /// writing it in the structure's condition runs it exactly as often, and in the same order, as
    /// the bytecode ran it. A store, an increment, a call, a return or an operation this subset does
    /// not model is an effect with no place in the shape, and the shape is quoted instead.
    StatementFree,
    /// **Metadata**: an attribute- or declaration-derived fact the shape needs (a debug name, a
    /// line number, a member's access flags) before it could be written.
    ///
    /// **No rule of 1.3 requires metadata**, and that is a decision rather than an omission:
    /// a body compiled without debug information is presented with deterministic ordinal names
    /// (A10) instead of being refused, and the emitter never writes a source position beyond the BCI
    /// an origin carries, so no shape's precondition is a name or a line. The variant exists because
    /// the family is one of the three decision 1 names, and a rule that *did* require a
    /// `LineNumberTable` would have to state it here rather than assume it. P3 2.2 registers the
    /// first rule that does require one: the *bridge* rule needs the member's `access_flags`, which
    /// is a declaration fact of the same family — the caller read it with the class's header and the
    /// payload does not carry it. P3 2.3 adds three more of the same family, and they are why this
    /// variant is now the busiest one: `field@1` and `init@1` need the **declaring class** (the
    /// `Fieldref` JVMS 4.10.1.9 allows before a constructor call, and the class JVMS 4.9.2 compares
    /// the prologue's owner against), and `declaration@1` needs both facts to tell an interface's
    /// `default` method from a class's ordinary one.
    Metadata { attribute: &'static str },
    /// **Effect**: every value the shape *repeats* has to be one whose text can be written where the
    /// shape reads it without running anything again.
    ///
    /// A shape can move a value's text into a place that runs at a different time from the
    /// instruction that produced it — a captured argument written inside a lambda body is read when
    /// the lambda is *invoked*, not when the site that captured it ran. For a local read or a
    /// literal that is the same value twice and no effect; for a call, a field read or an arithmetic
    /// over one, the text would run a second time, or run later than it did. So the rule that decides
    /// such a shape states this requirement and checks it per operand, and a value that is not
    /// replayable makes the site a fallback rather than a moved effect.
    Replayable,
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
            Self::Replayable => {
                "a captured value whose text can be read again where the shape writes it"
                    .to_string()
            }
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

/// The pass that presents a verified `LambdaMetafactory` call site as a lambda or method reference.
///
/// This is the first rule of this build whose output **is** a Java 8 construct: an `invokedynamic`
/// call site exists from the class-file version that introduced it, so `Some(8)` here is what the
/// output needs rather than a promise about the input, and a profile that presents the artifact as
/// Java 7 does not admit it ([`Pass::admits`]).
///
/// What it requires is stated, not assumed:
///
/// * the run's **decode** and its **pool**, which is where the site's own name, descriptor and
///   bootstrap index live;
/// * the run's **bootstrap table**, which is the only thing that says whether the factory is
///   `java/lang/invoke/LambdaMetafactory` at all — a site whose entry is not in it is refused with
///   this requirement named, and a site whose entry names something else is refused as a shape;
/// * the run's **names** (SSA), because the captured values are read off the same value flow as
///   every other operand, and their BCIs are what the segment table records;
/// * [`Precondition::Replayable`] per captured value, because the capture's text is written where
///   the lambda reads it.
pub const LAMBDA: Pass = Pass::new(
    RuleVersion::new("lambda", "1"),
    Some(8),
    &[
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
        Precondition::IrTable(IrTable::ConstantPool),
        Precondition::IrTable(IrTable::BootstrapMethods),
        Precondition::Replayable,
    ],
);

/// The pass that presents a verified `StringBuilder`/`StringBuffer` chain as a concatenation.
///
/// The output of this rule — `a + b + c` — is Java in every release, so the rule is
/// release-independent ([`Pass::required_release`] is `None`): what changes across releases is the
/// *input*, and that is a fact the rule states itself rather than a gate it is admitted through.
/// The classes it accepts as the chain's receiver are exactly the two compilers wrote for string
/// concatenation, `java/lang/StringBuilder` (javac from release 5 on) and `java/lang/StringBuffer`
/// (the spelling before that, and one a source may still ask for); a chain on any other class is
/// not this shape and the rule does not claim it.
///
/// It requires the run's **names**, its **decode** and its **pool** — the chain is read out of the
/// value flow and the instructions, and the class it builds, the `append` overloads it calls and
/// the field it writes are symbol facts of the pool — plus [`Precondition::StatementFree`]: every
/// instruction *between* the chain's first and last one has to be part of a value expression. That
/// is the requirement that keeps the concatenation's own order: a store, a call statement or an
/// increment sitting between two `append`s would be written after the expression that now holds
/// them, and an effect is never moved by this rule.
pub const CONCAT: Pass = Pass::new(
    RuleVersion::new("concat", "1"),
    None,
    &[
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
        Precondition::IrTable(IrTable::ConstantPool),
        Precondition::StatementFree,
    ],
);

/// The pass that presents a bridge method's verified forward as the call it forwards.
///
/// A bridge is a member the class **declares** as one ([`jarde_java::MethodFacts::is_bridge`] over
/// the access flags the caller states), so this rule reads a declaration fact before it looks at a
/// body: the shape a body happens to have never makes a bridge. Its output — `return x.m(args)`
/// with the erased signature's own spelling — is Java in every release, hence `None`; bridges
/// themselves exist from release 5 on, which is a fact about the input.
///
/// It requires the run's names, decode and pool (the forwarded target is a pool symbol), and
/// [`Precondition::Metadata`] for the member's **access flags**: without them the run has no bridge
/// to present, and the rule says exactly that instead of inferring one from a body that looks like a
/// forward.
pub const BRIDGE: Pass = Pass::new(
    RuleVersion::new("bridge", "1"),
    None,
    &[
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
        Precondition::IrTable(IrTable::ConstantPool),
        Precondition::Metadata {
            attribute: "access_flags",
        },
    ],
);

/// The pass that presents a verified synthetic accessor call as a direct field access.
///
/// A synthetic accessor is a **different member** of the same class, so this rule reads the class's
/// own member list ([`IrTable::Members`]): the callee's declaration says whether it is `static` and
/// `synthetic`, and the callee's decoded body says whether it reads or writes one field and nothing
/// else. What the rule writes — `x.f`, or `x.f = v` — is Java in every release, hence `None`;
/// accessors themselves exist from the first release javac needed them.
///
/// A call site that names a member the run does not hold is refused with the *table* named
/// ([`crate::accessor`]), never presented from the call's own name: a member called `access$100`
/// that reads a field is not the reason to print a field access, the member's own verified body is.
pub const ACCESSOR: Pass = Pass::new(
    RuleVersion::new("accessor", "1"),
    None,
    &[
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
        Precondition::IrTable(IrTable::ConstantPool),
        Precondition::IrTable(IrTable::Members),
    ],
);

/// Every pass this build registers, in the order the design lists them.
pub const PASSES: [Pass; 13] = [
    STRAIGHT,
    IF,
    LOOP,
    SWITCH,
    LAMBDA,
    CONCAT,
    BRIDGE,
    ACCESSOR,
    NEW,
    FIELD,
    ENUMSWITCH,
    INIT,
    DECLARATION,
];

/// The pass that presents a verified allocation, its copy and its constructor call as `new T(…)`.
///
/// This is the rule that makes a **local, anonymous or inner class's use** presentable: such a class
/// is an ordinary class whose name the pool spells the way the compiler minted it (`p/Outer$1`), and
/// its instantiation is exactly this shape. What the rule does *not* read is the nesting relation —
/// `InnerClasses`, `EnclosingMethod` and the meaning of an enclosing-instance argument are
/// class-level facts, and one body's payload holds none of them (§3 of the slice notes) — so no
/// `Outer.this`, no nested `new Inner()` spelling and no synthetic field is invented. What the
/// source wrote as the enclosing instance is written as the value the site really read.
///
/// Its output — `new T(…)` — is Java in every release, hence `None`; allocations and inner classes
/// are not release questions.
///
/// It requires the run's names and decode (the instance's value flow and the instruction that
/// constructs it) and [`Precondition::StatementFree`]: nothing between the copy and the constructor
/// call may be an effect, because the arguments are written inside the `new` expression and an
/// effect written there would run at a different time.
pub const NEW: Pass = Pass::new(
    RuleVersion::new("new", "1"),
    None,
    &[
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
        Precondition::StatementFree,
    ],
);

/// The pass that presents a field instruction of the body itself as the field access it is.
///
/// A `getfield`/`getstatic`/`putfield`/`putstatic` becomes `receiver.f`, `Type.f`, `receiver.f = v`
/// or `Type.f = v`, and what makes that text mean the same member is the receiver's **stated type**:
/// for an instance access the frames must type the receiver exactly as the pool names the member's
/// owner, or `receiver.f` could name a field the instruction did not read (`B extends A` may declare
/// its own `f`). A static access has no receiver and no such question.
///
/// It requires the run's names and decode, plus [`Precondition::Metadata`] for the **declaring
/// class**: the one write whose receiver is not a typed value is the store an instance initializer
/// makes on its own uninitialized `this` before its constructor call — JVMS 4.10.1.9 allows exactly
/// the `Fieldref` that names the class being constructed, and a run that was not told which class
/// that is refuses those writes with this requirement named.
pub const FIELD: Pass = Pass::new(
    RuleVersion::new("field", "1"),
    None,
    &[
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
        Precondition::Metadata {
            attribute: "declaring_class",
        },
    ],
);

/// The pass that presents the dispatch-table read a compiler's enum `switch` performs.
///
/// The shape is a `getstatic` of a static `int[]` field indexed by the result of an `int`-valued
/// instance call: the read is written where it is consumed, which for this shape is the selector of
/// the `switch`. The rule states what it does **not** claim in its own module: the mapping from the
/// table's entries to enum constants is another class's declaration and the table's contents are
/// another class's initializer, so `switch (e)` with `case T.CONST:` labels is never written from
/// this run's facts — the table read the bytecode performs is ([`crate::enumswitch`]).
pub const ENUMSWITCH: Pass = Pass::new(
    RuleVersion::new("enumswitch", "1"),
    None,
    &[
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
    ],
);

/// The pass that presents a constructor's prologue as `super(…)` or `this(…)`.
///
/// The receiver is the frames' `UninitializedThis` — a token only a constructor's own `this` carries
/// before its constructor call — and JVMS 4.9.2 lets such a call name exactly the class that
/// declares the constructor or that class's direct superclass. Which of the two spellings it is
/// therefore comes down to one comparison, and the class it compares against is stated by the
/// caller: [`Precondition::Metadata`] for the **declaring class**. A run that was not told it
/// refuses the prologue with that requirement named rather than writing `super` for a `this` call,
/// which would run a different constructor.
///
/// It requires the run's names and decode as well: the receiver's type is what proves the call is a
/// prologue at all.
pub const INIT: Pass = Pass::new(
    RuleVersion::new("init", "1"),
    None,
    &[
        Precondition::IrTable(IrTable::Ssa),
        Precondition::IrTable(IrTable::Code),
        Precondition::Metadata {
            attribute: "declaring_class",
        },
    ],
);

/// The pass that states the presented member's declaration in the artifact's envelope.
///
/// A `default` method is not a flag of its own (JVMS 4.6): it is an interface's method that is
/// neither `static` nor `abstract`, so reading it takes both the member's own flags and the class's
/// own `ACC_INTERFACE` — and one body's payload holds neither. Both are caller-stated declaration
/// facts, and the rule refuses, with the missing one named, instead of guessing one from the other:
/// a `public` method that is not abstract is a `default` method in an interface and an ordinary
/// method in a class.
pub const DECLARATION: Pass = Pass::new(
    RuleVersion::new("declaration", "1"),
    None,
    &[
        Precondition::Metadata {
            attribute: "access_flags",
        },
        Precondition::Metadata {
            attribute: "declaring_class",
        },
    ],
);

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
        assert_eq!(PASSES.len(), 13);
        assert_eq!(
            PASSES
                .iter()
                .map(|pass| pass.rule().citation())
                .collect::<Vec<_>>(),
            vec![
                "straight@1".to_string(),
                "if@1".to_string(),
                "loop@1".to_string(),
                "switch@1".to_string(),
                "lambda@1".to_string(),
                "concat@1".to_string(),
                "bridge@1".to_string(),
                "accessor@1".to_string(),
                "new@1".to_string(),
                "field@1".to_string(),
                "enumswitch@1".to_string(),
                "init@1".to_string(),
                "declaration@1".to_string()
            ]
        );
        // Which rules are release-independent and which one is not (P3 2.1). Until then this loop
        // asserted `None` for **every** registered pass and recorded in its comment that the gate
        // had no production instance; `lambda@1` is that instance, so the assertion now pins each
        // rule's own requirement instead of pinning that none has one.
        for pass in PASSES {
            match pass.rule().rule() {
                "lambda" => assert_eq!(
                    pass.required_release(),
                    Some(8),
                    "{pass:?} presents `invokedynamic`, which is a Java 8 construct"
                ),
                // The three pattern rules of P3 2.2 are release-independent for the reason their
                // own documentation states: what they *write* (`a + b`, `x.f = v`, `return x.m()`)
                // is Java in every release, and the shapes they *read* are stated by the rule itself
                // (the two concatenation classes, the declared bridge flag, the synthetic accessor's
                // body) rather than through the profile gate.
                "concat" | "bridge" | "accessor" => assert_eq!(
                    pass.required_release(),
                    None,
                    "{pass:?} writes a construct every release spells the same way"
                ),
                // P3 2.3's rules are release-independent for the same reason: `new T(…)`, `x.f`,
                // `x.f = v`, `super(…)`, a dispatch-table read and a declaration line are Java in
                // every release. What they *read* is an input fact — an allocation, a field
                // instruction, a declared interface — and each rule states its own shape.
                "new" | "field" | "enumswitch" | "init" | "declaration" => assert_eq!(
                    pass.required_release(),
                    None,
                    "{pass:?} writes a construct every release spells the same way"
                ),
                _ => assert_eq!(
                    pass.required_release(),
                    None,
                    "{pass:?} is a structure rule: the same Java in every release"
                ),
            }
            assert!(
                !pass.requires(Precondition::Metadata {
                    attribute: "LocalVariableTable"
                }),
                "{pass:?} does not require debug metadata: a body without it is named, not refused"
            );
        }
        // The effect preconditions: the loop pass and the concatenation pass state `StatementFree`
        // (each for its own block-shaped reason), and the lambda rule states `Replayable` for the
        // values it captures; no other rule states either.
        assert!(LOOP.requires(Precondition::StatementFree));
        assert!(CONCAT.requires(Precondition::StatementFree));
        assert!(!IF.requires(Precondition::StatementFree));
        assert!(!SWITCH.requires(Precondition::StatementFree));
        assert!(!STRAIGHT.requires(Precondition::StatementFree));
        assert!(!BRIDGE.requires(Precondition::StatementFree));
        assert!(!ACCESSOR.requires(Precondition::StatementFree));
        // The construction site of P3 2.3 states it too, and for the same reason the concatenation
        // does: its arguments are written *inside* the `new` expression, so an effect between the
        // copy and the constructor call would have to move.
        assert!(NEW.requires(Precondition::StatementFree));
        assert!(!FIELD.requires(Precondition::StatementFree));
        assert!(!ENUMSWITCH.requires(Precondition::StatementFree));
        assert!(!INIT.requires(Precondition::StatementFree));
        assert!(!DECLARATION.requires(Precondition::StatementFree));
        assert!(LAMBDA.requires(Precondition::Replayable));
        assert!(!LOOP.requires(Precondition::Replayable));
        assert!(!CONCAT.requires(Precondition::Replayable));
        assert!(!LAMBDA.requires(Precondition::StatementFree));
        // The declaration facts P3 2.3 reads: the class that declares the body — which two rules
        // need to compare a class name against one the bytes state (`field@1` for the write JVMS
        // 4.10.1.9 allows before a constructor call, `init@1` for the call JVMS 4.9.2 lets an
        // instance initializer make) — and the member's own flags, which only the declaration rule
        // reads.
        assert!(FIELD.requires(Precondition::Metadata {
            attribute: "declaring_class"
        }));
        assert!(INIT.requires(Precondition::Metadata {
            attribute: "declaring_class"
        }));
        assert!(DECLARATION.requires(Precondition::Metadata {
            attribute: "access_flags"
        }));
        assert!(DECLARATION.requires(Precondition::Metadata {
            attribute: "declaring_class"
        }));
        for pass in [
            STRAIGHT, IF, LOOP, SWITCH, LAMBDA, CONCAT, BRIDGE, ACCESSOR, NEW, ENUMSWITCH,
        ] {
            assert!(
                !pass.requires(Precondition::Metadata {
                    attribute: "declaring_class"
                }),
                "{pass:?} compares no class name"
            );
        }
        // The IR preconditions a pass may state name the tables that exist, and the lambda rule is
        // the one that reads the class's bootstrap table beside the decode it reads through. That
        // table is checked where a site is claimed rather than by the run-level gate — a class with
        // no bootstrap table is an ordinary class whose other regions are presented as usual — which
        // is stated in `Precondition::IrTable`'s own documentation and in the refusal this build
        // produces for a site whose entry the table does not hold.
        for pass in PASSES {
            for requirement in pass.preconditions() {
                if let Precondition::IrTable(table) = requirement {
                    assert!(
                        matches!(
                            table,
                            IrTable::Canonical
                                | IrTable::Ssa
                                | IrTable::Code
                                | IrTable::ConstantPool
                                | IrTable::BootstrapMethods
                                | IrTable::Members
                        ),
                        "{pass:?} states a table that is not one of this build's"
                    );
                }
            }
        }
        assert!(LAMBDA.requires(Precondition::IrTable(IrTable::BootstrapMethods)));
        assert!(LAMBDA.requires(Precondition::IrTable(IrTable::ConstantPool)));
        assert!(ACCESSOR.requires(Precondition::IrTable(IrTable::Members)));
        assert!(BRIDGE.requires(Precondition::Metadata {
            attribute: "access_flags"
        }));
        assert_eq!(IrTable::Members.name(), "class members");
        for pass in [
            STRAIGHT,
            IF,
            LOOP,
            SWITCH,
            CONCAT,
            BRIDGE,
            NEW,
            FIELD,
            ENUMSWITCH,
            INIT,
            DECLARATION,
        ] {
            assert!(!pass.requires(Precondition::IrTable(IrTable::BootstrapMethods)));
            assert!(!pass.requires(Precondition::IrTable(IrTable::Members)));
        }
        assert_eq!(pass("loop"), Some(LOOP));
        assert_eq!(pass("lambda"), Some(LAMBDA));
        assert_eq!(pass("concat"), Some(CONCAT));
        assert_eq!(pass("bridge"), Some(BRIDGE));
        assert_eq!(pass("accessor"), Some(ACCESSOR));
        assert_eq!(pass("new"), Some(NEW));
        assert_eq!(pass("field"), Some(FIELD));
        assert_eq!(pass("enumswitch"), Some(ENUMSWITCH));
        assert_eq!(pass("init"), Some(INIT));
        assert_eq!(pass("declaration"), Some(DECLARATION));
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
