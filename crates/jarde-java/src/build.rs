//! ③→④ The AST builder: turns the region tree plus the IR's value flow plus the decoded facts into
//! [`Stmt`]s, without writing a single byte of text.
//!
//! # The rule that keeps this layer honest
//!
//! A statement is built only where every piece of its text is *evidence*:
//!
//! * the shape comes from the region ([`crate::region`]) — a straight run, an `if`, or a fallback;
//! * the operands come from the IR and the decoded operations: a value whose definition is an
//!   instruction is rendered from that instruction's [`Operation`], a value described by a frame
//!   entry or a phi of a local slot is rendered as that local's name (both arms assigned it, so the
//!   join reads it by name), and anything else — a caught exception reference, a stack value no
//!   instruction produced, an operation this subset does not model, a receiver that cannot be
//!   rendered — makes the statement it belongs to a [`StmtKind::Fallback`] that quotes its bytecode.
//!
//! Every `None`, `Err` and `fallback` below is therefore a *stated* one: the artifact says "this BCI
//! is bytecode this layer cannot present" instead of printing a guess, and the report counts it (a
//! run with any fallback is `Mixed`/`Fallback`, never `Java`/`Structured`).
//!
//! # Which instruction becomes a statement
//!
//! * [`Operation::Store`], [`Operation::Invoke`] and [`Operation::Return`] are statements: they are
//!   what the bytecode does to the program's state.
//! * [`Operation::Push`], [`Operation::Load`], [`Operation::Arithmetic`], [`Operation::Shift`], [`Operation::Bitwise`]
//!   and [`Operation::Negate`] are not: they produce a value whose text lands where the value is
//!   consumed, and a load whose value nobody consumes has no Java effect either.
//! * [`Operation::Comparison`] is not: the branch that tests it prints it — as an `if`'s condition
//!   (the *negation* of the jump sense, because a branch transfers only when its sense holds), or as
//!   part of the quoted bytecode when the region could not be structured.
//! * [`Operation::Other`], and an instruction this run decoded no operation for, are *stated* as
//!   unusable: neither is dropped silently, because a silently dropped instruction is exactly the
//!   failure a presentation must not have.
//!
//! The array instructions of P3 2b are on both sides of that line at once, and the operand decides:
//! an element **write** is a statement of its own (`array[index] = value;`), while a read, a length
//! read and a creation produce a value whose text lands where the value is consumed — with the one
//! question each of them asks of the body, whether some reader really writes that value, because an
//! instruction nothing reads would be an effect silently dropped.

use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use jarde_jvm::method_ir::{
    CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind, Definition, PhiInput, RefType, Slot,
    SsaInstruction, SsaTable, SsaValue, Value, ValueId,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{
    BootstrapMethodFacts, CpEntryFacts, DescriptorKind, MethodCodeFacts, cp_class_name,
    descriptor_facts,
};

use crate::accessor::{self, AccessorRecord, AccessorRefusal, AccessorShape};
use crate::ast::{
    AssignOp, BinaryOp, ConcatPart, ConstructorTarget, Expr, ExprKind, LambdaParam, ResourceDecl,
    Stmt, StmtKind, SwitchArm, SwitchLabels, Type,
};
use crate::bridge;
use crate::concat;
use crate::decode::Operations;
use crate::enumswitch;
use crate::evidence::Publication;
use crate::facts::ACC_PRIVATE;
use crate::facts::{
    ArithmeticOp, BitwiseOp, CallTarget, ClassMembers, CompareOp, ConstantValue, DynamicSite,
    FieldAccess, InvokeKind, NumericComparisonOp, Operation, ShiftOp, SpecialValuePresentation,
};
use crate::field;
use crate::guard;
use crate::init;
use crate::lambda::{
    self, LambdaCapture, LambdaForm, LambdaRecord, LambdaRefusal, Reach, Refusal, TypeConversion,
};
use crate::names::{LocalVariable, NameTable, RenderedName, is_java_identifier};
use crate::pass::{LAMBDA, Precondition, RecoveryProfile};
use crate::refusal::Gap;
use crate::region::{
    CatchClause, Continuation, FallbackReason, ForHeader, LoopForm, Region, SwitchGroup,
};
use crate::reuse;
use crate::source_map::{Origin, OriginSet};
use crate::stop::{StopReason, charge, poll};

fn loop_label(header_bci: u32) -> String {
    format!("jarde_loop_{header_bci}")
}

/// How deep a value expression may nest before the builder refuses it.
///
/// Bytecode nests as deeply as the source expression did, and a source expression is bounded by the
/// source; the bound exists so that a pathological body cannot make this layer recurse without
/// limit. Twenty-four levels is far above what the corpus holds and far below a stack this process
/// cannot afford.
pub(crate) const MAX_VALUE_DEPTH: usize = 24;

/// The statements of one method, in method order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Program {
    /// The body, in method order.
    pub(crate) stmts: Vec<Stmt>,
    /// Proved legacy field updates, retained only to check their final `Return` text.
    pub(crate) field_increments: BTreeMap<u32, LegacyFieldIncrement>,
    /// How many statements the build produced.
    pub(crate) statements: usize,
    /// Whether any statement is a fallback: the run's representation and quality read this.
    pub(crate) ragged: bool,
    /// Every `invokedynamic` site of this body, in BCI order, with what the class states about it
    /// and what this build did with it (P3 2.1). A site is here whether it was presented or refused,
    /// because "which bootstrap was it, and why was it not a lambda" is the question A04 asks and a
    /// record that only listed the presented ones could not answer it.
    ///
    /// What is here is the **decision**, not the owning record: `lambda@1`'s record is materialized
    /// from these sites by [`Program::materialize_lambdas`] after the artifact is committed, and only
    /// when the request selected `RuleDetails` (change `add-demand-driven-core-results`, D3).
    pub(crate) lambdas: Vec<LambdaSite>,
    /// Every synthetic accessor call site of this body, in BCI order, with what the rule read about
    /// its callee and what this build did with it (P3 2.2, A12). As for a lambda, a refusal is part
    /// of the answer: a call that kept the call it had says which link of the verification failed.
    pub(crate) accessors: Vec<AccessorSite>,
    /// Every dynamic site this build did not present, with the refusal that says which link failed.
    /// A gap is not the optional evidence: it is what every selection reports about this rule.
    pub(crate) lambda_refusals: Vec<Gap>,
    /// Every accessor call site this build did not present, with the refusal that says which link
    /// failed.
    pub(crate) accessor_refusals: Vec<Gap>,
    /// How many dynamic sites this build presented, and how many accessor call sites. The counts
    /// describe the rule's work over the whole body, so the report states them whatever the request
    /// selected — which is what makes the summary lines of two selections say the same thing.
    pub(crate) lambdas_presented: u64,
    pub(crate) accessors_presented: u64,
}

/// The two real field instructions behind a legacy `Local("field++")` return.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LegacyFieldIncrement {
    pub(crate) read: u32,
    pub(crate) write: u32,
    pub(crate) returns: u32,
}

/// One dynamic site's decision, as this build made it (change `add-demand-driven-core-results`, D3).
///
/// This is the **plan** side of `lambda@1`'s record: the site's own coordinates, the rule's verdict
/// (the bootstrap evidence, the form it took and the refusal when it did not) and the captures the
/// record states. No [`LambdaRecord`] is built while the body is decided; the records are written
/// from these sites by [`Program::materialize_lambdas`], one per charge, after the artifact is
/// committed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LambdaSite {
    /// The BCI of the `invokedynamic` instruction.
    pub(crate) use_site: u32,
    /// The constant-pool index of the site's own `InvokeDynamic` entry.
    pub(crate) site_cp: u16,
    /// The `BootstrapMethods` entry the site names.
    pub(crate) bootstrap_index: u16,
    /// The name the site presents.
    pub(crate) sam_name: String,
    /// The descriptor the site presents.
    pub(crate) sam_descriptor: String,
    /// The bootstrap evidence the rule read.
    pub(crate) evidence: lambda::Evidence,
    /// Which writing the site took; `None` when it was refused.
    pub(crate) form: Option<LambdaForm>,
    /// Why the site was not presented, when it was not.
    pub(crate) refusal: Option<LambdaRefusal>,
    /// Every value the site captures, in the order the instruction reads it off the stack.
    pub(crate) captures: Vec<LambdaCapture>,
}

impl LambdaSite {
    /// Every driver BCI this site states: its own instruction and every capture it reads. The
    /// positions a driver range selects on — the site is kept or dropped as a unit with them, and
    /// the captures a kept site states are part of its own closure.
    fn positions(&self) -> Vec<u32> {
        let mut positions = vec![self.use_site];
        positions.extend(self.captures.iter().filter_map(|capture| capture.bci));
        positions
    }

    /// The owning record, built here and only here.
    fn record(&self) -> LambdaRecord {
        crate::demand_counts::record_built(crate::evidence::RecoveryEvidenceKind::RuleDetails);
        LambdaRecord {
            use_site: self.use_site,
            site_cp: self.site_cp,
            bootstrap_index: self.bootstrap_index,
            bootstrap: self.evidence.bootstrap.clone(),
            bootstrap_arguments: self.evidence.bootstrap_arguments,
            sam_name: self.sam_name.clone(),
            sam_descriptor: self.sam_descriptor.clone(),
            sam_method_type: self.evidence.sam_method_type.clone(),
            instantiated_method_type: self.evidence.instantiated_method_type.clone(),
            implementation: self.evidence.implementation.clone(),
            captures: self.captures.clone(),
            form: self.form,
            refusal: self.refusal.clone(),
        }
    }
}

/// One synthetic accessor call site's decision, as this build made it (change
/// `add-demand-driven-core-results`, D3).
///
/// This is the **plan** side of `accessor@1`'s record: what the rule read about the call site and
/// its callee, whether the call site was presented, and the refusal when it was not. No
/// [`AccessorRecord`] is built while the body is decided; the records are written from these sites
/// by [`Program::materialize_accessors`] after the artifact is committed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AccessorSite {
    /// The BCI of the call site in the presented body.
    pub(crate) call_site: u32,
    /// What the rule read about the call site and the callee it names.
    pub(crate) evidence: accessor::Evidence,
    /// Whether the call site was presented as the field access it forwards.
    pub(crate) presented: bool,
    /// Why it was not, when it was not.
    pub(crate) refusal: Option<AccessorRefusal>,
}

impl AccessorSite {
    /// The owning record, built here and only here.
    fn record(&self) -> AccessorRecord {
        crate::demand_counts::record_built(crate::evidence::RecoveryEvidenceKind::RuleDetails);
        AccessorRecord::of(
            self.call_site,
            &self.evidence,
            self.presented,
            self.refusal.clone(),
        )
    }
}

impl Program {
    /// The owning records `lambda@1` publishes under `publication`, in BCI order, within the phase's
    /// remaining allowance.
    ///
    /// The rules ran when the body was built; what the selection decides is which records exist. A
    /// site whose own positions are outside the selected driver range is not built at all — a
    /// callee's bytecode index is never one of these positions, so a range of the driver body never
    /// selects a site by a callee's BCI — and a site the phase cannot pay for ends the category with
    /// the prefix it already built.
    pub(crate) fn materialize_lambdas(
        &self,
        publication: Publication,
        phase: &mut crate::evidence::EvidencePhase,
        budget: &mut Budget,
    ) -> (Vec<LambdaRecord>, crate::evidence::Materialized) {
        phase.materialize(
            budget,
            self.lambdas
                .iter()
                .filter(|site| publication.publishes(&site.positions())),
            LambdaSite::record,
        )
    }

    /// The owning records `accessor@1` publishes under `publication`, in BCI order, within the
    /// phase's remaining allowance.
    pub(crate) fn materialize_accessors(
        &self,
        publication: Publication,
        phase: &mut crate::evidence::EvidencePhase,
        budget: &mut Budget,
    ) -> (Vec<AccessorRecord>, crate::evidence::Materialized) {
        phase.materialize(
            budget,
            self.accessors
                .iter()
                .filter(|site| publication.publishes(&[site.call_site])),
            AccessorSite::record,
        )
    }
}

/// The facts of one run the build reads beside the regions and the region tree's own inputs.
///
/// These are the payload's own decode facts (the class's pool and its bootstrap table, the profile
/// whose rule set the run admits) plus the names table the statements are written with. They travel
/// as one value because every one of them is *the same run's*, and a builder that took them one by
/// one could be handed two of something.
pub(crate) struct Inputs<'a> {
    /// The decoded prefix from which this run's canonical graph was built.
    pub(crate) code: &'a MethodCodeFacts,
    /// The class's constant pool, as the same header read decoded it.
    pub(crate) pool: &'a [CpEntryFacts],
    /// The class's `BootstrapMethods` table, as the same read decoded it.
    pub(crate) bootstrap: &'a [BootstrapMethodFacts],
    /// The profile this run presents under: the gate the `lambda@1` rule is admitted through.
    pub(crate) profile: RecoveryProfile,
    /// How many local slots the method's parameters occupy, `this` included when the caller
    /// counted it: the slots below this one are declared by the signature, not by the body.
    pub(crate) parameters: u16,
    /// Whether slot 0 is the member's receiver, as its own declaration states.
    pub(crate) has_receiver: bool,
    /// The type each parameter slot holds, as the member's own descriptor states it (P3-R5): a
    /// `boolean` parameter and an `int` one share a slot shape, and only this fact tells them apart.
    pub(crate) parameter_types: &'a BTreeMap<u16, Type>,
    /// The type the member's own descriptor **returns**, as the same reading of the same descriptor
    /// states it (`None` for `V`, and for a descriptor this layer cannot read). The frames state one
    /// slot shape for the four int-sized primitives, so this signature fact is what says whether a
    /// `return` in this body presents a boolean — and it is the requirement every other `return` of
    /// the body is written under; the caller derives it where it derives
    /// [`Self::parameter_types`], so that a builder handed this run's facts never reads a second
    /// opinion out of a descriptor itself.
    pub(crate) return_type: Option<Type>,
    /// The names the presentation decided, in slot order.
    pub(crate) names: &'a NameTable,
    /// The variables each local slot holds (P3 3.4): one per slot unless either disjoint debug
    /// ranges or a proved SSA/CFG lifetime boundary separates its uses. Each segment may or may not
    /// have its own debug name.
    pub(crate) reuse: &'a reuse::Plan,
    /// The concatenation chains of this body, with the shape's own declaration of which
    /// instructions they own (P3 2.2).
    pub(crate) chains: &'a concat::Plan,
    /// The class's other members, as the caller read them: the only evidence a synthetic accessor
    /// call site can be decided from (P3 2.2, A12).
    pub(crate) members: Option<&'a ClassMembers>,
    /// The selected class-source member targets whose proved paths may spell nested owners in this
    /// run's static calls.
    pub(crate) member_inner_targets: &'a [crate::report::ProvedMemberInnerTarget],
    /// The exact interface-special targets whose Java source binding the facade proved.
    pub(crate) interface_super_calls: &'a [crate::report::ProvedInterfaceSuperCall],
    /// The class that declares this member, in the class file's own internal form
    /// (`java/lang/Integer`), as the run's own member declaration states it.
    ///
    /// This is the class a static call's pool owner is compared against: a call to **this** class
    /// is written with the member's bare name (`own(arg0)`), and a call to any other class is
    /// written with that class (`java.lang.Integer.valueOf(arg0)`, P3 4.4). A run that states no
    /// declaration cannot tell the two apart, and qualifies nothing.
    pub(crate) declaring_class: Option<&'a str>,
    /// The direct superclass from the same class header as the decoded body.
    pub(crate) direct_super_class: Option<&'a jarde_reader::model::JvmBytes>,
    /// The direct interfaces from the same class header as the decoded body.
    pub(crate) direct_interfaces: &'a [jarde_reader::model::JvmString],
    /// The current class's member headers from that same class header, when available.
    pub(crate) class_methods: Option<&'a [jarde_reader::classfile::MemberHeader]>,
    /// The verdict of the `bridge@1` rule for this very body, when the member is declared a bridge
    /// or its body is the forward a bridge is written as.
    pub(crate) bridge: Option<&'a bridge::Plan>,
    /// The construction sites of this body, with the shape's own declaration of which instructions
    /// they own (P3 2.3, `new@1`).
    pub(crate) sites: &'a init::Sites,
    /// The constructor prologue of this body, when it is an instance initializer (P3 2.3, `init@1`).
    pub(crate) prologues: &'a init::Prologues,
    /// The field accesses this body's instructions were verified to be (P3 2.3, `field@1`).
    pub(crate) fields: &'a field::Plan,
    /// The dispatch-table reads this body performs (P3 2.3, `enumswitch@1`).
    pub(crate) enums: &'a enumswitch::Plan,
    /// Whether this presentation may omit a proved array helper and write `T[]::new`. The
    /// class-source assembler leaves this off until it can prove the helper is unused class-wide.
    pub(crate) allow_array_constructor_method_references: bool,
}

/// The path of a region in the method's region tree (P3 3.1).
///
/// Each step is the index of a nested region inside the one that holds it, so `[2, 1]` is the
/// `else` arm of the third top-level region. The **empty** path is the method body itself, and a
/// declaration written there is in scope for the whole body.
type RegionPath = Vec<u32>;

/// Where each local variable's declaration is written (P3 3.1).
///
/// The invariant this plan keeps: a variable's declaration is written at the start of a region that
/// contains **every** use of that variable — every read and every write — so the declaration is in
/// scope at each of them. A Java local is in scope from its declaration to the end of the block that
/// declares it, and that is not the same thing as "the whole method knows this slot": a slot filled
/// in a `then` arm and read in the `else` arm or after the join is declared in a place where most of
/// its uses cannot see it, and the text does not compile.
///
/// Two cases, and the evidence for each:
///
/// * **the write that first fills the variable is in the innermost region that contains all uses** —
///   then that write's own statement declares the variable with the value it writes, which is the
///   text a source would have (`int x = 1;`) and the text this layer has always written;
/// * **otherwise** the declaration moves to the start of that innermost region and **every** write
///   becomes a plain assignment: the declaration is no longer any write's, so no write may carry it.
///
/// The "innermost region that contains all uses" is read from the region tree itself: every block
/// belongs to exactly one region (the innermost one that claims it), and the region that contains
/// all uses is the longest common prefix of their region paths.
///
/// Which variable a use belongs to is [`crate::reuse`]'s decision, and it is what makes P3 3.4's
/// slot reuse come out right: the two variables one reused slot holds have **disjoint** uses, so
/// each is planned on its own — the arm's own variable is declared at the write that fills it, in
/// the arm, while a variable written in both arms and read after the join is hoisted above the
/// branch. Both rules are the same rule; only the use sets differ.
///
/// A variable whose uses are all inside one **quoted** region is left exactly as it was: a
/// [`Region::Fallback`] writes no Java statements this layer could declare a local in, so there is
/// nothing to hoist into, and that run is already `Mixed`/`Fallback`. The same holds for a use whose
/// block the region tree does not claim: with no region to name, the variable keeps the declaration
/// it has today instead of a guess.
#[derive(Default)]
struct Declarations {
    /// The variables declared at the start of one region, by that region's path, in slot and
    /// variable order.
    at_region: BTreeMap<RegionPath, Vec<HoistedDeclaration>>,
    /// What the plan decided about each variable's type, by that variable's own identity: the one
    /// decision the hoisted declaration, the in-place declaration, the assignments, the conditions
    /// and the returns all present the variable with.
    decided: BTreeMap<LocalVariable, Decided>,
    /// The placement decision for every named, non-parameter local touched by this body. This is
    /// kept beside `at_region`: a missing hoist must say whether the local belongs in its own
    /// lexical region or whether the cross-region evidence was incomplete.
    placements: BTreeMap<LocalVariable, DeclarationPlacement>,
    /// The smallest region that contains a cross-region local whose definition/use proof was
    /// incomplete. An empty path means the method body itself is the only safe refusal boundary.
    incomplete: BTreeMap<RegionPath, String>,
}

/// The lexical declaration decision made before the AST exists.
#[derive(Clone, Debug, Eq, PartialEq)]
enum DeclarationPlacement {
    /// Every access is in one lexical owner; the first write declares the local there.
    Local { owner: RegionPath },
    /// The local is declared at the common owner before its regions and every write is an
    /// assignment. This is used only when all reaching values are backed by writes the builder
    /// will actually present.
    Elevated { owner: RegionPath },
    /// The common lexical owner is known, but an incomplete region or reaching definition prevents
    /// a safe declaration and assignment from being stated.
    Incomplete { owner: RegionPath },
}

/// What the plan decided about one local variable's type before a statement of this body existed.
///
/// The decision is the same evidence the layer always read — a descriptor, a read of a variable the
/// same plan decided `boolean`, otherwise the frame's own type — read **once**, for every variable,
/// and then only consumed. Deciding here rather than at each use is what keeps the two declaration
/// paths from answering differently, and what lets every write be checked against one answer.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Decided {
    /// The variable holds this type: `boolean` when a descriptor states it or a variable this plan
    /// decided boolean is copied into it, and otherwise the type the frames state for the value its
    /// first write stores.
    Type(Type),
    /// No type can be decided for the variable, with the fact that stopped it: a write of it can
    /// then not be published as a declaration, and the structure that write belongs to is refused.
    Unknown(NoType),
}

/// The kind of evidence one int-shaped value carries when considered as a boolean operand.
///
/// A `0`/`1` literal can fill a boolean operand once another operand anchors the expression, but it
/// cannot start a boolean declaration by itself. `Proven` therefore always contains an independent
/// descriptor or already-decided-local fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BooleanEvidence {
    None,
    Literal,
    Proven,
}

impl BooleanEvidence {
    fn is_boolean(self) -> bool {
        !matches!(self, Self::None)
    }

    fn has_seed(self) -> bool {
        matches!(self, Self::Proven)
    }
}

impl Decided {
    /// Whether this decision states a `boolean`.
    fn is_boolean(&self) -> bool {
        matches!(self, Self::Type(Type::Boolean))
    }
}

/// Why the plan could not decide a variable's type.
///
/// The reasons a declaration cannot state one sound Java type for every write of a local. They
/// are recorded in the plan so the hoisted and in-place paths make the same refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
enum NoType {
    /// The value the variable's first write stores has no frame entry that states a type
    /// (`Top`, a category-2 value's second slot, an uninitialized value, a return address).
    NoFrameEntry,
    /// The value the variable's first write stores has a frame entry that names a descriptor this
    /// layer cannot spell as a Java type.
    Unspellable(String),
    /// The slot was treated as one source local, but a later write has a value in the other JVM
    /// type category. A reference and a primitive cannot share one Java declaration.
    ConflictingWrites {
        first: u32,
        later: u32,
        declared: Type,
        observed: Type,
    },
}

impl NoType {
    /// The reason a refused declaration states, at the write that would have carried it.
    fn message(&self, slot: u16, at: u32) -> String {
        match self {
            Self::NoFrameEntry => format!("local {slot} has no frame entry stating its type"),
            Self::Unspellable(name) => format!(
                "the local written at BCI {at} holds a value the frames name `{name}`, which is a descriptor this layer cannot spell as a Java type, so the declaration is refused instead of writing it"
            ),
            Self::ConflictingWrites {
                first,
                later,
                declared,
                observed,
            } => format!(
                "local {slot} is treated as one source variable, but BCI {first} writes `{}` and BCI {later} writes `{}`; no Java declaration can hold both, so this region is refused instead of publishing a contradictory local",
                declared.spell(),
                observed.spell()
            ),
        }
    }
}

/// One declaration written at the start of a region instead of at the write that fills the variable.
#[derive(Clone)]
struct HoistedDeclaration {
    variable: LocalVariable,
    /// The plan's own decision ([`Declarations::decided`]) for this variable: the one type both
    /// declaration paths write, so a variable cannot be typed one way above the branch and another
    /// way inside it.
    ty: Type,
    /// The write whose value states the type and whose frame entry states the variable's type: the
    /// anchor the declaration is written under, exactly like the in-place declaration it replaces.
    at: u32,
}

/// What one write's declaration did.
///
/// The three outcomes are three different answers, and the ambiguity this enum removes is the defect
/// the change fixes: before it, `Ok(None)` meant both "no declaration is due" and "the declaration
/// failed and a fallback was written", so a caller could not tell a variable that needs no
/// declaration from one whose declaration was refused — and went on to write the assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Declaration {
    /// This write carries the variable's declaration, with the plan's type for it.
    Declared(Type),
    /// No declaration is due: a parameter's (the signature declares it), an already declared
    /// variable's, or a variable with no name to declare.
    NotDue,
    /// No declaration could be written: the plan decided no type for the variable, the fallback that
    /// states why is already recorded, and the assignment this write would otherwise become may not
    /// be written.
    Refused,
}

/// Plans where each local variable's declaration is written, and what type every variable's uses
/// are presented with.
///
/// The two are one plan because they are one decision: a variable's type is decided here, by the
/// variable's own identity, before a statement of the body exists, and the declaration's *place*
/// only says where that decision is written. Deciding the type here is what keeps the hoisted path
/// (which runs before any declaration exists) and the in-place path (which used to recognize the
/// declarations it had already written) from answering differently about the same value, and what
/// makes the answer independent of the order the regions are walked in.
#[allow(clippy::too_many_arguments)]
fn declarations(
    regions: &[Region],
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    operations: &Operations,
    names: &NameTable,
    reuse: &reuse::Plan,
    parameters: u16,
    parameter_types: &BTreeMap<u16, Type>,
    return_type: Option<&Type>,
    fields: &field::Plan,
    budget: &mut Budget,
) -> Result<Declarations, StopReason> {
    let paths = region_paths(regions);
    let uses = slot_uses(ssa, operations, reuse, &paths);
    let short_circuit_booleans = short_circuit_local_booleans(
        regions,
        canonical,
        ssa,
        operations,
        names,
        reuse,
        fields,
        return_type,
        parameters,
        &uses,
        &paths,
        budget,
    )?;
    let mut plan = Declarations {
        decided: decide_types(
            &uses,
            ssa,
            operations,
            reuse,
            parameters,
            parameter_types,
            fields,
            &short_circuit_booleans,
            budget,
        )?,
        ..Declarations::default()
    };
    // The slots a guarded statement declares **in its own header** (P3 2.4): a `try (T n = …)`
    // header is the declaration, no statement of the body writes one, and hoisting a second
    // declaration above the statement would declare the same name twice.
    let resources = resource_slots(regions);
    // The lexical plan is a plan of the live flow. An uncovered quote is dead only when none of
    // its entry edges can run: an exception edge fed by a real throw site is a live entry even
    // though the normal-flow region walk did not visit its handler. Type decisions above still
    // read every access, including those in a dead quote.
    let mut dead = BTreeSet::new();
    for (index, region) in regions.iter().enumerate() {
        poll(budget, None)?;
        charge(budget, CountedBudgetDimension::AnalysisSteps, 1, None)?;
        if let Region::Fallback {
            blocks,
            reason: FallbackReason::UncoveredBlocks { .. },
        } = region
            && dead_uncovered_quote(blocks, canonical, budget)?
        {
            dead.insert(child(&[], u32::try_from(index).unwrap_or(u32::MAX)));
        }
    }
    let mut live_uses = BTreeMap::new();
    for (variable, variable_uses) in &uses {
        let at = variable_uses.first().map(|use_| use_.bci);
        poll(budget, at)?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(variable_uses.len()).unwrap_or(u64::MAX),
            at,
        )?;
        let filtered: Vec<SlotUse> = variable_uses
            .iter()
            .filter(|use_| use_.path.as_ref().is_none_or(|path| !dead.contains(path)))
            .cloned()
            .collect();
        if !filtered.is_empty() {
            live_uses.insert(*variable, filtered);
        }
    }
    for (variable, variable_uses) in &live_uses {
        // A parameter's declaration is the signature, not the body, and a variable this layer has no
        // name for is one whose writes are already reported as a fallback of their own.
        if variable.slot() < parameters || names.text(*variable).is_none() {
            continue;
        }
        let Some(owner) = common_owner(variable_uses) else {
            plan.placements.insert(
                *variable,
                DeclarationPlacement::Incomplete { owner: Vec::new() },
            );
            plan.incomplete.entry(Vec::new()).or_insert_with(|| {
                format!(
                    "local {} has an access outside the recovered region tree, so its lexical owner and complete definition-use slice cannot be proved",
                    variable.slot()
                )
            });
            continue;
        };
        let spans_regions = variable_uses
            .iter()
            .any(|use_| use_.path.as_ref() != Some(&owner));
        if !spans_regions {
            plan.placements
                .insert(*variable, DeclarationPlacement::Local { owner });
            continue;
        }
        let has_unpresented_access = variable_uses.iter().any(|use_| {
            use_.path
                .as_ref()
                .is_some_and(|path| paths.fallbacks.contains(path))
        });
        if has_unpresented_access {
            plan.placements.insert(
                *variable,
                DeclarationPlacement::Incomplete {
                    owner: owner.clone(),
                },
            );
            plan.incomplete.entry(owner).or_insert_with(|| {
                format!(
                    "local {} crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
                    variable.slot()
                )
            });
            continue;
        }
        let mut escaping_catch = None;
        for (path, (slot, handler)) in &paths.catch_parameters {
            if *slot == variable.slot()
                && variable_uses.iter().any(|use_| {
                    use_.path
                        .as_ref()
                        .is_some_and(|use_path| use_path.starts_with(path))
                })
                && variable_uses.iter().any(|use_| {
                    !use_
                        .path
                        .as_ref()
                        .is_some_and(|use_path| use_path.starts_with(path))
                })
                && !catch_parameter_stays_in_clause(
                    ssa,
                    operations,
                    variable_uses,
                    path,
                    handler,
                    *slot,
                    budget,
                )?
            {
                escaping_catch = Some(path.clone());
                break;
            }
        }
        if let Some(catch_path) = escaping_catch {
            let owner = common_owner(variable_uses).unwrap_or_default();
            plan.placements.insert(
                *variable,
                DeclarationPlacement::Incomplete {
                    owner: owner.clone(),
                },
            );
            plan.incomplete.entry(owner).or_insert_with(|| {
                format!(
                    "local {} escapes catch parameter scope at region {catch_path:?}; its catch header cannot declare a method-visible local",
                    variable.slot()
                )
            });
            continue;
        }
        if resources.contains(&variable.slot()) {
            continue;
        }
        let Some(region) = declaration_region(variable_uses, &paths) else {
            plan.placements.insert(
                *variable,
                DeclarationPlacement::Incomplete {
                    owner: owner.clone(),
                },
            );
            plan.incomplete.entry(owner).or_insert_with(|| {
                format!(
                    "local {} spans accesses with no complete recovered lexical owner",
                    variable.slot()
                )
            });
            continue;
        };
        // The write that fills the variable first, in method order: the region path a block stands
        // in is the order the regions are written in, and the bytecode index orders the blocks of
        // one region.
        let Some(first) = variable_uses
            .iter()
            .filter(|use_| use_.written.is_some())
            .min_by_key(|use_| (use_.path.clone(), use_.bci))
        else {
            continue;
        };
        if first.path.as_ref() == Some(&owner) {
            // Every use is in the first write's own region: the declaration the write carries is in
            // scope for all of them, which is the text this layer has always written.
            plan.placements
                .insert(*variable, DeclarationPlacement::Local { owner });
            continue;
        }
        let crosses_exception = crosses_exception_region(variable_uses, &paths);
        let store_type_is_proven = matches!(
            plan.decided.get(variable),
            Some(Decided::Type(Type::Int | Type::Boolean))
        );
        if crosses_exception
            && (!store_type_is_proven
                || !all_reads_reach_presented_writes(
                    ssa,
                    operations,
                    variable_uses,
                    &paths,
                    budget,
                )?)
        {
            plan.placements.insert(
                *variable,
                DeclarationPlacement::Incomplete {
                    owner: region.clone(),
                },
            );
            plan.incomplete.entry(region).or_insert_with(|| {
                format!(
                    "local {} crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write",
                    variable.slot()
                )
            });
            continue;
        }
        // The declaration states the type the plan decided for this variable — the same answer the
        // write that fills it would state in place. A variable whose type could not be decided keeps
        // the write's own refusal, which is what the in-place path states for it.
        let Some(Decided::Type(ty)) = plan.decided.get(variable) else {
            continue;
        };
        plan.placements.insert(
            *variable,
            DeclarationPlacement::Elevated {
                owner: region.clone(),
            },
        );
        plan.at_region
            .entry(region)
            .or_default()
            .push(HoistedDeclaration {
                variable: *variable,
                ty: ty.clone(),
                at: first.bci,
            });
    }
    let invalid = validate_declaration_placements(&plan, &live_uses, budget)?;
    for (variable, owner) in invalid {
        for declarations in plan.at_region.values_mut() {
            declarations.retain(|declaration| declaration.variable != variable);
        }
        plan.placements.insert(
            variable,
            DeclarationPlacement::Incomplete {
                owner: owner.clone(),
            },
        );
        plan.incomplete.entry(owner).or_insert_with(|| {
            format!(
                "local {} has a declaration placement that does not cover all known reads and writes",
                variable.slot()
            )
        });
    }
    Ok(plan)
}

/// Checks the decisions the planner made against the exact access paths it consumed.
///
/// This is deliberately a plan invariant check, not a second AST/name traversal: every access is
/// read from `slot_uses`, every owner is a RegionPath, and a failed check adds the smallest known
/// common owner to the pre-build refusal set.
fn validate_declaration_placements(
    plan: &Declarations,
    uses: &BTreeMap<LocalVariable, Vec<SlotUse>>,
    budget: &mut Budget,
) -> Result<Vec<(LocalVariable, RegionPath)>, StopReason> {
    let mut invalid = Vec::new();
    for (variable, placement) in &plan.placements {
        let Some(variable_uses) = uses.get(variable) else {
            invalid.push((*variable, Vec::new()));
            continue;
        };
        let owner = match placement {
            DeclarationPlacement::Local { owner }
            | DeclarationPlacement::Elevated { owner }
            | DeclarationPlacement::Incomplete { owner } => owner,
        };
        let anchor = variable_uses.first().map(|use_| use_.bci);
        charge(budget, CountedBudgetDimension::AnalysisSteps, 1, anchor)?;
        let mut covered = true;
        for use_ in variable_uses {
            poll(budget, Some(use_.bci))?;
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(use_.bci),
            )?;
            if !use_
                .path
                .as_ref()
                .is_some_and(|path| path.starts_with(owner))
            {
                covered = false;
                break;
            }
        }
        let consistent = match placement {
            DeclarationPlacement::Local { owner } => {
                covered
                    && variable_uses
                        .iter()
                        .filter(|use_| use_.written.is_some())
                        .min_by_key(|use_| (use_.path.clone(), use_.bci))
                        .is_some_and(|first| first.path.as_ref() == Some(owner))
            }
            DeclarationPlacement::Elevated { owner } => {
                covered
                    && plan.at_region.get(owner).is_some_and(|declared| {
                        declared
                            .iter()
                            .any(|declaration| declaration.variable == *variable)
                    })
                    && variable_uses
                        .iter()
                        .filter(|use_| use_.written.is_some())
                        .min_by_key(|use_| (use_.path.clone(), use_.bci))
                        .is_some_and(|first| first.path.as_ref() != Some(owner))
            }
            DeclarationPlacement::Incomplete { owner } => {
                !plan.at_region.values().any(|declarations| {
                    declarations
                        .iter()
                        .any(|declaration| declaration.variable == *variable)
                }) && plan.incomplete.contains_key(owner)
            }
        };
        if !consistent {
            invalid.push((*variable, owner.clone()));
        }
    }
    Ok(invalid)
}

/// Every local access of this body, grouped by the variable it belongs to.
///
/// A read is recorded where it reads and a write where it writes, with the region path of the block
/// the instruction stands in — the order the regions are written in is the path's own order, which
/// is what makes "the first write" a fact about the method rather than about this walk.
fn slot_uses(
    ssa: &SsaTable,
    operations: &Operations,
    reuse: &reuse::Plan,
    paths: &RegionPaths,
) -> BTreeMap<LocalVariable, Vec<SlotUse>> {
    let mut uses: BTreeMap<LocalVariable, Vec<SlotUse>> = BTreeMap::new();
    for block in ssa.blocks() {
        let path = paths.paths.get(block.block());
        for instruction in block.instructions() {
            for (slot, value) in instruction.reads() {
                if let Slot::Local(slot) = slot
                    && let Some(variable) = reuse.variable_at(*slot, instruction.bci())
                {
                    uses.entry(variable).or_default().push(SlotUse {
                        path: path.cloned(),
                        bci: instruction.bci(),
                        read: Some(*value),
                        written: None,
                        stored: None,
                    });
                }
            }
            for (slot, value) in instruction.writes() {
                if let Slot::Local(slot) = slot
                    && let Some(variable) = reuse.variable_at(*slot, instruction.bci())
                {
                    uses.entry(variable).or_default().push(SlotUse {
                        path: path.cloned(),
                        bci: instruction.bci(),
                        read: None,
                        written: Some(*value),
                        stored: store_operand(operations, instruction),
                    });
                }
            }
        }
    }
    uses
}

/// A stack Phi may seed one Boolean local only when its sole store and every subsequent read
/// have an explicit Boolean consumer. The ordinary frame type is `Int` for all int-sized JVM
/// primitives, so this narrow evidence must be established before `decide_types` runs.
#[allow(clippy::too_many_arguments)]
fn short_circuit_local_booleans(
    regions: &[Region],
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    operations: &Operations,
    names: &NameTable,
    reuse: &reuse::Plan,
    fields: &field::Plan,
    return_type: Option<&Type>,
    parameters: u16,
    uses: &BTreeMap<LocalVariable, Vec<SlotUse>>,
    paths: &RegionPaths,
    budget: &mut Budget,
) -> Result<BTreeSet<LocalVariable>, StopReason> {
    let mut proved = BTreeSet::new();
    let mut pending = regions.iter().collect::<Vec<_>>();
    while let Some(region) = pending.pop() {
        poll(budget, region.blocks().first().map(|block| block.bci()))?;
        match region {
            Region::ShortCircuitValue { consumer_bci, .. } => {
                if !matches!(operations.get(*consumer_bci), Some(Operation::Store { .. })) {
                    continue;
                }
                let ShortCircuitValueAttempt::Proved(proof) = prove_short_circuit_value(
                    region,
                    canonical,
                    ssa,
                    operations,
                    return_type,
                    budget,
                )?
                else {
                    continue;
                };
                let ShortCircuitConsumer::Local(slot, written) = proof.consumer else {
                    continue;
                };
                let at = proof.consumer_bci;
                let Some(variable) = reuse.variable_at(slot, at) else {
                    continue;
                };
                if slot < parameters
                    || names.text(variable).is_none()
                    || !matches!(ssa.value(written).def(), Definition::Instruction { bci, .. } if *bci == at)
                    || ssa.value(written).replaced_by().is_some()
                {
                    continue;
                }
                let Some(accesses) = uses.get(&variable) else {
                    continue;
                };
                let writes = accesses
                    .iter()
                    .filter(|access| access.written.is_some())
                    .collect::<Vec<_>>();
                let [write] = writes.as_slice() else {
                    continue;
                };
                if declaration_region(accesses, paths).is_none()
                    || write.bci != at
                    || write.written != Some(written)
                    || write.stored != Some(proof.phi)
                    // This first Store consumer only claims a declaration in its own region.
                    // A read in another lexical region needs an independently proved elevated
                    // assignment and is conservatively left to a later scope change.
                    || accesses.iter().any(|access| access.path != write.path)
                {
                    continue;
                }
                let reads = accesses
                    .iter()
                    .filter(|access| access.read.is_some())
                    .collect::<Vec<_>>();
                let direct_readers = ssa
                    .value(written)
                    .uses()
                    .iter()
                    .filter_map(|reader| reader.bci())
                    .collect::<BTreeSet<_>>();
                let read_bcis = reads
                    .iter()
                    .map(|access| access.bci)
                    .collect::<BTreeSet<_>>();
                if reads.is_empty()
                    || reads.len() != read_bcis.len()
                    || ssa.value(written).uses().len() != reads.len()
                    || direct_readers != read_bcis
                    || reads.iter().any(|access| {
                        access.bci <= at
                            || access.read != Some(written)
                            || reuse.variable_at(slot, access.bci) != Some(variable)
                    })
                {
                    continue;
                }
                let mut complete = true;
                for access in reads {
                    poll(budget, Some(access.bci))?;
                    charge(
                        budget,
                        CountedBudgetDimension::AnalysisSteps,
                        1,
                        Some(access.bci),
                    )?;
                    let Some(load) = instruction_at(ssa, access.bci) else {
                        complete = false;
                        break;
                    };
                    let Some(Operation::Load { slot: load_slot }) = operations.get(access.bci)
                    else {
                        complete = false;
                        break;
                    };
                    let [(Slot::Stack(_), loaded)] = load.writes() else {
                        complete = false;
                        break;
                    };
                    if *load_slot != slot
                        || !matches!(load.opcode(), 0x15 | 0x1a..=0x1d)
                        || load.reads() != [(Slot::Local(slot), written)]
                    {
                        complete = false;
                        break;
                    }
                    let [reader] = ssa.value(*loaded).uses() else {
                        complete = false;
                        break;
                    };
                    let Some(consumer_bci) = reader.bci() else {
                        complete = false;
                        break;
                    };
                    let Some(consumer) = instruction_at(ssa, consumer_bci) else {
                        complete = false;
                        break;
                    };
                    if !matches!(consumer.reads(), [(Slot::Stack(_), value)] if *value == *loaded) {
                        complete = false;
                        break;
                    }
                    let boolean_position = match (consumer.opcode(), operations.get(consumer_bci)) {
                        (
                            0xb3,
                            Some(Operation::Field {
                                access: FieldAccess::Write,
                                is_static: true,
                                descriptor,
                                ..
                            }),
                        ) if descriptor == "Z" => fields
                            .claim(consumer_bci)
                            .is_some_and(|(_, shape)| shape.value == Some(*loaded)),
                        (0xac, Some(Operation::Return)) => return_type == Some(&Type::Boolean),
                        _ => false,
                    };
                    if !boolean_position {
                        complete = false;
                        break;
                    }
                }
                if complete {
                    proved.insert(variable);
                }
            }
            Region::Sequence { regions } | Region::Loop { body: regions, .. } => {
                pending.extend(regions);
            }
            Region::If {
                then_arm, else_arm, ..
            } => pending.extend([then_arm.as_ref(), else_arm.as_ref()]),
            Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => {
                pending.extend(groups.iter().map(|group| group.arm.as_ref()));
            }
            Region::Try { body, catches, .. } => {
                pending.push(body);
                pending.extend(catches.iter().map(|clause| clause.body()));
            }
            Region::Straight { .. }
            | Region::TwoExitReturn { .. }
            | Region::Fallback { .. }
            | Region::Guard { .. }
            | Region::LoopBreak { .. }
            | Region::LoopContinue { .. } => {}
        }
    }
    Ok(proved)
}

/// The write one variable's type is decided from, and the values that state it.
struct FirstWrite {
    /// The write's own bytecode index: the anchor a declaration or a refusal is written under.
    at: u32,
    /// The value the slot takes.
    written: ValueId,
    /// The value the writing instruction **reads** as the one it stores, when the two differ: it is
    /// this value — not the slot's own — whose evidence states what the variable holds.
    stored: ValueId,
}

/// Decides every variable's type once, from a finite list of evidence, before a statement exists.
///
/// The evidence is a closed list, and nothing is followed anywhere else:
///
/// * a **descriptor** fact about the value a variable's first write stores: a `Z` parameter slot's
///   load, a call whose callee descriptor returns `Z`, a field read a `field@1` claim states is `Z`
///   ([`boolean_proof`]) — for a parameter variable, its own descriptor is that same fact, read
///   directly;
/// * a **read of another variable this plan decided boolean**, one read and one hop: a variable whose
///   first write stores such a read is boolean too, and a chain of copies therefore reaches a
///   fixpoint. Nothing else is followed by this copy propagation: a store's own value and a value
///   two definitions away state nothing without a separate proof;
/// * a **bounded short-circuit Phi/Store proof** from [`short_circuit_local_booleans`]: the exact
///   1/0 graph, unique `istore`, one named local and all its reachable reads at explicit Boolean
///   consumers form one initiating seed before this type decision runs;
/// * the **frame's type** for the value a variable's first write stores — the type of every
///   non-boolean decision, because the frames state one slot shape for the four int-sized
///   primitives and cannot tell a `boolean` from an `int`;
/// * **nothing else**: an isolated `0`/`1` literal is refused as *initiating* evidence (a fresh local
///   states no type, so `int x = 0;` and `boolean c = true;` are the same bytes), and it adapts to
///   `true`/`false` only where the target is already decided boolean — which is how the consumers
///   spell it, not a decision of this pass.
///
/// A variable whose first write states no usable type is [`Decided::Unknown`]: the write that would
/// have declared it refuses instead, and no second place decides a type for it.
///
/// The propagation runs on a worklist to a fixpoint, so a chain of copies is decided the same way
/// whatever order the variables are met in, and every queue entry is billed and polled against the
/// run's own budget and cancellation before it is processed. The queue holds each variable at most
/// once, so the pass is bounded by the variables' own write count.
#[allow(clippy::too_many_arguments)]
fn decide_types(
    uses: &BTreeMap<LocalVariable, Vec<SlotUse>>,
    ssa: &SsaTable,
    operations: &Operations,
    reuse: &reuse::Plan,
    parameters: u16,
    parameter_types: &BTreeMap<u16, Type>,
    fields: &field::Plan,
    short_circuit_booleans: &BTreeSet<LocalVariable>,
    budget: &mut Budget,
) -> Result<BTreeMap<LocalVariable, Decided>, StopReason> {
    // The write each variable's type is decided from: the first one in method order, which is the
    // write both declaration paths read today.
    let mut first: BTreeMap<LocalVariable, FirstWrite> = BTreeMap::new();
    for (variable, variable_uses) in uses {
        let Some(use_) = variable_uses
            .iter()
            .filter(|use_| use_.written.is_some())
            .min_by_key(|use_| (use_.path.clone(), use_.bci))
        else {
            continue;
        };
        let Some(written) = use_.written else {
            continue;
        };
        first.insert(
            *variable,
            FirstWrite {
                at: use_.bci,
                written,
                stored: use_.stored.unwrap_or(written),
            },
        );
    }
    // The variables whose first writes have complete boolean evidence. A bitwise expression needs
    // two boolean operands and at least one descriptor/decided-local seed; its literal operands are
    // admitted only after that seed exists.
    let mut boolean_variables: BTreeSet<LocalVariable> = BTreeSet::new();
    let mut queue: VecDeque<LocalVariable> = VecDeque::new();
    for (variable, write) in &first {
        let descriptor_seed = variable.slot() < parameters
            && matches!(parameter_types.get(&variable.slot()), Some(Type::Boolean));
        let expression_proof = if descriptor_seed {
            true
        } else {
            let is_boolean_local = |value, at| {
                read_variable(ssa, operations, reuse, value, at)
                    .is_some_and(|dependency| boolean_variables.contains(&dependency))
            };
            let mut visit = |at| charge_bitwise_proof_node(operations, at, budget);
            let mut context = BooleanProofContext {
                ssa,
                operations,
                parameter_types,
                fields,
                is_boolean_local,
                visit: &mut visit,
            };
            boolean_proof(
                &mut context,
                write.stored,
                write.at,
                0,
                &mut BTreeMap::new(),
            )?
            .has_seed()
        };
        if (expression_proof || short_circuit_booleans.contains(variable))
            && boolean_variables.insert(*variable)
        {
            queue.push_back(*variable);
        }
    }
    // Which locals can affect each first-write proof. For a bitwise value the walk follows only its
    // nested bitwise operands; the queue later retries the entire proof after one dependency becomes
    // boolean, so seeing one boolean child cannot bless a mixed boolean/integer pair.
    let mut readers: BTreeMap<LocalVariable, BTreeSet<LocalVariable>> = BTreeMap::new();
    {
        let mut context = BitwiseDependencyContext {
            ssa,
            operations,
            reuse,
            budget,
        };
        for (variable, write) in &first {
            let mut dependencies = BTreeSet::new();
            context.collect(
                write.stored,
                write.at,
                0,
                &mut BTreeSet::new(),
                &mut dependencies,
            )?;
            for read in dependencies {
                readers.entry(read).or_default().insert(*variable);
            }
        }
    }
    while let Some(variable) = queue.pop_front() {
        let Some(write) = first.get(&variable) else {
            continue;
        };
        // The work this entry pays for is the decision it spreads; the entry's own write BCI is
        // where a run that cannot afford it stops, exactly as the statements bill their own.
        charge(budget, CountedBudgetDimension::IrItems, 1, Some(write.at))?;
        poll(budget, Some(write.at))?;
        for reader in readers.get(&variable).into_iter().flatten() {
            let Some(candidate) = first.get(reader) else {
                continue;
            };
            if boolean_variables.contains(reader) {
                continue;
            }
            let proof = {
                let is_boolean_local = |value, at| {
                    read_variable(ssa, operations, reuse, value, at)
                        .is_some_and(|dependency| boolean_variables.contains(&dependency))
                };
                let mut visit = |at| charge_bitwise_proof_node(operations, at, budget);
                let mut context = BooleanProofContext {
                    ssa,
                    operations,
                    parameter_types,
                    fields,
                    is_boolean_local,
                    visit: &mut visit,
                };
                boolean_proof(
                    &mut context,
                    candidate.stored,
                    candidate.at,
                    0,
                    &mut BTreeMap::new(),
                )?
            };
            if proof.has_seed() && boolean_variables.insert(*reader) {
                queue.push_back(*reader);
            }
        }
    }
    // The decision every consumer reads: the descriptor's boolean for a parameter, the propagated
    // boolean for a variable a read proved one, and otherwise the frame's own type for the value
    // the first write stores.
    let mut decided: BTreeMap<LocalVariable, Decided> = BTreeMap::new();
    for (variable, write) in &first {
        let decision = if variable.slot() < parameters {
            // A parameter's type is the signature's, not the frames': the descriptor states it, and
            // a body's write cannot widen it.
            match parameter_types.get(&variable.slot()) {
                Some(ty) => Decided::Type(ty.clone()),
                None => continue,
            }
        } else if boolean_variables.contains(variable) {
            Decided::Type(Type::Boolean)
        } else {
            // The frames' reading of the value, or — where they state only an unknown reference — the
            // array a creation of this very body built (P3 2b): a `newarray`'s frame entry states no
            // name, and the instruction's own element type is what types the local it filled.
            match written_type(ssa, operations, write.written) {
                Ok(Some(ty)) => Decided::Type(ty),
                Ok(None) => Decided::Unknown(NoType::NoFrameEntry),
                Err(name) => Decided::Unknown(NoType::Unspellable(name)),
            }
        };
        let decision = match decision {
            Decided::Type(declared) => {
                let mut conflict = None;
                for use_ in uses.get(variable).into_iter().flatten() {
                    let Some(written) = use_.written else {
                        continue;
                    };
                    poll(budget, Some(use_.bci))?;
                    charge(
                        budget,
                        CountedBudgetDimension::AnalysisSteps,
                        1,
                        Some(use_.bci),
                    )?;
                    // A caught value has its own handler header and does not inherit the ordinary
                    // local that occupied this slot on the normal predecessor. The existing
                    // catch/guard proof decides whether that header can be presented.
                    if use_.stored.is_some_and(|stored| {
                        matches!(ssa.value(stored).def(), Definition::Caught { .. })
                    }) {
                        continue;
                    }
                    if let Ok(Some(observed)) = written_type(ssa, operations, written)
                        && matches!(declared, Type::Reference(_))
                            != matches!(observed, Type::Reference(_))
                    {
                        conflict = Some(NoType::ConflictingWrites {
                            first: write.at,
                            later: use_.bci,
                            declared: declared.clone(),
                            observed,
                        });
                        break;
                    }
                }
                conflict.map_or(Decided::Type(declared), Decided::Unknown)
            }
            unknown => unknown,
        };
        decided.insert(*variable, decision);
    }
    Ok(decided)
}

/// The variable one value denotes a read of, when it denotes a read at all.
///
/// A `load` names the variable the slot holds at the load's own BCI; the entry state or the phi of
/// a local slot names the variable the slot holds at the use point `at`. Everything else — a value
/// another instruction produced, a stack value, a caught exception — denotes no variable, which is
/// the boundary the plan's propagation keeps: one read, one hop, and no definition chain.
fn read_variable(
    ssa: &SsaTable,
    operations: &Operations,
    reuse: &reuse::Plan,
    value: ValueId,
    at: u32,
) -> Option<LocalVariable> {
    let (slot, read_at) = match ssa.value(value).def() {
        Definition::Instruction { bci, .. } => match operations.get(*bci) {
            Some(Operation::Load { slot }) => (*slot, *bci),
            _ => return None,
        },
        Definition::Entry { slot, .. } | Definition::Phi { slot, .. } => match slot {
            Slot::Local(slot) => (*slot, at),
            Slot::Stack(_) => return None,
        },
        Definition::Caught { .. } => return None,
    };
    reuse.variable_at(slot, read_at)
}

/// Every slot a statement's **own header** declares.
///
/// Two headers declare a local and write no statement of the body for it: the resource of a
/// `try (T n = …)` header, and the parameter of a `catch (T n)` clause. In both cases the header
/// *is* the declaration, so hoisting a second one above the statement would declare the same name
/// twice — and a slot a clause header declares is never split, for the same reason a resource's is
/// not (P3 3.4).
pub(crate) fn resource_slots(regions: &[Region]) -> BTreeSet<u16> {
    let mut slots: BTreeSet<u16> = BTreeSet::new();
    let mut walk = |region: &Region| match region {
        Region::Guard { plan, .. } => {
            if let guard::Shape::Resources { resources, .. } = plan.shape() {
                for resource in resources {
                    slots.insert(resource.slot());
                }
            }
        }
        Region::Try { catches, .. } => {
            for clause in catches {
                slots.insert(clause.parameter());
            }
        }
        _ => {}
    };
    for region in regions {
        collect_guards(region, &mut walk);
    }
    slots
}

/// Calls one visitor on every guarded region of a tree.
fn collect_guards(region: &Region, visit: &mut impl FnMut(&Region)) {
    visit(region);
    match region {
        Region::Sequence { regions } => {
            for region in regions {
                collect_guards(region, visit);
            }
        }
        Region::If {
            then_arm, else_arm, ..
        } => {
            collect_guards(then_arm, visit);
            collect_guards(else_arm, visit);
        }
        Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => {
            for group in groups {
                collect_guards(&group.arm, visit);
            }
        }
        Region::Loop { body, .. } => {
            for region in body {
                collect_guards(region, visit);
            }
        }
        // A `try` of the exception table's own shape holds regions of any kind, a guarded statement
        // among them: its header's slot is declared where both are.
        Region::Try { body, catches, .. } => {
            collect_guards(body, visit);
            for clause in catches {
                collect_guards(clause.body(), visit);
            }
        }
        Region::ShortCircuitValue { .. } | Region::TwoExitReturn { .. } => {}
        Region::Guard { .. }
        | Region::Straight { .. }
        | Region::Fallback { .. }
        | Region::LoopBreak { .. }
        | Region::LoopContinue { .. } => {}
    }
}

/// Carries the exit identity of each already-proved synchronized return to value placement.
///
/// This walk reads only Guard plans produced by `guard::monitor`; it does not inspect bytecode to
/// infer monitor ownership. It is charged as a bounded region-tree walk so cancellation cannot
/// publish a partial ownership table.
fn guard_return_ownership(
    regions: &[Region],
    budget: &mut Budget,
) -> Result<BTreeMap<u32, GuardReturnOwnership>, StopReason> {
    let mut pending: Vec<&Region> = regions.iter().rev().collect();
    let mut ownership = BTreeMap::new();
    let mut ambiguous_returns = BTreeSet::new();
    while let Some(region) = pending.pop() {
        poll(budget, None)?;
        charge(budget, CountedBudgetDimension::IrItems, 1, None)?;
        match region {
            Region::Guard { plan, .. } => {
                if let guard::Shape::Monitor {
                    normal_exit_bci,
                    returns: Some(return_bci),
                    ..
                } = plan.shape()
                {
                    let record = GuardReturnOwnership {
                        normal_exit_bci: *normal_exit_bci,
                        body: plan.body(),
                    };
                    if !ambiguous_returns.contains(return_bci)
                        && ownership.insert(*return_bci, record).is_some()
                    {
                        ownership.remove(return_bci);
                        ambiguous_returns.insert(*return_bci);
                    }
                }
            }
            Region::Sequence { regions } | Region::Loop { body: regions, .. } => {
                pending.extend(regions.iter().rev());
            }
            Region::If {
                then_arm, else_arm, ..
            } => {
                pending.push(else_arm);
                pending.push(then_arm);
            }
            Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => {
                pending.extend(groups.iter().rev().map(|group| group.arm.as_ref()));
            }
            Region::Try { body, catches, .. } => {
                pending.extend(catches.iter().rev().map(|clause| clause.body()));
                pending.push(body);
            }
            Region::Straight { .. }
            | Region::TwoExitReturn { .. }
            | Region::Fallback { .. }
            | Region::ShortCircuitValue { .. }
            | Region::LoopBreak { .. }
            | Region::LoopContinue { .. } => {}
        }
    }
    Ok(ownership)
}

/// One local access of one instruction, with the region the instruction's block stands in.
#[derive(Clone)]
struct SlotUse {
    /// The innermost region whose statements hold the instruction, or `None` when the region tree
    /// does not claim its block.
    path: Option<RegionPath>,
    bci: u32,
    /// The value a read sees, when this is a read. The SSA value is the frame/exception-flow fact
    /// used to prove that every path reaching a post-region read has a real local definition.
    read: Option<ValueId>,
    /// The value a write stores; absent for a read.
    written: Option<ValueId>,
    /// The value a write **reads** as the one it stores, when the two are different values: a
    /// `…; istore` writes the slot and reads the stack, and only the second is the value whose own
    /// evidence says what type the slot holds. Absent where the write reads no stack value (an
    /// `iinc`) and where the instruction produces the value it writes (an invocation the SSA
    /// already places in the slot).
    stored: Option<ValueId>,
}

/// The path of the innermost region every use of one slot sits in.
///
/// `None` means there is no such region this layer could write a declaration in: a use whose block
/// the region tree does not claim, or **any** use inside a region that is quoted bytecode. The second
/// case is why this is not just "the common region is not a fallback": a slot whose uses span a quoted
/// region and a structured one would have to be declared at their common ancestor — but the quoted
/// text does not name the slot at all, so a declaration written above it would declare a variable the
/// produced text never uses. Such a slot keeps the declaration it has today (inside a quote: none),
/// and that run is already `Mixed`/`Fallback`.
fn declaration_region(uses: &[SlotUse], paths: &RegionPaths) -> Option<RegionPath> {
    let region = common_owner(uses)?;
    for use_ in uses {
        let path = use_.path.as_ref()?;
        if paths.fallbacks.contains(path) {
            return None;
        }
    }
    (!paths.fallbacks.contains(&region)).then_some(region)
}

/// The innermost lexical owner that contains every known access, regardless of whether one of its
/// descendants is quoted. This is separate from [`declaration_region`], which only returns a place
/// where a declaration can actually be written.
fn common_owner(uses: &[SlotUse]) -> Option<RegionPath> {
    let mut owner: Option<RegionPath> = None;
    for use_ in uses {
        let path = use_.path.as_ref()?;
        owner = Some(match owner {
            None => path.clone(),
            Some(current) => common_prefix(&current, path),
        });
    }
    owner
}

/// Whether a local has accesses on both sides of a try/catch statement.
fn crosses_exception_region(uses: &[SlotUse], paths: &RegionPaths) -> bool {
    paths.tries.iter().any(|try_path| {
        let mut clauses = BTreeSet::new();
        for use_ in uses {
            let Some(path) = use_.path.as_ref() else {
                clauses.insert(None);
                continue;
            };
            if path.starts_with(try_path) {
                // A try's prefix has no child index; protected code and each handler have one.
                clauses.insert(Some(path.get(try_path.len()).copied().unwrap_or(u32::MAX)));
            } else {
                clauses.insert(None);
            }
        }
        clauses.len() > 1
    })
}

/// A catch header owns its entry store. Reusing its slot in another clause is harmless only when
/// the SSA value written by that entry cannot reach a read outside this clause. Unknown phi inputs
/// keep the old refusal; bytecode order or a matching slot alone is not a binding proof.
fn catch_parameter_stays_in_clause(
    ssa: &SsaTable,
    operations: &Operations,
    uses: &[SlotUse],
    clause: &[u32],
    handler: &CanonicalBlockId,
    slot: u16,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    let Some(entry) = ssa.block(handler).and_then(|block| {
        block
            .instructions()
            .iter()
            .find(|instruction| instruction.bci() == handler.bci())
            .and_then(|instruction| {
                instruction
                    .writes()
                    .iter()
                    .find_map(|(written_slot, value)| {
                        (written_slot == &Slot::Local(slot)).then_some(*value)
                    })
            })
    }) else {
        return Ok(false);
    };
    let mut phis = BTreeMap::new();
    for phi in ssa.phis() {
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(handler.bci()),
        )?;
        phis.insert(phi.value(), phi.inputs());
    }
    for use_ in uses {
        if use_
            .path
            .as_ref()
            .is_some_and(|path| path.starts_with(clause))
        {
            continue;
        }
        let Some(read) = use_.read else { continue };
        let mut pending = vec![read];
        let mut visited = BTreeSet::new();
        while let Some(value) = pending.pop() {
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(use_.bci),
            )?;
            if !visited.insert(value) {
                continue;
            }
            if value == entry {
                return Ok(false);
            }
            let named = ssa.value(value);
            if let Some(replacement) = named.replaced_by() {
                pending.push(replacement);
            } else if let Definition::Phi { .. } = named.def() {
                let Some(inputs) = phis.get(&value) else {
                    return Ok(false);
                };
                for input in *inputs {
                    match input {
                        PhiInput::Value(value) => pending.push(*value),
                        PhiInput::Itself => return Ok(false),
                    }
                }
            } else if let Definition::Instruction { bci, .. } = named.def() {
                let Some(instruction) = instruction_at(ssa, *bci) else {
                    return Ok(false);
                };
                match operations.get(*bci) {
                    Some(Operation::Store { .. }) => {
                        let Some(stored) = store_operand(operations, instruction) else {
                            return Ok(false);
                        };
                        pending.push(stored);
                    }
                    Some(Operation::Load { .. }) => {
                        let Some((_, loaded)) = instruction
                            .reads()
                            .iter()
                            .find(|(slot, _)| matches!(slot, Slot::Local(_)))
                        else {
                            return Ok(false);
                        };
                        pending.push(*loaded);
                    }
                    // A fresh value cannot contain the caught reference. An operation whose
                    // relationship to it is unknown cannot prove an independent binding.
                    Some(Operation::Push(_)) => {}
                    _ => return Ok(false),
                }
            }
        }
    }
    Ok(true)
}

/// Proves that each local read's SSA value is made only from stores the region builder can present.
///
/// The frame pass has already merged normal and exception inputs into the SSA phis. Requiring every
/// leaf to be an instruction that writes this same `LocalVariable` uses that merge as the DA proof:
/// an entry value, caught reference, unrepresented predecessor, or loop self-edge is insufficient.
/// This intentionally rejects more shapes than the JVM could verify; it never infers assignment
/// from bytecode order.
fn all_reads_reach_presented_writes(
    ssa: &SsaTable,
    operations: &Operations,
    uses: &[SlotUse],
    paths: &RegionPaths,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    // A reaching local definition is useful only if its value can be written at the store. Keep
    // this proof smaller than the general expression renderer: the accepted chain is an unshared
    // int literal or a same-block load / static call / addition tree. In particular, the call must
    // remain in the protected arm containing its store, with its result consumed exactly once.
    for write in uses.iter().filter(|use_| use_.written.is_some()) {
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(write.bci),
        )?;
        let Some(stored) = write.stored else {
            return Ok(false);
        };
        let Some(written) = write.written else {
            return Ok(false);
        };
        let Definition::Instruction { block, bci } = ssa.value(written).def() else {
            return Ok(false);
        };
        if *bci != write.bci {
            return Ok(false);
        }
        let mut seen = BTreeSet::new();
        if !presented_int_store_value(
            ssa, operations, paths, stored, block, write.bci, &mut seen, budget, 0,
        )? {
            return Ok(false);
        }
        let Some(first) = seen
            .iter()
            .filter_map(|value| match ssa.value(*value).def() {
                Definition::Instruction { bci, .. } => Some(*bci),
                _ => None,
            })
            .min()
        else {
            return Ok(false);
        };
        // Inline rendering evaluates the whole tree at the store. A separate instruction between
        // its first producer and that store could have an effect or change a local the tree reads.
        // Every instruction in that interval must therefore belong to this one expression.
        let Some(block) = ssa.block(block) else {
            return Ok(false);
        };
        for instruction in block
            .instructions()
            .iter()
            .filter(|instruction| instruction.bci() >= first && instruction.bci() < write.bci)
        {
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(instruction.bci()),
            )?;
            if !seen.iter().any(|value| {
                matches!(
                    ssa.value(*value).def(),
                    Definition::Instruction { bci, .. } if *bci == instruction.bci()
                )
            }) {
                return Ok(false);
            }
        }
    }
    let writes: BTreeSet<u32> = uses
        .iter()
        .filter(|use_| use_.written.is_some())
        .map(|use_| use_.bci)
        .collect();
    if writes.is_empty() {
        return Ok(false);
    }
    let mut phis: BTreeMap<ValueId, &[PhiInput]> = BTreeMap::new();
    for phi in ssa.phis() {
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(phi.block().bci()),
        )?;
        phis.insert(phi.value(), phi.inputs());
    }
    let mut saw_read = false;
    for use_ in uses.iter().filter(|use_| use_.read.is_some()) {
        saw_read = true;
        let mut pending = vec![use_.read.expect("filtered local read")];
        let mut visited = BTreeSet::new();
        while let Some(value) = pending.pop() {
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(use_.bci),
            )?;
            if !visited.insert(value) {
                continue;
            }
            let named = ssa.value(value);
            if let Some(replaced) = named.replaced_by() {
                pending.push(replaced);
                continue;
            }
            match named.def() {
                Definition::Instruction { bci, .. } if writes.contains(bci) => {}
                Definition::Phi { .. } => {
                    let Some(inputs) = phis.get(&value) else {
                        return Ok(false);
                    };
                    if inputs.is_empty() {
                        return Ok(false);
                    }
                    for input in inputs.iter() {
                        match input {
                            PhiInput::Value(value) => pending.push(*value),
                            PhiInput::Itself => return Ok(false),
                        }
                    }
                }
                Definition::Entry { .. }
                | Definition::Instruction { .. }
                | Definition::Caught { .. } => return Ok(false),
            }
        }
    }
    Ok(saw_read)
}

/// The computed value a cross-exception store can safely present at that store's position.
/// Each producer belongs to the store's basic block and is consumed once by the next node in this
/// expression tree. This rules out moving a call across a branch, a catch boundary, or another
/// statement, and rules out printing an effectful producer twice. Other expression shapes keep the
/// existing whole-slice refusal until their own presentation rules can supply the same proof.
#[allow(clippy::too_many_arguments)]
fn presented_int_store_value(
    ssa: &SsaTable,
    operations: &Operations,
    paths: &RegionPaths,
    value: ValueId,
    store_block: &CanonicalBlockId,
    consumer: u32,
    seen: &mut BTreeSet<ValueId>,
    budget: &mut Budget,
    depth: usize,
) -> Result<bool, StopReason> {
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        1,
        Some(consumer),
    )?;
    if depth > MAX_VALUE_DEPTH || !seen.insert(value) {
        return Ok(false);
    }
    let Definition::Instruction { block, bci } = ssa.value(value).def() else {
        return Ok(false);
    };
    if !matches!(ssa.value(value).ty(), Value::Int)
        || block != store_block
        || *bci >= consumer
        || !single_use_at_with_budget(ssa, value, block, consumer, budget)?
        || paths
            .paths
            .get(block)
            .is_none_or(|path| paths.fallbacks.contains(path))
    {
        return Ok(false);
    }
    let Some(instruction) = instruction_at(ssa, *bci) else {
        return Ok(false);
    };
    let operands = stack_operands(instruction);
    let presentable = match operations.get(*bci) {
        Some(Operation::Push(ConstantValue::Int(_))) => operands.is_empty(),
        Some(Operation::Load { slot }) => {
            operands.is_empty() && local_read(instruction, *slot).is_some()
        }
        Some(Operation::Arithmetic {
            op: ArithmeticOp::Add,
        }) => operands.len() == 2,
        Some(Operation::Invoke(target)) => {
            let protected = paths.tries.iter().any(|try_path| {
                paths.paths.get(block).is_some_and(|path| {
                    path.starts_with(try_path) && path.get(try_path.len()) == Some(&0)
                })
            });
            protected
                && target.kind() == InvokeKind::Static
                && return_type(target.descriptor()) == Some(Type::Int)
                && is_java_identifier(target.name())
        }
        _ => false,
    };
    if !presentable {
        return Ok(false);
    }
    // A call and its operands are evaluated at the store; the region path is attached to the
    // basic block, so requiring the same block also preserves the exception table's try arm.
    // The final store itself stays at its original BCI and retains its normal/exception edges.
    for (_, operand) in operands {
        if !presented_int_store_value(
            ssa,
            operations,
            paths,
            operand,
            store_block,
            *bci,
            seen,
            budget,
            depth + 1,
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The longest prefix two region paths share: the innermost region that contains both.
fn common_prefix(left: &RegionPath, right: &RegionPath) -> RegionPath {
    left.iter()
        .zip(right)
        .take_while(|(left, right)| left == right)
        .map(|(step, _)| *step)
        .collect()
}

/// The path of a region nested inside the region `path` names, at the given child index.
fn child(path: &[u32], index: u32) -> RegionPath {
    let mut nested = path.to_vec();
    nested.push(index);
    nested
}

/// Which region holds each block, as the path from the method body down to it.
struct RegionPaths {
    paths: BTreeMap<CanonicalBlockId, RegionPath>,
    /// The paths whose region is a [`Region::Fallback`]: a quoted run, which holds no statements
    /// this layer could declare a local in.
    fallbacks: BTreeSet<RegionPath>,
    /// The lexical paths of exception regions, used to distinguish ordinary branch hoisting from
    /// a declaration whose value must cross a protected range or handler.
    tries: BTreeSet<RegionPath>,
    /// Catch parameters belong to their handler clause's block, not to the whole method. Keeping
    /// the parameter slot and clause path lets the declaration plan reject a reused identity that
    /// escapes that lexical owner.
    catch_parameters: BTreeMap<RegionPath, (u16, CanonicalBlockId)>,
}

/// The region tree as paths, for the slots whose declaration has to be moved.
fn region_paths(regions: &[Region]) -> RegionPaths {
    let mut paths = RegionPaths {
        paths: BTreeMap::new(),
        fallbacks: BTreeSet::new(),
        tries: BTreeSet::new(),
        catch_parameters: BTreeMap::new(),
    };
    for (index, region) in regions.iter().enumerate() {
        collect_paths(
            region,
            &child(&[], u32::try_from(index).unwrap_or(u32::MAX)),
            &mut paths,
        );
    }
    paths
}

/// Fills the path of every block one region claims, innermost region first.
///
/// A nested region is walked **before** the region that holds it, so a block both could claim
/// belongs to the inner one: a loop's header is the body's first block, and the body's statements are
/// where its statements are written while the loop's own test is written inside the loop statement.
fn collect_paths(region: &Region, path: &RegionPath, out: &mut RegionPaths) {
    match region {
        Region::If {
            then_arm, else_arm, ..
        } => {
            collect_paths(then_arm, &child(path, 0), out);
            collect_paths(else_arm, &child(path, 1), out);
        }
        Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => {
            for (index, group) in groups.iter().enumerate() {
                collect_paths(
                    &group.arm,
                    &child(path, u32::try_from(index).unwrap_or(u32::MAX)),
                    out,
                );
            }
        }
        Region::Loop { body, .. } => {
            for (index, region) in body.iter().enumerate() {
                collect_paths(
                    region,
                    &child(path, u32::try_from(index).unwrap_or(u32::MAX)),
                    out,
                );
            }
        }
        // A `try`'s protected range and each of its clause bodies are regions of their own: every
        // one of them writes statements, so a slot used inside one is declared there.
        Region::Try { body, catches, .. } => {
            out.tries.insert(path.clone());
            collect_paths(body, &child(path, 0), out);
            for (index, clause) in catches.iter().enumerate() {
                out.catch_parameters.insert(
                    child(path, u32::try_from(index + 1).unwrap_or(u32::MAX)),
                    (clause.parameter(), clause.handler().clone()),
                );
                collect_paths(
                    clause.body(),
                    &child(path, u32::try_from(index + 1).unwrap_or(u32::MAX)),
                    out,
                );
            }
        }
        Region::Sequence { regions } => {
            for (index, region) in regions.iter().enumerate() {
                collect_paths(
                    region,
                    &child(path, u32::try_from(index).unwrap_or(u32::MAX)),
                    out,
                );
            }
        }
        Region::Straight { .. }
        | Region::TwoExitReturn { .. }
        | Region::Fallback { .. }
        | Region::ShortCircuitValue { .. }
        | Region::Guard { .. }
        | Region::LoopBreak { .. }
        | Region::LoopContinue { .. } => {}
    }
    if matches!(region, Region::Fallback { .. }) {
        out.fallbacks.insert(path.clone());
    }
    for block in own_blocks(region) {
        out.paths.entry(block).or_insert_with(|| path.clone());
    }
}

/// A top-level uncovered quote is outside every execution path only if the graph has no usable
/// entry into it. The canonical graph also states range-only exception edges for handlers whose
/// protected code cannot throw; those edges do not make the quoted handler live (P3 2.15).
fn dead_uncovered_quote(
    blocks: &[CanonicalBlockId],
    canonical: &CanonicalCfg,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    let mut held = BTreeSet::new();
    for block in blocks {
        poll(budget, Some(block.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(block.bci()),
        )?;
        held.insert(block);
    }
    if held.is_empty()
        || canonical
            .blocks()
            .first()
            .is_some_and(|entry| held.contains(entry.id()))
    {
        return Ok(false);
    }
    for edge in canonical.edges() {
        poll(budget, Some(edge.to().bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(edge.to().bci()),
        )?;
        if !held.contains(edge.to()) || held.contains(edge.from()) {
            continue;
        }
        match edge.kind() {
            CanonicalEdgeKind::Exception { handler_ordinal } => {
                for site in canonical.throw_sites() {
                    poll(budget, Some(site.bci()))?;
                    charge(
                        budget,
                        CountedBudgetDimension::AnalysisSteps,
                        1,
                        Some(site.bci()),
                    )?;
                    if site.block() == edge.from() && site.handlers().contains(&handler_ordinal) {
                        return Ok(false);
                    }
                }
            }
            CanonicalEdgeKind::Normal
            | CanonicalEdgeKind::Call { .. }
            | CanonicalEdgeKind::Return { .. } => return Ok(false),
        }
    }
    Ok(true)
}

/// The blocks a region writes the statements of, as opposed to the ones its nested regions own.
fn own_blocks(region: &Region) -> Vec<CanonicalBlockId> {
    match region {
        Region::Straight { blocks } => blocks.clone(),
        Region::Sequence { .. } => Vec::new(),
        Region::If { prefix, branch, .. } => {
            let mut blocks = prefix.clone();
            blocks.push(branch.clone());
            blocks
        }
        Region::Switch { prefix, branch, .. } => {
            let mut blocks = prefix.clone();
            blocks.push(branch.clone());
            blocks
        }
        Region::StringSwitch { dispatch, .. } => dispatch.clone(),
        // The test block is the condition written *inside* the loop statement; the header belongs
        // to the body when the body claims it (`collect_paths` walks the body first).
        Region::Loop { header, test, .. } => vec![header.clone(), test.clone()],
        Region::LoopBreak { .. } | Region::LoopContinue { .. } => Vec::new(),
        Region::Fallback { blocks, .. } => blocks.clone(),
        Region::ShortCircuitValue {
            prefix,
            tests,
            gateways,
            true_producer,
            false_producer,
            consumer,
            ..
        } => {
            let mut blocks = prefix.clone();
            for (block, _) in tests {
                if !blocks.contains(block) {
                    blocks.push(block.clone());
                }
            }
            for (block, _) in gateways {
                if !blocks.contains(block) {
                    blocks.push(block.clone());
                }
            }
            for block in [true_producer, false_producer, consumer] {
                if !blocks.contains(block) {
                    blocks.push(block.clone());
                }
            }
            blocks
        }
        Region::TwoExitReturn { .. } => region.blocks().into_iter().cloned().collect(),
        // A guarded statement writes its own header and the blocks of its body; every other block it
        // claims — the closes, the handlers, the later resources' initialisations — produces no
        // statement of its own, which is exactly what keeps a close from running twice.
        Region::Guard { prefix, plan } => {
            let mut blocks = prefix.clone();
            blocks.extend(plan.owned().iter().cloned());
            blocks
        }
        // A `try` writes its protected range and every clause body: the handler entry is where a
        // clause body starts, and the store that fills the clause's parameter is written by the
        // clause's own header, not as a statement of the body.
        Region::Try {
            prefix,
            lead: _,
            body,
            catches,
        } => {
            let mut blocks = prefix.clone();
            blocks.extend(body.blocks().into_iter().cloned());
            for clause in catches {
                blocks.extend(clause.body().blocks().into_iter().cloned());
            }
            blocks
        }
    }
}

/// Physical instruction starts a fallback region could not attach to a canonical block.
fn unaccounted_region_bcis(region: &Region) -> Vec<u32> {
    let mut bcis = Vec::new();
    match region {
        Region::Sequence { regions } => {
            for region in regions {
                bcis.extend(unaccounted_region_bcis(region));
            }
        }
        Region::If {
            then_arm, else_arm, ..
        } => {
            bcis.extend(unaccounted_region_bcis(then_arm));
            bcis.extend(unaccounted_region_bcis(else_arm));
        }
        Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => {
            for group in groups {
                bcis.extend(unaccounted_region_bcis(&group.arm));
            }
        }
        Region::Loop { body, .. } => {
            for region in body {
                bcis.extend(unaccounted_region_bcis(region));
            }
        }
        Region::Try { body, catches, .. } => {
            bcis.extend(unaccounted_region_bcis(body));
            for clause in catches {
                bcis.extend(unaccounted_region_bcis(clause.body()));
            }
        }
        Region::Fallback { reason, .. } => bcis.extend_from_slice(reason.unaccounted()),
        Region::ShortCircuitValue { .. } | Region::TwoExitReturn { .. } => {}
        Region::Straight { .. }
        | Region::Guard { .. }
        | Region::LoopBreak { .. }
        | Region::LoopContinue { .. } => {}
    }
    bcis
}

/// The result of the bounded precondition check for one conditional value.
///
/// Task 2.1 establishes only that the existing `Region::If` and SSA artifacts state one safe
/// two-arm stack value. The result is deliberately private and is not published or consumed by the
/// builder until the expression work in task 2.2.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConditionalValueProof {
    pub(crate) branch: CanonicalBlockId,
    pub(crate) branch_bci: u32,
    pub(crate) join: CanonicalBlockId,
    pub(crate) phi: ValueId,
    pub(crate) stack_depth: u32,
    pub(crate) when_true: ValueId,
    pub(crate) when_false: ValueId,
    pub(crate) consumer_bci: u32,
}

/// Why one `Region::If` did not prove a two-arm stack value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConditionalValueRefusal {
    NotIf,
    NoJoin,
    NonStraightArm,
    EmptyOrOverlappingArms,
    BranchEdges,
    IncomingEdges,
    StackPhiCount,
    PhiIdentity,
    PhiInputCount,
    PhiSelfInput,
    InputPredecessor,
    InputOutsideArm,
    ProducerUse,
    PhiUseCount,
    PhiConsumer,
}

/// A proof attempt is data, not a syntax node or a public diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ConditionalValueAttempt {
    Proved(ConditionalValueProof),
    Refused(ConditionalValueRefusal),
}

/// The one consumer owned by a short-circuit region. This is deliberately
/// separate from the ordinary, disjoint-arm conditional proof and is not an emission plan.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ShortCircuitConsumer {
    Field(String, String, String),
    InstanceField(String, String, String, ValueId),
    BooleanArray(ValueId, ValueId),
    Return,
    Invoke(CallTarget),
    Local(u16, ValueId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ShortCircuitValueProof {
    test_bcis: Vec<u32>,
    true_producer: ValueId,
    false_producer: ValueId,
    phi: ValueId,
    stack_depth: u32,
    consumer_bci: u32,
    consumer: ShortCircuitConsumer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShortCircuitValueRefusal {
    NotShortCircuit,
    MissingSsa,
    Edges,
    Test,
    TestEffect,
    Producer,
    Phi,
    Consumer,
    Field,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ShortCircuitValueAttempt {
    Proved(ShortCircuitValueProof),
    Refused(ShortCircuitValueRefusal),
}

enum ConditionalValueBuildError {
    Refused(String),
    Stop(StopReason),
}

/// A value renderer can refuse a Java shape, or stop the whole build on its shared budget.
#[derive(Clone)]
enum ValueRenderFailure {
    Refusal(String),
    Stop(StopReason),
}

impl From<String> for ValueRenderFailure {
    fn from(reason: String) -> Self {
        Self::Refusal(reason)
    }
}

impl From<&String> for ValueRenderFailure {
    fn from(reason: &String) -> Self {
        Self::Refusal(reason.clone())
    }
}

impl From<&str> for ValueRenderFailure {
    fn from(reason: &str) -> Self {
        Self::Refusal(reason.to_owned())
    }
}

impl From<&ValueRenderFailure> for ValueRenderFailure {
    fn from(failure: &ValueRenderFailure) -> Self {
        failure.clone()
    }
}

impl From<StopReason> for ValueRenderFailure {
    fn from(stop: StopReason) -> Self {
        Self::Stop(stop)
    }
}

#[derive(Clone, Debug)]
enum ConditionalBranchPlan {
    Folded(ValueId),
    Refused(String, Option<Vec<u32>>),
}

impl From<String> for ConditionalValueBuildError {
    fn from(reason: String) -> Self {
        Self::Refused(reason)
    }
}

impl From<ValueRenderFailure> for ConditionalValueBuildError {
    fn from(failure: ValueRenderFailure) -> Self {
        match failure {
            ValueRenderFailure::Refusal(reason) => Self::Refused(reason),
            ValueRenderFailure::Stop(stop) => Self::Stop(stop),
        }
    }
}

/// Proves one narrow conditional-value shape from the region, canonical edges and published SSA.
///
/// This intentionally accepts only a straight run in each arm. The join must have exactly two
/// normal predecessors, each inside a different arm, and one stack Phi whose two inputs are the
/// values those predecessor blocks hand over. Direct consumers read the Phi once in the join. A
/// carried invocation value also needs a separately proved following conditional and exact
/// same-value stack transfer. Local Phis, loop back edges and exception edges remain outside it.
pub(crate) fn prove_conditional_value(
    region: &Region,
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    operations: &Operations,
    budget: &mut Budget,
) -> Result<ConditionalValueAttempt, StopReason> {
    prove_conditional_value_with_forward(region, canonical, ssa, operations, None, budget)
}

fn prove_conditional_value_with_forward(
    region: &Region,
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    operations: &Operations,
    forward: Option<(&ConditionalValueProof, &Region)>,
    budget: &mut Budget,
) -> Result<ConditionalValueAttempt, StopReason> {
    let Region::If {
        branch,
        branch_bci,
        then_arm,
        else_arm,
        join: Some(join),
        ..
    } = region
    else {
        return Ok(ConditionalValueAttempt::Refused(
            if matches!(region, Region::If { .. }) {
                ConditionalValueRefusal::NoJoin
            } else {
                ConditionalValueRefusal::NotIf
            },
        ));
    };
    let (
        Region::Straight {
            blocks: then_blocks,
        },
        Region::Straight {
            blocks: else_blocks,
        },
    ) = (then_arm.as_ref(), else_arm.as_ref())
    else {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::NonStraightArm,
        ));
    };
    if then_blocks.is_empty() || else_blocks.is_empty() {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::EmptyOrOverlappingArms,
        ));
    }
    charge(
        budget,
        CountedBudgetDimension::IrItems,
        u64::try_from(then_blocks.len().saturating_add(else_blocks.len())).unwrap_or(u64::MAX),
        Some(*branch_bci),
    )?;
    let then_blocks: BTreeSet<_> = then_blocks.iter().cloned().collect();
    let else_blocks: BTreeSet<_> = else_blocks.iter().cloned().collect();
    if !then_blocks.is_disjoint(&else_blocks) {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::EmptyOrOverlappingArms,
        ));
    }

    // Charge a conservative upper bound before scanning the graph and SSA vectors. The region
    // pass already bounds block count; this proof repeats only bounded table walks and two
    // predecessor lookups.
    let instruction_count = ssa
        .blocks()
        .iter()
        .map(|block| block.instructions().len())
        .sum::<usize>();
    let scan_items = canonical
        .edges()
        .len()
        .saturating_mul(2)
        .saturating_add(ssa.phis().len())
        .saturating_add(ssa.blocks().len().saturating_mul(5))
        .saturating_add(instruction_count)
        .saturating_add(then_blocks.len())
        .saturating_add(else_blocks.len());
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(scan_items).unwrap_or(u64::MAX),
        Some(*branch_bci),
    )?;
    let mut branch_targets = Vec::new();
    let mut incoming = Vec::new();
    let Some((_, taken_target)) = operations.get(*branch_bci).and_then(Operation::comparison)
    else {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::BranchEdges,
        ));
    };
    for edge in canonical.edges() {
        if edge.from() == branch {
            if edge.kind() != CanonicalEdgeKind::Normal {
                return Ok(ConditionalValueAttempt::Refused(
                    ConditionalValueRefusal::BranchEdges,
                ));
            }
            charge(
                budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(*branch_bci),
            )?;
            branch_targets.push(edge.to().clone());
        }
        if edge.to() == join {
            if edge.kind() != CanonicalEdgeKind::Normal {
                return Ok(ConditionalValueAttempt::Refused(
                    ConditionalValueRefusal::IncomingEdges,
                ));
            }
            charge(
                budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(*branch_bci),
            )?;
            incoming.push(edge.from().clone());
        }
    }
    if branch_targets.len() != 2 || branch_targets[0] == branch_targets[1] {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::BranchEdges,
        ));
    }
    // Region recovery orders an if's arms by the condition it emits: true continues along
    // fall-through (`then_arm`), false follows the branch target (`else_arm`). Check those exact
    // identities here; mere membership of one successor in each arm would allow a swapped Phi.
    let then_entry = match then_arm.as_ref() {
        Region::Straight { blocks } => &blocks[0],
        _ => unreachable!("straight-arm shape was checked above"),
    };
    let else_entry = match else_arm.as_ref() {
        Region::Straight { blocks } => &blocks[0],
        _ => unreachable!("straight-arm shape was checked above"),
    };
    let taken_edges: Vec<_> = branch_targets
        .iter()
        .filter(|target| target.bci() == taken_target && target.path() == branch.path())
        .collect();
    if taken_edges.len() != 1
        || taken_edges[0] != else_entry
        || !branch_targets
            .iter()
            .any(|target| target == then_entry && target.path() == branch.path())
    {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::BranchEdges,
        ));
    }
    if incoming.len() != 2 || incoming[0] == incoming[1] {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::IncomingEdges,
        ));
    }
    charge(
        budget,
        CountedBudgetDimension::IrItems,
        u64::try_from(incoming.len()).unwrap_or(u64::MAX),
        Some(*branch_bci),
    )?;
    let side_facts = incoming
        .iter()
        .map(|predecessor| {
            match (
                then_blocks.contains(predecessor),
                else_blocks.contains(predecessor),
            ) {
                (true, false) => Some(true),
                (false, true) => Some(false),
                _ => None,
            }
        })
        .collect::<Option<Vec<_>>>();
    let Some(incoming_sides) = side_facts else {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::IncomingEdges,
        ));
    };
    if incoming_sides[0] == incoming_sides[1] {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::IncomingEdges,
        ));
    }

    // A straight arm has one normal successor per block: the next listed block, or the join at
    // its end. It also has no outside entry beyond the branch edge to its first block. This closes
    // the gap between Region's claimed arm and the complete canonical graph, including edges from
    // otherwise unclaimed blocks and exception edges.
    let mut arm_incoming: BTreeMap<CanonicalBlockId, Vec<CanonicalBlockId>> = BTreeMap::new();
    let mut arm_outgoing: BTreeMap<CanonicalBlockId, Vec<(CanonicalEdgeKind, CanonicalBlockId)>> =
        BTreeMap::new();
    for edge in canonical.edges() {
        if then_blocks.contains(edge.to()) || else_blocks.contains(edge.to()) {
            arm_incoming
                .entry(edge.to().clone())
                .or_default()
                .push(edge.from().clone());
            if edge.kind() != CanonicalEdgeKind::Normal {
                return Ok(ConditionalValueAttempt::Refused(
                    ConditionalValueRefusal::IncomingEdges,
                ));
            }
        }
        if then_blocks.contains(edge.from()) || else_blocks.contains(edge.from()) {
            arm_outgoing
                .entry(edge.from().clone())
                .or_default()
                .push((edge.kind(), edge.to().clone()));
        }
    }
    for (blocks, side_entry) in [
        (then_arm.as_ref(), then_entry),
        (else_arm.as_ref(), else_entry),
    ] {
        let Region::Straight { blocks } = blocks else {
            unreachable!("straight-arm shape was checked above")
        };
        for (index, block) in blocks.iter().enumerate() {
            let expected_predecessor = if index == 0 {
                branch
            } else {
                &blocks[index - 1]
            };
            if arm_incoming.get(block).map(Vec::as_slice)
                != Some(std::slice::from_ref(expected_predecessor))
            {
                return Ok(ConditionalValueAttempt::Refused(
                    ConditionalValueRefusal::IncomingEdges,
                ));
            }
            let expected_successor = blocks.get(index + 1).unwrap_or(join);
            if arm_outgoing.get(block).map(Vec::as_slice)
                != Some(&[(CanonicalEdgeKind::Normal, expected_successor.clone())][..])
            {
                return Ok(ConditionalValueAttempt::Refused(
                    ConditionalValueRefusal::IncomingEdges,
                ));
            }
        }
        debug_assert_eq!(blocks.first(), Some(side_entry));
    }

    charge(
        budget,
        CountedBudgetDimension::IrItems,
        u64::try_from(ssa.phis().len()).unwrap_or(u64::MAX),
        Some(*branch_bci),
    )?;
    let stack_phis: Vec<_> = ssa
        .phis()
        .iter()
        .filter(|phi| phi.block() == join && matches!(phi.slot(), Slot::Stack(_)))
        .collect();
    // An enclosing arithmetic/call expression can carry a value across both arms at another
    // stack depth. Such a Phi is an identity transfer, not a second conditional value.
    let varying_stack_phis: Vec<_> = stack_phis
        .iter()
        .copied()
        .filter(|phi| {
            let [left, right] = phi.inputs() else {
                return true;
            };
            left != right
        })
        .collect();
    let [phi] = varying_stack_phis.as_slice() else {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::StackPhiCount,
        ));
    };
    if stack_phis.iter().any(|other| {
        other.value() != phi.value()
            && !matches!(other.inputs(), [PhiInput::Value(left), PhiInput::Value(right)] if left == right)
    }) {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::StackPhiCount,
        ));
    }
    let Slot::Stack(stack_depth) = phi.slot() else {
        unreachable!("the candidate was filtered to stack slots")
    };
    let phi_value = ssa.value(phi.value());
    if phi_value.replaced_by().is_some()
        || !matches!(phi_value.def(), Definition::Phi { block, slot } if block == join && *slot == phi.slot())
    {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::PhiIdentity,
        ));
    }
    if phi.inputs().len() != 2 {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::PhiInputCount,
        ));
    }

    let mut arm_values = [None, None];
    for input in phi.inputs() {
        let PhiInput::Value(value) = input else {
            return Ok(ConditionalValueAttempt::Refused(
                ConditionalValueRefusal::PhiSelfInput,
            ));
        };
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            u64::try_from(incoming.len()).unwrap_or(u64::MAX),
            Some(*branch_bci),
        )?;
        let matching_predecessors: Vec<_> = incoming
            .iter()
            .zip(&incoming_sides)
            .filter(|(predecessor, _)| {
                ssa.block(predecessor).is_some_and(|block| {
                    block
                        .exit()
                        .iter()
                        .any(|(slot, exit_value)| *slot == phi.slot() && exit_value == value)
                })
            })
            .collect();
        let [(_, is_then)] = matching_predecessors.as_slice() else {
            return Ok(ConditionalValueAttempt::Refused(
                ConditionalValueRefusal::InputPredecessor,
            ));
        };
        let is_then = **is_then;
        let Definition::Instruction { block, .. } = ssa.value(*value).def() else {
            return Ok(ConditionalValueAttempt::Refused(
                ConditionalValueRefusal::InputOutsideArm,
            ));
        };
        if !(if is_then {
            then_blocks.contains(block)
        } else {
            else_blocks.contains(block)
        }) {
            return Ok(ConditionalValueAttempt::Refused(
                ConditionalValueRefusal::InputOutsideArm,
            ));
        }
        let uses = ssa.value(*value).uses();
        if uses.len() != 1 || uses[0].block() != join || uses[0].bci().is_some() {
            return Ok(ConditionalValueAttempt::Refused(
                ConditionalValueRefusal::ProducerUse,
            ));
        }
        let target = if is_then {
            &mut arm_values[0]
        } else {
            &mut arm_values[1]
        };
        if target.replace(*value).is_some() {
            return Ok(ConditionalValueAttempt::Refused(
                ConditionalValueRefusal::InputPredecessor,
            ));
        }
    }
    let [Some(when_true), Some(when_false)] = arm_values else {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::InputPredecessor,
        ));
    };

    let uses = phi_value.uses();
    let carried_consumer = if let Some((next, next_region)) = forward {
        if next.branch == *join {
            prove_carried_conditional_use(
                phi.value(),
                phi.slot(),
                next,
                next_region,
                canonical,
                ssa,
                budget,
            )?
        } else {
            None
        }
    } else {
        None
    };
    let [use_record] = uses else {
        if let Some(consumer_bci) = carried_consumer {
            return Ok(ConditionalValueAttempt::Proved(ConditionalValueProof {
                branch: branch.clone(),
                branch_bci: *branch_bci,
                join: join.clone(),
                phi: phi.value(),
                stack_depth,
                when_true,
                when_false,
                consumer_bci,
            }));
        }
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::PhiUseCount,
        ));
    };
    let Some(consumer_bci) = use_record.bci() else {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::PhiConsumer,
        ));
    };
    let Some(join_block) = ssa.block(join) else {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::PhiConsumer,
        ));
    };
    if use_record.block() != join
        || join_block
            .instructions()
            .iter()
            .filter(|instruction| instruction.bci() == consumer_bci)
            .flat_map(|instruction| instruction.reads().iter())
            .filter(|(_, value)| *value == phi.value())
            .count()
            != 1
    {
        return Ok(ConditionalValueAttempt::Refused(
            ConditionalValueRefusal::PhiConsumer,
        ));
    }

    Ok(ConditionalValueAttempt::Proved(ConditionalValueProof {
        branch: branch.clone(),
        branch_bci: *branch_bci,
        join: join.clone(),
        phi: phi.value(),
        stack_depth,
        when_true,
        when_false,
        consumer_bci,
    }))
}

/// A previously joined argument may remain in its exact stack slot while the next independently
/// proved conditional computes the following argument. The final same-value Phi is trivial in SSA:
/// its instruction use has moved to the earlier value, but its operand use remains recorded.
fn prove_carried_conditional_use(
    value: ValueId,
    slot: Slot,
    next: &ConditionalValueProof,
    next_region: &Region,
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    budget: &mut Budget,
) -> Result<Option<u32>, StopReason> {
    let Region::If {
        prefix,
        then_arm,
        else_arm,
        ..
    } = next_region
    else {
        return Ok(None);
    };
    let (
        Region::Straight {
            blocks: then_blocks,
        },
        Region::Straight {
            blocks: else_blocks,
        },
    ) = (then_arm.as_ref(), else_arm.as_ref())
    else {
        return Ok(None);
    };
    if !prefix.is_empty() || next.join == next.branch {
        return Ok(None);
    }
    let Some(branch_block) = ssa.block(&next.branch) else {
        return Ok(None);
    };
    if !branch_block.exit().contains(&(slot, value)) {
        return Ok(None);
    }
    let mut arm_blocks = then_blocks.iter().chain(else_blocks.iter());
    for block in &mut arm_blocks {
        poll(budget, Some(block.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(block.bci()),
        )?;
        let Some(names) = ssa.block(block) else {
            return Ok(None);
        };
        if !names.entry().contains(&(slot, value))
            || !names.exit().contains(&(slot, value))
            || names.instructions().iter().any(|instruction| {
                instruction.reads().iter().any(|(_, read)| *read == value)
                    || instruction
                        .writes()
                        .iter()
                        .any(|(written, _)| *written == slot)
            })
        {
            return Ok(None);
        }
    }
    // The following conditional's ordinary proof closes its branch and both arms. Recheck the
    // physical predecessor set at the same-value merge: every incoming edge must hand the same
    // original SSA identity to the same stack slot.
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(canonical.edges().len().saturating_add(ssa.phis().len())).unwrap_or(u64::MAX),
        Some(next.consumer_bci),
    )?;
    let incoming = canonical
        .edges()
        .iter()
        .filter(|edge| edge.to() == &next.join)
        .collect::<Vec<_>>();
    if incoming.len() != 2
        || incoming.iter().any(|edge| {
            edge.kind() != CanonicalEdgeKind::Normal
                || !then_blocks
                    .iter()
                    .chain(else_blocks)
                    .any(|block| block == edge.from())
                || !ssa
                    .block(edge.from())
                    .is_some_and(|block| block.exit().contains(&(slot, value)))
        })
    {
        return Ok(None);
    }
    let Some(forward_phi) = ssa
        .phis()
        .iter()
        .find(|phi| phi.block() == &next.join && phi.slot() == slot)
    else {
        return Ok(None);
    };
    if forward_phi.inputs() != [PhiInput::Value(value), PhiInput::Value(value)]
        || ssa.value(forward_phi.value()).replaced_by() != Some(value)
        || !ssa.value(forward_phi.value()).uses().is_empty()
    {
        return Ok(None);
    }
    let Some(final_block) = ssa.block(&next.join) else {
        return Ok(None);
    };
    if !final_block.entry().contains(&(slot, value)) {
        return Ok(None);
    }
    let uses = ssa.value(value).uses();
    if uses.len() != 3
        || uses
            .iter()
            .filter(|usage| usage.block() == &next.join && usage.bci().is_none())
            .count()
            != 2
        || !uses
            .iter()
            .any(|usage| usage.block() == &next.join && usage.bci() == Some(next.consumer_bci))
        || final_block
            .instructions()
            .iter()
            .filter(|instruction| instruction.bci() == next.consumer_bci)
            .flat_map(|instruction| instruction.reads())
            .filter(|(read_slot, read_value)| *read_slot == slot && *read_value == value)
            .count()
            != 1
    {
        return Ok(None);
    }
    Ok(Some(next.consumer_bci))
}

/// Checks either shared-producer diamond without changing the complete-quote fallback.
/// The charge covers two bounded edge passes, all SSA instructions and phis, plus their local
/// value walks. A refusal never stores a partial expression or suppresses a region instruction.
#[allow(dead_code)] // task 1.3 consumes the proof; task 1.2 keeps the complete quote
fn prove_short_circuit_value(
    region: &Region,
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    operations: &Operations,
    return_type: Option<&Type>,
    budget: &mut Budget,
) -> Result<ShortCircuitValueAttempt, StopReason> {
    let Region::ShortCircuitValue {
        prefix,
        tests,
        gateways,
        true_producer,
        false_producer,
        consumer,
        consumer_bci,
        ..
    } = region
    else {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::NotShortCircuit,
        ));
    };
    let Some((_, first_bci)) = tests.first() else {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Edges,
        ));
    };
    let at = Some(*first_bci);
    poll(budget, at)?;
    let instruction_count = ssa
        .blocks()
        .iter()
        .map(|block| block.instructions().len())
        .sum::<usize>();
    let operand_count = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .map(|instruction| {
            instruction
                .reads()
                .len()
                .saturating_add(instruction.writes().len())
        })
        .sum::<usize>();
    let scan_items = canonical
        .edges()
        .len()
        .saturating_mul(2)
        .saturating_add(instruction_count.saturating_mul(3))
        .saturating_add(operand_count.saturating_mul(2))
        .saturating_add(ssa.phis().len().saturating_mul(2))
        .saturating_add(ssa.blocks().len().saturating_mul(5))
        .saturating_add(tests.len().saturating_mul(8))
        .saturating_add(gateways.len().saturating_mul(8))
        .saturating_add(tests.len().saturating_mul(tests.len()).saturating_mul(4))
        .saturating_add(32);
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(scan_items).unwrap_or(u64::MAX),
        at,
    )?;
    if tests.len() < 2 || tests.len() > MAX_VALUE_DEPTH {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Edges,
        ));
    }
    let mut members = tests.iter().map(|(block, _)| block).collect::<Vec<_>>();
    members.extend(gateways.iter().map(|(block, _)| block));
    members.extend([true_producer, false_producer, consumer]);
    if members
        .iter()
        .enumerate()
        .any(|(index, block)| members[index + 1..].contains(block))
    {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Edges,
        ));
    }
    let mut incoming: BTreeMap<&CanonicalBlockId, Vec<&CanonicalBlockId>> = BTreeMap::new();
    let mut outgoing: BTreeMap<&CanonicalBlockId, Vec<&CanonicalBlockId>> = BTreeMap::new();
    for edge in canonical.edges() {
        if members.contains(&edge.from()) || members.contains(&edge.to()) {
            if edge.kind() != CanonicalEdgeKind::Normal {
                return Ok(ShortCircuitValueAttempt::Refused(
                    ShortCircuitValueRefusal::Edges,
                ));
            }
            if members.contains(&edge.from()) {
                outgoing.entry(edge.from()).or_default().push(edge.to());
            }
            if members.contains(&edge.to()) {
                incoming.entry(edge.to()).or_default().push(edge.from());
            }
        }
    }
    let same = |actual: Option<&Vec<&CanonicalBlockId>>, expected: &[&CanonicalBlockId]| {
        actual.is_some_and(|actual| {
            actual.len() == expected.len()
                && expected
                    .iter()
                    .all(|block| actual.iter().filter(|item| *item == block).count() == 1)
        })
    };
    let Region::ShortCircuitValue { test_edges, .. } = region else {
        unreachable!()
    };
    if test_edges.len() != tests.len() {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Edges,
        ));
    }
    for ((block, _), (fallthrough, taken)) in tests.iter().zip(test_edges) {
        if !same(outgoing.get(block), &[fallthrough, taken])
            || block.path() != fallthrough.path()
            || block.path() != taken.path()
            || fallthrough.bci() <= block.bci()
            || taken.bci() <= block.bci()
        {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Edges,
            ));
        }
    }
    for (block, _) in tests.iter().skip(1) {
        let mut expected = tests
            .iter()
            .zip(test_edges)
            .filter(|(_, (fallthrough, taken))| fallthrough == block || taken == block)
            .map(|((predecessor, _), _)| predecessor)
            .collect::<Vec<_>>();
        expected.extend(
            gateways
                .iter()
                .filter(|(_, successor)| successor == block)
                .map(|(gateway, _)| gateway),
        );
        if expected.is_empty() || !same(incoming.get(block), &expected) {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Edges,
            ));
        }
    }
    for (gateway, successor) in gateways {
        let mut expected = tests
            .iter()
            .zip(test_edges)
            .filter(|(_, (fallthrough, taken))| fallthrough == gateway || taken == gateway)
            .map(|((predecessor, _), _)| predecessor)
            .collect::<Vec<_>>();
        expected.extend(
            gateways
                .iter()
                .filter(|(_, target)| target == gateway)
                .map(|(prior, _)| prior),
        );
        let Some(gateway_ssa) = ssa.block(gateway) else {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::MissingSsa,
            ));
        };
        let [transfer] = gateway_ssa.instructions() else {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Edges,
            ));
        };
        if expected.len() != 1
            || !same(incoming.get(gateway), &expected)
            || !same(outgoing.get(gateway), &[successor])
            || successor.bci() <= gateway.bci()
            || successor.path() != gateway.path()
            || !matches!(transfer.opcode(), 0xa7 | 0xc8)
            || !matches!(operations.get(transfer.bci()), Some(Operation::Transfer))
            || !transfer.reads().is_empty()
            || !transfer.writes().is_empty()
        {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Edges,
            ));
        }
    }
    for producer in [true_producer, false_producer] {
        let mut expected = tests
            .iter()
            .zip(test_edges)
            .filter(|(_, (fallthrough, taken))| fallthrough == producer || taken == producer)
            .map(|((predecessor, _), _)| predecessor)
            .collect::<Vec<_>>();
        expected.extend(
            gateways
                .iter()
                .filter(|(_, successor)| successor == producer)
                .map(|(gateway, _)| gateway),
        );
        if expected.is_empty()
            || !same(incoming.get(producer), &expected)
            || !same(outgoing.get(producer), &[consumer])
        {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Edges,
            ));
        }
    }
    if !same(incoming.get(consumer), &[true_producer, false_producer])
        || outgoing.get(consumer).is_some_and(|edges| edges.len() > 1)
    {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Edges,
        ));
    }

    let (Some(true_ssa), Some(false_ssa), Some(consumer_ssa)) = (
        ssa.block(true_producer),
        ssa.block(false_producer),
        ssa.block(consumer),
    ) else {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::MissingSsa,
        ));
    };
    for (index, ((block, branch_bci), (fallthrough, taken))) in
        tests.iter().zip(test_edges).enumerate()
    {
        let Some(block_ssa) = ssa.block(block) else {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::MissingSsa,
            ));
        };
        let Some((comparison, target)) =
            operations.get(*branch_bci).and_then(Operation::comparison)
        else {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Test,
            ));
        };
        if target != taken.bci()
            || block.path() != taken.path()
            || block.path() != fallthrough.path()
            || block_ssa.instructions().last().map(SsaInstruction::bci) != Some(*branch_bci)
        {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Test,
            ));
        }
        let Some(branch_instruction) = block_ssa.instructions().last() else {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Test,
            ));
        };
        let instruction_index: HashMap<_, _> = block_ssa
            .instructions()
            .iter()
            .map(|instruction| (instruction.bci(), instruction))
            .collect();
        if instruction_index.len() != block_ssa.instructions().len() {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Test,
            ));
        }
        let mut dependencies = HashSet::new();
        let mut active = HashSet::new();
        if branch_instruction.reads().len() != if comparison.reads_two() { 2 } else { 1 }
            || branch_instruction
                .reads()
                .iter()
                .any(|(slot, _)| !matches!(slot, Slot::Stack(_)))
        {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Test,
            ));
        }
        for (_, value) in branch_instruction.reads() {
            if !short_circuit_test_sources(
                *value,
                block,
                &instruction_index,
                ssa,
                operations,
                &mut dependencies,
                &mut active,
                0,
                *branch_bci,
            ) {
                return Ok(ShortCircuitValueAttempt::Refused(
                    ShortCircuitValueRefusal::TestEffect,
                ));
            }
        }
        let first_test_bci = dependencies.iter().copied().min().unwrap_or(*branch_bci);
        if block_ssa
            .instructions()
            .iter()
            .filter(|instruction| {
                instruction.bci() >= first_test_bci && instruction.bci() < *branch_bci
            })
            .any(|instruction| !dependencies.contains(&instruction.bci()))
            || (index > 0
                && block_ssa.instructions().iter().any(|instruction| {
                    instruction.bci() < *branch_bci && !dependencies.contains(&instruction.bci())
                }))
        {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::TestEffect,
            ));
        }
    }

    let producer = |block: &CanonicalBlockId, names: &jarde_jvm::method_ir::SsaBlock, expected| {
        let (first, tail) = names.instructions().split_first()?;
        if !matches!(operations.get(first.bci()), Some(Operation::Push(ConstantValue::Int(value))) if *value == expected)
            || !tail.iter().all(|instruction| {
                matches!(operations.get(instruction.bci()), Some(Operation::Transfer))
            })
        {
            return None;
        }
        let [(Slot::Stack(depth), value)] = first.writes() else {
            return None;
        };
        if !matches!(ssa.value(*value).def(), Definition::Instruction { block: origin, bci } if origin == block && *bci == first.bci())
            || ssa.value(*value).replaced_by().is_some()
            || !matches!(ssa.value(*value).uses(), [usage] if usage.block() == consumer && usage.bci().is_none())
        {
            return None;
        }
        Some((*depth, *value))
    };
    let (Some((true_depth, true_value)), Some((false_depth, false_value))) = (
        producer(true_producer, true_ssa, 1),
        producer(false_producer, false_ssa, 0),
    ) else {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Producer,
        ));
    };
    if true_value == false_value || true_depth != false_depth {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Producer,
        ));
    }
    let slot = Slot::Stack(true_depth);
    if true_ssa
        .exit()
        .iter()
        .filter(|(s, _)| *s == slot)
        .map(|(_, v)| *v)
        .collect::<Vec<_>>()
        != [true_value]
        || false_ssa
            .exit()
            .iter()
            .filter(|(s, _)| *s == slot)
            .map(|(_, v)| *v)
            .collect::<Vec<_>>()
            != [false_value]
    {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Producer,
        ));
    }
    let stack_phis: Vec<_> = ssa
        .phis()
        .iter()
        .filter(|phi| {
            phi.block() == consumer
                && matches!(phi.slot(), Slot::Stack(_))
                && ssa.value(phi.value()).replaced_by().is_none()
        })
        .collect();
    let [phi] = stack_phis.as_slice() else {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Phi,
        ));
    };
    let phi_value = ssa.value(phi.value());
    if phi.slot() != slot
        || phi.inputs().len() != 2
        || phi_value.replaced_by().is_some()
        || !matches!(phi_value.def(), Definition::Phi { block, slot: phi_slot } if block == consumer && *phi_slot == slot)
        || !consumer_ssa.entry().contains(&(slot, phi.value()))
    {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Phi,
        ));
    }
    // SSA does not promise that the Phi operand vector follows canonical edge order. Match each
    // operand to exactly one predecessor's exit at this same stack depth, then require both sides.
    let mut matched = [false, false];
    for input in phi.inputs() {
        let PhiInput::Value(value) = input else {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Phi,
            ));
        };
        let predecessors = [(true_ssa, true_value), (false_ssa, false_value)];
        let matches: Vec<_> = predecessors
            .iter()
            .enumerate()
            .filter_map(|(index, (predecessor, _))| {
                predecessor
                    .exit()
                    .iter()
                    .any(|(exit_slot, exit_value)| *exit_slot == slot && exit_value == value)
                    .then_some(index)
            })
            .collect();
        let [index] = matches.as_slice() else {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Phi,
            ));
        };
        if matched[*index] || *value != predecessors[*index].1 {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Phi,
            ));
        }
        matched[*index] = true;
    }
    if !matched.into_iter().all(|side| side) {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Phi,
        ));
    }
    let [reader] = phi_value.uses() else {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Consumer,
        ));
    };
    let Some(first_consumer) = consumer_ssa.instructions().first() else {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Consumer,
        ));
    };
    let instance_receiver = match first_consumer.reads() {
        [(value_slot, value), (Slot::Stack(0), receiver)]
            if first_consumer.opcode() == 0xb5
                && *value_slot == slot
                && *value == phi.value()
                && slot == Slot::Stack(1) =>
        {
            Some(*receiver)
        }
        _ => None,
    };
    let array_operands = match first_consumer.reads() {
        [
            (Slot::Stack(2), value),
            (Slot::Stack(1), index),
            (Slot::Stack(0), array),
        ] if first_consumer.opcode() == 0x54 && slot == Slot::Stack(2) && *value == phi.value() => {
            Some((*array, *index))
        }
        _ => None,
    };
    if reader.block() != consumer
        || reader.bci() != Some(*consumer_bci)
        || first_consumer.bci() != *consumer_bci
        || (first_consumer.reads() != [(slot, phi.value())]
            && instance_receiver.is_none()
            && array_operands.is_none())
    {
        return Ok(ShortCircuitValueAttempt::Refused(
            ShortCircuitValueRefusal::Consumer,
        ));
    }
    let consumer = match (first_consumer.opcode(), operations.get(*consumer_bci)) {
        (0x54, Some(Operation::ArrayStore { .. })) => {
            let Some((array, index)) = array_operands else {
                return Ok(ShortCircuitValueAttempt::Refused(
                    ShortCircuitValueRefusal::Consumer,
                ));
            };
            if !prefix.is_empty()
                || !first_consumer.writes().is_empty()
                || array_element(ssa, operations, array, None) != Some(Type::Boolean)
                || !short_circuit_array_operands(
                    array,
                    index,
                    &tests[0].0,
                    *first_bci,
                    consumer,
                    *consumer_bci,
                    ssa,
                    operations,
                    budget,
                )?
            {
                return Ok(ShortCircuitValueAttempt::Refused(
                    ShortCircuitValueRefusal::Consumer,
                ));
            }
            ShortCircuitConsumer::BooleanArray(array, index)
        }
        (
            0xb3,
            Some(Operation::Field {
                access: FieldAccess::Write,
                is_static: true,
                owner,
                name,
                descriptor,
            }),
        ) if descriptor == "Z" && !owner.is_empty() && !name.is_empty() => {
            ShortCircuitConsumer::Field(owner.clone(), name.clone(), descriptor.clone())
        }
        (
            0xb5,
            Some(Operation::Field {
                access: FieldAccess::Write,
                is_static: false,
                owner,
                name,
                descriptor,
            }),
        ) if descriptor == "Z" && !owner.is_empty() && !name.is_empty() => {
            let Some(receiver) = instance_receiver else {
                return Ok(ShortCircuitValueAttempt::Refused(
                    ShortCircuitValueRefusal::Consumer,
                ));
            };
            let source = ssa.value(receiver);
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(
                    source
                        .uses()
                        .len()
                        .saturating_add(ssa.phis().len())
                        .saturating_add(ssa.phis().iter().fold(0usize, |count, merge| {
                            count.saturating_add(merge.inputs().len())
                        })),
                )
                .unwrap_or(u64::MAX),
                Some(*consumer_bci),
            )?;
            let direct_uses = source
                .uses()
                .iter()
                .filter(|use_| use_.bci().is_some())
                .collect::<Vec<_>>();
            let trivial_merges = ssa
                .phis()
                .iter()
                .filter(|merge| {
                    merge.slot() == Slot::Stack(0)
                        && ssa.value(merge.value()).replaced_by() == Some(receiver)
                        && merge.inputs().iter().all(
                            |input| matches!(input, PhiInput::Value(value) if *value == receiver),
                        )
                })
                .map(|merge| merge.block())
                .collect::<BTreeSet<_>>();
            let only_trivial_stack_merges = source
                .uses()
                .iter()
                .filter(|use_| use_.bci().is_none())
                .all(|use_| trivial_merges.contains(use_.block()));
            if source.replaced_by().is_some()
                || !matches!(direct_uses.as_slice(), [use_] if use_.block() == consumer && use_.bci() == Some(*consumer_bci))
                || !only_trivial_stack_merges
                || !matches!(source.def(), Definition::Instruction { block, bci } if (prefix.contains(block) || block == &tests[0].0) && *bci < *first_bci)
                || !first_consumer.writes().is_empty()
            {
                return Ok(ShortCircuitValueAttempt::Refused(
                    ShortCircuitValueRefusal::Consumer,
                ));
            }
            ShortCircuitConsumer::InstanceField(
                owner.clone(),
                name.clone(),
                descriptor.clone(),
                receiver,
            )
        }
        (0xb5, _) => {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Field,
            ));
        }
        (0xb3, _) => {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Field,
            ));
        }
        (0xac, Some(Operation::Return))
            if return_type == Some(&Type::Boolean) && consumer_ssa.instructions().len() == 1 =>
        {
            ShortCircuitConsumer::Return
        }
        (0xb8, Some(Operation::Invoke(target)))
            if target.kind() == InvokeKind::Static
                && !target.is_interface_reference()
                && target.descriptor() == "(Z)V"
                && !target.owner().is_empty()
                && !target.name().is_empty()
                && first_consumer.writes().is_empty() =>
        {
            ShortCircuitConsumer::Invoke(target.clone())
        }
        (0x36 | 0x3b..=0x3e, Some(Operation::Store { slot })) if matches!(first_consumer.writes(), [(Slot::Local(written_slot), _)] if written_slot == slot) =>
        {
            let [(Slot::Local(_), written)] = first_consumer.writes() else {
                unreachable!("the store's sole local write was checked")
            };
            ShortCircuitConsumer::Local(*slot, *written)
        }
        _ => {
            return Ok(ShortCircuitValueAttempt::Refused(
                ShortCircuitValueRefusal::Consumer,
            ));
        }
    };
    Ok(ShortCircuitValueAttempt::Proved(ShortCircuitValueProof {
        test_bcis: tests.iter().map(|(_, bci)| *bci).collect(),
        true_producer: true_value,
        false_producer: false_value,
        phi: phi.value(),
        stack_depth: true_depth,
        consumer_bci: *consumer_bci,
        consumer,
    }))
}

/// A retained array target may be inlined into the final assignment only when its two
/// expression trees are disjoint, single-use prefixes of the first test block. This preserves
/// the bytecode's array -> index -> RHS evaluation order even when either tree contains a call.
#[allow(clippy::too_many_arguments)]
fn short_circuit_array_source(
    value: ValueId,
    reader_block: &CanonicalBlockId,
    reader_bci: u32,
    source_block: &CanonicalBlockId,
    first_test_bci: u32,
    root: bool,
    root_stack: u32,
    ssa: &SsaTable,
    operations: &Operations,
    bcis: &mut BTreeSet<u32>,
    active: &mut BTreeSet<ValueId>,
    budget: &mut Budget,
    depth: usize,
) -> Result<bool, StopReason> {
    if depth > MAX_VALUE_DEPTH || !active.insert(value) {
        return Ok(false);
    }
    poll(budget, Some(reader_bci))?;
    charge(budget, CountedBudgetDimension::IrItems, 1, Some(reader_bci))?;
    let facts = ssa.value(value);
    let direct_uses = facts
        .uses()
        .iter()
        .filter(|usage| usage.bci().is_some())
        .collect::<Vec<_>>();
    let trivial_merges = if root {
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(ssa.phis().len()).unwrap_or(u64::MAX),
            Some(reader_bci),
        )?;
        ssa.phis()
            .iter()
            .filter(|merge| {
                merge.slot() == Slot::Stack(root_stack)
                    && ssa.value(merge.value()).replaced_by() == Some(value)
                    && merge.inputs().iter().all(|input| {
                        matches!(input, PhiInput::Value(input_value) if *input_value == value)
                    })
            })
            .map(|merge| merge.block())
            .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    let uses_match = matches!(direct_uses.as_slice(), [usage]
        if usage.block() == reader_block && usage.bci() == Some(reader_bci))
        && facts
            .uses()
            .iter()
            .filter(|usage| usage.bci().is_none())
            .all(|usage| root && trivial_merges.contains(usage.block()));
    let result = if uses_match && facts.replaced_by().is_none() {
        match facts.def() {
            Definition::Instruction { block, bci }
                if block == source_block && *bci < first_test_bci =>
            {
                let Some(instruction) = ssa
                    .block(block)
                    .and_then(|names| instruction_in_block(names, *bci))
                else {
                    active.remove(&value);
                    return Ok(false);
                };
                let expression = matches!(
                    operations.get(*bci),
                    Some(
                        Operation::Push(_)
                            | Operation::Load { .. }
                            | Operation::Arithmetic { .. }
                            | Operation::Shift { .. }
                            | Operation::Bitwise { .. }
                            | Operation::Negate
                            | Operation::PrimitiveConversion { .. }
                            | Operation::ArrayLoad
                            | Operation::ArrayElementLoad { .. }
                            | Operation::ArrayLength
                            | Operation::NewArray { .. }
                            | Operation::CheckCast { .. }
                            | Operation::InstanceOf { .. }
                    )
                ) || matches!(
                    operations.get(*bci),
                    Some(Operation::Field {
                        access: FieldAccess::Read,
                        ..
                    })
                ) || matches!(operations.get(*bci), Some(Operation::Invoke(target))
                        if is_java_identifier(target.name()));
                if !expression
                    || !instruction
                        .writes()
                        .iter()
                        .any(|(_, written)| *written == value)
                    || !bcis.insert(*bci)
                {
                    false
                } else {
                    let mut complete = true;
                    for (_, operand) in stack_operands(instruction) {
                        complete &= short_circuit_array_source(
                            operand,
                            block,
                            *bci,
                            source_block,
                            first_test_bci,
                            false,
                            root_stack,
                            ssa,
                            operations,
                            bcis,
                            active,
                            budget,
                            depth + 1,
                        )?;
                    }
                    complete
                }
            }
            _ => false,
        }
    } else {
        false
    };
    active.remove(&value);
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn short_circuit_array_operands(
    array: ValueId,
    index: ValueId,
    source_block: &CanonicalBlockId,
    first_test_bci: u32,
    consumer_block: &CanonicalBlockId,
    consumer_bci: u32,
    ssa: &SsaTable,
    operations: &Operations,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    let Some(block) = ssa.block(source_block) else {
        return Ok(false);
    };
    let mut array_bcis = BTreeSet::new();
    let mut index_bcis = BTreeSet::new();
    if !short_circuit_array_source(
        array,
        consumer_block,
        consumer_bci,
        source_block,
        first_test_bci,
        true,
        0,
        ssa,
        operations,
        &mut array_bcis,
        &mut BTreeSet::new(),
        budget,
        0,
    )? || !short_circuit_array_source(
        index,
        consumer_block,
        consumer_bci,
        source_block,
        first_test_bci,
        true,
        1,
        ssa,
        operations,
        &mut index_bcis,
        &mut BTreeSet::new(),
        budget,
        0,
    )? || !array_bcis.is_disjoint(&index_bcis)
        || array_bcis
            .last()
            .zip(index_bcis.first())
            .is_none_or(|(a, i)| a >= i)
    {
        return Ok(false);
    }
    let Some(branch) = instruction_in_block(block, first_test_bci) else {
        return Ok(false);
    };
    let instruction_index = block
        .instructions()
        .iter()
        .map(|instruction| (instruction.bci(), instruction))
        .collect::<HashMap<_, _>>();
    let mut test_bcis = HashSet::new();
    for (_, value) in branch.reads() {
        if !short_circuit_test_sources(
            *value,
            source_block,
            &instruction_index,
            ssa,
            operations,
            &mut test_bcis,
            &mut HashSet::new(),
            0,
            first_test_bci,
        ) {
            return Ok(false);
        }
    }
    if index_bcis
        .last()
        .zip(test_bcis.iter().min())
        .is_none_or(|(i, t)| i >= t)
    {
        return Ok(false);
    }
    for instruction in block
        .instructions()
        .iter()
        .take_while(|instruction| instruction.bci() < first_test_bci)
    {
        poll(budget, Some(instruction.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(instruction.bci()),
        )?;
        if !array_bcis.contains(&instruction.bci())
            && !index_bcis.contains(&instruction.bci())
            && !test_bcis.contains(&instruction.bci())
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// A test may inline only its own single-use value tree. Its remaining instructions are checked
/// by the caller, so an unrelated store, call, allocation or throw cannot disappear.
#[allow(clippy::too_many_arguments)] // keep the bounded walk's evidence explicit without storing mutable proof state
fn short_circuit_test_sources(
    value: ValueId,
    block: &CanonicalBlockId,
    instruction_index: &HashMap<u32, &SsaInstruction>,
    ssa: &SsaTable,
    operations: &Operations,
    dependencies: &mut HashSet<u32>,
    active: &mut HashSet<ValueId>,
    depth: usize,
    reader_bci: u32,
) -> bool {
    if depth > MAX_VALUE_DEPTH || !active.insert(value) {
        return false;
    }
    let result = match ssa.value(value).def() {
        Definition::Entry {
            slot: Slot::Local(_),
            ..
        }
        | Definition::Phi {
            slot: Slot::Local(_),
            ..
        } => true,
        Definition::Instruction { block: origin, bci } if origin == block => {
            let instruction = instruction_index.get(bci).copied();
            instruction.is_some_and(|instruction| {
                ssa.value(value).replaced_by().is_none()
                    && matches!(ssa.value(value).uses(), [usage] if usage.block() == block && usage.bci() == Some(reader_bci))
                    && instruction
                        .writes()
                        .iter()
                        .any(|(_, written)| *written == value)
                    && matches!(
                        operations.get(*bci),
                        Some(
                            Operation::Push(_)
                                | Operation::Load { .. }
                                | Operation::Arithmetic { .. }
                                | Operation::Shift { .. }
                                | Operation::Bitwise { .. }
                                | Operation::Negate
                                | Operation::PrimitiveConversion { .. }
                                | Operation::NumericComparison { .. }
                                | Operation::InstanceOf { .. }
                                | Operation::ArrayLoad
                                | Operation::ArrayElementLoad { .. }
                                | Operation::ArrayLength
                                | Operation::Field {
                                    access: FieldAccess::Read,
                                    ..
                                }
                                | Operation::Invoke(_)
                        )
                    )
                    && dependencies.insert(*bci)
                    && stack_operands(instruction).iter().all(|(_, operand)| {
                        short_circuit_test_sources(
                            *operand,
                            block,
                            instruction_index,
                            ssa,
                            operations,
                            dependencies,
                            active,
                            depth + 1,
                            instruction.bci(),
                        )
                    })
            })
        }
        _ => false,
    };
    active.remove(&value);
    result
}

/// Builds the statements of one method from its regions.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build(
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    operations: &Operations,
    inputs: Inputs<'_>,
    regions: &[Region],
    budget: &mut Budget,
) -> Result<Program, StopReason> {
    let guarded_return_exits = guard_return_ownership(regions, budget)?;
    let mut instructions: BTreeMap<u32, &SsaInstruction> = BTreeMap::new();
    let mut block_of: BTreeMap<u32, CanonicalBlockId> = BTreeMap::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            instructions.insert(instruction.bci(), instruction);
            block_of.insert(instruction.bci(), block.block().clone());
        }
    }
    let compounds = CompoundAssignments::prove(ssa, operations, inputs.fields, budget)?;
    let array_initializers = ArrayInitializers::prove(ssa, operations, inputs.fields, budget)?;
    let declarations = declarations(
        regions,
        canonical,
        ssa,
        operations,
        inputs.names,
        inputs.reuse,
        inputs.parameters,
        inputs.parameter_types,
        inputs.return_type.as_ref(),
        inputs.fields,
        budget,
    )?;
    let mut builder = Builder {
        canonical,
        ssa,
        code: inputs.code,
        operations,
        pool: inputs.pool,
        bootstrap: inputs.bootstrap,
        profile: inputs.profile,
        parameters: inputs.parameters,
        has_receiver: inputs.has_receiver,
        parameter_types: inputs.parameter_types,
        return_type: inputs.return_type,
        names: inputs.names,
        reuse: inputs.reuse,
        chains: inputs.chains,
        members: inputs.members,
        member_inner_targets: inputs.member_inner_targets,
        interface_super_calls: inputs.interface_super_calls,
        declaring_class: inputs.declaring_class,
        direct_super_class: inputs.direct_super_class,
        direct_interfaces: inputs.direct_interfaces,
        class_methods: inputs.class_methods,
        bridge: inputs.bridge,
        sites: inputs.sites,
        prologues: inputs.prologues,
        fields: inputs.fields,
        enums: inputs.enums,
        allow_array_constructor_method_references: inputs.allow_array_constructor_method_references,
        compounds,
        postfix: PostfixUpdates::default(),
        array_initializers,
        instructions,
        block_of,
        budget,
        declared: BTreeSet::new(),
        chained_firsts: BTreeMap::new(),
        undeclared: BTreeSet::new(),
        stmts: Vec::new(),
        statements: 0,
        ragged: false,
        lambdas: Vec::new(),
        lambda_params: BTreeSet::new(),
        accessors: Vec::new(),
        deferred: Vec::new(),
        binding_plans: BTreeMap::new(),
        guarded_return_exits,
        binding_rejections: BTreeMap::new(),
        bindings: BTreeMap::new(),
        binding_refused: BTreeSet::new(),
        synthetic_names: BTreeSet::new(),
        declarations,
        increments: OnceCell::new(),
        pops: OnceCell::new(),
        // The instructions a `catch` clause's own header already wrote: the store that fills its
        // parameter. Every other instruction of a handler is a statement of the clause's body.
        clause_parameters: BTreeSet::new(),
        settled: BTreeSet::new(),
        lambdas_presented: 0,
        lambda_refusals: Vec::new(),
        accessors_presented: 0,
        accessor_refusals: Vec::new(),
        conditional_values: BTreeMap::new(),
        conditional_branches: BTreeMap::new(),
        short_circuit_statements: BTreeMap::new(),
        two_exit_returns: BTreeMap::new(),
        short_circuit_operands: BTreeSet::new(),
        loop_headers: Vec::new(),
        labeled_loop_headers: BTreeSet::new(),
        switch_depth: 0,
    };
    if let Some(reason) = builder.declarations.incomplete.get(&Vec::new()).cloned() {
        // An access outside every claimed region has no narrower complete closure. Refuse the
        // method before any statement or declaration is published, and account for every
        // instruction the canonical graph holds.
        let bcis = builder
            .canonical
            .blocks()
            .iter()
            .flat_map(|block| block.blocks().iter().copied())
            .chain(
                builder
                    .canonical
                    .unreachable()
                    .iter()
                    .flat_map(|block| builder.covered_bcis(block)),
            )
            .chain(regions.iter().flat_map(unaccounted_region_bcis))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let at = bcis.first().copied().unwrap_or(0);
        builder.fallback(bcis, &reason, at)?;
    } else {
        if builder.return_type == Some(Type::Int) {
            let legacy_returns = builder.increments().statements.keys().copied().collect();
            builder.postfix = PostfixUpdates::prove(
                builder.ssa,
                builder.operations,
                builder.fields,
                &legacy_returns,
                builder.budget,
            )?;
        }
        builder.prepare_conditional_regions(regions)?;
        builder.prepare_deferred_bindings()?;
        // A slot whose uses span more than one top-level region is declared wherever every one
        // of them can see it: at the start of the body, before the first region's text.
        builder.declare_at(&[])?;
        for (index, region) in regions.iter().enumerate() {
            let path = child(&[], u32::try_from(index).unwrap_or(u32::MAX));
            builder.region(region, &path)?;
        }
    }
    let mut field_increments = BTreeMap::new();
    if let Some(plan) = builder.increments.get() {
        for increment in plan.statements.values() {
            charge(
                builder.budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(increment.update),
            )?;
            for at in &increment.anchors {
                charge(
                    builder.budget,
                    CountedBudgetDimension::AnalysisSteps,
                    1,
                    Some(*at),
                )?;
                if builder.fields.claim(*at).is_some_and(|(evidence, shape)| {
                    evidence.access == FieldAccess::Read && !shape.writes()
                }) {
                    field_increments.insert(
                        increment.returns,
                        LegacyFieldIncrement {
                            read: *at,
                            write: increment.update,
                            returns: increment.returns,
                        },
                    );
                    break;
                }
            }
        }
    }
    Ok(Program {
        field_increments,
        statements: builder.statements,
        ragged: builder.ragged,
        stmts: builder.stmts,
        lambdas: builder.lambdas,
        accessors: builder.accessors,
        lambda_refusals: builder.lambda_refusals,
        accessor_refusals: builder.accessor_refusals,
        lambdas_presented: builder.lambdas_presented,
        accessors_presented: builder.accessors_presented,
    })
}

/// Adds stable local reads reached while quoting a call's operands. The ordinary producer walk
/// omits these when their declared local still denotes the same value; a quoted call has no
/// argument text where that declaration could stand in for the actual load. The ordered set makes
/// these additional origins deterministic and the shared quote list keeps them unique.
fn append_quoted_stable_loads(
    into: &mut Vec<u32>,
    loads: BTreeSet<u32>,
    budget: Option<&Budget>,
) -> Result<(), StopReason> {
    for bci in loads {
        if let Some(budget) = budget {
            poll(budget, Some(bci))?;
        }
        if !into.contains(&bci) {
            into.push(bci);
        }
    }
    Ok(())
}

struct Builder<'a> {
    canonical: &'a CanonicalCfg,
    ssa: &'a SsaTable,
    code: &'a MethodCodeFacts,
    operations: &'a Operations,
    /// The class's pool: where a dynamic site's own entry, the bootstrap handles and the method
    /// types are named (P3 2.1).
    pool: &'a [CpEntryFacts],
    /// The class's bootstrap table: the only fact that says whether a site is a lambda.
    bootstrap: &'a [BootstrapMethodFacts],
    /// The profile whose rule set this build admits.
    profile: RecoveryProfile,
    /// How many local slots the method's parameters occupy, `this` included when the caller
    /// counted it: the slots below this one are declared by the signature, not by the body.
    parameters: u16,
    /// The type each parameter slot holds, as the member's own descriptor states it (P3-R5).
    parameter_types: &'a BTreeMap<u16, Type>,
    /// Whether slot 0 is the member's receiver, as its own declaration states.
    has_receiver: bool,
    /// The type the member's own descriptor returns: the requirement every `return` of this body is
    /// written under. It decides the boolean shape (P3-R5's reading, in the return position), the
    /// conversion the `return` itself performs where the value's own type is narrower (P3 2c.29) and
    /// the character a `char` return writes for an `int` constant (P3 2c.30), all as
    /// [`meeting_position`] reads them.
    return_type: Option<Type>,
    names: &'a NameTable,
    /// The variables each local slot holds (P3 3.4): which of a slot's two variables a use point
    /// belongs to, and therefore which name that use is written with.
    reuse: &'a reuse::Plan,
    /// The concatenation chains this body's verified shapes own (P3 2.2).
    chains: &'a concat::Plan,
    /// The class's other members, when the caller handed them over (P3 2.2, A12).
    members: Option<&'a ClassMembers>,
    member_inner_targets: &'a [crate::report::ProvedMemberInnerTarget],
    interface_super_calls: &'a [crate::report::ProvedInterfaceSuperCall],
    /// The class this body belongs to, in internal form, when the run's own member declaration
    /// states it: the fact a static call's pool owner is compared against (P3 4.4).
    declaring_class: Option<&'a str>,
    /// The direct superclass from the same class header as the decoded body.
    direct_super_class: Option<&'a jarde_reader::model::JvmBytes>,
    /// The direct interfaces from the same class header as the decoded body.
    direct_interfaces: &'a [jarde_reader::model::JvmString],
    /// The current class's member headers from that same class header, when available.
    class_methods: Option<&'a [jarde_reader::classfile::MemberHeader]>,
    /// The `bridge@1` rule's verdict for this body, when it has one (P3 2.2).
    bridge: Option<&'a bridge::Plan>,
    /// The construction sites this body builds (P3 2.3).
    sites: &'a init::Sites,
    /// The prologue of this body, when the body is an instance initializer (P3 2.3).
    prologues: &'a init::Prologues,
    /// The field accesses this body's instructions were verified to be (P3 2.3).
    fields: &'a field::Plan,
    /// The dispatch-table reads this body performs (P3 2.3).
    enums: &'a enumswitch::Plan,
    /// Whether this body may replace a physical array-helper call with `T[]::new`.
    allow_array_constructor_method_references: bool,
    /// The bounded `int` field and array updates proved from this body's final stores.
    compounds: CompoundAssignments,
    /// Complete postfix old-value returns, owned only after the SSA and evaluation-order proof.
    postfix: PostfixUpdates,
    /// Complete, same-block array initializer chains proved from their allocation through their
    /// final consumer. Their copy/index/store scaffolding is hidden only after this pass succeeds.
    array_initializers: ArrayInitializers,
    instructions: BTreeMap<u32, &'a SsaInstruction>,
    /// The block each instruction belongs to: which block's own entry state and writes state what a
    /// local slot holds where that instruction runs (P3 1.3d).
    block_of: BTreeMap<u32, CanonicalBlockId>,
    budget: &'a mut Budget,
    /// The variables declared so far: a write of a variable whose declaration is already written
    /// becomes an assignment, and every variable's declaration is written once.
    declared: BTreeSet<LocalVariable>,
    /// The local each written first store of a chained assignment filled: the name the second store
    /// of the pair reads, keyed by the first store's own BCI (P3 2c.14). A pair whose first store
    /// wrote no statement has no entry, and its second store is then refused on its own copy rather
    /// than reading a local nothing declared.
    chained_firsts: BTreeMap<u32, (LocalVariable, String)>,
    /// The names of the locals whose declaration this build **refused** (P3 2b.2).
    ///
    /// A local's declaration is the statement its first write carries, or the one hoisted into the
    /// region that holds its uses. A write whose value could not be rendered, whose value did not
    /// meet the variable's type, or whose type the plan could not decide never writes that
    /// declaration — and the name it would have declared is a name the text still spells at every
    /// later use of the slot. Spelling it would publish a body that reads a variable nothing
    /// declares, so a statement that wants to is refused instead (P3 2b.2: "a quoted `istore` must
    /// not let a later use be written as an undeclared `localN`").
    ///
    /// The set is keyed by **name** and not by variable because the check is one a statement's own
    /// text answers: a name is what the text spells, and only the names recorded here are ones
    /// nothing declared — a lambda's parameters are names this layer invents beside every local's
    /// ([`NameTable::free_name`]), so no spelling of another kind can collide with one of them.
    undeclared: BTreeSet<String>,
    stmts: Vec<Stmt>,
    statements: usize,
    ragged: bool,
    /// Every dynamic site this build read, in the order it reached them: the decisions `lambda@1`'s
    /// records are materialized from, not the records themselves.
    lambdas: Vec<LambdaSite>,
    /// The parameter names the lambda shapes of this body have already taken, so that no two of
    /// them spell the same identifier.
    lambda_params: BTreeSet<String>,
    /// Every synthetic accessor call site this build read, in the order it reached them: the
    /// decisions `accessor@1`'s records are materialized from.
    accessors: Vec<AccessorSite>,
    /// The values one instruction's statement was deferred to a reader for, with the BCI of the
    /// instruction that produced them: what a quote has to name when the reader turns out not to
    /// write them after all (P3 2.3 §0).
    deferred: Vec<(ValueId, u32)>,
    /// Values whose one consumer is separated from their producer by a proven independent effect.
    /// The plan is keyed by the instruction at which the producer's value is complete (the
    /// constructor for a construction site), so the declaration is written before the old deferred
    /// value could be rendered at its consumer.
    binding_plans: BTreeMap<u32, BindingPlan>,
    /// Producer positions whose final consumer or dependency closure could not be proved. They
    /// are quoted at the producer so an unproved value never falls back to the old late inline.
    binding_rejections: BTreeMap<u32, BindingRejection>,
    /// The one normal exit owned by each proved synchronized return, keyed by the exact return
    /// consumer. These facts came from the Guard's proof and are used only by deferred placement.
    guarded_return_exits: BTreeMap<u32, GuardReturnOwnership>,
    /// Values successfully published as synthetic locals during this build. A plan is not visible
    /// here until its declaration has actually been pushed into the current statement list.
    bindings: BTreeMap<ValueId, Binding>,
    /// Values whose producer was quoted because a saved declaration could not be committed. A
    /// later consumer must refuse and quote the value too, rather than recomputing its producer.
    binding_refused: BTreeSet<ValueId>,
    /// Names allocated for saved values, shared with the lambda parameter allocator.
    synthetic_names: BTreeSet<String>,
    /// Where each local slot's declaration is written (P3 3.1), and what type the plan decided for
    /// every variable: the slot whose declaration is not the first write's is declared at the start
    /// of the region that contains all of its uses, and every consumer of a variable's type reads
    /// this one decision instead of deciding again.
    declarations: Declarations,
    /// The `field++`/`++field` shapes of this body (P3 2c.10/2c.18), found once from the body's own
    /// instructions, names and field evidence, and then read for every instruction one of them
    /// accounts for.
    increments: OnceCell<FieldIncrements>,
    /// The `pop`s whose evaluation another instruction's own text writes (P3 2c.31), found once from
    /// the body's own instructions: the qualifier a static call is written with, and the call whose
    /// own value its statement discards.
    pops: OnceCell<DiscardedEvaluations>,
    /// How many dynamic sites this build presented, and every site it did not.
    lambdas_presented: u64,
    lambda_refusals: Vec<Gap>,
    /// The BCIs of the stores a `catch` clause's header has already written as its parameter: the
    /// handler's entry store is not a statement of the clause's body.
    clause_parameters: BTreeSet<u32>,
    /// The instructions a statement's own **lead** has already written, while the region that holds
    /// them is being built — and the join `return` every arm of a `switch` expression has written
    /// as its own (P3 2c.25).
    ///
    /// A `try` whose protected range begins inside the block that holds it (`int x = 1; try { … }`)
    /// writes the block's instructions before the range in front of the statement, and the body then
    /// begins in that same block: writing the body whole would write those instructions twice, which
    /// would move an effect the bytecode runs once. The set is scoped to that one body — every other
    /// region is built with it empty, and an instruction outside a lead is never skipped.
    ///
    /// A `switch` expression's join is the other instruction a statement already wrote: its one
    /// `return` is written into each arm ([`Self::switch_join`]), and walking the join again would
    /// write the arm's value where the bytecode holds the merged one.
    settled: BTreeSet<u32>,
    /// How many accessor call sites this build presented, and every one it did not.
    accessors_presented: u64,
    accessor_refusals: Vec<Gap>,
    /// Fully built, proved conditional values whose `if` arms were absorbed at their later unique
    /// stack-Phi consumer. Building these once before suppressing the arms validates every child
    /// and prevents calls from being emitted twice.
    conditional_values: BTreeMap<ValueId, Expr>,
    conditional_branches: BTreeMap<u32, ConditionalBranchPlan>,
    /// A complete field write, return or call prepared before any branch or producer is suppressed.
    short_circuit_statements: BTreeMap<CanonicalBlockId, Stmt>,
    two_exit_returns: BTreeMap<CanonicalBlockId, Stmt>,
    /// A staged instance field assignment owns this one producer in its receiver expression.
    /// Pre-evaluated lvalue operands retained inside one proved short-circuit statement.
    short_circuit_operands: BTreeSet<ValueId>,
    /// Active loop identities and the subset that require Java labels for a non-local transfer.
    loop_headers: Vec<u32>,
    labeled_loop_headers: BTreeSet<u32>,
    /// A switch intercepts an unlabelled break, so a loop break from one of its arms is labeled.
    switch_depth: usize,
}

#[derive(Clone, Debug)]
struct BindingPlan {
    value: ValueId,
    anchor: u32,
    producer: u32,
    name: String,
}

#[derive(Clone, Copy, Debug)]
struct GuardReturnOwnership {
    normal_exit_bci: u32,
    body: (u32, u32),
}

#[derive(Clone, Debug)]
struct BindingRejection {
    value: ValueId,
    anchor: u32,
    producer: u32,
    reason: String,
}

#[derive(Clone, Debug)]
struct Binding {
    name: String,
    ty: Type,
    anchor: u32,
}

/// The two bounded lvalue-update shapes that the final write can prove.
#[derive(Clone, Copy)]
enum CompoundUpdate {
    Field {
        receiver: ValueId,
        rhs: ValueId,
        duplicate: u32,
        read: u32,
        add: u32,
    },
    Array {
        array: ValueId,
        index: ValueId,
        rhs: ValueId,
        duplicate: u32,
        read: u32,
        add: u32,
    },
}

/// Only the verified copy instruction is hidden until the final store writes its complete update.
#[derive(Default)]
struct CompoundAssignments {
    updates: BTreeMap<u32, CompoundUpdate>,
    copies: BTreeSet<u32>,
    reads: BTreeSet<u32>,
    inline_values: BTreeSet<u32>,
}

/// A postfix expression is published only at its immediate `ireturn`. The owned instructions
/// include its complete left-hand evaluation so a call cannot also become an earlier statement.
#[derive(Default)]
struct PostfixUpdates {
    returns: BTreeMap<u32, PostfixUpdate>,
    refusals: BTreeMap<u32, Vec<u32>>,
    owned: BTreeSet<u32>,
}

#[derive(Clone)]
struct PostfixUpdate {
    target: PostfixTarget,
    read: u32,
    store: u32,
    returns: u32,
    /// Every instruction absorbed by the expression, excluding its store and return.
    anchors: Vec<u32>,
}

#[derive(Clone)]
enum PostfixTarget {
    Field { receiver: ValueId, name: String },
    Array { array: ValueId, index: ValueId },
}

impl PostfixUpdates {
    fn owns(&self, bci: u32) -> bool {
        self.owned.contains(&bci)
    }

    fn at_return(&self, bci: u32) -> Option<&PostfixUpdate> {
        self.returns.get(&bci)
    }

    fn refusal_at(&self, bci: u32) -> Option<&[u32]> {
        self.refusals.get(&bci).map(Vec::as_slice)
    }

    fn prove(
        ssa: &SsaTable,
        operations: &Operations,
        fields: &field::Plan,
        legacy_returns: &BTreeSet<u32>,
        budget: &mut Budget,
    ) -> Result<Self, StopReason> {
        let mut plan = Self::default();
        let mut effects = None;
        for block in ssa.blocks() {
            for (position, returns) in block.instructions().iter().enumerate() {
                let at = returns.bci();
                if returns.opcode() != 0xac || legacy_returns.contains(&at) || position < 6 {
                    continue;
                }
                if !block.instructions()[..position]
                    .iter()
                    .any(|instruction| matches!(instruction.opcode(), OPCODE_DUP_X1 | 0x5b))
                {
                    continue;
                }
                poll(budget, Some(at))?;
                charge(budget, CountedBudgetDimension::AnalysisSteps, 1, Some(at))?;
                let window = &block.instructions()[position - 6..=position];
                let proved =
                    match prove_postfix_field(ssa, operations, fields, block, window, budget)? {
                        Some(update) => Some(update),
                        None => {
                            prove_postfix_array(ssa, operations, fields, block, window, budget)?
                        }
                    };
                if let Some(update) = proved {
                    if effects.is_none() {
                        effects = Some(index_postfix_effects(ssa, budget)?);
                    }
                    if postfix_handlers_match(
                        effects
                            .as_ref()
                            .expect("the candidate built an effects index"),
                        block.block(),
                        &update,
                        budget,
                    )? && update
                        .anchors
                        .iter()
                        .chain(std::iter::once(&update.store))
                        .all(|bci| !plan.owned.contains(bci))
                    {
                        plan.owned.extend(update.anchors.iter().copied());
                        plan.owned.insert(update.store);
                        plan.returns.insert(at, update);
                        continue;
                    }
                }
                // An unproved old-value chain is refused as one physical interval. Earlier
                // independent effects stay at their original statement positions; its operand
                // producers are quoted for provenance but are not silently claimed as Java text.
                if let Some(start) = postfix_refusal_candidate(block, position, budget)? {
                    let mut sources = BTreeSet::new();
                    let context = ExpressionBciContext {
                        ssa,
                        operations,
                        fields,
                        block,
                        start: 0,
                        end: start,
                    };
                    for (_, value) in stack_operands(&block.instructions()[start]) {
                        let mut dependencies = BTreeSet::new();
                        if collect_expression_bcis(
                            &context,
                            value,
                            &mut dependencies,
                            &mut BTreeSet::new(),
                            budget,
                            0,
                        )? {
                            sources.extend(dependencies);
                        }
                    }
                    let mut bcis: Vec<u32> = sources.into_iter().collect();
                    for instruction in &block.instructions()[start..=position] {
                        poll(budget, Some(instruction.bci()))?;
                        charge(
                            budget,
                            CountedBudgetDimension::AnalysisSteps,
                            1,
                            Some(instruction.bci()),
                        )?;
                        bcis.push(instruction.bci());
                        if instruction.bci() != at {
                            plan.owned.insert(instruction.bci());
                        }
                    }
                    plan.refusals.insert(at, bcis);
                }
            }
        }
        Ok(plan)
    }
}

fn index_postfix_effects<'a>(
    ssa: &'a SsaTable,
    budget: &mut Budget,
) -> Result<
    BTreeMap<(CanonicalBlockId, u32), &'a jarde_jvm::method_ir::CanonicalInstructionEffect>,
    StopReason,
> {
    let mut effects = BTreeMap::new();
    for effect in ssa.effects().instructions() {
        poll(budget, Some(effect.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(effect.bci()),
        )?;
        effects.insert((effect.block().clone(), effect.bci()), effect);
    }
    Ok(effects)
}

fn postfix_refusal_candidate(
    block: &jarde_jvm::method_ir::SsaBlock,
    return_pos: usize,
    budget: &mut Budget,
) -> Result<Option<usize>, StopReason> {
    let instructions = block.instructions();
    for read_pos in 1..return_pos.saturating_sub(3) {
        let read = &instructions[read_pos];
        poll(budget, Some(read.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(read.bci()),
        )?;
        let copy = &instructions[read_pos + 1];
        let field = read.opcode() == 0xb4 && copy.opcode() == OPCODE_DUP_X1;
        let array = read.opcode() == 0x2e && copy.opcode() == 0x5b;
        if !(field || array)
            || instructions[read_pos + 2].opcode() != 0x04
            || instructions[read_pos + 3].opcode() != 0x60
        {
            continue;
        }
        let duplicate = &instructions[read_pos - 1];
        if duplicate.opcode() != if field { OPCODE_DUP } else { 0x5c } {
            continue;
        }
        for instruction in &instructions[read_pos + 4..return_pos] {
            poll(budget, Some(instruction.bci()))?;
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(instruction.bci()),
            )?;
            if instruction.opcode() == if field { 0xb5 } else { 0x4f } {
                return Ok(Some(read_pos - 1));
            }
        }
    }
    Ok(None)
}

/// A canonical block may cross an exception-table range boundary. Moving an operand into the
/// returned Java expression is safe only when every physical instruction it absorbs has the same
/// ordered handler targets as that lexical return position. Requiring this even for nonthrowing
/// copies keeps the proof simple and deliberately rejects a boundary it cannot represent.
fn postfix_handlers_match(
    effects: &BTreeMap<(CanonicalBlockId, u32), &jarde_jvm::method_ir::CanonicalInstructionEffect>,
    block: &CanonicalBlockId,
    update: &PostfixUpdate,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    let Some(return_effect) = effects.get(&(block.clone(), update.returns)) else {
        return Ok(false);
    };
    let expected = return_effect.handlers();
    for bci in update.anchors.iter().chain(std::iter::once(&update.store)) {
        poll(budget, Some(*bci))?;
        charge(budget, CountedBudgetDimension::AnalysisSteps, 1, Some(*bci))?;
        let Some(effect) = effects.get(&(block.clone(), *bci)) else {
            return Ok(false);
        };
        if effect.handlers() != expected {
            return Ok(false);
        }
    }
    Ok(true)
}

/// One same-block array chain whose allocation, stores and final consumer were proved together.
/// `owned` contains the allocation's replaced length, copy/index/store scaffolding, and any
/// proved local postfix update whose statement moves into one element;
/// element producers remain in their expression trees and are rendered once in order.
#[derive(Clone, Debug)]
struct ArrayInitializer {
    allocation_value: ValueId,
    final_value: ValueId,
    /// Each element stays paired with the array-store instruction that proved its assignment.
    /// The separate, ordered pair is the only authority for a conversion's store provenance.
    elements: Vec<(ValueId, u32)>,
    local_postfix: BTreeMap<u32, LocalPostfixElement>,
    element_sources: Vec<u32>,
    owned: Vec<u32>,
    sources: Vec<u32>,
    consumer: u32,
    children: Vec<u32>,
    depth: usize,
}

#[derive(Clone, Copy, Debug)]
struct LocalPostfixElement {
    slot: u16,
    old_local: ValueId,
    load: u32,
    update: u32,
}

#[derive(Default)]
struct ArrayInitializers {
    allocations: BTreeMap<u32, ArrayInitializer>,
    aliases: BTreeMap<ValueId, ValueId>,
    owned: BTreeSet<u32>,
    element_sources: BTreeSet<u32>,
}

impl ArrayInitializers {
    fn owns(&self, at: u32) -> bool {
        self.owned.contains(&at)
    }

    fn at_allocation(&self, at: u32) -> Option<&ArrayInitializer> {
        self.allocations.get(&at)
    }

    fn allocation_value(&self, value: ValueId) -> Option<ValueId> {
        self.aliases.get(&value).copied()
    }

    fn controls_binding(&self, at: u32) -> bool {
        self.owns(at) || self.element_sources.contains(&at)
    }

    fn prove(
        ssa: &SsaTable,
        operations: &Operations,
        fields: &field::Plan,
        budget: &mut Budget,
    ) -> Result<Self, StopReason> {
        let mut proved = Self::default();
        // Most bodies never enter this proof: read the already-published instruction facts for an
        // array allocation first, without spending the presentation budget. In particular, a
        // body with no array allocation must keep its pre-rule budget profile. The effects map is
        // only needed to compare exception handlers for a candidate initializer chain.
        if !ssa
            .effects()
            .instructions()
            .iter()
            .any(|effect| matches!(effect.opcode(), 0xbc | 0xbd))
        {
            return Ok(proved);
        }

        let mut effects = BTreeMap::new();
        for effect in ssa.effects().instructions() {
            poll(budget, Some(effect.bci()))?;
            charge(
                budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(effect.bci()),
            )?;
            effects.insert(effect.bci(), effect);
        }

        for block in ssa.blocks() {
            let mut candidates = BTreeMap::new();
            let mut candidate_values = BTreeMap::new();
            for allocation in block.instructions().iter().rev() {
                poll(budget, Some(allocation.bci()))?;
                charge(
                    budget,
                    CountedBudgetDimension::IrItems,
                    1,
                    Some(allocation.bci()),
                )?;
                if !matches!(
                    operations.get(allocation.bci()),
                    Some(Operation::NewArray { .. })
                ) {
                    continue;
                }
                if let Some(initializer) = prove_array_initializer(
                    ssa,
                    operations,
                    fields,
                    block,
                    allocation,
                    &effects,
                    &candidates,
                    &candidate_values,
                    budget,
                )? {
                    candidate_values.insert(initializer.final_value, allocation.bci());
                    candidates.insert(allocation.bci(), initializer);
                }
            }
            // A child whose sole consumer is an array store cannot stand alone: only a closed
            // parent expression may commit it. Walk from ordinary consumers after all proofs.
            for (at, candidate) in &candidates {
                if matches!(
                    operations.get(candidate.consumer),
                    Some(Operation::ArrayStore { .. })
                ) {
                    continue;
                }
                let mut chain = Vec::new();
                let mut stack = vec![*at];
                let mut closed = true;
                while let Some(next) = stack.pop() {
                    let Some(node) = candidates.get(&next) else {
                        closed = false;
                        break;
                    };
                    chain.push(next);
                    stack.extend(node.children.iter().copied());
                }
                let mut claimed = BTreeSet::new();
                if !closed
                    || chain.iter().any(|bci| {
                        candidates[bci]
                            .owned
                            .iter()
                            .any(|owned| proved.owned.contains(owned) || !claimed.insert(*owned))
                    })
                {
                    continue;
                }
                for bci in chain {
                    let node = candidates[&bci].clone();
                    proved.owned.extend(node.owned.iter().copied());
                    proved
                        .element_sources
                        .extend(node.element_sources.iter().copied());
                    proved
                        .aliases
                        .insert(node.final_value, node.allocation_value);
                    proved.allocations.insert(bci, node);
                }
            }
        }
        Ok(proved)
    }
}

// Keep the raw SSA, operation, effect and child-proof facts explicit at this bounded proof site;
// bundling them into another state object would duplicate the existing ArrayInitializers plan.
#[allow(clippy::too_many_arguments)]
fn prove_array_initializer(
    ssa: &SsaTable,
    operations: &Operations,
    fields: &field::Plan,
    block: &jarde_jvm::method_ir::SsaBlock,
    allocation: &SsaInstruction,
    effects: &BTreeMap<u32, &jarde_jvm::method_ir::CanonicalInstructionEffect>,
    children: &BTreeMap<u32, ArrayInitializer>,
    child_values: &BTreeMap<ValueId, u32>,
    budget: &mut Budget,
) -> Result<Option<ArrayInitializer>, StopReason> {
    let Some(Operation::NewArray {
        element,
        dimensions: 1,
        total_dimensions,
    }) = operations.get(allocation.bci())
    else {
        return Ok(None);
    };
    let component = if *total_dimensions == 1 {
        element.clone()
    } else {
        Type::Reference(format!(
            "{}{}",
            element.spell(),
            "[]".repeat(usize::from(*total_dimensions) - 1)
        ))
    };
    let operands = stack_operands(allocation);
    let [(_, length_value)] = operands.as_slice() else {
        return Ok(None);
    };
    let Some(length) = integer_constant(ssa, operations, *length_value) else {
        return Ok(None);
    };
    let Ok(length) = usize::try_from(length) else {
        return Ok(None);
    };
    if length == 0 {
        return Ok(None);
    }
    let Some((_, mut array_value)) = one_stack_output(allocation) else {
        return Ok(None);
    };
    let allocation_value = array_value;
    let Some(allocation_pos) = position_in_block(block, allocation.bci()) else {
        return Ok(None);
    };
    // Every element needs at least `dup; index; value; arraystore`, and the surviving copy needs
    // one final consumer. This also bounds the Vec capacity by the already decoded Code/IR size.
    let Some(required) = length.checked_mul(4).and_then(|items| items.checked_add(1)) else {
        return Ok(None);
    };
    if block
        .instructions()
        .len()
        .saturating_sub(allocation_pos + 1)
        < required
    {
        return Ok(None);
    }
    if !single_use_at_with_budget(ssa, *length_value, block.block(), allocation.bci(), budget)? {
        return Ok(None);
    }

    let Some(length_bci) = definition_in_block(ssa, *length_value, block.block()) else {
        return Ok(None);
    };
    let allocation_effect = effects.get(&allocation.bci()).copied();
    let Some(allocation_effect) = allocation_effect else {
        return Ok(None);
    };
    let expected_handlers = allocation_effect.handlers();

    let mut owned = vec![allocation.bci(), length_bci];
    let mut elements = Vec::with_capacity(length);
    let mut local_postfix = BTreeMap::new();
    let mut dependencies = BTreeSet::new();
    let mut element_sources = BTreeSet::new();
    let mut child_allocations = Vec::new();
    let mut depth = 1;
    let mut cursor = allocation_pos + 1;

    for expected_index in 0..length {
        let Some(duplicate) = block.instructions().get(cursor) else {
            return Ok(None);
        };
        charge_array_initializer_instruction(duplicate, budget)?;
        if duplicate.opcode() != OPCODE_DUP
            || !matches!(operations.get(duplicate.bci()), Some(Operation::Duplicate))
            || !single_use_at_with_budget(ssa, array_value, block.block(), duplicate.bci(), budget)?
        {
            return Ok(None);
        }
        let Some((_, duplicated)) = single_stack_read(duplicate) else {
            return Ok(None);
        };
        if duplicated != array_value {
            return Ok(None);
        }
        let copies = stack_outputs(duplicate);
        let [(_, first_copy), (_, second_copy)] = copies.as_slice() else {
            return Ok(None);
        };
        if first_copy == second_copy {
            return Ok(None);
        }

        let Some(index_instruction) = block.instructions().get(cursor + 1) else {
            return Ok(None);
        };
        charge_array_initializer_instruction(index_instruction, budget)?;
        let Some((_, index_value)) = one_stack_output(index_instruction) else {
            return Ok(None);
        };
        let Some(expected_index) = i64::try_from(expected_index).ok() else {
            return Ok(None);
        };
        if !matches!(
            operations.get(index_instruction.bci()),
            Some(Operation::Push(ConstantValue::Int(index))) if *index == expected_index
        ) {
            return Ok(None);
        }

        // The first store using either copy is the only store this `dup` can own. An intervening
        // escape, unrelated effect, wrong index or unknown operation fails the closure checks below.
        let mut store_match = None;
        for (store_pos, candidate) in block.instructions().iter().enumerate().skip(cursor + 2) {
            charge_array_initializer_instruction(candidate, budget)?;
            if !matches!(
                operations.get(candidate.bci()),
                Some(Operation::ArrayStore { .. })
            ) {
                continue;
            }
            let candidate_operands = stack_operands(candidate);
            if let [(_, candidate_array), (_, _), (_, _)] = candidate_operands.as_slice()
                && (*candidate_array == *first_copy || *candidate_array == *second_copy)
            {
                store_match = Some((store_pos, candidate, candidate_operands));
                break;
            }
        }
        let Some((store_pos, store, store_operands)) = store_match else {
            return Ok(None);
        };
        let [(_, stored_array), (_, stored_index), (_, stored_value)] = store_operands.as_slice()
        else {
            return Ok(None);
        };
        if *stored_index != index_value
            || !single_use_at_with_budget(ssa, index_value, block.block(), store.bci(), budget)?
        {
            return Ok(None);
        }
        let retained = if *stored_array == *first_copy {
            *second_copy
        } else if *stored_array == *second_copy {
            *first_copy
        } else {
            return Ok(None);
        };
        if !single_use_at_with_budget(ssa, *stored_array, block.block(), store.bci(), budget)? {
            return Ok(None);
        }

        let Some(Operation::ArrayStore {
            element: opcode_element,
        }) = operations.get(store.bci())
        else {
            return Ok(None);
        };
        let Some(actual_element) = array_element(ssa, operations, array_value, None) else {
            return Ok(None);
        };
        if actual_element != component
            || !array_store_opcode_matches(&actual_element, opcode_element.as_ref(), store.opcode())
        {
            return Ok(None);
        }

        let mut element_dependencies = BTreeSet::new();
        let value_context = ExpressionBciContext {
            ssa,
            operations,
            fields,
            block,
            start: cursor + 2,
            end: store_pos,
        };
        let child = child_values
            .get(stored_value)
            .and_then(|at| children.get(at).map(|child| (at, child)))
            .filter(|(_, child)| child.consumer == store.bci());
        if let Some((child_at, child)) = child {
            poll(budget, Some(*child_at))?;
            depth = depth.max(child.depth + 1);
            // Rendering each nested level traverses its final alias and then its allocation.
            if depth > MAX_VALUE_DEPTH / 2 {
                return Ok(None);
            }
            charge(
                budget,
                CountedBudgetDimension::IrItems,
                child.sources.len() as u64,
                Some(*child_at),
            )?;
            let child_type = match operations.get(*child_at) {
                Some(Operation::NewArray {
                    element,
                    total_dimensions,
                    ..
                }) => Type::Reference(format!(
                    "{}{}",
                    element.spell(),
                    "[]".repeat(usize::from(*total_dimensions))
                )),
                _ => return Ok(None),
            };
            if component != child_type
                || child.sources.iter().any(|bci| {
                    *bci != store.bci()
                        && !position_in_block(block, *bci)
                            .is_some_and(|pos| pos >= cursor + 2 && pos < store_pos)
                })
            {
                return Ok(None);
            }
            element_dependencies.extend(
                child
                    .sources
                    .iter()
                    .copied()
                    .filter(|bci| *bci != store.bci()),
            );
            child_allocations.push(*child_at);
        } else {
            let postfix = if component == Type::Int {
                prove_local_postfix_element(
                    ssa,
                    operations,
                    block,
                    cursor + 2,
                    store_pos,
                    *stored_value,
                    budget,
                )?
            } else {
                None
            };
            if let Some(postfix) = postfix {
                element_dependencies.extend([postfix.load, postfix.update]);
                owned.push(postfix.update);
                local_postfix.insert(store.bci(), postfix);
            } else if !collect_expression_bcis(
                &value_context,
                *stored_value,
                &mut element_dependencies,
                &mut BTreeSet::new(),
                budget,
                0,
            )? {
                return Ok(None);
            }
            if !expression_values_have_single_use(
                ssa,
                block,
                &element_dependencies,
                store.bci(),
                budget,
            )? || !dependency_uses_stay_within(
                ssa,
                block,
                &element_dependencies,
                store.bci(),
                budget,
            )? {
                return Ok(None);
            }
        }
        if element_dependencies.is_empty()
            || !interval_is_expression(block, cursor + 2, store_pos, &element_dependencies, budget)?
        {
            return Ok(None);
        }
        dependencies.extend(element_dependencies.iter().copied());
        element_sources.extend(element_dependencies.iter().copied());
        elements.push((*stored_value, store.bci()));
        owned.extend([duplicate.bci(), index_instruction.bci(), store.bci()]);

        if expected_index + 1 == i64::try_from(length).unwrap_or(i64::MAX) {
            let Some(consumer) = block.instructions().get(store_pos + 1) else {
                return Ok(None);
            };
            charge_array_initializer_instruction(consumer, budget)?;
            if !array_initializer_consumer(operations, fields, consumer, retained)
                && !(consumer.opcode() == 0x53
                    && matches!(stack_operands(consumer).as_slice(), [(_, _), (_, _), (_, value)] if *value == retained))
                || !single_use_at_with_budget(ssa, retained, block.block(), consumer.bci(), budget)?
            {
                return Ok(None);
            }
        } else {
            let Some(next_duplicate) = block.instructions().get(store_pos + 1) else {
                return Ok(None);
            };
            if !matches!(
                operations.get(next_duplicate.bci()),
                Some(Operation::Duplicate)
            ) || !single_use_at_with_budget(
                ssa,
                retained,
                block.block(),
                next_duplicate.bci(),
                budget,
            )? {
                return Ok(None);
            }
        }
        array_value = retained;
        cursor = store_pos + 1;
    }

    let consumer = block.instructions()[cursor].bci();
    let mut source_bcis: BTreeSet<u32> = owned.iter().copied().collect();
    source_bcis.extend(dependencies);
    source_bcis.insert(consumer);
    for bci in &source_bcis {
        poll(budget, Some(*bci))?;
        charge(budget, CountedBudgetDimension::IrItems, 1, Some(*bci))?;
        let Some(effect) = effects.get(bci) else {
            return Ok(None);
        };
        if effect.may_throw() && effect.handlers() != expected_handlers {
            return Ok(None);
        }
    }

    Ok(Some(ArrayInitializer {
        allocation_value,
        final_value: array_value,
        elements,
        local_postfix,
        element_sources: element_sources.into_iter().collect(),
        owned,
        sources: source_bcis.into_iter().collect(),
        consumer,
        children: child_allocations,
        depth,
    }))
}

/// The only local update admitted inside an initializer element is the physical
/// `iload slot; iinc slot, +1; iastore` window. The old local version may feed exactly
/// the load and update; the old stack copy may feed only this store.
fn prove_local_postfix_element(
    ssa: &SsaTable,
    operations: &Operations,
    block: &jarde_jvm::method_ir::SsaBlock,
    start: usize,
    store_pos: usize,
    stored_value: ValueId,
    budget: &mut Budget,
) -> Result<Option<LocalPostfixElement>, StopReason> {
    if store_pos != start + 2 {
        return Ok(None);
    }
    let [load, update] = &block.instructions()[start..store_pos] else {
        return Ok(None);
    };
    charge_array_initializer_instruction(load, budget)?;
    charge_array_initializer_instruction(update, budget)?;
    let Some(Operation::Load { slot }) = operations.get(load.bci()) else {
        return Ok(None);
    };
    if !matches!(load.opcode(), 0x15 | 0x1a..=0x1d)
        || !matches!(operations.get(update.bci()), Some(Operation::Increment { slot: updated, amount: 1 }) if updated == slot)
    {
        return Ok(None);
    }
    let ([(Slot::Local(read_slot), old_local)], [(Slot::Stack(_), loaded)]) =
        (load.reads(), load.writes())
    else {
        return Ok(None);
    };
    let ([(Slot::Local(update_slot), update_old)], [(Slot::Local(written_slot), new_local)]) =
        (update.reads(), update.writes())
    else {
        return Ok(None);
    };
    if read_slot != slot
        || update_slot != slot
        || written_slot != slot
        || old_local != update_old
        || old_local == new_local
        || *loaded != stored_value
        || !single_use_at_with_budget(
            ssa,
            stored_value,
            block.block(),
            block.instructions()[store_pos].bci(),
            budget,
        )?
    {
        return Ok(None);
    }
    let uses = ssa.value(*old_local).uses();
    for usage in uses {
        poll(budget, usage.bci())?;
        charge(budget, CountedBudgetDimension::IrItems, 1, usage.bci())?;
    }
    if uses.len() != 2
        || !uses
            .iter()
            .any(|use_site| use_site.block() == block.block() && use_site.bci() == Some(load.bci()))
        || !uses.iter().any(|use_site| {
            use_site.block() == block.block() && use_site.bci() == Some(update.bci())
        })
    {
        return Ok(None);
    }
    Ok(Some(LocalPostfixElement {
        slot: *slot,
        old_local: *old_local,
        load: load.bci(),
        update: update.bci(),
    }))
}

fn charge_array_initializer_instruction(
    instruction: &SsaInstruction,
    budget: &mut Budget,
) -> Result<(), StopReason> {
    poll(budget, Some(instruction.bci()))?;
    charge(
        budget,
        CountedBudgetDimension::IrItems,
        1,
        Some(instruction.bci()),
    )
}

fn integer_constant(ssa: &SsaTable, operations: &Operations, value: ValueId) -> Option<i64> {
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return None;
    };
    match operations.get(*bci)? {
        Operation::Push(ConstantValue::Int(value)) => Some(*value),
        _ => None,
    }
}

fn single_use_at_with_budget(
    ssa: &SsaTable,
    value: ValueId,
    block: &CanonicalBlockId,
    at: u32,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    poll(budget, Some(at))?;
    charge(budget, CountedBudgetDimension::IrItems, 1, Some(at))?;
    let uses = ssa.value(value).uses();
    for usage in uses {
        poll(budget, usage.bci())?;
        charge(budget, CountedBudgetDimension::IrItems, 1, usage.bci())?;
    }
    Ok(matches!(uses, [usage] if usage.block() == block && usage.bci() == Some(at)))
}

fn array_store_opcode_matches(component: &Type, opcode_element: Option<&Type>, opcode: u8) -> bool {
    match component {
        Type::Reference(_) => opcode == 0x53 && opcode_element.is_none(),
        Type::Boolean | Type::Byte => opcode == 0x54 && opcode_element == Some(&Type::Int),
        Type::Char => opcode == 0x55 && opcode_element == Some(&Type::Int),
        Type::Short => opcode == 0x56 && opcode_element == Some(&Type::Int),
        Type::Int => opcode == 0x4f && opcode_element == Some(&Type::Int),
        Type::Long => opcode == 0x50 && opcode_element == Some(&Type::Long),
        Type::Float => opcode == 0x51 && opcode_element == Some(&Type::Float),
        Type::Double => opcode == 0x52 && opcode_element == Some(&Type::Double),
    }
}

fn array_initializer_consumer(
    operations: &Operations,
    fields: &field::Plan,
    instruction: &SsaInstruction,
    value: ValueId,
) -> bool {
    if !stack_operands(instruction)
        .iter()
        .any(|(_, operand)| *operand == value)
    {
        return false;
    }
    match operations.get(instruction.bci()) {
        Some(Operation::Return | Operation::Store { .. } | Operation::Invoke(_))
        | Some(Operation::InvokeDynamic(_)) => true,
        Some(Operation::Field {
            access: FieldAccess::Write,
            ..
        }) => fields
            .claim(instruction.bci())
            .is_some_and(|(_, shape)| shape.writes()),
        _ => false,
    }
}

impl CompoundAssignments {
    fn prove(
        ssa: &SsaTable,
        operations: &Operations,
        fields: &field::Plan,
        budget: &mut Budget,
    ) -> Result<Self, StopReason> {
        let mut plan = Self::default();
        for block in ssa.blocks() {
            for instruction in block.instructions() {
                let at = instruction.bci();
                poll(budget, Some(at))?;
                charge(budget, CountedBudgetDimension::IrItems, 1, Some(at))?;
                let update = match operations.get(at) {
                    Some(Operation::Field {
                        access: FieldAccess::Write,
                        ..
                    }) => prove_field_update(ssa, operations, fields, block, instruction, budget)?,
                    Some(Operation::ArrayStore { .. }) if instruction.opcode() == 0x4f => {
                        prove_array_update(ssa, operations, fields, block, instruction, budget)?
                    }
                    _ => None,
                };
                if let Some((update, rhs_dependencies)) = update {
                    let duplicate = match update {
                        CompoundUpdate::Field { duplicate, .. }
                        | CompoundUpdate::Array { duplicate, .. } => duplicate,
                    };
                    if plan.copies.insert(duplicate) {
                        let (read, _) = match update {
                            CompoundUpdate::Field { read, add, .. }
                            | CompoundUpdate::Array { read, add, .. } => (read, add),
                        };
                        plan.reads.insert(read);
                        plan.inline_values.extend(rhs_dependencies);
                        plan.updates.insert(at, update);
                    }
                }
            }
        }
        Ok(plan)
    }

    fn update_at(&self, store: u32) -> Option<CompoundUpdate> {
        self.updates.get(&store).copied()
    }

    fn owns_copy(&self, bci: u32) -> bool {
        self.copies.contains(&bci)
    }

    fn owns_read(&self, bci: u32) -> bool {
        self.reads.contains(&bci)
    }

    fn keeps_inline(&self, bci: u32) -> bool {
        self.inline_values.contains(&bci)
    }
}

fn prove_field_update(
    ssa: &SsaTable,
    operations: &Operations,
    fields: &field::Plan,
    block: &jarde_jvm::method_ir::SsaBlock,
    store: &SsaInstruction,
    budget: &mut Budget,
) -> Result<Option<(CompoundUpdate, BTreeSet<u32>)>, StopReason> {
    let at = store.bci();
    let Some((written_field, write_shape)) = fields.claim(at) else {
        return Ok(None);
    };
    if store.opcode() != 0xb5
        || written_field.access != FieldAccess::Write
        || written_field.is_static
        || written_field.descriptor != "I"
    {
        return Ok(None);
    }
    let (Some(receiver_copy), Some(sum)) = (write_shape.receiver, write_shape.value) else {
        return Ok(None);
    };
    let Some(add_bci) = definition_in_block(ssa, sum, block.block()) else {
        return Ok(None);
    };
    let Some(add) = instruction_in_block(block, add_bci) else {
        return Ok(None);
    };
    if add.opcode() != 0x60
        || !matches!(
            operations.get(add_bci),
            Some(Operation::Arithmetic {
                op: ArithmeticOp::Add
            })
        )
    {
        return Ok(None);
    }
    let Some((old, rhs)) = two_stack_values(add) else {
        return Ok(None);
    };
    let Some(read_bci) = definition_in_block(ssa, old, block.block()) else {
        return Ok(None);
    };
    let Some(read) = instruction_in_block(block, read_bci) else {
        return Ok(None);
    };
    if read.opcode() != 0xb4 {
        return Ok(None);
    }
    let Some((read_field, read_shape)) = fields.claim(read_bci) else {
        return Ok(None);
    };
    if read_field.access != FieldAccess::Read
        || read_field.is_static
        || read_field.owner != written_field.owner
        || read_field.name != written_field.name
        || read_field.descriptor != written_field.descriptor
    {
        return Ok(None);
    }
    let Some(read_receiver) = read_shape.receiver else {
        return Ok(None);
    };
    let Some(duplicate_bci) = definition_in_block(ssa, read_receiver, block.block()) else {
        return Ok(None);
    };
    if definition_in_block(ssa, receiver_copy, block.block()) != Some(duplicate_bci) {
        return Ok(None);
    }
    let Some(duplicate) = instruction_in_block(block, duplicate_bci) else {
        return Ok(None);
    };
    if duplicate.opcode() != 0x59
        || !matches!(operations.get(duplicate_bci), Some(Operation::Duplicate))
    {
        return Ok(None);
    }
    let Some((_, receiver)) = single_stack_read(duplicate) else {
        return Ok(None);
    };
    let copies = stack_outputs(duplicate);
    if copies.len() != 2
        || copies[0].1 == copies[1].1
        || read_receiver != copies[1].1
        || receiver_copy != copies[0].1
    {
        return Ok(None);
    }
    let Some((_, old_read)) = one_stack_output(read) else {
        return Ok(None);
    };
    if old_read != old
        || !single_use_at(ssa, receiver, block.block(), duplicate_bci)
        || !single_use_at(ssa, copies[0].1, block.block(), at)
        || !single_use_at(ssa, copies[1].1, block.block(), read_bci)
        || !single_use_at(ssa, old, block.block(), add_bci)
        || !single_use_at(ssa, rhs, block.block(), add_bci)
        || !single_use_at(ssa, sum, block.block(), at)
    {
        return Ok(None);
    }
    let Some(duplicate_pos) = position_in_block(block, duplicate_bci) else {
        return Ok(None);
    };
    let Some(read_pos) = position_in_block(block, read_bci) else {
        return Ok(None);
    };
    let Some(add_pos) = position_in_block(block, add_bci) else {
        return Ok(None);
    };
    let Some(store_pos) = position_in_block(block, at) else {
        return Ok(None);
    };
    if read_pos != duplicate_pos + 1 || !(read_pos < add_pos && add_pos + 1 == store_pos) {
        return Ok(None);
    }
    let mut receiver_dependencies = BTreeSet::new();
    let receiver_context = ExpressionBciContext {
        ssa,
        operations,
        fields,
        block,
        start: 0,
        end: duplicate_pos,
    };
    if !collect_expression_bcis(
        &receiver_context,
        receiver,
        &mut receiver_dependencies,
        &mut BTreeSet::new(),
        budget,
        0,
    )? || !dependency_uses_stay_within(
        ssa,
        block,
        &receiver_dependencies,
        duplicate_bci,
        budget,
    )? || !dependencies_fill_prefix(block, duplicate_pos, &receiver_dependencies, budget)?
    {
        return Ok(None);
    }
    let mut rhs_dependencies = BTreeSet::new();
    let rhs_context = ExpressionBciContext {
        ssa,
        operations,
        fields,
        block,
        start: read_pos + 1,
        end: add_pos,
    };
    if !collect_expression_bcis(
        &rhs_context,
        rhs,
        &mut rhs_dependencies,
        &mut BTreeSet::new(),
        budget,
        0,
    )? || !dependency_uses_stay_within(ssa, block, &rhs_dependencies, add_bci, budget)?
        || !interval_is_expression(block, read_pos + 1, add_pos, &rhs_dependencies, budget)?
    {
        return Ok(None);
    }
    Ok(Some((
        CompoundUpdate::Field {
            receiver,
            rhs,
            duplicate: duplicate_bci,
            read: read_bci,
            add: add_bci,
        },
        rhs_dependencies,
    )))
}

fn prove_array_update(
    ssa: &SsaTable,
    operations: &Operations,
    fields: &field::Plan,
    block: &jarde_jvm::method_ir::SsaBlock,
    store: &SsaInstruction,
    budget: &mut Budget,
) -> Result<Option<(CompoundUpdate, BTreeSet<u32>)>, StopReason> {
    let at = store.bci();
    if !matches!(
        operations.get(at),
        Some(Operation::ArrayStore {
            element: Some(Type::Int)
        })
    ) {
        return Ok(None);
    }
    let store_operands = stack_operands(store);
    let [(_, array_store), (_, index_store), (_, sum)] = store_operands.as_slice() else {
        return Ok(None);
    };
    let (array_store, index_store, sum) = (*array_store, *index_store, *sum);
    if array_of_value(ssa, operations, array_store, 0) != Some((Type::Int, 1)) {
        return Ok(None);
    }
    let Some(add_bci) = definition_in_block(ssa, sum, block.block()) else {
        return Ok(None);
    };
    let Some(add) = instruction_in_block(block, add_bci) else {
        return Ok(None);
    };
    if add.opcode() != 0x60
        || !matches!(
            operations.get(add_bci),
            Some(Operation::Arithmetic {
                op: ArithmeticOp::Add
            })
        )
    {
        return Ok(None);
    }
    let Some((old, rhs)) = two_stack_values(add) else {
        return Ok(None);
    };
    let Some(read_bci) = definition_in_block(ssa, old, block.block()) else {
        return Ok(None);
    };
    let Some(read) = instruction_in_block(block, read_bci) else {
        return Ok(None);
    };
    if read.opcode() != 0x2e || !matches!(operations.get(read_bci), Some(Operation::ArrayLoad)) {
        return Ok(None);
    }
    let read_operands = stack_operands(read);
    let [(_, array_read), (_, index_read)] = read_operands.as_slice() else {
        return Ok(None);
    };
    let (array_read, index_read) = (*array_read, *index_read);
    let Some(duplicate_bci) = definition_in_block(ssa, array_read, block.block()) else {
        return Ok(None);
    };
    if definition_in_block(ssa, index_read, block.block()) != Some(duplicate_bci)
        || definition_in_block(ssa, array_store, block.block()) != Some(duplicate_bci)
        || definition_in_block(ssa, index_store, block.block()) != Some(duplicate_bci)
    {
        return Ok(None);
    }
    let Some(duplicate) = instruction_in_block(block, duplicate_bci) else {
        return Ok(None);
    };
    if duplicate.opcode() != 0x5c {
        return Ok(None);
    }
    let duplicate_operands = stack_operands(duplicate);
    let [(_, array), (_, index)] = duplicate_operands.as_slice() else {
        return Ok(None);
    };
    let (array, index) = (*array, *index);
    if array_of_value(ssa, operations, array, 0) != Some((Type::Int, 1)) {
        return Ok(None);
    }
    let copies = stack_outputs(duplicate);
    if copies.len() != 4
        || copies[0].1 == copies[1].1
        || copies[0].1 == copies[2].1
        || copies[0].1 == copies[3].1
        || copies[1].1 == copies[2].1
        || copies[1].1 == copies[3].1
        || copies[2].1 == copies[3].1
        || array_store != copies[0].1
        || index_store != copies[1].1
        || array_read != copies[2].1
        || index_read != copies[3].1
    {
        return Ok(None);
    }
    let Some((_, old_read)) = one_stack_output(read) else {
        return Ok(None);
    };
    if old_read != old
        || !single_use_at(ssa, array, block.block(), duplicate_bci)
        || !single_use_at(ssa, index, block.block(), duplicate_bci)
        || !single_use_at(ssa, copies[0].1, block.block(), at)
        || !single_use_at(ssa, copies[1].1, block.block(), at)
        || !single_use_at(ssa, copies[2].1, block.block(), read_bci)
        || !single_use_at(ssa, copies[3].1, block.block(), read_bci)
        || !single_use_at(ssa, old, block.block(), add_bci)
        || !single_use_at(ssa, rhs, block.block(), add_bci)
        || !single_use_at(ssa, sum, block.block(), at)
    {
        return Ok(None);
    }
    let Some(duplicate_pos) = position_in_block(block, duplicate_bci) else {
        return Ok(None);
    };
    let Some(read_pos) = position_in_block(block, read_bci) else {
        return Ok(None);
    };
    let Some(add_pos) = position_in_block(block, add_bci) else {
        return Ok(None);
    };
    let Some(store_pos) = position_in_block(block, at) else {
        return Ok(None);
    };
    if read_pos != duplicate_pos + 1 || !(read_pos < add_pos && add_pos + 1 == store_pos) {
        return Ok(None);
    }
    let mut array_dependencies = BTreeSet::new();
    let mut index_dependencies = BTreeSet::new();
    let lvalue_context = ExpressionBciContext {
        ssa,
        operations,
        fields,
        block,
        start: 0,
        end: duplicate_pos,
    };
    if !collect_expression_bcis(
        &lvalue_context,
        array,
        &mut array_dependencies,
        &mut BTreeSet::new(),
        budget,
        0,
    )? || !collect_expression_bcis(
        &lvalue_context,
        index,
        &mut index_dependencies,
        &mut BTreeSet::new(),
        budget,
        0,
    )? || array_dependencies
        .iter()
        .any(|bci| index_dependencies.contains(bci))
        || !dependencies_precede(&array_dependencies, &index_dependencies, block)
        || !dependency_uses_stay_within(ssa, block, &array_dependencies, duplicate_bci, budget)?
        || !dependency_uses_stay_within(ssa, block, &index_dependencies, duplicate_bci, budget)?
        || !dependencies_fill_prefix(
            block,
            duplicate_pos,
            &array_dependencies
                .union(&index_dependencies)
                .copied()
                .collect(),
            budget,
        )?
    {
        return Ok(None);
    }
    let mut rhs_dependencies = BTreeSet::new();
    let rhs_context = ExpressionBciContext {
        ssa,
        operations,
        fields,
        block,
        start: read_pos + 1,
        end: add_pos,
    };
    if !collect_expression_bcis(
        &rhs_context,
        rhs,
        &mut rhs_dependencies,
        &mut BTreeSet::new(),
        budget,
        0,
    )? || !dependency_uses_stay_within(ssa, block, &rhs_dependencies, add_bci, budget)?
        || !interval_is_expression(block, read_pos + 1, add_pos, &rhs_dependencies, budget)?
    {
        return Ok(None);
    }
    Ok(Some((
        CompoundUpdate::Array {
            array,
            index,
            rhs,
            duplicate: duplicate_bci,
            read: read_bci,
            add: add_bci,
        },
        rhs_dependencies,
    )))
}

/// `dup; getfield; dup_x1; iconst_1; iadd; putfield; ireturn` is a postfix only when
/// each of the three `dup_x1` outputs has its exact, different consumer. In particular the
/// return must take the bottom copy of the read value while the store takes the sum.
fn prove_postfix_field(
    ssa: &SsaTable,
    operations: &Operations,
    fields: &field::Plan,
    block: &jarde_jvm::method_ir::SsaBlock,
    window: &[SsaInstruction],
    budget: &mut Budget,
) -> Result<Option<PostfixUpdate>, StopReason> {
    let [dup, read, copy, one, add, store, returns] = window else {
        return Ok(None);
    };
    if dup.opcode() != OPCODE_DUP
        || read.opcode() != 0xb4
        || copy.opcode() != OPCODE_DUP_X1
        || one.opcode() != 0x04
        || add.opcode() != 0x60
        || store.opcode() != 0xb5
        || !matches!(
            operations.get(one.bci()),
            Some(Operation::Push(ConstantValue::Int(1)))
        )
        || !matches!(
            operations.get(add.bci()),
            Some(Operation::Arithmetic {
                op: ArithmeticOp::Add
            })
        )
        || !matches!(operations.get(returns.bci()), Some(Operation::Return))
    {
        return Ok(None);
    }
    let (Some((read_field, read_shape)), Some((write_field, write_shape))) =
        (fields.claim(read.bci()), fields.claim(store.bci()))
    else {
        return Ok(None);
    };
    if read_field.access != FieldAccess::Read
        || write_field.access != FieldAccess::Write
        || read_field.is_static
        || write_field.is_static
        || read_field.descriptor != "I"
        || read_field.owner != write_field.owner
        || read_field.name != write_field.name
        || read_field.descriptor != write_field.descriptor
        || read_shape.writes()
        || !write_shape.writes()
    {
        return Ok(None);
    }
    let Some((_, receiver)) = single_stack_read(dup) else {
        return Ok(None);
    };
    let receiver_copies = stack_outputs(dup);
    let read_old = one_stack_output(read).map(|(_, value)| value);
    let copy_reads = dup_x1_operands(copy);
    let copy_outputs = stack_outputs(copy);
    let one_value = one_stack_output(one).map(|(_, value)| value);
    let sum = one_stack_output(add).map(|(_, value)| value);
    let returned = single_stack_read(returns).map(|(_, value)| value);
    let (Some(old), Some((top, below)), Some(one_value), Some(sum), Some(returned)) =
        (read_old, copy_reads, one_value, sum, returned)
    else {
        return Ok(None);
    };
    if receiver_copies.len() != 2
        || copy_outputs.len() != 3
        || receiver_copies[0].1 == receiver_copies[1].1
        || copy_outputs
            .iter()
            .map(|(_, value)| value)
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || read_shape.receiver != Some(receiver_copies[1].1)
        || write_shape.receiver != Some(copy_outputs[1].1)
        || write_shape.value != Some(sum)
        || below != receiver_copies[0].1
        || top != old
        || returned != copy_outputs[0].1
        || !matches!(two_stack_values(add), Some((old_copy, increment)) if old_copy == copy_outputs[2].1 && increment == one_value)
        || !single_use_at(ssa, receiver, block.block(), dup.bci())
        || !single_use_at(ssa, receiver_copies[0].1, block.block(), copy.bci())
        || !single_use_at(ssa, receiver_copies[1].1, block.block(), read.bci())
        || !single_use_at(ssa, old, block.block(), copy.bci())
        || !single_use_at(ssa, copy_outputs[0].1, block.block(), returns.bci())
        || !single_use_at(ssa, copy_outputs[1].1, block.block(), store.bci())
        || !single_use_at(ssa, copy_outputs[2].1, block.block(), add.bci())
        || !single_use_at(ssa, one_value, block.block(), add.bci())
        || !single_use_at(ssa, sum, block.block(), store.bci())
    {
        return Ok(None);
    }
    let Some(dup_pos) = position_in_block(block, dup.bci()) else {
        return Ok(None);
    };
    let mut dependencies = BTreeSet::new();
    let context = ExpressionBciContext {
        ssa,
        operations,
        fields,
        block,
        start: 0,
        end: dup_pos,
    };
    if !collect_expression_bcis(
        &context,
        receiver,
        &mut dependencies,
        &mut BTreeSet::new(),
        budget,
        0,
    )? || !expression_values_have_single_use(ssa, block, &dependencies, dup.bci(), budget)?
        || !dependency_uses_stay_within(ssa, block, &dependencies, dup.bci(), budget)?
        || !dependencies_fill_prefix(block, dup_pos, &dependencies, budget)?
    {
        return Ok(None);
    }
    let mut anchors: Vec<u32> = dependencies.into_iter().collect();
    anchors.extend([dup.bci(), read.bci(), copy.bci(), one.bci(), add.bci()]);
    Ok(Some(PostfixUpdate {
        target: PostfixTarget::Field {
            receiver,
            name: read_field.name.clone(),
        },
        read: read.bci(),
        store: store.bci(),
        returns: returns.bci(),
        anchors,
    }))
}

/// The array counterpart uses the four distinct `dup_x2` outputs: old value for return,
/// array and index for the write, and old value for the addition.
fn prove_postfix_array(
    ssa: &SsaTable,
    operations: &Operations,
    fields: &field::Plan,
    block: &jarde_jvm::method_ir::SsaBlock,
    window: &[SsaInstruction],
    budget: &mut Budget,
) -> Result<Option<PostfixUpdate>, StopReason> {
    let [dup, read, copy, one, add, store, returns] = window else {
        return Ok(None);
    };
    if dup.opcode() != 0x5c
        || read.opcode() != 0x2e
        || copy.opcode() != 0x5b
        || one.opcode() != 0x04
        || add.opcode() != 0x60
        || store.opcode() != 0x4f
        || !matches!(operations.get(read.bci()), Some(Operation::ArrayLoad))
        || !matches!(
            operations.get(one.bci()),
            Some(Operation::Push(ConstantValue::Int(1)))
        )
        || !matches!(
            operations.get(add.bci()),
            Some(Operation::Arithmetic {
                op: ArithmeticOp::Add
            })
        )
        || !matches!(
            operations.get(store.bci()),
            Some(Operation::ArrayStore {
                element: Some(Type::Int)
            })
        )
        || !matches!(operations.get(returns.bci()), Some(Operation::Return))
    {
        return Ok(None);
    }
    let Some((array, index)) = two_stack_values(dup) else {
        return Ok(None);
    };
    if array_of_value(ssa, operations, array, 0) != Some((Type::Int, 1)) {
        return Ok(None);
    }
    let dup_outputs = stack_outputs(dup);
    let read_inputs = two_stack_values(read);
    let read_old = one_stack_output(read).map(|(_, value)| value);
    let copy_inputs = stack_operands(copy);
    let copy_outputs = stack_outputs(copy);
    let one_value = one_stack_output(one).map(|(_, value)| value);
    let sum = one_stack_output(add).map(|(_, value)| value);
    let store_inputs = stack_operands(store);
    let returned = single_stack_read(returns).map(|(_, value)| value);
    let (Some((read_array, read_index)), Some(old), Some(one_value), Some(sum), Some(returned)) =
        (read_inputs, read_old, one_value, sum, returned)
    else {
        return Ok(None);
    };
    if dup_outputs.len() != 4
        || copy_outputs.len() != 4
        || dup_outputs
            .iter()
            .map(|(_, value)| value)
            .collect::<BTreeSet<_>>()
            .len()
            != 4
        || copy_outputs
            .iter()
            .map(|(_, value)| value)
            .collect::<BTreeSet<_>>()
            .len()
            != 4
        || read_array != dup_outputs[2].1
        || read_index != dup_outputs[3].1
        || copy_inputs.len() != 3
        || copy_inputs[0].1 != dup_outputs[0].1
        || copy_inputs[1].1 != dup_outputs[1].1
        || copy_inputs[2].1 != old
        || store_inputs.len() != 3
        || store_inputs[0].1 != copy_outputs[1].1
        || store_inputs[1].1 != copy_outputs[2].1
        || store_inputs[2].1 != sum
        || returned != copy_outputs[0].1
        || !matches!(two_stack_values(add), Some((old_copy, increment)) if old_copy == copy_outputs[3].1 && increment == one_value)
        || !single_use_at(ssa, array, block.block(), dup.bci())
        || !single_use_at(ssa, index, block.block(), dup.bci())
        || !single_use_at(ssa, dup_outputs[0].1, block.block(), copy.bci())
        || !single_use_at(ssa, dup_outputs[1].1, block.block(), copy.bci())
        || !single_use_at(ssa, dup_outputs[2].1, block.block(), read.bci())
        || !single_use_at(ssa, dup_outputs[3].1, block.block(), read.bci())
        || !single_use_at(ssa, old, block.block(), copy.bci())
        || !single_use_at(ssa, copy_outputs[0].1, block.block(), returns.bci())
        || !single_use_at(ssa, copy_outputs[1].1, block.block(), store.bci())
        || !single_use_at(ssa, copy_outputs[2].1, block.block(), store.bci())
        || !single_use_at(ssa, copy_outputs[3].1, block.block(), add.bci())
        || !single_use_at(ssa, one_value, block.block(), add.bci())
        || !single_use_at(ssa, sum, block.block(), store.bci())
    {
        return Ok(None);
    }
    let Some(dup_pos) = position_in_block(block, dup.bci()) else {
        return Ok(None);
    };
    let mut array_dependencies = BTreeSet::new();
    let mut index_dependencies = BTreeSet::new();
    let context = ExpressionBciContext {
        ssa,
        operations,
        fields,
        block,
        start: 0,
        end: dup_pos,
    };
    if !collect_expression_bcis(
        &context,
        array,
        &mut array_dependencies,
        &mut BTreeSet::new(),
        budget,
        0,
    )? || !collect_expression_bcis(
        &context,
        index,
        &mut index_dependencies,
        &mut BTreeSet::new(),
        budget,
        0,
    )? || !array_dependencies.is_disjoint(&index_dependencies)
        || !dependencies_precede(&array_dependencies, &index_dependencies, block)
    {
        return Ok(None);
    }
    let dependencies: BTreeSet<u32> = array_dependencies
        .union(&index_dependencies)
        .copied()
        .collect();
    if !expression_values_have_single_use(ssa, block, &dependencies, dup.bci(), budget)?
        || !dependency_uses_stay_within(ssa, block, &dependencies, dup.bci(), budget)?
        || !dependencies_fill_prefix(block, dup_pos, &dependencies, budget)?
    {
        return Ok(None);
    }
    let mut anchors: Vec<u32> = dependencies.into_iter().collect();
    anchors.extend([dup.bci(), read.bci(), copy.bci(), one.bci(), add.bci()]);
    Ok(Some(PostfixUpdate {
        target: PostfixTarget::Array { array, index },
        read: read.bci(),
        store: store.bci(),
        returns: returns.bci(),
        anchors,
    }))
}

fn definition_in_block(ssa: &SsaTable, value: ValueId, block: &CanonicalBlockId) -> Option<u32> {
    match ssa.value(value).def() {
        Definition::Instruction {
            block: definition_block,
            bci,
        } if definition_block == block => Some(*bci),
        _ => None,
    }
}

fn instruction_in_block(
    block: &jarde_jvm::method_ir::SsaBlock,
    bci: u32,
) -> Option<&SsaInstruction> {
    block
        .instructions()
        .iter()
        .find(|instruction| instruction.bci() == bci)
}

fn position_in_block(block: &jarde_jvm::method_ir::SsaBlock, bci: u32) -> Option<usize> {
    block
        .instructions()
        .iter()
        .position(|instruction| instruction.bci() == bci)
}

fn stack_outputs(instruction: &SsaInstruction) -> Vec<(u32, ValueId)> {
    let mut outputs: Vec<(u32, ValueId)> = instruction
        .writes()
        .iter()
        .filter_map(|(slot, value)| match slot {
            Slot::Stack(depth) => Some((*depth, *value)),
            Slot::Local(_) => None,
        })
        .collect();
    outputs.sort_unstable_by_key(|(depth, _)| *depth);
    outputs
}

fn two_stack_values(instruction: &SsaInstruction) -> Option<(ValueId, ValueId)> {
    match stack_operands(instruction).as_slice() {
        [(_, first), (_, second)] => Some((*first, *second)),
        _ => None,
    }
}

fn one_stack_output(instruction: &SsaInstruction) -> Option<(u32, ValueId)> {
    match stack_outputs(instruction).as_slice() {
        [output] => Some(*output),
        _ => None,
    }
}

fn single_use_at(ssa: &SsaTable, value: ValueId, block: &CanonicalBlockId, at: u32) -> bool {
    matches!(
        ssa.value(value).uses(),
        [usage] if usage.block() == block && usage.bci() == Some(at)
    )
}

struct ExpressionBciContext<'a> {
    ssa: &'a SsaTable,
    operations: &'a Operations,
    fields: &'a field::Plan,
    block: &'a jarde_jvm::method_ir::SsaBlock,
    start: usize,
    end: usize,
}

fn collect_expression_bcis(
    context: &ExpressionBciContext<'_>,
    value: ValueId,
    bcis: &mut BTreeSet<u32>,
    seen: &mut BTreeSet<ValueId>,
    budget: &mut Budget,
    depth: usize,
) -> Result<bool, StopReason> {
    if depth > MAX_VALUE_DEPTH {
        return Ok(false);
    }
    if !seen.insert(value) {
        return Ok(true);
    }
    poll(budget, None)?;
    charge(budget, CountedBudgetDimension::IrItems, 1, None)?;
    match context.ssa.value(value).def() {
        Definition::Entry { .. } => Ok(true),
        Definition::Instruction {
            block: definition_block,
            bci,
        } if definition_block == context.block.block() => {
            let Some(position) = position_in_block(context.block, *bci) else {
                return Ok(false);
            };
            if position < context.start || position >= context.end {
                return Ok(false);
            }
            let Some(instruction) = instruction_in_block(context.block, *bci) else {
                return Ok(false);
            };
            let expression = match context.operations.get(*bci) {
                Some(
                    Operation::Push(_)
                    | Operation::Load { .. }
                    | Operation::Arithmetic { .. }
                    | Operation::Shift { .. }
                    | Operation::Bitwise { .. }
                    | Operation::Negate
                    | Operation::PrimitiveConversion { .. }
                    | Operation::Invoke(_)
                    | Operation::InvokeDynamic(_)
                    | Operation::ArrayLoad
                    | Operation::ArrayElementLoad { .. }
                    | Operation::ArrayLength
                    | Operation::NewArray { .. }
                    | Operation::CheckCast { .. }
                    | Operation::InstanceOf { .. },
                ) => true,
                Some(Operation::Field {
                    access: FieldAccess::Read,
                    ..
                }) => context.fields.claim(*bci).is_some_and(|(evidence, shape)| {
                    evidence.access == FieldAccess::Read && !shape.writes()
                }),
                _ => false,
            };
            if !expression {
                return Ok(false);
            }
            bcis.insert(*bci);
            for (_, operand) in stack_operands(instruction) {
                if !collect_expression_bcis(context, operand, bcis, seen, budget, depth + 1)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn dependency_uses_stay_within(
    ssa: &SsaTable,
    block: &jarde_jvm::method_ir::SsaBlock,
    dependencies: &BTreeSet<u32>,
    terminal: u32,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    for bci in dependencies {
        poll(budget, Some(*bci))?;
        charge(budget, CountedBudgetDimension::IrItems, 1, Some(*bci))?;
        let Some(instruction) = instruction_in_block(block, *bci) else {
            return Ok(false);
        };
        for (_, value) in stack_outputs(instruction) {
            for usage in ssa.value(value).uses() {
                poll(budget, usage.bci())?;
                charge(budget, CountedBudgetDimension::IrItems, 1, usage.bci())?;
                if usage.block() != block.block()
                    || !usage
                        .bci()
                        .is_some_and(|at| at == terminal || dependencies.contains(&at))
                {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

/// Rendering an expression DAG recursively writes each child where it occurs in the text. A
/// producer with two SSA readers would therefore be evaluated twice even if both readers belong to
/// this one element expression. Refuse that shape instead of asking deferred binding to hoist the
/// producer out of the allocation/initializer evaluation unit.
fn expression_values_have_single_use(
    ssa: &SsaTable,
    block: &jarde_jvm::method_ir::SsaBlock,
    dependencies: &BTreeSet<u32>,
    terminal: u32,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    for bci in dependencies {
        poll(budget, Some(*bci))?;
        charge(budget, CountedBudgetDimension::IrItems, 1, Some(*bci))?;
        let Some(instruction) = instruction_in_block(block, *bci) else {
            return Ok(false);
        };
        for (_, value) in stack_outputs(instruction) {
            let uses = ssa.value(value).uses();
            for usage in uses {
                poll(budget, usage.bci())?;
                charge(budget, CountedBudgetDimension::IrItems, 1, usage.bci())?;
            }
            if uses.len() != 1 {
                return Ok(false);
            }
        }
    }
    // The companion containment proof names the expected terminal store. Charging it here makes
    // this check's anchor explicit even when an expression source has no stack output.
    poll(budget, Some(terminal))?;
    charge(budget, CountedBudgetDimension::IrItems, 1, Some(terminal))?;
    Ok(true)
}

fn dependencies_precede(
    first: &BTreeSet<u32>,
    second: &BTreeSet<u32>,
    block: &jarde_jvm::method_ir::SsaBlock,
) -> bool {
    let first_last = first
        .iter()
        .filter_map(|bci| position_in_block(block, *bci))
        .max();
    let second_first = second
        .iter()
        .filter_map(|bci| position_in_block(block, *bci))
        .min();
    match (first_last, second_first) {
        (Some(first), Some(second)) => first < second,
        _ => true,
    }
}

fn interval_is_expression(
    block: &jarde_jvm::method_ir::SsaBlock,
    start: usize,
    end: usize,
    dependencies: &BTreeSet<u32>,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    for instruction in block.instructions().get(start..end).unwrap_or_default() {
        poll(budget, Some(instruction.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(instruction.bci()),
        )?;
        if !dependencies.contains(&instruction.bci()) {
            return Ok(false);
        }
    }
    Ok(dependencies.iter().all(|bci| {
        position_in_block(block, *bci).is_some_and(|position| position >= start && position < end)
    }))
}

/// The expression held on the operand stack must be the entire uninterrupted prefix ending at its
/// duplication point. Otherwise presenting it inside the final assignment could move its effects
/// across an instruction that ran while the value was retained on the stack.
fn dependencies_fill_prefix(
    block: &jarde_jvm::method_ir::SsaBlock,
    end: usize,
    dependencies: &BTreeSet<u32>,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    let Some(start) = dependencies
        .iter()
        .filter_map(|bci| position_in_block(block, *bci))
        .min()
    else {
        return Ok(false);
    };
    interval_is_expression(block, start, end, dependencies, budget)
}

fn compound_origin(store: u32, duplicate: u32, read: u32, add: u32) -> OriginSet {
    [duplicate, read, add]
        .into_iter()
        .fold(OriginSet::new(Origin::direct(store)), |origin, bci| {
            origin.plus_derived(Origin::derived(bci))
        })
}

impl Builder<'_> {
    /// Build the entire two-return statement before any of its blocks are suppressed.
    fn build_two_exit_return(
        &mut self,
        region: &Region,
    ) -> Result<Stmt, ConditionalValueBuildError> {
        let Region::TwoExitReturn {
            tests,
            test_edges,
            gateways,
            true_return,
            false_return,
            ..
        } = region
        else {
            unreachable!("two-exit builder needs its region")
        };
        if self.return_type != Some(Type::Boolean) {
            return Err(ConditionalValueBuildError::Refused(
                "two terminal returns require a boolean method descriptor".into(),
            ));
        }
        let mut leaves = BTreeMap::new();
        for (block, expected, opcode) in [(true_return, 1, 0x04), (false_return, 0, 0x03)] {
            let Some(names) = self.ssa.block(block) else {
                return Err(ConditionalValueBuildError::Refused(
                    "terminal return has no SSA block".into(),
                ));
            };
            let [push, returned] = names.instructions() else {
                return Err(ConditionalValueBuildError::Refused(
                    "terminal return is not an exact constant return".into(),
                ));
            };
            if push.opcode() != opcode
                || returned.opcode() != 0xac
                || !matches!(self.operations.get(push.bci()), Some(Operation::Push(ConstantValue::Int(value))) if *value == expected)
                || !matches!(self.operations.get(returned.bci()), Some(Operation::Return))
                || !push.reads().is_empty()
                || push.writes().len() != 1
                || returned.reads() != push.writes()
                || !returned.writes().is_empty()
                || self.ssa.value(push.writes()[0].1).uses().len() != 1
            {
                return Err(ConditionalValueBuildError::Refused(format!(
                    "return block at BCI {} is not an exact boolean constant return",
                    block.bci()
                )));
            }
            leaves.insert(
                block.clone(),
                Expr::direct(ExprKind::Boolean(expected == 1), push.bci())
                    .derived_from(returned.bci()),
            );
        }
        let mut expressions = BTreeMap::<CanonicalBlockId, Expr>::new();
        let mut sizes = BTreeMap::<CanonicalBlockId, usize>::new();
        let mut order = tests
            .iter()
            .enumerate()
            .map(|(index, (block, _))| (block.bci(), false, index))
            .chain(
                gateways
                    .iter()
                    .enumerate()
                    .map(|(index, (block, _))| (block.bci(), true, index)),
            )
            .collect::<Vec<_>>();
        order.sort_unstable_by_key(|entry| std::cmp::Reverse(entry.0));
        for (_, gateway, index) in order {
            if gateway {
                let (block, target) = &gateways[index];
                let Some(value) = leaves
                    .get(target)
                    .or_else(|| expressions.get(target))
                    .cloned()
                else {
                    return Err(ConditionalValueBuildError::Refused(
                        "transfer target has no proved expression".into(),
                    ));
                };
                let Some(names) = self.ssa.block(block) else {
                    return Err(ConditionalValueBuildError::Refused(
                        "transfer has no SSA block".into(),
                    ));
                };
                let [instruction] = names.instructions() else {
                    return Err(ConditionalValueBuildError::Refused(
                        "transfer has independent instructions".into(),
                    ));
                };
                if !matches!(
                    self.operations.get(instruction.bci()),
                    Some(Operation::Transfer)
                ) {
                    return Err(ConditionalValueBuildError::Refused(
                        "transfer is not pure".into(),
                    ));
                }
                charge(
                    self.budget,
                    CountedBudgetDimension::IrItems,
                    1,
                    Some(block.bci()),
                )
                .map_err(ConditionalValueBuildError::Stop)?;
                sizes.insert(block.clone(), sizes.get(target).copied().unwrap_or(1));
                expressions.insert(block.clone(), value.derived_from(instruction.bci()));
                continue;
            }
            let (block, bci) = &tests[index];
            let (fallthrough, taken) = &test_edges[index];
            let Some(when_true) = leaves
                .get(fallthrough)
                .or_else(|| expressions.get(fallthrough))
                .cloned()
            else {
                return Err(ConditionalValueBuildError::Refused(format!(
                    "test BCI {bci} has an unproved fallthrough"
                )));
            };
            let Some(when_false) = leaves
                .get(taken)
                .or_else(|| expressions.get(taken))
                .cloned()
            else {
                return Err(ConditionalValueBuildError::Refused(format!(
                    "test BCI {bci} has an unproved taken edge"
                )));
            };
            let size = 1usize
                .saturating_add(sizes.get(fallthrough).copied().unwrap_or(1))
                .saturating_add(sizes.get(taken).copied().unwrap_or(1));
            if size > 1024 {
                return Err(ConditionalValueBuildError::Refused(
                    "two-return expression exceeds expansion bound".into(),
                ));
            }
            charge(
                self.budget,
                CountedBudgetDimension::IrItems,
                u64::try_from(size).unwrap_or(u64::MAX),
                Some(*bci),
            )
            .map_err(ConditionalValueBuildError::Stop)?;
            sizes.insert(block.clone(), size);
            let Some(names) = self.ssa.block(block) else {
                return Err(ConditionalValueBuildError::Refused(
                    "test has no SSA block".into(),
                ));
            };
            let Some(branch) = names
                .instructions()
                .last()
                .filter(|instruction| instruction.bci() == *bci)
            else {
                return Err(ConditionalValueBuildError::Refused(
                    "test has no terminal branch".into(),
                ));
            };
            let mut dependencies = BTreeSet::new();
            for (_, operand) in stack_operands(branch) {
                self.conditional_dependencies(operand, &mut dependencies, &mut BTreeSet::new(), 0)?;
            }
            if dependencies
                .iter()
                .any(|source| self.block_of.get(source) != Some(block))
                || names.instructions().iter().any(|instruction| {
                    instruction.bci() != *bci && !dependencies.contains(&instruction.bci())
                })
            {
                return Err(ConditionalValueBuildError::Refused(format!(
                    "test BCI {bci} contains an independent effect or escaped producer"
                )));
            }
            let test = self.test_expr(*bci, false)?.derived_from(*bci);
            if test.presented != Some(Type::Boolean) {
                return Err(ConditionalValueBuildError::Refused(format!(
                    "test BCI {bci} has no Java boolean expression"
                )));
            }
            expressions.insert(
                block.clone(),
                Expr::direct(
                    ExprKind::Conditional {
                        test: Box::new(test),
                        when_true: Box::new(when_true),
                        when_false: Box::new(when_false),
                    },
                    *bci,
                ),
            );
        }
        let Some(value) = expressions.remove(&tests[0].0) else {
            return Err(ConditionalValueBuildError::Refused(
                "outer test has no expression".into(),
            ));
        };
        if value.presented != Some(Type::Boolean) {
            return Err(ConditionalValueBuildError::Refused(
                "two-return expression has no Java boolean type".into(),
            ));
        }
        let at = tests[0].1;
        let mut origin = OriginSet::new(Origin::direct(at));
        for block in region.blocks() {
            if let Some(names) = self.ssa.block(block) {
                for instruction in names.instructions() {
                    origin = origin.plus_derived(Origin::derived(instruction.bci()));
                }
            }
        }
        Ok(Stmt::new(StmtKind::Return { value: Some(value) }, origin))
    }

    /// Proves and builds conditional expressions before deferred-binding analysis. That analysis
    /// can then treat a successful stack Phi as one closed expression instead of following it into
    /// two control-flow arms it intentionally cannot traverse.
    fn prepare_conditional_regions(&mut self, regions: &[Region]) -> Result<(), StopReason> {
        let mut index = 0;
        while index < regions.len() {
            if let Some(next) = regions.get(index + 1)
                && self.prepare_carried_conditional_pair(&regions[index], next)?
            {
                index += 2;
                continue;
            }
            self.prepare_conditional_region(&regions[index])?;
            index += 1;
        }
        Ok(())
    }

    /// Stage both argument expressions and the final invocation before publishing either folded
    /// branch. A refused pair remains a complete quote, including its producer BCIs.
    fn prepare_carried_conditional_pair(
        &mut self,
        first: &Region,
        second: &Region,
    ) -> Result<bool, StopReason> {
        let (
            Region::If {
                branch_bci: first_bci,
                join: Some(first_join),
                then_arm: first_true,
                else_arm: first_false,
                ..
            },
            Region::If {
                branch: second_branch,
                branch_bci: second_bci,
                then_arm: second_true,
                else_arm: second_false,
                ..
            },
        ) = (first, second)
        else {
            return Ok(false);
        };
        if first_join != second_branch {
            return Ok(false);
        }
        let ConditionalValueAttempt::Proved(second_proof) = prove_conditional_value(
            second,
            self.canonical,
            self.ssa,
            self.operations,
            self.budget,
        )?
        else {
            return Ok(false);
        };
        if !matches!(
            self.operations.get(second_proof.consumer_bci),
            Some(Operation::Invoke(_))
        ) {
            return Ok(false);
        }
        let first_attempt = prove_conditional_value_with_forward(
            first,
            self.canonical,
            self.ssa,
            self.operations,
            Some((&second_proof, second)),
            self.budget,
        )?;
        let ConditionalValueAttempt::Proved(first_proof) = first_attempt else {
            let reason = "the earlier conditional argument has no closed carried-value proof";
            self.refuse_carried_conditional_pair(first, second, *first_bci, *second_bci, reason)?;
            return Ok(true);
        };
        let prepared = (|| -> Result<(Expr, Expr), ConditionalValueBuildError> {
            let first_expr = self.build_conditional_value(&first_proof, first_true, first_false)?;
            let second_expr =
                self.build_conditional_value(&second_proof, second_true, second_false)?;
            let at = second_proof.consumer_bci;
            let Some(instruction) = self.instructions.get(&at).copied() else {
                return Err(ConditionalValueBuildError::Refused(
                    "the carried-value invocation has no SSA instruction".into(),
                ));
            };
            let Some(Operation::Invoke(target)) = self.operations.get(at) else {
                return Err(ConditionalValueBuildError::Refused(
                    "the carried-value consumer is not an invocation".into(),
                ));
            };
            // All work between the two argument expressions must be part of the second test.
            // An independent call there cannot be moved into either Java argument.
            let test = self.test_expr(*second_bci, false)?;
            let test_sources = test.origin.bcis();
            let Some(test_block) = self.ssa.block(second_branch) else {
                return Err(ConditionalValueBuildError::Refused(
                    "the second argument has no test block".into(),
                ));
            };
            for instruction in test_block.instructions() {
                poll(self.budget, Some(instruction.bci()))
                    .map_err(ConditionalValueBuildError::Stop)?;
                charge(
                    self.budget,
                    CountedBudgetDimension::IrItems,
                    1,
                    Some(instruction.bci()),
                )
                .map_err(ConditionalValueBuildError::Stop)?;
                if instruction.bci() != *second_bci && !test_sources.contains(&instruction.bci()) {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the carried argument crosses an independent instruction at BCI {}",
                        instruction.bci()
                    )));
                }
            }
            let arguments = stack_operands(instruction);
            let first_argument = usize::from(target.kind() != InvokeKind::Static);
            if arguments.len() != first_argument + 2
                || arguments[first_argument]
                    != (Slot::Stack(first_proof.stack_depth), first_proof.phi)
                || arguments[first_argument + 1]
                    != (Slot::Stack(second_proof.stack_depth), second_proof.phi)
                || first_proof.stack_depth >= second_proof.stack_depth
            {
                return Err(ConditionalValueBuildError::Refused(
                    "the carried conditional values are not consecutive invocation arguments"
                        .into(),
                ));
            }
            let Some(descriptor) = self.invoke_descriptor(at) else {
                return Err(ConditionalValueBuildError::Refused(
                    "the carried-value invocation has no descriptor".into(),
                ));
            };
            self.arguments(
                &descriptor,
                vec![first_expr.clone(), second_expr.clone()],
                at,
            )?;
            if target.name() == "<init>" {
                if self.prologues.at(at).is_none() {
                    return Err(ConditionalValueBuildError::Refused(
                        "the carried-value constructor call has no proved prologue".into(),
                    ));
                }
            } else {
                self.conditional_values
                    .insert(first_proof.phi, first_expr.clone());
                self.conditional_values
                    .insert(second_proof.phi, second_expr.clone());
                let call = self.call_expr(at, instruction, target, at, 0);
                self.conditional_values.remove(&first_proof.phi);
                self.conditional_values.remove(&second_proof.phi);
                call.map_err(ConditionalValueBuildError::from)?;
            }
            Ok((first_expr, second_expr))
        })();
        match prepared {
            Ok((first_expr, second_expr)) => {
                self.conditional_values.insert(first_proof.phi, first_expr);
                self.conditional_values
                    .insert(second_proof.phi, second_expr);
                self.conditional_branches
                    .insert(*first_bci, ConditionalBranchPlan::Folded(first_proof.phi));
                self.conditional_branches
                    .insert(*second_bci, ConditionalBranchPlan::Folded(second_proof.phi));
            }
            Err(ConditionalValueBuildError::Refused(reason)) => {
                self.refuse_carried_conditional_pair(
                    first,
                    second,
                    *first_bci,
                    *second_bci,
                    &reason,
                )?;
            }
            Err(ConditionalValueBuildError::Stop(stop)) => return Err(stop),
        }
        Ok(true)
    }

    fn refuse_carried_conditional_pair(
        &mut self,
        first: &Region,
        second: &Region,
        first_bci: u32,
        second_bci: u32,
        reason: &str,
    ) -> Result<(), StopReason> {
        let mut quotes = Vec::with_capacity(2);
        for region in [first, second] {
            let mut bcis = Vec::new();
            for block in region.blocks() {
                let instructions = self.fallback_instruction_bcis(block)?;
                if instructions.is_empty() {
                    poll(self.budget, Some(block.bci()))?;
                    charge(
                        self.budget,
                        CountedBudgetDimension::AnalysisSteps,
                        1,
                        Some(block.bci()),
                    )?;
                    bcis.push(block.bci());
                } else {
                    bcis.extend(instructions);
                }
            }
            quotes.push(bcis);
        }
        for (bci, bcis) in [
            (first_bci, quotes.remove(0)),
            (second_bci, quotes.remove(0)),
        ] {
            self.conditional_branches.insert(
                bci,
                ConditionalBranchPlan::Refused(reason.to_owned(), Some(bcis)),
            );
        }
        Ok(())
    }

    fn prepare_conditional_region(&mut self, region: &Region) -> Result<(), StopReason> {
        match region {
            Region::Sequence { regions } => {
                self.prepare_conditional_regions(regions)?;
            }
            Region::If {
                then_arm,
                else_arm,
                branch_bci,
                ..
            } => {
                match prove_conditional_value(
                    region,
                    self.canonical,
                    self.ssa,
                    self.operations,
                    self.budget,
                )? {
                    ConditionalValueAttempt::Proved(proof) => {
                        match self.build_conditional_value(&proof, then_arm, else_arm) {
                            Ok(expression) => {
                                self.conditional_values.insert(proof.phi, expression);
                                self.conditional_branches
                                    .insert(*branch_bci, ConditionalBranchPlan::Folded(proof.phi));
                            }
                            Err(ConditionalValueBuildError::Refused(reason)) => {
                                self.conditional_branches.insert(
                                    *branch_bci,
                                    ConditionalBranchPlan::Refused(reason, None),
                                );
                            }
                            Err(ConditionalValueBuildError::Stop(stop)) => return Err(stop),
                        }
                    }
                    ConditionalValueAttempt::Refused(_) => {}
                }
                self.prepare_conditional_region(then_arm)?;
                self.prepare_conditional_region(else_arm)?;
            }
            Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => {
                for group in groups {
                    self.prepare_conditional_region(&group.arm)?;
                }
            }
            Region::Loop { body, .. } => {
                for region in body {
                    self.prepare_conditional_region(region)?;
                }
            }
            Region::Try { body, catches, .. } => {
                self.prepare_conditional_region(body)?;
                for clause in catches {
                    self.prepare_conditional_region(clause.body())?;
                }
            }
            Region::ShortCircuitValue { consumer, .. } => {
                if let ShortCircuitValueAttempt::Proved(proof) = prove_short_circuit_value(
                    region,
                    self.canonical,
                    self.ssa,
                    self.operations,
                    self.return_type.as_ref(),
                    self.budget,
                )? {
                    match self.build_short_circuit_statement(region, &proof) {
                        Ok((expression, statement)) => {
                            if let ShortCircuitConsumer::InstanceField(_, _, _, receiver) =
                                proof.consumer
                            {
                                self.short_circuit_operands.insert(receiver);
                            }
                            if let ShortCircuitConsumer::BooleanArray(array, index) = proof.consumer
                            {
                                self.short_circuit_operands.extend([array, index]);
                            }
                            self.conditional_values.insert(proof.phi, expression);
                            self.short_circuit_statements
                                .insert(consumer.clone(), statement);
                        }
                        Err(ConditionalValueBuildError::Refused(_)) => {}
                        Err(ConditionalValueBuildError::Stop(stop)) => return Err(stop),
                    }
                }
            }
            Region::TwoExitReturn { tests, .. } => match self.build_two_exit_return(region) {
                Ok(statement) => {
                    self.two_exit_returns.insert(tests[0].0.clone(), statement);
                }
                Err(ConditionalValueBuildError::Refused(_)) => {}
                Err(ConditionalValueBuildError::Stop(stop)) => return Err(stop),
            },
            Region::Straight { .. }
            | Region::Fallback { .. }
            | Region::Guard { .. }
            | Region::LoopBreak { .. }
            | Region::LoopContinue { .. } => {}
        }
        Ok(())
    }

    /// Prepares the complete consumer before suppressing either test or producer. The Phi
    /// expression and statement are published together only after Java typing succeeds.
    fn build_short_circuit_statement(
        &mut self,
        region: &Region,
        proof: &ShortCircuitValueProof,
    ) -> Result<(Expr, Stmt), ConditionalValueBuildError> {
        let at = proof.consumer_bci;
        let when_true = self.render_value(proof.true_producer, at, 0)?;
        let when_false = self.render_value(proof.false_producer, at, 0)?;
        let Region::ShortCircuitValue {
            true_producer,
            false_producer,
            ..
        } = region
        else {
            unreachable!("the short-circuit proof belongs to its region")
        };
        let Region::ShortCircuitValue {
            tests,
            test_edges,
            gateways,
            ..
        } = region
        else {
            unreachable!()
        };
        let mut expressions = BTreeMap::new();
        let mut expanded_sizes = BTreeMap::<CanonicalBlockId, usize>::new();
        let branch = |successor: &CanonicalBlockId,
                      expressions: &BTreeMap<CanonicalBlockId, Expr>| {
            if successor == true_producer {
                Some(when_true.clone())
            } else if successor == false_producer {
                Some(when_false.clone())
            } else {
                expressions.get(successor).cloned()
            }
        };
        let mut order = tests
            .iter()
            .enumerate()
            .map(|(index, (block, _))| (block.bci(), false, index))
            .chain(
                gateways
                    .iter()
                    .enumerate()
                    .map(|(index, (block, _))| (block.bci(), true, index)),
            )
            .collect::<Vec<_>>();
        order.sort_unstable_by_key(|entry| std::cmp::Reverse(entry.0));
        for (_, gateway, index) in order {
            if gateway {
                let (block, successor) = &gateways[index];
                let Some(value) = branch(successor, &expressions) else {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit transfer at BCI {} has an unproved successor",
                        block.bci()
                    )));
                };
                expanded_sizes.insert(
                    block.clone(),
                    expanded_sizes.get(successor).copied().unwrap_or(1),
                );
                charge(
                    self.budget,
                    CountedBudgetDimension::IrItems,
                    1,
                    Some(block.bci()),
                )
                .map_err(ConditionalValueBuildError::Stop)?;
                expressions.insert(block.clone(), value.derived_from(block.bci()));
                continue;
            }
            let (block, bci) = &tests[index];
            let (fallthrough, taken) = &test_edges[index];
            let Some(when_fallthrough) = branch(fallthrough, &expressions) else {
                return Err(ConditionalValueBuildError::Refused(format!(
                    "the short-circuit test at BCI {bci} has an unproved fallthrough"
                )));
            };
            let Some(when_taken) = branch(taken, &expressions) else {
                return Err(ConditionalValueBuildError::Refused(format!(
                    "the short-circuit test at BCI {bci} has an unproved taken edge"
                )));
            };
            let size = 1usize
                .saturating_add(expanded_sizes.get(fallthrough).copied().unwrap_or(1))
                .saturating_add(expanded_sizes.get(taken).copied().unwrap_or(1));
            if size > 1024 {
                return Err(ConditionalValueBuildError::Refused(format!(
                    "the short-circuit value at BCI {bci} exceeds its expression expansion bound"
                )));
            }
            expanded_sizes.insert(block.clone(), size);
            charge(
                self.budget,
                CountedBudgetDimension::IrItems,
                u64::try_from(size).unwrap_or(u64::MAX),
                Some(*bci),
            )
            .map_err(ConditionalValueBuildError::Stop)?;
            let test = self.test_expr(*bci, false)?.derived_from(*bci);
            expressions.insert(
                block.clone(),
                Expr::direct(
                    ExprKind::Conditional {
                        test: Box::new(test),
                        when_true: Box::new(when_fallthrough),
                        when_false: Box::new(when_taken),
                    },
                    *bci,
                ),
            );
        }
        let expression = expressions
            .remove(&tests[0].0)
            .expect("the outer test was built");
        if expression.presented != Some(Type::Int) {
            return Err(ConditionalValueBuildError::Refused(format!(
                "the short-circuit Phi at BCI {at} has no Java integer conditional type"
            )));
        }
        let (value, kind) = match &proof.consumer {
            ShortCircuitConsumer::Field(owner, name, descriptor) => {
                let Some((evidence, shape)) = self.fields.claim(at) else {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit field write at BCI {at} was not claimed by field@1"
                    )));
                };
                if evidence.access != FieldAccess::Write
                    || !evidence.is_static
                    || (&evidence.owner, &evidence.name, &evidence.descriptor)
                        != (owner, name, descriptor)
                    || shape.receiver.is_some()
                    || shape.value != Some(proof.phi)
                {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the field plan at BCI {at} does not name the proved static Phi consumer"
                    )));
                }
                let receiver = if self.fields.simple_static_final_write(at) {
                    None
                } else {
                    let owner = spell_reference(owner).ok_or_else(|| {
                        ConditionalValueBuildError::Refused(format!(
                            "the short-circuit field owner at BCI {at} has no Java type spelling"
                        ))
                    })?;
                    Some(Expr::direct(ExprKind::Path(owner), at))
                };
                let value =
                    self.field_value(Some(descriptor), proof.phi, expression.clone(), at)?;
                let kind = StmtKind::FieldAssign {
                    receiver,
                    name: name.clone(),
                    op: AssignOp::Assign,
                    value: value.clone(),
                };
                (value, kind)
            }
            ShortCircuitConsumer::InstanceField(owner, name, descriptor, receiver_value) => {
                let Some((evidence, shape)) = self.fields.claim(at) else {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit instance field at BCI {at} has no field claim"
                    )));
                };
                if evidence.access != FieldAccess::Write
                    || evidence.is_static
                    || (&evidence.owner, &evidence.name, &evidence.descriptor)
                        != (owner, name, descriptor)
                    || shape.receiver != Some(*receiver_value)
                    || shape.value != Some(proof.phi)
                {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the field plan at BCI {at} does not bind the proved instance receiver and Phi"
                    )));
                }
                let Some(owner_type) = spell_reference(owner) else {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit field owner at BCI {at} has no Java type spelling"
                    )));
                };
                let receiver = self.render_value(*receiver_value, at, 0)?;
                if receiver.presented != Some(Type::Reference(owner_type)) {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit receiver at BCI {at} has no exact field owner type"
                    )));
                }
                let value =
                    self.field_value(Some(descriptor), proof.phi, expression.clone(), at)?;
                let kind = StmtKind::FieldAssign {
                    receiver: Some(receiver),
                    name: name.clone(),
                    op: AssignOp::Assign,
                    value: value.clone(),
                };
                (value, kind)
            }
            ShortCircuitConsumer::BooleanArray(array_value, index_value) => {
                let Some(instruction) = self.instructions.get(&at).copied() else {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit array store at BCI {at} has no SSA instruction"
                    )));
                };
                if instruction.opcode() != 0x54
                    || !matches!(self.operations.get(at), Some(Operation::ArrayStore { .. }))
                    || array_element(self.ssa, self.operations, *array_value, None)
                        != Some(Type::Boolean)
                {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit array store at BCI {at} has no proven boolean component"
                    )));
                }
                let array = self.render_value(*array_value, at, 0)?;
                let index = self.render_value(*index_value, at, 0)?;
                if array.presented != Some(Type::Reference("boolean[]".to_string()))
                    || !matches!(
                        index.presented,
                        Some(Type::Byte | Type::Char | Type::Short | Type::Int)
                    )
                {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit array and index at BCI {at} have no Java operand types"
                    )));
                }
                let value = integer_low_bit_boolean(expression.clone(), at);
                let kind = StmtKind::IndexAssign {
                    array,
                    index,
                    op: AssignOp::Assign,
                    value: value.clone(),
                };
                (value, kind)
            }
            ShortCircuitConsumer::Return => {
                let value = self.adapt_return(expression.clone(), at)?;
                let kind = StmtKind::Return {
                    value: Some(value.clone()),
                };
                (value, kind)
            }
            ShortCircuitConsumer::Invoke(target) => {
                if self.pops().qualifier_at(at).is_some()
                    || self.conditional_values.contains_key(&proof.phi)
                {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit invocation at BCI {at} has an extra qualifier or value owner"
                    )));
                }
                let Some(instruction) = self.instructions.get(&at).copied() else {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit invocation at BCI {at} has no SSA instruction"
                    )));
                };
                let value = integer_low_bit_boolean(expression.clone(), at);
                self.conditional_values.insert(proof.phi, value.clone());
                let rendered = self.call_expr(at, instruction, target, at, 0);
                self.conditional_values.remove(&proof.phi);
                (value, StmtKind::Expr(rendered?))
            }
            ShortCircuitConsumer::Local(slot, written) => {
                let Some(variable) = self.reuse.variable_at(*slot, at) else {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit local at BCI {at} has no stable slot identity"
                    )));
                };
                let Some(name) = self.names.text(variable) else {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit local at BCI {at} has no Java name"
                    )));
                };
                if !self.decided_boolean(variable)
                    || !matches!(self.instructions.get(&at).map(|instruction| instruction.writes()), Some([(Slot::Local(write_slot), value)]) if write_slot == slot && value == written)
                {
                    return Err(ConditionalValueBuildError::Refused(format!(
                        "the short-circuit local at BCI {at} has no closed Boolean declaration decision"
                    )));
                }
                let value = integer_low_bit_boolean(expression.clone(), at);
                let kind = match self.declarations.placements.get(&variable) {
                    Some(DeclarationPlacement::Local { .. }) => StmtKind::Declare {
                        ty: Type::Boolean,
                        name: name.to_string(),
                        value: Some(value.clone()),
                    },
                    _ => {
                        return Err(ConditionalValueBuildError::Refused(format!(
                            "the short-circuit local at BCI {at} has no complete lexical owner"
                        )));
                    }
                };
                (value, kind)
            }
        };
        if value.presented != Some(Type::Boolean) {
            return Err(ConditionalValueBuildError::Refused(format!(
                "the short-circuit consumer value at BCI {at} is not a Java boolean"
            )));
        }
        let mut origin = OriginSet::new(Origin::direct(at));
        for bci in self.short_circuit_value_quote(region, proof.test_bcis[0], false) {
            if bci < at {
                origin = origin.plus_derived(Origin::derived(bci));
            }
        }
        let statement = Stmt::new(kind, origin);
        Ok((expression, statement))
    }

    /// Builds a conditional only after its two entire straight arms have been accounted for by the
    /// expressions the Phi inputs name. Extra statements, unknown types and any child this renderer
    /// refuses keep the whole region quoted.
    fn build_conditional_value(
        &mut self,
        proof: &ConditionalValueProof,
        then_arm: &Region,
        else_arm: &Region,
    ) -> Result<Expr, ConditionalValueBuildError> {
        let mut true_sources = BTreeSet::new();
        let mut false_sources = BTreeSet::new();
        self.conditional_dependencies(proof.when_true, &mut true_sources, &mut BTreeSet::new(), 0)?;
        self.conditional_dependencies(
            proof.when_false,
            &mut false_sources,
            &mut BTreeSet::new(),
            0,
        )?;
        self.conditional_arm_is_expression(then_arm, &true_sources, proof.branch_bci)?;
        self.conditional_arm_is_expression(else_arm, &false_sources, proof.branch_bci)?;

        let test = self
            .test_expr(proof.branch_bci, false)?
            .derived_from(proof.branch_bci);
        let when_true = self.render_value(proof.when_true, proof.consumer_bci, 0)?;
        let when_false = self.render_value(proof.when_false, proof.consumer_bci, 0)?;
        let boolean_return_values = self
            .instructions
            .get(&proof.consumer_bci)
            .filter(|instruction| instruction.opcode() == 0xac)
            .filter(|_| self.return_type == Some(Type::Boolean))
            .filter(|_| test.presented == Some(Type::Boolean))
            .and_then(|_| {
                if self.ssa.value(proof.when_true).replaced_by().is_some()
                    || self.ssa.value(proof.when_false).replaced_by().is_some()
                {
                    return None;
                }
                Some((
                    integer_constant(self.ssa, self.operations, proof.when_true)?,
                    integer_constant(self.ssa, self.operations, proof.when_false)?,
                ))
            });
        let kind = match boolean_return_values {
            Some((1, 0)) => test.kind.clone(),
            Some((0, 1)) => ExprKind::Not {
                value: Box::new(test.clone()),
            },
            _ => ExprKind::Conditional {
                test: Box::new(test.clone()),
                when_true: Box::new(when_true),
                when_false: Box::new(when_false),
            },
        };
        let expression = Expr::direct(kind, proof.consumer_bci);
        let Some(ty) = expression.presented.clone() else {
            return Err(ConditionalValueBuildError::Refused(format!(
                "the two values joined at BCI {} do not have a conditional Java type this run can prove",
                proof.consumer_bci
            )));
        };
        let mut origin = expression.origin;
        origin = origin.plus_derived(Origin::derived(proof.branch_bci));
        // A projected test no longer has its own Expr node, so carry its root anchor onto the
        // replacement node as well as retaining its operand subtree.
        for bci in test.origin.bcis() {
            origin = origin.plus_derived(Origin::derived(bci));
        }
        for bci in true_sources.iter().chain(&false_sources) {
            origin = origin.plus_derived(Origin::derived(*bci));
        }
        // Transfers are structural evidence of the two arms. They do not become child expressions,
        // so retain their BCI anchors on the conditional node itself.
        for arm in [then_arm, else_arm] {
            if let Region::Straight { blocks } = arm {
                for block in blocks {
                    if let Some(ssa_block) = self.ssa.block(block) {
                        for instruction in ssa_block.instructions() {
                            if matches!(
                                self.operations.get(instruction.bci()),
                                Some(Operation::Transfer)
                            ) {
                                origin = origin.plus_derived(Origin::derived(instruction.bci()));
                            }
                        }
                    }
                }
            }
        }
        Ok(Expr::new(expression.kind, origin).presenting(ty))
    }

    /// Collects every instruction value the renderer would inline into one arm expression.
    fn conditional_dependencies(
        &mut self,
        value: ValueId,
        dependencies: &mut BTreeSet<u32>,
        active: &mut BTreeSet<ValueId>,
        depth: usize,
    ) -> Result<(), ConditionalValueBuildError> {
        if depth > MAX_VALUE_DEPTH || !active.insert(value) {
            return Err(ConditionalValueBuildError::Refused(
                "a conditional arm value is cyclic or exceeds the expression depth bound"
                    .to_string(),
            ));
        }
        let Some(bci) = (match self.ssa.value(value).def() {
            Definition::Instruction { bci, .. } => Some(*bci),
            _ => None,
        }) else {
            active.remove(&value);
            return Ok(());
        };
        poll(self.budget, Some(bci)).map_err(ConditionalValueBuildError::Stop)?;
        charge(self.budget, CountedBudgetDimension::IrItems, 1, Some(bci))
            .map_err(ConditionalValueBuildError::Stop)?;
        let fresh = dependencies.insert(bci);
        if !fresh {
            active.remove(&value);
            return Ok(());
        }
        let Some(instruction) = self.instructions.get(&bci).copied() else {
            return Err(ConditionalValueBuildError::Refused(format!(
                "no names record exists for conditional arm value BCI {bci}"
            )));
        };
        let uses = self.ssa.value(value).uses();
        for usage in uses {
            charge(self.budget, CountedBudgetDimension::IrItems, 1, usage.bci())
                .map_err(ConditionalValueBuildError::Stop)?;
        }
        // Rendering a producer twice can run a call or allocation twice. Every instruction value
        // in either expression, including its Phi root, must therefore have exactly one reader.
        if uses.len() != 1 {
            active.remove(&value);
            return Err(ConditionalValueBuildError::Refused(format!(
                "the conditional arm producer at BCI {bci} does not have exactly one expression use"
            )));
        }
        if matches!(self.operations.get(bci), None | Some(Operation::Other)) {
            return Err(ConditionalValueBuildError::Refused(format!(
                "the conditional arm operation at BCI {bci} is not modeled as a value expression"
            )));
        }
        for (_, operand) in stack_operands(instruction) {
            self.conditional_dependencies(operand, dependencies, active, depth + 1)?;
        }
        active.remove(&value);
        Ok(())
    }

    fn conditional_arm_is_expression(
        &mut self,
        arm: &Region,
        dependencies: &BTreeSet<u32>,
        at: u32,
    ) -> Result<(), ConditionalValueBuildError> {
        let Region::Straight { blocks } = arm else {
            return Err(ConditionalValueBuildError::Refused(format!(
                "the conditional arm at BCI {at} is not straight-line code"
            )));
        };
        for block in blocks {
            let Some(ssa_block) = self.ssa.block(block) else {
                return Err(ConditionalValueBuildError::Refused(format!(
                    "the conditional arm at BCI {at} has no SSA block"
                )));
            };
            for instruction in ssa_block.instructions() {
                let bci = instruction.bci();
                poll(self.budget, Some(bci)).map_err(ConditionalValueBuildError::Stop)?;
                charge(self.budget, CountedBudgetDimension::IrItems, 1, Some(bci))
                    .map_err(ConditionalValueBuildError::Stop)?;
                if dependencies.contains(&bci)
                    || matches!(self.operations.get(bci), Some(Operation::Transfer))
                {
                    continue;
                }
                return Err(ConditionalValueBuildError::Refused(format!(
                    "the conditional arm contains an independent instruction at BCI {bci}"
                )));
            }
        }
        Ok(())
    }

    /// What the plan decided about one variable's type, when the plan reached that variable.
    fn decision(&self, variable: LocalVariable) -> Option<&Decided> {
        self.declarations.decided.get(&variable)
    }

    /// Whether the plan decided one variable holds a `boolean`.
    fn decided_boolean(&self, variable: LocalVariable) -> bool {
        self.decision(variable).is_some_and(Decided::is_boolean)
    }

    /// Appends the statements of one region, with the region's own declarations first.
    fn region(&mut self, region: &Region, path: &RegionPath) -> Result<(), StopReason> {
        if let Region::TwoExitReturn { prefix, tests, .. } = region {
            let (outer, at) = &tests[0];
            if let Some(incomplete) = self.declarations.incomplete.get(path).cloned() {
                let bcis = self.two_exit_return_quote(region, *at)?;
                return self.fallback(bcis, &incomplete, *at);
            }
            let Some(statement) = self.two_exit_returns.get(outer).cloned() else {
                let bcis = self.two_exit_return_quote(region, *at)?;
                return self.fallback(
                    bcis,
                    "the shared terminal boolean return is not completely proved",
                    *at,
                );
            };
            if undeclared_local(&statement, &self.undeclared).is_some() {
                let bcis = self.two_exit_return_quote(region, *at)?;
                return self.fallback(
                    bcis,
                    "the shared terminal boolean return reads an undeclared local",
                    *at,
                );
            }
            self.declare_at(path)?;
            for block in prefix {
                self.block(block)?;
            }
            return self.push(statement);
        }
        if let Region::ShortCircuitValue {
            prefix,
            tests,
            consumer,
            consumer_bci,
            reason,
            ..
        } = region
        {
            let (outer_branch, outer_branch_bci) = &tests[0];
            if let Some(incomplete) = self.declarations.incomplete.get(path).cloned() {
                let bcis = self.short_circuit_value_quote(region, 0, true);
                return self.fallback(bcis, &incomplete, *outer_branch_bci);
            }
            // A candidate Region owns the whole control-flow closure even when its value proof
            // fails. Quote that closure before emitting any prefix or test effects; otherwise a
            // near miss can leave only the join's consumer visible and lose its producers.
            if !self.short_circuit_statements.contains_key(consumer) {
                let bcis = self.short_circuit_value_quote(region, 0, true);
                return self.fallback(bcis, reason.message(), *outer_branch_bci);
            }
            self.declare_at(path)?;
            let Some(statement) = self.short_circuit_statements.get(consumer).cloned() else {
                unreachable!("the staged consumer was checked before declaring locals")
            };
            if undeclared_local(&statement, &self.undeclared).is_some()
                || self.settled.contains(consumer_bci)
            {
                let bcis = self.short_circuit_value_quote(region, 0, true);
                return self.fallback(bcis, reason.message(), *outer_branch_bci);
            }
            for block in prefix {
                self.block(block)?;
            }
            self.test_effects(outer_branch, *outer_branch_bci)?;
            let declared_local = matches!(statement.kind, StmtKind::Declare { .. });
            self.push(statement)?;
            if declared_local
                && let Some(Operation::Store { slot }) = self.operations.get(*consumer_bci)
                && let Some(variable) = self.reuse.variable_at(*slot, *consumer_bci)
            {
                self.declared.insert(variable);
            }
            self.settled.insert(*consumer_bci);
            let suffix = self.block(consumer);
            self.settled.remove(consumer_bci);
            return suffix;
        }
        if let Some(reason) = self.declarations.incomplete.get(path).cloned() {
            let bcis = self.region_quote(region, region.blocks().first().map_or(0, |b| b.bci()));
            let at = bcis.first().copied().unwrap_or(0);
            return self.fallback(bcis, &reason, at);
        }
        self.declare_at(path)?;
        match region {
            Region::Sequence { regions } => {
                for (index, region) in regions.iter().enumerate() {
                    self.region(
                        region,
                        &child(path, u32::try_from(index).unwrap_or(u32::MAX)),
                    )?;
                }
                Ok(())
            }
            Region::Straight { blocks } => {
                for block in blocks {
                    self.block(block)?;
                }
                Ok(())
            }
            Region::If {
                prefix,
                branch,
                branch_bci,
                then_arm,
                else_arm,
                ..
            } => {
                for block in prefix {
                    self.block(block)?;
                }
                // Everything the test block does *before* its branch ran before the branch in the
                // bytecode too, so it is written here, in order: a store or a call the test block
                // makes is a statement of its own, and it must not be dropped just because the
                // branch that follows it is what becomes the `if`.
                match self.conditional_branches.get(branch_bci).cloned() {
                    Some(ConditionalBranchPlan::Folded(_phi)) => {
                        self.test_effects(branch, *branch_bci)?;
                        // The join's one consumer writes the complete conditional value. Emitting
                        // either arm here would run its producer separately and then again inside
                        // that expression.
                        return Ok(());
                    }
                    Some(ConditionalBranchPlan::Refused(reason, Some(bcis))) => {
                        return self.fallback(bcis, &reason, *branch_bci);
                    }
                    Some(ConditionalBranchPlan::Refused(reason, None)) => {
                        self.test_effects(branch, *branch_bci)?;
                        let bcis = self.region_quote(region, *branch_bci);
                        return self.fallback(bcis, &reason, *branch_bci);
                    }
                    None => {}
                }
                self.test_effects(branch, *branch_bci)?;
                let cond = match self.test_expr(*branch_bci, false) {
                    Ok(cond) => cond,
                    Err(reason) => {
                        let bcis = self.region_quote(region, *branch_bci);
                        return self.fallback(bcis, &reason, *branch_bci);
                    }
                };
                // The condition's text is tested *by* the branch: the `if` statement is the branch,
                // and the condition node presents it. One BCI reaching two nodes with the two
                // provenances is exactly what the segment table records as it writes.
                let cond = cond.derived_from(*branch_bci);
                let mut then_body = Vec::new();
                self.arm(then_arm, &mut then_body, &child(path, 0))?;
                let mut else_body = Vec::new();
                self.arm(else_arm, &mut else_body, &child(path, 1))?;
                self.push(Stmt::new(
                    StmtKind::If {
                        cond,
                        then_body,
                        else_body,
                    },
                    OriginSet::new(Origin::direct(*branch_bci)),
                ))
            }
            Region::StringSwitch {
                dispatch,
                proof,
                groups,
                join,
            } => {
                let at = proof.hash_switch_bci;
                let Some(selector_store) =
                    self.instructions.get(&proof.selector_store_bci).copied()
                else {
                    let bcis = self.region_quote(region, at);
                    return self.fallback(
                        bcis,
                        "the String selector save has no SSA instruction",
                        at,
                    );
                };
                let Some((_, selector_id)) = stack_operands(selector_store).last().copied() else {
                    let bcis = self.region_quote(region, at);
                    return self.fallback(bcis, "the String selector save has no value", at);
                };
                let mut dependencies = BTreeSet::new();
                if !self.collect_dependency_bcis(
                    selector_id,
                    &mut BTreeSet::new(),
                    &mut dependencies,
                    0,
                )? {
                    let bcis = self.region_quote(region, at);
                    return self.fallback(
                        bcis,
                        "the String selector's expression is incomplete",
                        at,
                    );
                }
                let dispatch_bcis: BTreeSet<u32> = dispatch
                    .iter()
                    .filter_map(|block| self.ssa.block(block))
                    .flat_map(|block| block.instructions().iter().map(SsaInstruction::bci))
                    .collect();
                if !dependencies.is_subset(&dispatch_bcis) {
                    let bcis = self.region_quote(region, at);
                    return self.fallback(
                        bcis,
                        "the String selector's producers escape its dispatch region",
                        at,
                    );
                }
                // Independent effects before the selector save keep their original order. The
                // selector's producer tree is written once inside the switch expression.
                for block in dispatch {
                    let instructions: Vec<SsaInstruction> = self
                        .ssa
                        .block(block)
                        .map(|block| block.instructions().to_vec())
                        .unwrap_or_default();
                    for instruction in &instructions {
                        if instruction.bci() < proof.selector_store_bci
                            && !dependencies.contains(&instruction.bci())
                        {
                            self.instruction(instruction)?;
                        }
                    }
                }
                let value = match self.render_value(selector_id, proof.selector_store_bci, 0) {
                    Ok(value) => value.derived_from(at).derived_from(proof.final_switch_bci),
                    Err(reason) => {
                        let bcis = self.region_quote(region, at);
                        return self.fallback(bcis, &reason, at);
                    }
                };
                let joined = self.switch_join(join.as_ref(), groups);
                let mut arms = Vec::with_capacity(groups.len());
                self.switch_depth += 1;
                for (index, group) in groups.iter().enumerate() {
                    let mut body = Vec::new();
                    self.switch_arm(
                        &group.arm,
                        &mut body,
                        &child(path, u32::try_from(index).unwrap_or(u32::MAX)),
                    )?;
                    let labels = group
                        .keys
                        .iter()
                        .filter_map(|key| {
                            proof
                                .labels
                                .iter()
                                .find(|(_, mapped)| mapped == key)
                                .map(|(literal, _)| literal.clone())
                        })
                        .collect();
                    arms.push(SwitchArm {
                        keys: group.keys.clone(),
                        labels: Some(SwitchLabels::String(labels)),
                        default: group.default,
                        fall_through: group.fall_through,
                        body,
                    });
                }
                self.switch_depth -= 1;
                if let Some((join_bci, values)) = joined {
                    let mut returns = Vec::with_capacity(values.len());
                    for (value, at) in values {
                        let value = match self.return_expr(value, at, join_bci) {
                            Ok(value) => value,
                            Err(reason) => {
                                let bcis = self.region_quote(region, proof.final_switch_bci);
                                return self.fallback(bcis, &reason, proof.final_switch_bci);
                            }
                        };
                        returns.push(Stmt::new(
                            StmtKind::Return { value: Some(value) },
                            OriginSet::new(Origin::direct(join_bci)),
                        ));
                    }
                    if let Some((name, _)) = returns
                        .iter()
                        .find_map(|statement| undeclared_local(statement, &self.undeclared))
                    {
                        let bcis = self.region_quote(region, proof.final_switch_bci);
                        return self.fallback(bcis, format!("the switch expression's arm return reads `{name}`, but its declaration was refused"), proof.final_switch_bci);
                    }
                    for (arm, statement) in arms.iter_mut().zip(returns) {
                        self.push_into(&mut arm.body, statement)?;
                    }
                    self.settled.insert(join_bci);
                }
                let mut origin = OriginSet::new(Origin::direct(proof.final_switch_bci));
                for bci in &proof.owned_bcis {
                    origin = origin.plus_derived(Origin::derived(*bci));
                }
                self.push(Stmt::new(StmtKind::Switch { value, arms }, origin))
            }
            Region::Switch {
                prefix,
                branch,
                branch_bci,
                groups,
                join,
            } => {
                for block in prefix {
                    self.block(block)?;
                }
                // As for an `if`: what the switch block does before its selector is read runs
                // before the switch in the bytecode too.
                self.test_effects(branch, *branch_bci)?;
                let Some(instruction) = self.instructions.get(branch_bci).copied() else {
                    let bcis = self.region_quote(region, *branch_bci);
                    return self.fallback(
                        bcis,
                        format!("no names record for the switch at BCI {branch_bci}"),
                        *branch_bci,
                    );
                };
                let Some((_, value)) = stack_operands(instruction).last().copied() else {
                    let bcis = self.region_quote(region, *branch_bci);
                    return self.fallback(
                        bcis,
                        format!("the switch at BCI {branch_bci} reads no value to select on"),
                        *branch_bci,
                    );
                };
                // The selector is a value the switch reads; the text that renders it is anchored
                // where the value was produced, and the switch that reads it is added as a
                // presented anchor, exactly like a branch's condition.
                let value = match self.render_value(value, *branch_bci, 0) {
                    Ok(value) => value.derived_from(*branch_bci),
                    Err(reason) => {
                        let bcis = self.region_quote(region, *branch_bci);
                        return self.fallback(bcis, &reason, *branch_bci);
                    }
                };
                // P3 2c.25: the shape a `switch` **expression** compiles to — every arm leaves one
                // value on the operand stack and transfers to the block whose only instruction is
                // the member's own `return`, the default arm falling into it among them. The join's
                // entry state names a stack phi that no instruction produced, which no arm's text
                // can write; the `return` is written into each arm instead, where its own value is
                // produced, and the join's instruction is then skipped ([`Self::settled`]).
                //
                // Keep only the arm value identities until the arms have been built. A producer may
                // be refused while an arm is walked; rendering the return before that walk would
                // retain an executable expression that the refusal had already made unavailable.
                let joined = self.switch_join(join.as_ref(), groups);
                let mut arms = Vec::with_capacity(groups.len());
                self.switch_depth += 1;
                for (index, group) in groups.iter().enumerate() {
                    let mut body = Vec::new();
                    self.switch_arm(
                        &group.arm,
                        &mut body,
                        &child(path, u32::try_from(index).unwrap_or(u32::MAX)),
                    )?;
                    arms.push(SwitchArm {
                        keys: group.keys.clone(),
                        labels: None,
                        default: group.default,
                        fall_through: group.fall_through,
                        body,
                    });
                }
                self.switch_depth -= 1;
                // Render the `return`s only now, after all producer refusals and successful
                // declarations from the arms are visible. An arm may declare the local the join's
                // value reads (the shape `int local1 = arg0 + 1; … return local1;` has), so the
                // question "did this body declare it" can only be answered once the arms exist.
                if let Some((join_bci, values)) = joined {
                    let mut returns = Vec::with_capacity(values.len());
                    for (value, at) in values {
                        let value = match self.return_expr(value, at, join_bci) {
                            Ok(value) => value,
                            Err(reason) => {
                                let bcis = self.region_quote(region, *branch_bci);
                                return self.fallback(bcis, &reason, *branch_bci);
                            }
                        };
                        returns.push(Stmt::new(
                            StmtKind::Return { value: Some(value) },
                            OriginSet::new(Origin::direct(join_bci)),
                        ));
                    }
                    if let Some((name, _)) = returns
                        .iter()
                        .find_map(|statement| undeclared_local(statement, &self.undeclared))
                    {
                        let bcis = self.region_quote(region, *branch_bci);
                        return self.fallback(
                            bcis,
                            format!(
                                "the switch expression's arm return reads `{name}`, but its declaration was refused"
                            ),
                            *branch_bci,
                        );
                    }
                    for (arm, statement) in arms.iter_mut().zip(returns) {
                        self.push_into(&mut arm.body, statement)?;
                    }
                    // The join's own `return` is the statement every arm now ends in: walking it
                    // again would write the arm's value at the join, where the phi — not the arm's
                    // own instruction — is what the bytecode holds.
                    self.settled.insert(join_bci);
                }
                self.push(Stmt::new(
                    StmtKind::Switch { value, arms },
                    OriginSet::new(Origin::direct(*branch_bci)),
                ))
            }
            Region::Loop {
                header,
                test_bci,
                form,
                continuation,
                for_header,
                body,
                ..
            } => {
                // The test is written *inside* the loop statement, so the values it reads and the
                // calls it makes run once per evaluation — the loop's own count, not the count of a
                // hoisted copy. Which of the two senses continues the loop is a decode fact
                // (`Continuation`), and the condition is that sense, not its negation.
                let taken = *continuation == Continuation::Taken;
                let cond = match self.test_expr(*test_bci, taken) {
                    Ok(cond) => cond,
                    Err(reason) => {
                        let bcis = self.region_quote(region, *test_bci);
                        return self.fallback(bcis, &reason, *test_bci);
                    }
                };
                let cond = cond.derived_from(*test_bci);
                let for_init_index = for_header.as_ref().and_then(|proof| {
                    let index = self.stmts.iter().rposition(|stmt| {
                        stmt.origin.primary().bci() == proof.init_bci
                            && matches!(
                                stmt.kind,
                                StmtKind::Declare { value: Some(_), .. } | StmtKind::Assign { .. }
                            )
                    })?;
                    self.stmts[index + 1..]
                        .iter()
                        .all(|stmt| matches!(stmt.kind, StmtKind::Declare { value: None, .. }))
                        .then_some(index)
                });
                if let Some(proof) = for_header {
                    if for_init_index.is_none() {
                        let bcis = self.region_quote(region, *test_bci);
                        return self.fallback(
                            bcis,
                            "the counted loop's preceding local initialisation did not produce one Java header clause",
                            *test_bci,
                        );
                    }
                    self.settled.insert(proof.update_bci);
                }
                let header_bci = header.bci();
                self.loop_headers.push(header_bci);
                let outer = std::mem::take(&mut self.stmts);
                let outer_declared = self.declared.clone();
                let walked = body.iter().enumerate().try_for_each(|(index, region)| {
                    self.region(
                        region,
                        &child(path, u32::try_from(index).unwrap_or(u32::MAX)),
                    )
                });
                let loop_body = std::mem::replace(&mut self.stmts, outer);
                self.declared = outer_declared;
                self.loop_headers.pop();
                walked?;
                let label = self
                    .labeled_loop_headers
                    .contains(&header_bci)
                    .then(|| loop_label(header_bci));
                let kind = if let Some(proof) = for_header {
                    self.settled.remove(&proof.update_bci);
                    let Some(update_instruction) =
                        self.instructions.get(&proof.update_bci).copied()
                    else {
                        let bcis = self.region_quote(region, *test_bci);
                        return self.fallback(
                            bcis,
                            "the proved for update has no SSA instruction",
                            *test_bci,
                        );
                    };
                    let outer = std::mem::take(&mut self.stmts);
                    let built = self.instruction(update_instruction);
                    let mut update_stmts = std::mem::replace(&mut self.stmts, outer);
                    built?;
                    let Some(update) = update_stmts.pop() else {
                        let bcis = self.region_quote(region, *test_bci);
                        return self.fallback(
                            bcis,
                            "the proved for update made no Java assignment",
                            *test_bci,
                        );
                    };
                    if !update_stmts.is_empty() || !matches!(update.kind, StmtKind::Assign { .. }) {
                        let bcis = self.region_quote(region, *test_bci);
                        return self.fallback(
                            bcis,
                            "the proved for update did not make one local assignment",
                            *test_bci,
                        );
                    }
                    let Some(initial) = for_init_index.and_then(|index| self.stmts.get(index))
                    else {
                        let bcis = self.region_quote(region, *test_bci);
                        return self.fallback(
                            bcis,
                            "the proved for initialisation is absent",
                            *test_bci,
                        );
                    };
                    let init_name = match &initial.kind {
                        StmtKind::Declare { name, .. } | StmtKind::Assign { name, .. } => name,
                        _ => unreachable!("initialisation checked before building loop body"),
                    };
                    let StmtKind::Assign {
                        name: update_name, ..
                    } = &update.kind
                    else {
                        unreachable!("update checked above")
                    };
                    if init_name != update_name {
                        let bcis = self.region_quote(region, *test_bci);
                        return self.fallback(
                            bcis,
                            "the for initialisation and update name different locals",
                            *test_bci,
                        );
                    }
                    let initial = self
                        .stmts
                        .remove(for_init_index.expect("checked initialisation index"));
                    self.statements = self.statements.saturating_sub(1);
                    self.statements = self.statements.saturating_sub(1);
                    StmtKind::For {
                        label,
                        init: Box::new(initial),
                        cond,
                        update: Box::new(update),
                        body: loop_body,
                    }
                } else {
                    match form {
                        LoopForm::While => StmtKind::While {
                            label,
                            cond,
                            body: loop_body,
                        },
                        LoopForm::DoWhile => StmtKind::DoWhile {
                            label,
                            cond,
                            body: loop_body,
                        },
                    }
                };
                let projected = if let Some(proof) = for_header {
                    self.array_for_each_candidate(proof, *test_bci, path, &kind, body)?
                } else {
                    None
                };
                let iterable_projected = if for_header.is_none() {
                    self.iterable_for_each_candidate(*test_bci, &kind, body)?
                } else {
                    None
                };
                let (kind, origin) =
                    if let Some((kind, origin, removed_element, declarations)) = projected {
                        for index in declarations.iter().rev().copied() {
                            self.stmts.remove(index);
                        }
                        self.stmts.pop(); // The proved, otherwise unused length cache.
                        self.statements = self.statements.saturating_sub(
                            if removed_element { 2 } else { 1 } + declarations.len(),
                        );
                        if !removed_element {
                            let StmtKind::ForEach { name, .. } = &kind else {
                                unreachable!("array projection produces an enhanced for")
                            };
                            self.synthetic_names.insert(name.clone());
                        }
                        (kind, origin)
                    } else if let Some((kind, origin, mut remove, name)) = iterable_projected {
                        remove.sort_unstable_by(|left, right| right.cmp(left));
                        for index in remove {
                            self.stmts.remove(index);
                            self.statements = self.statements.saturating_sub(1);
                        }
                        self.synthetic_names.insert(name);
                        (kind, origin)
                    } else {
                        (kind, OriginSet::new(Origin::direct(*test_bci)))
                    };
                self.push(Stmt::new(kind, origin))
            }
            Region::LoopBreak {
                source_bci,
                loop_header,
            } => {
                let target = loop_header.bci();
                let Some(position) = self
                    .loop_headers
                    .iter()
                    .rposition(|header| *header == target)
                else {
                    return self.fallback(
                        vec![*source_bci],
                        "a loop break targets no enclosing loop proven by the region tree",
                        *source_bci,
                    );
                };
                let label = if position + 1 != self.loop_headers.len() || self.switch_depth > 0 {
                    self.labeled_loop_headers.insert(target);
                    Some(loop_label(target))
                } else {
                    None
                };
                self.push(Stmt::new(
                    StmtKind::Break { label },
                    OriginSet::new(Origin::direct(*source_bci)),
                ))
            }
            Region::LoopContinue {
                source_bci,
                loop_header,
            } => {
                let target = loop_header.bci();
                let Some(position) = self
                    .loop_headers
                    .iter()
                    .rposition(|header| *header == target)
                else {
                    return self.fallback(
                        vec![*source_bci],
                        "a loop continue targets no enclosing loop proven by the region tree",
                        *source_bci,
                    );
                };
                let label = (position + 1 != self.loop_headers.len()).then(|| {
                    self.labeled_loop_headers.insert(target);
                    loop_label(target)
                });
                self.push(Stmt::new(
                    StmtKind::Continue { label },
                    OriginSet::new(Origin::direct(*source_bci)),
                ))
            }
            Region::Guard { prefix, plan } => {
                // The walk wrote nothing of the statement's own header: the first resource's
                // initialisation (or the monitor's entry) is the last block it reached, and the
                // region owns it now.
                for block in prefix {
                    self.block(block)?;
                }
                self.range(plan.lead())?;
                match plan.shape() {
                    guard::Shape::Resources {
                        resources, returns, ..
                    } => {
                        let mut declarations = Vec::with_capacity(resources.len());
                        for resource in resources {
                            match self.resource_declaration(resource) {
                                Ok(declaration) => declarations.push(declaration),
                                Err(reason) => {
                                    let at = resource.close_bci();
                                    let bcis = self.region_quote(region, at);
                                    return self.fallback(bcis, &reason, at);
                                }
                            }
                        }
                        // The body's statements are written between the braces, in the order the
                        // instructions run: the closes the normal path performs are *not* written
                        // here — the compiler writes them for the resource the header declares,
                        // which is what makes each close run exactly once per path.
                        let mut body = self.body_range(plan.body())?;
                        if let Some(return_bci) = returns {
                            let statement = match self.guarded_return(*return_bci) {
                                Ok(statement) => statement,
                                Err(reason) => {
                                    let bcis = self.region_quote(region, *return_bci);
                                    return self.fallback(bcis, &reason, *return_bci);
                                }
                            };
                            body.push(statement);
                        }
                        let mut origin = OriginSet::new(Origin::direct(
                            resources
                                .first()
                                .map(|resource| resource.init().0)
                                .unwrap_or(0),
                        ));
                        for bci in plan.facts() {
                            // Every instruction the proof read — the closes, the suppressions, the
                            // rethrows — is an anchor of the text that took its place, so the
                            // relationship the statement preserves is answerable from the artifact.
                            origin = origin.plus_derived(Origin::derived(*bci));
                        }
                        self.push(Stmt::new(
                            StmtKind::Try {
                                resources: declarations,
                                // A guarded statement has no clauses: a `catch` beside a `try` header
                                // is the `Unexplained` refusal, and this node is only written where
                                // that proof succeeded.
                                catches: Vec::new(),
                                body,
                                finally_body: None,
                            },
                            origin,
                        ))
                    }
                    guard::Shape::Monitor {
                        enter_bci, returns, ..
                    } => {
                        let lock = match self.lock_expr(*enter_bci) {
                            Ok(lock) => lock,
                            Err(reason) => {
                                let bcis = self.region_quote(region, *enter_bci);
                                return self.fallback(bcis, &reason, *enter_bci);
                            }
                        };
                        let mut body = self.body_range(plan.body())?;
                        // The `return` shape: the normal path returns the value the body's own
                        // instructions left on the stack, and the statement has to end with that
                        // `return` **inside** its braces — the field read it names is written here,
                        // where the value is consumed, and not where the `getfield` runs, so the
                        // member is read exactly once and no local is invented for it.
                        if let Some(return_bci) = *returns {
                            let statement = match self.guarded_return(return_bci) {
                                Ok(statement) => statement,
                                Err(reason) => {
                                    let bcis = self.region_quote(region, return_bci);
                                    return self.fallback(bcis, &reason, return_bci);
                                }
                            };
                            body.push(statement);
                        }
                        let mut origin = OriginSet::new(Origin::direct(*enter_bci));
                        for bci in plan.facts() {
                            origin = origin.plus_derived(Origin::derived(*bci));
                        }
                        self.push(Stmt::new(StmtKind::Synchronized { lock, body }, origin))
                    }
                    guard::Shape::Finally {
                        normal_cleanup,
                        returns,
                    } => {
                        // The guard has already rejected every control transfer inside this flat
                        // range. The saved local is written here before cleanup can run.
                        let mut body = self.body_range(plan.body())?;
                        let return_stmt = match self.guarded_return(*returns) {
                            Ok(statement) => statement,
                            Err(reason) => {
                                let bcis = self.region_quote(region, *returns);
                                return self.fallback(bcis, &reason, *returns);
                            }
                        };
                        body.push(return_stmt);
                        let finally_body = self.body_range(*normal_cleanup)?;
                        if body
                            .iter()
                            .chain(&finally_body)
                            .any(|statement| matches!(statement.kind, StmtKind::Fallback { .. }))
                        {
                            let bcis = self.region_quote(region, *returns);
                            return self.fallback(
                                bcis,
                                "the proved finally contains an instruction this Java writer cannot state",
                                *returns,
                            );
                        }
                        let mut origin = OriginSet::new(Origin::direct(plan.body().0));
                        for bci in plan.facts() {
                            origin = origin.plus_derived(Origin::derived(*bci));
                        }
                        self.push(Stmt::new(
                            StmtKind::Try {
                                resources: Vec::new(),
                                catches: Vec::new(),
                                body,
                                finally_body: Some(finally_body),
                            },
                            origin,
                        ))
                    }
                }
            }
            Region::Try {
                prefix,
                lead,
                body,
                catches,
            } => {
                for block in prefix {
                    self.block(block)?;
                }
                // The statement's own block may hold what ran before the protected range began —
                // `int x = 1;` in front of `try { … }` — and those instructions are not part of the
                // range: they are written here, before the `try`, exactly like the block's own lead
                // in front of an `if`. The body still begins in that same block, so the instructions
                // this wrote are the ones it must not write again.
                self.range(*lead)?;
                let lead: BTreeSet<u32> = self
                    .instructions
                    .range(lead.0..lead.1)
                    .map(|(bci, _)| *bci)
                    .collect();
                // The clause headers come from the class file's own facts and are read **before**
                // anything is written: a clause whose type or parameter cannot be named leaves the
                // whole region quoted rather than writing half a `try` with a header the text cannot
                // state.
                let mut headers = Vec::with_capacity(catches.len());
                for clause in catches {
                    match self.catch_header(clause) {
                        Ok(header) => headers.push(header),
                        Err(reason) => {
                            let at = clause.handler().bci();
                            let bcis = self.region_quote(region, at);
                            return self.fallback(bcis, &reason, at);
                        }
                    }
                }
                let mut body_statements = Vec::new();
                let outer = std::mem::replace(&mut self.settled, lead);
                let walked = self.arm(body, &mut body_statements, &child(path, 0));
                self.settled = outer;
                walked?;
                let mut written = Vec::with_capacity(catches.len());
                for (index, (clause, (ty, name))) in catches.iter().zip(headers).enumerate() {
                    let outer_declared = self.declared.clone();
                    // The header declares the local the handler's own entry store fills: the store is
                    // therefore not a statement of the body, and every later read of that local is a
                    // read of the name this header states.
                    self.declared
                        .insert(LocalVariable::whole(clause.parameter()));
                    self.clause_parameters.insert(clause.handler().bci());
                    let mut handler = Vec::new();
                    self.arm(
                        clause.body(),
                        &mut handler,
                        &child(path, u32::try_from(index + 1).unwrap_or(u32::MAX)),
                    )?;
                    // P3 2.2's third negative: a clause whose body walk wrote **no** statement
                    // while the body's own region still holds instructions the header and the
                    // control flow do not account for is not an empty catch. The handler body is
                    // what this build could not present, so the clause keeps its header and the
                    // body becomes the BCI reference any refused region writes — empty braces
                    // would state a handler that runs nothing, and the dropped statements would
                    // be named nowhere.
                    if handler.is_empty()
                        && let Some((reason, bcis)) =
                            self.unpresented_clause_body(clause.body(), clause.handler().bci())?
                    {
                        let at = bcis
                            .first()
                            .copied()
                            .unwrap_or_else(|| clause.handler().bci());
                        let origin = bcis
                            .iter()
                            .filter(|bci| **bci != at)
                            .fold(OriginSet::new(Origin::direct(at)), |origin, bci| {
                                origin.plus_derived(Origin::derived(*bci))
                            });
                        self.push_into(
                            &mut handler,
                            Stmt::new(StmtKind::Fallback { reason, bcis }, origin),
                        )?;
                    }
                    // A catch parameter is visible through its own clause body only. The builder's
                    // `declared` set is a construction aid, so restore the enclosing lexical scope
                    // before the next handler or the code following this try is built.
                    self.declared = outer_declared;
                    written.push(crate::ast::CatchClause {
                        ty,
                        name,
                        body: handler,
                    });
                }
                // The statement is the protected range's own start; every clause adds the handler
                // entry it was read from, so the table's own rows are the anchors of the text that
                // states them.
                let mut origin = OriginSet::new(Origin::direct(
                    body.blocks().first().map(|block| block.bci()).unwrap_or(0),
                ));
                for clause in catches {
                    origin = origin.plus_derived(Origin::derived(clause.handler().bci()));
                }
                self.push(Stmt::new(
                    StmtKind::Try {
                        resources: Vec::new(),
                        catches: written,
                        body: body_statements,
                        finally_body: None,
                    },
                    origin,
                ))
            }
            Region::ShortCircuitValue { .. } => {
                unreachable!("short-circuit regions use the whole-source fallback path above")
            }
            Region::TwoExitReturn { .. } => {
                unreachable!("two-exit regions use the whole-source fallback path above")
            }
            Region::Fallback { blocks, reason } => {
                let mut bcis = Vec::new();
                for block in blocks {
                    bcis.extend(self.fallback_instruction_bcis(block)?);
                }
                // P3-R7: a refusal may state instruction starts that no block covers at all — the
                // ones the graph failed to account for. The quote has to name them beside the
                // region's own blocks, or the artifact would refuse a body while dropping exactly
                // the bytes it refused it for, which is the silence the refusal exists to undo.
                for &bci in reason.unaccounted() {
                    poll(self.budget, Some(bci))?;
                    charge(
                        self.budget,
                        CountedBudgetDimension::AnalysisSteps,
                        1,
                        Some(bci),
                    )?;
                    bcis.push(bci);
                }
                bcis.sort_unstable();
                bcis.dedup();
                let at = bcis.first().copied().unwrap_or(0);
                self.fallback(bcis, reason.message(), at)
            }
        }
    }

    /// A refused two-exit node quotes every decoded instruction, including fused blocks' inner
    /// instructions and both shared return leaves. No prefix or test is emitted first.
    fn two_exit_return_quote(&mut self, region: &Region, at: u32) -> Result<Vec<u32>, StopReason> {
        let mut bcis = Vec::new();
        for block in region.blocks() {
            bcis.extend(self.fallback_instruction_bcis(block)?);
        }
        bcis.extend(self.quoted_bcis(at));
        bcis.sort_unstable();
        bcis.dedup();
        Ok(bcis)
    }

    /// The `return` each arm of one `switch` writes, with the BCI of the join whose own instruction
    /// it replaces — or `None` when the switch is not the shape the rule states (P3 2c.25).
    ///
    /// A `switch` **expression** compiles to an ordinary `lookupswitch`: every arm leaves one value
    /// on the operand stack and transfers to one shared block whose single instruction is the
    /// member's own `return`, which the no-match arm usually falls into. The join's entry state
    /// names a stack phi that no instruction produced — [`Self::render_value`] refuses it — so the
    /// text that states the shape is one `return` per arm, written where that arm's value is
    /// produced, and the join's own instruction is left to the skip set instead of walked again.
    ///
    /// All of it or none of it. An empty arm, an arm that leaves zero or several values, a join that
    /// holds anything but exactly that one `return`, a join a block **outside** the arms also
    /// transfers to ([`Self::arms_are_the_only_way_in`]), and an arm whose value this layer cannot
    /// render all answer `None`: the switch is written as it was before this rule, with each arm's
    /// own statements and the join's quoted instruction. Half the shape would be worse than neither
    /// half — a `return` appended to some arms and not others, or a join skipped for a value no arm
    /// was given the statement to return.
    fn switch_join(
        &self,
        join: Option<&CanonicalBlockId>,
        groups: &[SwitchGroup],
    ) -> Option<(u32, Vec<(ValueId, u32)>)> {
        let join = join?;
        let join_bci = self.switch_join_return(join)?;
        // A switch whose every target *is* the join holds no arm of its own, and there is therefore
        // no arm for the value the join returns to be written in.
        if groups.is_empty() {
            return None;
        }
        if groups.iter().any(|group| group.fall_through) {
            return None;
        }
        if !self.arms_are_the_only_way_in(join, groups) {
            return None;
        }
        let values = groups
            .iter()
            .map(|group| self.switch_arm_value(&group.arm))
            .collect::<Option<Vec<(ValueId, u32)>>>()?;
        Some((join_bci, values))
    }

    /// The BCI of the one `return` a join block holds alone, when it holds exactly that.
    ///
    /// The block is eligible only as that whole shape: exactly one instruction, and that instruction
    /// a `return` reading exactly one stack value. Every other join — a block that stores before it
    /// returns, a `void` return, a block this run holds no instruction for — is a statement of its
    /// own, and no arm of the switch can state what it does.
    fn switch_join_return(&self, join: &CanonicalBlockId) -> Option<u32> {
        let names = self.ssa.block(join)?;
        let [instruction] = names.instructions() else {
            return None;
        };
        (matches!(
            self.operations.get(instruction.bci()),
            Some(Operation::Return)
        ) && stack_operands(instruction).len() == 1)
            .then(|| instruction.bci())
    }

    /// Whether the arms of one `switch` are the **only** way into its join.
    ///
    /// Every path into the join hands it the one value its own `return` states, so the join may be
    /// left out of the text only while the arms that were given a `return` are the **only** paths
    /// that reach it. `javac` states two shapes where they are not:
    ///
    /// * a nested `switch` expression — `return switch (n) { case 1 -> switch (a) { … }; … }` — whose
    ///   inner arms' transfer and enclosing arm's own transfer reach **one** block, which the
    ///   enclosing `switch` is the one to decide the text of;
    /// * a `switch` expression whose value is one arm of a `?:`, whose other arm transfers to the
    ///   same block.
    ///
    /// A join this switch's arms do not own is therefore refused here, and the enclosing region walks
    /// it as it did before the rule. Suppressing it instead would drop a value no arm of this switch
    /// states — and drop it silently, because the enclosing structure writes nothing there either.
    fn arms_are_the_only_way_in(&self, join: &CanonicalBlockId, groups: &[SwitchGroup]) -> bool {
        let claimed: BTreeSet<&CanonicalBlockId> =
            groups.iter().flat_map(|group| group.arm.blocks()).collect();
        self.canonical
            .edges()
            .iter()
            .filter(|edge| edge.to() == join)
            .all(|edge| claimed.contains(edge.from()))
    }

    /// The one stack value one arm of a `switch` leaves for the join, with the BCI of the arm's own
    /// last instruction: the point the arm's `return` is evaluated at.
    ///
    /// A value is left behind when an instruction of the arm writes it and **no later** instruction
    /// of the arm reads it, over the arm's blocks in the order the region states them. An arm that
    /// leaves no value — every arm of a `switch` **statement** that returns in its own branch — and
    /// one that leaves several both state no single value for the join, and `None` is how this
    /// answers that.
    ///
    /// The point the value is judged at is the arm's own end and not the join: what a name denotes
    /// is a fact about where the text is written, and the join's entry state is the merge of every
    /// arm rather than the state any one arm left. A load of a local the arm writes again after the
    /// load is refused there rather than spelled with a name that now denotes another value.
    fn switch_arm_value(&self, arm: &Region) -> Option<(ValueId, u32)> {
        let mut left: Vec<ValueId> = Vec::new();
        let mut at: Option<u32> = None;
        for block in arm.blocks() {
            let names = self.ssa.block(block)?;
            for instruction in names.instructions() {
                at = Some(instruction.bci());
                for (slot, value) in instruction.reads() {
                    if matches!(slot, Slot::Stack(_)) {
                        left.retain(|held| held != value);
                    }
                }
                for (slot, value) in instruction.writes() {
                    if matches!(slot, Slot::Stack(_)) {
                        left.push(*value);
                    }
                }
            }
        }
        match left.as_slice() {
            [value] => Some((*value, at?)),
            _ => None,
        }
    }

    /// The header declaration one proved resource becomes.
    ///
    /// The type is the **value's own**: the value stored into the resource's slot is what the frames
    /// type it as, and a value they name as no reference type (or as a bare `Object`) is refused —
    /// the header would have to declare a resource the text cannot name, and `Object` is not a
    /// resource a `try` header can hold.
    fn resource_declaration(
        &mut self,
        resource: &guard::Resource,
    ) -> Result<ResourceDecl, ValueRenderFailure> {
        let (from, to) = resource.init();
        // The initialisation is an instruction range, not a block: the canonical graph fuses
        // straight-line code, and the store that lands the resource is the range's last instruction.
        let Some(store) = self
            .instructions
            .range(from..to)
            .next_back()
            .map(|(_, instruction)| *instruction)
        else {
            return Err(format!(
                "the resource's initialisation at BCI {from} holds no instruction this run decoded"
            )
            .into());
        };
        let Some((_, value)) = store
            .writes()
            .iter()
            .find(|(slot, _)| matches!(slot, Slot::Local(_)))
        else {
            return Err(format!(
                "the initialisation at BCI {} stores no local this run names",
                store.bci()
            )
            .into());
        };
        let value = *value;
        // The type comes from the value the store **wrote** — the slot's own type — and the text
        // from the value it **read**: the store itself is not an expression, and rendering its own
        // write would ask the store to produce one.
        let ty = if self.exact_null_resource_initializer(resource) {
            self.null_resource_type(resource, store.bci())?
        } else {
            match value_type(self.ssa.value(value).ty()) {
                Ok(Some(Type::Reference(name))) if name != "Object" => Type::Reference(name),
                Ok(_) => {
                    return Err(format!(
                        "the resource at BCI {} holds a value the frames name as no reference type, so the header cannot declare it",
                        store.bci()
                    ).into());
                }
                // A name the frames state that cannot be spelled as a Java type: the header has no
                // declaration to write, and the fact that stopped it is the reason.
                Err(name) => {
                    return Err(format!(
                        "the resource at BCI {} holds a value the frames name `{name}`, which is a descriptor this layer cannot spell as a Java type, so the header cannot declare it",
                        store.bci()
                    ).into());
                }
            }
        };
        // The slot of a guarded statement's header is never split (P3 3.4): the header writes the
        // declaration, so the slot it takes stays one variable, and a run that somehow split it
        // states no name here rather than writing one of its two variables' names for both.
        let Some(name) = self.names.whole(resource.slot()).map(RenderedName::text) else {
            return Err(format!(
                "the resource at BCI {} lives in slot {}, which has no name",
                store.bci(),
                resource.slot()
            )
            .into());
        };
        let name = name.to_owned();
        let at = store.bci();
        let Some((_, stored)) = stack_operands(store).first().copied() else {
            return Err(format!(
                "the resource's initialisation at BCI {at} stores no value this run names"
            )
            .into());
        };
        let value = self.render_value(stored, at, 0)?;
        // The header declares the slot: a body that wrote it again would otherwise declare it a
        // second time, and the close the compiler writes reads the name the header gives it.
        self.declared.insert(LocalVariable::whole(resource.slot()));
        Ok(ResourceDecl { ty, name, value })
    }

    /// Whether this resource's own proved header initializer is exactly `aconst_null; astore`.
    fn exact_null_resource_initializer(&self, resource: &guard::Resource) -> bool {
        let (from, to) = resource.init();
        let instructions: Vec<u32> = self
            .instructions
            .range(from..to)
            .map(|(bci, _)| *bci)
            .collect();
        let [push, store] = instructions.as_slice() else {
            return false;
        };
        matches!(
            self.operations.get(*push),
            Some(Operation::Push(ConstantValue::Null))
        ) && matches!(
            self.operations.get(*store),
            Some(Operation::Store { slot }) if *slot == resource.slot()
        )
    }

    /// The type a direct null resource may use when this run proves all three local class facts:
    /// this class directly implements `AutoCloseable`, and both closes dispatch to its own `close`.
    fn null_resource_type(
        &self,
        resource: &guard::Resource,
        init_bci: u32,
    ) -> Result<Type, String> {
        let Some(declaring_class) = self.declaring_class else {
            return Err(format!(
                "the null resource at BCI {init_bci} has no declaring-class fact from which this header can name a type"
            ));
        };
        let Some(simple_name) = current_class_simple_name(declaring_class) else {
            return Err(format!(
                "the null resource at BCI {init_bci} belongs to current class `{declaring_class}`, whose name this layer cannot safely spell in a resource header"
            ));
        };
        if !self
            .direct_interfaces
            .iter()
            .any(|interface| interface.raw().0.as_slice() == b"java/lang/AutoCloseable")
        {
            return Err(format!(
                "the null resource at BCI {init_bci} cannot be typed as the current class because its class header does not directly declare `java/lang/AutoCloseable`"
            ));
        }
        for close_bci in [resource.close_bci(), resource.exceptional_close_bci()] {
            let owned_here = matches!(
                self.operations.get(close_bci),
                Some(Operation::Invoke(target))
                    if target.name() == "close"
                        && target.descriptor() == "()V"
                        && target.owner() == declaring_class
            );
            if !owned_here {
                return Err(format!(
                    "the null resource at BCI {init_bci} cannot be typed as the current class because its close at BCI {close_bci} is not owned by `{declaring_class}`"
                ));
            }
        }
        Ok(Type::Reference(simple_name))
    }

    /// The type and the parameter name one `catch` clause's header states.
    ///
    /// The type is the **rows' own** `catch_type` list: each entry is the pool's `CONSTANT_Class`
    /// entry the compiler wrote into the exception table, and the clause names those classes and no
    /// others — widening one to a superclass would claim the handler catches exceptions the table
    /// says it does not, and narrowing it would catch fewer. Several entries are the multi-catch the
    /// table states, spelled `A | B` in table order. Each is spelled from the pool's own internal
    /// form, so the name in the text is the name the class file states.
    ///
    /// The parameter is the local the handler's **entry store** fills, named by the same table every
    /// other local is named by: a body compiled without debug metadata states no name, and the slot's
    /// own ordinal (`localN`, `argN`) is what the text writes then. No name is invented for it.
    fn catch_header(&self, clause: &CatchClause) -> Result<(String, String), String> {
        let mut types: Vec<String> = Vec::with_capacity(clause.type_indices().len());
        for index in clause.type_indices() {
            let internal = cp_class_name(self.pool, *index).map_err(|_| {
                format!(
                    "the catch type at constant-pool index {index} is not a `CONSTANT_Class` entry, so the clause names no class"
                )
            })?;
            let internal = String::from_utf8_lossy(&internal.0).into_owned();
            let ty = spell_reference(&internal).ok_or_else(|| {
                format!(
                    "the catch type `{internal}` is not a class name this layer can spell as a Java type, so the clause cannot be named"
                )
            })?;
            types.push(ty);
        }
        // The slot of a clause's parameter is never split (P3 3.4): the header writes the
        // declaration, so the slot it takes stays one variable, and a run that somehow split it
        // states no name here rather than writing one of its two variables' names for both.
        let Some(name) = self.names.whole(clause.parameter()).map(RenderedName::text) else {
            return Err(format!(
                "the catch parameter at BCI {} lives in slot {}, which has no name",
                clause.handler().bci(),
                clause.parameter()
            ));
        };
        Ok((types.join(" | "), name.to_owned()))
    }

    /// Writes the statements of one instruction range, in order.
    ///
    /// This is how a guarded statement writes the region it guards: the body's instructions and the
    /// block that holds them are not the same thing, because the canonical graph fuses the resource's
    /// initialisation, the body and the first close of the normal path into one node.
    fn range(&mut self, span: (u32, u32)) -> Result<(), StopReason> {
        let instructions: Vec<SsaInstruction> = self
            .instructions
            .range(span.0..span.1)
            .map(|(_, instruction)| (*instruction).clone())
            .collect();
        for instruction in &instructions {
            self.instruction(instruction)?;
        }
        Ok(())
    }

    /// The guarded body's statements, as the list the statement node holds.
    fn body_range(&mut self, span: (u32, u32)) -> Result<Vec<Stmt>, StopReason> {
        let mark = self.stmts.len();
        self.range(span)?;
        Ok(self.stmts.split_off(mark))
    }

    /// The lock expression one verified `synchronized` header reads.
    ///
    /// The value is the `monitorenter`'s own operand — the object the bytecode entered — read
    /// through the duplications javac's idiom puts in front of it (`dup; astore slot; monitorenter`):
    /// a `dup` is not an expression, and the value it duplicated is the one the header writes. It is
    /// rendered where it is produced, so the header runs exactly what the entry block ran.
    fn lock_expr(&mut self, enter_bci: u32) -> Result<Expr, ValueRenderFailure> {
        let Some(instruction) = self.instructions.get(&enter_bci).copied() else {
            return Err(format!(
                "the monitor at BCI {enter_bci} has no record in this run's names"
            )
            .into());
        };
        let Some((_, value)) = stack_operands(instruction).first().copied() else {
            return Err(format!("the monitor at BCI {enter_bci} reads no value to lock").into());
        };
        let mut value = value;
        for _ in 0..8 {
            let Definition::Instruction { bci, .. } = self.ssa.value(value).def() else {
                break;
            };
            if !matches!(self.operations.get(*bci), Some(Operation::Duplicate)) {
                break;
            }
            let Some(producer) = self.instructions.get(bci).copied() else {
                break;
            };
            let Some((_, duplicated)) = stack_operands(producer).first().copied() else {
                break;
            };
            value = duplicated;
        }
        self.render_value(value, enter_bci, 0)
    }

    /// The `return` statement a `synchronized` body ends in where the normal path returns the value
    /// the body left on the stack.
    ///
    /// The statement is the method's own `return`, written **inside** the braces: the value it names
    /// is the one the body's last read produced, so the read is written where it is consumed and
    /// nowhere else ([`Self::return_expr`] renders it, exactly as a `return` outside any statement
    /// would). It is never pushed as a statement of its own — the region's statements are the ones
    /// between the braces, and a `return` written after the statement would run after the exit.
    fn guarded_return(&mut self, return_bci: u32) -> Result<Stmt, ValueRenderFailure> {
        let Some(instruction) = self.instructions.get(&return_bci).copied() else {
            return Err(format!(
                "the return at BCI {return_bci} has no record in this run's names"
            )
            .into());
        };
        let Some((_, value)) = stack_operands(instruction).last().copied() else {
            return Err(format!("the return at BCI {return_bci} reads no value to return").into());
        };
        let value = self.return_expr(value, return_bci, return_bci)?;
        Ok(Stmt::new(
            StmtKind::Return { value: Some(value) },
            OriginSet::new(Origin::direct(return_bci)),
        ))
    }

    /// Writes what a test block does before the test itself, in the order it does it.
    ///
    /// A block whose terminal instruction is a branch can still hold statements before it — an
    /// assignment made while the condition is evaluated, a call whose result the test reads. For an
    /// `if` or a `switch` those run exactly once, before the test, so writing them there keeps their
    /// count and their order. (A loop's test block is the one place this cannot be done: its
    /// statements would run once per iteration, and a `while (…)` has nowhere to write them that
    /// does — which is why [`crate::region`] refuses such a loop instead of hoisting them.)
    fn test_effects(&mut self, block: &CanonicalBlockId, test_bci: u32) -> Result<(), StopReason> {
        let Some(names) = self.ssa.block(block) else {
            return Ok(());
        };
        let instructions: Vec<SsaInstruction> = names
            .instructions()
            .iter()
            .filter(|instruction| instruction.bci() != test_bci)
            .cloned()
            .collect();
        for instruction in &instructions {
            self.instruction(instruction)?;
        }
        Ok(())
    }

    /// The first iterable slice keeps the original cast and body binding. Only exact Java 8
    /// Iterable platform owners whose presented source type agrees can replace the existing while.
    fn iterable_for_each_candidate(
        &mut self,
        test_bci: u32,
        kind: &StmtKind,
        regions: &[Region],
    ) -> Result<Option<(StmtKind, OriginSet, Vec<usize>, String)>, StopReason> {
        let StmtKind::While { cond, body, .. } = kind else {
            return Ok(None);
        };
        if self.profile.java_release < 8
            || !matches!(&cond.kind, ExprKind::Call { name, args, .. }
                if name == "hasNext" && args.is_empty())
            || !matches!(self.operations.get(cond.origin.primary().bci()),
                Some(Operation::Invoke(target))
                    if target.kind() == InvokeKind::Interface
                        && target.is_interface_reference()
                        && target.owner() == "java/util/Iterator"
                        && target.name() == "hasNext"
                        && target.descriptor() == "()Z")
            || !body
                .first()
                .is_some_and(|stmt| matches!(stmt.kind, StmtKind::Declare { value: Some(_), .. }))
        {
            return Ok(None);
        }
        poll(self.budget, Some(test_bci))?;
        let instructions = self.ssa.effects().instructions().len();
        let phis = self.ssa.phis().len();
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(
                self.ssa
                    .blocks()
                    .iter()
                    .map(|block| block.instructions().len() + 1)
                    .sum::<usize>()
                    .saturating_add(
                        instructions
                            .saturating_add(phis)
                            .saturating_mul(phis.saturating_add(1))
                            .saturating_mul(4),
                    )
                    .saturating_add(regions.len())
                    .saturating_add(self.stmts.len()),
            )
            .unwrap_or(u64::MAX),
            Some(test_bci),
        )?;

        let candidate = (|| {
            let StmtKind::While { label, cond, body } = kind else {
                return None;
            };
            let ExprKind::Call {
                receiver: Some(test_receiver),
                name: test_name,
                args: test_args,
            } = &cond.kind
            else {
                return None;
            };
            let ExprKind::Local(iterator_name) = &test_receiver.kind else {
                return None;
            };
            if test_name != "hasNext" || !test_args.is_empty() {
                return None;
            }
            let [binding, ..] = body.as_slice() else {
                return None;
            };
            let StmtKind::Declare {
                name: binding_name,
                value: Some(binding_value),
                ..
            } = &binding.kind
            else {
                return None;
            };
            let (next, inline_cast_bci) = match &binding_value.kind {
                ExprKind::Call { .. } => (binding_value, None),
                ExprKind::Cast { value, .. } if matches!(value.kind, ExprKind::Call { .. }) => {
                    (value.as_ref(), Some(binding_value.origin.primary().bci()))
                }
                _ => return None,
            };
            let ExprKind::Call {
                receiver: Some(next_receiver),
                name: next_name,
                args: next_args,
            } = &next.kind
            else {
                return None;
            };
            if next_name != "next"
                || !next_args.is_empty()
                || !matches!(&next_receiver.kind, ExprKind::Local(name) if name == iterator_name)
            {
                return None;
            }
            let (init_index, init_stmt, iterable) =
                self.stmts
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(index, stmt)| {
                        let (name, value) = match &stmt.kind {
                            StmtKind::Assign { name, value }
                            | StmtKind::Declare {
                                name,
                                value: Some(value),
                                ..
                            } => (name, value),
                            _ => return None,
                        };
                        let ExprKind::Call {
                            receiver: Some(receiver),
                            name: call_name,
                            args,
                        } = &value.kind
                        else {
                            return None;
                        };
                        (name == iterator_name && call_name == "iterator" && args.is_empty())
                            .then_some((index, stmt, receiver.as_ref()))
                    })?;
            let ExprKind::Local(iterable_name) = &iterable.kind else {
                return None;
            };
            let Some(Type::Reference(iterable_type)) = &iterable.presented else {
                return None;
            };
            let declaration_index = self.stmts[..init_index].iter().rposition(|stmt| {
                matches!(&stmt.kind, StmtKind::Declare { ty: Type::Reference(ty), name, value: None }
                    if name == iterator_name && (ty == "java.util.Iterator" || ty == "Iterator"))
            })?;
            if self.stmts[init_index + 1..]
                .iter()
                .any(|stmt| match &stmt.kind {
                    StmtKind::Declare { value: None, .. } => false,
                    StmtKind::Declare {
                        value: Some(value), ..
                    }
                    | StmtKind::Assign { value, .. } => !matches!(
                        value.kind,
                        ExprKind::Integer(_) | ExprKind::Long(_) | ExprKind::Boolean(_)
                    ),
                    _ => true,
                }) || self.stmts[init_index + 1..].iter().any(|stmt| {
                matches!(&stmt.kind, StmtKind::Declare { name, .. } | StmtKind::Assign { name, .. }
                    if name == iterable_name || name == iterator_name)
            }) {
                return None;
            }
            let init_bci = match &init_stmt.kind {
                StmtKind::Assign { value, .. }
                | StmtKind::Declare {
                    value: Some(value), ..
                } => value.origin.primary().bci(),
                _ => return None,
            };
            let init_store_bci = init_stmt.origin.primary().bci();
            let has_next_bci = cond.origin.primary().bci();
            let next_bci = next.origin.primary().bci();
            let next_store_bci = binding.origin.primary().bci();
            let test_load_bci = test_receiver.origin.primary().bci();
            let next_load_bci = next_receiver.origin.primary().bci();
            let exact_call = |bci, owner, name, descriptor| {
                matches!(self.operations.get(bci), Some(Operation::Invoke(target))
                    if target.kind() == InvokeKind::Interface
                        && target.is_interface_reference()
                        && target.owner() == owner
                        && target.name() == name
                        && target.descriptor() == descriptor)
            };
            let iterable_owner = match self.operations.get(init_bci) {
                Some(Operation::Invoke(target))
                    if target.kind() == InvokeKind::Interface
                        && target.is_interface_reference()
                        && target.name() == "iterator"
                        && target.descriptor() == "()Ljava/util/Iterator;" =>
                {
                    target.owner()
                }
                _ => return None,
            };
            let source_matches_owner = match iterable_owner {
                "java/lang/Iterable" => {
                    iterable_type == "java.lang.Iterable" || iterable_type == "Iterable"
                }
                "java/util/List" => iterable_type == "java.util.List",
                "java/util/Collection" => iterable_type == "java.util.Collection",
                _ => false,
            };
            if !source_matches_owner
                || !exact_call(
                    init_bci,
                    iterable_owner,
                    "iterator",
                    "()Ljava/util/Iterator;",
                )
                || !exact_call(has_next_bci, "java/util/Iterator", "hasNext", "()Z")
                || !exact_call(
                    next_bci,
                    "java/util/Iterator",
                    "next",
                    "()Ljava/lang/Object;",
                )
            {
                return None;
            }
            let instruction = |bci| self.instructions.get(&bci).copied();
            let stack_result = |instruction: &SsaInstruction| {
                instruction
                    .writes()
                    .iter()
                    .find_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value))
            };
            let Some(Operation::Store {
                slot: iterator_slot,
            }) = self.operations.get(init_store_bci)
            else {
                return None;
            };
            if !matches!(self.operations.get(test_load_bci), Some(Operation::Load { slot }) if slot == iterator_slot)
                || !matches!(self.operations.get(next_load_bci), Some(Operation::Load { slot }) if slot == iterator_slot)
                || !matches!(
                    self.operations.get(next_store_bci),
                    Some(Operation::Store { .. })
                )
            {
                return None;
            }
            let iterator_value =
                instruction(init_store_bci)?
                    .writes()
                    .iter()
                    .find_map(|(slot, value)| {
                        (*slot == Slot::Local(*iterator_slot)).then_some(*value)
                    })?;
            let init_result = stack_result(instruction(init_bci)?)?;
            if self.ssa.value(init_result).uses().len() != 1
                || self.ssa.value(init_result).uses()[0].bci() != Some(init_store_bci)
            {
                return None;
            }
            let header = self.block_of.get(&test_bci)?;
            if self
                .ssa
                .block(header)?
                .instructions()
                .iter()
                .any(|instruction| {
                    ![test_load_bci, has_next_bci, test_bci].contains(&instruction.bci())
                })
            {
                return None;
            }
            let next_block = self.block_of.get(&next_bci)?;
            if self
                .ssa
                .block(next_block)?
                .instructions()
                .iter()
                .any(|instruction| {
                    instruction.bci() < next_bci && instruction.bci() != next_load_bci
                })
            {
                return None;
            }
            let phi =
                self.ssa.phis().iter().find(|phi| {
                    phi.block() == header && phi.slot() == Slot::Local(*iterator_slot)
                })?;
            let resolves = |mut value| {
                for _ in 0..=self.ssa.phis().len() {
                    if value == iterator_value {
                        return true;
                    }
                    value = match self.ssa.value(value).replaced_by() {
                        Some(next) => next,
                        None => return false,
                    };
                }
                false
            };
            if phi.inputs().len() < 2
                || phi
                    .inputs()
                    .iter()
                    .filter(|input| **input == PhiInput::Value(iterator_value))
                    .count()
                    != 1
                || !phi.inputs().contains(&PhiInput::Itself)
                || !phi.inputs().iter().all(|input| {
                    *input == PhiInput::Value(iterator_value) || *input == PhiInput::Itself
                })
                || !resolves(phi.value())
                || self.ssa.blocks().iter().any(|block| {
                    block.instructions().iter().any(|instruction| {
                        instruction.bci() != init_store_bci
                            && instruction
                                .writes()
                                .iter()
                                .any(|(slot, _)| *slot == Slot::Local(*iterator_slot))
                    })
                })
            {
                return None;
            }
            let local_use = |bci| {
                instruction(bci)?.reads().iter().find_map(|(slot, value)| {
                    (*slot == Slot::Local(*iterator_slot)).then_some(*value)
                })
            };
            let test_operands = stack_operands(instruction(has_next_bci)?);
            let [(_, test_receiver_value)] = test_operands.as_slice() else {
                return None;
            };
            if !resolves(local_use(test_load_bci)?)
                || !resolves(local_use(next_load_bci)?)
                || *test_receiver_value != stack_result(instruction(test_load_bci)?)?
            {
                return None;
            }
            let next_operands = stack_operands(instruction(next_bci)?);
            let [(_, next_receiver_value)] = next_operands.as_slice() else {
                return None;
            };
            if *next_receiver_value != stack_result(instruction(next_load_bci)?)? {
                return None;
            }
            for value in [iterator_value, phi.value()] {
                if self.ssa.value(value).uses().iter().any(|use_| {
                    ![Some(test_load_bci), Some(next_load_bci)].contains(&use_.bci())
                        && !(use_.bci().is_none()
                            && use_.block() == header
                            && phi.inputs().iter().all(|input| {
                                *input == PhiInput::Value(iterator_value)
                                    || *input == PhiInput::Itself
                            }))
                }) {
                    return None;
                }
            }
            let next_result = stack_result(instruction(next_bci)?)?;
            let next_consumer = inline_cast_bci.unwrap_or(next_store_bci);
            if self.ssa.value(next_result).uses().len() != 1
                || self.ssa.value(next_result).uses()[0].bci() != Some(next_consumer)
            {
                return None;
            }
            if let Some(cast_bci) = inline_cast_bci {
                if !matches!(
                    self.operations.get(cast_bci),
                    Some(Operation::CheckCast { .. })
                ) || self
                    .ssa
                    .value(stack_result(instruction(cast_bci)?)?)
                    .uses()
                    .len()
                    != 1
                    || self.ssa.value(stack_result(instruction(cast_bci)?)?).uses()[0].bci()
                        != Some(next_store_bci)
                {
                    return None;
                }
            } else {
                if !matches!(&binding_value.presented, Some(Type::Reference(ty)) if ty == "java.lang.Object" || ty == "Object")
                {
                    return None;
                }
                let StmtKind::Declare {
                    value: Some(cast), ..
                } = &body.get(1)?.kind
                else {
                    return None;
                };
                let ExprKind::Cast {
                    value: cast_input, ..
                } = &cast.kind
                else {
                    return None;
                };
                if !matches!(&cast_input.kind, ExprKind::Local(name) if name == binding_name) {
                    return None;
                }
                let Some(Operation::Store { slot: element_slot }) =
                    self.operations.get(next_store_bci)
                else {
                    return None;
                };
                let element_value =
                    instruction(next_store_bci)?
                        .writes()
                        .iter()
                        .find_map(|(slot, value)| {
                            (*slot == Slot::Local(*element_slot)).then_some(*value)
                        })?;
                let cast_load_bci = cast_input.origin.primary().bci();
                if self.ssa.value(element_value).uses().len() != 1
                    || self.ssa.value(element_value).uses()[0].bci() != Some(cast_load_bci)
                {
                    return None;
                }
            }
            let effect = |bci| {
                self.ssa.effects().instructions().iter().find(|effect| {
                    effect.bci() == bci && Some(effect.block()) == self.block_of.get(&bci)
                })
            };
            let handlers = effect(has_next_bci)?.handlers();
            if effect(init_bci)?.handlers() != handlers
                || effect(next_bci)?.handlers() != handlers
                || effect(test_bci)?.handlers() != handlers
                || inline_cast_bci
                    .is_some_and(|bci| effect(bci).map(|e| e.handlers()) != Some(handlers))
            {
                return None;
            }
            let mut bcis = Vec::new();
            let mut names = Vec::new();
            stated_by_statement(init_stmt, &mut names, &mut bcis);
            stated_by_expression(cond, &mut names, &mut bcis);
            stated_by_expression(next, &mut names, &mut bcis);
            bcis.push(test_bci);
            let origin = bcis
                .into_iter()
                .fold(OriginSet::new(Origin::direct(test_bci)), |origin, bci| {
                    origin.plus_derived(Origin::derived(bci))
                });
            Some((
                label.clone(),
                iterable.clone(),
                body.to_vec(),
                next_bci,
                origin,
                vec![declaration_index, init_index],
            ))
        })();
        let Some((label, iterable, mut body, next_bci, origin, remove)) = candidate else {
            return Ok(None);
        };
        let mut base = format!("iteratorElement{next_bci}");
        let name = loop {
            let candidate = self.names.free_name_with(&base, || {
                poll(self.budget, Some(next_bci))?;
                charge(
                    self.budget,
                    CountedBudgetDimension::IrItems,
                    1,
                    Some(next_bci),
                )
            })?;
            if !self.synthetic_names.contains(&candidate)
                && !self.lambda_params.contains(&candidate)
            {
                break candidate;
            }
            base = format!("{candidate}_");
        };
        let Some(Stmt {
            kind: StmtKind::Declare {
                value: Some(value), ..
            },
            ..
        }) = body.first_mut()
        else {
            return Ok(None);
        };
        let next = match &mut value.kind {
            ExprKind::Call { .. } => value,
            ExprKind::Cast { value, .. } => value.as_mut(),
            _ => return Ok(None),
        };
        if next.origin.primary().bci() != next_bci {
            return Ok(None);
        }
        *next = Expr::new(ExprKind::Local(name.clone()), next.origin.clone())
            .presenting(Type::Reference("java.lang.Object".to_owned()));
        Ok(Some((
            StmtKind::ForEach {
                label,
                ty: Type::Reference("java.lang.Object".to_owned()),
                name: name.clone(),
                iterable,
                body,
            },
            origin,
            remove,
            name,
        )))
    }

    /// Project a counted array loop only when every deleted expression and local has one proven
    /// consumer. The array capture remains where it was evaluated, including a Supplier call.
    fn array_for_each_candidate(
        &mut self,
        proof: &ForHeader,
        test_bci: u32,
        path: &[u32],
        kind: &StmtKind,
        regions: &[Region],
    ) -> Result<Option<(StmtKind, OriginSet, bool, Vec<usize>)>, StopReason> {
        if self.profile.java_release < 8
            || !matches!(kind, StmtKind::For { .. })
            || !self.stmts.last().is_some_and(|stmt| {
                matches!(
                    stmt.kind,
                    StmtKind::Declare {
                        value: Some(Expr {
                            kind: ExprKind::ArrayLength { .. },
                            ..
                        }),
                        ..
                    } | StmtKind::Assign {
                        value: Expr {
                            kind: ExprKind::ArrayLength { .. },
                            ..
                        },
                        ..
                    }
                )
            })
        {
            return Ok(None);
        }
        poll(self.budget, Some(test_bci))?;
        let instructions = self.ssa.effects().instructions().len();
        let phis = self.ssa.phis().len();
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(
                self.ssa
                    .blocks()
                    .iter()
                    .map(|block| block.instructions().len() + 1)
                    .sum::<usize>()
                    .saturating_add(
                        instructions
                            .saturating_add(phis)
                            .saturating_mul(phis.saturating_add(1))
                            .saturating_mul(4),
                    )
                    .saturating_add(regions.len()),
            )
            .unwrap_or(u64::MAX),
            Some(test_bci),
        )?;

        let wrapped_read_bci = match kind {
            StmtKind::For { body, .. } => body.first().and_then(|stmt| match &stmt.kind {
                StmtKind::Declare {
                    value: Some(value), ..
                } if !matches!(value.kind, ExprKind::Index { .. }) => {
                    leading_wrapped_array_read(value)
                }
                StmtKind::Assign { value, .. } if !matches!(value.kind, ExprKind::Index { .. }) => {
                    leading_wrapped_array_read(value)
                }
                _ => None,
            }),
            _ => None,
        }
        .map(|read| read.origin.primary().bci());
        let wrapped_name = if let Some(read_bci) = wrapped_read_bci {
            let mut base = format!("arrayElement{read_bci}");
            Some(loop {
                let candidate = self.names.free_name_with(&base, || {
                    poll(self.budget, Some(read_bci))?;
                    charge(
                        self.budget,
                        CountedBudgetDimension::IrItems,
                        1,
                        Some(read_bci),
                    )
                })?;
                if !self.synthetic_names.contains(&candidate)
                    && !self.lambda_params.contains(&candidate)
                {
                    break candidate;
                }
                base = format!("{candidate}_");
            })
        } else {
            None
        };

        let candidate = (|| {
            let StmtKind::For {
                label,
                init,
                cond,
                update,
                body,
            } = kind
            else {
                return None;
            };
            let length_stmt = self.stmts.last()?;
            let (length_name, length_expr) = match &length_stmt.kind {
                StmtKind::Declare {
                    name,
                    value: Some(value),
                    ..
                }
                | StmtKind::Assign { name, value } => (name, value),
                _ => return None,
            };
            let ExprKind::ArrayLength { array } = &length_expr.kind else {
                return None;
            };
            let ExprKind::Local(array_name) = &array.kind else {
                return None;
            };
            let Type::Reference(array_type) = array.presented.as_ref()? else {
                return None;
            };
            if !array_type.ends_with("[]") {
                return None;
            }
            let (StmtKind::Declare {
                name: index_name,
                value: Some(initial),
                ..
            }
            | StmtKind::Assign {
                name: index_name,
                value: initial,
            }) = &init.kind
            else {
                return None;
            };
            if !matches!(initial.kind, ExprKind::Integer(0)) {
                return None;
            }
            let ExprKind::Binary {
                op: BinaryOp::Less,
                left,
                right,
            } = &cond.kind
            else {
                return None;
            };
            if !matches!(&left.kind, ExprKind::Local(name) if name == index_name)
                || !matches!(&right.kind, ExprKind::Local(name) if name == length_name)
            {
                return None;
            }
            let StmtKind::Assign {
                name: update_name,
                value: update_expr,
            } = &update.kind
            else {
                return None;
            };
            let ExprKind::Binary {
                op: BinaryOp::Add,
                left: update_base,
                right: step,
            } = &update_expr.kind
            else {
                return None;
            };
            if update_name != index_name
                || !matches!(&update_base.kind, ExprKind::Local(name) if name == index_name)
                || !matches!(step.kind, ExprKind::Integer(1))
            {
                return None;
            }
            let [element, remainder @ ..] = body.as_slice() else {
                return None;
            };
            let (value, direct_binding) = match &element.kind {
                StmtKind::Declare {
                    value: Some(value), ..
                } if matches!(value.kind, ExprKind::Index { .. }) => (value, true),
                StmtKind::Assign { value, .. } if matches!(value.kind, ExprKind::Index { .. }) => {
                    return None;
                }
                StmtKind::Declare {
                    value: Some(value), ..
                }
                | StmtKind::Assign { value, .. } => (leading_wrapped_array_read(value)?, false),
                _ => return None,
            };
            let ExprKind::Index {
                array: read_array,
                index: read_index,
            } = &value.kind
            else {
                return None;
            };
            if !matches!(&read_array.kind, ExprKind::Local(local) if local == array_name)
                || !matches!(&read_index.kind, ExprKind::Local(local) if local == index_name)
            {
                return None;
            }

            let instruction = |bci| self.instructions.get(&bci).copied();
            let stack_result = |instruction: &SsaInstruction| {
                instruction
                    .writes()
                    .iter()
                    .find_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value))
            };
            let local_read = |bci, slot| {
                instruction(bci)?
                    .reads()
                    .iter()
                    .find_map(|(at, value)| (*at == Slot::Local(slot)).then_some(*value))
            };
            let load_source = |bci| {
                let inst = instruction(bci)?;
                let Some(Operation::Load { slot }) = self.operations.get(bci) else {
                    return None;
                };
                local_read(inst.bci(), *slot)
            };
            let length_bci = length_expr.origin.primary().bci();
            let length_store_bci = length_stmt.origin.primary().bci();
            let length_read_bci = right.origin.primary().bci();
            let test_index_bci = left.origin.primary().bci();
            let element_bci = value.origin.primary().bci();
            let element_store_bci = direct_binding.then(|| element.origin.primary().bci());
            let element_array_bci = read_array.origin.primary().bci();
            let element_index_bci = read_index.origin.primary().bci();
            if !matches!(
                self.operations.get(length_bci),
                Some(Operation::ArrayLength)
            ) || !matches!(
                self.operations.get(length_store_bci),
                Some(Operation::Store { .. })
            ) || !matches!(
                self.operations.get(element_bci),
                Some(Operation::ArrayLoad | Operation::ArrayElementLoad { .. })
            ) || element_store_bci.is_some_and(|bci| {
                !matches!(self.operations.get(bci), Some(Operation::Store { .. }))
            }) {
                return None;
            }
            let length_operands = stack_operands(instruction(length_bci)?);
            let [(_, length_array_value)] = length_operands.as_slice() else {
                return None;
            };
            let element_operands = stack_operands(instruction(element_bci)?);
            let [(_, element_array_value), (_, element_index_value)] = element_operands.as_slice()
            else {
                return None;
            };
            let Definition::Instruction {
                bci: length_array_bci,
                ..
            } = self.ssa.value(*length_array_value).def()
            else {
                return None;
            };
            let proved_element =
                array_element(self.ssa, self.operations, *element_array_value, None)?;
            let same_element = match &element.kind {
                StmtKind::Declare { ty, .. } if direct_binding => {
                    proved_element == *ty
                        || matches!(
                            (&proved_element, ty),
                            (Type::Reference(actual), Type::Reference(spelled))
                                if actual == "java.lang.Object" && spelled == "Object"
                        )
                }
                _ => true,
            };
            if *length_array_bci != array.origin.primary().bci()
                || load_source(*length_array_bci)? != load_source(element_array_bci)?
                || *element_array_value != stack_result(instruction(element_array_bci)?)?
                || !same_element
            {
                return None;
            }
            let length_store = instruction(length_store_bci)?;
            let Some(Operation::Store { slot: length_slot }) =
                self.operations.get(length_store_bci)
            else {
                return None;
            };
            let length_local = length_store
                .writes()
                .iter()
                .find_map(|(slot, value)| (*slot == Slot::Local(*length_slot)).then_some(*value))?;
            let length_load = instruction(length_read_bci)?;
            let mut loop_blocks: BTreeSet<_> =
                regions.iter().flat_map(Region::blocks).cloned().collect();
            loop_blocks.insert(self.block_of.get(&test_bci)?.clone());
            let length_phi = self.ssa.phis().iter().find(|phi| {
                Some(phi.block()) == self.block_of.get(&test_bci)
                    && phi.slot() == Slot::Local(*length_slot)
            });
            let resolves_to_length = |mut value| {
                for _ in 0..=self.ssa.phis().len() {
                    if value == length_local {
                        return true;
                    }
                    let Some(replacement) = self.ssa.value(value).replaced_by() else {
                        return false;
                    };
                    value = replacement;
                }
                false
            };
            let phi_passes_length = length_phi.is_some_and(|phi| {
                phi.inputs().len() == 2
                    && phi.inputs().contains(&PhiInput::Value(length_local))
                    && phi.inputs().contains(&PhiInput::Itself)
                    && resolves_to_length(phi.value())
            });
            if local_read(length_read_bci, *length_slot)? != length_local
                || !phi_passes_length
                || self.ssa.blocks().iter().any(|block| {
                    loop_blocks.contains(block.block())
                        && block.instructions().iter().any(|instruction| {
                            instruction
                                .writes()
                                .iter()
                                .any(|(slot, _)| *slot == Slot::Local(*length_slot))
                        })
                })
                || self
                    .ssa
                    .value(length_local)
                    .uses()
                    .iter()
                    .filter(|use_| use_.bci() == Some(length_read_bci))
                    .count()
                    != 1
                || self.ssa.value(length_local).uses().iter().any(|use_| {
                    use_.bci() != Some(length_read_bci)
                        && !(use_.bci().is_none()
                            && self.ssa.phis().iter().any(|phi| {
                                phi.block() == use_.block()
                                    && phi.slot() == Slot::Local(*length_slot)
                                    && resolves_to_length(phi.value())
                                    && phi.inputs().iter().all(|input| {
                                        *input == PhiInput::Value(length_local)
                                            || *input == PhiInput::Itself
                                    })
                            }))
                })
                || self.ssa.value(stack_result(length_load)?).uses().len() != 1
                || self.ssa.value(stack_result(length_load)?).uses()[0].bci() != Some(test_bci)
            {
                return None;
            }
            let header = self.block_of.get(&test_bci)?;
            let phi = self
                .ssa
                .phis()
                .iter()
                .find(|phi| phi.block() == header && phi.slot() == Slot::Local(proof.slot))?;
            if local_read(test_index_bci, proof.slot)? != phi.value()
                || local_read(element_index_bci, proof.slot)? != phi.value()
                || *element_index_value != stack_result(instruction(element_index_bci)?)?
            {
                return None;
            }
            let update_block = self.ssa.block(&proof.update_block)?;
            let update_read_bci = match self.operations.get(proof.update_bci) {
                Some(Operation::Increment { slot, amount: 1 }) if *slot == proof.slot => {
                    proof.update_bci
                }
                Some(Operation::Store { slot }) if *slot == proof.slot => {
                    update_block.instructions().iter().rev().nth(4)?.bci()
                }
                _ => return None,
            };
            let resolves_to_induction = |mut value| {
                for _ in 0..=self.ssa.phis().len() {
                    if value == phi.value() {
                        return true;
                    }
                    let Some(replacement) = self.ssa.value(value).replaced_by() else {
                        return false;
                    };
                    value = replacement;
                }
                false
            };
            if !resolves_to_induction(local_read(update_read_bci, proof.slot)?)
                || self.ssa.value(phi.value()).uses().iter().any(|use_| {
                    ![test_index_bci, element_index_bci, update_read_bci]
                        .contains(&use_.bci().unwrap_or(u32::MAX))
                        && !(use_.bci().is_none()
                            && self.ssa.phis().iter().any(|join| {
                                join.block() == use_.block()
                                    && join.slot() == Slot::Local(proof.slot)
                                    && resolves_to_induction(join.value())
                                    && join.inputs().iter().all(|input| {
                                        *input == PhiInput::Value(phi.value())
                                            || *input == PhiInput::Itself
                                    })
                            }))
                })
            {
                return None;
            }
            let body_blocks: BTreeSet<_> =
                regions.iter().flat_map(Region::blocks).cloned().collect();
            if !direct_binding
                && self
                    .ssa
                    .blocks()
                    .iter()
                    .filter(|block| loop_blocks.contains(block.block()))
                    .flat_map(|block| block.instructions())
                    .filter(|instruction| {
                        matches!(
                            self.operations.get(instruction.bci()),
                            Some(Operation::ArrayLoad | Operation::ArrayElementLoad { .. })
                        )
                    })
                    .count()
                    != 1
            {
                return None;
            }
            if let Some(element_store_bci) = element_store_bci {
                let element_store = instruction(element_store_bci)?;
                let Some(Operation::Store { slot: element_slot }) =
                    self.operations.get(element_store_bci)
                else {
                    return None;
                };
                let element_local = element_store.writes().iter().find_map(|(slot, value)| {
                    (*slot == Slot::Local(*element_slot)).then_some(*value)
                })?;
                let resolves_to_element = |mut value| {
                    for _ in 0..=self.ssa.phis().len() {
                        if value == element_local {
                            return true;
                        }
                        let Some(replacement) = self.ssa.value(value).replaced_by() else {
                            return false;
                        };
                        value = replacement;
                    }
                    false
                };
                if self.ssa.value(element_local).uses().iter().any(|use_| {
                    !body_blocks.contains(use_.block())
                        || (use_.bci().is_none()
                            && !self.ssa.phis().iter().any(|join| {
                                join.block() == use_.block()
                                    && join.slot() == Slot::Local(*element_slot)
                                    && resolves_to_element(join.value())
                                    && join.inputs().iter().all(|input| {
                                        *input == PhiInput::Value(element_local)
                                            || *input == PhiInput::Itself
                                    })
                            }))
                }) {
                    return None;
                }
            }
            let element_uses = self
                .ssa
                .value(stack_result(instruction(element_bci)?)?)
                .uses();
            if element_uses.len() != 1
                || element_store_bci.is_some_and(|bci| element_uses[0].bci() != Some(bci))
                || (!direct_binding && element_uses[0].bci().is_none())
            {
                return None;
            }
            let effect = |bci| {
                self.ssa.effects().instructions().iter().find(|effect| {
                    effect.bci() == bci && Some(effect.block()) == self.block_of.get(&bci)
                })
            };
            if effect(length_bci)?.handlers() != effect(test_bci)?.handlers()
                || effect(element_bci)?.handlers() != effect(test_bci)?.handlers()
            {
                return None;
            }
            let mut bcis = Vec::new();
            let mut names = Vec::new();
            for stmt in [length_stmt, init, update] {
                stated_by_statement(stmt, &mut names, &mut bcis);
            }
            if direct_binding {
                stated_by_statement(element, &mut names, &mut bcis);
            } else {
                stated_by_expression(value, &mut names, &mut bcis);
            }
            stated_by_expression(cond, &mut names, &mut bcis);
            let origin = bcis
                .into_iter()
                .fold(OriginSet::new(Origin::direct(test_bci)), |origin, bci| {
                    origin.plus_derived(Origin::derived(bci))
                });
            let (name, projected_body) = if direct_binding {
                let StmtKind::Declare { name, .. } = &element.kind else {
                    return None;
                };
                (name.clone(), remainder.to_vec())
            } else {
                let name = wrapped_name.clone()?;
                let mut first = element.clone();
                let expression = match &mut first.kind {
                    StmtKind::Declare {
                        value: Some(value), ..
                    }
                    | StmtKind::Assign { value, .. } => value,
                    _ => return None,
                };
                if !replace_wrapped_array_read(expression, element_bci, &name, &proved_element) {
                    return None;
                }
                let mut projected_body = Vec::with_capacity(body.len());
                projected_body.push(first);
                projected_body.extend_from_slice(remainder);
                (name, projected_body)
            };
            Some((
                StmtKind::ForEach {
                    label: label.clone(),
                    ty: proved_element,
                    name,
                    iterable: (**array).clone(),
                    body: projected_body,
                },
                origin,
                direct_binding,
                (
                    *length_slot,
                    length_store_bci,
                    proof.slot,
                    proof.init_bci,
                    loop_blocks,
                ),
            ))
        })();
        let Some((
            kind,
            origin,
            direct_binding,
            (length_slot, length_bci, index_slot, init_bci, loop_blocks),
        )) = candidate
        else {
            return Ok(None);
        };
        let declarations = self.proved_array_cache_declarations(
            path,
            [(length_slot, length_bci), (index_slot, init_bci)],
            &loop_blocks,
            &origin,
        )?;
        Ok(Some((kind, origin, direct_binding, declarations)))
    }

    /// Finds only the two no-value declarations whose exact variables and source anchors belong
    /// to this successful projection. Any later access assigned to the same `LocalVariable` keeps
    /// its declaration: a slot alone does not prove that the cache and the later value are one
    /// lexical local, nor that they are different ones.
    fn proved_array_cache_declarations(
        &mut self,
        path: &[u32],
        caches: [(u16, u32); 2],
        loop_blocks: &BTreeSet<CanonicalBlockId>,
        origin: &OriginSet,
    ) -> Result<Vec<usize>, StopReason> {
        let [(length_slot, length_store_bci), (index_slot, init_bci)] = caches;
        let Some(length_variable) = self.reuse.variable_at(length_slot, length_store_bci) else {
            return Ok(Vec::new());
        };
        let Some(index_variable) = self.reuse.variable_at(index_slot, init_bci) else {
            return Ok(Vec::new());
        };
        if length_variable == index_variable {
            return Ok(Vec::new());
        }

        let origin_bcis = origin.bcis();
        let mut removable = Vec::new();
        for (variable, anchor, slot) in [
            (length_variable, length_store_bci, length_slot),
            (index_variable, init_bci, index_slot),
        ] {
            poll(self.budget, Some(anchor))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(anchor),
            )?;
            if !origin_bcis.contains(&anchor) {
                return Ok(Vec::new());
            }
            let planned_matches: Vec<_> = self
                .declarations
                .at_region
                .iter()
                .filter(|(owner, _)| path.starts_with(owner.as_slice()))
                .flat_map(|(_, declarations)| declarations)
                .filter(|declaration| declaration.variable == variable && declaration.at == anchor)
                .collect();
            if planned_matches.len() != 1 {
                continue; // No hoisted declaration exists, or the plan is not unique.
            }
            let Some(name) = self.names.text(variable) else {
                continue;
            };
            let statements: Vec<_> = self
                .stmts
                .iter()
                .enumerate()
                .filter(|(_, stmt)| {
                    stmt.origin.primary().bci() == anchor
                        && matches!(
                            &stmt.kind,
                            StmtKind::Declare {
                                name: declared_name,
                                value: None,
                                ..
                            } if declared_name == name
                        )
                })
                .map(|(index, _)| index)
                .collect();
            if statements.len() != 1 {
                continue;
            }

            let mut later_same_variable_access = false;
            for block in self.ssa.blocks() {
                for instruction in block.instructions() {
                    let bci = instruction.bci();
                    if bci == anchor
                        || origin_bcis.contains(&bci)
                        || loop_blocks.contains(block.block())
                    {
                        continue;
                    }
                    poll(self.budget, Some(bci))?;
                    charge(
                        self.budget,
                        CountedBudgetDimension::AnalysisSteps,
                        1,
                        Some(bci),
                    )?;
                    let touches = instruction
                        .reads()
                        .iter()
                        .chain(instruction.writes().iter())
                        .any(|(access, _)| *access == Slot::Local(slot));
                    if touches && self.reuse.variable_at(slot, bci) == Some(variable) {
                        later_same_variable_access = true;
                        break;
                    }
                }
                if later_same_variable_access {
                    break;
                }
            }
            if !later_same_variable_access {
                removable.push(statements[0]);
            }
        }
        removable.sort_unstable();
        removable.dedup();
        Ok(removable)
    }

    /// The condition one branch's test states, as the structure that holds it writes it.
    ///
    /// `taken` says which sense of the branch the structure continues on: an `if` always writes the
    /// fall-through condition (the branch *leaves* the `if` when its sense holds), a loop writes the
    /// sense that iterates. Both are the same decode fact read two ways.
    fn test_expr(&mut self, branch_bci: u32, taken: bool) -> Result<Expr, ValueRenderFailure> {
        let Some((op, _)) = self
            .operations
            .get(branch_bci)
            .and_then(Operation::comparison)
        else {
            return Err(format!(
                "the branch at BCI {branch_bci} has no decoded sense, so its condition cannot be written"
            )
            .into());
        };
        let Some(instruction) = self.instructions.get(&branch_bci).copied() else {
            return Err(format!("no names record for the branch at BCI {branch_bci}").into());
        };
        condition(op, &stack_operands(instruction), self, branch_bci, taken)
    }

    /// Proves that one numeric comparison's result has exactly one same-block adjacent zero-branch
    /// reader. The check is shared by the instruction walk and the condition composer so a value
    /// cannot be silently dropped or composed from a different consumer.
    fn numeric_comparison_reader(
        &self,
        instruction: &SsaInstruction,
    ) -> Result<(u32, ValueId), String> {
        let at = instruction.bci();
        let Some(Operation::NumericComparison { .. }) = self.operations.get(at) else {
            return Err(format!(
                "the instruction at BCI {at} is not a numeric comparison"
            ));
        };
        let writes: Vec<ValueId> = instruction
            .writes()
            .iter()
            .filter_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value))
            .collect();
        let [value] = writes.as_slice() else {
            return Err(format!(
                "the numeric comparison at BCI {at} writes {} stack value(s), not one",
                writes.len()
            ));
        };
        let uses = self.ssa.value(*value).uses();
        if uses.len() != 1 {
            return Err(format!(
                "the numeric comparison at BCI {at} has {} SSA reader(s), not one",
                uses.len()
            ));
        }
        let use_ = &uses[0];
        let Some(branch_bci) = use_.bci() else {
            return Err(format!(
                "the numeric comparison at BCI {at} reaches a phi input instead of a branch"
            ));
        };
        let Some(Operation::Comparison { op, .. }) = self.operations.get(branch_bci) else {
            return Err(format!(
                "the numeric comparison at BCI {at} is read at BCI {branch_bci}, which is not a conditional branch"
            ));
        };
        if !matches!(
            op,
            CompareOp::JumpIfZero
                | CompareOp::JumpIfNotZero
                | CompareOp::JumpIfNegative
                | CompareOp::JumpIfNotNegative
                | CompareOp::JumpIfPositive
                | CompareOp::JumpIfNotPositive
        ) {
            return Err(format!(
                "the numeric comparison at BCI {at} is not consumed by a zero branch"
            ));
        }
        let Some(branch_block) = self.block_of.get(&branch_bci) else {
            return Err(format!(
                "the zero branch at BCI {branch_bci} has no SSA block"
            ));
        };
        if self.block_of.get(&at) != Some(branch_block) || use_.block() != branch_block {
            return Err(format!(
                "the numeric comparison at BCI {at} and its zero branch at BCI {branch_bci} are not in one block"
            ));
        }
        let Some(block) = self.ssa.block(branch_block) else {
            return Err(format!(
                "the zero branch at BCI {branch_bci} has no SSA instruction block"
            ));
        };
        let Some(branch_index) = block
            .instructions()
            .iter()
            .position(|candidate| candidate.bci() == branch_bci)
        else {
            return Err(format!(
                "the zero branch at BCI {branch_bci} is missing from its SSA block"
            ));
        };
        if branch_index == 0 || block.instructions()[branch_index - 1].bci() != at {
            return Err(format!(
                "the numeric comparison at BCI {at} is not immediately before zero branch BCI {branch_bci}"
            ));
        }
        let branch_instruction = &block.instructions()[branch_index];
        let branch_operands = stack_operands(branch_instruction);
        if branch_operands.len() != 1 || branch_operands[0].1 != *value {
            return Err(format!(
                "the zero branch at BCI {branch_bci} does not read the numeric comparison's result directly"
            ));
        }
        let compare_operands = stack_operands(instruction);
        if compare_operands.len() != 2 || instruction.reads().len() != 2 {
            return Err(format!(
                "the numeric comparison at BCI {at} reads {} logical stack value(s), not two",
                compare_operands.len()
            ));
        }
        Ok((branch_bci, *value))
    }

    /// Appends one arm's statements to a vector of its own, so the `if` can hold them.
    fn arm(
        &mut self,
        region: &Region,
        into: &mut Vec<Stmt>,
        path: &RegionPath,
    ) -> Result<(), StopReason> {
        let outer = std::mem::take(&mut self.stmts);
        let outer_declared = self.declared.clone();
        let walked = self.region(region, path);
        let arm = std::mem::replace(&mut self.stmts, outer);
        self.declared = outer_declared;
        walked?;
        into.extend(arm);
        Ok(())
    }

    /// Appends one switch case body without resetting `declared` between cases.
    ///
    /// Java switch labels share one lexical block unless the source explicitly emits braces. The
    /// region emitter writes no per-case braces, so a name declared in an earlier case remains in
    /// lexical scope for later cases; `declarations` still prevents such a name from escaping the
    /// switch when a later consumer exists.
    fn switch_arm(
        &mut self,
        region: &Region,
        into: &mut Vec<Stmt>,
        path: &RegionPath,
    ) -> Result<(), StopReason> {
        let outer = std::mem::take(&mut self.stmts);
        let walked = self.region(region, path);
        let arm = std::mem::replace(&mut self.stmts, outer);
        walked?;
        into.extend(arm);
        Ok(())
    }

    /// Writes the declarations a region holds at its own start (P3 3.1).
    ///
    /// A variable is here when the write that first fills it is *not* in the innermost region that
    /// contains all of its uses: declaring it at that write would put it out of scope at the uses
    /// outside that write's region. The declaration written here carries no value — the writes that
    /// fill the variable are the assignments that follow — and every write of the variable then
    /// writes a plain assignment, because the declaration is not any write's any more.
    ///
    /// The statement is anchored at the write whose value states the variable's type, the same anchor
    /// the in-place declaration carried: the declaration's evidence is that instruction.
    fn declare_at(&mut self, path: &[u32]) -> Result<(), StopReason> {
        let Some(declared) = self.declarations.at_region.get(path).cloned() else {
            return Ok(());
        };
        for declaration in declared {
            let Some(name) = self.names.text(declaration.variable).map(str::to_owned) else {
                continue;
            };
            self.declared.insert(declaration.variable);
            self.push(Stmt::new(
                StmtKind::Declare {
                    ty: declaration.ty,
                    name,
                    value: None,
                },
                OriginSet::new(Origin::direct(declaration.at)),
            ))?;
        }
        Ok(())
    }

    /// Appends the statements of one block, in BCI order.
    fn block(&mut self, block: &CanonicalBlockId) -> Result<(), StopReason> {
        let Some(names) = self.ssa.block(block) else {
            let bcis = self.covered_bcis(block);
            let at = bcis.first().copied().unwrap_or(block.bci());
            return self.fallback(
                bcis,
                format!("no names record for the block at BCI {}", block.bci()),
                at,
            );
        };
        let instructions: Vec<SsaInstruction> = names.instructions().to_vec();
        for instruction in &instructions {
            self.instruction(instruction)?;
        }
        Ok(())
    }

    /// Finds the values whose expression has to be materialized before an independent instruction
    /// runs. The consumer walk follows only one expression value at a time; it never crosses a
    /// local store, a merge, or an unknown operation, so a final statement is the position whose
    /// effect the producer must precede.
    fn prepare_deferred_bindings(&mut self) -> Result<(), StopReason> {
        let mut candidates = Vec::new();
        let mut rejections = Vec::new();
        for (value, value_facts) in self.ssa.values_with_ids() {
            let anchor_for_charge = match value_facts.def() {
                Definition::Instruction { bci, .. } | Definition::Caught { bci, .. } => Some(*bci),
                Definition::Entry { block, .. } | Definition::Phi { block, .. } => {
                    Some(block.bci())
                }
            };
            poll(self.budget, anchor_for_charge)?;
            charge(
                self.budget,
                CountedBudgetDimension::IrItems,
                1,
                anchor_for_charge,
            )?;
            // A verified concatenation's own values have no life outside the chain — the shape
            // renders them in one expression — and the bounded final-consumer walk below would
            // refuse every `append`'s return once the receiver chain outgrows `MAX_VALUE_DEPTH`.
            if matches!(
                value_facts.def(),
                Definition::Instruction { bci, .. }
                    if self.compounds.owns_read(*bci)
                        || self.compounds.keeps_inline(*bci)
                        || self.postfix.owns(*bci)
                        || self.chains.owns(*bci)
            ) {
                continue;
            }
            let Some((producer, anchor, last)) = self.binding_producer(value, value_facts) else {
                continue;
            };
            if self.short_circuit_operands.contains(&value) {
                continue;
            }
            // A proved initializer is one evaluation unit: moving its allocation or one of its
            // element producers to a saved local would run it outside the array expression and
            // can put element effects before allocation failure. Keep the complete unit in the
            // expression position selected by its final consumer.
            if self.array_initializers.controls_binding(producer) {
                continue;
            }
            let Some(instruction) = self.instructions.get(&producer).copied() else {
                continue;
            };
            if !instruction
                .writes()
                .iter()
                .any(|(slot, written)| matches!(slot, Slot::Stack(_)) && *written == value)
            {
                continue;
            };
            if value_facts.uses().len() != 1 {
                rejections.push(BindingRejection {
                    value,
                    anchor,
                    producer,
                    reason: format!(
                        "the saved producer at BCI {anchor} has {} consumers, so one local binding cannot prove its execution count",
                        value_facts.uses().len()
                    ),
                });
                continue;
            }
            let Some(reader) = self.terminal_consumer(value, 0)? else {
                rejections.push(BindingRejection {
                    value,
                    anchor,
                    producer,
                    reason: format!(
                        "the saved producer at BCI {anchor} has no bounded final expression consumer"
                    ),
                });
                continue;
            };
            if reader <= last {
                continue;
            }
            let same_block = self
                .block_of
                .get(&anchor)
                .zip(self.block_of.get(&reader))
                .is_some_and(|(producer_block, reader_block)| producer_block == reader_block);
            if !same_block {
                rejections.push(BindingRejection {
                    value,
                    anchor,
                    producer,
                    reason: format!(
                        "the saved producer at BCI {anchor} and its final consumer at BCI {reader} do not share a proven declaration region"
                    ),
                });
                continue;
            }
            let owned_exit = self.guarded_return_exit(producer, last, reader);
            // A floating constant that feeds a fold-vulnerable operation is a candidate for the
            // fold boundary, not for ordering: the interval between it and its consumer holding
            // no independent statement is exactly the closed-constant shape the save exists for.
            let folding = matches!(
                self.operations.get(producer),
                Some(
                    Operation::Push(ConstantValue::Float(_))
                        | Operation::Push(ConstantValue::Double(_))
                )
            );
            match self.has_independent_boundary(value, last, reader, owned_exit)? {
                Some(true) => candidates.push((value, producer, anchor)),
                Some(false) if folding => candidates.push((value, producer, anchor)),
                Some(false) => {}
                None => rejections.push(BindingRejection {
                    value,
                    anchor,
                    producer,
                    reason: format!(
                        "the dependency chain from BCI {anchor} to final consumer {reader} is not bounded"
                    ),
                }),
            }
        }

        // A supported producer nested directly in another supported producer is covered by the
        // outer declaration only when the interval between them contains no independent effect.
        // Keeping both across such an effect is necessary: each value must be materialized at its
        // own execution point. A rejected outer producer is absent from this set, so its inner
        // candidate remains available for the conservative fallback. A folding candidate is never
        // covered that way: the outer declaration's initializer would be a closed constant
        // expression again, which is the fold the save exists to prevent — each real operation in
        // the tree keeps its own saved operand.
        let candidate_producers: BTreeSet<u32> = candidates
            .iter()
            .map(|(_, producer, _)| *producer)
            .collect();
        let folding_producers: BTreeSet<u32> = candidates
            .iter()
            .map(|(_, producer, _)| *producer)
            .filter(|producer| {
                matches!(
                    self.operations.get(*producer),
                    Some(
                        Operation::Push(ConstantValue::Float(_))
                            | Operation::Push(ConstantValue::Double(_))
                    )
                )
            })
            .collect();
        let mut retained = Vec::with_capacity(candidates.len());
        for candidate @ (value, producer, _) in candidates {
            let Some(reader) = self
                .ssa
                .value(value)
                .uses()
                .first()
                .and_then(|use_| use_.bci())
            else {
                retained.push(candidate);
                continue;
            };
            if folding_producers.contains(&producer) {
                retained.push(candidate);
                continue;
            }
            if candidate_producers.contains(&reader)
                && !matches!(
                    self.has_independent_boundary(value, producer, reader, None)?,
                    Some(false)
                )
            {
                retained.push(candidate);
                continue;
            }
            if !candidate_producers.contains(&reader) {
                retained.push(candidate);
            }
        }
        let candidates = retained.into_iter();
        for (index, (value, producer, anchor)) in candidates.enumerate() {
            poll(self.budget, Some(anchor))?;
            charge(
                self.budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(anchor),
            )?;
            let mut base = format!("saved{index}");
            let name = loop {
                let candidate = self.names.free_name_with(&base, || {
                    poll(self.budget, Some(anchor))?;
                    charge(
                        self.budget,
                        CountedBudgetDimension::IrItems,
                        1,
                        Some(anchor),
                    )
                })?;
                if !self.synthetic_names.contains(&candidate)
                    && !self.lambda_params.contains(&candidate)
                {
                    break candidate;
                }
                base = format!("{candidate}_");
            };
            self.synthetic_names.insert(name.clone());
            self.binding_plans.insert(
                anchor,
                BindingPlan {
                    value,
                    anchor,
                    producer,
                    name,
                },
            );
        }
        for rejection in rejections {
            poll(self.budget, Some(rejection.anchor))?;
            charge(
                self.budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(rejection.anchor),
            )?;
            self.binding_rejections
                .entry(rejection.anchor)
                .or_insert(rejection);
        }
        Ok(())
    }

    /// The instruction and source anchor that produce one value, when this value is one of the
    /// supported deferred shapes. A construction site is one producer even though its value's SSA
    /// definition is the constructor instruction rather than the allocation.
    fn binding_producer(&self, _value: ValueId, value_facts: &SsaValue) -> Option<(u32, u32, u32)> {
        let Definition::Instruction { bci, .. } = value_facts.def() else {
            return None;
        };
        let producer = *bci;
        if let Some(site) = self.sites.site_of(producer) {
            if producer != site.constructor {
                return None;
            }
            // The allocation/dup/constructor run is one expression, and its completed value is
            // available only at the constructor's instruction. The declaration therefore sits at
            // that final site position while the initializer keeps every site anchor in its
            // expression source map.
            return Some((producer, site.constructor, site.constructor));
        }
        let supported = match self.operations.get(producer) {
            Some(Operation::Invoke(_)) => true,
            Some(Operation::Field { .. }) => self
                .fields
                .claim(producer)
                .is_some_and(|(_, shape)| !shape.writes()),
            Some(
                Operation::ArrayLoad | Operation::ArrayElementLoad { .. } | Operation::ArrayLength,
            ) if !self.enums.owns(producer) => true,
            Some(Operation::NewArray { .. }) => true,
            Some(Operation::Arithmetic { op }) => {
                matches!(op, ArithmeticOp::Divide | ArithmeticOp::Remainder)
            }
            Some(Operation::PrimitiveConversion { .. }) => true,
            Some(Operation::InstanceOf { .. }) => true,
            Some(Operation::CheckCast { .. }) => !self.bridge_owns(producer),
            // A special floating constant is a supported producer for one reason only: the real
            // JVM operation that consumes it would otherwise be recovered as a closed Java constant
            // expression, and the compiler's fold of that expression is not provably the operation
            // the class file ran (`fneg` of the canonical NaN is the recorded case). Saving the
            // operand into the existing non-final named value keeps the operation at run time.
            Some(Operation::Push(constant)) => self.folding_candidate(value_facts, constant),
            _ => false,
        };
        supported.then_some((producer, producer, producer))
    }

    /// Whether one floating constant's value has to become a saved non-final local for the real
    /// JVM operation that consumes it to stay a runtime operation.
    ///
    /// Three facts have to meet, and all three are read, never guessed. The constant must be one
    /// of the admitted special values: a finite literal round-trips its hex spelling exactly and
    /// IEEE arithmetic over finite constants folds to the bits the JVM computed, so finite leaves
    /// never gain a local. The value's one consumer must be a floating negation or arithmetic —
    /// the constructions whose recovered text closes over the constant tree. And that tree must
    /// close over floating constants and floating operations alone: a tree with any other leaf (a
    /// local, a call, a merge, a conversion) is not a compile-time constant at all, and the
    /// operation it feeds is already runtime text that gains nothing from a local.
    fn folding_candidate(&self, value_facts: &SsaValue, constant: &ConstantValue) -> bool {
        let special = match constant {
            ConstantValue::Float(bits) => !ConstantValue::float_is_finite(*bits),
            ConstantValue::Double(bits) => !ConstantValue::double_is_finite(*bits),
            _ => return false,
        };
        if !special {
            return false;
        }
        let uses = value_facts.uses();
        let [use_] = uses else {
            return false;
        };
        let Some(use_bci) = use_.bci() else {
            return false;
        };
        if !matches!(
            self.operations.get(use_bci),
            Some(Operation::Negate) | Some(Operation::Arithmetic { .. })
        ) {
            return false;
        }
        let Some(use_instruction) = self.instructions.get(&use_bci) else {
            return false;
        };
        let operands = stack_operands(use_instruction);
        if operands.is_empty() {
            return false;
        }
        let mut seen = BTreeSet::new();
        operands
            .iter()
            .all(|(_, operand)| self.closed_floating_constant_tree(*operand, &mut seen, 0))
    }

    /// Whether the value tree closing over one operation's operands is made of floating constants
    /// and floating `Negate`/`Arithmetic` operations alone, within [`MAX_VALUE_DEPTH`].
    ///
    /// This is the bounded constant-tree judgment the fold boundary is taken with: the walk reads
    /// only what each value's own definition states, refuses at anything it cannot prove constant
    /// (an entry, a merge, a caught value, a local, a call, a conversion, another width), and so
    /// states "this recovered text would be one compile-time constant" or nothing.
    fn closed_floating_constant_tree(
        &self,
        value: ValueId,
        seen: &mut BTreeSet<ValueId>,
        depth: usize,
    ) -> bool {
        if depth > MAX_VALUE_DEPTH {
            return false;
        }
        // A value visited once already has its subtree's answer pending; a shared subtree is
        // re-entered only after its first walk completed, so a repeat is the closed shape it was
        // first found to be.
        if !seen.insert(value) {
            return true;
        }
        let Definition::Instruction { bci, .. } = self.ssa.value(value).def() else {
            return false;
        };
        match self.operations.get(*bci) {
            Some(
                Operation::Push(ConstantValue::Float(_))
                | Operation::Push(ConstantValue::Double(_)),
            ) => true,
            Some(Operation::Negate | Operation::Arithmetic { .. }) => {
                let Some(instruction) = self.instructions.get(bci) else {
                    return false;
                };
                let operands = stack_operands(instruction);
                !operands.is_empty()
                    && operands.iter().all(|(_, operand)| {
                        self.closed_floating_constant_tree(*operand, seen, depth + 1)
                    })
            }
            _ => false,
        }
    }

    /// Whether a value's same-block interval contains an instruction outside the expression that
    /// computes it. Pure arithmetic and stack/local plumbing are transparent; calls, reads,
    /// checks, stores, control transfers and unknown operations remain ordering boundaries.
    fn has_independent_boundary(
        &mut self,
        value: ValueId,
        last: u32,
        reader: u32,
        owned_monitor_exit: Option<u32>,
    ) -> Result<Option<bool>, StopReason> {
        let mut dependency_bcis = BTreeSet::new();
        if !self.collect_dependency_bcis(value, &mut BTreeSet::new(), &mut dependency_bcis, 0)? {
            return Ok(None);
        }
        let Some(block) = self.block_of.get(&reader).cloned() else {
            return Ok(None);
        };
        let Some(names) = self.ssa.block(&block) else {
            return Ok(None);
        };
        let mut consumer_bcis = BTreeSet::new();
        let Some(reader_instruction) = names
            .instructions()
            .iter()
            .find(|instruction| instruction.bci() == reader)
        else {
            return Ok(None);
        };
        let operands: Vec<ValueId> = reader_instruction
            .reads()
            .iter()
            .filter_map(|(slot, operand)| matches!(slot, Slot::Stack(_)).then_some(*operand))
            .collect();
        for operand in operands {
            if !self.collect_dependency_bcis(
                operand,
                &mut BTreeSet::new(),
                &mut consumer_bcis,
                0,
            )? {
                return Ok(None);
            }
        }
        for instruction in names.instructions() {
            let bci = instruction.bci();
            if bci <= last || bci >= reader {
                continue;
            }
            poll(self.budget, Some(bci))?;
            charge(self.budget, CountedBudgetDimension::IrItems, 1, Some(bci))?;
            if dependency_bcis.contains(&bci)
                || consumer_bcis.contains(&bci)
                || owned_monitor_exit == Some(bci)
                || self.transparent_between(instruction)
            {
                continue;
            }
            return Ok(Some(true));
        }
        Ok(Some(false))
    }

    /// Returns the one monitor exit that may be ignored for this exact Guard return and producer.
    /// The Guard plan supplies both the return's identity and the protected body range; all three
    /// positions must line up before the boundary walk receives an exemption.
    fn guarded_return_exit(&self, producer: u32, last: u32, reader: u32) -> Option<u32> {
        let ownership = self.guarded_return_exits.get(&reader)?;
        (ownership.body.0 <= producer
            && producer < ownership.body.1
            && last < ownership.normal_exit_bci
            && ownership.normal_exit_bci < reader)
            .then_some(ownership.normal_exit_bci)
    }

    /// Follows a one-use expression chain to its final statement. For example, `value() + 1`
    /// reaches the return after the `iadd`, so an independent effect between those instructions is
    /// part of the producer-order proof rather than being missed at the arithmetic reader.
    fn terminal_consumer(
        &mut self,
        value: ValueId,
        depth: usize,
    ) -> Result<Option<u32>, StopReason> {
        if depth > MAX_VALUE_DEPTH {
            return Ok(None);
        }
        let uses = self.ssa.value(value).uses();
        if uses.len() != 1 {
            return Ok(None);
        }
        let Some(reader) = uses[0].bci() else {
            return Ok(None);
        };
        poll(self.budget, Some(reader))?;
        charge(
            self.budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(reader),
        )?;
        let Some(instruction) = self.instructions.get(&reader).copied() else {
            return Ok(None);
        };
        if !self.expression_value_instruction(reader) {
            return Ok(Some(reader));
        }
        let Some(output) = instruction
            .writes()
            .iter()
            .find_map(|(slot, written)| matches!(slot, Slot::Stack(_)).then_some(*written))
        else {
            return Ok(None);
        };
        self.terminal_consumer(output, depth + 1)
    }

    /// An already supported expression-producing instruction that may be followed to the
    /// statement consuming its result. This reuses operation claims and does not add a second
    /// opcode effect table.
    fn expression_value_instruction(&self, bci: u32) -> bool {
        let Some(instruction) = self.instructions.get(&bci).copied() else {
            return false;
        };
        if !instruction
            .writes()
            .iter()
            .any(|(slot, _)| matches!(slot, Slot::Stack(_)))
        {
            return false;
        }
        if self.sites.site_of(bci).is_some() {
            return true;
        }
        match self.operations.get(bci) {
            Some(
                Operation::Arithmetic { .. }
                | Operation::Shift { .. }
                | Operation::Bitwise { .. }
                | Operation::Negate
                | Operation::PrimitiveConversion { .. }
                | Operation::Invoke(_),
            ) => true,
            Some(Operation::Field { .. }) => self
                .fields
                .claim(bci)
                .is_some_and(|(_, shape)| !shape.writes()),
            Some(
                Operation::ArrayLoad | Operation::ArrayElementLoad { .. } | Operation::ArrayLength,
            ) if !self.enums.owns(bci) => true,
            Some(Operation::NewArray { .. }) => true,
            Some(Operation::InstanceOf { .. }) => true,
            Some(Operation::CheckCast { .. }) => !self.bridge_owns(bci),
            _ => false,
        }
    }

    fn collect_dependency_bcis(
        &mut self,
        value: ValueId,
        seen: &mut BTreeSet<ValueId>,
        bcis: &mut BTreeSet<u32>,
        depth: usize,
    ) -> Result<bool, StopReason> {
        if depth > MAX_VALUE_DEPTH {
            return Ok(false);
        }
        if !seen.insert(value) {
            return Ok(true);
        }
        if let Some(expression) = self.conditional_values.get(&value) {
            bcis.extend(expression.origin.bcis());
            return Ok(true);
        }
        let definition = self.ssa.value(value).def().clone();
        let Definition::Instruction { bci, .. } = definition else {
            // Parameters and `this` are declared facts with no bytecode position. A merge or a
            // caught exception, however, crosses a control-flow boundary whose execution path
            // this straight-line proof cannot establish, so neither is a completed dependency.
            return Ok(matches!(definition, Definition::Entry { .. }));
        };
        poll(self.budget, Some(bci))?;
        charge(self.budget, CountedBudgetDimension::IrItems, 1, Some(bci))?;
        bcis.insert(bci);
        if let Some(site) = self.sites.site_of(bci) {
            bcis.extend(site.owned.iter().copied());
        }
        if matches!(self.operations.get(bci), None | Some(Operation::Other)) {
            return Ok(false);
        }
        let Some(instruction) = self.instructions.get(&bci).copied() else {
            return Ok(false);
        };
        let operands: Vec<ValueId> = instruction
            .reads()
            .iter()
            .filter_map(|(slot, operand)| matches!(slot, Slot::Stack(_)).then_some(*operand))
            .collect();
        let mut complete = true;
        for operand in operands {
            complete &= self.collect_dependency_bcis(operand, seen, bcis, depth + 1)?;
        }
        Ok(complete)
    }

    fn transparent_between(&self, instruction: &SsaInstruction) -> bool {
        match self.operations.get(instruction.bci()) {
            Some(
                Operation::Push(_)
                | Operation::Load { .. }
                | Operation::Duplicate
                | Operation::Shift { .. }
                | Operation::Bitwise { .. }
                | Operation::Negate
                | Operation::PrimitiveConversion { .. }
                | Operation::InstanceOf { .. },
            ) => true,
            Some(Operation::Arithmetic { op }) => matches!(
                op,
                ArithmeticOp::Add | ArithmeticOp::Subtract | ArithmeticOp::Multiply
            ),
            _ => false,
        }
    }

    /// Publishes one saved value at its producer/site anchor. The binding is inserted only after
    /// the declaration statement has been accepted, so a failed render never leaves a local name
    /// for later consumers to read.
    fn mark_binding_refused(&mut self, value: ValueId, producer: u32) {
        self.binding_refused.insert(value);
        if !self.deferred.iter().any(|(deferred, _)| *deferred == value) {
            self.deferred.push((value, producer));
        }
    }

    fn refuse_binding(&mut self, plan: &BindingPlan) {
        self.mark_binding_refused(plan.value, plan.producer);
    }

    /// Adds a binding site's owned bytecode to one refusal quote, charging each source identity
    /// that the new binding path inspects. The ordinary `quoted_bcis` list already accounts for
    /// the instruction being refused; this extension accounts only for the verified construction
    /// identities it must keep visible beside that instruction.
    fn binding_quote_bcis(&mut self, anchor: u32) -> Result<Vec<u32>, StopReason> {
        let budget = &*self.budget;
        let (mut bcis, visited_values) = self.quoted_bcis_with_budget(anchor, Some(budget))?;
        // The walker is read-only, so its budget is borrowed immutably while it traverses. Charge
        // the exact cardinality of its shared first-visit set afterwards: this accounts ordinary
        // SSA nodes as well as the source BCIs that happen to be quoted, and the set makes a shared
        // DAG one unit of work per ValueId rather than one unit per path through it.
        if visited_values != 0 {
            charge(
                self.budget,
                CountedBudgetDimension::IrItems,
                visited_values as u64,
                Some(anchor),
            )?;
        }
        if let Some(site) = self.sites.site_of(anchor) {
            for bci in site.owned.iter().copied() {
                if bcis.contains(&bci) {
                    continue;
                }
                poll(self.budget, Some(bci))?;
                charge(self.budget, CountedBudgetDimension::IrItems, 1, Some(bci))?;
                bcis.push(bci);
            }
        }
        bcis.sort_unstable();
        Ok(bcis)
    }

    fn reject_binding(&mut self, rejection: BindingRejection) -> Result<(), StopReason> {
        self.mark_binding_refused(rejection.value, rejection.producer);
        let bcis = self.binding_quote_bcis(rejection.anchor)?;
        self.fallback(bcis, &rejection.reason, rejection.anchor)
    }

    fn bind_value(&mut self, plan: BindingPlan) -> Result<(), StopReason> {
        let expression = match self.render_value(plan.value, plan.anchor, 0) {
            Ok(expression) => expression,
            Err(reason) => {
                self.refuse_binding(&plan);
                let bcis = self.binding_quote_bcis(plan.anchor)?;
                return self.fallback(bcis, &reason, plan.anchor);
            }
        };
        let Some(ty) = expression.presented.clone() else {
            self.refuse_binding(&plan);
            let bcis = self.binding_quote_bcis(plan.anchor)?;
            return self.fallback(
                bcis,
                format!(
                    "the saved value at BCI {} has no declared Java type",
                    plan.anchor
                ),
                plan.anchor,
            );
        };
        self.push(Stmt::new(
            StmtKind::Declare {
                ty: ty.clone(),
                name: plan.name.clone(),
                value: Some(expression),
            },
            OriginSet::new(Origin::direct(plan.anchor)),
        ))?;
        if !matches!(
            self.stmts.last().map(|statement| &statement.kind),
            Some(StmtKind::Declare { name, .. }) if name == &plan.name
        ) {
            self.refuse_binding(&plan);
            return Ok(());
        }
        self.bindings.insert(
            plan.value,
            Binding {
                name: plan.name,
                ty,
                anchor: plan.anchor,
            },
        );
        Ok(())
    }

    /// Appends the statement one instruction became, if it became one.
    fn instruction(&mut self, instruction: &SsaInstruction) -> Result<(), StopReason> {
        poll(self.budget, Some(instruction.bci()))?;
        let at = instruction.bci();
        if let Some(rejection) = self.binding_rejections.get(&at).cloned() {
            return self.reject_binding(rejection);
        }
        if let Some(plan) = self.binding_plans.get(&at).cloned() {
            return self.bind_value(plan);
        }
        // An instruction a verified concatenation chain or a verified construction site owns
        // produces no statement of its own: the text it would have written is written *inside* the
        // expression that shape became, and skipping it here is exactly what keeps an operand from
        // being evaluated twice (P3 2.2/2.3).
        //
        // A `catch` clause's parameter is the same shape for a different reason: the handler's entry
        // store *is* the declaration of `catch (T n)`, so writing it as a statement of the body
        // would write the declaration twice — and it reads the exception itself, which is a value
        // with no expression outside that declaration.
        //
        // A statement's own lead is the third: the instructions a `try` wrote in front of itself
        // (the block's run before the protected range began) are already in the text, and the body
        // that begins in that block holds them only because the block is where the range starts.
        if self.array_initializers.owns(at)
            || self.chains.owns(at)
            || self.sites.owns(at)
            || self.clause_parameters.contains(&at)
            || self.settled.contains(&at)
            || self.compounds.owns_copy(at)
        {
            return Ok(());
        }
        // Every instruction of one `field++`/`++field` shape but its `return` is presented by the
        // statement that `return` states (P3 2c.10/2c.18): the receiver load and its copy are the
        // update's receiver, the `getfield` and the `putfield` are the member it updates, the
        // constant and the `iadd` are the `1` it adds, and the `dup_x1` left the value the method
        // returns. None of them writes a statement of its own — the `putfield` would otherwise write
        // the field assignment this shape exists to spell as one update.
        if self.increments().owns(at) || self.postfix.owns(at) {
            return Ok(());
        }
        let write = instruction
            .writes()
            .iter()
            .find_map(|(slot, value)| match slot {
                Slot::Local(slot) => Some((*slot, *value)),
                Slot::Stack(_) => None,
            });
        match self.operations.get(at) {
            Some(Operation::Store { .. }) => {
                let Some((slot, _written)) = write else {
                    return self.fallback(
                        vec![at],
                        format!("the store at BCI {at} writes no local slot this run names"),
                        at,
                    );
                };
                let (variable, target) = match self.write_target(slot, at, "store") {
                    Ok(target) => target,
                    Err(reason) => {
                        return self.fallback(vec![at], &reason, at);
                    }
                };
                let Some((_, stored)) = stack_operands(instruction).last().copied() else {
                    self.declaration_refused(slot, at);
                    return self.fallback(
                        vec![at],
                        format!("the store at BCI {at} reads no value to store"),
                        at,
                    );
                };
                // `x = y = value` compiles to `…; dup; store; store`: the copy leaves the value on
                // the stack twice and each store takes one copy. The two stores write it **once** —
                // the first writes the expression the copy duplicated, and the second reads the
                // local the first one filled, which still holds that value because nothing runs
                // between them (P3 2c.14). Writing the expression into both stores would evaluate
                // it twice, which is a program the bytecode does not have.
                let chained = self.chained(at, stored);
                if let Some(Chained::Second { first, .. }) = chained
                    && let Some((first_variable, first_name)) =
                        self.chained_firsts.get(&first).cloned()
                {
                    // The local the first store wrote is named where the second store takes the
                    // copy: its text is that local's name, anchored at the store that filled it.
                    let value = self.local(first_variable, &first_name, first);
                    return self
                        .write_statement(variable, target, stored, value, at)
                        .map(|_| ());
                }
                let rendered = match chained {
                    // The first store writes the value the copy duplicated, and the copy's own BCI
                    // stays in the segment table as a derived anchor of the text that presents it.
                    Some(Chained::First { duplicated, dup_bci }) => {
                        self.render_value(duplicated, at, 0)
                            .map(|value| value.derived_from(dup_bci))
                    }
                    _ => self.render_value(stored, at, 0),
                };
                let value = match rendered {
                    Ok(value) => value,
                    Err(reason) => {
                        let bcis = self.chained_quote(at, chained);
                        self.declaration_refused(slot, at);
                        return self.fallback(bcis, &reason, at);
                    }
                };
                let written = self.write_statement(variable, target.clone(), stored, value, at)?;
                // The second store may name the local of the first only where the first really
                // wrote a statement: a declaration that was refused, a value that could not be
                // rendered and a type the write did not meet all leave the name unstated, and the
                // second store then refuses on its own copy exactly as it did before this rule.
                if written && matches!(chained, Some(Chained::First { .. })) {
                    self.chained_firsts.insert(at, (variable, target));
                }
                Ok(())
            }
            Some(Operation::Invoke(target)) => {
                // The call an instance initializer makes on its own uninitialized `this` is the
                // first thing a constructor does and is written as `super(…)` or `this(…)` — which
                // of the two is `init@1`'s verdict, read from the frames' token and the class the
                // caller stated (P3 2.3).
                if let Some(target_kind) = self.prologues.at(at).map(|prologue| prologue.target) {
                    return self.constructor_call(at, instruction, target_kind);
                }
                // A prologue the rule could not spell is quoted — the instruction itself, with the
                // requirement it fell short of. Writing it as an ordinary call would put a member
                // named `<init>` into the text and, worse, an assignment to the receiver the SSA
                // says the call converted: neither is a program.
                if let Some(reason) = self
                    .prologues
                    .refused_at(at)
                    .map(|refusal| refusal.message().to_string())
                {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(bcis, &reason, at);
                }
                // A synthetic accessor's call site is a direct field access, or it is not: the
                // verdict is `accessor@1`'s, and it is taken here so that a *presented* accessor
                // becomes the field access the source had — and a refused one keeps the call it had,
                // with the reason recorded against its BCI (P3 2.2, A12).
                let verdict = accessor::verify(target, self.members, self.pool);
                match verdict {
                    accessor::Verdict::Ordinary => {}
                    accessor::Verdict::Refused { evidence, refusal } => {
                        self.publish_accessor(at, evidence, false, Some(&refusal));
                    }
                    accessor::Verdict::Accessor { evidence, shape } => match shape.kind {
                        // A read accessor's value is written where the value is *consumed*: the
                        // store or the call that reads it renders `x.f`. Nothing consumes it here,
                        // so the invocation the bytecode made has no place in the body — unless
                        // something does consume it, and then this instruction writes nothing.
                        AccessorShape::FieldRead => {
                            if self.produced_value_reaches_a_reader(instruction) {
                                return Ok(());
                            }
                            let refusal = Refusal::shape(
                                "jre_accessor_unconsumed",
                                "nothing in this method reads the field the accessor returns, so the invocation the call site makes has no place in the body".to_string(),
                            );
                            self.publish_accessor(at, evidence, false, Some(&refusal));
                        }
                        AccessorShape::FieldWrite => {
                            return self.accessor_write(at, instruction, target, evidence, shape);
                        }
                    },
                }
                // A call whose value something *reads* is written where that value is read — by the
                // store, the return, the cast or the condition that renders it. Writing it here as
                // well would evaluate it twice, which is the invariant this layer keeps. What is
                // deliberately not a reader here is a **dynamic site**: a site that is refused still
                // has to leave the invocation that produced a value it captured in the body, and
                // that instruction is where it is written ([`Self::value_is_consumed`] answers that
                // question for the site's own arm).
                if write.is_none() && self.produced_value_reaches_a_reader(instruction) {
                    // The statement is deferred to the reader that will write the value. Which
                    // invocation was deferred is remembered with the value it produced, so that a
                    // reader that cannot write after all still has the invocation quoted rather
                    // than lost (P3 2.3 §0).
                    for (slot, value) in instruction.writes() {
                        if matches!(slot, Slot::Stack(_)) {
                            self.deferred.push((*value, at));
                        }
                    }
                    return Ok(());
                }
                let call = match self.call_expr(at, instruction, target, at, 0) {
                    Ok(call) => call,
                    Err(reason) => {
                        // A call whose result a local slot takes is a write of that slot, and this
                        // one did not happen: the variable keeps the declaration it never got
                        // (P3 2b.2).
                        if let Some((slot, _)) = write {
                            self.declaration_refused(slot, at);
                        }
                        let bcis = self.quoted_bcis(at);
                        return self.fallback(bcis, &reason, at);
                    }
                };                let Some((slot, written)) = write else {
                    // No local slot takes the result: the call is a statement of its own.
                    return self.push(Stmt::new(
                        StmtKind::Expr(call),
                        OriginSet::new(Origin::direct(at)),
                    ));
                };
                let (variable, target_name) = match self.write_target(slot, at, "call") {
                    Ok(target) => target,
                    Err(reason) => {
                        return self.fallback(vec![at], &reason, at);
                    }
                };
                // The invocation produces the value its own slot takes, so the value written and the
                // value whose evidence types it are one and the same.
                self.write_statement(variable, target_name, written, call, at)
                    .map(|_| ())
            }
            Some(Operation::Return) => {
                // The `return` one `field++`/`++field` shape ends in states the whole update: the
                // value it returns is the update's own, which no rendering of the shape's `dup_x1`
                // could spell (P3 2c.10/2c.18).
                if let Some(increment) = self.increments().statement_at(at) {
                    let update = increment_expression(increment);
                    let value = match self.adapt_return(update, at) {
                        Ok(value) => value,
                        Err(reason) => return self.fallback(self.quoted_bcis(at), &reason, at),
                    };
                    return self.push(Stmt::new(
                        StmtKind::Return { value: Some(value) },
                        OriginSet::new(Origin::direct(at)),
                    ));
                }
                if let Some(update) = self.postfix.at_return(at).cloned() {
                    let result = self.postfix_expression(&update, at);
                    let value = match result {
                        Ok(value) => value,
                        Err(reason) => {
                            let bcis = self.postfix_quote(&update);
                            return self.fallback(bcis, &reason, at);
                        }
                    };
                    return self.push(Stmt::new(
                        StmtKind::Return { value: Some(value) },
                        OriginSet::new(Origin::direct(at)),
                    ));
                }
                if let Some(bcis) = self.postfix.refusal_at(at) {
                    let bcis = bcis.to_vec();
                    return self.fallback(
                        bcis,
                        format!(
                            "the old-value update ending at BCI {at} has no complete same-target, single-consumer, same-handler and evaluation-order proof"
                        ),
                        at,
                    );
                }
                let value = match stack_operands(instruction).last().copied() {
                    Some((_, value)) => match self.return_expr(value, at, at) {
                        Ok(value) => Some(value),
                        Err(reason) => {
                            let bcis = self.quoted_bcis(at);
                            return self.fallback(bcis, &reason, at);
                        }
                    },
                    None => None,
                };
                self.push(Stmt::new(
                    StmtKind::Return { value },
                    OriginSet::new(Origin::direct(at)),
                ))
            }
            Some(Operation::Throw) => {
                let Some((_, value)) = single_stack_read(instruction) else {
                    return self.fallback(
                        self.quoted_bcis(at),
                        format!(
                            "the throw at BCI {at} does not read exactly one exception value this run states"
                        ),
                        at,
                    );
                };
                let value = match self.render_value(value, at, 0) {
                    Ok(value) => value,
                    Err(reason) => {
                        return self.fallback(self.quoted_bcis(at), &reason, at);
                    }
                };
                self.push(Stmt::new(
                    StmtKind::Throw { value },
                    OriginSet::new(Origin::direct(at)),
                ))
            }
            Some(Operation::Increment { slot, amount }) => {
                // The increment writes an assignment and never a declaration: an `iinc` reads the
                // slot as well as writing it, so its text is the same whether the variable was
                // declared here or earlier.
                let (variable, target_name) = match self.write_target(*slot, at, "increment") {
                    Ok(target) => target,
                    Err(reason) => {
                        return self.fallback(vec![at], &reason, at);
                    }
                };
                let (op, magnitude) = if *amount < 0 {
                    (BinaryOp::Subtract, -i64::from(*amount))
                } else {
                    (BinaryOp::Add, i64::from(*amount))
                };
                // The base of the increment presents the type its own declaration states, so the sum
                // it is written as has the type the arithmetic really produces (`int` for the whole
                // int-shaped family) and the assignment below can check it against the variable: an
                // increment of a `char`, a `byte` or a `short` is an `int` write the variable's own
                // type refuses, and no text of that variable's name states it.
                let base = self.local(variable, &target_name, at);
                let value = Expr::direct(
                    ExprKind::Binary {
                        op,
                        left: Box::new(base),
                        right: Box::new(Expr::direct(ExprKind::Integer(magnitude), at)),
                    },
                    at,
                );
                let value = match self.decided_type(variable) {
                    Some(Type::Boolean) => value,
                    Some(ty) => match meeting_position(
                        value,
                        &ty,
                        &format!(
                            "the increment at BCI {at} writes `{target_name}`, which this run decided holds `{}`",
                            ty.spell()
                        ),
                        Widening::Position,
                    ) {
                        Ok(value) => value,
                        Err(reason) => return self.fallback(vec![at], &reason, at),
                    },
                    None => value,
                };
                self.push(Stmt::new(
                    StmtKind::Assign {
                        name: target_name,
                        value,
                    },
                    OriginSet::new(Origin::direct(at)),
                ))
            }
            // Values whose text lands where they are consumed, the branches and switches the region
            // layer turned into `if`/`while`/`switch`, and the transfer the region's edge already
            // stands for: no statement of their own.
            Some(
                Operation::Push(_)
                | Operation::Load { .. }
                | Operation::Arithmetic { .. }
                | Operation::Shift { .. }
                | Operation::Bitwise { .. }
                | Operation::Negate
                | Operation::Comparison { .. }
                | Operation::Switch { .. }
                | Operation::Transfer,
            ) => Ok(()),
            Some(Operation::NumericComparison { .. }) => {
                // The same proof used by the condition composer decides whether this producer can
                // defer its text. Anything outside that proof remains an explicit quote.
                if self.numeric_comparison_reader(instruction).is_ok() {
                    Ok(())
                } else {
                    self.fallback(
                        self.quoted_bcis(at),
                        format!(
                            "the numeric comparison at BCI {at} has no single adjacent zero-branch reader"
                        ),
                        at,
                    )
                }
            }
            // A dynamic call site produces the instance it presents, so — like a push or a load —
            // its text lands where the value is consumed. What is *not* the same as a push is what
            // it means to drop it: creating the instance is an invocation the bytecode really makes,
            // so a site whose value nothing reads is quoted instead of being written off as a value
            // with no effect.
            Some(Operation::InvokeDynamic(site)) => {
                let value = instruction
                    .writes()
                    .iter()
                    .find_map(|(slot, value)| match slot {
                        Slot::Stack(_) => Some(*value),
                        Slot::Local(_) => None,
                    });
                // The instance this site produces is a value whose text lands where the value is
                // consumed — the store or the call that reads it renders it, and *that* rendering is
                // where the site is read, presented and recorded. So the only case this instruction
                // has anything of its own to say about is the one where nothing consumes it: the
                // invocation the site makes has no place in the body, and the site is quoted.
                let consumed = value.is_some_and(|value| self.value_is_consumed(value));
                if consumed {
                    return Ok(());
                }
                match self.lambda_expr(at, instruction, site, false) {
                    Ok(_) => Ok(()),
                    Err(reason) => self.fallback(vec![at], &reason, at),
                }
            }
            // A bridge cast is erased only where `bridge@1` proved it is the erasure of the value
            // it casts. An ordinary cast is a real expression, but it still has no statement of
            // its own: its final consumer writes the expression and therefore reads this value.
            Some(Operation::CheckCast { .. }) if self.bridge_owns(at) => Ok(()),
            Some(Operation::InstanceOf { .. }) => {
                if self.produced_value_reaches_a_reader(instruction) {
                    Ok(())
                } else {
                    self.fallback(
                        self.quoted_bcis(at),
                        format!("the type test at BCI {at} has no supported consumer"),
                        at,
                    )
                }
            }
            Some(Operation::CheckCast { .. }) => {
                if self.produced_value_reaches_a_reader(instruction) {
                    return Ok(());
                }
                self.fallback(
                    self.quoted_bcis(at),
                    format!(
                        "the cast at BCI {at} is not consumed by a statement this run can write, so its runtime check remains quoted"
                    ),
                    at,
                )
            }
            // A primitive conversion is a value-only expression, with no exception of its own.
            // Its text belongs at the final reader; when there is no reader, the quote walk still
            // retains any effectful operand that would otherwise be lost behind this pure opcode.
            Some(Operation::PrimitiveConversion { .. }) => {
                if self.produced_value_reaches_a_reader(instruction) {
                    Ok(())
                } else {
                    self.fallback(
                        self.quoted_bcis(at),
                        format!(
                            "the primitive conversion at BCI {at} produces a value nothing in this body reads, so its instruction and operand effects remain quoted"
                        ),
                        at,
                    )
                }
            }
            // A field instruction is a field access exactly where `field@1` proved which member it
            // names (P3 2.3). A claimed **write** is a statement of its own — `receiver.f = value`,
            // written where the instruction runs, which is what keeps a constructor's initializer
            // sequence in the order its bytes have it. A claimed **read** is a value: its text lands
            // where the value is consumed, so nothing is written here.
            Some(Operation::Field { .. }) => {
                let fields = self.fields;
                let Some((evidence, shape)) = fields.claim(at) else {
                    return self.fallback(
                        self.quoted_bcis(at),
                        format!(
                            "the field access at BCI {at} is not one this run proved names the member its own receiver's type declares, and a field instruction is presented only where the member it names is proven"
                        ),
                        at,
                    );
                };
                if shape.writes() {
                    return self.field_write(at, evidence, shape);
                }
                Ok(())
            }
            // An array element write is a statement of its own (P3 2b): `array[index] = value;`,
            // written where the instruction runs, with the three values it reads.
            Some(operation @ Operation::ArrayStore { .. }) => {
                self.array_write(at, instruction, stated_element(operation).as_ref())
            }
            // The reads of an array and the three instructions that allocate one produce a **value**
            // whose text lands where the value is consumed (P3 2b): the store, the call, the
            // `return` or another subscript renders it, and the instruction writes no statement of
            // its own. The dispatch table of an enum `switch` is one of those reads, claimed by the
            // rule that proved which table and which index it indexes and written into the switch's
            // selector (P3 2.3).
            //
            // A read or a creation whose value **no** reader writes is the case that question is
            // asked for: the instruction has no place in the body then, and dropping it would drop
            // an effect the bytecode really performs — the allocation, or the
            // `NullPointerException`/`ArrayIndexOutOfBoundsException` a read can throw. So it is
            // quoted, exactly like an unclaimed operation (the rule a dynamic site is held to).
            Some(
                Operation::ArrayLoad
                | Operation::ArrayElementLoad { .. }
                | Operation::ArrayLength
                | Operation::NewArray { .. },
            ) => {
                if self.enums.owns(at) || self.produced_value_reaches_a_reader(instruction) {
                    return Ok(());
                }
                self.fallback(
                    self.quoted_bcis(at),
                    format!(
                        "the array instruction at BCI {at} produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text"
                    ),
                    at,
                )
            }
            // A copy that a chained assignment writes writes no statement of its own: its value is
            // the first of the two stores that follow it (`x = y = value`, P3 2c.14), and the copy
            // itself is anchored there. Every other copy belongs to no verified shape of this body,
            // like an allocation or an unproven cast: a *stated* gap, quoted with its own BCI rather
            // than presented from half a proof.
            Some(Operation::Duplicate) => {
                if self.chained_pair(at).is_some() {
                    return Ok(());
                }
                self.fallback(
                    self.quoted_bcis(at),
                    format!(
                        "the instruction at BCI {at} belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds"
                    ),
                    at,
                )
            }
            // An allocation belongs to no verified shape of this body: it is a *stated* gap,
            // quoted with its own BCI rather than presented from half a proof.
            Some(Operation::Allocate { .. }) => self.fallback(
                self.quoted_bcis(at),
                format!(
                    "the instruction at BCI {at} belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds"
                ),
                at,
            ),
            // Monitor instructions remain shape facts: a `synchronized` block is the monitor's
            // enter and its exits. Where a guard proved that shape, these instructions are written
            // by the statement and never reach here; an unclaimed monitor has no statement of its
            // own, exactly like every other unclaimed operation.
            Some(Operation::Monitor { .. }) => self.fallback(
                vec![at],
                format!(
                    "the monitor at BCI {at} belongs to no guarded shape this run proved"
                ),
                at,
            ),
            Some(Operation::Other)
                // A `pop` whose evaluation another instruction's own text writes is neither a
                // statement nor a quote (P3 2c.31): the evaluation it discarded is the qualifier of
                // the static call in front of it (`arg0.stat()`, where writing the bare `stat()` would
                // drop the evaluation the source performed), or it is the value of the call behind it,
                // whose statement discards what the `pop` discards (`arg0.add("x");` for an `add`
                // whose `boolean` result nothing reads). Every other `pop`, and both `pop2` shapes,
                // are the stated gap below.
                if self.pops().accounts_for(at) =>
            {
                Ok(())
            }
            Some(Operation::Other) | None => self.fallback(
                vec![at],
                format!("the instruction at BCI {at} is not part of the provable subset"),
                at,
            ),
        }
    }

    /// The expression one `return` writes, typed by the member's own return descriptor.
    ///
    /// A method whose descriptor returns `Z` returns a **boolean** (P3-R5's reading, in the return
    /// position): the frames state one `int` shape for the four int-sized primitives, so only the
    /// signature says whether the `ireturn` of this body presents a boolean. A value with boolean
    /// evidence ([`Self::boolean_value`]) is written as one — the literal `0`/`1` becomes
    /// `false`/`true`, and every other proven value already prints what it is — while a value
    /// without it is rendered at its evaluation BCI and adapted only if the actual `ireturn`
    /// consumes a presented int-sized primitive. Other returns render the value at its evaluation
    /// BCI, then adapt it using the actual return instruction: a B/C/S `ireturn` may require a cast
    /// that Java's ordinary return position does not perform. Widening still belongs to the Java
    /// `return` position (P3 2c.29), a `char` descriptor can spell an in-range `int` constant as
    /// a character (P3 2c.30), and an unrelated value is refused ([`meeting_position`]).
    fn return_expr(
        &mut self,
        value: ValueId,
        eval_bci: u32,
        return_bci: u32,
    ) -> Result<Expr, ValueRenderFailure> {
        if matches!(self.return_type, Some(Type::Boolean)) {
            let rendered = self.render_value(value, eval_bci, 0)?;
            if rendered.presented == Some(Type::Boolean)
                && self
                    .conditional_values
                    .get(&value)
                    .is_some_and(|expression| expression.presented == Some(Type::Boolean))
            {
                return Ok(rendered);
            }
            if self.boolean_value(value, eval_bci) {
                return Ok(boolean_spelling(rendered));
            }
            return Ok(self.adapt_return(rendered, return_bci)?);
        }
        let rendered = self.render_value(value, eval_bci, 0)?;
        Ok(self.adapt_return(rendered, return_bci)?)
    }

    /// Apply the conversion proved by this method's actual `ireturn`, after the value was
    /// rendered at its evaluation position. A narrow cast or `Z` low-bit expression retains
    /// operand anchors and adds the return; synthetic constants use that same consumption BCI.
    fn adapt_return(&self, rendered: Expr, return_bci: u32) -> Result<Expr, String> {
        let Some(required) = self.return_type.as_ref() else {
            return Ok(rendered);
        };
        if matches!(required, Type::Boolean) {
            if self
                .instructions
                .get(&return_bci)
                .is_some_and(|instruction| instruction.opcode() == 0xac)
                && matches!(
                    rendered.presented.as_ref(),
                    Some(Type::Byte | Type::Char | Type::Short | Type::Int)
                )
            {
                return Ok(integer_low_bit_boolean(rendered, return_bci));
            }
            return Err(format!(
                "the value at BCI {} is returned from a method whose own descriptor returns `Z`, and this layer has no evidence that the value is a boolean or a presented int-sized primitive at `ireturn` BCI {return_bci}",
                rendered.origin.primary().bci()
            ));
        }
        if matches!(required, Type::Byte | Type::Char | Type::Short)
            && self
                .instructions
                .get(&return_bci)
                .is_some_and(|instruction| instruction.opcode() == 0xac)
        {
            let Some(presented) = rendered.presented.as_ref() else {
                return Err(format!(
                    "the value at BCI {} has no presented integer type for the narrow return at BCI {return_bci}",
                    rendered.origin.primary().bci()
                ));
            };
            if matches!(presented, Type::Byte | Type::Char | Type::Short | Type::Int)
                && presented != required
                && !widens(presented, required)
                && !narrowed_constant(&rendered, required)
            {
                return Ok(cast_argument(rendered, required, return_bci));
            }
        }
        meeting_position(
            rendered,
            required,
            &format!("the member's own descriptor returns `{}`", required.spell()),
            Widening::Position,
        )
    }

    /// Whether one value is **proven** boolean in a position that requires a boolean: a `Z` return,
    /// a branch test, or a store into a variable the run already knows holds a `boolean`.
    ///
    /// Three kinds of fact make up the proof, and each is a statement of the class file or of this
    /// body rather than a guess from the value's shape:
    ///
    /// * [`Self::boolean_evidence`] — the class's own descriptors: a `boolean` parameter's load, a
    ///   call whose callee descriptor returns `Z`, a claimed field read whose pool descriptor is `Z`;
    /// * [`Self::boolean_literal`] — the `0`/`1` literal, which is how a `boolean` is pushed, and
    ///   which only a context that *already* requires a boolean can read as one (`int x = 1;` and
    ///   `boolean c = true;` are the same bytes, and a fresh local states no type);
    /// * [`Self::boolean_local`] — a read of a local variable a write of this body declared
    ///   `boolean`, which is the item the plan calls "a local the body has proven boolean".
    ///
    /// Every other value — an arithmetic result, a comparison, a value a branch or a phi merged out
    /// of unproven parts, a `load` of a slot no write proved boolean — is *not* proven boolean, and
    /// a boolean context refuses it rather than guessing. This is not a type system: it reads the
    /// same descriptors ([`typed_arguments`] does) plus the declarations this very build wrote.
    fn boolean_value(&self, value: ValueId, at: u32) -> bool {
        self.boolean_proven(value, at) || self.boolean_literal(value)
    }

    /// Whether the layer presents one value as a boolean **by its own evidence**, outside any
    /// position that requires one: a descriptor fact, or a read of a variable the plan decided
    /// boolean ([`Self::boolean_local`]).
    ///
    /// This is the proof the `0`/`1` literal is deliberately not part of: the literal is how both a
    /// boolean and an `int` are pushed, so it is a boolean only where a position that already
    /// requires one reads it ([`Self::boolean_value`]), and an `int` everywhere else.
    fn boolean_proven(&self, value: ValueId, at: u32) -> bool {
        self.boolean_evidence(value, at).has_seed()
    }

    /// Why no Java comparison spells one branch's pair comparison, when the operands' own evidence
    /// says so.
    ///
    /// A pair comparison (`if_icmp*`, `if_acmp*`) reads two `int`s or two references, and the layer
    /// presents each operand the way that operand's own evidence spells it. One operand it proves
    /// boolean beside one it does not is therefore a comparison with no Java spelling — the text
    /// `flag() == 1` is refused by javac (`incomparable types: boolean and int`) — and ordering two
    /// booleans has none either (`bad operand types for binary operator '<'`), while equality between
    /// two of them does. Both are the same rule as a write that cannot be spelled as its variable's
    /// decided type: the structure terminates instead of publishing text the compiler rejects.
    ///
    /// The shape is the hand-built `Boundary` class in `tests/p3_boolean_contexts.rs` — a compiler
    /// folds `flag() == true` into `flag()`, so source never reaches it — and the change that fixed
    /// the operands' spelling recorded it as a boundary and handed its disposition to this rule.
    fn pair_comparison_refusal(
        &self,
        op: BinaryOp,
        operands: &[(Slot, ValueId)],
        at: u32,
    ) -> Option<String> {
        let left = self.boolean_proven(operands[0].1, at);
        let right = self.boolean_proven(operands[1].1, at);
        if left != right {
            return Some(format!(
                "the branch at BCI {at} compares a value this layer proves boolean with one it does not (`{}`), and no Java comparison spells that pair of operands: the text this layer would write is refused by javac (`incomparable types: boolean and int`)",
                op.spell()
            ));
        }
        (left && !matches!(op, BinaryOp::Equal | BinaryOp::NotEqual)).then(|| {
            format!(
                "the branch at BCI {at} orders two values this layer proves boolean with `{}`, which no Java source spells on `boolean` operands",
                op.spell()
            )
        })
    }

    /// The same bounded boolean proof the declaration plan applies, now reading locals that plan
    /// already decided. This keeps inline consumers, nested bitwise operands and declarations on
    /// one evidence rule.
    fn boolean_evidence(&self, value: ValueId, at: u32) -> BooleanEvidence {
        let is_boolean_local = |value, at| self.boolean_local(value, at);
        let mut visit = |_| Ok(());
        let mut context = BooleanProofContext {
            ssa: self.ssa,
            operations: self.operations,
            parameter_types: self.parameter_types,
            fields: self.fields,
            is_boolean_local,
            visit: &mut visit,
        };
        boolean_proof(&mut context, value, at, 0, &mut BTreeMap::new())
            .unwrap_or(BooleanEvidence::None)
    }

    /// Whether one value is a `0`/`1` literal: the constant a `boolean` is pushed as, in the
    /// positions that already require a boolean.
    fn boolean_literal(&self, value: ValueId) -> bool {
        let Definition::Instruction { bci, .. } = self.ssa.value(value).def() else {
            return false;
        };
        matches!(
            self.operations.get(*bci),
            Some(Operation::Push(ConstantValue::Int(0 | 1)))
        )
    }

    /// Whether one value reads a local variable the plan decided boolean: a `load` of that
    /// variable, or the entry/phi its own slot holds at `at`.
    ///
    /// The proof is the plan's own decision for that variable — read from one place, whatever the
    /// order the body is walked in — and it is the item the plan calls "a local the run proved
    /// boolean". Following the value further (through the write that filled the variable, or
    /// through a phi's operands) is exactly what this does **not** do: a variable no evidence
    /// proves is not a boolean local, however boolean the shape of its flow looks.
    fn boolean_local(&self, value: ValueId, at: u32) -> bool {
        read_variable(self.ssa, self.operations, self.reuse, value, at)
            .is_some_and(|variable| self.decided_boolean(variable))
    }

    /// Which store of a chained assignment one store is, when it is one of the two (P3 2c.14).
    ///
    /// `x = y = value` compiles to `…; dup; store; store`: the copy leaves the value on the stack
    /// twice and each store takes one copy. The two stores write that value **once** — the first
    /// writes the expression the copy duplicated, the second reads the local the first one filled —
    /// because writing the expression into both stores would evaluate it twice (`x = y = f()` would
    /// call `f` twice, `x = y = new C()` would allocate twice). Which store `at` is, is its own
    /// answer, so each of the two is written where the bytecode runs it and in that order.
    ///
    /// Every other `dup` is not this shape and keeps the quote it has: `newarray; dup; iconst_0;
    /// iastore` (an array initializer) has no store after the copy at all, `dup; astore; monitorenter`
    /// (a monitor's header) has one store and not two, and `dup_x1` is another operation entirely.
    fn chained(&self, at: u32, stored: ValueId) -> Option<Chained> {
        // The value this store takes off the stack must be one the copy produced: a store of any
        // other value is no part of this shape, whatever the instructions around it are.
        let Definition::Instruction { bci, .. } = self.ssa.value(stored).def() else {
            return None;
        };
        let dup_bci = *bci;
        let pair = self.chained_pair(dup_bci)?;
        if pair.first == at {
            return Some(Chained::First {
                duplicated: pair.duplicated,
                dup_bci,
            });
        }
        (pair.second == at).then_some(Chained::Second {
            first: pair.first,
            dup_bci,
        })
    }

    /// The two stores one copy writes, when the three instructions state that shape at all.
    ///
    /// The shape is exactly `dup; store; store`: three consecutive instructions of one block, the
    /// two stores writing **local** slots, and the two of them taking the two values the copy
    /// produced — one copy each, so that what the first store writes is what the second one reads
    /// by name. The instructions are consecutive *in the block*, which is what makes "nothing runs
    /// between them" a fact about the bytes rather than an assumption about them.
    fn chained_pair(&self, dup_bci: u32) -> Option<ChainedPair> {
        let dup = self.instructions.get(&dup_bci).copied()?;
        // A `dup` copies one category-1 value: one read, and the two stack slots its copies land in.
        let duplicated = match stack_operands(dup).as_slice() {
            [(_, value)] => *value,
            _ => return None,
        };
        let copies: Vec<ValueId> = dup
            .writes()
            .iter()
            .filter_map(|(slot, value)| match slot {
                Slot::Stack(_) => Some(*value),
                Slot::Local(_) => None,
            })
            .collect();
        if copies.len() != 2 {
            return None;
        }
        // The token of an allocation the constructor has not run for yet is not an instance: writing
        // `new C(…)` where the bytecode stored that token would move the construction in front of the
        // stores it never preceded, so a copy of one is not this shape. A copy of the **constructed**
        // instance is the shape's own case (`x = y = new C()`), and it renders as the one `new`
        // expression the construction site already owns.
        if let Definition::Instruction { bci, .. } = self.ssa.value(duplicated).def()
            && let Some(site) = self.sites.site_of(*bci)
            && (*bci == site.head || *bci == site.dup)
        {
            return None;
        }
        let block = self
            .block_of
            .get(&dup_bci)
            .and_then(|block| self.ssa.block(block))?;
        let index = block
            .instructions()
            .iter()
            .position(|instruction| instruction.bci() == dup_bci)?;
        let first = block.instructions().get(index + 1)?;
        let second = block.instructions().get(index + 2)?;
        let first_stored = local_store_value(self.operations, first)?;
        let second_stored = local_store_value(self.operations, second)?;
        // Each store takes one of the two copies, and they are two *different* copies: the value the
        // first store takes is not the value the second one does.
        if first_stored == second_stored
            || !copies.contains(&first_stored)
            || !copies.contains(&second_stored)
        {
            return None;
        }
        Some(ChainedPair {
            duplicated,
            first: first.bci(),
            second: second.bci(),
        })
    }

    /// The `field++`/`++field` shapes of this body (P3 2c.10/2c.18), found once.
    fn increments(&self) -> &FieldIncrements {
        self.increments.get_or_init(|| self.field_increments())
    }

    /// The `pop`s of this body whose evaluation another instruction's own text writes (P3 2c.31),
    /// found once.
    fn pops(&self) -> &DiscardedEvaluations {
        self.pops.get_or_init(|| self.discarded_evaluations())
    }

    /// Every `pop` of this body the two shapes of P3 2c.31 account for.
    ///
    /// The shapes are read off the instructions alone, and every one of their conditions is an
    /// identity the bytecode states rather than a guess from the sequence:
    ///
    /// * the instruction is a `pop` — not a `pop2`, which discards two slots — that reads exactly
    ///   one value;
    /// * the value was produced by the instruction **immediately before** it in the same block: a
    ///   `pop` between the evaluation and the call is the whole shape, and a value that reached the
    ///   `pop` across other instructions is not this spelling;
    /// * the one shape is the static call immediately **after** the `pop`, whose text is written with
    ///   that evaluation as its qualifier; the other is the call immediately **before** it, whose own
    ///   value is the one the `pop` discards and which fills **no local slot** — a call that fills one
    ///   is written as a declaration or an assignment, not as the statement that discards a result.
    ///   When both shapes meet at one `pop`, the preceding call's result is the discard statement:
    ///   writing it independently preserves the evaluation and avoids embedding it a second time.
    ///
    /// Whether a call really renders, and whether the evaluation can be written at all, is decided
    /// where the text is written: a call that refuses quotes itself, and its quote names the `pop`
    /// this plan gives it ([`Self::quoted_bcis`]), so nothing this layer cannot present is dropped
    /// from the artifact — which is the one failure a presentation may not have.
    fn discarded_evaluations(&self) -> DiscardedEvaluations {
        let mut plan = DiscardedEvaluations::default();
        for instruction in self.instructions.values() {
            if instruction.opcode() != OPCODE_POP {
                continue;
            }
            let at = instruction.bci();
            let Some(block) = self.block_of.get(&at).and_then(|id| self.ssa.block(id)) else {
                continue;
            };
            let instructions = block.instructions();
            let Some(index) = instructions
                .iter()
                .position(|instruction| instruction.bci() == at)
            else {
                continue;
            };
            let Some(previous) = index.checked_sub(1).and_then(|i| instructions.get(i)) else {
                continue;
            };
            let Some(next) = instructions.get(index + 1) else {
                continue;
            };
            let operands = stack_operands(instruction);
            let [(_, discarded)] = operands.as_slice() else {
                continue;
            };
            if !comes_from(self.ssa, *discarded, previous.bci()) {
                continue;
            }
            // An invocation whose result is uniquely consumed by this `pop` is already a Java
            // statement. Prefer that existing discard representation over reusing the invocation
            // as the next static call's expression qualifier; both plans must never own one value.
            if let Some(Operation::Invoke(_)) = self.operations.get(previous.bci())
                && self.call_result_is_discarded(previous, *discarded, at)
            {
                plan.accounted.insert(at);
                plan.discards.insert(previous.bci(), at);
                continue;
            }
            // The static call the `pop` sits in front of: the evaluation it discarded is the
            // qualifier the call's text has to carry.
            if let Some(Operation::Invoke(target)) = self.operations.get(next.bci())
                && target.kind() == InvokeKind::Static
            {
                // An ordinary checkcast immediately before the pop has no supported reader here:
                // treating it as the static call's qualifier would manufacture a receiver from a
                // value whose runtime check the bytecode discarded (and can produce invalid text
                // such as `((String) arg0).source()`). Bridge erasure is already proven by its own
                // rule and keeps the established qualifier shape; an ordinary cast remains quoted
                // while the static call is recovered independently.
                if matches!(
                    self.operations.get(previous.bci()),
                    Some(Operation::CheckCast { .. }) | Some(Operation::PrimitiveConversion { .. })
                ) && !self.bridge_owns(previous.bci())
                {
                    continue;
                }
                plan.accounted.insert(at);
                plan.qualifiers.insert(next.bci(), (at, *discarded));
                continue;
            }
        }
        plan
    }

    /// Whether one call's own value is what the `pop` after it discards, and nothing else reads it.
    ///
    /// The call has to be one whose result takes **no local slot**: a call whose value the SSA
    /// already places in a slot is written as that slot's declaration or assignment, and the
    /// statement that discards a result is not the text such a call has. The result is read by the
    /// `pop` and by nothing else — a second reader would have written the call where *it* reads the
    /// value, and the discard would then be of a value whose evaluation is written twice. The SSA
    /// value already owns its complete use list, so this checks only those recorded uses rather than
    /// rescanning every block for each `pop`.
    fn call_result_is_discarded(
        &self,
        call: &SsaInstruction,
        discarded: ValueId,
        pop: u32,
    ) -> bool {
        let produced = call
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .collect::<Vec<_>>();
        let [(_, written)] = produced.as_slice() else {
            return false;
        };
        if *written != discarded {
            return false;
        }
        if call
            .writes()
            .iter()
            .any(|(slot, _)| matches!(slot, Slot::Local(_)))
        {
            return false;
        }
        let uses = self.ssa.value(discarded).uses();
        uses.len() == 1 && uses[0].bci() == Some(pop)
    }

    /// Every `field++`/`++field` shape this body's own instructions, names and field evidence state.
    ///
    /// The shape is the eight consecutive instructions `load; dup; getfield; …; putfield; ireturn` of
    /// one update the method returns, in one of the two orders javac writes:
    ///
    /// * `load; dup; getfield; dup_x1; iconst_1; iadd; putfield; ireturn` — the copy duplicates the
    ///   field's **old** value before the sum, so what the `ireturn` reads is that old value: `n++`;
    /// * `load; dup; getfield; iconst_1; iadd; dup_x1; putfield; ireturn` — the sum is added first and
    ///   the copy duplicates it, so the `ireturn` reads the sum: `++n`.
    ///
    /// Which of the two a shape is, is therefore the identity of the value the copy duplicated, and
    /// never the position of the instructions alone. What the two have in common is the rest of the
    /// evidence: the receiver's `dup` copies the receiver **load**, that same copy is the receiver
    /// the `getfield` reads *and* the value below the one the `dup_x1` duplicates, the `getfield` and
    /// the `putfield` were both claimed by `field@1` for the **same** member, the `putfield` stores
    /// the update's own result, the constant is `1`, the `iadd` adds it to the field's value, and the
    /// `ireturn` — the block's instruction right after the write — returns the value the copy left. A
    /// `dup` or a `dup_x1` no shape states stays quoted exactly as it was.
    fn field_increments(&self) -> FieldIncrements {
        let mut plan = FieldIncrements::default();
        for instruction in self.instructions.values() {
            // The `dup_x1` is the shape's own centre: unlike the receiver's `dup` — which a chained
            // assignment's copy also is (P3 2c.14) — it exists in this body for nothing but the
            // update, so every candidate is taken from it and then checked whole.
            if instruction.opcode() != OPCODE_DUP_X1 {
                continue;
            }
            if let Some(shape) = self.field_increment(instruction) {
                plan.add(shape);
            }
        }
        plan
    }

    /// Render the already proved left side once, before the update, at its final Java position.
    /// The read is a real Field/Index child; the write is the postfix node's direct anchor.
    fn postfix_expression(
        &mut self,
        update: &PostfixUpdate,
        at: u32,
    ) -> Result<Expr, ValueRenderFailure> {
        let read = update.read;
        let target = match &update.target {
            PostfixTarget::Field { receiver, name } => {
                let receiver = self.render_value(*receiver, at, 1)?;
                Expr::direct(
                    ExprKind::Field {
                        receiver: Box::new(receiver),
                        name: name.clone(),
                    },
                    read,
                )
                .presenting(Type::Int)
            }
            PostfixTarget::Array { array, index } => {
                let array = self.render_value(*array, at, 1)?;
                let index = self.render_value(*index, at, 1)?;
                Expr::direct(
                    ExprKind::Index {
                        array: Box::new(array),
                        index: Box::new(index),
                    },
                    read,
                )
                .presenting(Type::Int)
            }
        };
        let origin = update.anchors.iter().fold(
            OriginSet::new(Origin::direct(update.store)),
            |origin, bci| origin.plus_derived(Origin::derived(*bci)),
        );
        let expression = Expr::new(
            ExprKind::PostIncrement {
                target: Box::new(target),
            },
            origin,
        )
        .presenting(Type::Int);
        Ok(self.adapt_return(expression, at)?)
    }

    /// If rendering fails after ownership was proved, one refusal names the entire suppressed
    /// chain and its producer BCIs. The prefix proof already collected every producer recursively.
    fn postfix_quote(&self, update: &PostfixUpdate) -> Vec<u32> {
        let mut bcis = update.anchors.clone();
        bcis.extend([update.store, update.returns]);
        bcis
    }

    /// One `field++`/`++field` shape, from the `dup_x1` that leaves the value the method returns.
    fn field_increment(&self, dup_x1: &SsaInstruction) -> Option<FieldIncrement> {
        let block = self.ssa.block(self.block_of.get(&dup_x1.bci())?)?;
        let instructions = block.instructions();
        let index = instructions
            .iter()
            .position(|instruction| instruction.bci() == dup_x1.bci())?;
        // The shape is one block's own eight consecutive instructions, one of the two windows the
        // copy sits in: the fourth instruction from the load (`n++`) or the sixth (`++n`).
        for start in [index.checked_sub(3), index.checked_sub(5)]
            .into_iter()
            .flatten()
        {
            let Some(window) = instructions.get(start..start.saturating_add(8)) else {
                continue;
            };
            if let Some(shape) = self.increment_window(window) {
                return Some(shape);
            }
        }
        None
    }

    /// The update one window of eight consecutive instructions states, when it states one.
    ///
    /// The window is `load; dup; getfield; _; _; _; putfield; ireturn`, with the copy, the constant
    /// and the sum in the three middle slots in one of the two orders [`Self::field_increments`]
    /// states and the method's `return` in the last. Everything else is an identity between the
    /// values those instructions carry, which is what keeps the text this shape becomes a statement
    /// of the class file rather than a guess from the instruction sequence.
    fn increment_window(&self, window: &[SsaInstruction]) -> Option<FieldIncrement> {
        let [load, dup, getfield, first, second, third, putfield, returns] = window else {
            return None;
        };
        // The window's last instruction is the **return** the update's value is returned from. A copy
        // whose value some other instruction reads is not this shape: `this.n++ + this.n++` sums the
        // two old values and stores `this.n++` puts one in a local, and the eight instructions are
        // not one update whose value is returned. Reading only "the last instruction reads the copy"
        // would claim `return this.n++;` for the first and *skip* the instructions that read the sum,
        // which is the one thing this layer may never do — drop an effect from the text.
        if !matches!(self.operations.get(returns.bci()), Some(Operation::Return)) {
            return None;
        }
        // The receiver: a **load** the `dup` copies. javac writes a simple receiver like this
        // (`aload_0; dup; getfield …`), and the update's text is written from the name that load's
        // own variable has — so a receiver whose name this layer cannot state (a call's result, a
        // nested read) leaves the copy outside the shape and quoted as before.
        let load_slot = match self.operations.get(load.bci())? {
            Operation::Load { slot } => *slot,
            _ => return None,
        };
        if dup.opcode() != OPCODE_DUP {
            return None;
        }
        let (_, duplicated_receiver) = single_stack_read(dup)?;
        if !comes_from(self.ssa, duplicated_receiver, load.bci()) {
            return None;
        }
        // The name of the variable that load reads: the same lookup that renders the load, so the
        // update writes the one name the run gave that value.
        let variable = self.reuse.variable_at(load_slot, load.bci())?;
        let name = self.names.text(variable)?;
        // Both field instructions, as `field@1` claimed them: a read and a write of the **same**
        // member, on the copy the receiver's `dup` produced. A field instruction this rule refused
        // states no member, and a shape whose member this layer cannot name is not one it presents.
        let (read, read_shape) = self.fields.claim(getfield.bci())?;
        let (write, write_shape) = self.fields.claim(putfield.bci())?;
        if read_shape.writes() || !write_shape.writes() {
            return None;
        }
        if !read_shape
            .receiver
            .is_some_and(|receiver| comes_from(self.ssa, receiver, dup.bci()))
        {
            return None;
        }
        if read.owner != write.owner
            || read.name != write.name
            || read.descriptor != write.descriptor
        {
            return None;
        }
        // The three middle instructions: which of the two orders they are in is which update the
        // shape is (see [`Self::field_increments`]).
        let (post, constant, add) = match (first.opcode(), second.opcode(), third.opcode()) {
            (OPCODE_DUP_X1, _, _) => (true, second, third),
            (_, _, OPCODE_DUP_X1) => (false, first, second),
            _ => return None,
        };
        if !matches!(
            self.operations.get(constant.bci()),
            Some(Operation::Push(ConstantValue::Int(1)))
        ) {
            return None;
        }
        if !matches!(
            self.operations.get(add.bci()),
            Some(Operation::Arithmetic {
                op: ArithmeticOp::Add
            })
        ) {
            return None;
        }
        // The copy's two operands: the value on top of the stack, which is the one it duplicates,
        // and the value below it — the other copy of the receiver, which the `putfield` reaches
        // under the value it stores. Nothing runs between the copy and that write but the constant
        // and the sum, which is what makes the receiver that write reads this copy and no other.
        let copy = if post { first } else { third };
        let (duplicated, below) = dup_x1_operands(copy)?;
        if !comes_from(self.ssa, below, dup.bci()) {
            return None;
        }
        // What the method returns: the value the copy left. Nothing else consumes it — the
        // `putfield` took the value it stores and the receiver — and the `ireturn` ends the shape.
        let (_, returned) = stack_operands(returns).last().copied()?;
        if !comes_from(self.ssa, returned, copy.bci()) {
            return None;
        }
        let stored = write_shape.value?;
        if post {
            // `n++`: the copy duplicates the field's **old** value, the sum adds one to that copy,
            // and the write stores the sum. What the method returns is the copy of the old value.
            if !comes_from(self.ssa, duplicated, getfield.bci())
                || !reads_from_two(self.ssa, add, copy.bci(), constant.bci())
                || !comes_from(self.ssa, stored, add.bci())
            {
                return None;
            }
        } else {
            // `++n`: the sum adds one to the value the field read produced, the copy duplicates the
            // **sum**, and the write stores it — the sum itself, or the copy the `putfield` reads.
            // What the method returns is that copy of the sum.
            if !reads_from_two(self.ssa, add, getfield.bci(), constant.bci())
                || !comes_from(self.ssa, duplicated, add.bci())
                || !(comes_from(self.ssa, stored, add.bci())
                    || comes_from(self.ssa, stored, copy.bci()))
            {
                return None;
            }
        }
        let mut anchors = vec![
            load.bci(),
            dup.bci(),
            getfield.bci(),
            constant.bci(),
            add.bci(),
            copy.bci(),
        ];
        anchors.sort_unstable();
        Some(FieldIncrement {
            update: putfield.bci(),
            returns: returns.bci(),
            anchors,
            text: if post {
                format!("{name}.{field}++", field = read.name)
            } else {
                format!("++{name}.{field}", field = read.name)
            },
            field_type: descriptor_type(&read.descriptor),
        })
    }

    /// The BCIs one refused store quotes when it is part of a chained assignment.
    ///
    /// The copy writes no statement of its own — the shape's two stores are what presents it — so a
    /// store that refuses has to name the copy as well, or the refusal would leave an instruction of
    /// this body unaccounted for ([`Self::quoted_bcis`] names the values behind a store, which for
    /// this shape are the copy's own operand's producers).
    fn chained_quote(&self, at: u32, chained: Option<Chained>) -> Vec<u32> {
        let mut bcis = self.quoted_bcis(at);
        if let Some(chained) = chained {
            let dup_bci = match chained {
                Chained::First { dup_bci, .. } | Chained::Second { dup_bci, .. } => dup_bci,
            };
            if !bcis.contains(&dup_bci) {
                bcis.push(dup_bci);
            }
        }
        bcis
    }

    /// The statement one write of a value into a variable becomes, and whether it wrote one.
    ///
    /// The type is never decided here: [`Self::declare`] answers with the plan's own decision for
    /// the variable, and this method only spells the value that decision requires. Three outcomes:
    ///
    /// * this write **carries** the variable's declaration: the plan's type, with a boolean value
    ///   spelled `true`/`false`;
    /// * an earlier write carried it, or the signature states it: an assignment, whose value is
    ///   checked against the same decision in both directions — a value the layer presents as a
    ///   boolean is not published into a variable the decision says holds something else
    ///   (`int local3; … local3 = <boolean value>;` is text the variable's own type rejects), and a
    ///   value it cannot prove boolean is not published into a variable the decision says holds a
    ///   `boolean` (`boolean local1; … local1 = 2;`, the same rejection read the other way);
    /// * the plan could not decide the variable's type: [`Self::declare`] has already refused the
    ///   structure this write belongs to, and nothing is written for it.
    ///
    /// `stored` is the value the writing instruction reads: a store writes the slot and reads the
    /// stack, and it is that value whose evidence the decision was taken from — so it is the value
    /// the assignment's compatibility check reads too. An instruction that produces the value it
    /// writes (an invocation the SSA already places in the slot) stores nothing it read, and names
    /// its own result there.
    ///
    /// The answer says whether a statement was written at all, and the second store of a chained
    /// assignment is what reads it: a refused declaration, a refused value and a refused type all
    /// leave the name unstated, and nothing may read a local no statement declared (P3 2c.14).
    fn write_statement(
        &mut self,
        variable: LocalVariable,
        name: String,
        stored: ValueId,
        value: Expr,
        at: u32,
    ) -> Result<bool, StopReason> {
        // The value meets the type the plan decided for this variable, read from that one decision
        // **before** the declaration is marked written: a value that cannot meet it refuses the write
        // whole — the declaration and the assignment both — so no name is declared that no value was
        // published for, and the write that follows (if any) keeps its own answer.
        //
        // A `boolean` variable is deliberately not this shape: which value may be published as a
        // boolean is the boolean rules' question (a `0`/`1` literal is how both are pushed, a proven
        // boolean already prints what it is, and a value with no evidence is refused by
        // [`Self::assignment`]), and this mechanism converts none of it.
        let value = match self.decided_type(variable) {
            Some(Type::Boolean) => value,
            Some(ty) => match meeting_position(
                value,
                &ty,
                &format!(
                    "the write at BCI {at} stores into `{name}`, which this run decided holds `{}`",
                    ty.spell()
                ),
                Widening::Position,
            ) {
                Ok(value) => value,
                Err(reason) => {
                    self.declaration_refused(variable.slot(), at);
                    return self.fallback(vec![at], &reason, at).map(|()| false);
                }
            },
            None => value,
        };
        match self.declare(variable, at)? {
            Declaration::Declared(ty) => {
                let value = if ty == Type::Boolean {
                    boolean_spelling(value)
                } else {
                    value
                };
                self.push(Stmt::new(
                    StmtKind::Declare {
                        ty,
                        name,
                        value: Some(value),
                    },
                    OriginSet::new(Origin::direct(at)),
                ))?;
                Ok(true)
            }
            // The declaration this write would have carried was refused, and the fallback that says
            // so is already recorded: the assignment after it is not written, because a name that
            // never got a declaration may not be assigned and a refused write may not be published
            // in a spelling the refusal contradicts.
            Declaration::Refused => {
                self.declaration_refused(variable.slot(), at);
                Ok(false)
            }
            Declaration::NotDue => self.assignment(variable, name, stored, value, at),
        }
    }

    /// The assignment one write becomes when its declaration is already written elsewhere.
    ///
    /// The value is checked against the variable's decided type before it is published, and the
    /// check reads the same one decision the declaration does — never a second guess. The two
    /// directions are one rule: the value must be spellable as the decided type. A `0`/`1` literal
    /// is spellable as either (it is how both are pushed, and it adapts to `true`/`false` where the
    /// decision is `boolean`), while a value only a descriptor or a decided-boolean local proves is
    /// a boolean and cannot be presented as anything else. The answer says whether the assignment
    /// was written, for the same reason [`Self::write_statement`]'s does.
    fn assignment(
        &mut self,
        variable: LocalVariable,
        name: String,
        stored: ValueId,
        value: Expr,
        at: u32,
    ) -> Result<bool, StopReason> {
        match self.decision(variable) {
            Some(Decided::Type(Type::Boolean)) => {
                if !self.boolean_value(stored, at) {
                    return self
                        .fallback(
                            vec![at],
                            format!(
                                "the value at BCI {at} is stored into `{name}`, which this run already stated holds a `boolean`, and this layer has no evidence that the value is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the variable's own type rejects"
                            ),
                            at,
                        )
                        .map(|()| false);
                }
                let value = boolean_spelling(value);
                self.push(Stmt::new(
                    StmtKind::Assign { name, value },
                    OriginSet::new(Origin::direct(at)),
                ))?;
                Ok(true)
            }
            Some(Decided::Type(ty)) => {
                // A value the layer presents as a boolean is not spellable as `{ty}`. The `0`/`1`
                // literal is not one of them: it is the same bytes for both, and it keeps the
                // integer spelling the decision states.
                if self.boolean_proven(stored, at) {
                    return self
                        .fallback(
                            vec![at],
                            format!(
                                "the value at BCI {at} is stored into `{name}`, which this run decided holds `{}`, and this layer presents the value as a boolean (a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this run decided `boolean`): the `boolean` spelling this layer would write is text the variable's own type rejects",
                                ty.spell()
                            ),
                            at,
                        )
                        .map(|()| false);
                }
                self.push(Stmt::new(
                    StmtKind::Assign { name, value },
                    OriginSet::new(Origin::direct(at)),
                ))?;
                Ok(true)
            }
            // The variable's own type could not be decided: the structure this write belongs to is
            // refused rather than written with a type nobody stated.
            Some(Decided::Unknown(reason)) => {
                let reason = reason.message(variable.slot(), at);
                self.fallback(vec![at], &reason, at).map(|()| false)
            }
            // The plan reached no variable here, which its own use map cannot produce for a write:
            // the write keeps the assignment it has always been.
            None => {
                self.push(Stmt::new(
                    StmtKind::Assign { name, value },
                    OriginSet::new(Origin::direct(at)),
                ))?;
                Ok(true)
            }
        }
    }

    /// Whether a local variable has to be declared at this write, and with which type.
    ///
    /// The type is the plan's own decision for this variable ([`decide_types`]), taken before any
    /// statement of the body existed and read here — at the in-place declaration the write carries,
    /// exactly as the hoisted declaration reads it. Deciding it here instead would be the defect
    /// this rule exists to close: the plan runs before the body's statements, so the two declaration
    /// paths read one answer instead of two, and the answer does not depend on the order the regions
    /// are walked in.
    ///
    /// `Declared` means this write carries the declaration; `NotDue` means it does not (the
    /// variable is a parameter's, its declaration is already written, or it has no name to declare);
    /// `Refused` means no declaration can be written and the write's assignment may not follow.
    fn declare(&mut self, variable: LocalVariable, at: u32) -> Result<Declaration, StopReason> {
        if self.declared.contains(&variable) || variable.slot() < self.parameters {
            return Ok(Declaration::NotDue);
        }
        if self.names.text(variable).is_none() {
            // No name to declare: the assignment that follows states the same thing and is already
            // reported as a fallback of its own.
            return Ok(Declaration::NotDue);
        }
        // The decision, and the only one: a variable the plan could not type is refused at the write
        // that would have declared it — the fallback quotes this write's bytecode, and the
        // declaration it would have carried is not written either. A frame entry that states no type
        // and one that states a name this layer cannot spell as a Java type (`spell_reference`) are
        // the two facts that reach this, and both are stated on the variable by the plan.
        let refusal = match self.decision(variable) {
            Some(Decided::Unknown(reason)) => Some(reason.message(variable.slot(), at)),
            _ => None,
        };
        if let Some(reason) = refusal {
            self.fallback(vec![at], &reason, at)?;
            return Ok(Declaration::Refused);
        }
        let Some(Decided::Type(ty)) = self.decision(variable).cloned() else {
            // No decision at all: the plan's own use map holds no write for this variable, which a
            // write cannot produce. Nothing is declared here and the assignment keeps the text it has
            // always had rather than inventing a second answer.
            return Ok(Declaration::NotDue);
        };
        self.declared.insert(variable);
        Ok(Declaration::Declared(ty))
    }

    /// The variable one write of local `slot` at BCI `at` fills, with the text its name is written
    /// as.
    ///
    /// `what` names the shape doing the writing, so that a refusal reads as the shape the caller
    /// refused. Two things stop it, and neither is a name to guess: a slot this run has no name for,
    /// and — P3 3.4 — a slot whose two variables this run cannot place the write in, which happens
    /// only where the evidence itself did not support splitting the slot in the first place.
    fn write_target(
        &self,
        slot: u16,
        at: u32,
        what: &str,
    ) -> Result<(LocalVariable, String), String> {
        let Some(variable) = self.reuse.variable_at(slot, at) else {
            return Err(format!(
                "the {what} at BCI {at} writes local {slot}, and the two variables the debug table states over that slot do not place this write in either"
            ));
        };
        match self.names.text(variable) {
            Some(name) => Ok((variable, name.to_string())),
            None => Err(format!(
                "the {what} at BCI {at} writes local {slot}, which has no name"
            )),
        }
    }

    /// Records that one write of local `slot` did not declare its variable (P3 2b.2).
    ///
    /// The statement a write carries is the only one that declares its variable — the other
    /// declaration path, the region-start one, is always written — so a write this build refused
    /// leaves the name a later use of that slot would spell without any declaration. Recording it
    /// here is what [`Self::undeclared_statement`] reads when it asks whether a statement may be
    /// published.
    fn declaration_refused(&mut self, slot: u16, at: u32) {
        let Some(variable) = self.reuse.variable_at(slot, at) else {
            return;
        };
        // A variable that already has its declaration — the one a region wrote at its own start, a
        // `catch` clause's parameter, a resource header — lost nothing here: every one of those
        // declarations precedes every use of the variable, so the name a later statement spells is
        // declared. Only the in-place declaration this write would have carried can be refused.
        if self.declared.contains(&variable) {
            return;
        }
        if let Some(name) = self.names.text(variable) {
            let name = name.to_string();
            self.undeclared.insert(name);
        }
    }

    /// Whether writing the **name** of local `slot` at the use BCI `at` denotes the value `denotes`.
    ///
    /// A local's name is not a name for a value: it is a name for whatever the slot holds where the
    /// name is read. The two agree exactly where the value in use at `at` *is* `denotes` — the last
    /// write to the slot before `at`, or the block's own entry state where nothing in the block wrote
    /// it. The smallest disagreement is a post-increment, `iload_0; iinc 0,1; ireturn`: the load
    /// reads the slot, the increment writes it, and the return reads the *loaded* value — so `local0`
    /// at the return denotes the incremented value, and naming the slot there is the opposite program.
    ///
    /// `at` is the **use** point — the BCI of the instruction that reads the value — and never the
    /// definition the value came from: what a slot holds is a statement about the reader's own point
    /// of the program, which is why a load's value can stop being the slot's value after it was read.
    fn slot_name_denotes_the_same_value(&self, slot: u16, denotes: ValueId, at: u32) -> bool {
        let Some(block) = self
            .block_of
            .get(&at)
            .and_then(|block| self.ssa.block(block))
        else {
            // No block of this run holds the use: nothing states what the slot holds there, and a
            // name written on no evidence states the wrong value half the time.
            return false;
        };
        let mut in_use = block
            .entry()
            .iter()
            .find_map(|(candidate, value)| match candidate {
                Slot::Local(candidate) if *candidate == slot => Some(*value),
                _ => None,
            });
        for instruction in block.instructions() {
            if instruction.bci() >= at {
                break;
            }
            for (written, value) in instruction.writes() {
                if matches!(written, Slot::Local(written) if *written == slot) {
                    in_use = Some(*value);
                }
            }
        }
        in_use == Some(denotes)
    }

    /// One local's name as the expression it is written as, presenting the type its **declaration**
    /// states.
    ///
    /// The type is the plan's one decision for that variable ([`Decided`]), which is also what the
    /// declaration and every assignment of it were written with — never a second reading of the
    /// frames, which state one slot shape for the four int-sized primitives and would therefore
    /// present a `char` parameter as the `int` a `+` converts as a **number**: that reading is the
    /// defect this rule closes (`append((int) c)` written `"" + c`). A variable whose type the plan
    /// could not decide presents none, which is the honest answer: no position converts a value
    /// whose own type it cannot state.
    fn local(&self, variable: LocalVariable, name: &str, at: u32) -> Expr {
        let local = Expr::direct(ExprKind::Local(name.to_string()), at);
        match self.decided_type(variable) {
            Some(ty) => local.presenting(ty),
            None => local,
        }
    }

    /// The type the plan decided for one variable, when it decided one.
    ///
    /// A **parameter** is decided by the member's own descriptor rather than by the plan: a
    /// parameter slot is written by the caller and never by the body, so the plan's map — which is
    /// keyed by the write each variable's type is decided from — holds no entry for it, while
    /// [`crate::facts::MethodFacts::parameter_types`] states what it holds (and is the only fact
    /// that can tell a `char`, a `byte` and a `short` from an `int`, since the frames state one
    /// shape for all four).
    fn decided_type(&self, variable: LocalVariable) -> Option<Type> {
        if variable.slot() == 0
            && self.has_receiver
            && let Some(declaring_class) = self.declaring_class
            && let Some(class) = spell_reference(declaring_class)
        {
            return Some(Type::Reference(class));
        }
        if variable.slot() < self.parameters {
            return self.parameter_types.get(&variable.slot()).cloned();
        }
        match self.decision(variable) {
            Some(Decided::Type(ty)) => Some(ty.clone()),
            Some(Decided::Unknown(_)) | None => None,
        }
    }

    /// Renders one SSA value as an expression, for a use at BCI `at`.
    ///
    /// `at` is the position at which the text produced here is **evaluated**: the instruction whose
    /// statement the expression is written into, or the consumer a nested expression is rendered
    /// for. It is never the definition the value came from, and keeping the two apart is what P3-R8
    /// is about — a value the bytecode computed at BCI 2 can be written into a statement that runs
    /// after a write at BCI 3, so every check that asks what a slot *holds*, and every refusal that
    /// quotes a position, has to be taken at `at` rather than at the producer's own BCI. Which is
    /// why a recursive call passes the context it was given and not the BCI of the instruction it is
    /// descending through.
    ///
    /// The nodes themselves keep the BCIs they were produced at as their own anchors
    /// (`Expr::direct`, `OriginSet` and the derived chain): origin is about provenance, `at` is about
    /// evaluation, and a refusal is the only thing that moves between the two.
    fn render_value(
        &mut self,
        value: ValueId,
        at: u32,
        depth: usize,
    ) -> Result<Expr, ValueRenderFailure> {
        if depth > MAX_VALUE_DEPTH {
            return Err(
                format!("the value at BCI {at} nests deeper than this layer renders").into(),
            );
        }
        if let Some(expression) = self.conditional_values.get(&value) {
            return Ok(expression.clone());
        }
        if let Some(binding) = self.bindings.get(&value).cloned() {
            return Ok(Expr::direct(ExprKind::Local(binding.name), at)
                .presenting(binding.ty)
                .derived_from(binding.anchor));
        }
        if self.binding_refused.contains(&value) {
            return Err(format!(
                "the value at BCI {at} was produced by a saved declaration this run could not commit"
            ).into());
        }
        if let Some(allocation_value) = self.array_initializers.allocation_value(value) {
            return self.render_value(allocation_value, at, depth + 1);
        }
        match self.ssa.value(value).def() {
            Definition::Entry { block, slot } | Definition::Phi { block, slot } => match slot {
                Slot::Local(slot) => {
                    // The value is a slot's own, and a slot's name is a name for it only where the
                    // slot still holds it at the use (P3 1.3d). Where the body wrote the slot in
                    // between, the name denotes the newer value, so this value is refused rather
                    // than spelled wrongly.
                    if !self.slot_name_denotes_the_same_value(*slot, value, at) {
                        return Err(format!(
                            "the value at BCI {at} is what local {slot} holds at BCI {}, and the body writes the slot again before BCI {at}: the slot's name would denote the value written in between, not this one",
                            block.bci()
                        ).into());
                    }
                    // Which of the slot's variables that name is (P3 3.4): a reused slot holds one
                    // variable in one arm and another in the other, and the name written here is the
                    // name of the variable whose range covers this use.
                    match self.reuse.variable_at(*slot, at) {
                        Some(variable) => match self.names.text(variable) {
                            Some(name) => Ok(self.local(variable, name, at)),
                            None => Err(format!("local {slot} has no name to write").into()),
                        },
                        None => Err(format!(
                            "the value at BCI {at} is what local {slot} holds, and the two variables the debug table states over that slot do not place this use in either"
                        ).into()),
                    }
                }
                Slot::Stack(stack) => Err(format!(
                    "the value at BCI {at} is the entry state of stack depth {stack}, which no instruction produced"
                ).into()),
            },
            Definition::Instruction { bci, .. } => {
                let bci = *bci;
                // The instance a verified construction site builds: the value a store, a call or a
                // `return` reads *is* the `new` expression, written here and nowhere else (P3 2.3).
                let sites = self.sites;
                if let Some(site) = sites.site_of(bci) {
                    return self.new_expr(site, at, depth);
                }
                let Some(operation) = self.operations.get(bci) else {
                    return Err(format!(
                        "the value at BCI {at} comes from BCI {bci}, whose operation this run did not decode"
                    ).into());
                };
                match operation {
                    Operation::Push(constant) => {
                        let origin = match constant {
                            ConstantValue::Class { ty, pool_index } => {
                                if class_literal_path_is_shadowed(ty, self.declaring_class) {
                                    return Err(format!(
                                        "the Class constant at BCI {bci} names `{ty}`, whose Java type path is shadowed by this class's own type name"
                                    ).into());
                                }
                                OriginSet::new(Origin::direct(bci).with_cp(*pool_index))
                            }
                            _ => OriginSet::new(Origin::direct(bci)),
                        };
                        // A floating constant's own bits decide what it is presented as: a finite
                        // value is its exact literal leaf, the three admitted special values are
                        // the proved constant divisions of those leaves, and any other NaN pattern
                        // names no expression this run may publish — the refusal is the value's
                        // own answer, stated at the constant's BCI so the consumer quoting it
                        // cannot be mistaken for a recovery.
                        match match constant {
                            ConstantValue::Float(bits) => Some((
                                ConstantValue::float_presentation(*bits),
                                SpecialValueWidth::Float,
                            )),
                            ConstantValue::Double(bits) => Some((
                                ConstantValue::double_presentation(*bits),
                                SpecialValueWidth::Double,
                            )),
                            _ => None,
                        } {
                            None => Ok(Expr::new(literal(constant), origin)),
                            Some((SpecialValuePresentation::Finite, _)) => {
                                Ok(Expr::new(literal(constant), origin))
                            }
                            Some((presentation @ (SpecialValuePresentation::PositiveInfinity
                            | SpecialValuePresentation::NegativeInfinity
                            | SpecialValuePresentation::CanonicalNan), width)) => {
                                Ok(special_value(bci, presentation, width))
                            }
                            Some((SpecialValuePresentation::UnpresentableNan, _)) => {
                                Err(format!(
                                    "the floating constant at BCI {bci} has a NaN sign, payload or signaling pattern this run cannot present exactly, and no constant expression of this layer normalizes it"
                                )
                                .into())
                            }
                        }
                    }
                    Operation::NumericComparison { .. } => Err(format!(
                        "the numeric comparison at BCI {bci} is not consumed by its proven zero branch"
                    ).into()),
                    // A load yields the value its slot held *where the load ran*, and that value is
                    // what a reader of it means — not the slot. Writing the slot's name at the use
                    // is the same expression only while the slot still holds it (P3 1.3d); where
                    // the body wrote the slot in between, the name would read the newer value and
                    // the text would state the opposite program, so the read is refused instead.
                    Operation::Load { slot } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!("no names record for the load at BCI {bci}").into());
                        };
                        let Some(read) = local_read(instruction, *slot) else {
                            return Err(format!(
                                "the value at BCI {at} comes from the load at BCI {bci}, whose read of local {slot} this run does not state"
                            ).into());
                        };
                        if !self.slot_name_denotes_the_same_value(*slot, read, at) {
                            return Err(format!(
                                "the value at BCI {at} is the value local {slot} held at BCI {bci}, and the slot does not hold it at BCI {at}: the slot's name would read the value the body wrote in between"
                            ).into());
                        }
                        // The load is a read of the slot, so the variable it names is the one whose
                        // record covers the load's own BCI (P3 3.4).
                        match self.reuse.variable_at(*slot, bci) {
                            Some(variable) => match self.names.text(variable) {
                                Some(name) => Ok(self.local(variable, name, bci)),
                                None => Err(format!("local {slot} has no name to write").into()),
                            },
                            None => Err(format!(
                                "the load at BCI {bci} reads local {slot}, and the two variables the debug table states over that slot do not place this read in either"
                            ).into()),
                        }
                    }
                    Operation::Arithmetic { op } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!(
                                "no names record for the instruction at BCI {bci}"
                            ).into());
                        };
                        let operands = stack_operands(instruction);
                        if operands.len() != 2 {
                            return Err(format!(
                                "the arithmetic at BCI {bci} reads {} values, not two",
                                operands.len()
                            ).into());
                        }
                        // Both operands are read by the sum, so both are checked where the sum is
                        // evaluated: a nested arithmetic does not move the use point to itself
                        // (P3-R8 — the value the operand names is on the stack when the sum runs,
                        // whatever the slot holds by then).
                        let left = self.render_value(operands[0].1, at, depth + 1)?;
                        let right = self.render_value(operands[1].1, at, depth + 1)?;
                        Ok(Expr::direct(
                            ExprKind::Binary {
                                op: arithmetic_op(*op),
                                left: Box::new(left),
                                right: Box::new(right),
                            },
                            bci,
                        ))
                    }
                    Operation::Shift { op } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!("no names record for the shift at BCI {bci}").into());
                        };
                        let operands = stack_operands(instruction);
                        let [(_, left_value), (_, right_value)] = operands.as_slice() else {
                            return Err(format!(
                                "the shift at BCI {bci} reads {} values, not two",
                                operands.len()
                            ).into());
                        };
                        let left = self.render_value(*left_value, at, depth + 1)?;
                        let right = self.render_value(*right_value, at, depth + 1)?;
                        let expected = match instruction.opcode() {
                            0x78 | 0x7a | 0x7c => Type::Int,
                            0x79 | 0x7b | 0x7d => Type::Long,
                            opcode => {
                                return Err(format!(
                                    "the shift at BCI {bci} has unexpected opcode {opcode:#04x}"
                                ).into());
                            }
                        };
                        let expression = Expr::direct(
                            ExprKind::Binary {
                                op: shift_op(*op),
                                left: Box::new(left),
                                right: Box::new(right),
                            },
                            bci,
                        );
                        if expression.presented.as_ref() != Some(&expected) {
                            return Err(format!(
                                "the shift `{}` at BCI {bci} has operands that cannot present the opcode's {} result in Java",
                                shift_op(*op).spell(),
                                expected.spell()
                            ).into());
                        }
                        Ok(expression)
                    }
                    Operation::Bitwise { op } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!(
                                "no names record for the bitwise instruction at BCI {bci}"
                            ).into());
                        };
                        let operands = stack_operands(instruction);
                        let [(_, left_value), (_, right_value)] = operands.as_slice() else {
                            return Err(format!(
                                "the bitwise operator at BCI {bci} reads {} values, not two",
                                operands.len()
                            ).into());
                        };
                        let boolean = self.boolean_evidence(value, at).has_seed();
                        let left = self.render_value(*left_value, at, depth + 1)?;
                        let right = self.render_value(*right_value, at, depth + 1)?;
                        let (left, right) = if boolean {
                            (boolean_spelling(left), boolean_spelling(right))
                        } else {
                            (left, right)
                        };
                        let left_type = left
                            .presented
                            .as_ref()
                            .map_or_else(|| "unknown".to_owned(), |ty| ty.spell().to_owned());
                        let right_type = right
                            .presented
                            .as_ref()
                            .map_or_else(|| "unknown".to_owned(), |ty| ty.spell().to_owned());
                        let expression = Expr::direct(
                            ExprKind::Binary {
                                op: bitwise_op(*op),
                                left: Box::new(left),
                                right: Box::new(right),
                            },
                            bci,
                        );
                        if expression.presented.is_none() {
                            return Err(format!(
                                "the bitwise operator `{}` at BCI {bci} has operands presented as `{left_type}` and `{right_type}`, which no Java integral or boolean bitwise expression accepts",
                                bitwise_op(*op).spell()
                            ).into());
                        }
                        Ok(expression)
                    }
                    Operation::Negate => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!("no names record for the negation at BCI {bci}").into());
                        };
                        let operands = stack_operands(instruction);
                        if operands.len() != 1 {
                            return Err(format!(
                                "the negation at BCI {bci} reads {} values, not one",
                                operands.len()
                            ).into());
                        }
                        let value = self.render_value(operands[0].1, at, depth + 1)?;
                        let negated = Expr::direct(
                            ExprKind::Neg {
                                value: Box::new(value),
                            },
                            bci,
                        );
                        if negated.presented.is_none() {
                            return Err(format!(
                                "the negation at BCI {bci} reads a value whose numeric type this run cannot state"
                            ).into());
                        }
                        Ok(negated)
                    }
                    Operation::PrimitiveConversion { source, target } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!(
                                "no names record for the primitive conversion at BCI {bci}"
                            ).into());
                        };
                        let Some((_, operand)) = single_stack_read(instruction) else {
                            return Err(format!(
                                "the primitive conversion at BCI {bci} does not read exactly one stack value this run states"
                            ).into());
                        };
                        let value = self.render_value(operand, at, depth + 1)?;
                        let Some(presented) = value.presented.as_ref() else {
                            return Err(format!(
                                "the primitive conversion at BCI {bci} expects `{}`, but its operand has no proved Java primitive type",
                                source.spell()
                            ).into());
                        };
                        if !primitive_conversion_source_matches(source, presented) {
                            return Err(format!(
                                "the primitive conversion at BCI {bci} expects `{}` but its operand is presented as `{}`, which does not meet the opcode's source category",
                                source.spell(),
                                presented.spell()
                            ).into());
                        }
                        Ok(Expr::new(
                            ExprKind::Cast {
                                ty: target.clone(),
                                value: Box::new(value),
                            },
                            OriginSet::new(Origin::direct(bci)),
                        ))
                    }
                    Operation::Invoke(target) => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!("no names record for the call at BCI {bci}").into());
                        };
                        // The `toString` a verified concatenation chain ends in *is* the
                        // concatenation: the expression is written here, where its value is read,
                        // and every original BCI of the chain stays in the table as an anchor.
                        if let Some(chain) = self.chains.value_at(bci) {
                            return self.concat_expr(chain, at, depth);
                        }
                        // The call is written where its value is consumed, so its receiver and its
                        // arguments are evaluated *there* and are checked at `at` — not at the
                        // call's own BCI, which nothing in the text rewinds to (P3-R8's call half).
                        self.invoke_expr(bci, instruction, target, at, depth)
                    }
                    Operation::CheckCast { ty } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!("no names record for the cast at BCI {bci}").into());
                        };
                        let Some((_, value)) = single_stack_read(instruction) else {
                            return Err(format!(
                                "the cast at BCI {bci} does not read exactly one value this run states"
                            ).into());
                        };
                        if self.bridge_owns(bci) {
                            // The cast is dropped *and* kept: the value is written where the
                            // forwarded invocation wrote it, and the cast's own BCI stays in the
                            // segment table as a derived anchor of the node that presents it. The
                            // value it drops is read where the cast's own value is used, so it is
                            // checked at `at`.
                            let expr = self.render_value(value, at, depth + 1)?;
                            return Ok(expr.derived_from(bci));
                        }
                        // A normal checkcast is a real Java expression. Its pool fact is the target
                        // type, and its operand is evaluated at the final consumer's position so
                        // deferred calls, locals and nested checks keep the same single-evaluation
                        // protocol as every other value expression.
                        let ty = spell_reference(ty).ok_or_else(|| {
                            format!(
                                "the cast at BCI {bci} names `{ty}`, which this layer cannot spell as a Java reference type"
                            )
                        })?;
                        let value = self.render_value(value, at, depth + 1)?;
                        Ok(Expr::new(
                            ExprKind::Cast {
                                ty: Type::Reference(ty),
                                value: Box::new(value),
                            },
                            OriginSet::new(Origin::direct(bci)),
                        ))
                    }
                    Operation::InstanceOf { ty } => {
                        let instruction =
                            self.instructions.get(&bci).copied().ok_or_else(|| {
                                format!("no names record for the type test at BCI {bci}")
                            })?;
                        let (_, operand) = single_stack_read(instruction).ok_or_else(|| {
                            format!(
                                "the type test at BCI {bci} does not read exactly one stack value"
                            )
                        })?;
                        let ty = spell_reference(ty).ok_or_else(|| {
                            format!(
                                "the type test at BCI {bci} names an invalid Java reference type"
                            )
                        })?;
                        let value = self.render_value(operand, at, depth + 1)?;
                        let value = if matches!(
                            value.kind,
                            ExprKind::Lambda { .. } | ExprKind::MethodReference { .. }
                        ) {
                            let target = value.presented.clone().ok_or_else(|| {
                                format!("the functional operand of the type test at BCI {bci} has no proven factory target")
                            })?;
                            Expr::new(
                                ExprKind::Cast {
                                    ty: target,
                                    value: Box::new(value),
                                },
                                OriginSet::new(Origin::derived(bci)),
                            )
                        } else {
                            value
                        };
                        let value = match value.presented.as_ref() {
                            None if matches!(value.kind, ExprKind::Null) => value,
                            Some(Type::Reference(name)) if name == "java.lang.Object" => value,
                            Some(Type::Reference(_)) => Expr::new(
                                ExprKind::Cast {
                                    ty: Type::Reference("java.lang.Object".to_string()),
                                    value: Box::new(value),
                                },
                                OriginSet::new(Origin::derived(bci)),
                            ),
                            _ => {
                                return Err(format!(
                                    "the type test at BCI {bci} has no proven reference operand"
                                ).into());
                            }
                        };
                        Ok(Expr::direct(
                            ExprKind::InstanceOf {
                                value: Box::new(value),
                                ty,
                            },
                            bci,
                        ))
                    }
                    // A dynamic call site is a *value* whose shape this layer decides from the
                    // class's own bootstrap table: a verified `LambdaMetafactory` site becomes a
                    // lambda or a method reference, and every other site is refused with the reason
                    // recorded against it (A04). Nothing here falls back to "it looks like a
                    // lambda": the shape comes from the bootstrap, never from the opcode.
                    Operation::InvokeDynamic(site) => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!(
                                "no names record for the dynamic site at BCI {bci}"
                            ).into());
                        };
                        // A value is being rendered *because* something consumes it.
                        self.lambda_expr(bci, instruction, site, true)
                    }
                    // A field read `field@1` proved is `receiver.f` — or `Type.f` for a static field,
                    // whose receiver is the owner type itself. The member's owner and name are the
                    // instruction's own pool facts; nothing is spelled from a guess (P3 2.3).
                    Operation::Field { .. } => {
                        let fields = self.fields;
                        let Some((evidence, shape)) = fields.claim(bci) else {
                            return Err(format!(
                                "the value at BCI {at} comes from the field access at BCI {bci}, which this run did not prove names the member its receiver's type declares"
                            ).into());
                        };
                        let receiver = match shape.receiver {
                            // The receiver the read dereferences is evaluated where the read's own
                            // value is used: a chain of reads is one expression, and its inner
                            // reads answer to the position its value is consumed at (P3-R8).
                            Some(value) => self.render_value(value, at, depth + 1)?,
                            None => {
                                // A static read's receiver is the owner type itself, and the owner
                                // is the pool's own name: one this layer cannot spell as a Java type
                                // has no expression to be, and is refused where it would be written.
                                let owner = spell_reference(&evidence.owner).ok_or_else(|| {
                                    format!(
                                        "the field read at BCI {bci} names the owner `{}`, which this layer cannot spell as a Java type",
                                        evidence.owner
                                    )
                                })?;
                                Expr::direct(ExprKind::Path(owner), bci)
                            }
                        };
                        let field = Expr::new(
                            ExprKind::Field {
                                receiver: Box::new(receiver),
                                name: evidence.name.clone(),
                            },
                            OriginSet::new(Origin::direct(bci)),
                        );
                        // The read presents the type the field's own descriptor states: `field@1`
                        // claimed this member by its pool entry, and the descriptor of that entry is
                        // the fact that says what the read's value is.
                        Ok(match descriptor_type(&evidence.descriptor) {
                            Some(ty) => field.presenting(ty),
                            None => field,
                        })
                    }
                    // One element of an array (P3 2b): the array and the index are the two values
                    // the read indexes with, and the read's value is used at `at` — so both are
                    // checked there, exactly as a field read's receiver is (P3-R8). What the element
                    // **is** comes from the array's own type where the frames state one, and from
                    // the opcode where they do not ([`array_element`]).
                    Operation::ArrayLoad | Operation::ArrayElementLoad { .. } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!("no names record for the array read at BCI {bci}").into());
                        };
                        let operands = stack_operands(instruction);
                        if operands.len() != 2 {
                            return Err(format!(
                                "the array read at BCI {bci} reads {} value(s), not an array and an index",
                                operands.len()
                            ).into());
                        }
                        // The dispatch table of an enum `switch` is this very read, claimed by the
                        // rule that proved which table and which index it indexes (P3 2.3): its text
                        // is the table indexed by the call the switch read its case index out of,
                        // and both original BCIs stay as derived anchors of the node.
                        let enums = self.enums;
                        if let Some((table, index)) = enums.claim(bci) {
                            let array = self.render_value(operands[0].1, at, depth + 1)?;
                            let selector = self.render_value(operands[1].1, at, depth + 1)?;
                            let origin = OriginSet::new(Origin::direct(bci))
                                .plus_derived(Origin::derived(table.bci))
                                .plus_derived(Origin::derived(index.bci));
                            return Ok(Expr::new(
                                ExprKind::Index {
                                    array: Box::new(array),
                                    index: Box::new(selector),
                                },
                                origin,
                            )
                            // The table the rule claimed is an `int[]` indexed by an `int`: the
                            // read's value is the `int` the switch's selector is.
                            .presenting(Type::Int));
                        }
                        let stated = stated_element(operation);
                        let element = array_element(
                            self.ssa,
                            self.operations,
                            operands[0].1,
                            stated.as_ref(),
                        );
                        let array = self.render_value(operands[0].1, at, depth + 1)?;
                        let index = self.render_value(operands[1].1, at, depth + 1)?;
                        let read = Expr::new(
                            ExprKind::Index {
                                array: Box::new(array),
                                index: Box::new(index),
                            },
                            OriginSet::new(Origin::direct(bci)),
                        );
                        // The element type, where the array's own type or the opcode states one: it
                        // is what the consuming position converts against, and stating none is the
                        // honest answer for a read of an array whose type this run cannot name.
                        Ok(match element {
                            Some(ty) => read.presenting(ty),
                            None => read,
                        })
                    }
                    // `array.length` (P3 2b): the length of the one array the instruction reads. Its
                    // own shape states the type — a length is an `int` whatever the element is.
                    Operation::ArrayLength => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!(
                                "no names record for the length read at BCI {bci}"
                            ).into());
                        };
                        let Some((_, array)) = single_stack_read(instruction) else {
                            return Err(format!(
                                "the length read at BCI {bci} does not read exactly one array"
                            ).into());
                        };
                        let array = self.render_value(array, at, depth + 1)?;
                        Ok(Expr::new(
                            ExprKind::ArrayLength {
                                array: Box::new(array),
                            },
                            OriginSet::new(Origin::direct(bci)),
                        ))
                    }
                    // `new T[n]`, and the multi-dimensional `new T[n][m]` (P3 2b): one length per
                    // dimension the instruction allocates, in the order it reads them off the
                    // stack, over the element type its own operand states. Like every other value,
                    // the lengths are evaluated where the creation's value is consumed — the
                    // instruction runs where its text lands.
                    Operation::NewArray {
                        element,
                        dimensions,
                        total_dimensions,
                    } => {
                        let Some(instruction) = self.instructions.get(&bci).copied() else {
                            return Err(format!(
                                "no names record for the array creation at BCI {bci}"
                            ).into());
                        };
                        let operands = stack_operands(instruction);
                        if operands.len() != usize::from(*dimensions) {
                            return Err(format!(
                                "the array creation at BCI {bci} allocates {} dimension(s) and its record states {} length(s)",
                                *dimensions,
                                operands.len()
                            ).into());
                        }
                        let mut lengths = Vec::with_capacity(operands.len());
                        let mut initializers = None;
                        let mut origin = OriginSet::new(Origin::direct(bci));
                        if let Some(initializer) =
                            self.array_initializers.at_allocation(bci).cloned()
                        {
                            let component = if *total_dimensions == 1 {
                                element.clone()
                            } else {
                                Type::Reference(format!(
                                    "{}{}", element.spell(),
                                    "[]".repeat(usize::from(*total_dimensions) - 1)
                                ))
                            };
                            let mut rendered = Vec::with_capacity(initializer.elements.len());
                            for (value, store_bci) in initializer.elements {
                                let element_value = if let Some(postfix) =
                                    initializer.local_postfix.get(&store_bci)
                                {
                                    let (variable, name) = self
                                        .write_target(postfix.slot, postfix.update, "postfix increment")
                                        .map_err(ValueRenderFailure::from)?;
                                    if self.reuse.variable_at(postfix.slot, postfix.load) != Some(variable)
                                        || self.decided_type(variable) != Some(Type::Int)
                                        || !self.slot_name_denotes_the_same_value(
                                            postfix.slot, postfix.old_local, postfix.load,
                                        )
                                    {
                                        return Err(format!(
                                            "the local postfix element at BCI {} has no single declared int local name for its old and updated values",
                                            postfix.update,
                                        ).into());
                                    }
                                    Expr::direct(
                                        ExprKind::PostIncrement {
                                            target: Box::new(self.local(variable, &name, postfix.load)),
                                        },
                                        postfix.update,
                                    )
                                } else {
                                    self.render_value(value, at, depth + 1)?
                                };
                                rendered.push(self.array_initializer_element(
                                    value,
                                    element_value,
                                    &component,
                                    at,
                                    store_bci,
                                )?);
                            }
                            for source in initializer.owned {
                                if source != bci {
                                    origin = origin.plus_derived(Origin::derived(source));
                                }
                            }
                            initializers = Some(rendered);
                        } else {
                            for (_, length) in operands {
                                lengths.push(self.render_value(length, at, depth + 1)?);
                            }
                        }
                        Ok(Expr::new(
                            ExprKind::NewArray {
                                element: element.clone(),
                                lengths,
                                initializers,
                                total_dimensions: *total_dimensions,
                            },
                            origin,
                        ))
                    }
                    other => Err(format!(
                        "the value at BCI {at} comes from an {other:?} at BCI {bci}, which produces no expression this subset writes"
                    ).into()),
                }
            }
            Definition::Caught { bci, .. } => Err(format!(
                "the value at BCI {at} is the exception reference of the throw site at BCI {bci}"
            ).into()),
        }
    }

    /// Renders one invocation, with its receiver and its arguments.
    ///
    /// `bci` is the invocation's own instruction, which is the node's anchor and the BCI the pool
    /// is read through; `at` is the position at which the rendered call is **evaluated**. They are
    /// the same for the statement a call writes of its own ([`Self::call_statement`]), and they
    /// differ where the call is written where its *value* is consumed: a nested `tick(x)` inside
    /// `tick(x) + ++x` reads its argument where the sum runs, after the increment (P3-R8).
    ///
    /// `depth` is the value-nesting depth the call is rendered at. It is passed **through** the
    /// call rather than restarted at it: a receiver or an argument is one more level of the same
    /// expression, and a recursion that does not count those levels is a native-stack recursion
    /// this layer has no bound on (the abort this change fixes).
    fn call_expr(
        &mut self,
        bci: u32,
        instruction: &SsaInstruction,
        target: &CallTarget,
        at: u32,
        depth: usize,
    ) -> Result<Expr, ValueRenderFailure> {
        let operands = stack_operands(instruction);
        let (receiver, args) = match target.kind() {
            // A static call reads no receiver from the stack: what its text names is the class its
            // own pool entry names, and the only thing that decides how it is written is whether
            // that class **is** this body's own. A call to this class keeps the bare name the
            // source had (`own(arg0)`); a call to any other class is written with that class
            // (`java.lang.Integer.valueOf(arg0)`, P3 4.4), because the bare `valueOf(arg0)` names
            // nothing this class declares. A run that states no declaring class cannot prove the
            // two differ, and writes the bare name rather than qualifying every static call.
            InvokeKind::Static => match (self.pops().qualifier_at(bci), self.declaring_class) {
                // The evaluation the bytecode popped is the call's **qualifier** (P3 2c.31a): the
                // source wrote `expr.staticMethod()`, so the bytecode evaluates `expr` — the only
                // thing a receiver can be *for* in front of an `invokestatic` — and throws the value
                // away. The text writes that evaluation where the call is, so exceptions thrown while
                // evaluating it are not dropped; a null result itself does not throw for a static
                // method qualifier. The class the pool entry names is not written, because the source's
                // own qualifier was the expression.
                (Some((pop, discarded)), _) => {
                    if target.is_interface_reference() {
                        return Err(format!(
                            "the call at BCI {bci} names static interface member `{}.{}` after the `pop` at BCI {pop}; Java does not allow an instance expression to qualify an interface static method",
                            target.owner(),
                            target.name()
                        ).into());
                    }
                    let target_owner = spell_reference(target.owner()).ok_or_else(|| {
                        format!(
                            "the call at BCI {bci} names static owner `{}`, which this layer cannot spell as a Java type",
                            target.owner()
                        )
                    })?;
                    let evaluation = self
                        .render_value(discarded, at, depth + 1)
                        .map_err(|failure| match failure {
                            ValueRenderFailure::Refusal(reason) => ValueRenderFailure::Refusal(
                                format!(
                                    "the call at BCI {bci} is the static call whose qualifier the bytecode evaluated and the `pop` at BCI {pop} discarded, and this layer cannot write that evaluation: {reason}"
                                ),
                            ),
                            ValueRenderFailure::Stop(stop) => ValueRenderFailure::Stop(stop),
                        })?;
                    // Only an expression of a **reference** type can be a call's qualifier: a primitive
                    // evaluation (an `int` constant, an arithmetic) has no member to read, and writing
                    // it as a qualifier would publish text `javac` refuses. The rendered expression
                    // states its own type, which is this layer's one reading of it.
                    match &evaluation.presented {
                        Some(Type::Reference(name)) if name == &target_owner => {
                            (Some(Box::new(evaluation)), operands.as_slice())
                        }
                        Some(Type::Reference(name)) => {
                            return Err(format!(
                                "the call at BCI {bci} names static owner `{target_owner}`, but the expression discarded at BCI {pop} has Java type `{name}`; this layer cannot prove that expression selects the constant-pool target"
                            ).into());
                        }
                        Some(ty) => {
                            return Err(format!(
                                "the call at BCI {bci} is the static call whose qualifier the bytecode evaluated and the `pop` at BCI {pop} discarded, and that evaluation presents `{}`: only an expression of a reference type can be a call's qualifier",
                                ty.spell()
                            ).into());
                        }
                        None => {
                            return Err(format!(
                                "the call at BCI {bci} is the static call whose qualifier the bytecode evaluated and the `pop` at BCI {pop} discarded, and this layer states no type for that evaluation: only an expression of a reference type can be a call's qualifier"
                            ).into());
                        }
                    }
                }
                (None, Some(declaring)) if declaring != target.owner() => {
                    // The owner is the pool's own name, spelled as Java spells a type: `/` becomes
                    // `.` and `$` is kept. A name with no Java type to be has no receiver to be
                    // either, and the call is refused where it would have been written.
                    let owner = self
                        .source_type_path_name(target.owner())
                        .or_else(|| spell_reference(target.owner()))
                        .ok_or_else(|| {
                        format!(
                            "the call at BCI {bci} names the owner `{}`, which this layer cannot spell as a Java type",
                            target.owner()
                        )
                    })?;
                    (
                        Some(Box::new(Expr::direct(ExprKind::Path(owner), bci))),
                        operands.as_slice(),
                    )
                }
                _ => (None, operands.as_slice()),
            },
            _ => {
                let (receiver, args) = operands
                    .split_first()
                    .ok_or_else(|| format!("the call at BCI {bci} reads no receiver"))?;
                // The receiver is a value the call reads; when the subset cannot render it (an
                // uninitialized `new`, an operation it does not model) the call falls back rather
                // than naming the owner type in its place.
                let rendered = self.render_value(receiver.1, at, depth + 1)?;
                let rendered = if target.kind() == InvokeKind::Special && target.name() != "<init>"
                {
                    self.special_receiver(bci, target, receiver.1, rendered)?
                } else {
                    rendered
                };
                let rendered = if matches!(
                    &rendered.kind,
                    ExprKind::Lambda { .. } | ExprKind::MethodReference { .. }
                ) {
                    immediate_functional_receiver(rendered, target, bci)?
                } else {
                    rendered
                };
                (Some(Box::new(rendered)), args)
            }
        };
        let mut arguments = Vec::with_capacity(args.len());
        for (_, value) in args {
            arguments.push(self.render_value(*value, at, depth + 1)?);
        }
        let arguments = self.arguments(target.descriptor(), arguments, bci)?;
        let call = Expr::direct(
            ExprKind::Call {
                receiver,
                name: target.name().to_string(),
                args: arguments,
            },
            bci,
        );
        Ok(match return_type(target.descriptor()) {
            Some(ty) => call.presenting(ty),
            None => call,
        })
    }

    /// The source spelling proved for a nested type in this class-source request, if any. Multiple
    /// target sites must agree on the same selected path before it can affect a static owner.
    fn source_type_path_name(&self, binary_name: &str) -> Option<String> {
        let mut selected: Option<&str> = None;
        for target in self.member_inner_targets {
            for segment in &target.source_type_path {
                if segment.binary_name != binary_name {
                    continue;
                }
                if selected.is_some_and(|previous| previous != segment.source_name) {
                    return None;
                }
                selected = Some(&segment.source_name);
            }
        }
        selected.map(str::to_owned)
    }

    /// Chooses the receiver of a non-constructor `invokespecial` from the same run's facts.
    ///
    /// A private member keeps the value the bytecode actually supplied, including another object.
    /// Every other special call needs both a direct-supertype match and proof that its receiver is
    /// the instance method's entry `this`; otherwise the call is refused so the enclosing region
    /// keeps its existing source-mapped fallback.
    fn special_receiver(
        &self,
        bci: u32,
        target: &CallTarget,
        receiver: ValueId,
        rendered: Expr,
    ) -> Result<Expr, String> {
        if self.declares_private(target) {
            return Ok(rendered);
        }
        if !self.has_receiver || !self.receiver_is_entry_this(receiver) {
            return Err(format!(
                "the non-constructor invokespecial at BCI {bci} is not proven to receive the entry `this`"
            ));
        }
        let owner = target.owner().as_bytes();
        if !target.is_interface_reference()
            && self
                .direct_super_class
                .is_some_and(|super_class| super_class.0.as_slice() == owner)
        {
            return Ok(Expr::new(
                ExprKind::Super { qualifier: None },
                rendered.origin,
            ));
        }
        if target.is_interface_reference()
            && self
                .direct_interfaces
                .iter()
                .any(|interface| interface.raw().0.as_slice() == owner)
        {
            if !self.interface_super_calls.iter().any(|call| {
                call.owner == target.owner()
                    && call.name == target.name()
                    && call.descriptor == target.descriptor()
            }) {
                return Err(format!(
                    "the interface-special target `{}.{}` with descriptor `{}` at BCI {bci} has no selected proof of a legal source qualifier and unique default binding",
                    target.owner(),
                    target.name(),
                    target.descriptor()
                ));
            }
            let qualifier = spell_reference(target.owner()).ok_or_else(|| {
                format!(
                    "the direct interface owner `{}` of the invokespecial at BCI {bci} cannot be spelled as a Java type",
                    target.owner()
                )
            })?;
            return Ok(Expr::new(
                ExprKind::Super {
                    qualifier: Some(qualifier),
                },
                rendered.origin,
            ));
        }
        Err(format!(
            "the non-constructor invokespecial at BCI {bci} names `{}` without a matching direct supertype or private declaration",
            target.owner()
        ))
    }

    /// Proves a receiver is the direct `aload 0` value from the method entry.
    fn receiver_is_entry_this(&self, receiver: ValueId) -> bool {
        match self.ssa.value(receiver).def() {
            Definition::Entry {
                slot: Slot::Local(0),
                ..
            } => true,
            Definition::Instruction { bci, .. } => {
                let Some(Operation::Load { slot: 0 }) = self.operations.get(*bci) else {
                    return false;
                };
                let Some(instruction) = self.instructions.get(bci) else {
                    return false;
                };
                let Some(entry) = local_read(instruction, 0) else {
                    return false;
                };
                matches!(
                    self.ssa.value(entry).def(),
                    Definition::Entry {
                        slot: Slot::Local(0),
                        ..
                    }
                )
            }
            Definition::Entry {
                slot: Slot::Stack(_),
                ..
            }
            | Definition::Entry {
                slot: Slot::Local(1..),
                ..
            }
            | Definition::Phi { .. }
            | Definition::Caught { .. } => false,
        }
    }

    /// Whether the current class header declares this exact special target private.
    fn declares_private(&self, target: &CallTarget) -> bool {
        let Some(owner) = self.declaring_class else {
            return false;
        };
        if owner != target.owner() {
            return false;
        }
        // A same-read class header is authoritative, including an explicit non-private flag. The
        // lower-level ClassMembers seam is only a fallback for callers that did not carry that
        // header, and it must never override a public declaration from the header.
        if let Some(methods) = self.class_methods {
            return methods.iter().any(|method| {
                method.name.raw().0.as_slice() == target.name().as_bytes()
                    && method.descriptor.raw().0.as_slice() == target.descriptor().as_bytes()
                    && method.access_flags & ACC_PRIVATE != 0
            });
        }
        self.members.is_some_and(|members| {
            members.owner() == owner
                && members.members().iter().any(|member| {
                    member.name() == target.name()
                        && member.descriptor() == target.descriptor()
                        && member.access_flags() & ACC_PRIVATE != 0
                })
        })
    }

    /// The arguments of one invocation, each written as the **callee's own descriptor** requires.
    ///
    /// Two readings of the same descriptor, and both are the callee's facts: [`typed_arguments`]
    /// spells the `boolean` a `Z` parameter declares (`append(true)`'s `iconst_1` is the `int`-shaped
    /// `1`, and only the descriptor says so), and [`Self::invocation_argument`] makes the selected
    /// primitive or reference type explicit. A `char` argument passed to an `int` parameter therefore
    /// receives the invocation's own cast, while a `byte`/`short` constant receives the explicit
    /// narrow cast the descriptor requires; a reference relation this layer cannot prove is refused.
    ///
    /// A descriptor this layer cannot read, or one whose parameter count differs from the arguments
    /// the call reads, is also refused. The call then keeps its existing source-mapped fallback
    /// rather than spelling arguments under a guessed signature.
    ///
    /// The arguments a **lambda's factory site** binds are deliberately not this shape: their types
    /// are the `invokedynamic` descriptor's statement about the SAM and not about the implementation
    /// the reference names ([`Self::lambda_expr`] builds them), so nothing there is re-typed. This
    /// entry point serves the invocations a body really performs: `invoke*` and the constructor call
    /// of a construction site and of a `super(…)`/`this(…)` prologue.
    fn arguments(
        &self,
        descriptor: &str,
        arguments: Vec<Expr>,
        bci: u32,
    ) -> Result<Vec<Expr>, String> {
        self.arguments_with_parameter_offset(descriptor, arguments, bci, 0)
    }

    /// A member-class `<init>` descriptor starts with the verified synthetic outer parameter.
    /// That physical parameter is emitted as the qualified receiver, leaving source arguments to
    /// be checked under the descriptor's remaining positions without weakening overload fixing.
    fn arguments_after_first_parameter(
        &self,
        descriptor: &str,
        arguments: Vec<Expr>,
        bci: u32,
        member: Option<&init::MemberInnerSite>,
    ) -> Result<Vec<Expr>, String> {
        let member = member.ok_or_else(|| {
            format!("the constructor at BCI {bci} has no proved member parameter prefix")
        })?;
        let Some((parameters, _)) = lambda::parse_method(descriptor) else {
            return Err(format!(
                "the member constructor at BCI {bci} has an unreadable method descriptor"
            ));
        };
        let expected_outer = spell_reference(&format!("L{};", member.outer))
            .ok_or_else(|| format!("the proven outer type `{}` cannot be spelled", member.outer))?;
        if parameters.first() != Some(&Type::Reference(expected_outer.clone())) {
            return Err(format!(
                "the first physical parameter of the member constructor at BCI {bci} does not name the proven outer type `{expected_outer}`"
            ));
        }
        if member.generic_diamond {
            if parameters.len().saturating_sub(1) != arguments.len() {
                return Err(format!(
                    "the proved generic member constructor at BCI {bci} declares {} source parameter(s), but reads {} source argument(s)",
                    parameters.len().saturating_sub(1),
                    arguments.len()
                ));
            }
            // The selected target proof includes a unique constructor Signature `(V)V` under
            // the member's class type parameter. Keeping the erased Object cast here would force
            // diamond inference to Object and make a typed `Generic<Integer>` return unspellable;
            // the source constructor Signature itself types each argument and the unique physical
            // constructor fixes the call binding.
            return Ok(arguments);
        }
        self.arguments_with_parameter_offset(descriptor, arguments, bci, 1)
    }

    fn arguments_with_parameter_offset(
        &self,
        descriptor: &str,
        arguments: Vec<Expr>,
        bci: u32,
        parameter_offset: usize,
    ) -> Result<Vec<Expr>, String> {
        let arguments = if parameter_offset == 0 {
            typed_arguments(descriptor, arguments)
        } else {
            typed_arguments_with_offset(descriptor, arguments, parameter_offset)
        };
        let Some((parameters, _)) = lambda::parse_method(descriptor) else {
            return Err(format!(
                "the invocation at BCI {bci} has an unreadable method descriptor"
            ));
        };
        if parameters.len().saturating_sub(parameter_offset) != arguments.len() {
            return Err(format!(
                "the invocation at BCI {bci} declares {} source parameter(s) after its {parameter_offset} physical prefix parameter(s), but reads {} source argument(s)",
                parameters.len().saturating_sub(parameter_offset),
                arguments.len()
            ));
        }
        let mut written = Vec::with_capacity(arguments.len());
        for (index, (argument, parameter)) in arguments
            .into_iter()
            .zip(parameters.into_iter().skip(parameter_offset))
            .enumerate()
        {
            written.push(self.invocation_argument(
                argument,
                &parameter,
                bci,
                &format!(
                    "the parameter {index} of the invocation at BCI {bci} is declared `{}`",
                    parameter.spell()
                ),
            )?);
        }
        Ok(written)
    }

    /// Writes one invocation argument under the parameter descriptor that selected the call.
    ///
    /// A call consumes a type in its own right.  Assignment and return positions may leave a
    /// widening or a reference value in the text, while source overload resolution only sees the
    /// pool descriptor if the argument states it.  A generic call and a method reference also need
    /// a target type even when their presented type already equals the descriptor.  Other reference
    /// relations are refused because a descriptor alone is not evidence for a runtime checkcast.
    fn invocation_argument(
        &self,
        argument: Expr,
        required: &Type,
        bci: u32,
        position: &str,
    ) -> Result<Expr, String> {
        if matches!(argument.kind, ExprKind::Null) {
            return match required {
                Type::Reference(_) => Ok(cast_argument(argument, required, bci)),
                _ => Err(format!(
                    "{position} requires primitive `{}` but the argument is null",
                    required.spell()
                )),
            };
        }
        let presented = argument.presented.clone();
        match required {
            Type::Reference(required_name) => {
                let Some(presented) = presented else {
                    return Err(format!(
                        "{position} has no reference type fact for the argument"
                    ));
                };
                let Type::Reference(presented_name) = presented else {
                    return Err(format!(
                        "{position} requires `{}` but the argument presents `{}`",
                        required.spell(),
                        presented.spell()
                    ));
                };
                let target_typed = matches!(
                    argument.kind,
                    ExprKind::Call { .. }
                        | ExprKind::Lambda { .. }
                        | ExprKind::MethodReference { .. }
                );
                let argument = if matches!(
                    argument.kind,
                    ExprKind::Lambda { .. } | ExprKind::MethodReference { .. }
                ) {
                    // A lambda or method reference has no target type of its own.  The verified
                    // factory descriptor is its first cast, even when the outer call consumes it
                    // as Object; `(Object) Probe::method` is not Java source.
                    let factory = argument.presented.clone().ok_or_else(|| {
                        format!(
                            "{position} has no verified functional factory type for the argument"
                        )
                    })?;
                    cast_argument(argument, &factory, bci)
                } else {
                    argument
                };
                if required_name == "java.lang.Object" {
                    if presented_name == "java.lang.Object" && !target_typed {
                        return Ok(argument);
                    }
                    return Ok(cast_argument(argument, required, bci));
                }
                if presented_name == *required_name {
                    if target_typed {
                        return Ok(cast_argument(argument, required, bci));
                    }
                    return Ok(argument);
                }
                if array_reference_widens(&presented_name, required_name) {
                    // Arrays have a small, closed set of reference supertypes that follows from
                    // their component shape alone. Keep the pool's selected parameter type in the
                    // source so overload resolution cannot retarget this call to a more specific
                    // overload. Javac may emit a redundant checkcast for the source cast, but
                    // the proved widening means that check cannot fail for this operand.
                    return Ok(cast_argument(argument, required, bci));
                }
                Err(format!(
                    "{position} presents `{presented_name}` but the invocation requires `{required_name}` and this layer has no safe reference conversion evidence"
                ))
            }
            _ => {
                let Some(presented) = presented else {
                    return Err(format!(
                        "{position} has no type fact for the primitive argument"
                    ));
                };
                if matches!(presented, Type::Reference(_)) {
                    return Err(format!(
                        "{position} requires `{}` but the argument presents `{}`",
                        required.spell(),
                        presented.spell()
                    ));
                }
                if presented == *required {
                    return Ok(argument);
                }
                if narrowed_constant(&argument, required) {
                    return match required {
                        Type::Byte | Type::Short => Ok(cast_argument(argument, required, bci)),
                        Type::Char => Ok(narrowed_literal(argument, required)),
                        _ => Ok(argument),
                    };
                }
                match conversion(&presented, required) {
                    Conversion::Widening => Ok(cast_argument(argument, required, bci)),
                    Conversion::Same => Ok(argument),
                    Conversion::Unspellable => Err(format!(
                        "{position} presents `{}` and this layer has no proven conversion to `{}`",
                        presented.spell(),
                        required.spell()
                    )),
                }
            }
        }
    }

    /// Renders one invocation: a verified synthetic accessor's field access, or the call itself.
    ///
    /// A read accessor's call site is a field read of the instance the site passed, and the field is
    /// the one the accessor's own verified body named. The node carries **both** original BCIs: the
    /// call site's as its own anchor and the field access inside the accessor's body as a derived
    /// one — which is exactly what A12 asks the segment table to keep (P3 2.2).
    ///
    /// `at` is the position the rendered expression is evaluated at, passed through to the receiver
    /// for the same reason [`Self::call_expr`] takes it: the instance the accessor reads is read
    /// where the expression is, not where the call site was.
    fn invoke_expr(
        &mut self,
        bci: u32,
        instruction: &SsaInstruction,
        target: &CallTarget,
        at: u32,
        depth: usize,
    ) -> Result<Expr, ValueRenderFailure> {
        // A call whose bytecode evaluated and popped a qualifier is the static call of P3 2c.31a, and
        // its text has to carry that evaluation: the accessor rule below reads the *arguments* a call
        // passes and spells the call site as the field access the callee performs, which would leave
        // the evaluated qualifier unwritten — the one thing the shape exists to avoid.
        if self.pops().qualifier_at(bci).is_some() {
            return self.call_expr(bci, instruction, target, at, depth);
        }
        if let accessor::Verdict::Accessor { evidence, shape } =
            accessor::verify(target, self.members, self.pool)
            && shape.kind == AccessorShape::FieldRead
        {
            let receiver = stack_operands(instruction)
                .first()
                .copied()
                .map(|(_, value)| value);
            match receiver.map(|receiver| self.render_value(receiver, at, depth + 1)) {
                Some(Ok(receiver)) => {
                    let origin = OriginSet::new(Origin::direct(bci))
                        .plus_derived(Origin::derived(shape.field_bci).in_method(&shape.method));
                    let field = Expr::new(
                        ExprKind::Field {
                            receiver: Box::new(receiver),
                            name: shape.name.clone(),
                        },
                        origin,
                    );
                    // The type the read presents is the descriptor of the field the accessor's own
                    // body named — the same evidence the record carries back. It is read out of the
                    // verdict before the decision takes the evidence for its own record.
                    let presented = match evidence
                        .field
                        .as_ref()
                        .and_then(|field| descriptor_type(&field.descriptor))
                    {
                        Some(ty) => field.presenting(ty),
                        None => field,
                    };
                    self.publish_accessor(bci, evidence, true, None);
                    return Ok(presented);
                }
                Some(Err(ValueRenderFailure::Stop(stop))) => {
                    return Err(ValueRenderFailure::Stop(stop));
                }
                other => {
                    let reason = match other {
                        Some(Err(ValueRenderFailure::Refusal(reason))) => reason,
                        _ => "the call reads no instance this run states".to_string(),
                    };
                    let refusal = Refusal::shape(
                        "jre_accessor_arguments",
                        format!(
                            "the instance the accessor call at BCI {bci} reads produces no expression this subset writes: {reason}"
                        ),
                    );
                    self.publish_accessor(bci, evidence, false, Some(&refusal));
                }
            }
        }
        self.call_expr(bci, instruction, target, at, depth)
    }

    /// Presents one verified write accessor's call site as the assignment it performs.
    ///
    /// The two values the call reads are rendered where the call site is, and a value this subset
    /// cannot write makes the call site a *refused* one — recorded — rather than a guessed
    /// assignment: the call it had is written instead.
    fn accessor_write(
        &mut self,
        at: u32,
        instruction: &SsaInstruction,
        target: &CallTarget,
        evidence: accessor::Evidence,
        shape: accessor::Shape,
    ) -> Result<(), StopReason> {
        let rendered = match stack_operands(instruction).as_slice() {
            // The second operand is the value the write stores. It is carried beside the two
            // rendered expressions so that the field's own descriptor can decide what that value is
            // spelled as — and so that a `boolean` field's proof reads the value the instruction
            // really read, exactly as a `putfield` hands its own stored value.
            [(_, receiver), (_, stored)] => {
                let (receiver, stored) = (*receiver, *stored);
                Some((
                    self.render_value(receiver, at, 0),
                    self.render_value(stored, at, 0),
                    stored,
                ))
            }
            _ => None,
        };
        match rendered {
            Some((Ok(receiver), Ok(value), stored)) => {
                // The write is one position whether the accessor's body reads a `putfield` or the
                // call site that used to spell it, so the value meets the type the field the
                // accessor's own body writes declares: a `char`/`byte`/`short` value under an `int`
                // descriptor states that conversion here too ([`Self::field_value`]).
                let value = match self.field_value(
                    evidence
                        .field
                        .as_ref()
                        .map(|field| field.descriptor.as_str()),
                    stored,
                    value,
                    at,
                ) {
                    Ok(value) => value,
                    Err(reason) => {
                        let refusal = Refusal::shape(
                            "jre_accessor_arguments",
                            format!(
                                "the write accessor call at BCI {at} was not presented: {reason}"
                            ),
                        );
                        self.publish_accessor(at, evidence, false, Some(&refusal));
                        return self.call_statement(at, instruction, target);
                    }
                };
                let origin = OriginSet::new(Origin::direct(at))
                    .plus_derived(Origin::derived(shape.field_bci).in_method(&shape.method));
                self.publish_accessor(at, evidence, true, None);
                self.push(Stmt::new(
                    StmtKind::FieldAssign {
                        receiver: Some(receiver),
                        name: shape.name.clone(),
                        op: AssignOp::Assign,
                        value,
                    },
                    origin,
                ))
            }
            Some((Err(ValueRenderFailure::Stop(stop)), _, _))
            | Some((_, Err(ValueRenderFailure::Stop(stop)), _)) => Err(stop),
            other => {
                let reason = match other {
                    Some((Err(ValueRenderFailure::Refusal(reason)), _, _))
                    | Some((_, Err(ValueRenderFailure::Refusal(reason)), _)) => reason,
                    _ => "the call does not read exactly the instance and the value it writes"
                        .to_string(),
                };
                let refusal = Refusal::shape(
                    "jre_accessor_arguments",
                    format!("the write accessor call at BCI {at} was not presented: {reason}"),
                );
                self.publish_accessor(at, evidence, false, Some(&refusal));
                self.call_statement(at, instruction, target)
            }
        }
    }

    /// The statement an invocation becomes when no rule presented it as a field access.
    ///
    /// The call writes a statement of its own, so it is evaluated where the instruction is: the
    /// evaluation context passed to [`Self::call_expr`] is the instruction's own BCI.
    fn call_statement(
        &mut self,
        at: u32,
        instruction: &SsaInstruction,
        target: &CallTarget,
    ) -> Result<(), StopReason> {
        let call = match self.call_expr(at, instruction, target, at, 0) {
            Ok(call) => call,
            Err(reason) => {
                let bcis = self.quoted_bcis(at);
                return self.fallback(bcis, &reason, at);
            }
        };
        self.push(Stmt::new(
            StmtKind::Expr(call),
            OriginSet::new(Origin::direct(at)),
        ))
    }

    /// Whether the `bridge@1` rule proved one instruction to be the erasure of the value it casts.
    fn bridge_owns(&self, bci: u32) -> bool {
        self.bridge.is_some_and(|plan| plan.owns(bci))
    }

    /// Renders one verified construction site as the `new` expression it stands for (P3 2.3).
    ///
    /// The arguments are written in the order the constructor call reads them — each an expression of
    /// its own, anchored where *it* was produced — and every BCI the site owns stays in the segment
    /// table as an anchor: one construction reaches several original instructions, and the table says
    /// so rather than keeping one of them.
    ///
    /// The arguments are read by the construction, which is written where the value at `at` is used:
    /// they are checked at `at`, not at the constructor's own BCI (`at` and `site.constructor` differ
    /// only where the `new` expression is evaluated later than the call that initialises it).
    fn new_expr(
        &mut self,
        site: &init::Site,
        at: u32,
        depth: usize,
    ) -> Result<Expr, ValueRenderFailure> {
        let Some(instruction) = self.instructions.get(&site.constructor).copied() else {
            return Err(format!(
                "no names record for the constructor call at BCI {} of the construction the value at BCI {at} comes from",
                site.constructor
            ).into());
        };
        let operands = stack_operands(instruction);
        let member = site.member_inner.as_ref();
        let qualifier = if let Some(member) = member {
            let Some(qualifier_read) = self.instructions.get(&member.qualifier).copied() else {
                return Err(format!(
                    "no names record exists for the proven member qualifier read at BCI {}",
                    member.qualifier
                )
                .into());
            };
            let Some((_, qualifier_value)) = qualifier_read.writes().first() else {
                return Err(format!(
                    "the proven member qualifier read at BCI {} does not produce a value",
                    member.qualifier
                )
                .into());
            };
            if operands.get(1).is_none() {
                return Err(format!(
                    "the member constructor at BCI {} has no physical outer-instance argument",
                    site.constructor
                )
                .into());
            }
            // The verified member site proved that the physical parameter and this read's checked
            // copy are the same SSA value. Rendering the read itself keeps its source BCI and avoids
            // re-entering `new_expr` through the member site's owned `dup` value.
            let qualifier = self.render_value(*qualifier_value, at, depth + 1)?;
            let expected_outer =
                spell_reference(&format!("L{};", member.outer)).ok_or_else(|| {
                    format!("the proven outer type `{}` cannot be spelled", member.outer)
                })?;
            if qualifier.presented != Some(Type::Reference(expected_outer.clone())) {
                return Err(format!(
                    "the physical outer-instance argument of the member constructor at BCI {} presents {:?}, not the proven enclosing type `{expected_outer}`",
                    site.constructor, qualifier.presented
                )
                .into());
            }
            Some(Box::new(qualifier))
        } else {
            None
        };
        let mut args = Vec::new();
        for (_, value) in operands.iter().skip(if member.is_some() { 2 } else { 1 }) {
            args.push(self.render_value(*value, at, depth + 1)?);
        }
        // The constructor's **own** descriptor types the arguments, exactly as a call's does: a
        // construction site writes the call the class file holds, and `new Res(arg0, 0)` for a
        // `Res(String, boolean)` constructor is a call the member's own signature refuses to compile.
        let args = match self.invoke_descriptor(site.constructor) {
            Some(descriptor) if member.is_some() => {
                self.arguments_after_first_parameter(&descriptor, args, site.constructor, member)?
            }
            Some(descriptor) => self.arguments(&descriptor, args, site.constructor)?,
            None if member.is_some() => {
                return Err(format!(
                    "the member constructor at BCI {} has no readable descriptor for its ordinary source arguments",
                    site.constructor
                )
                .into());
            }
            None => args,
        };
        let origin = site
            .owned
            .iter()
            .filter(|bci| **bci != site.constructor)
            .fold(
                OriginSet::new(Origin::direct(site.constructor)),
                |set, bci| set.plus_derived(Origin::derived(*bci)),
            );
        Ok(Expr::new(
            ExprKind::New {
                ty: spell_reference(&site.class).ok_or_else(|| {
                    format!(
                        "the construction at BCI {} names the class `{}`, which this layer cannot spell as a Java type",
                        site.constructor, site.class
                    )
                })?,
                qualifier,
                member_name: member.map(|member| member.simple_name.clone()),
                diamond: member.is_some_and(|member| member.generic_diamond),
                args,
            },
            origin,
        ))
    }

    /// Writes the call an instance initializer makes on its own uninitialized `this` (P3 2.3).
    ///
    /// The receiver is deliberately not rendered: it *is* what `super` and `this` name, and the value
    /// it holds is the frames' own `UninitializedThis` token rather than any expression. The arguments
    /// are written where the call is, in the order it reads them.
    fn constructor_call(
        &mut self,
        at: u32,
        instruction: &SsaInstruction,
        target: ConstructorTarget,
    ) -> Result<(), StopReason> {
        let mut args = Vec::new();
        for (_, value) in stack_operands(instruction).iter().skip(1) {
            match self.render_value(*value, at, 0) {
                Ok(arg) => args.push(arg),
                Err(reason) => {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(bcis, &reason, at);
                }
            }
        }
        // `super(…)`/`this(…)` is an invocation like any other: the constructor it names states the
        // parameter types its arguments are written under.
        let args = match self.invoke_descriptor(at) {
            Some(descriptor) => match self.arguments(&descriptor, args, at) {
                Ok(args) => args,
                Err(reason) => {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(bcis, &reason, at);
                }
            },
            None => args,
        };
        let mut origin = OriginSet::new(Origin::direct(at));
        if let Some((_, receiver)) = stack_operands(instruction).first()
            && let Definition::Instruction { bci, .. } = self.ssa.value(*receiver).def()
        {
            origin = origin.plus_derived(Origin::derived(*bci));
        }
        self.push(Stmt::new(
            StmtKind::ConstructorCall { target, args },
            origin,
        ))
    }

    /// The descriptor of the invocation one BCI holds, as the pool decoded it states it.
    ///
    /// `None` when the instruction is not an invocation of this decode: a site that names no
    /// `invoke*` states no descriptor, and a caller then leaves its arguments as they were rendered
    /// rather than guessing a signature (P3-R5's argument side).
    fn invoke_descriptor(&self, bci: u32) -> Option<String> {
        match self.operations.get(bci) {
            Some(Operation::Invoke(target)) => Some(target.descriptor().to_string()),
            _ => None,
        }
    }

    /// Writes one verified field write as the assignment it performs (P3 2.3).
    ///
    /// The receiver is the instance the instruction read — or, for a static field, the owner type,
    /// which is how a static write is spelled. Nothing is moved: the statement is written where the
    /// instruction runs, which is what keeps a constructor's initializer sequence in the order its
    /// own bytes have it.
    /// The statement one array element write is: `array[index] = value;` (P3 2b).
    ///
    /// The three values are the instruction's own operands, in the order JVMS 6.5 has the
    /// instruction read them — the array, the index and the value — and all three are evaluated
    /// where the write runs, so all three are rendered at `at`.
    ///
    /// The value meets the **element type the array's own type states** ([`array_element`]).
    /// Widening conversions and in-range constants keep the ordinary assignment spelling. For
    /// byte/char/short, an otherwise incompatible presented integer gets a cast only when the real
    /// store opcode matches that proven component. A `[Z` element keeps existing boolean proofs;
    /// for a presented B/C/S/I value, only its real `bastore` authorizes the JVM low-bit conversion.
    /// Neither the opcode alone nor an unknown array element type authorizes it.
    fn array_write(
        &mut self,
        at: u32,
        instruction: &SsaInstruction,
        stated: Option<&Type>,
    ) -> Result<(), StopReason> {
        if let Some(CompoundUpdate::Array {
            array,
            index,
            rhs,
            duplicate,
            read,
            add,
        }) = self.compounds.update_at(at)
        {
            let rendered = self
                .render_value(array, at, 0)
                .map(|array| array.derived_from(duplicate))
                .and_then(|array| {
                    self.render_value(index, at, 0)
                        .map(|index| (array, index.derived_from(duplicate)))
                })
                .and_then(|(array, index)| {
                    self.render_value(rhs, at, 0)
                        .map(|value| (array, index, value))
                });
            let (array, index, value) = match rendered {
                Ok((array, index, value)) if value.presented.as_ref() != Some(&Type::Boolean) => {
                    (array, index, value)
                }
                Ok(_) => {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(
                        bcis,
                        format!("the right side of the int array update at BCI {at} is presented as `boolean`"),
                        at,
                    );
                }
                Err(reason) => {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(bcis, &reason, at);
                }
            };
            let origin = compound_origin(at, duplicate, read, add);
            self.push(Stmt::new(
                StmtKind::IndexAssign {
                    array,
                    index,
                    op: AssignOp::Add,
                    value,
                },
                origin,
            ))?;
            return Ok(());
        }
        let operands = stack_operands(instruction);
        let [array, index, value] = operands.as_slice() else {
            let bcis = self.quoted_bcis(at);
            return self.fallback(
                bcis,
                format!(
                    "the array write at BCI {at} reads {} value(s), not an array, an index and a value",
                    operands.len()
                ),
                at,
            );
        };
        let (array_value, index_value, written) = (array.1, index.1, value.1);
        // These opcodes name an int verifier shape, not the element type: `bastore` in particular
        // is also the boolean store. Only the array's own facts may choose B/Z/C/S here.
        let element = if matches!(instruction.opcode(), 0x54..=0x56) {
            array_element(self.ssa, self.operations, array_value, None)
        } else {
            array_element(self.ssa, self.operations, array_value, stated)
        };
        let opcode_matches_proven_element = element
            .as_ref()
            .is_some_and(|ty| array_store_opcode_matches(ty, stated, instruction.opcode()));
        let narrow_store_matches = element.as_ref().is_some_and(|ty| {
            matches!(ty, Type::Byte | Type::Char | Type::Short) && opcode_matches_proven_element
        });
        if let Some(ty) = element.as_ref().filter(|_| !opcode_matches_proven_element) {
            let bcis = self.quoted_bcis(at);
            return self.fallback(
                bcis,
                format!(
                    "the array store at BCI {at} uses opcode 0x{:02x}, which does not match the proven `{}` component",
                    instruction.opcode(),
                    ty.spell()
                ),
                at,
            );
        }
        let array = match self.render_value(array_value, at, 0) {
            Ok(array) => array,
            Err(reason) => {
                let bcis = self.quoted_bcis(at);
                return self.fallback(bcis, &reason, at);
            }
        };
        let index = match self.render_value(index_value, at, 0) {
            Ok(index) => index,
            Err(reason) => {
                let bcis = self.quoted_bcis(at);
                return self.fallback(bcis, &reason, at);
            }
        };
        let value = match self.render_value(written, at, 0) {
            Ok(value) => value,
            Err(reason) => {
                let bcis = self.quoted_bcis(at);
                return self.fallback(bcis, &reason, at);
            }
        };
        let value = match element {
            Some(Type::Boolean) => {
                if self.boolean_literal(written) || self.boolean_proven(written, at) {
                    boolean_spelling(value)
                } else if instruction.opcode() == 0x54
                    && matches!(
                        value.presented.as_ref(),
                        Some(Type::Byte | Type::Char | Type::Short | Type::Int)
                    )
                {
                    integer_low_bit_boolean(value, at)
                } else {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(
                        bcis,
                        format!(
                            "the element written at BCI {at} is one of a `boolean[]`, and this layer has no evidence that the value it reads at BCI {at} is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, a read of a `boolean[]` proven `[Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the array's own type rejects"
                        ),
                        at,
                    );
                }
            }
            Some(ty) => {
                let position = format!(
                    "the element written at BCI {at} is one of the `{}` array it indexes",
                    ty.spell()
                );
                match meeting_position(value.clone(), &ty, &position, Widening::Position) {
                    Ok(value) => value,
                    Err(_reason)
                        if matches!(
                            value.presented.as_ref(),
                            Some(Type::Byte | Type::Char | Type::Short | Type::Int)
                        ) && narrow_store_matches =>
                    {
                        // Only a proven B/C/S component plus the matching real array-store opcode
                        // authorizes this cast. In particular, `bastore` on a proven `boolean[]`
                        // took its own boolean branch above and never arrives here.
                        cast_argument(value, &ty, at)
                    }
                    Err(reason) => {
                        let bcis = self.quoted_bcis(at);
                        return self.fallback(bcis, &reason, at);
                    }
                }
            }
            None if matches!(instruction.opcode(), 0x54..=0x56) => {
                let bcis = self.quoted_bcis(at);
                return self.fallback(
                    bcis,
                    format!(
                        "the int-shaped array store at BCI {at} does not prove whether its component is `boolean`, `byte`, `char` or `short`, so this layer cannot choose the write conversion"
                    ),
                    at,
                );
            }
            // The element type is not stated — an array whose own type the frames do not name and
            // whose opcode states none (`aastore`): nothing is claimed, so nothing is converted
            // either, and the value is written as it was rendered.
            None => value,
        };
        self.push(Stmt::new(
            StmtKind::IndexAssign {
                array,
                index,
                op: AssignOp::Assign,
                value,
            },
            OriginSet::new(Origin::direct(at)),
        ))
    }

    /// The value one proved array initializer stores into its component. `at` is where the element
    /// expression is evaluated in the recovered text; `store_bci` is the paired physical store and
    /// anchors any conversion performed for that assignment. Reference compatibility stays
    /// deliberately closed: without a hierarchy fact, only an exact component type, `Object`, or
    /// `null` proves an `aastore` cannot gain or lose an `ArrayStoreException`.
    fn array_initializer_element(
        &self,
        value_id: ValueId,
        value: Expr,
        component: &Type,
        at: u32,
        store_bci: u32,
    ) -> Result<Expr, String> {
        if *component == Type::Boolean {
            if self.boolean_literal(value_id) || self.boolean_proven(value_id, at) {
                return Ok(boolean_spelling(value));
            }
            let real_boolean_store = self
                .instructions
                .get(&store_bci)
                .is_some_and(|instruction| {
                    instruction.opcode() == 0x54
                        && matches!(
                            self.operations.get(store_bci),
                            Some(Operation::ArrayStore {
                                element: Some(Type::Int)
                            })
                        )
                });
            if real_boolean_store
                && matches!(
                    value.presented.as_ref(),
                    Some(Type::Byte | Type::Char | Type::Short | Type::Int)
                )
            {
                return Ok(integer_low_bit_boolean(value, store_bci));
            }
            return Err(format!(
                "the array initializer element evaluated at BCI {at} and paired with array store BCI {store_bci} has no boolean evidence or eligible integer presentation for a proven `bastore`, so it cannot be assigned to `boolean`"
            ));
        }
        if let Type::Reference(expected) = component {
            if matches!(value.kind, ExprKind::Null) {
                return Ok(value);
            }
            let Some(Type::Reference(actual)) = value.presented.as_ref() else {
                return Err(format!(
                    "the array initializer element at BCI {at} has no reference type this run can prove assignable to `{}`",
                    component.spell()
                ));
            };
            let is_object = matches!(expected.as_str(), "Object" | "java.lang.Object");
            if actual == expected || is_object {
                return Ok(value);
            }
            return Err(format!(
                "the array initializer element at BCI {at} is presented as `{}`, while the array component is `{}`; without a hierarchy fact this `aastore` cannot be rewritten as a Java initializer without changing its compatibility or exception behavior",
                actual, expected
            ));
        }
        if value.presented.is_none() {
            return Err(format!(
                "the array initializer element at BCI {at} has no primitive type this run can prove assignable to `{}`",
                component.spell()
            ));
        }
        meeting_position(
            value,
            component,
            &format!(
                "the array initializer element at BCI {at} fills a `{}` component",
                component.spell()
            ),
            Widening::Position,
        )
    }

    fn field_write(
        &mut self,
        at: u32,
        evidence: &field::Evidence,
        shape: &field::Shape,
    ) -> Result<(), StopReason> {
        if let Some(CompoundUpdate::Field {
            receiver,
            rhs,
            duplicate,
            read,
            add,
        }) = self.compounds.update_at(at)
        {
            let rendered = self
                .render_value(receiver, at, 0)
                .map(|receiver| receiver.derived_from(duplicate))
                .and_then(|receiver| self.render_value(rhs, at, 0).map(|value| (receiver, value)));
            let (receiver, value) = match rendered {
                Ok((receiver, value)) if value.presented.as_ref() != Some(&Type::Boolean) => {
                    (receiver, value)
                }
                Ok(_) => {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(
                        bcis,
                        format!("the right side of the int field update at BCI {at} is presented as `boolean`"),
                        at,
                    );
                }
                Err(reason) => {
                    let bcis = self.quoted_bcis(at);
                    return self.fallback(bcis, &reason, at);
                }
            };
            self.push(Stmt::new(
                StmtKind::FieldAssign {
                    receiver: Some(receiver),
                    name: evidence.name.clone(),
                    op: AssignOp::Add,
                    value,
                },
                compound_origin(at, duplicate, read, add),
            ))?;
            return Ok(());
        }
        let receiver = if self.fields.simple_static_final_write(at) {
            None
        } else {
            Some(match shape.receiver {
                Some(value) => match self.render_value(value, at, 0) {
                    Ok(receiver) => receiver,
                    Err(reason) => {
                        let bcis = self.quoted_bcis(at);
                        return self.fallback(bcis, &reason, at);
                    }
                },
                None => match spell_reference(&evidence.owner) {
                    Some(owner) => Expr::direct(ExprKind::Path(owner), at),
                    // A static write's receiver is the owner type itself: a name this layer cannot
                    // spell as a Java type has no receiver to be, and the write is quoted.
                    None => {
                        let bcis = self.quoted_bcis(at);
                        return self.fallback(
                            bcis,
                            format!(
                                "the field write at BCI {at} names the owner `{}`, which this layer cannot spell as a Java type",
                                evidence.owner
                            ),
                            at,
                        );
                    }
                },
            })
        };
        let Some(stored) = shape.value else {
            return self.fallback(
                self.quoted_bcis(at),
                format!("the field write at BCI {at} reads no value to store"),
                at,
            );
        };
        let value = match self.render_value(stored, at, 0) {
            Ok(value) => value,
            Err(reason) => {
                let bcis = self.quoted_bcis(at);
                return self.fallback(bcis, &reason, at);
            }
        };
        let value = match self.field_value(Some(&evidence.descriptor), stored, value, at) {
            Ok(value) => value,
            Err(reason) => {
                let bcis = self.quoted_bcis(at);
                return self.fallback(bcis, &reason, at);
            }
        };
        self.push(Stmt::new(
            StmtKind::FieldAssign {
                receiver,
                name: evidence.name.clone(),
                op: AssignOp::Assign,
                value,
            },
            OriginSet::new(Origin::direct(at)),
        ))
    }

    /// The value one field write is spelled as, from the field's **own descriptor**.
    ///
    /// A field write is a position like every other one this rule reads: the descriptor the pool
    /// states for the member says what the value has to be presented as — the same reading a read of
    /// the field presents — so a value the write itself widens is written as it was rendered, a `char`
    /// field's in-range `int` constant becomes the character it stands for. If the JVM's stack
    /// value is an otherwise unrepresentable int-shaped primitive, a proven B/C/S field position
    /// adds the existing [`ExprKind::Cast`] at this write; every other no-conversion case is still
    /// refused ([`meeting_position`]).
    ///
    /// A `boolean` field is the boolean rules' position, exactly as a variable the plan decided
    /// `boolean` is: the frames state one slot shape for a `boolean` and an `int`, so the `0`/`1`
    /// literal is spelled `false`/`true` where the evidence proves it (the same
    /// [`Self::boolean_literal`]/[`Self::boolean_proven`] proof the `Z` return and the
    /// boolean-decided variable read). A value without that proof is accepted here only when its
    /// rendered expression states an int-sized primitive; the `Z` field store consumes its low bit,
    /// which this position spells as `value % 2 != 0`. Other values remain refused. `stored` is the
    /// value the writing instruction reads, which is the value that proof or presentation fact is
    /// about.
    ///
    /// A descriptor this layer cannot spell — and a write whose field no rule stated a descriptor
    /// for — states no requirement, and the value is written as it was rendered.
    fn field_value(
        &self,
        descriptor: Option<&str>,
        stored: ValueId,
        value: Expr,
        at: u32,
    ) -> Result<Expr, String> {
        let Some(ty) = descriptor.and_then(descriptor_type) else {
            return Ok(value);
        };
        if ty == Type::Boolean {
            if self.boolean_literal(stored) || self.boolean_proven(stored, at) {
                return Ok(boolean_spelling(value));
            }
            if matches!(
                value.presented.as_ref(),
                Some(Type::Byte | Type::Char | Type::Short | Type::Int)
            ) {
                return Ok(integer_low_bit_boolean(value, at));
            }
            return Err(format!(
                "the field written at BCI {at} is declared `boolean`, and this layer has no evidence that the value it reads at BCI {at} is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the field's own type rejects"
            ));
        }
        let position = format!("the field written at BCI {at} is declared `{}`", ty.spell());
        match meeting_position(value.clone(), &ty, &position, Widening::Position) {
            Ok(value) => Ok(value),
            Err(_)
                if matches!(ty, Type::Byte | Type::Char | Type::Short)
                    && matches!(
                        value.presented.as_ref(),
                        Some(Type::Int | Type::Byte | Type::Char | Type::Short)
                    ) =>
            {
                Ok(cast_argument(value, &ty, at))
            }
            Err(reason) => Err(reason),
        }
    }

    /// Whether the value one instruction produced is read by another this build **writes it into**.
    ///
    /// This is the question the call arm asks before it writes a statement of its own, and the one
    /// the array instructions ask before they write none (P3 2b): a value whose text lands where the
    /// value is consumed may be produced by an instruction that writes nothing here, and one whose
    /// value no reader writes must name itself.
    ///
    /// It is deliberately narrower than [`Self::value_is_consumed`]: a **dynamic site** is not a
    /// reader here, because a site that is refused writes the site's own BCIs and not the invocation
    /// that produced the value it captured — so the instruction that produced that value is the only
    /// place left where the invocation can be written.
    ///
    /// The reader must be one this build **writes the value into**, which is what
    /// [`Self::renders_the_value_it_reads`] decides. Treating an instruction that is quoted instead
    /// of presented as a reader loses the invocation altogether — the call writes nothing because
    /// "something reads it", and the reader writes nothing because it is quoted bytecode. That is
    /// the shape P3 2.3 §0 measured before this guard existed: an effect silently dropped, which is
    /// worse than one written twice, because nothing in the artifact says it happened.
    fn produced_value_reaches_a_reader(&self, instruction: &SsaInstruction) -> bool {
        instruction
            .writes()
            .iter()
            .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
            .any(|(_, value)| {
                let value = *value;
                self.ssa.blocks().iter().any(|block| {
                    block.instructions().iter().any(|reader| {
                        reader.reads().iter().any(|(_, read)| *read == value)
                            && self.renders_the_value_it_reads(reader.bci())
                    })
                })
            })
    }

    /// Whether the instruction at one BCI writes the values it reads into the text this build
    /// produces.
    ///
    /// Every operation that is *presented* renders its operands as part of what it becomes — a
    /// store's initialiser, a call's receiver and arguments, a `return`'s value, a condition, a
    /// switch's selector, an arithmetic — and every operation that is *quoted* writes no value at
    /// all. Which of the two an instruction is, is a fact about what the rules of this build claim:
    /// a `checkcast` is rendered only where `bridge@1` proved it is an erasure, a field access only
    /// where the `field@1` rule claimed it, an array read only where `enumswitch@1` claimed it.
    /// Everything else is a stated gap, and a value whose only reader is a stated gap has no place
    /// in the body: the instruction that produced it must write it itself.
    fn renders_the_value_it_reads(&self, bci: u32) -> bool {
        if self.compounds.owns_copy(bci) {
            return true;
        }
        match self.operations.get(bci) {
            Some(
                Operation::Store { .. }
                | Operation::Invoke(_)
                | Operation::Return
                | Operation::Throw
                | Operation::Comparison { .. }
                | Operation::Switch { .. }
                | Operation::Arithmetic { .. }
                | Operation::Shift { .. }
                | Operation::Bitwise { .. }
                | Operation::Negate
                | Operation::PrimitiveConversion { .. }
                | Operation::InstanceOf { .. },
            ) => true,
            Some(Operation::NumericComparison { .. }) => self
                .instructions
                .get(&bci)
                .is_some_and(|instruction| self.numeric_comparison_reader(instruction).is_ok()),
            // An ordinary checkcast writes a real expression at its final consumer. A bridge cast
            // is also a reader from the value-flow perspective: its erasure is omitted from the
            // text, but the value still reaches the consumer that writes it.
            Some(Operation::CheckCast { .. }) => true,
            // A copy that states a chained assignment is a reader too, and for the same reason a
            // store is one: the value it takes off the stack is written by the first of the two
            // stores that follow it (P3 2c.14). Not being one here would let the call that produced
            // the value write a statement of its own *and* be rendered inside the store — the same
            // effect written twice, which is exactly what the copy's shape exists to avoid.
            Some(Operation::Duplicate) => self.chained_pair(bci).is_some(),
            // A field access and an array read are readers exactly where their own rules claimed
            // them (P3 2.3): a claimed access renders the value it reads into its text, and one no
            // rule claimed is quoted and writes nothing.
            Some(Operation::Field { .. }) => self.fields.owns(bci),
            // An array access renders the values it reads into its own text (P3 2b): the read its
            // array and its index, the write its array, its index and its value, the creation its
            // lengths. So a value whose only reader is one of them is written after all — the
            // dispatch-table read of an enum `switch` included, whose operands the selector's own
            // text renders (P3 2.3).
            Some(
                Operation::ArrayLoad
                | Operation::ArrayElementLoad { .. }
                | Operation::ArrayStore { .. }
                | Operation::ArrayLength
                | Operation::NewArray { .. },
            ) => true,
            _ => false,
        }
    }

    /// The BCIs to quote for one instruction this build could not write: the instruction itself, and
    /// every invocation whose own statement was deferred to a reader that did not end up writing the
    /// value it read (P3 2.3 §0).
    fn quoted_bcis(&self, at: u32) -> Vec<u32> {
        self.quoted_bcis_with_budget(at, None)
            .expect("an unbudgeted quote walk cannot stop")
            .0
    }

    /// The same quote walk with an optional cancellation/deadline checkpoint. Ordinary callers keep
    /// the historical read-only path; binding refusal supplies the build budget so a large deferred
    /// chain observes cancellation while it is being traversed. One `seen` set is shared by all
    /// stack operands, so a shared SSA value is expanded once in its first operand order.
    fn quoted_bcis_with_budget(
        &self,
        at: u32,
        budget: Option<&Budget>,
    ) -> Result<(Vec<u32>, usize), StopReason> {
        let mut bcis = vec![at];
        let mut seen = BTreeSet::new();
        let mut stable_loads = BTreeSet::new();
        // The `pop`s this instruction's own text was going to carry (P3 2c.31): a call that refused
        // takes the qualifier it would have written and the discard its statement would have been, so
        // the quote names them beside itself instead of dropping the evaluations they stood for.
        for pop in self.pops().pops_of(at) {
            if !bcis.contains(&pop) {
                bcis.push(pop);
            }
        }
        self.quoted_qualifier_producer(at, at, &mut bcis, &mut seen, &mut stable_loads, 0, budget)?;
        if let Some(instruction) = self.instructions.get(&at).copied() {
            for (_, value) in stack_operands(instruction) {
                self.deferred_producers(
                    value,
                    at,
                    &mut bcis,
                    &mut seen,
                    &mut stable_loads,
                    0,
                    budget,
                )?;
            }
        }
        if bcis
            .iter()
            .any(|bci| matches!(self.operations.get(*bci), Some(Operation::Invoke(_))))
        {
            append_quoted_stable_loads(&mut bcis, stable_loads, budget)?;
        }
        Ok((bcis, seen.len()))
    }

    /// Adds the source of a static call's popped expression qualifier to the same bounded quote
    /// walk that owns the call and its `pop`. The ordinary producer walk intentionally omits a
    /// stable local read because its value is already named by the method declaration; a rejected
    /// qualifier still needs its exact producer BCI alongside the call/pop/consumer relationship.
    fn quoted_qualifier_producer(
        &self,
        call: u32,
        reader: u32,
        into: &mut Vec<u32>,
        seen: &mut BTreeSet<ValueId>,
        stable_loads: &mut BTreeSet<u32>,
        depth: usize,
        budget: Option<&Budget>,
    ) -> Result<(), StopReason> {
        let Some((pop, value)) = self.pops().qualifier_at(call) else {
            return Ok(());
        };
        if !into.contains(&pop) {
            into.push(pop);
        }
        self.deferred_producers(value, reader, into, seen, stable_loads, depth + 1, budget)?;
        if depth <= MAX_VALUE_DEPTH
            && let Definition::Instruction { bci, .. } = self.ssa.value(value).def()
            && !into.contains(bci)
        {
            into.push(*bci);
        }
        Ok(())
    }

    /// The BCIs of the instructions behind one value that no statement wrote, every one of them read
    /// by the instruction at `reader`.
    ///
    /// A call whose value reaches a reader writes no statement of its own
    /// ([`Self::produced_value_reaches_a_reader`]), and the reader *usually* writes the value: that is
    /// the whole reason for deferring. When the reader turns out not to be able to — its own
    /// operands may still be bytecode this layer cannot present — the invocation has no place in the
    /// artifact unless a quote names it, which is what this walk collects: through producers this
    /// build did present, the deferred producers of the values they read. A producer that is a
    /// deferred invocation is quoted, and its receiver/arguments are walked as well so nested
    /// deferred effects remain accounted for.
    ///
    /// A **load** belongs to the same class for the same reason: it writes no statement either, its
    /// text lands where its value is consumed, and where the slot no longer holds what it read at
    /// `reader` that text is refused (P3 1.3d) — so the read itself is what the quote has to name,
    /// and the read is named rather than dropped from the answer.
    ///
    /// A **claimed field read** belongs here too: it writes no statement of its own either — a
    /// claimed read is a value whose text lands where the value is consumed — so a reader that
    /// refuses must name it, or the answer drops an effect the bytecode really performs (running a
    /// static initializer, or the `NullPointerException` an instance read can throw). The walk names
    /// it and keeps descending, because the receiver's own lost producers are the read's producers
    /// as well — that is how `External.holder.value` is answered for down to the `getstatic` behind
    /// the `getfield` (P3-R9).
    ///
    /// An **array read or creation** (P3 2b) belongs to the same class for the same reason: it
    /// writes no statement of its own either, and a reader that refuses must name it, or the answer
    /// drops an effect the bytecode really performs — the allocation, or the
    /// `NullPointerException`/`ArrayIndexOutOfBoundsException` a read can throw. The walk keeps
    /// descending, because the array and the index are values whose own producers are behind the
    /// same refusal.
    ///
    /// A Class literal has no statement of its own either; when its consumer cannot be written, the
    /// quote names the `ldc` producer beside the consuming instruction so the source map retains
    /// both ends of that refused value.
    ///
    /// What is deliberately *not* here: an `invokedynamic` site's linkage and an `enumswitch@1`
    /// dispatch-table read stay unnamed when their consumer refuses. Naming them would state a
    /// linkage this walk does not own, and no rule of this change extends the rule to them.
    ///
    /// Every judgement in the walk is taken at the position the walk **started** from: the renderer
    /// that refused judged the loads it could not write at the consumer's use point, so the quote
    /// has to name the same read the refusal was about — a recursion that re-narrowed the use point
    /// to each intermediate instruction would judge a load where the artifact does not evaluate it
    /// and stay silent about the read it actually refused (P3-R8, and P3-R1's `post` for the
    /// direct-operand case). `seen` is shared across all operands of this quote, so a common SSA
    /// value in a DAG is expanded in its first operand order only; the BCI list remains ordered by
    /// the first traversal that discovered each source identity.
    fn deferred_producers(
        &self,
        value: ValueId,
        reader: u32,
        into: &mut Vec<u32>,
        seen: &mut BTreeSet<ValueId>,
        stable_loads: &mut BTreeSet<u32>,
        depth: usize,
        budget: Option<&Budget>,
    ) -> Result<(), StopReason> {
        if let Some(budget) = budget {
            poll(budget, Some(reader))?;
        }
        if depth > MAX_VALUE_DEPTH {
            return Ok(());
        }
        if !seen.insert(value) {
            return Ok(());
        }
        let Definition::Instruction { bci, .. } = self.ssa.value(value).def() else {
            return Ok(());
        };
        let bci = *bci;
        if let Some((_, deferred)) = self
            .deferred
            .iter()
            .find(|(deferred_value, _)| *deferred_value == value)
        {
            let first_visit = !into.contains(deferred);
            if first_visit {
                into.push(*deferred);
                if let Some(site) = self.sites.site_of(*deferred) {
                    for bci in &site.owned {
                        if !into.contains(bci) {
                            into.push(*bci);
                        }
                    }
                }
            }
            // The call this walk stopped at may be one whose bytecode evaluated and popped a qualifier
            // (P3 2c.31a): the text that would have written that evaluation is the *reader*'s, which is
            // the very text this quote takes the place of, so the `pop` is named here too.
            for pop in self.pops().pops_of(*deferred) {
                if !into.contains(&pop) {
                    into.push(pop);
                }
            }
            self.quoted_qualifier_producer(
                *deferred,
                reader,
                into,
                seen,
                stable_loads,
                depth + 1,
                budget,
            )?;
            if first_visit && let Some(instruction) = self.instructions.get(deferred).copied() {
                for (_, operand) in stack_operands(instruction) {
                    self.deferred_producers(
                        operand,
                        reader,
                        into,
                        seen,
                        stable_loads,
                        depth + 1,
                        budget,
                    )?;
                }
            }
            return Ok(());
        }
        if let Some(instruction) = self.instructions.get(&bci).copied() {
            if let Some(Operation::Load { slot }) = self.operations.get(bci)
                && let Some(read) = local_read(instruction, *slot)
            {
                if self.slot_name_denotes_the_same_value(*slot, read, reader) {
                    stable_loads.insert(bci);
                } else if !into.contains(&bci) {
                    into.push(bci);
                }
                return Ok(());
            }
            // A refused downstream consumer still has to name every ordinary checkcast in the
            // value chain. The cast that reaches the consumer is not itself a producer that writes
            // a statement, but its runtime check is an effect the quote must retain; walking its
            // operand reaches an earlier nested check and then any deferred invocation behind it.
            if matches!(self.operations.get(bci), Some(Operation::CheckCast { .. }))
                && !self.bridge_owns(bci)
                && !into.contains(&bci)
            {
                into.push(bci);
            }
            // A claimed read is named like a deferred call, and the walk continues through its
            // operands because their producers are behind the same refusal.
            let writes_no_statement = self
                .fields
                .claim(bci)
                .is_some_and(|(_, shape)| !shape.writes())
                || match self.operations.get(bci) {
                    // Every array instruction except the dispatch-table read of an enum `switch`:
                    // that one stays unnamed, because naming it would state a linkage this walk does
                    // not own (P3 2.3).
                    Some(Operation::ArrayLoad) => !self.enums.owns(bci),
                    Some(
                        Operation::ArrayElementLoad { .. }
                        | Operation::ArrayLength
                        | Operation::NewArray { .. },
                    ) => true,
                    // A Class literal has no statement of its own; if the consumer refuses (for
                    // example, because the type path is shadowed), the quote must keep the `ldc`
                    // producer beside the consumer it fed.
                    Some(Operation::Push(ConstantValue::Class { .. })) => true,
                    // Every floating constant is in that same class — its text lands where its
                    // value is consumed, so a refused consumer must name the constant beside
                    // itself, whatever its own presentation would have done with the bits.
                    Some(
                        Operation::Push(ConstantValue::Float(_))
                        | Operation::Push(ConstantValue::Double(_)),
                    ) => true,
                    Some(Operation::NumericComparison { .. }) => true,
                    Some(Operation::Bitwise { .. }) => true,
                    Some(Operation::PrimitiveConversion { .. }) => true,
                    Some(Operation::InstanceOf { .. }) => true,
                    _ => false,
                };
            if writes_no_statement && !into.contains(&bci) {
                into.push(bci);
            }
            if let Some(initializer) = self.array_initializers.at_allocation(bci) {
                for source in &initializer.sources {
                    if into.contains(source) {
                        continue;
                    }
                    if let Some(budget) = budget {
                        poll(budget, Some(*source))?;
                    }
                    into.push(*source);
                }
            }
            for (_, operand) in stack_operands(instruction) {
                // The operand of *this* instruction is read where the reader the walk started from
                // reads it: the walk descends through instructions, but the use point does not move
                // with it — the renderer's checks are taken at the consumer's position.
                self.deferred_producers(
                    operand,
                    reader,
                    into,
                    seen,
                    stable_loads,
                    depth + 1,
                    budget,
                )?;
            }
        }
        Ok(())
    }

    /// Every BCI one region covers, in method order: the bytecode a quote for that region has to
    /// name.
    ///
    /// A region the walk refuses *after* its condition could not be read still covers its arms'
    /// blocks, and those instructions produce no statements of their own — so a quote that named only
    /// the branch would lose them exactly as a refused reader loses a call (P3 2.3 §0). The quote
    /// names every instruction the region would have presented.
    fn region_bcis(&self, region: &Region) -> Vec<u32> {
        let mut bcis: Vec<u32> = Vec::new();
        for block in region.blocks() {
            for bci in self.covered_bcis(block) {
                if !bcis.contains(&bci) {
                    bcis.push(bci);
                }
            }
        }
        bcis
    }

    /// Whether one clause's body walk left the body's own instructions unpresented, and the
    /// quote — its message and every instruction start of the body's own range — that states it
    /// when it did.
    ///
    /// The clause's parameter store is the header's own declaration, a `goto` is control the
    /// structure itself states, and the builder's other skip rules own what they proved — those
    /// are the instructions an empty clause body may legitimately hide. Anything else the body's
    /// region holds had to become a statement of the clause; when the walk wrote none, the text
    /// would present a handler that runs nothing, so the body is rewritten as the quote of the
    /// bytecode its own range holds (P3 2.2: the header stays, the body is a BCI reference, and
    /// no statement is dropped without a reference). A body whose walk wrote any statement is
    /// not an empty body and is left as it was walked.
    fn unpresented_clause_body(
        &mut self,
        body: &Region,
        handler_bci: u32,
    ) -> Result<Option<(String, Vec<u32>)>, StopReason> {
        // This is a second read of ownership, independent of the arm walk: it classifies each
        // instruction against the proofs that may legitimately hide it. Bill that inspection here
        // just as [`Self::fallback_instruction_bcis`] bills its own ownership read.
        let mut held: Vec<u32> = Vec::new();
        let mut held_set = BTreeSet::new();
        for block in body.blocks() {
            let at = block.bci();
            poll(self.budget, Some(at))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(at),
            )?;
            if let Some(ssa) = self.ssa.block(block) {
                for instruction in ssa.instructions() {
                    self.append_clause_bci(instruction.bci(), &mut held, &mut held_set)?;
                }
            } else {
                let mut canonical = None;
                for candidate in self.canonical.blocks() {
                    let candidate_at = candidate.id().bci();
                    poll(self.budget, Some(candidate_at))?;
                    charge(
                        self.budget,
                        CountedBudgetDimension::AnalysisSteps,
                        1,
                        Some(candidate_at),
                    )?;
                    if candidate.id() == block {
                        canonical = Some(candidate);
                        break;
                    }
                }
                match canonical {
                    Some(canonical) if let [start] = canonical.blocks() => {
                        let begin = self
                            .code
                            .instructions
                            .partition_point(|instruction| instruction.bci < *start);
                        for instruction in self.code.instructions[begin..]
                            .iter()
                            .take_while(|instruction| instruction.bci < canonical.end_bci())
                        {
                            self.append_clause_bci(instruction.bci, &mut held, &mut held_set)?;
                        }
                    }
                    Some(canonical) => {
                        for &bci in canonical.blocks() {
                            self.append_clause_bci(bci, &mut held, &mut held_set)?;
                        }
                    }
                    None => {}
                }
            }
        }
        if held.is_empty() {
            return Ok(None);
        }
        let mut unpresented = false;
        for bci in &held {
            poll(self.budget, Some(*bci))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(*bci),
            )?;
            let owned = self.array_initializers.owns(*bci)
                || self.chains.owns(*bci)
                || self.sites.owns(*bci)
                || self.settled.contains(bci)
                || self.compounds.owns_copy(*bci);
            if !(owned
                || (*bci == handler_bci && self.clause_parameters.contains(&handler_bci))
                || matches!(self.operations.get(*bci), Some(Operation::Transfer)))
            {
                unpresented = true;
                break;
            }
        }
        Ok(unpresented.then(|| {
            (
                "the handler body proved no statement beside its header, so the clause quotes the bytecode its own range holds".to_string(),
                held,
            )
        }))
    }

    fn append_clause_bci(
        &mut self,
        bci: u32,
        held: &mut Vec<u32>,
        held_set: &mut BTreeSet<u32>,
    ) -> Result<(), StopReason> {
        poll(self.budget, Some(bci))?;
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(bci),
        )?;
        if held_set.insert(bci) {
            held.push(bci);
        }
        Ok(())
    }

    /// The quote for one region whose test this build could not read: every BCI the region covers,
    /// and the invocations deferred to a reader inside it that did not end up writing them.
    fn region_quote(&self, region: &Region, test_bci: u32) -> Vec<u32> {
        let mut bcis = self.region_bcis(region);
        for extra in self.quoted_bcis(test_bci) {
            if !bcis.contains(&extra) {
                bcis.push(extra);
            }
        }
        bcis
    }

    /// The bytecodes the unproved short-circuit node must keep visible. A full refusal includes
    /// the prefix and every instruction before the outer branch, since an unproved test may have
    /// independent effects or value producers there. Every instruction in the consumer block
    /// stays quoted too: a suffix may depend on state the quoted putstatic did not establish.
    fn short_circuit_value_quote(
        &self,
        region: &Region,
        outer_branch_bci: u32,
        include_prefix: bool,
    ) -> Vec<u32> {
        let Region::ShortCircuitValue {
            prefix,
            tests,
            gateways,
            true_producer,
            false_producer,
            consumer,
            ..
        } = region
        else {
            return Vec::new();
        };
        let mut bcis = Vec::new();
        let mut blocks: Vec<_> = if include_prefix {
            prefix.iter().map(|block| (block, 0)).collect()
        } else {
            Vec::new()
        };
        blocks.extend(tests.iter().map(|(block, _)| {
            (
                block,
                if block == &tests[0].0 {
                    outer_branch_bci
                } else {
                    0
                },
            )
        }));
        blocks.extend(gateways.iter().map(|(block, _)| (block, 0)));
        blocks.extend([(true_producer, 0), (false_producer, 0), (consumer, 0)]);
        for (block, lower) in blocks {
            if let Some(ssa_block) = self.ssa.block(block) {
                for instruction in ssa_block.instructions() {
                    if instruction.bci() >= lower && !bcis.contains(&instruction.bci()) {
                        bcis.push(instruction.bci());
                    }
                }
            }
            let Some(canonical_block) = self
                .canonical
                .blocks()
                .iter()
                .find(|candidate| candidate.id() == block)
            else {
                continue;
            };
            let starts = canonical_block.blocks();
            for (index, start) in starts.iter().enumerate() {
                let upper = starts
                    .get(index + 1)
                    .copied()
                    .unwrap_or(canonical_block.end_bci());
                for (bci, _) in self.operations.iter() {
                    if *bci >= (*start).max(lower) && *bci < upper && !bcis.contains(bci) {
                        bcis.push(*bci);
                    }
                }
            }
        }
        for (_, test_bci) in tests {
            let test_bci = *test_bci;
            for bci in self.quoted_bcis(test_bci) {
                if !bcis.contains(&bci) {
                    bcis.push(bci);
                }
            }
        }
        bcis.sort_unstable();
        bcis
    }

    /// Renders one verified concatenation chain as the parts its `+` expression is made of.
    ///
    /// The parts are written in the order the chain's `append` calls read them, and each one is an
    /// expression of its own — anchored where *it* was produced — so an operand that calls something
    /// calls it once, in the bytecode's order. The `toString` the chain ends in anchors the
    /// expression, and every BCI the chain owns is kept as a derived anchor of it: one concatenation
    /// reaches many original instructions, and the table says so rather than keeping one of them
    /// (P3 2.2, the source-map requirement).
    ///
    /// The parts are a **sequence** ([`ExprKind::Concat`]) and not a nested `Binary` fold, which is
    /// what makes one `append` one entry of a `Vec` rather than one more level of a `Box` chain: the
    /// construction, the printing, the cloning and the release of this node are bounded by the parts'
    /// own nesting (each part's expression is rendered under the value-nesting bound), never by the
    /// chain's length. The fold is what this method replaced — it built the tree the printer then had
    /// to walk level by level, so a chain long enough exhausted the process stack while every
    /// individual value was well inside `MAX_VALUE_DEPTH`.
    ///
    /// Each part keeps its own conversion, and the conversion is applied here, where the `append`'s
    /// parameter type and the value's own evidence meet:
    ///
    /// * every accepted overload's text in a string context is `String.valueOf`'s, which is the
    ///   value's own text — so nothing about the part changes except the empty string the printer
    ///   starts the chain from when the first part is not already a `String`;
    /// * `boolean` is the exception: `append(true)` is `iconst_1`, and only the descriptor's `Z` says
    ///   that this `1` is a boolean, so a part whose parameter is `boolean` is spelled as the boolean
    ///   it is (`boolean_spelling`) — the same reading the call-argument, `return` and store paths
    ///   make of a `Z` position. A `boolean` part whose value has **no** boolean evidence is refused
    ///   rather than spelled as the integer it would otherwise look like.
    ///
    /// `at` is the position the concatenation is evaluated at — the consumer that renders it, since
    /// the chain's own text lands there — and every piece is checked there for the reason
    /// [`Self::render_value`] takes `at` at all (P3-R8).
    ///
    /// `depth` is the value-nesting depth the chain is rendered at, passed through to every piece:
    /// a concatenation inside an argument is one more level of the same native recursion, and the
    /// bound has to see it.
    fn concat_expr(
        &mut self,
        chain: &concat::Chain,
        at: u32,
        depth: usize,
    ) -> Result<Expr, ValueRenderFailure> {
        let mut parts: Vec<ConcatPart> = Vec::with_capacity(chain.appends.len());
        for (append_bci, parameter) in &chain.appends {
            let Some(instruction) = self.instructions.get(append_bci).copied() else {
                return Err(format!("no names record for the `append` at BCI {append_bci}").into());
            };
            let operands = stack_operands(instruction);
            let Some((_, value)) = operands.last().copied() else {
                return Err(format!(
                    "the `append` at BCI {append_bci} appends no value this run states"
                )
                .into());
            };
            // The value's own expression, anchored where it was produced, presenting the `append`
            // that converts it: the part and the instruction are one, and the table says which.
            let rendered = self
                .render_value(value, at, depth + 1)?
                .derived_from(*append_bci);
            let mut part = ConcatPart::new(parameter.clone(), rendered);
            if part.parameter == Type::Boolean {
                if !(self.boolean_literal(value) || self.boolean_proven(value, at)) {
                    return Err(format!(
                        "the `append` at BCI {append_bci} takes `boolean`, and this layer has no evidence that the value it reads at BCI {at} is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the overload's own conversion rejects"
                    ).into());
                }
                part.value = boolean_spelling(part.value);
            }
            // The part's own conversion, decided where the descriptor meets the value: a `char` part
            // of an `append(I)` states `(int) arg0`, because `"" + arg0` is a string concatenation
            // of a character where the `append` converted the code unit — the one position whose
            // conversion the **text** performs, so the text is what states it ([`Widening::Text`]).
            // Every other part is left exactly as it was rendered.
            part.value = meeting_position(
                part.value,
                parameter,
                &format!(
                    "the `append` at BCI {append_bci} takes `{}`",
                    parameter.spell()
                ),
                Widening::Text,
            )?;
            parts.push(part);
        }
        let origin = chain
            .owned
            .iter()
            .filter(|bci| **bci != chain.tail)
            .fold(OriginSet::new(Origin::direct(chain.tail)), |set, bci| {
                set.plus_derived(Origin::derived(*bci))
            });
        Ok(Expr::new(ExprKind::Concat { parts }, origin))
    }

    /// Renders one dynamic call site as a lambda or a method reference — or refuses it and records
    /// which link of the chain failed (P3 2.1, A04).
    ///
    /// Everything read here is the payload's own: the class's bootstrap table and pool decide *what
    /// the site is* ([`crate::lambda`]), the run's names decide the captured values and their BCIs,
    /// and the frames decide a capture's type. The record is pushed here, where the decision is
    /// made, so no site can be presented without a record and no record can describe text that was
    /// never written.
    fn lambda_expr(
        &mut self,
        bci: u32,
        instruction: &SsaInstruction,
        site: &DynamicSite,
        consumed: bool,
    ) -> Result<Expr, ValueRenderFailure> {
        let operands = stack_operands(instruction);
        let captures: Vec<(Option<u32>, Option<Type>)> = operands
            .iter()
            .map(|(_, value)| {
                (
                    self.value_bci(*value),
                    // A capture whose frame fact is a descriptor with no Java spelling is a value
                    // this layer cannot state a type for, and `lambda::plan` refuses a site with an
                    // unstated capture type under its own preconditions: the site is refused with
                    // its record and the capture's BCI, which is the answer a spelling failure gets
                    // here rather than a second refusal vocabulary.
                    value_type(self.ssa.value(*value).ty()).ok().flatten(),
                )
            })
            .collect();
        let verdict = lambda::plan(
            site,
            self.bootstrap,
            self.pool,
            self.members,
            &captures,
            &self.profile,
            self.budget,
            bci,
        )?;
        let evidence = verdict.evidence;
        // A refused site is recorded with the operands it reads and no text: a refusal states where
        // its evidence came from without pretending to have written it.
        let unrendered = || -> Vec<LambdaCapture> {
            captures
                .iter()
                .map(|(at, _)| LambdaCapture { bci: *at })
                .collect()
        };
        let mut plan = match verdict.outcome {
            Ok(plan) => plan,
            Err(refusal) => {
                self.publish_lambda(
                    bci,
                    site,
                    evidence,
                    None,
                    Some(refusal.clone()),
                    &unrendered(),
                );
                return Err(refusal.message().to_string().into());
            }
        };
        if plan.array_constructor.is_some() && !self.allow_array_constructor_method_references {
            // Class-source retains the physical helper until it can prove class-wide omission is
            // safe. The lambda planner independently requires a capture-free, unary SAM before it
            // grants this array-constructor proof.
            plan.form = LambdaForm::Lambda;
        }
        if !consumed {
            // The shape is verified, but the instance it creates reaches no statement: creating it
            // is still an invocation the bytecode makes, so the site is quoted rather than written
            // off as a value with no effect (a `push` whose value nobody reads is dropped; this is
            // not a `push`).
            let refusal = Refusal::shape(
                "jre_lambda_unconsumed",
                "nothing in this method reads the instance the site creates, so the invocation the site makes has no place in the body".to_string(),
            );
            self.publish_lambda(
                bci,
                site,
                evidence,
                None,
                Some(refusal.clone()),
                &unrendered(),
            );
            return Err(refusal.message().to_string().into());
        }
        // Every captured value has to be replayable before any of its text is written: the value is
        // written where the shape reads it, and the instruction that produced it may also have been
        // written as a statement of its own. The rule states the requirement and this is the check
        // point, so a capture that would run again (or run later) makes the site a fallback.
        for index in 0..plan.captures {
            let (at, _) = captures[index];
            let (_, value) = operands[index];
            if let Some(reason) = self.unreplayable(value) {
                let refusal = Refusal::unmet(
                    &LAMBDA,
                    Precondition::Replayable,
                    format!(
                        "the value captured for the site's argument {index} (BCI {}) cannot be written where the shape reads it: {reason}",
                        at.map_or_else(|| "unknown".to_string(), |at| at.to_string())
                    ),
                );
                self.publish_lambda(
                    bci,
                    site,
                    evidence,
                    None,
                    Some(refusal.clone()),
                    &unrendered(),
                );
                return Err(refusal.message().to_string().into());
            }
        }
        // The captured arguments, in the order the site reads them off the stack — which is the order
        // the implementation handle receives them in. Each expression carries its own anchor (the
        // BCI it was produced at), and the nodes whose text reproduces a capture present it as
        // derived evidence.
        let mut captures_rendered: Vec<Expr> = Vec::with_capacity(plan.captures);
        let mut captured: Vec<LambdaCapture> = Vec::with_capacity(plan.captures);
        for index in 0..plan.captures {
            let (at, _) = captures[index];
            let (_, value) = operands[index];
            let expr = match self.render_value(value, bci, 0) {
                Ok(expr) => expr,
                Err(ValueRenderFailure::Stop(stop)) => return Err(stop.into()),
                Err(ValueRenderFailure::Refusal(_)) => {
                    let refusal = Refusal::shape(
                        "jre_lambda_capture",
                        format!(
                            "the value captured for the site's argument {index} (BCI {}) produces no expression this subset writes",
                            at.map_or_else(|| "unknown".to_string(), |at| at.to_string())
                        ),
                    );
                    self.publish_lambda(
                        bci,
                        site,
                        evidence,
                        None,
                        Some(refusal.clone()),
                        &unrendered(),
                    );
                    return Err(refusal.message().to_string().into());
                }
            };
            captured.push(LambdaCapture { bci: at });
            captures_rendered.push(expr);
        }
        // The site's own anchor (with the pool entry that states it), and the same anchor carrying
        // the captures as evidence the text reproduces: one lambda expression reaches more than one
        // original BCI, and the table says so. The nodes *inside* the body — the receiver's type
        // name, the parameters — are anchored at the site alone: they do not reproduce a captured
        // value, and a node that claimed to would be evidence that is not true.
        let site_origin = OriginSet::new(Origin::direct(bci).with_cp(site.cp()));
        let origin = captured
            .iter()
            .fold(site_origin.clone(), |set, capture| match capture.bci {
                Some(at) => set.plus_derived(Origin::derived(at)),
                None => set,
            });
        let mut params: Vec<LambdaParam> = Vec::new();
        let mut parameters_rendered: Vec<Expr> = Vec::new();
        if plan.form == LambdaForm::Lambda {
            let adapter_nodes = plan
                .adaptation
                .parameters
                .iter()
                .map(|adaptation| {
                    1_u64
                        + u64::from(adaptation.sam_to_dynamic == TypeConversion::CheckCast)
                        + u64::from(matches!(
                            adaptation.dynamic_to_implementation,
                            TypeConversion::CheckCast | TypeConversion::WidenToObject
                        ))
                })
                .sum::<u64>();
            poll(self.budget, Some(bci)).map_err(ValueRenderFailure::Stop)?;
            if adapter_nodes != 0 {
                charge(
                    self.budget,
                    CountedBudgetDimension::IrItems,
                    adapter_nodes,
                    Some(bci),
                )
                .map_err(ValueRenderFailure::Stop)?;
            }
            params.reserve(plan.adaptation.parameters.len());
            parameters_rendered.reserve(plan.adaptation.parameters.len());
            for (index, adaptation) in plan.adaptation.parameters.iter().enumerate() {
                poll(self.budget, Some(bci)).map_err(ValueRenderFailure::Stop)?;
                let name = self.param_name(index);
                let mut value = Expr::new(ExprKind::Local(name.clone()), site_origin.clone())
                    .presenting(adaptation.sam.clone());
                if adaptation.sam_to_dynamic == TypeConversion::CheckCast {
                    value = Expr::new(
                        ExprKind::Cast {
                            ty: adaptation.dynamic.clone(),
                            value: Box::new(value),
                        },
                        site_origin.clone(),
                    );
                }
                if matches!(
                    adaptation.dynamic_to_implementation,
                    TypeConversion::CheckCast | TypeConversion::WidenToObject
                ) {
                    value = Expr::new(
                        ExprKind::Cast {
                            ty: adaptation.implementation.clone(),
                            value: Box::new(value),
                        },
                        site_origin.clone(),
                    );
                }
                parameters_rendered.push(value);
                params.push(LambdaParam {
                    ty: adaptation.sam.clone(),
                    name,
                });
            }
        }
        let factory_type = plan.target_type.clone();
        let expr = match plan.form {
            LambdaForm::MethodReference => {
                // The captures *are* what the handle's own receiver needs and nothing else, so the
                // site is a reference to the member itself: a type for a static or constructor one,
                // the bound receiver — the one capture — for an instance one. There are no arguments
                // to write: a `::` reference takes none.
                let qualifier = if let Some(array_constructor) = &plan.array_constructor {
                    Expr::new(
                        ExprKind::Path(array_constructor.array_type.spell().to_string()),
                        site_origin.clone(),
                    )
                } else {
                    match (plan.reach, captures_rendered.first()) {
                        (Reach::Receiver, Some(receiver)) => receiver.clone(),
                        _ => Expr::new(
                            ExprKind::Path(implementation_owner(&plan, bci)?),
                            site_origin.clone(),
                        ),
                    }
                };
                let name = if plan.array_constructor.is_some() {
                    "new".to_string()
                } else {
                    match plan.reach {
                        Reach::Constructor => "new".to_string(),
                        _ => plan.implementation.name().to_string(),
                    }
                };
                Expr::new(
                    ExprKind::MethodReference {
                        qualifier: Box::new(qualifier),
                        name,
                    },
                    origin.clone(),
                )
            }
            LambdaForm::Lambda => {
                let mut bound = captures_rendered;
                let parameter_count = parameters_rendered.len();
                bound.extend(parameters_rendered);
                let body = match plan.reach {
                    Reach::Constructor => Expr::new(
                        ExprKind::New {
                            ty: implementation_owner(&plan, bci)?,
                            qualifier: None,
                            member_name: None,
                            diamond: false,
                            args: bound,
                        },
                        origin.clone(),
                    ),
                    Reach::Static => Expr::new(
                        ExprKind::Call {
                            receiver: Some(Box::new(Expr::new(
                                ExprKind::Path(implementation_owner(&plan, bci)?),
                                site_origin.clone(),
                            ))),
                            name: plan.implementation.name().to_string(),
                            args: bound,
                        },
                        origin.clone(),
                    ),
                    Reach::Receiver => {
                        let mut operands_bound = bound.into_iter();
                        let receiver = operands_bound.next().ok_or_else(|| {
                            "a receiver-kind implementation with nothing bound to its receiver"
                                .to_string()
                        })?;
                        Expr::new(
                            ExprKind::Call {
                                receiver: Some(Box::new(receiver)),
                                name: plan.implementation.name().to_string(),
                                args: operands_bound.collect(),
                            },
                            origin.clone(),
                        )
                    }
                };
                debug_assert_eq!(params.len(), parameter_count);
                Expr::new(
                    ExprKind::Lambda {
                        params,
                        body: Box::new(body),
                    },
                    origin.clone(),
                )
            }
        };
        self.publish_lambda(bci, site, evidence, Some(plan.form), None, &captured);
        Ok(expr.presenting(factory_type))
    }

    /// One dynamic site's verdict: its refusal stated as a gap in every selection, and its record
    /// built only when this run publishes rule records and the site's own positions — the site and
    /// every capture it reads — are inside the selected driver range.
    ///
    /// The decision is taken **before** the record is built, so a site the selection excludes costs
    /// no owning record at all, and the counts the report states about this rule are the rule's own
    /// work over the whole body whatever the selection was.
    fn publish_lambda(
        &mut self,
        bci: u32,
        site: &DynamicSite,
        evidence: crate::lambda::Evidence,
        form: Option<LambdaForm>,
        refusal: Option<Refusal>,
        captures: &[LambdaCapture],
    ) {
        let shape = refusal.map(|refusal| LambdaRefusal::of(&refusal, bci));
        match &shape {
            Some(shape) => {
                self.lambda_refusals
                    .push(Gap::at(shape.code, bci, shape.message.clone()))
            }
            None => self.lambdas_presented += 1,
        }
        self.lambdas.push(LambdaSite {
            use_site: bci,
            site_cp: site.cp(),
            bootstrap_index: site.bootstrap_index(),
            sam_name: site.name().to_string(),
            sam_descriptor: site.descriptor().to_string(),
            evidence,
            form,
            refusal: shape,
            captures: captures.to_vec(),
        });
    }

    /// One accessor verdict at one call site: its refusal stated as a gap in every selection, and
    /// its record built only when this run publishes rule records and the call site is inside the
    /// selected driver range.
    ///
    /// The callee's own bytecode index is deliberately not a driver position: the call site is the
    /// driver's instruction, and what the record says about the callee (its identity, its field and
    /// the index the field access is at) travels inside the record as the proof of the verdict.
    fn publish_accessor(
        &mut self,
        call_site: u32,
        evidence: accessor::Evidence,
        presented: bool,
        refusal: Option<&Refusal>,
    ) {
        let shape = refusal.map(|refusal| AccessorRefusal::of(refusal, call_site));
        match &shape {
            Some(shape) => {
                self.accessor_refusals
                    .push(Gap::at(shape.code, call_site, shape.message.clone()))
            }
            None => self.accessors_presented += 1,
        }
        self.accessors.push(AccessorSite {
            call_site,
            evidence,
            presented,
            refusal: shape,
        });
    }

    /// Why one captured value cannot be written where the shape reads it, when it cannot.
    ///
    /// A capture's text is written into the artifact, and the instruction that produced it may also
    /// have been written as a statement of its own — a call whose result the site captures is a call
    /// statement *and* an argument. So the value is replayable only when re-reading its text is the
    /// same thing as having read it once: a literal, or a local the body writes exactly once.
    fn unreplayable(&self, value: ValueId) -> Option<String> {
        match self.ssa.value(value).def() {
            Definition::Instruction { bci, .. } => match self.operations.get(*bci) {
                Some(Operation::Push(_)) => None,
                Some(Operation::Load { slot }) => self.written_once(*slot),
                Some(operation) => Some(format!(
                    "it comes from an {operation:?} at BCI {bci}, whose text would run again (or run later) inside the shape"
                )),
                None => Some(format!(
                    "it comes from the instruction at BCI {bci}, which this run did not decode"
                )),
            },
            Definition::Entry {
                slot: Slot::Local(slot),
                ..
            }
            | Definition::Phi {
                slot: Slot::Local(slot),
                ..
            } => self.written_once(*slot),
            Definition::Entry {
                slot: Slot::Stack(depth),
                ..
            }
            | Definition::Phi {
                slot: Slot::Stack(depth),
                ..
            } => Some(format!(
                "it is the entry state of stack depth {depth}, which no instruction produced"
            )),
            Definition::Caught { bci, .. } => Some(format!(
                "it is the exception reference of the throw site at BCI {bci}"
            )),
        }
    }

    /// Why one local's text cannot be read again, when it cannot: the body writes it more than once.
    ///
    /// A local the body writes exactly once is *effectively final* in the source's own sense, and
    /// reading it again — which is what writing it inside a lambda body does — reads the same value.
    /// A local written twice might not: a later write could land between the site and the lambda's
    /// invocation, and the recovered text would then read a value the bytecode never captured.
    /// Refusing is the honest answer, because this layer has no liveness analysis that could prove
    /// the writes apart.
    fn written_once(&self, slot: u16) -> Option<String> {
        let mut writes = 0usize;
        for block in self.ssa.blocks() {
            for instruction in block.instructions() {
                if instruction
                    .writes()
                    .iter()
                    .any(|(written, _)| *written == Slot::Local(slot))
                {
                    writes += 1;
                }
            }
        }
        (writes > 1).then(|| {
            format!(
                "the local {slot} is written {writes} times in this body, so reading it again inside the shape would not read the value the site captured"
            )
        })
    }

    /// Whether one value reaches a statement — a read by an instruction this subset *renders* its
    /// operands for.
    ///
    /// A dynamic site's instance is a value like any other, except that dropping it would drop the
    /// invocation the site makes. The instructions that count are the ones whose rendering writes
    /// the values they read (a store, a call, a return, a branch, a switch, an arithmetic); a `pop`
    /// reads a value and writes nothing with it, which is exactly the case this question is about.
    fn value_is_consumed(&self, value: ValueId) -> bool {
        self.ssa.blocks().iter().any(|block| {
            block.instructions().iter().any(|instruction| {
                instruction.reads().iter().any(|(_, read)| *read == value)
                    && (self.renders_the_value_it_reads(instruction.bci())
                        // A dynamic site reads its captured values, but it writes the site's own
                        // BCIs rather than a rendering of the instruction that produced one of them
                        // — which is why it is not a reader in [`Self::produced_value_reaches_a_reader`]
                        // and is one here, where the question is about the *site's* value.
                        || matches!(
                            self.operations.get(instruction.bci()),
                            Some(Operation::InvokeDynamic(_))
                        ))
            })
        })
    }

    /// The BCI the instruction that produced one value sits at, when an instruction produced it.
    fn value_bci(&self, value: ValueId) -> Option<u32> {
        match self.ssa.value(value).def() {
            Definition::Instruction { bci, .. } | Definition::Caught { bci, .. } => Some(*bci),
            Definition::Phi { block, .. } => Some(block.bci()),
            Definition::Entry { .. } => None,
        }
    }

    /// One parameter name for a lambda of this body: the first free `pN`.
    ///
    /// The class file names no lambda parameter, so the name is derived — and it has to differ from
    /// every name the body's own locals carry *and* from every parameter name another lambda of this
    /// body already took (JLS 6.4 forbids a lambda parameter that shadows either). Allocation follows
    /// the order the shapes are written, so the names are a function of the evidence alone.
    fn param_name(&mut self, index: usize) -> String {
        let mut base = format!("p{index}");
        let name = loop {
            let candidate = self.names.free_name(&base);
            if !self.lambda_params.contains(&candidate)
                && !self.synthetic_names.contains(&candidate)
            {
                break candidate;
            }
            base = format!("{candidate}_");
        };
        self.lambda_params.insert(name.clone());
        name
    }

    /// Appends the quoted bytecode of one region or instruction.
    fn fallback(
        &mut self,
        bcis: Vec<u32>,
        failure: impl Into<ValueRenderFailure>,
        at: u32,
    ) -> Result<(), StopReason> {
        let reason = match failure.into() {
            ValueRenderFailure::Refusal(reason) => reason,
            ValueRenderFailure::Stop(stop) => return Err(stop),
        };
        self.ragged = true;
        // Every BCI the quote's text names is an anchor of the quoted node — the region's own
        // instruction first, the rest presented — so a Mixed artifact's fallback is mapped as
        // completely as one of its structured regions: for each bytecode the quote accounts for,
        // the table answers which text covers it. The list is the same one the emitter prints, so
        // the anchors and the quoted line cannot disagree (P3 3.2, A12/A13).
        let origin = bcis
            .iter()
            .filter(|bci| **bci != at)
            .fold(OriginSet::new(Origin::direct(at)), |origin, bci| {
                origin.plus_derived(Origin::derived(*bci))
            });
        self.push(Stmt::new(StmtKind::Fallback { reason, bcis }, origin))
    }

    /// Bills and appends one statement.
    fn push(&mut self, stmt: Stmt) -> Result<(), StopReason> {
        let at = stmt.origin.primary().bci();
        poll(self.budget, Some(at))?;
        charge(self.budget, CountedBudgetDimension::IrItems, 1, Some(at))?;
        let stmt = match self.undeclared_statement(&stmt) {
            Some(refused) => refused,
            None => stmt,
        };
        self.statements += 1;
        self.stmts.push(stmt);
        Ok(())
    }

    /// Bills and appends one statement of a body that is not the one being written.
    ///
    /// A `switch` expression's per-arm `return` is written after the arm it ends ([`Self::switch_join`]),
    /// so it lands in the arm's own body rather than in [`Self::stmts`]; the bill and the count are
    /// the ones [`Self::push`] makes, made where the statement really lands.
    fn push_into(&mut self, body: &mut Vec<Stmt>, stmt: Stmt) -> Result<(), StopReason> {
        let at = stmt.origin.primary().bci();
        poll(self.budget, Some(at))?;
        charge(self.budget, CountedBudgetDimension::IrItems, 1, Some(at))?;
        let stmt = match self.undeclared_statement(&stmt) {
            Some(refused) => refused,
            None => stmt,
        };
        self.statements += 1;
        body.push(stmt);
        Ok(())
    }

    /// The quote one statement is published as **instead**, when it spells a name no statement of
    /// this body declared (P3 2b.2).
    ///
    /// The statement is asked before it is published, because the name it would spell is one of the
    /// artifact's own: a local whose declaring write this build refused has no declaration in the
    /// text, and a statement reading it would name a variable nothing declares. The quote takes the
    /// statement's place and names every bytecode index its own text would have named, so the
    /// instructions the refused statement held are accounted for exactly as they were.
    fn undeclared_statement(&mut self, stmt: &Stmt) -> Option<Stmt> {
        let (name, bcis) = undeclared_local(stmt, &self.undeclared)?;
        let at = stmt.origin.primary().bci();
        self.ragged = true;
        let origin = bcis
            .iter()
            .filter(|bci| **bci != at)
            .fold(OriginSet::new(Origin::direct(at)), |origin, bci| {
                origin.plus_derived(Origin::derived(*bci))
            });
        Some(Stmt::new(
            StmtKind::Fallback {
                reason: format!(
                    "the statement at BCI {at} reads `{name}`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)"
                ),
                bcis,
            },
            origin,
        ))
    }

    /// The instruction starts one canonical block covers.
    fn covered_bcis(&self, block: &CanonicalBlockId) -> Vec<u32> {
        self.canonical
            .blocks()
            .iter()
            .find(|candidate| candidate.id() == block)
            .map(|block| block.blocks().to_vec())
            .unwrap_or_default()
    }

    /// Enumerates the decoded instructions owned by a refused canonical node. SSA states exact
    /// path-specific ownership for a live node, including fusion. A node without SSA can still
    /// use Code when it represents one raw block: its canonical end is that raw block's exact end.
    /// A fused node may jump over other blocks, so its overall BCI range is not ownership proof.
    fn fallback_instruction_bcis(&mut self, id: &CanonicalBlockId) -> Result<Vec<u32>, StopReason> {
        let Some(block) = self
            .canonical
            .blocks()
            .iter()
            .find(|block| block.id() == id)
        else {
            return Ok(Vec::new());
        };
        let mut bcis = Vec::new();
        if let Some(ssa) = self.ssa.block(id) {
            for instruction in ssa.instructions() {
                let bci = instruction.bci();
                poll(self.budget, Some(bci))?;
                charge(
                    self.budget,
                    CountedBudgetDimension::AnalysisSteps,
                    1,
                    Some(bci),
                )?;
                bcis.push(bci);
            }
        } else if let [start] = block.blocks() {
            let begin = self
                .code
                .instructions
                .partition_point(|instruction| instruction.bci < *start);
            for instruction in self.code.instructions[begin..]
                .iter()
                .take_while(|instruction| instruction.bci < block.end_bci())
            {
                let bci = instruction.bci;
                poll(self.budget, Some(bci))?;
                charge(
                    self.budget,
                    CountedBudgetDimension::AnalysisSteps,
                    1,
                    Some(bci),
                )?;
                bcis.push(bci);
            }
        } else {
            for &bci in block.blocks() {
                poll(self.budget, Some(bci))?;
                charge(
                    self.budget,
                    CountedBudgetDimension::AnalysisSteps,
                    1,
                    Some(bci),
                )?;
                bcis.push(bci);
            }
        }
        Ok(bcis)
    }
}

/// The arguments of one call, typed by the **callee's own descriptor** (P3-R5's argument side).
///
/// A `boolean` parameter and an `int` one are one slot shape, and the value the bytecode pushes for
/// `true`/`false` is the `int`-shaped `1`/`0` (`iconst_1`/`iconst_0`): a call site cannot state which
/// of the two it is passing, and the descriptor of the member it calls is the only evidence that can.
/// So an argument that is a **literal** `0`/`1` — and only a literal: an expression that computes one
/// is a value whose type this layer would be guessing at — is written `false`/`true` where the
/// parameter is declared `boolean`.
///
/// Every other parameter is normalized by [`Builder::invocation_argument`], which makes primitive
/// conversions and safe reference casts explicit. This descriptor-only helper keeps the boolean
/// literal conversion separate so existing callers can reuse it.
///
/// The mapping is by position, so it is only applied where the descriptor and the arguments really
/// line up. A descriptor that does not parse, or one whose parameter count differs from the number of
/// arguments the call reads, is refused by the shared call path rather than typing one argument from a
/// guess. This is the shared path every invocation of this layer goes through — `invokevirtual`,
/// `invokespecial`, `invokestatic` and `invokeinterface` alike, the constructor call of a `new` and
/// the `super(…)`/`this(…)` of an instance initializer — because "which argument is a `boolean`" is a
/// fact about the callee, not about the shape that writes the call.
///
/// One place is deliberately **not** this shape: the arguments a lambda's factory site binds are the
/// site's own captures, whose types the `invokedynamic` descriptor states for the SAM rather than for
/// the implementation the reference names, so nothing there is re-typed.
pub(crate) fn typed_arguments(descriptor: &str, arguments: Vec<Expr>) -> Vec<Expr> {
    typed_arguments_with_offset(descriptor, arguments, 0)
}

fn typed_arguments_with_offset(
    descriptor: &str,
    arguments: Vec<Expr>,
    parameter_offset: usize,
) -> Vec<Expr> {
    let Some(parameters) = parameter_descriptors(descriptor) else {
        return arguments;
    };
    if parameters.len().saturating_sub(parameter_offset) != arguments.len() {
        return arguments;
    }
    arguments
        .into_iter()
        .zip(parameters.into_iter().skip(parameter_offset))
        .map(|(argument, parameter)| typed_argument(argument, parameter))
        .collect()
}

/// Applies the one descriptor-only spelling shared by every call position: an int-shaped literal
/// `0`/`1` passed to a boolean parameter is the boolean the descriptor declares.
fn typed_argument(argument: Expr, parameter: &str) -> Expr {
    if parameter != "Z" {
        return argument;
    }
    let ExprKind::Integer(value) = argument.kind else {
        return argument;
    };
    let spelled = match value {
        0 => false,
        1 => true,
        _ => return argument,
    };
    Expr::new(ExprKind::Boolean(spelled), argument.origin)
}

/// One call-site cast. Its anchors remain the argument's original producers: this is a source
/// spelling decision, not an invented bytecode instruction.
fn cast_argument(argument: Expr, required: &Type, bci: u32) -> Expr {
    if matches!(&argument.kind, ExprKind::Cast { ty, .. } if ty == required) {
        return argument;
    }
    let origin = argument.origin.clone().plus_derived(Origin::derived(bci));
    Expr::new(
        ExprKind::Cast {
            ty: required.clone(),
            value: Box::new(argument),
        },
        origin,
    )
}

/// Gives a direct LambdaMetafactory expression its Java target type when it is consumed as a call
/// receiver. The factory descriptor and the invocation's own interface-method owner must state the
/// same spellable type; no subtype or interface hierarchy is inferred here.
fn immediate_functional_receiver(
    receiver: Expr,
    target: &CallTarget,
    bci: u32,
) -> Result<Expr, String> {
    let functional_type = receiver.presented.clone().ok_or_else(|| {
        format!(
            "the direct functional receiver of the call at BCI {bci} has no verified factory target type"
        )
    })?;
    let Type::Reference(name) = &functional_type else {
        return Err(format!(
            "the direct functional receiver of the call at BCI {bci} presents `{}`, not a Java functional-interface type",
            functional_type.spell()
        ));
    };
    if name.ends_with("[]") {
        return Err(format!(
            "the direct functional receiver of the call at BCI {bci} presents array type `{name}`, not a Java functional-interface type"
        ));
    }
    if target.kind() != InvokeKind::Interface || !target.is_interface_reference() {
        return Err(format!(
            "the direct functional receiver of the call at BCI {bci} is consumed through unsupported owner `{}`; this layer requires an interface-method reference",
            target.owner()
        ));
    }
    let owner = spell_reference(target.owner()).ok_or_else(|| {
        format!(
            "the direct functional receiver of the call at BCI {bci} names owner `{}`, which this layer cannot spell as a Java type",
            target.owner()
        )
    })?;
    if owner.ends_with("[]") {
        return Err(format!(
            "the direct functional receiver of the call at BCI {bci} names array owner `{owner}`, not a functional-interface type"
        ));
    }
    if *name != owner {
        return Err(format!(
            "the direct functional receiver of the call at BCI {bci} has factory type `{name}`, but the call owner is `{owner}`; this layer cannot prove they are the same interface"
        ));
    }
    Ok(cast_argument(receiver, &functional_type, bci))
}

/// One value written where a position **requires a type**, with the conversion between the two made
/// explicit in the expression.
///
/// This is the rule `make-required-conversions-explicit` states, and it exists because the
/// conversion used to happen *in the printing context* instead of in the expression: `append((int) c)`
/// — `c` a `char` — was written `"" + arg0 + "!"`, which is a different program (`"A!"` where the
/// class answers `"65!"`) and compiles just as well. A `char`, a `byte` and a `short` share one slot
/// shape with an `int`, so no instruction in the body states the conversion; the facts that do are
/// the value's **presented** type ([`Expr::presented`], which reaches a local from its declaration
/// and a call or a field read from a descriptor) and the position's requirement, which the caller
/// states as the phrase `position` names it by.
///
/// Three outcomes, and the second is where the text has to know what the position does for itself:
///
/// * the two types are the same, an `int` **constant** is taken by the position's own rule
///   (JLS 5.2: `byte b = 65;`, and a `char` return reads the
///   character its constant stands for, [`narrowed_literal`]) or either side is a reference: the
///   value is written exactly as it was rendered, with nothing added;
/// * a **widening primitive conversion** (JLS 5.1.2) connects them: what the text writes depends on
///   which side of the conversion the position is ([`Widening`]) — the position performs it for the
///   value where the text is an assignment or a `return` (P3 2c.29), and the text
///   states it where the text's own semantics select the conversion (a concatenation part, T5);
/// * nothing this layer's evidence states connects them — a `boolean` beside another primitive,
///   which no conversion relates at all (JLS 5.5), or a narrowing this layer cannot prove: the region
///   is refused. Publishing the value's own text would write another value or text `javac` rejects,
///   and "the two texts happen to compile" is exactly what the defect this rule closes was.
fn meeting_position(
    value: Expr,
    required: &Type,
    position: &str,
    widening: Widening,
) -> Result<Expr, String> {
    let Some(presented) = value.presented.clone() else {
        // The layer states no type for this value (`null`, a lambda, an arithmetic it cannot type):
        // no mismatch is provable, so nothing is converted — a rule that refused here would refuse
        // every value it cannot name, which is not what "the position requires a type" says.
        return Ok(value);
    };
    if &presented == required {
        return Ok(value);
    }
    if narrowed_constant(&value, required) {
        return Ok(narrowed_literal(value, required));
    }
    match conversion(&presented, required) {
        Conversion::Same => Ok(value),
        Conversion::Widening => match widening {
            // The position converts the value itself and the value's own text is what it reads: no
            // instruction ran and Java widens at the assignment and the `return`, so
            // `(int) arg0.charAt(arg1)` states a conversion the source never had (P3 2c.29).
            Widening::Position => Ok(value),
            // The text's own conversion is selected by the operand's type, so it is the text that has
            // to state it: the node carries the value's own anchors, because the conversion has no
            // instruction of its own to anchor it.
            Widening::Text => {
                let origin = value.origin.clone();
                Ok(Expr::new(
                    ExprKind::Cast {
                        ty: required.clone(),
                        value: Box::new(value),
                    },
                    origin,
                ))
            }
        },
        Conversion::Unspellable => Err(format!(
            "the value at BCI {} is presented as `{}` and {position}, and no conversion this layer's \
             evidence states connects the two: a widening primitive conversion (JLS 5.1.2) is the \
             only one a position performs for itself, so the region is refused rather than published \
             with the value the position would convert differently or with text `javac` refuses",
            value.origin.primary().bci(),
            presented.spell()
        )),
    }
}

/// Whether a presented Java type meets the source category one primitive conversion opcode names.
///
/// The opcode's `int` category also admits byte/char/short expressions: their values occupy the
/// same verifier shape and Java can feed each directly to a numeric cast. Boolean shares that JVM
/// shape but is not a numeric Java expression, so it is explicitly refused here.
fn primitive_conversion_source_matches(source: &Type, presented: &Type) -> bool {
    match source {
        Type::Int => matches!(presented, Type::Byte | Type::Char | Type::Short | Type::Int),
        Type::Long => presented == &Type::Long,
        Type::Float => presented == &Type::Float,
        Type::Double => presented == &Type::Double,
        Type::Boolean | Type::Byte | Type::Char | Type::Short | Type::Reference(_) => false,
    }
}

/// How one value's presented type meets the type a consuming position requires of it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Conversion {
    /// The text already is what the position reads.
    Same,
    /// A widening primitive conversion (JLS 5.1.2) connects the two: what the text writes is
    /// [`Widening`]'s decision, because it is a fact about the position.
    Widening,
    /// No conversion this layer's evidence states connects them.
    Unspellable,
}

/// Which side of a **widening primitive conversion** (JLS 5.1.2) writes the conversion the bytecode
/// performed with no instruction.
///
/// The two positions differ in a fact about the *text*, not in taste, and that is why the caller
/// states it rather than this layer deciding from the types:
///
/// * an assignment, a field write and a `return` **perform** the conversion themselves — the
///   compiler wrote no instruction because the position's own type already gives the value the
///   wider type (P3 2c.29). Invocation arguments use their own descriptor-aware rule above so
///   overload selection remains stable;
/// * a **concatenation part** does not: the text is a `+`, and `+` converts an operand by the
///   operand's own type. `"" + c` for a `char` `c` is `append(C)` — the character — where the
///   bytecode called `append(I)` and appended the code unit, so the part has to state the conversion
///   the `append` performed (`"" + (int) arg0 + "!"` answers `"65!"`, T5).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Widening {
    /// The position performs the conversion: the value is written exactly as it was rendered.
    Position,
    /// The text's own semantics select the conversion, so the text states it.
    Text,
}

/// The conversion from one presented type to the type a position requires.
///
/// Two references, and a reference beside a primitive, are `Same`: a reference conversion is not
/// this primitive mechanism's question — `+`, `append`, a return and a write each have their own
/// reference semantics, while invocation arguments use the stricter descriptor-aware rule above.
/// Which class is assignable to which is a **subtype judgment** this layer deliberately does not
/// make (`make-required-conversions-explicit`'s non-goal). `boolean` beside any other primitive is
/// `Unspellable`: JLS 5.5 states there is no conversion between them at all, in either direction.
fn conversion(presented: &Type, required: &Type) -> Conversion {
    if presented == required {
        return Conversion::Same;
    }
    if matches!(presented, Type::Reference(_)) || matches!(required, Type::Reference(_)) {
        return Conversion::Same;
    }
    if widens(presented, required) {
        Conversion::Widening
    } else {
        Conversion::Unspellable
    }
}

/// Whether JLS 5.1.2's widening primitive conversion relates one type to the other.
fn widens(presented: &Type, required: &Type) -> bool {
    matches!(
        (presented, required),
        (
            Type::Byte,
            Type::Short | Type::Int | Type::Long | Type::Float | Type::Double
        ) | (
            Type::Short,
            Type::Int | Type::Long | Type::Float | Type::Double
        ) | (
            Type::Char,
            Type::Int | Type::Long | Type::Float | Type::Double
        ) | (Type::Int, Type::Long | Type::Float | Type::Double)
            | (Type::Long, Type::Float | Type::Double)
            | (Type::Float, Type::Double)
    )
}

/// Whether the value is an `int` **constant** the required type takes by constant narrowing
/// (JLS 5.2 for an assignment or a `return`). Invocation arguments add their explicit cast in
/// [`Builder::invocation_argument`].
///
/// This is not a conversion the text has to state: `byte b = 65;` and `takes(65)` for a `short`
/// parameter are already legal Java and already carry the value the bytecode pushed — a constant
/// expression of type `int` narrows where it is representable. The rule exists because the
/// alternative reading (every `int` in a narrower position is a mismatch) would refuse the shapes
/// Java itself writes, and the check is the constant's own value, which is the fact the language
/// narrows by. What the position then *writes* is [`narrowed_literal`]'s business.
fn narrowed_constant(value: &Expr, required: &Type) -> bool {
    let ExprKind::Integer(constant) = value.kind else {
        return false;
    };
    match required {
        Type::Byte => i8::try_from(constant).is_ok(),
        Type::Short => i16::try_from(constant).is_ok(),
        Type::Char => u16::try_from(constant).is_ok(),
        _ => false,
    }
}

/// The `int` **constant** a narrow position takes by constant narrowing, written as the literal
/// that type's own text has (P3 2c.30).
///
/// A `byte` and a `short` position write the decimal number the constant already is: `return 3;` in
/// a `byte` method is legal Java (JLS 5.2), carries the value the bytecode pushed, and no second
/// spelling of it exists. A `char` position is the one that has a second spelling of the same value,
/// and it is the one a compiler writes: `return 'A';` compiles to the `bipush 65; ireturn` this
/// layer read, so `65` and `'A'` are one code unit and only the character states the type the
/// member's own descriptor declares. The literal is spelled by the emitter's own escape table
/// ([`crate::emit::char_literal`]): `'A'`, `'\n'`, `'\''`, `'\u2028'` — the spelling a `char`
/// `switch` selector's keys already get, from the one table.
///
/// The node that carries the text is [`ExprKind::Local`], exactly as the `field++`/`++field` update
/// carries its own text ([`increment_expression`], P3 2c.10): there is no literal node for a
/// character, and the type is stated by [`Expr::presenting`] — the fact this position read — so no
/// consumer has to spell the literal again. A value this function was not called for is returned
/// unchanged, and the caller's own test ([`narrowed_constant`]) is what says the constant is in
/// range: this function never widens its own reach.
fn narrowed_literal(value: Expr, required: &Type) -> Expr {
    let ExprKind::Integer(constant) = value.kind else {
        return value;
    };
    match required {
        Type::Char => match u16::try_from(constant) {
            Ok(unit) => Expr::new(
                ExprKind::Local(crate::emit::char_literal(unit)),
                value.origin.clone(),
            )
            .presenting(Type::Char),
            Err(_) => value,
        },
        _ => value,
    }
}

/// The type one field descriptor states, when this layer can spell it.
///
/// A parameter's own descriptor is a field descriptor (JVMS 4.3.2/4.3.3), so this is the one
/// reading of both: a callee's parameter, a field's own declaration, and the type a write has to
/// meet.
fn descriptor_type(descriptor: &str) -> Option<Type> {
    let facts = descriptor_facts(descriptor.as_bytes(), DescriptorKind::Field).ok()?;
    lambda::type_of_component(facts.single()?)
}

/// Whether one presented array type can be widened to a proven array type or array supertype using
/// only Java's closed array component rules. Class and interface inheritance is deliberately not
/// consulted here.
///
/// Both names are Java spellings read from existing type facts; this helper only peels their array
/// suffixes. A descriptor can contain at most 255 array dimensions, which bounds the component
/// walk.
fn array_reference_widens(presented: &str, required: &str) -> bool {
    const MAX_ARRAY_DIMENSIONS: usize = 255;

    fn array_parts(name: &str) -> Option<(&str, usize)> {
        let mut base = name;
        let mut dimensions = 0usize;
        while let Some(component) = base.strip_suffix("[]") {
            dimensions += 1;
            if dimensions > MAX_ARRAY_DIMENSIONS {
                return None;
            }
            base = component;
        }
        if dimensions == 0 || base.is_empty() || base == "void" {
            return None;
        }
        Some((base, dimensions))
    }

    fn primitive(name: &str) -> bool {
        matches!(
            name,
            "boolean" | "byte" | "char" | "short" | "int" | "long" | "float" | "double"
        )
    }

    let Some((presented_base, mut presented_dimensions)) = array_parts(presented) else {
        return false;
    };
    let (required_base, mut required_dimensions) = match array_parts(required) {
        Some((base, dimensions)) => (base, dimensions),
        // Every array type implements these two marker interfaces. A class component with one of
        // these names is not enough evidence: the marker shortcut applies only to an array value.
        None if matches!(required, "java.lang.Cloneable" | "java.io.Serializable") => {
            return true;
        }
        None => return false,
    };

    loop {
        // The current outer arrays have been reduced to their component types. An array component
        // is a reference even when its own element type is primitive (`int[]` is an Object).
        let presented_component_is_array = presented_dimensions > 1;
        let required_component_is_array = required_dimensions > 1;
        let presented_component_is_reference =
            presented_component_is_array || !primitive(presented_base);

        if !required_component_is_array
            && required_base == "java.lang.Object"
            && presented_component_is_reference
        {
            return true;
        }

        if !required_component_is_array
            && matches!(
                required_base,
                "java.lang.Cloneable" | "java.io.Serializable"
            )
            && presented_component_is_array
        {
            return true;
        }

        if presented_component_is_array && required_component_is_array {
            presented_dimensions -= 1;
            required_dimensions -= 1;
            continue;
        }

        if !presented_component_is_array && !required_component_is_array {
            // Primitive components are invariant; reference components are admitted only when
            // identical here. The Object and marker cases above are the sole widenings proved by
            // array shape.
            return presented_base == required_base;
        }

        // An array component cannot become an arbitrary class/interface component, and an
        // arbitrary class/interface component cannot become an array component.
        return false;
    }
}

/// The type one method descriptor's **result** states, when this layer can spell it (`V` is `None`).
///
/// The member's own descriptor is the requirement every `return` of its body is written under, and a
/// callee's descriptor is the type of the *value* a call site produces: the same reading answers
/// both, and it is the reader's own reading of the production ([`lambda::parse_method`]).
pub(crate) fn return_type(descriptor: &str) -> Option<Type> {
    lambda::parse_method(descriptor)?.1
}

/// The parameter type descriptors one method descriptor states, in order
/// (`(Ljava/lang/String;Z)V` → `["Ljava/lang/String;", "Z"]`), or `None` when it does not parse.
///
/// The components are the reader's own reading of the production ([`descriptor_facts`]), and each
/// one's bytes are cut out of the descriptor by the span the reader stated: the argument list this
/// answers is what a call's own descriptor says, in the same reading every other consumer of that
/// descriptor uses. A descriptor that does not parse states no parameters a caller could line
/// arguments up with, so the whole list is `None` rather than a prefix.
fn parameter_descriptors(descriptor: &str) -> Option<Vec<&str>> {
    let bytes = descriptor.as_bytes();
    let facts = descriptor_facts(bytes, DescriptorKind::Method).ok()?;
    facts
        .parameters()
        .iter()
        .map(|component| {
            let raw = component.bytes(bytes)?;
            std::str::from_utf8(raw).ok()
        })
        .collect()
}

/// The array one value holds: its element type and how many dimensions it has.
///
/// Two sources, and the walk exists because the second one is not always there:
///
/// * the **frames**, when they name the value's reference: an array's frame entry is its whole
///   descriptor (`[I`, `[[Ljava/lang/String;`), which is where a parameter, a field read, an
///   `anewarray` and a `multianewarray` state their array. A named reference that is not an array
///   descriptor states no array here (and a class name is never read as one);
/// * the **instruction that built the array**, for the one creation the frames leave unknown:
///   `newarray`'s array has a primitive element the bootstrap loader defines, and a standalone read
///   of one class does not declare that loader, so the frame pass keeps a conservative unknown
///   reference for it and the instruction's own `atype` is the fact that says what the array is.
///   This is the case of every `new int[n]`, `new char[n]` and `new boolean[n]` of a body.
///
/// The second source is read through the instructions that only **move** a value, so that a local
/// holding a created array answers exactly as the creation does: a copy holds what it duplicated, a
/// load reads the slot, a slot holds what the last store before it wrote, and a store's own value is
/// the stack value it stored ([`store_operand`]). Nothing else is followed — a call's result, a
/// field read and a value merged out of several definitions state no array here — which is what
/// keeps this a reading of the facts that name an array and not a type system.
fn array_of_value(
    ssa: &SsaTable,
    operations: &Operations,
    value: ValueId,
    depth: usize,
) -> Option<(Type, u32)> {
    if depth > MAX_VALUE_DEPTH {
        return None;
    }
    if let Value::Ref(RefType::Named { name, .. }) = ssa.value(value).ty() {
        return array_descriptor(&String::from_utf8_lossy(name));
    }
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return None;
    };
    let bci = *bci;
    match operations.get(bci)? {
        Operation::NewArray {
            element,
            dimensions,
            total_dimensions,
        } => {
            if *dimensions == 0 || *dimensions > *total_dimensions {
                return None;
            }
            Some((element.clone(), u32::from(*total_dimensions)))
        }
        Operation::Duplicate => {
            let (_, copied) = single_stack_read(instruction_at(ssa, bci)?)?;
            array_of_value(ssa, operations, copied, depth + 1)
        }
        Operation::Load { slot } => {
            let read = local_read(instruction_at(ssa, bci)?, *slot)?;
            array_of_value(ssa, operations, read, depth + 1)
        }
        // A value a slot holds is the value the instruction that wrote the slot takes: the SSA
        // states the store's own value, and what it stored is the stack value it read.
        Operation::Store { .. } => {
            let stored = store_operand(operations, instruction_at(ssa, bci)?)?;
            array_of_value(ssa, operations, stored, depth + 1)
        }
        _ => None,
    }
}

/// The array one **descriptor** states, when this layer can spell it: its element type and its
/// dimension count, read by this crate's one descriptor reading ([`crate::lambda::type_of_base`] for
/// the base, the reader's own facts for the `[`s).
fn array_descriptor(descriptor: &str) -> Option<(Type, u32)> {
    let facts = descriptor_facts(descriptor.as_bytes(), DescriptorKind::Field).ok()?;
    let component = facts.single()?;
    let dimensions = component.dimensions();
    if dimensions == 0 {
        return None;
    }
    Some((lambda::type_of_base(component.base())?, dimensions))
}

/// The Java spelling of one array type: its element type and one `[]` per dimension.
fn array_spelling(element: &Type, dimensions: u32) -> Option<Type> {
    let dimensions = usize::try_from(dimensions).ok()?;
    Some(Type::Reference(format!(
        "{}{}",
        element.spell(),
        "[]".repeat(dimensions)
    )))
}

/// The element type one `array[index]` reads out of an array of that shape: the element itself for a
/// one-dimensional array, and the array one dimension less for a deeper one (`[[I` holds `int[]`
/// elements, so a read out of it is one `[]` shorter than the array it indexed). A shape with no
/// dimension at all is no array to index.
fn element_of_dimension(element: &Type, dimensions: u32) -> Option<Type> {
    match dimensions {
        0 => None,
        1 => Some(element.clone()),
        deeper => array_spelling(element, deeper - 1),
    }
}

/// The element type one array instruction's **opcode** states, with nothing else consulted.
///
/// This is the second source [`array_element`] falls back to, and it is deliberately a reading of the
/// operation alone: `ArrayLoad` — the `int[]` read the enum-switch rule names — states `int`, which
/// is the whole of what JVMS 6.5's `iaload` says; `ArrayElementLoad` and `ArrayStore` state the
/// element their own decode recorded (`laload`'s `long`, `baload`'s `int`, `aaload`'s `None`); and an
/// operation of any other kind states none.
fn stated_element(operation: &Operation) -> Option<Type> {
    match operation {
        Operation::ArrayLoad => Some(Type::Int),
        Operation::ArrayElementLoad { element } | Operation::ArrayStore { element } => {
            element.clone()
        }
        _ => None,
    }
}

/// The element type one array access reads or writes, from every fact this body has for the array
/// (P3 2b).
///
/// The array's own shape comes first ([`array_of_value`]), because the four int-sized primitives are
/// the one case the opcode cannot decide: JVMS 2.11.1 gives a `boolean`, a `byte`, a `char`, a
/// `short` and an `int` one value shape and one slot, so `baload` on a `[Z` array and on a `[B` array
/// are the same instruction — the array's own type says which, and nothing about the opcode does.
///
/// `stated` is what the **opcode** itself states: the element type of `laload`/`faload`/`daload`, and
/// nothing at all for the int-sized family (whose element family is what `crate::decode` leaves
/// unstated) and for `aaload`. Where neither source states one, `None` is *no type claimed* — an
/// `aaload` of an array this run cannot name presents no element type, and specifically not the
/// `int` its shape must never claim.
fn array_element(
    ssa: &SsaTable,
    operations: &Operations,
    value: ValueId,
    stated: Option<&Type>,
) -> Option<Type> {
    match array_of_value(ssa, operations, value, 0) {
        Some((element, dimensions)) => element_of_dimension(&element, dimensions),
        None => stated.cloned(),
    }
}

/// The type one write publishes for the variable it fills.
///
/// The frames answer for every value they type, and one case is answered by a fact of the body
/// instead: a local holding a `newarray` creation — whose frame entry is the conservative unknown
/// reference that instruction's element type forces — is declared with the array the creation's own
/// operand states (`int[] local1 = new int[arg0];`), because a declaration of `Object` for an array
/// the same statement just built is a body no compiler writes.
fn written_type(
    ssa: &SsaTable,
    operations: &Operations,
    value: ValueId,
) -> Result<Option<Type>, String> {
    if let Some((element, dimensions)) = array_of_value(ssa, operations, value, 0) {
        return Ok(array_spelling(&element, dimensions));
    }
    value_type(ssa.value(value).ty())
}

/// The instruction one bytecode index holds, as the SSA table's blocks publish it.
///
/// The SSA is read by value and by block everywhere else; a rule that has a BCI and needs the reads
/// the instruction performed, in their order, is what this lookup is for.
fn instruction_at(ssa: &SsaTable, bci: u32) -> Option<&SsaInstruction> {
    ssa.blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == bci)
}

/// The boolean evidence one SSA value carries, including nested eager bitwise combinations.
///
/// The `0`/`1` literal is usable only beside another boolean operand and never seeds a proof. An
/// `&`, `^` or `|` is boolean only when both operands are boolean evidence and at least one side has
/// a descriptor or already-decided-local seed. The frame's shared `Int` shape never chooses between
/// `boolean` and integer operands.
struct BooleanProofContext<'a, IsLocal, Visit> {
    ssa: &'a SsaTable,
    operations: &'a Operations,
    parameter_types: &'a BTreeMap<u16, Type>,
    fields: &'a field::Plan,
    is_boolean_local: IsLocal,
    visit: Visit,
}

fn boolean_proof<IsLocal, Visit>(
    context: &mut BooleanProofContext<'_, IsLocal, Visit>,
    value: ValueId,
    at: u32,
    depth: usize,
    memo: &mut BTreeMap<(ValueId, usize), BooleanEvidence>,
) -> Result<BooleanEvidence, StopReason>
where
    IsLocal: Fn(ValueId, u32) -> bool,
    Visit: FnMut(u32) -> Result<(), StopReason>,
{
    if depth > MAX_VALUE_DEPTH {
        return Ok(BooleanEvidence::None);
    }
    let key = (value, MAX_VALUE_DEPTH - depth);
    if let Some(evidence) = memo.get(&key) {
        return Ok(*evidence);
    }
    let bci = match context.ssa.value(value).def() {
        Definition::Instruction { bci, .. } | Definition::Caught { bci, .. } => *bci,
        Definition::Entry { .. } | Definition::Phi { .. } => at,
    };
    (context.visit)(bci)?;

    let evidence = if parameter_boolean(
        context.ssa,
        context.operations,
        context.parameter_types,
        value,
    ) || (context.is_boolean_local)(value, at)
    {
        BooleanEvidence::Proven
    } else {
        let Definition::Instruction { bci, .. } = context.ssa.value(value).def() else {
            memo.insert(key, BooleanEvidence::None);
            return Ok(BooleanEvidence::None);
        };
        match context.operations.get(*bci) {
            Some(Operation::Push(ConstantValue::Int(0 | 1))) => BooleanEvidence::Literal,
            Some(Operation::InstanceOf { .. }) => BooleanEvidence::Proven,
            Some(Operation::Invoke(target)) if returns_boolean(target.descriptor()) => {
                BooleanEvidence::Proven
            }
            Some(Operation::Field { .. })
                if context
                    .fields
                    .claim(*bci)
                    .is_some_and(|(evidence, _)| evidence.descriptor == "Z") =>
            {
                BooleanEvidence::Proven
            }
            // The element type comes from the array's proven `[Z` shape; `baload` on `[B` has the
            // same opcode and does not prove a boolean.
            Some(Operation::ArrayLoad | Operation::ArrayElementLoad { .. }) => {
                let boolean_array = instruction_at(context.ssa, *bci).is_some_and(|instruction| {
                    stack_operands(instruction)
                        .first()
                        .is_some_and(|(_, array)| {
                            array_element(context.ssa, context.operations, *array, None)
                                == Some(Type::Boolean)
                        })
                });
                if boolean_array {
                    BooleanEvidence::Proven
                } else {
                    BooleanEvidence::None
                }
            }
            Some(Operation::Bitwise { .. }) => {
                let Some(instruction) = instruction_at(context.ssa, *bci) else {
                    memo.insert(key, BooleanEvidence::None);
                    return Ok(BooleanEvidence::None);
                };
                let operands = stack_operands(instruction);
                if let [(_, left), (_, right)] = operands.as_slice() {
                    let left = boolean_proof(context, *left, at, depth + 1, memo)?;
                    let right = boolean_proof(context, *right, at, depth + 1, memo)?;
                    if left.is_boolean()
                        && right.is_boolean()
                        && (left.has_seed() || right.has_seed())
                    {
                        BooleanEvidence::Proven
                    } else {
                        BooleanEvidence::None
                    }
                } else {
                    BooleanEvidence::None
                }
            }
            _ => BooleanEvidence::None,
        }
    };
    memo.insert(key, evidence);
    Ok(evidence)
}

/// Collect the local reads that can change one first-write boolean proof.
///
/// This follows only a direct local copy or nested bitwise operands. Arithmetic, unknown stores,
/// and other value producers remain outside this type decision. Every visited value is charged and
/// polled, and the same value/depth bound as rendering ends an oversized chain conservatively.
struct BitwiseDependencyContext<'a> {
    ssa: &'a SsaTable,
    operations: &'a Operations,
    reuse: &'a reuse::Plan,
    budget: &'a mut Budget,
}

impl BitwiseDependencyContext<'_> {
    fn collect(
        &mut self,
        value: ValueId,
        at: u32,
        depth: usize,
        seen: &mut BTreeSet<ValueId>,
        dependencies: &mut BTreeSet<LocalVariable>,
    ) -> Result<(), StopReason> {
        if depth > MAX_VALUE_DEPTH || !seen.insert(value) {
            return Ok(());
        }
        if let Some(variable) = read_variable(self.ssa, self.operations, self.reuse, value, at) {
            dependencies.insert(variable);
            return Ok(());
        }
        let Definition::Instruction { bci, .. } = self.ssa.value(value).def() else {
            return Ok(());
        };
        if !matches!(self.operations.get(*bci), Some(Operation::Bitwise { .. })) {
            return Ok(());
        }
        poll(self.budget, Some(*bci))?;
        charge(self.budget, CountedBudgetDimension::IrItems, 1, Some(*bci))?;
        let Some(instruction) = instruction_at(self.ssa, *bci) else {
            return Ok(());
        };
        for (_, operand) in stack_operands(instruction) {
            self.collect(operand, at, depth + 1, seen, dependencies)?;
        }
        Ok(())
    }
}

/// Bill each composite bitwise proof node while leaving the pre-existing direct-copy decision cost
/// unchanged. Its queue entries are already charged by `decide_types`.
fn charge_bitwise_proof_node(
    operations: &Operations,
    bci: u32,
    budget: &mut Budget,
) -> Result<(), StopReason> {
    if matches!(operations.get(bci), Some(Operation::Bitwise { .. })) {
        poll(budget, Some(bci))?;
        charge(budget, CountedBudgetDimension::IrItems, 1, Some(bci))?;
    }
    Ok(())
}

/// Which store of a chained assignment one store is, and the copy whose value it presents
/// (P3 2c.14, [`Builder::chained`]).
enum Chained {
    /// The first store of the pair: it writes the value the copy duplicated.
    First { duplicated: ValueId, dup_bci: u32 },
    /// The second store of the pair: it reads the local the first store filled.
    Second { first: u32, dup_bci: u32 },
}

/// The three instructions of one chained assignment's copy: the value it duplicated and the two
/// stores it feeds, in the order the block holds them ([`Builder::chained_pair`]).
struct ChainedPair {
    /// The value the copy duplicated: what its first store writes.
    duplicated: ValueId,
    /// The BCI of the first store, and the BCI of the second.
    first: u32,
    second: u32,
}

/// The opcode of `dup` (JVMS 6.5): the copy that leaves one value on the stack twice.
const OPCODE_DUP: u8 = 0x59;
/// The opcode of `dup_x1`: the copy that inserts the duplicated value below the one it read.
const OPCODE_DUP_X1: u8 = 0x5a;
/// The opcode of `pop` (JVMS 6.5): the discard of one category-1 value, which is what an
/// evaluated-but-unused receiver of a static call and an unread call result both compile to.
const OPCODE_POP: u8 = 0x57;

/// The `pop` instructions one body's own text already accounts for (P3 2c.31).
///
/// A `pop` is a compiler's way of discarding an evaluation, and both shapes it stands for are
/// written by the instruction beside it:
///
/// * `produce; pop; invokestatic` — when the producer cannot be emitted independently, the
///   qualified static call `expr.staticMethod()` writes the evaluated expression in its source
///   position; the value is discarded without a null check, and exceptions from evaluating the
///   expression remain before the static call;
/// * `invoke…; pop` — a call whose own value nothing reads: the `pop` discards exactly what the
///   call's statement discards, so the statement is the discard.
///
/// Both are found once per body ([`Builder::discarded_evaluations`]), from the instructions alone:
/// when both meet, the invocation's statement owns its result discard, so it is not also embedded
/// as the next call's qualifier. Which calls a local fills, which values are read and which labels
/// were verified are this build's other plans' business, and a `pop` neither shape states keeps the quote it had. `pop2` is never
/// one of them: it discards two slots, and no Java expression's evaluation is one category-2 value
/// (and one category-1 value beside it) in this spelling.
#[derive(Default)]
struct DiscardedEvaluations {
    /// Every `pop` a shape accounts for, in BCI order: it writes neither a statement nor a quote.
    accounted: BTreeSet<u32>,
    /// The evaluation one **static call** writes as its qualifier, keyed by the call's own BCI: the
    /// `pop` that discarded it and the value it discarded.
    qualifiers: BTreeMap<u32, (u32, ValueId)>,
    /// The `pop` that discards one **call's** own value, keyed by the call's own BCI.
    discards: BTreeMap<u32, u32>,
}

impl DiscardedEvaluations {
    /// Whether one instruction's text carries what a `pop` discarded, so that the `pop` writes
    /// nothing of its own.
    fn accounts_for(&self, at: u32) -> bool {
        self.accounted.contains(&at)
    }

    /// The evaluation one static call is written with, when the bytecode evaluated it and popped it:
    /// the `pop`'s BCI and the value it discarded.
    fn qualifier_at(&self, call: u32) -> Option<(u32, ValueId)> {
        self.qualifiers.get(&call).copied()
    }

    /// The `pop`s one instruction's own text accounts for: the qualifier a call writes and the call
    /// result its statement discards. A quote for that instruction names them beside it, or the
    /// artifact would refuse a call while dropping the `pop` whose evaluation its text took over.
    fn pops_of(&self, producer: u32) -> Vec<u32> {
        let mut pops = Vec::new();
        if let Some((pop, _)) = self.qualifiers.get(&producer) {
            pops.push(*pop);
        }
        if let Some(pop) = self.discards.get(&producer) {
            pops.push(*pop);
        }
        pops
    }
}

/// The `field++`/`++field` shapes one body's own facts state (P3 2c.10/2c.18).
///
/// The shapes are found once per body ([`Builder::field_increments`]) and read twice: every
/// instruction one of them accounts for is skipped, because the update its `return` states presents
/// it, and the statement those instructions become is written where the `return` is.
#[derive(Default)]
struct FieldIncrements {
    /// Every instruction the shapes account for, in BCI order. Nothing in the set writes a statement
    /// of its own, and the `return` that does is not one of them: it is the shape's own statement.
    owned: BTreeSet<u32>,
    /// The update one shape states, keyed by the `return` that carries what its copy left.
    statements: BTreeMap<u32, FieldIncrement>,
}

impl FieldIncrements {
    /// Whether one instruction's text is the update statement's, and not a statement of its own.
    fn owns(&self, at: u32) -> bool {
        self.owned.contains(&at)
    }

    /// The update one `return` states, when what it returns is a shape's leftover value.
    fn statement_at(&self, at: u32) -> Option<&FieldIncrement> {
        self.statements.get(&at)
    }

    /// Files one shape under every instruction it accounts for.
    fn add(&mut self, shape: FieldIncrement) {
        self.owned.extend(shape.anchors.iter().copied());
        self.owned.insert(shape.update);
        self.statements.insert(shape.returns, shape);
    }
}

/// One `field++`/`++field` shape: `load; dup; getfield; …; putfield; ireturn`, the eight
/// instructions of one field update whose value the method returns (P3 2c.10/2c.18,
/// [`Builder::increment_window`]).
struct FieldIncrement {
    /// The `putfield` that performs the update: the instruction the update's text is anchored at,
    /// because the text *is* that write.
    update: u32,
    /// The `ireturn` that states the update, whose operand is what the shape's copy left behind.
    returns: u32,
    /// The shape's other instructions, in BCI order: the anchors the update's text carries as
    /// derived ones, because the text presents what they read (the receiver load, the copy of it,
    /// the `getfield`, the constant, the `iadd`, and the copy the leftover value comes from).
    anchors: Vec<u32>,
    /// The whole update as text — `this.n++` or `++this.n` — from the **name** of the variable the
    /// receiver load reads and the member `field@1` named for both field instructions.
    text: String,
    /// The type the field's own descriptor states for the update's value, when this layer reads one:
    /// `n++` is that field's value, so it presents what the descriptor says.
    field_type: Option<Type>,
}

/// The one statement one `field++`/`++field` shape writes.
///
/// The update is the whole returned value, because what the method returns is the update's own: the
/// old value for `n++` and the new one for `++n` are what the shape's copy left, so `return
/// this.n++;` states both the write and the returned value — with no local, and without reading the
/// field once more for `++n`.
///
/// The AST this slice has writes no postfix or prefix operator of its own, and a *name* is the one
/// node the emitter writes verbatim: the update is therefore one [`ExprKind::Local`] whose text is
/// the whole `this.n++`, and every instruction it presents is carried by the expression's anchors
/// instead of by a node of its own. What the text says is still read and not guessed — the receiver
/// is the name the run gave the load's variable, the member is the `field@1` evidence's own, and
/// which operator it is comes from what the shape's copy duplicated.
fn increment_expression(increment: &FieldIncrement) -> Expr {
    let origin = increment.anchors.iter().fold(
        OriginSet::new(Origin::direct(increment.update)),
        |origin, bci| origin.plus_derived(Origin::derived(*bci)),
    );
    let update = Expr::new(ExprKind::Local(increment.text.clone()), origin);
    match &increment.field_type {
        Some(ty) => update.presenting(ty.clone()),
        None => update,
    }
}

/// Whether one value is what one instruction produced.
///
/// An instruction's copies are several values in the SSA — a `dup` writes its two copies as two
/// values, a `dup_x1` its three as three — and all of them name the same instruction, so identity
/// between a copy and the value it duplicates is the *definition* they share and not the number a run
/// gave either of them.
fn comes_from(ssa: &SsaTable, value: ValueId, bci: u32) -> bool {
    matches!(
        ssa.value(value).def(),
        Definition::Instruction { bci: produced, .. } if *produced == bci
    )
}

/// The one value one instruction reads off the operand stack, when it reads exactly one.
fn single_stack_read(instruction: &SsaInstruction) -> Option<(Slot, ValueId)> {
    match stack_operands(instruction).as_slice() {
        [read] => Some(*read),
        _ => None,
    }
}

/// The two values one `dup_x1` reads: the one it **duplicates** — the top of the stack — and the one
/// below it, which the copy it inserts keeps as the value under the duplicate.
fn dup_x1_operands(instruction: &SsaInstruction) -> Option<(ValueId, ValueId)> {
    // `stack_operands` orders by depth, so the last read is the top of the stack.
    match stack_operands(instruction).as_slice() {
        [(_, below), (_, top)] => Some((*top, *below)),
        _ => None,
    }
}

/// Whether one instruction reads exactly two values, one produced by the instruction at one of the
/// two BCIs and one by the other, in either order.
fn reads_from_two(ssa: &SsaTable, instruction: &SsaInstruction, first: u32, second: u32) -> bool {
    match stack_operands(instruction).as_slice() {
        [(_, left), (_, right)] => {
            (comes_from(ssa, *left, first) && comes_from(ssa, *right, second))
                || (comes_from(ssa, *left, second) && comes_from(ssa, *right, first))
        }
        _ => false,
    }
}

/// The value one **local** store takes off the stack, when the instruction is a store of one local
/// slot.
///
/// [`store_operand`] answers the value any writing instruction reads; this answers it only for the
/// store the shape P3 2c.14 verifies is made of. It is the *local* the store writes that decides
/// whether the store is a statement of the shape at all — an `iastore` writes no local and is not
/// one, and neither is a store whose slot this run cannot read.
fn local_store_value(operations: &Operations, instruction: &SsaInstruction) -> Option<ValueId> {
    if !matches!(
        operations.get(instruction.bci()),
        Some(Operation::Store { .. })
    ) {
        return None;
    }
    match instruction.writes() {
        [(Slot::Local(_), _)] => {}
        _ => return None,
    }
    stack_operands(instruction).last().map(|(_, value)| *value)
}

/// The value one writing instruction **reads** as the value it stores, when the two are different.
///
/// A `…; istore` writes the slot and reads the stack: the slot takes a value of the store's own, and
/// the value whose evidence states what the slot now holds is the one on the stack. An instruction
/// that writes a slot directly — an invocation whose result the SSA already places in the slot —
/// produces the value it writes, so it stores nothing it read, and `None` says so rather than naming
/// an argument of the call.
fn store_operand(operations: &Operations, instruction: &SsaInstruction) -> Option<ValueId> {
    match operations.get(instruction.bci()) {
        Some(Operation::Store { .. }) => {
            stack_operands(instruction).last().map(|(_, value)| *value)
        }
        _ => None,
    }
}

/// Whether one method descriptor's **return type** is `Z` (`(I)Z`, `()Z`, …).
///
/// The return side of the same reading [`MethodFacts::parameter_types`] makes for the parameters:
/// the frames state one `int` shape for the four int-sized primitives, so only a descriptor says
/// whether the position it types holds a `boolean` — and a call's result is where that fact types a
/// *value* ([`boolean_proof`]). The descriptor is read by the reader's own facts
/// ([`descriptor_facts`]) through [`return_type`], so the return position is the one component that
/// production states and not a substring of the text. A descriptor this reading cannot parse states
/// no return type at all, and a value under it keeps the type it was rendered with.
fn returns_boolean(descriptor: &str) -> bool {
    matches!(return_type(descriptor), Some(Type::Boolean))
}

/// Locate a read that Java can move to the enhanced-for binding before the first body statement.
/// Integer locals and integer additions before it cannot call, mutate, dereference or throw;
/// everything else, including a call argument or a field read, is deliberately outside this rule.
fn leading_wrapped_array_read(expr: &Expr) -> Option<&Expr> {
    match &expr.kind {
        ExprKind::Index { .. } => Some(expr),
        ExprKind::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } if expr.presented == Some(Type::Int) => leading_wrapped_array_read(left).or_else(|| {
            matches!(
                (&left.kind, &left.presented),
                (ExprKind::Local(_), Some(Type::Int)) | (ExprKind::Integer(_), Some(Type::Int))
            )
            .then(|| leading_wrapped_array_read(right))
            .flatten()
        }),
        _ => None,
    }
}

/// Change only the proven read node. The surrounding arithmetic and subsequent calls keep their
/// original evaluation order and anchors; failure leaves the cloned statement unpublished.
fn replace_wrapped_array_read(expr: &mut Expr, bci: u32, name: &str, ty: &Type) -> bool {
    if matches!(expr.kind, ExprKind::Index { .. }) && expr.origin.primary().bci() == bci {
        *expr =
            Expr::new(ExprKind::Local(name.to_owned()), expr.origin.clone()).presenting(ty.clone());
        return true;
    }
    if let ExprKind::Binary {
        op: BinaryOp::Add,
        left,
        right,
    } = &mut expr.kind
    {
        replace_wrapped_array_read(left, bci, name, ty)
            || replace_wrapped_array_read(right, bci, name, ty)
    } else {
        false
    }
}

/// Every local name and every bytecode index one statement's text states, in the order it states
/// them.
///
/// The walk is what P3 2b.2 is asked of a statement before it is published: the names its text
/// spells have to belong to locals this body declared, and the BCIs of its own subtree are what a
/// quote for the whole statement has to name instead. A **fallback** states no name of its own (its
/// text is a comment) and its own quoted BCIs are kept beside the statement's, so a refused
/// statement inside it is accounted for like any other.
///
/// A `Declare`'s own name is *not* collected: the statement is the declaration, and the walk is
/// asked about the names a statement **reads**.
fn stated_by_statement(stmt: &Stmt, names: &mut Vec<String>, bcis: &mut Vec<u32>) {
    bcis.extend(stmt.origin.bcis());
    match &stmt.kind {
        StmtKind::Declare { value, .. } => {
            if let Some(value) = value {
                stated_by_expression(value, names, bcis);
            }
        }
        StmtKind::Assign { name, value } => {
            names.push(name.clone());
            stated_by_expression(value, names, bcis);
        }
        StmtKind::Expr(expr) => stated_by_expression(expr, names, bcis),
        StmtKind::FieldAssign {
            receiver, value, ..
        } => {
            if let Some(receiver) = receiver {
                stated_by_expression(receiver, names, bcis);
            }
            stated_by_expression(value, names, bcis);
        }
        StmtKind::ConstructorCall { args, .. } => {
            for arg in args {
                stated_by_expression(arg, names, bcis);
            }
        }
        StmtKind::Return { value } => {
            if let Some(value) = value {
                stated_by_expression(value, names, bcis);
            }
        }
        StmtKind::Throw { value } => stated_by_expression(value, names, bcis),
        StmtKind::If {
            cond,
            then_body,
            else_body,
        } => {
            stated_by_expression(cond, names, bcis);
            for stmt in then_body.iter().chain(else_body) {
                stated_by_statement(stmt, names, bcis);
            }
        }
        StmtKind::While { cond, body, .. } | StmtKind::DoWhile { cond, body, .. } => {
            stated_by_expression(cond, names, bcis);
            for stmt in body {
                stated_by_statement(stmt, names, bcis);
            }
        }
        StmtKind::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            stated_by_statement(init, names, bcis);
            stated_by_expression(cond, names, bcis);
            stated_by_statement(update, names, bcis);
            for stmt in body {
                stated_by_statement(stmt, names, bcis);
            }
        }
        StmtKind::ForEach { iterable, body, .. } => {
            stated_by_expression(iterable, names, bcis);
            for stmt in body {
                stated_by_statement(stmt, names, bcis);
            }
        }
        StmtKind::Switch { value, arms } => {
            stated_by_expression(value, names, bcis);
            for arm in arms {
                for stmt in &arm.body {
                    stated_by_statement(stmt, names, bcis);
                }
            }
        }
        StmtKind::Try {
            resources,
            catches,
            body,
            finally_body,
        } => {
            for resource in resources {
                stated_by_expression(&resource.value, names, bcis);
            }
            for stmt in catches.iter().flat_map(|clause| &clause.body).chain(body) {
                stated_by_statement(stmt, names, bcis);
            }
            if let Some(finally_body) = finally_body {
                for stmt in finally_body {
                    stated_by_statement(stmt, names, bcis);
                }
            }
        }
        StmtKind::Synchronized { lock, body } => {
            stated_by_expression(lock, names, bcis);
            for stmt in body {
                stated_by_statement(stmt, names, bcis);
            }
        }
        StmtKind::IndexAssign {
            array,
            index,
            value,
            ..
        } => {
            stated_by_expression(array, names, bcis);
            stated_by_expression(index, names, bcis);
            stated_by_expression(value, names, bcis);
        }
        StmtKind::Fallback { bcis: quoted, .. } => bcis.extend(quoted),
        StmtKind::Break { .. } | StmtKind::Continue { .. } => {}
    }
}

/// Every local name and every bytecode index one expression's text states.
///
/// A lambda's own parameter list declares its parameters in the text itself, so the names it states
/// there are not collected: what the walk is asked about is the names an expression **reads**.
fn stated_by_expression(expr: &Expr, names: &mut Vec<String>, bcis: &mut Vec<u32>) {
    bcis.extend(expr.origin.bcis());
    match &expr.kind {
        ExprKind::Local(name) => names.push(name.clone()),
        ExprKind::Call { receiver, args, .. } => {
            if let Some(receiver) = receiver {
                stated_by_expression(receiver, names, bcis);
            }
            for arg in args {
                stated_by_expression(arg, names, bcis);
            }
        }
        ExprKind::New {
            qualifier, args, ..
        } => {
            if let Some(qualifier) = qualifier {
                stated_by_expression(qualifier, names, bcis);
            }
            for arg in args {
                stated_by_expression(arg, names, bcis);
            }
        }
        ExprKind::NewArray {
            lengths,
            initializers,
            ..
        } => {
            for arg in lengths {
                stated_by_expression(arg, names, bcis);
            }
            if let Some(initializers) = initializers {
                for initializer in initializers {
                    stated_by_expression(initializer, names, bcis);
                }
            }
        }
        ExprKind::Lambda { body, .. } => stated_by_expression(body, names, bcis),
        ExprKind::MethodReference { qualifier, .. } => {
            stated_by_expression(qualifier, names, bcis);
        }
        ExprKind::Field { receiver, .. }
        | ExprKind::ArrayLength { array: receiver }
        | ExprKind::PostIncrement { target: receiver } => {
            stated_by_expression(receiver, names, bcis);
        }
        ExprKind::Index { array, index } => {
            stated_by_expression(array, names, bcis);
            stated_by_expression(index, names, bcis);
        }
        ExprKind::Binary { left, right, .. } => {
            stated_by_expression(left, names, bcis);
            stated_by_expression(right, names, bcis);
        }
        ExprKind::Conditional {
            test,
            when_true,
            when_false,
        } => {
            stated_by_expression(test, names, bcis);
            stated_by_expression(when_true, names, bcis);
            stated_by_expression(when_false, names, bcis);
        }
        ExprKind::Concat { parts } => {
            for part in parts {
                stated_by_expression(&part.value, names, bcis);
            }
        }
        ExprKind::Cast { value, .. }
        | ExprKind::InstanceOf { value, .. }
        | ExprKind::Not { value }
        | ExprKind::Neg { value } => {
            stated_by_expression(value, names, bcis);
        }
        // A literal, a name used as a type, `null`, a boolean and an integer state no local name
        // and no further text.
        ExprKind::Integer(_)
        | ExprKind::Boolean(_)
        | ExprKind::Long(_)
        | ExprKind::Float(_)
        | ExprKind::Double(_)
        | ExprKind::Str(_)
        | ExprKind::Null
        | ExprKind::ClassLiteral { .. }
        | ExprKind::Path(_)
        | ExprKind::Super { .. } => {}
    }
}

/// The one name a statement spells that no statement of this body declared, with every bytecode
/// index its own subtree names.
///
/// `None` is the answer for every statement a text may publish (P3 2b.2). The BCIs are returned with
/// the name because the statement a refusal replaces has to be accounted for exactly as it was: the
/// quote names the instructions the statement's own text would have named, nothing more and nothing
/// less.
fn undeclared_local(stmt: &Stmt, undeclared: &BTreeSet<String>) -> Option<(String, Vec<u32>)> {
    let mut names = Vec::new();
    let mut bcis = Vec::new();
    stated_by_statement(stmt, &mut names, &mut bcis);
    bcis.sort_unstable();
    bcis.dedup();
    names
        .into_iter()
        .find(|name| undeclared.contains(name))
        .map(|name| (name, bcis))
}

/// The value one instruction reads out of one local slot, when it reads that slot at all.
///
/// A load of a slot *produces* a value rather than consuming one, so this is deliberately not
/// [`stack_operands`]: it is what the slot held where the instruction ran, which is what the value
/// the instruction yields stands for — and therefore what a reader of that value means.
fn local_read(instruction: &SsaInstruction, slot: u16) -> Option<ValueId> {
    instruction
        .reads()
        .iter()
        .find_map(|(read, value)| match read {
            Slot::Local(read) if *read == slot => Some(*value),
            _ => None,
        })
}

/// The values one instruction reads off the operand stack, ordered by depth.
///
/// A local read is not an operand: an instruction that loads a local *produces* a value, and reading
/// the local here would print `x = x` for a store that loads the slot it writes. A statement's
/// operands are exactly the stack values the instruction consumes, from the bottom of the consumed
/// region upwards, which is the order a call's receiver and arguments are written in.
///
/// The pattern rules of P3 2.2 read the same operands (the `append` an operand belongs to, the
/// instance a bridge must have been given, the receiver and value of an accessor call), so this is
/// the one reader of them.
pub(crate) fn stack_operands(instruction: &SsaInstruction) -> Vec<(Slot, ValueId)> {
    let mut reads: Vec<(Slot, ValueId)> = instruction
        .reads()
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Stack(_)))
        .copied()
        .collect();
    reads.sort_by_key(|(slot, _)| match slot {
        Slot::Stack(depth) => *depth,
        Slot::Local(slot) => u32::from(*slot),
    });
    reads
}

/// The expression a constant becomes.
fn literal(constant: &ConstantValue) -> ExprKind {
    match constant {
        ConstantValue::Int(value) => ExprKind::Integer(*value),
        ConstantValue::Long(value) => ExprKind::Long(*value),
        // The leaf keeps the bits the class file stated; the emitter spells them exactly.
        ConstantValue::Float(bits) => ExprKind::Float(*bits),
        ConstantValue::Double(bits) => ExprKind::Double(*bits),
        ConstantValue::String(value) => ExprKind::Str(value.clone()),
        ConstantValue::Null => ExprKind::Null,
        ConstantValue::Class { ty, .. } => ExprKind::ClassLiteral { ty: ty.clone() },
    }
}

/// The finite leaves the proved special-value divisions are built from.
const FLOAT_ONE_BITS: u32 = 0x3f80_0000;
const FLOAT_MINUS_ONE_BITS: u32 = 0xbf80_0000;
const FLOAT_ZERO_BITS: u32 = 0;
const DOUBLE_ONE_BITS: u64 = 0x3ff0_0000_0000_0000;
const DOUBLE_MINUS_ONE_BITS: u64 = 0xbff0_0000_0000_0000;
const DOUBLE_ZERO_BITS: u64 = 0;

/// The expression one special floating constant is presented as: the proved constant division of
/// two finite leaves (`1/0`, `-1/0`, `0/0`), every node of it derived from the constant's own BCI.
///
/// This is the constant's *presentation*, not an invented instruction: no BCI of the bytecode is
/// claimed as a division, the division node and both of its leaves derive from the one `ldc` (or
/// `fconst`/`dconst`) the class file states, and the division keeps the constant's own type. The
/// leaves are finite by construction, so the text recompiles to a constant expression whose bits
/// the compiler computes exactly as the JVM does — `1/0` is positive infinity, `-1/0` negative
/// infinity, and `0/0` the standard positive quiet NaN.
fn special_value(
    bci: u32,
    presentation: SpecialValuePresentation,
    width: SpecialValueWidth,
) -> Expr {
    let origin = || OriginSet::new(Origin::derived(bci));
    let leaf = |bits: LeafBits| match width {
        SpecialValueWidth::Float => Expr::new(ExprKind::Float(bits.float_bits()), origin()),
        SpecialValueWidth::Double => Expr::new(ExprKind::Double(bits.double_bits()), origin()),
    };
    let (numerator, denominator) = match presentation {
        SpecialValuePresentation::PositiveInfinity => (LeafBits::One, LeafBits::Zero),
        SpecialValuePresentation::NegativeInfinity => (LeafBits::MinusOne, LeafBits::Zero),
        SpecialValuePresentation::CanonicalNan => (LeafBits::Zero, LeafBits::Zero),
        // Finite and unpresentable patterns never reach this presentation: the caller matches on
        // the same classification this function's arms name.
        _ => unreachable!("a finite or unpresentable pattern has no division presentation"),
    };
    Expr::new(
        ExprKind::Binary {
            op: BinaryOp::Divide,
            left: Box::new(leaf(numerator)),
            right: Box::new(leaf(denominator)),
        },
        origin(),
    )
}

/// One finite floating leaf of a proved division, by its value: `1`, `-1` or the signed `0`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LeafBits {
    One,
    MinusOne,
    Zero,
}

impl LeafBits {
    /// The leaf's `float` bits.
    fn float_bits(self) -> u32 {
        match self {
            Self::One => FLOAT_ONE_BITS,
            Self::MinusOne => FLOAT_MINUS_ONE_BITS,
            Self::Zero => FLOAT_ZERO_BITS,
        }
    }

    /// The leaf's `double` bits.
    fn double_bits(self) -> u64 {
        match self {
            Self::One => DOUBLE_ONE_BITS,
            Self::MinusOne => DOUBLE_MINUS_ONE_BITS,
            Self::Zero => DOUBLE_ZERO_BITS,
        }
    }
}

/// The width one floating constant presentation keeps: the constant's own, never widened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpecialValueWidth {
    Float,
    Double,
}

/// Whether this class's own simple name occupies one component of a qualified Class literal path.
///
/// A type in the current compilation unit can take precedence over a package of the same name, so
/// `java.lang.String.class` cannot name `java.lang.String` from inside a class named `java`. Local
/// variables do not create this ambiguity in a class-literal type context. A single-class run has
/// no evidence about unrelated same-package types, so this bounded check uses only the declaration
/// it already carries and does not claim a cross-class name-resolution proof.
fn class_literal_path_is_shadowed(ty: &str, declaring_class: Option<&str>) -> bool {
    let Some(simple_name) = declaring_class.and_then(|name| name.rsplit('/').next()) else {
        return false;
    };
    let base = ty.trim_end_matches("[]");
    base != simple_name && base.split('.').next() == Some(simple_name)
}

/// The simple source type name this run may use for its own internal class name.
///
/// A `$` may separate a nested class from its top-level owner, but this header has no evidence
/// that the binary name is itself a top-level Java identifier. Rejecting it avoids writing a name
/// that could denote a different declaration; package components are checked by the same
/// spellability policy as the simple name.
fn current_class_simple_name(internal: &str) -> Option<String> {
    let mut simple = None;
    for component in internal.split('/') {
        if component.is_empty() || component.contains('$') || !is_java_identifier(component) {
            return None;
        }
        simple = Some(component);
    }
    simple.map(str::to_owned)
}

/// One expression whose value a boolean context has proven boolean, spelled as that boolean.
///
/// The only value whose *text* changes is the literal: the frames state `0`/`1` for a boolean
/// literal exactly as they do for an `int` one, and the boolean context is the fact that says which
/// of the two the bytecode pushed — the same reading [`typed_arguments`] makes for a `Z` argument.
/// Every other proven value (a `boolean` parameter's load, a call whose callee descriptor returns
/// `Z`, a claimed field read of descriptor `Z`, a read of a local this body declared `boolean`)
/// already prints what it is, and an expression of any other type is left untouched: the caller is
/// the only place that knows its context, and this helper never invents one.
fn boolean_spelling(expr: Expr) -> Expr {
    let spelled = match &expr.kind {
        ExprKind::Integer(0) => false,
        ExprKind::Integer(1) => true,
        // Everything else keeps the expression exactly as it was, **including** the type it presents
        // as: a value that already prints what it is says nothing new here, and the caller is the
        // only place that knows its context.
        _ => return expr,
    };
    let Expr { origin, .. } = expr;
    Expr::new(ExprKind::Boolean(spelled), origin)
}

/// Spell the low bit consumed by a boolean field write, actual `ireturn Z`, or proved `bastore`.
/// Admission belongs to the consuming site; this only builds one owned operand subtree.
fn integer_low_bit_boolean(value: Expr, bci: u32) -> Expr {
    let origin = value.origin.clone().plus_derived(Origin::derived(bci));
    let remainder = Expr::new(
        ExprKind::Binary {
            op: BinaryOp::Remainder,
            left: Box::new(value),
            right: Box::new(Expr::direct(ExprKind::Integer(2), bci)),
        },
        origin.clone(),
    );
    Expr::new(
        ExprKind::Binary {
            op: BinaryOp::NotEqual,
            left: Box::new(remainder),
            right: Box::new(Expr::direct(ExprKind::Integer(0), bci)),
        },
        origin,
    )
}

/// The operator an arithmetic operation becomes.
fn arithmetic_op(op: ArithmeticOp) -> BinaryOp {
    match op {
        ArithmeticOp::Add => BinaryOp::Add,
        ArithmeticOp::Subtract => BinaryOp::Subtract,
        ArithmeticOp::Multiply => BinaryOp::Multiply,
        ArithmeticOp::Divide => BinaryOp::Divide,
        ArithmeticOp::Remainder => BinaryOp::Remainder,
    }
}

fn shift_op(op: ShiftOp) -> BinaryOp {
    match op {
        ShiftOp::Left => BinaryOp::LeftShift,
        ShiftOp::Right => BinaryOp::RightShift,
        ShiftOp::UnsignedRight => BinaryOp::UnsignedRightShift,
    }
}

/// The source operator an integral/boolean bitwise instruction performs.
fn bitwise_op(op: BitwiseOp) -> BinaryOp {
    match op {
        BitwiseOp::And => BinaryOp::BitwiseAnd,
        BitwiseOp::Or => BinaryOp::BitwiseOr,
        BitwiseOp::Xor => BinaryOp::BitwiseXor,
    }
}

/// The condition one sense of a branch states, as the structure that holds it writes it.
///
/// `taken = false` writes the **fall-through** condition — the sense negated, which is what an `if`
/// statement tests: a branch transfers only when its sense holds, so control stays in the `if` when
/// it does not. `taken = true` writes the sense itself, which is what a loop tests when the branch's
/// target is the block that iterates. Writing either one here — once, next to the sense's meaning —
/// is what makes an emitted condition a consequence of the decoded fact rather than of the order two
/// successors happened to be published in.
///
/// The condition node is anchored where the value it tests was produced, and the branch that tests it
/// is added as a *presented* anchor by the caller: the text is the value's, the test is the branch's,
/// and keeping those two apart is what makes a shared BCI visible in the segment table with both
/// provenances instead of one node claiming both.
fn condition(
    op: CompareOp,
    operands: &[(Slot, ValueId)],
    builder: &mut Builder<'_>,
    branch_bci: u32,
    taken: bool,
) -> Result<Expr, ValueRenderFailure> {
    let expected = if op.reads_two() { 2 } else { 1 };
    if operands.len() != expected {
        return Err(format!(
            "the branch at BCI {branch_bci} reads {} values, but its decoded sense needs {expected}",
            operands.len()
        ).into());
    }
    // Whether this **position** requires a boolean is decided before any operand is spelled, and
    // from the shape of the test rather than from the shape of a value. A zero test whose operand is
    // **proven boolean** is not a comparison (P3-R5): the frames state one slot shape for the four
    // int-sized primitives, so the fact that decides the text there is a descriptor — `ifeq` on a
    // `boolean` parameter is `!b`, `ifne` is `b`, and any other zero test on it (`iflt`, `ifgt`, …)
    // has no Java spelling at all — the signature would refuse it — so the region is refused rather
    // than written as an int comparison. An integer binary comparison (`if_icmp*`) has no such
    // requirement: it reads two `int`s, and the fact that its *result* is a boolean says nothing
    // about its operands, so both keep the spelling their own evidence states. An operand the layer
    // cannot prove boolean keeps the integer comparison it has always had: that is not a defect, and
    // refusing it would trade the two shapes this rule is about for a layer that refuses every `int`
    // branch.
    //
    // The order is the regression this rule corrects. `boolean_value` counts the `0`/`1` literal as
    // one item of the proof, and a literal is a boolean only where the position already requires
    // one: reading it before the test's shape was known spelled the left operand of
    // `iconst_1; iload_0; if_icmpne` as `true` (`5a8c36a`), text javac refuses
    // (`incomparable types: boolean and int`), where the same bytes had been `1 == arg0`.
    //
    // The operator the *sense* states, and the operator its negation states.
    let (positive, negative) = match op {
        CompareOp::JumpIfZero => (Test::Zero(BinaryOp::Equal), Test::Zero(BinaryOp::NotEqual)),
        CompareOp::JumpIfNotZero => (Test::Zero(BinaryOp::NotEqual), Test::Zero(BinaryOp::Equal)),
        CompareOp::JumpIfNegative => (
            Test::Zero(BinaryOp::Less),
            Test::Zero(BinaryOp::GreaterOrEqual),
        ),
        CompareOp::JumpIfNotNegative => (
            Test::Zero(BinaryOp::GreaterOrEqual),
            Test::Zero(BinaryOp::Less),
        ),
        CompareOp::JumpIfPositive => (
            Test::Zero(BinaryOp::Greater),
            Test::Zero(BinaryOp::LessOrEqual),
        ),
        CompareOp::JumpIfNotPositive => (
            Test::Zero(BinaryOp::LessOrEqual),
            Test::Zero(BinaryOp::Greater),
        ),
        CompareOp::JumpIfNull => (Test::Null(BinaryOp::Equal), Test::Null(BinaryOp::NotEqual)),
        CompareOp::JumpIfNotNull => (Test::Null(BinaryOp::NotEqual), Test::Null(BinaryOp::Equal)),
        CompareOp::JumpIfSame => (Test::Pair(BinaryOp::Equal), Test::Pair(BinaryOp::NotEqual)),
        CompareOp::JumpIfDifferent => (Test::Pair(BinaryOp::NotEqual), Test::Pair(BinaryOp::Equal)),
        CompareOp::JumpIfLess => (
            Test::Pair(BinaryOp::Less),
            Test::Pair(BinaryOp::GreaterOrEqual),
        ),
        CompareOp::JumpIfLessOrEqual => (
            Test::Pair(BinaryOp::LessOrEqual),
            Test::Pair(BinaryOp::Greater),
        ),
        CompareOp::JumpIfGreater => (
            Test::Pair(BinaryOp::Greater),
            Test::Pair(BinaryOp::LessOrEqual),
        ),
        CompareOp::JumpIfGreaterOrEqual => (
            Test::Pair(BinaryOp::GreaterOrEqual),
            Test::Pair(BinaryOp::Less),
        ),
    };
    let test = if taken { positive } else { negative };
    // A numeric compare has a signed result rather than the two operands of an integer branch. It
    // is composed here from the already selected zero predicate, only when the SSA proves the
    // single adjacent zero-branch reader.
    if let Test::Zero(test_op) = test
        && let Some(result) = numeric_condition(test_op, operands, builder, branch_bci)
    {
        return result;
    }
    // A pair comparison is the one branch shape that carries a *requirement of its own* about its
    // operands: it reads two `int`s (or, for a null test, two references), so an operand this layer
    // proves boolean beside one it does not has no Java spelling at all — javac refuses
    // `flag() == 1` with `incomparable types: boolean and int` — and ordering two booleans has none
    // either. The two are refused before any operand is rendered, as one rule: a comparison the
    // layer's own evidence says cannot be spelled is not published with a type nobody could accept.
    // The predecessor decision that fixed the *spelling* of a pair's operands (`1 == arg0` keeps its
    // integer literal, each operand keeps its own evidence) is untouched by this: what is refused is
    // a pair whose two spellings cannot stand in one comparison.
    if let Test::Pair(op) = test
        && let Some(reason) = builder.pair_comparison_refusal(op, operands, branch_bci)
    {
        return Err(reason.into());
    }
    // The position's requirement, and only it, is what may read the literal proof: a boolean is
    // required here exactly where the test is a zero test on a value the evidence owns. The operands
    // are rendered **after** this decision, and each keeps its own evidence: `render_value` spells a
    // `Test::Pair` operand as the `int` it is, and `boolean_spelling` — the `0`/`1` to `false`/`true`
    // adaptation — happens inside the truth-test branch below and nowhere else.
    let boolean = matches!(test, Test::Zero(_)) && builder.boolean_value(operands[0].1, branch_bci);
    let left = builder.render_value(operands[0].1, branch_bci, 0)?;
    let anchor = left.origin.primary().bci();
    if let Test::Zero(op) = test
        && boolean
    {
        let left = boolean_spelling(left);
        return match op {
            BinaryOp::NotEqual => Ok(left),
            BinaryOp::Equal => Ok(Expr::new(
                ExprKind::Not {
                    value: Box::new(left),
                },
                OriginSet::new(crate::source_map::Origin::direct(anchor)),
            )),
            other => Err(format!(
                "the branch at BCI {branch_bci} tests a value proven boolean with `{}`, which no Java source spells",
                other.spell()
            ).into()),
        };
    }
    match test {
        // The second operand is rendered only where it is written, so a zero or null test does not
        // ask the value flow for a value the instruction never read.
        Test::Zero(op) => Ok(binary(op, left, zero(branch_bci), anchor)),
        Test::Null(op) => Ok(binary(
            op,
            left,
            Expr::direct(ExprKind::Null, branch_bci),
            anchor,
        )),
        Test::Pair(op) => {
            let right = builder.render_value(operands[1].1, branch_bci, 0)?;
            Ok(binary(op, left, right, anchor))
        }
    }
}

/// Composes one proven numeric comparison with the zero predicate its branch actually tests.
fn numeric_condition(
    predicate: BinaryOp,
    operands: &[(Slot, ValueId)],
    builder: &mut Builder<'_>,
    branch_bci: u32,
) -> Option<Result<Expr, ValueRenderFailure>> {
    let [(Slot::Stack(_), result)] = operands else {
        return None;
    };
    let Definition::Instruction {
        bci: compare_bci, ..
    } = builder.ssa.value(*result).def()
    else {
        return None;
    };
    let compare_bci = *compare_bci;
    let Some(Operation::NumericComparison { op: numeric_op }) = builder.operations.get(compare_bci)
    else {
        return None;
    };
    let numeric_op = *numeric_op;
    let Some(instruction) = builder.instructions.get(&compare_bci).copied() else {
        return Some(Err(format!(
            "no names record for the numeric comparison at BCI {compare_bci}"
        )
        .into()));
    };
    if let Err(reason) = builder.numeric_comparison_reader(instruction) {
        return Some(Err(reason.into()));
    }
    let compare_operands = stack_operands(instruction);
    let [(_, left_value), (_, right_value)] = compare_operands.as_slice() else {
        return Some(Err(format!(
            "the numeric comparison at BCI {compare_bci} does not read exactly two operands"
        )
        .into()));
    };
    let (binary_op, negate) = numeric_relation(numeric_op, predicate);
    let left = match builder.render_value(*left_value, branch_bci, 0) {
        Ok(value) => value,
        Err(reason) => return Some(Err(reason)),
    };
    let right = match builder.render_value(*right_value, branch_bci, 0) {
        Ok(value) => value,
        Err(reason) => return Some(Err(reason)),
    };
    let mut expression = binary(binary_op, left, right, branch_bci);
    if negate {
        let origin = expression.origin.clone();
        expression = Expr::new(
            ExprKind::Not {
                value: Box::new(expression),
            },
            origin,
        );
    }
    // The final condition is anchored at the branch and retains the comparison as a derived
    // producer source; the caller adds the same branch ownership to the containing statement.
    Some(Ok(expression.derived_from(compare_bci)))
}

/// Maps the signed result predicate while retaining the floating-point unordered bias.
fn numeric_relation(op: NumericComparisonOp, predicate: BinaryOp) -> (BinaryOp, bool) {
    match op {
        NumericComparisonOp::Long => (predicate, false),
        NumericComparisonOp::FloatGreater | NumericComparisonOp::DoubleGreater => match predicate {
            BinaryOp::Greater => (BinaryOp::LessOrEqual, true),
            BinaryOp::GreaterOrEqual => (BinaryOp::Less, true),
            _ => (predicate, false),
        },
        NumericComparisonOp::FloatLess | NumericComparisonOp::DoubleLess => match predicate {
            BinaryOp::Less => (BinaryOp::GreaterOrEqual, true),
            BinaryOp::LessOrEqual => (BinaryOp::Greater, true),
            _ => (predicate, false),
        },
    }
}

/// The comparison one sense of a branch states, before its operands are rendered.
enum Test {
    /// One value against the constant zero, with the operator the sense states.
    Zero(BinaryOp),
    /// One reference against `null`.
    Null(BinaryOp),
    /// The two values the branch reads.
    Pair(BinaryOp),
}

/// A binary expression anchored at one bytecode index.
fn binary(op: BinaryOp, left: Expr, right: Expr, bci: u32) -> Expr {
    Expr::direct(
        ExprKind::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
        bci,
    )
}

/// The `0` a zero test compares against, anchored where the test is.
fn zero(bci: u32) -> Expr {
    Expr::direct(ExprKind::Integer(0), bci)
}

/// Whether one value is read from a parameter slot the member's descriptor declares `boolean`.
///
/// The free functions of this module read the same fact the builder's own method reads: the value has
/// to be the result of a single `load` of that slot (P3-R5).
fn parameter_boolean(
    ssa: &SsaTable,
    operations: &Operations,
    parameter_types: &BTreeMap<u16, Type>,
    value: ValueId,
) -> bool {
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return false;
    };
    let Some(Operation::Load { slot }) = operations.get(*bci) else {
        return false;
    };
    matches!(parameter_types.get(slot), Some(Type::Boolean))
}

/// The class name one dynamic site's implementation handle states, spelled as Java source.
///
/// The handle's owner is the class the implementation member lives in: the qualifier of a `::`
/// reference, the type a constructor's `new` states, the receiver of a static call. One shape names
/// it in three places, so it is spelled once here — and a class name this layer cannot spell as a
/// Java type refuses the site instead of being written as the pool spells it ([`spell_reference`]).
fn implementation_owner(plan: &lambda::Plan, bci: u32) -> Result<String, String> {
    spell_reference(plan.implementation.owner()).ok_or_else(|| {
        format!(
            "the dynamic site at BCI {bci} implements `{}` in the class `{}`, which this layer cannot spell as a Java type",
            plan.implementation.name(),
            plan.implementation.owner()
        )
    })
}

/// The declared type of a local, when the frames state one.
///
/// A reference the frames *name* carries the descriptor form the class file states
/// (`Ljava/lang/Runnable;`, and every array as its own descriptor: `[B`, `[[Ljava/lang/String;`) —
/// that is the fact the frame pass read and kept — so this is where it is spelled as Java source
/// (`java.lang.Runnable`, `byte[]`, `java.lang.String[][]`).
///
/// `Err` carries the fact that has no spelling: the reference name the frames state, which
/// [`spell_reference`] refused as a descriptor with no Java type to be (`[`, `[Lfoo`, `L;`). The
/// caller states the position that would have carried it and refuses that position — never the
/// name, and never a placeholder type in its place.
fn value_type(value: &Value) -> Result<Option<Type>, String> {
    Ok(match value {
        Value::Int => Some(Type::Int),
        Value::Long => Some(Type::Long),
        Value::Float => Some(Type::Float),
        Value::Double => Some(Type::Double),
        Value::Ref(RefType::Named { name, .. }) => {
            let name = String::from_utf8_lossy(name).into_owned();
            Some(Type::Reference(spell_reference(&name).ok_or(name)?))
        }
        Value::Ref(_) | Value::Null => Some(Type::Reference("Object".to_string())),
        _ => None,
    })
}

/// One reference type as the frames and the class file state it, spelled as Java source.
///
/// Published to the crate because four rules write a type name from a class-file fact — a
/// declaration's class, a construction's class, a static field's owner, a dynamic site's
/// implementation class — and a second spelling of "internal form to source form" is exactly the
/// kind of duplicate that drifts.
///
/// Three forms reach this entry point, and `None` is the answer for a form that states no Java type:
///
/// * a **field descriptor** — the frames state a named reference this way (`Ljava/lang/Runnable;`),
///   and so does every array (`[B`, `[[I`, `[[Ljava/lang/String;`). What a descriptor describes is a
///   type, so it must be spelled as one: the `L…;` wrapping comes off, `/` becomes `.`, and an array
///   is spelled from the element outwards with one `[]` per dimension. The array spelling itself is
///   [`crate::lambda::parse_type`]'s, reused rather than copied, so a declaration here and a lambda
///   parameter there cannot drift. A descriptor that cannot be read as a type — a truncated array
///   (`[`, `[V`, `[Lfoo`), a class type with no name (`L;`) — has no Java spelling to publish and is
///   `None`.
/// * a **method descriptor** (`(I)V`) is not a type at all: `None`.
/// * an **internal name** (`java/lang/Math`, the pool's own spelling of a class) is not a descriptor
///   and keeps the historical reading, `/` → `.`. A bare name is never interpreted as one: a class
///   may be called `Lfoo`, so a name that merely looks like a truncated descriptor stays a name.
pub(crate) fn spell_reference(descriptor: &str) -> Option<String> {
    match descriptor.as_bytes().first()? {
        // An array descriptor is read by the one descriptor parser of this crate: the elements it
        // spells are the object-name rule below, applied to the element the descriptor states.
        b'[' => match crate::lambda::parse_type(descriptor.as_bytes(), 0) {
            Some((Type::Reference(spelled), end)) if end == descriptor.len() => Some(spelled),
            _ => None,
        },
        b'(' => None,
        b'L' => match descriptor
            .strip_prefix('L')
            .and_then(|rest| rest.strip_suffix(';'))
        {
            // The object-descriptor form: its interior is the class name, which is never empty and
            // never carries the `;` that would end it.
            Some(internal) if !internal.is_empty() && !internal.contains(';') => {
                Some(internal.replace('/', "."))
            }
            Some(_) => None,
            // No `L…;` wrapping to take off: the pool's own spelling of a class, kept as it is.
            None => Some(descriptor.replace('/', ".")),
        },
        _ => Some(descriptor.replace('/', ".")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_proved_member_construction_keeps_the_qualifier_and_hides_only_the_physical_prefix() {
        use jarde_jvm::engine::analyze_method_ir;
        use jarde_jvm::environment::ResolutionEnvironment;
        use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
        use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
        use jarde_reader::model::{
            ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
            PhysicalMethodId, PhysicalVariant,
        };
        use jarde_reader::view::{
            DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
            MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
            RuntimeView,
        };

        const CALLER: &[u8] = include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/classes/nested/UseInner.class"
        );
        let mut budget = Budget::new(jarde_reader::budget::Limits {
            input_bytes: 1 << 20,
            archive_entries: 100,
            entry_bytes: 1 << 20,
            read_bytes: 1 << 20,
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            output_bytes: 1 << 20,
            class_headers: 100,
            method_bodies: 100,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            nested_depth: 32,
            dependency_depth: 32,
            elapsed_millis: u64::MAX,
        });
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(CALLER.to_vec()), &mut budget)
            .expect("the frozen caller class opens");
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(CALLER).to_hex().to_string()),
                length: u64::try_from(CALLER.len()).expect("fixture length fits u64"),
            },
            variant: PhysicalVariant::Base,
        };
        let domain = LoadDomain {
            loader: LoaderId("app".to_string()),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots: vec![LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        };
        let environment = ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        };
        let descriptor = "(Lnested/SimpleOuter;I)Ljava/lang/Object;";
        let analysis = analyze_method_ir(
            &[snapshot],
            &MethodAnalysisRequest {
                environment,
                method: PhysicalMethodId {
                    owner: definition.clone(),
                    name: JvmBytes(b"make".to_vec()),
                    descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
                },
                stages: AnalysisStage::ALL.to_vec(),
            },
            &mut budget,
        )
        .expect("the frozen method completes JVM IR analysis");
        let method = crate::facts::MethodFacts::new("make", descriptor, 2)
            .with_access_flags(0x0009)
            .with_declaring_class(crate::facts::DeclaringClass::new("nested/UseInner", 0x0031));
        let facts = crate::facts::RecoveryFacts::new(method);
        let targets = [crate::report::ProvedMemberInnerTarget {
            // This direct layer test exercises the already-proved handoff shape. The class-source
            // adapter separately proves and supplies the target's actual physical definition.
            definition,
            owner: "nested/SimpleOuter$Inner".to_string(),
            outer: "nested/SimpleOuter".to_string(),
            simple_name: "Inner".to_string(),
            constructor_descriptor: "(Lnested/SimpleOuter;I)V".to_string(),
            capture_field: "this$0".to_string(),
            generic_diamond: false,
            source_type_path: Vec::new(),
        }];
        let request =
            crate::report::RecoveryRequest::new(analysis.ir(), &facts, crate::pass::JAVA_8)
                .with_member_inner_targets(&targets)
                .with_evidence(crate::evidence::RecoveryEvidenceRequest::all());
        let report = crate::report::recover(&request, &mut budget);

        assert!(report.produced(), "{:?}", report.stop());
        assert_eq!(report.representation, jarde_jvm::ir::Representation::Java);
        assert_eq!(report.quality, jarde_jvm::ir::Quality::Structured);
        assert!(
            report
                .text
                .contains("return arg0.new Inner(nested.SimpleOuter.mark(\"A\", arg1));"),
            "{}",
            report.text
        );
        let [site] = report.news.as_slice() else {
            panic!(
                "the presented body records its one construction site: {:?}",
                report.news
            );
        };
        assert!(site.presented(), "{site:?}");
        assert_eq!(
            site.arguments,
            [5, 13],
            "the record keeps physical arguments"
        );
        for bci in [4, 5, 6, 9, 13, 16] {
            assert!(
                !report.source_map.of_bci(bci).is_empty(),
                "BCI {bci} has no source-map span"
            );
        }
    }

    /// One assembled `Test.method` body, run through the same analysis and recovery a real class
    /// gets, so the floating-constant shapes are judged against a real run's tables and not
    /// against a hand-built fixture of this test's own choosing.
    fn recovered_synthetic(
        class_bytes: &[u8],
        descriptor: &[u8],
        parameters: u16,
    ) -> crate::report::RecoveryReport {
        use jarde_jvm::engine::analyze_method_ir;
        use jarde_jvm::environment::ResolutionEnvironment;
        use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
        use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
        use jarde_reader::model::{
            ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
            PhysicalMethodId, PhysicalVariant,
        };
        use jarde_reader::view::{
            DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
            MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
            RuntimeView,
        };

        let mut budget = Budget::new(jarde_reader::budget::Limits {
            input_bytes: 1 << 20,
            archive_entries: 100,
            entry_bytes: 1 << 20,
            read_bytes: 1 << 20,
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            output_bytes: 1 << 20,
            class_headers: 100,
            method_bodies: 100,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            nested_depth: 32,
            dependency_depth: 32,
            elapsed_millis: u64::MAX,
        });
        let snapshot =
            ArtifactSnapshot::open(ArtifactInput::bytes(class_bytes.to_vec()), &mut budget)
                .expect("the assembled class opens");
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class_bytes).to_hex().to_string()),
                length: u64::try_from(class_bytes.len()).expect("fixture length fits u64"),
            },
            variant: PhysicalVariant::Base,
        };
        let domain = LoadDomain {
            loader: LoaderId("app".to_string()),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots: vec![LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        };
        let environment = ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        };
        let analysis = analyze_method_ir(
            &[snapshot],
            &MethodAnalysisRequest {
                environment,
                method: PhysicalMethodId {
                    owner: definition,
                    name: JvmBytes(b"method".to_vec()),
                    descriptor: JvmBytes(descriptor.to_vec()),
                },
                stages: AnalysisStage::ALL.to_vec(),
            },
            &mut budget,
        )
        .expect("the assembled method completes JVM IR analysis");
        let method = crate::facts::MethodFacts::new(
            "method",
            String::from_utf8_lossy(descriptor).into_owned(),
            parameters,
        )
        .with_access_flags(0x0009)
        .with_declaring_class(crate::facts::DeclaringClass::new("Test", 0x0021));
        let facts = crate::facts::RecoveryFacts::new(method);
        let request =
            crate::report::RecoveryRequest::new(analysis.ir(), &facts, crate::pass::JAVA_8)
                .with_evidence(crate::evidence::RecoveryEvidenceRequest::all());
        crate::report::recover(&request, &mut budget)
    }

    /// A real `fneg` of the canonical NaN must not become `-(0.0f / 0.0f)`: javac folds that back
    /// to the *positive* NaN, while the JVM's `fneg` flipped the sign. The operand therefore goes
    /// through the accepted named-value save, and the negation stays a runtime operation on a
    /// non-final local.
    #[test]
    fn a_closed_nan_negation_saves_its_operand_so_the_real_operation_stays_runtime() {
        // ldc 20 (float 0x7fc00000); fneg; freturn
        let class_bytes = jarde_reader::classfile::test_class::single_method_floating(
            52,
            1,
            0,
            4,
            &0x7fc0_0000u32.to_be_bytes(),
            b"()F",
            &[0x12, 20, 0x76, 0xae],
        );
        let report = recovered_synthetic(&class_bytes, b"()F", 0);
        assert!(
            report.produced(),
            "stop={:?} text={}",
            report.stop(),
            report.text
        );
        assert_eq!(report.representation, jarde_jvm::ir::Representation::Java);
        assert!(
            report
                .text
                .contains("float saved0 = 0x0.000000p-126f / 0x0.000000p-126f;"),
            "the admitted NaN is the saved operand:\n{}",
            report.text
        );
        assert!(
            report.text.contains("return -saved0;"),
            "the negation reads the non-final local, so no compiler fold can touch it:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("return -0x"),
            "the negation must not be spelled over the constant tree:\n{}",
            report.text
        );

        // The same shape on a double: ldc2_w 20 (double 0x7ff8000000000000); dneg; dreturn.
        let class_bytes = jarde_reader::classfile::test_class::single_method_floating(
            52,
            1,
            0,
            6,
            &0x7ff8_0000_0000_0000u64.to_be_bytes(),
            b"()D",
            &[0x14, 0, 20, 0x77, 0xaf],
        );
        let report = recovered_synthetic(&class_bytes, b"()D", 0);
        assert!(
            report.produced(),
            "stop={:?} text={}",
            report.stop(),
            report.text
        );
        assert!(
            report
                .text
                .contains("double saved0 = 0x0.0000000000000p-1022d / 0x0.0000000000000p-1022d;"),
            "the admitted double NaN is the saved operand:\n{}",
            report.text
        );
        assert!(
            report.text.contains("return -saved0;"),
            "the double negation stays a runtime operation:\n{}",
            report.text
        );
    }

    /// The same closed shape over finite values gains no local: a finite literal round-trips its
    /// hex spelling exactly and the fold computes the bits the JVM computed, and a tree a
    /// parameter participates in is not a compile-time constant at all.
    #[test]
    fn finite_and_open_negations_render_inline_without_a_saved_local() {
        // fconst_1; fneg; freturn
        let finite = jarde_reader::classfile::test_class::single_method_floating(
            52,
            1,
            0,
            4,
            &0u32.to_be_bytes(),
            b"()F",
            &[0x0c, 0x76, 0xae],
        );
        let report = recovered_synthetic(&finite, b"()F", 0);
        assert!(
            report.produced(),
            "stop={:?} text={}",
            report.stop(),
            report.text
        );
        assert!(
            report.text.contains("return -0x1.000000p0f;"),
            "a finite negation is its own text:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("saved"),
            "finite constants never gain a saved local:\n{}",
            report.text
        );

        // fload_0; fconst_0; fadd; fneg; freturn — the parameter keeps the tree open.
        let open = jarde_reader::classfile::test_class::single_method_floating(
            52,
            2,
            1,
            4,
            &0u32.to_be_bytes(),
            b"(F)F",
            &[0x22, 0x0b, 0x62, 0x76, 0xae],
        );
        let report = recovered_synthetic(&open, b"(F)F", 1);
        assert!(
            report.produced(),
            "stop={:?} text={}",
            report.stop(),
            report.text
        );
        assert!(
            report.text.contains("return -(arg0 + 0x0.000000p-126f);"),
            "the open tree renders inline:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("saved"),
            "a non-constant tree never gains a saved local:\n{}",
            report.text
        );
    }

    /// A NaN bit pattern this change cannot present — here one payload bit — names no expression.
    /// The body stays a quote, the constant's own BCI is in it, and nothing is normalized to the
    /// default NaN.
    #[test]
    fn an_unpresentable_nan_bit_pattern_is_refused_at_its_own_bci() {
        // ldc 20 (float 0x7fc00001, one payload bit); freturn
        let class_bytes = jarde_reader::classfile::test_class::single_method_floating(
            52,
            1,
            0,
            4,
            &0x7fc0_0001u32.to_be_bytes(),
            b"()F",
            &[0x12, 20, 0xae],
        );
        let report = recovered_synthetic(&class_bytes, b"()F", 0);
        // The quote names the failed consumer (BCI 2, the `freturn` whose statement the quote
        // replaces) and the constant beside it (BCI 0).
        assert!(
            report.text.contains("@bytecode 2 0"),
            "the constant's own BCI stays visible beside its consumer:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("0x0.000000p-126f"),
            "nothing is normalized into a presentable pattern:\n{}",
            report.text
        );
        assert_ne!(
            report.representation,
            jarde_jvm::ir::Representation::Java,
            "the refusal is a stated boundary, not a recovery:\n{}",
            report.text
        );
    }

    #[test]
    fn member_constructor_boolean_adaptation_uses_the_tail_descriptor_position() {
        let source_arguments = typed_arguments_with_offset(
            "(Lnested/SimpleOuter;ZI)V",
            vec![
                Expr::direct(ExprKind::Integer(1), 12),
                Expr::direct(ExprKind::Integer(7), 13),
            ],
            1,
        );
        assert!(matches!(source_arguments[0].kind, ExprKind::Boolean(true)));
        assert!(matches!(source_arguments[1].kind, ExprKind::Integer(7)));

        let wrong_arity = typed_arguments_with_offset(
            "(Lnested/SimpleOuter;ZI)V",
            vec![Expr::direct(ExprKind::Integer(1), 12)],
            1,
        );
        assert!(matches!(wrong_arity[0].kind, ExprKind::Integer(1)));
    }

    #[test]
    fn quoted_call_loads_are_added_in_source_order_without_duplicates() {
        let mut quote = vec![4, 1];
        append_quoted_stable_loads(&mut quote, BTreeSet::from([0, 3, 4]), None)
            .expect("an unbudgeted quote extension cannot stop");
        assert_eq!(quote, [4, 1, 0, 3]);
    }

    #[test]
    fn array_reference_widening_uses_only_bounded_component_evidence() {
        for (presented, required) in [
            ("int[][]", "java.lang.Object[]"),
            ("java.lang.String[]", "java.lang.Object[]"),
            ("java.lang.String[][]", "java.lang.Object[][]"),
            ("java.lang.String[][]", "java.lang.Object[]"),
            ("int[]", "java.lang.Cloneable"),
            ("java.lang.String[]", "java.io.Serializable"),
            ("int[][]", "java.lang.Cloneable[]"),
            ("java.lang.String[][]", "java.io.Serializable[]"),
            ("int[][]", "int[][]"),
        ] {
            assert!(
                array_reference_widens(presented, required),
                "{presented} should widen to {required}"
            );
        }

        for (presented, required) in [
            ("int[]", "java.lang.Object[]"),
            ("int[][]", "java.lang.Object[][]"),
            ("java.lang.String[]", "java.lang.Object[][]"),
            ("int[]", "java.lang.Cloneable[]"),
            ("java.lang.String[]", "java.lang.Cloneable[]"),
            ("example.Child[]", "example.UserMarker"),
            ("example.Child[]", "example.Parent[]"),
            ("example.Implementation[]", "example.Interface[]"),
            ("example.Child", "example.Parent[]"),
            ("example.Child[]", "example.Parent"),
            ("void[]", "java.lang.Object[]"),
        ] {
            assert!(
                !array_reference_widens(presented, required),
                "{presented} must not widen to {required} from array shape alone"
            );
        }

        // JVMS descriptors cap array rank at 255. The walk accepts a valid boundary rank and
        // rejects an over-bound spelling before comparing any components.
        let rank_255 = format!("int{}", "[]".repeat(255));
        let rank_256 = format!("int{}", "[]".repeat(256));
        assert!(array_reference_widens(&rank_255, "java.lang.Object[]"));
        assert!(!array_reference_widens(&rank_256, "java.lang.Object[]"));
        assert!(!array_reference_widens(&rank_255, &rank_256));
    }

    fn functional_receiver(kind: ExprKind, presented: Option<Type>, bci: u32) -> Expr {
        let expression = Expr::direct(kind, bci);
        presented.map_or(expression.clone(), |ty| expression.presenting(ty))
    }

    fn interface_call(owner: &str) -> CallTarget {
        CallTarget::new(
            InvokeKind::Interface,
            owner,
            "apply",
            "(I)Ljava/lang/Object;",
            true,
        )
    }

    #[test]
    fn an_immediate_functional_receiver_is_typed_only_by_its_same_interface_owner() {
        let target_type = Type::Reference("java.util.function.IntUnaryOperator".to_string());
        let kinds = [
            ExprKind::Lambda {
                params: vec![],
                body: Box::new(Expr::direct(ExprKind::Integer(1), 10)),
            },
            ExprKind::MethodReference {
                qualifier: Box::new(Expr::direct(
                    ExprKind::Path("java.lang.Math".to_string()),
                    11,
                )),
                name: "abs".to_string(),
            },
        ];
        for kind in kinds {
            let receiver = functional_receiver(kind, Some(target_type.clone()), 7);
            let cast = immediate_functional_receiver(
                receiver,
                &interface_call("java/util/function/IntUnaryOperator"),
                15,
            )
            .expect("the factory and interface-call owner state the same spellable type");
            let ExprKind::Cast { ty, value } = cast.kind else {
                panic!("the direct poly expression receives the existing Cast node")
            };
            assert_eq!(ty, target_type);
            assert!(matches!(
                value.kind,
                ExprKind::Lambda { .. } | ExprKind::MethodReference { .. }
            ));
            assert_eq!(cast.origin.primary().bci(), 7);
            assert_eq!(cast.origin.derived().len(), 1);
            assert_eq!(cast.origin.derived()[0].bci(), 15);
            assert_eq!(
                cast.origin.derived()[0].provenance(),
                crate::source_map::Provenance::Derived
            );
        }
    }

    #[test]
    fn an_immediate_functional_receiver_refuses_missing_mismatched_and_unsupported_types() {
        let method_reference = || ExprKind::MethodReference {
            qualifier: Box::new(Expr::direct(
                ExprKind::Path("java.lang.Math".to_string()),
                4,
            )),
            name: "abs".to_string(),
        };
        let int_function = Type::Reference("java.util.function.IntFunction".to_string());

        let missing = functional_receiver(method_reference(), None, 4);
        assert!(
            immediate_functional_receiver(
                missing,
                &interface_call("java/util/function/IntFunction"),
                15
            )
            .expect_err("a missing factory target type refuses the receiver")
            .contains("no verified factory target type")
        );

        let mismatched = functional_receiver(method_reference(), Some(int_function), 4);
        assert!(
            immediate_functional_receiver(
                mismatched,
                &interface_call("java/util/function/IntUnaryOperator"),
                15,
            )
            .expect_err("a different interface owner is not guessed assignable")
            .contains("cannot prove they are the same interface")
        );

        let unspelled = functional_receiver(
            method_reference(),
            Some(Type::Reference("(I)V".to_string())),
            4,
        );
        assert!(
            immediate_functional_receiver(unspelled, &interface_call("(I)V"), 15)
                .expect_err("a method descriptor is not a Java type")
                .contains("cannot spell as a Java type")
        );

        let unsupported_invocation = functional_receiver(
            method_reference(),
            Some(Type::Reference(
                "java.util.function.IntUnaryOperator".to_string(),
            )),
            4,
        );
        let class_method = CallTarget::new(
            InvokeKind::Virtual,
            "java/util/function/IntUnaryOperator",
            "apply",
            "(I)Ljava/lang/Object;",
            false,
        );
        assert!(
            immediate_functional_receiver(unsupported_invocation, &class_method, 15)
                .expect_err("a non-interface owner shape is not treated as an SAM invocation")
                .contains("requires an interface-method reference")
        );
    }

    #[test]
    fn array_store_opcode_matches_the_exact_primitive_component() {
        for component in [Type::Boolean, Type::Byte] {
            assert!(array_store_opcode_matches(
                &component,
                Some(&Type::Int),
                0x54,
            ));
            assert!(!array_store_opcode_matches(
                &component,
                Some(&Type::Int),
                0x4f,
            ));
        }
        for (component, opcode) in [
            (Type::Char, 0x55),
            (Type::Short, 0x56),
            (Type::Int, 0x4f),
            (Type::Long, 0x50),
            (Type::Float, 0x51),
            (Type::Double, 0x52),
        ] {
            let opcode_element = match component {
                Type::Char | Type::Short | Type::Int => Type::Int,
                Type::Long => Type::Long,
                Type::Float => Type::Float,
                Type::Double => Type::Double,
                _ => unreachable!(),
            };
            assert!(array_store_opcode_matches(
                &component,
                Some(&opcode_element),
                opcode,
            ));
            assert!(!array_store_opcode_matches(
                &component,
                Some(&opcode_element),
                if opcode == 0x4f { 0x54 } else { 0x4f },
            ));
        }
        assert!(array_store_opcode_matches(
            &Type::Reference("java.lang.String".to_string()),
            None,
            0x53,
        ));
        assert!(!array_store_opcode_matches(
            &Type::Reference("java.lang.String".to_string()),
            None,
            0x4f,
        ));
    }

    #[test]
    fn a_descriptor_that_states_a_type_is_spelled_as_java_source() {
        // The object form, spelled exactly as it always was: the `L…;` wrapping comes off and the
        // separators become dots.
        assert_eq!(
            spell_reference("Ljava/lang/String;").as_deref(),
            Some("java.lang.String")
        );
        // A pool's own internal name (no wrapping to take off) keeps the same reading, which is what
        // keeps the class-name positions and the declarations spelling the same class the same way.
        assert_eq!(
            spell_reference("java/lang/String").as_deref(),
            Some("java.lang.String")
        );
        // An array descriptor is a type: the element is spelled first, with the object-name rule
        // above, and one `[]` follows per dimension.
        assert_eq!(spell_reference("[B").as_deref(), Some("byte[]"));
        assert_eq!(spell_reference("[Z").as_deref(), Some("boolean[]"));
        assert_eq!(
            spell_reference("[Ljava/lang/String;").as_deref(),
            Some("java.lang.String[]")
        );
        assert_eq!(spell_reference("[[I").as_deref(), Some("int[][]"));
        assert_eq!(
            spell_reference("[[Ljava/lang/String;").as_deref(),
            Some("java.lang.String[][]")
        );
    }

    #[test]
    fn a_name_that_states_no_java_type_is_refused_rather_than_written() {
        // The empty string and a method descriptor are not types.
        assert_eq!(spell_reference(""), None);
        assert_eq!(spell_reference("(I)V"), None);
        // A class type whose name is missing or carries the `;` that would end it earlier.
        assert_eq!(spell_reference("L;"), None);
        assert_eq!(spell_reference("Lfoo;bar;"), None);
        // A bare name is a name and not a descriptor: a class may be called `Lfoo`, so the forms
        // only a truncated descriptor could have are still read as the pool's own spelling.
        assert_eq!(spell_reference("Lfoo").as_deref(), Some("Lfoo"));
    }

    #[test]
    fn class_literal_qualifiers_are_refused_only_when_the_current_type_shadows_the_root() {
        assert!(!class_literal_path_is_shadowed(
            "java.lang.String",
            Some("String")
        ));
        assert!(class_literal_path_is_shadowed(
            "java.lang.String",
            Some("java")
        ));
        assert!(class_literal_path_is_shadowed(
            "java.lang.String[][]",
            Some("pkg/java")
        ));
        assert!(!class_literal_path_is_shadowed("java", Some("java")));
        assert!(!class_literal_path_is_shadowed("java[][]", Some("java")));
        assert!(!class_literal_path_is_shadowed(
            "ClassLiteralProbe",
            Some("ClassLiteralProbe")
        ));
    }

    #[test]
    fn a_position_converts_what_a_widening_conversion_states_and_nothing_else() {
        // The conversions a compiler writes with no instruction, and which the text therefore has to
        // state: the int-shaped family widened to `int`, and the widenings past it. Every one of them
        // is JLS 5.1.2, and every one of them is *value-faithful*.
        for (presented, required) in [
            (Type::Char, Type::Int),
            (Type::Char, Type::Long),
            (Type::Byte, Type::Int),
            (Type::Short, Type::Int),
            (Type::Byte, Type::Short),
            (Type::Int, Type::Long),
            (Type::Int, Type::Float),
            (Type::Int, Type::Double),
            (Type::Long, Type::Double),
            (Type::Float, Type::Double),
        ] {
            assert_eq!(
                conversion(&presented, &required),
                Conversion::Widening,
                "{presented:?} → {required:?} is a widening primitive conversion"
            );
        }
        // The same type needs nothing, and a reference beside anything is not this mechanism's
        // question: `+`, `append`, an invocation, a return and a write all convert a reference the
        // same way, and which class is assignable to which is a subtype judgment this layer does not
        // make.
        assert_eq!(conversion(&Type::Int, &Type::Int), Conversion::Same);
        assert_eq!(
            conversion(
                &Type::Reference("java.lang.String".to_string()),
                &Type::Reference("java.lang.Object".to_string())
            ),
            Conversion::Same
        );
        assert_eq!(
            conversion(&Type::Reference("java.lang.Object".to_string()), &Type::Int),
            Conversion::Same
        );
        // Nothing this layer's evidence states relates these: `boolean` and any other primitive have
        // no conversion at all (JLS 5.5), and a narrowing is one the layer cannot prove.
        for (presented, required) in [
            (Type::Boolean, Type::Int),
            (Type::Int, Type::Boolean),
            (Type::Boolean, Type::Boolean),
            (Type::Long, Type::Int),
            (Type::Double, Type::Float),
        ] {
            let expected = if presented == required {
                Conversion::Same
            } else {
                Conversion::Unspellable
            };
            assert_eq!(
                conversion(&presented, &required),
                expected,
                "{presented:?} → {required:?}"
            );
        }
    }

    #[test]
    fn explicit_conversion_source_categories_accept_int_family_but_reject_boolean() {
        for presented in [Type::Byte, Type::Char, Type::Short, Type::Int] {
            assert!(
                primitive_conversion_source_matches(&Type::Int, &presented),
                "the JVM int source category accepts `{}`",
                presented.spell()
            );
        }
        for (source, presented) in [
            (Type::Int, Type::Boolean),
            (Type::Long, Type::Int),
            (Type::Long, Type::Float),
            (Type::Float, Type::Long),
            (Type::Double, Type::Float),
            (Type::Float, Type::Boolean),
        ] {
            assert!(
                !primitive_conversion_source_matches(&source, &presented),
                "`{}` is not a valid presented source for the `{}` opcode category",
                presented.spell(),
                source.spell()
            );
        }
        for ty in [Type::Long, Type::Float, Type::Double] {
            assert!(primitive_conversion_source_matches(&ty, &ty));
        }
    }

    #[test]
    fn a_position_writes_an_implicit_widening_as_the_value_itself() {
        fn named(ty: Type) -> Expr {
            Expr::direct(ExprKind::Local("arg0".to_string()), 7).presenting(ty)
        }
        const POSITION: &str = "the `append` at BCI 8 takes `int`";

        // A widening primitive conversion is the **position's** own (P3 2c.29): no instruction runs,
        // and Java widens at the assignment and the `return`, so the text is the value's own. Calls
        // use the descriptor-aware invocation rule above, which may need an explicit cast to preserve
        // overload selection.
        for (presented, required) in [
            (Type::Char, Type::Int),
            (Type::Byte, Type::Short),
            (Type::Int, Type::Long),
            (Type::Float, Type::Double),
        ] {
            let value = named(presented.clone());
            let written = meeting_position(value.clone(), &required, POSITION, Widening::Position)
                .expect("a widening conversion is one the position itself performs");
            assert_eq!(written.kind, value.kind, "{presented:?} → {required:?}");
            assert_eq!(
                written.origin.primary().bci(),
                7,
                "{presented:?} → {required:?}"
            );
            assert_eq!(
                written.presented,
                Some(presented),
                "the value keeps its own type: the position widens it, not the text"
            );
        }

        // A value whose own type the layer states nothing about is written exactly as it was
        // rendered: no mismatch is provable, so nothing is converted.
        let unknown = Expr::direct(ExprKind::Null, 7);
        let written = meeting_position(unknown.clone(), &Type::Int, POSITION, Widening::Position)
            .expect("no evidence");
        assert_eq!(written.kind, unknown.kind);
        assert_eq!(written.presented, None);

        // An `int` constant is taken by the position's own rule (JLS 5.2/5.3): the decimal number in
        // a `byte` or a `short` position, and the character that constant stands for in a `char` one
        // (P3 2c.30) — the spelling a compiler gives the `bipush` of a `char` return.
        let fits = Expr::direct(ExprKind::Integer(65), 7);
        let narrowed = meeting_position(fits.clone(), &Type::Byte, POSITION, Widening::Position)
            .expect("`65` is a `byte` constant");
        assert_eq!(
            narrowed.kind, fits.kind,
            "a `byte` position writes the number"
        );
        let character = meeting_position(fits.clone(), &Type::Char, POSITION, Widening::Position)
            .expect("`65` is a `char` constant");
        assert_eq!(character.presented, Some(Type::Char));
        assert_eq!(character.origin.primary().bci(), 7);
        assert!(
            matches!(&character.kind, ExprKind::Local(text) if text == "'A'"),
            "{:?}",
            character.kind
        );
        // The escapes are the emitter's one table, the same one a `char` `switch` key is written
        // with: the delimiter, the backslash, the controls and the line terminators.
        for (constant, literal) in [
            (0x27, "'\\''"),
            (0x5c, "'\\\\'"),
            (0x0a, "'\\n'"),
            (0x0000, "'\\u0000'"),
            (0x2028, "'\\u2028'"),
            (0xffff, "'\\uffff'"),
        ] {
            let value = Expr::direct(ExprKind::Integer(constant), 7);
            let written = meeting_position(value, &Type::Char, POSITION, Widening::Position)
                .expect("an in-range `int` constant is a `char`");
            assert!(
                matches!(&written.kind, ExprKind::Local(text) if text == literal),
                "{constant}: {:?}",
                written.kind
            );
        }
        // An `int` position keeps the number: the wording of a `char` literal is the `char`
        // position's, and the constant is not narrowed where nothing requires a narrower type.
        let wide = meeting_position(
            Expr::direct(ExprKind::Integer(65), 7),
            &Type::Int,
            POSITION,
            Widening::Position,
        )
        .expect("`65` is an `int`");
        assert_eq!(wide.kind, ExprKind::Integer(65));
        // A constant the type cannot hold is the mismatch it is, and it is refused before any
        // literal is spelled.
        assert!(
            meeting_position(
                Expr::direct(ExprKind::Integer(300), 7),
                &Type::Byte,
                POSITION,
                Widening::Position
            )
            .is_err(),
            "`300` is no `byte`"
        );
        assert!(
            meeting_position(
                Expr::direct(ExprKind::Integer(-1), 7),
                &Type::Char,
                POSITION,
                Widening::Position
            )
            .is_err(),
            "`-1` is no `char`"
        );

        // The refusal names the value's BCI, the type it presents as and the position that requires
        // another one, so the report can be read without the text.
        let message = meeting_position(
            named(Type::Boolean),
            &Type::Int,
            POSITION,
            Widening::Position,
        )
        .expect_err("no conversion relates a `boolean` and an `int`");
        assert!(
            message.contains("BCI 7")
                && message.contains("`boolean`")
                && message.contains("`int`")
                && message.contains("the `append` at BCI 8"),
            "{message}"
        );
    }

    #[test]
    fn a_concatenation_part_states_the_widening_its_own_append_performed() {
        // The other side of [`Widening`] (T5): a part's text is a `+`, and `+` converts an operand by
        // the operand's own type — `"" + c` appends the *character* where the bytecode called
        // `append(I)` and appended the code unit. So this position is one the text itself performs,
        // and the conversion the bytecode performed is written into it.
        fn named(ty: Type) -> Expr {
            Expr::direct(ExprKind::Local("arg0".to_string()), 7).presenting(ty)
        }
        const POSITION: &str = "the `append` at BCI 8 takes `int`";

        let part = meeting_position(named(Type::Char), &Type::Int, POSITION, Widening::Text)
            .expect("`char` meets an `int` part by a widening conversion");
        assert!(
            matches!(part.kind, ExprKind::Cast { ref ty, .. } if *ty == Type::Int),
            "{:?}",
            part.kind
        );
        assert_eq!(part.presented, Some(Type::Int));
        assert_eq!(part.origin.primary().bci(), 7, "the value's own anchor");

        // A part the position has nothing to convert gains nothing, whichever rule the position
        // follows: the conversion is written where one happened and nowhere else.
        let kept = meeting_position(named(Type::Int), &Type::Int, POSITION, Widening::Text)
            .expect("`int` meets `int`");
        assert!(matches!(kept.kind, ExprKind::Local(_)), "{:?}", kept.kind);
    }

    const TERNARY_CORE_FIXTURE: &[u8] =
        include_bytes!("../../../tests/fixtures/p3-conditional-values/v8/TernaryCore.class");
    const TERNARY_VALUES_FIXTURE: &[u8] =
        include_bytes!("../../../tests/fixtures/p3-conditional-values/v8/TernaryValues.class");
    const CONDITIONAL_SWITCH_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/v8/ConditionalBoundarySwitch.class"
    );
    const SHORT_CIRCUIT_BASE_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/proved-java-structure/anonymous-super-dispatch/Base.class"
    );
    const SHORT_CIRCUIT_DOUBLE_WRITE_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/short-circuit-double-write/Base.class"
    );
    const SHORT_CIRCUIT_SHARED_TRUE_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/short-circuit-shared-true/SharedTrueShortCircuit.class"
    );
    const SHORT_CIRCUIT_CHAIN_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/short-circuit-chain-shared-true/ChainOrField.class"
    );
    const MIXED_SHORT_CIRCUIT_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-field/MixedBooleanField.class"
    );
    const MIXED_SHORT_CIRCUIT_RETURN_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-return/MixedLocalReturn.class"
    );
    const MIXED_SHORT_CIRCUIT_INT_RETURN_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-int-return/MixedIntReturn.class"
    );
    const MIXED_SHORT_CIRCUIT_ARGUMENT_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-argument/MixedBooleanArgument.class"
    );
    const MIXED_SHORT_CIRCUIT_LOCAL_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-local/MixedBooleanLocal.class"
    );
    const MIXED_SHORT_CIRCUIT_INSTANCE_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-instance-field/MixedShortCircuitField.class"
    );
    const MIXED_SHORT_CIRCUIT_INSTANCE_CONTROLS_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-instance-field/MixedInstanceControls.class"
    );
    const MIXED_SHORT_CIRCUIT_ARGUMENT_CONTROLS_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-argument-controls/MixedArgumentControls.class"
    );
    const SHORT_CIRCUIT_SHARED_TRUE_DUPLICATE_PHI_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/short-circuit-shared-true-controls/SharedTrueDuplicatePhi.class"
    );
    const SHORT_CIRCUIT_SHARED_TRUE_NON_BOOLEAN_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/short-circuit-shared-true-controls/SharedTrueNonBoolean.class"
    );
    const SHORT_CIRCUIT_LOCAL_FIXTURE: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-values/short-circuit-local/LocalShortCircuit.class"
    );
    const LOOP_TRANSFERS_FIXTURE: &[u8] =
        include_bytes!("../../../tests/fixtures/p3-loop-transfers/v8/OuterContinue.class");
    /// Runs the real JVM IR and region producers, then asks only this proof about their output.
    fn fixture_conditional_attempt(
        class: &[u8],
        name: &str,
        descriptor: &str,
    ) -> (
        Region,
        ConditionalValueAttempt,
        Option<(u32, u32)>,
        crate::region::Recovered,
        crate::report::RecoveryReport,
    ) {
        let (region, ordinary, producer_bcis, recovered, report, _) =
            fixture_value_attempts(class, name, descriptor, None);
        (region, ordinary, producer_bcis, recovered, report)
    }

    fn fixture_value_attempts(
        class: &[u8],
        name: &str,
        descriptor: &str,
        short_region: Option<&Region>,
    ) -> (
        Region,
        ConditionalValueAttempt,
        Option<(u32, u32)>,
        crate::region::Recovered,
        crate::report::RecoveryReport,
        ShortCircuitValueAttempt,
    ) {
        use jarde_jvm::engine::analyze_method_ir;
        use jarde_jvm::environment::ResolutionEnvironment;
        use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
        use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
        use jarde_reader::model::{
            ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
            PhysicalMethodId, PhysicalVariant,
        };
        use jarde_reader::view::{
            DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
            MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
            RuntimeView,
        };

        let mut budget = Budget::new(jarde_reader::budget::Limits {
            input_bytes: 1 << 20,
            archive_entries: 100,
            entry_bytes: 1 << 20,
            read_bytes: 1 << 20,
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            output_bytes: 1 << 20,
            class_headers: 100,
            method_bodies: 100,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            nested_depth: 32,
            dependency_depth: 32,
            elapsed_millis: u64::MAX,
        });
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
            .expect("the permanent fixture opens as a class");
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class).to_hex().to_string()),
                length: u64::try_from(class.len()).expect("fixture length fits u64"),
            },
            variant: PhysicalVariant::Base,
        };
        let method = PhysicalMethodId {
            owner: definition,
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        };
        let domain = LoadDomain {
            loader: LoaderId("app".to_string()),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots: vec![LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        };
        let environment = ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        };
        let request = MethodAnalysisRequest {
            environment,
            method,
            stages: AnalysisStage::ALL.to_vec(),
        };
        let analysis = analyze_method_ir(&[snapshot], &request, &mut budget)
            .expect("the fixture method completes its IR analysis");
        let ir = analysis.ir();
        let canonical = ir.canonical().expect("the method publishes its graph");
        let ssa = ir.ssa().expect("the method publishes SSA");
        let code = ir.code().expect("the method publishes its decode");
        let operations = Operations::of(code, ir.constant_pool());
        let view = crate::normal_flow::NormalFlowView::build(canonical, &mut budget)
            .expect("the bounded normal-flow view builds");
        let recovered = crate::region::recover(
            canonical,
            &view,
            ssa,
            &operations,
            code,
            Some(false),
            return_type(descriptor).is_some_and(|ty| matches!(ty, crate::ast::Type::Boolean)),
            &crate::pass::JAVA_8,
            &mut budget,
        )
        .expect("the fixture region tree is bounded");
        let candidate = recovered
            .regions
            .iter()
            .find(|region| matches!(region, Region::If { .. }))
            .or_else(|| recovered.regions.first())
            .expect("the method has a region");
        let candidate = candidate.clone();
        let attempt = prove_conditional_value(&candidate, canonical, ssa, &operations, &mut budget)
            .expect("the bounded conditional proof completes");
        let short_attempt = prove_short_circuit_value(
            short_region.unwrap_or(&candidate),
            canonical,
            ssa,
            &operations,
            return_type(descriptor).as_ref(),
            &mut budget,
        )
        .expect("the bounded short-circuit proof completes");
        let producer_bcis = match &attempt {
            ConditionalValueAttempt::Proved(proof) => {
                let producer_bci = |value| match ssa.value(value).def() {
                    Definition::Instruction { bci, .. } => Some(*bci),
                    _ => None,
                };
                producer_bci(proof.when_true).zip(producer_bci(proof.when_false))
            }
            ConditionalValueAttempt::Refused(_) => None,
        };
        let mut method_facts = crate::facts::MethodFacts::new(
            name,
            descriptor,
            if name == "<init>" && descriptor == "(Ljava/lang/String;I)V" {
                3
            } else if descriptor == "(ZZ)V" {
                2
            } else {
                u16::from(!descriptor.starts_with("()"))
            },
        );
        if name == "<init>" && descriptor == "(Ljava/lang/String;I)V" {
            method_facts = method_facts.with_access_flags(0x0001).with_declaring_class(
                crate::facts::DeclaringClass::new("ConstructorPairProbe", 0x0031),
            );
        }
        let facts = crate::facts::RecoveryFacts::new(method_facts);
        let request = crate::report::RecoveryRequest::new(ir, &facts, crate::pass::JAVA_8)
            .with_evidence(crate::evidence::RecoveryEvidenceRequest::all());
        let report = crate::report::recover(&request, &mut budget);
        (
            candidate,
            attempt,
            producer_bcis,
            recovered,
            report,
            short_attempt,
        )
    }

    #[test]
    fn a_conditional_proof_binds_each_straight_arm_to_one_stack_phi_and_consumer() {
        let (region, attempt, producer_bcis, _, _) =
            fixture_conditional_attempt(TERNARY_CORE_FIXTURE, "returned", "(Z)I");
        assert!(matches!(region, Region::If { .. }));
        let ConditionalValueAttempt::Proved(proof) = attempt else {
            panic!("the minimal legal conditional has a two-arm stack Phi: {attempt:?}");
        };
        assert_eq!(proof.branch_bci, 1);
        assert_eq!(proof.join.bci(), 13);
        assert_eq!(proof.stack_depth, 0);
        assert_eq!(proof.consumer_bci, 13);
        assert_ne!(proof.when_true, proof.when_false);
        assert_eq!(
            producer_bcis,
            Some((4, 10)),
            "true selects the fall-through a() arm"
        );

        for (name, descriptor) in [
            ("returned", "(Z)I"),
            ("assigned", "(Z)I"),
            ("arithmetic", "(Z)I"),
            ("callArgument", "(Z)I"),
            ("reference", "(Z)Ljava/lang/String;"),
            ("overloadChoice", "(Z)Ljava/lang/String;"),
            ("throwing", "(Z)I"),
        ] {
            let (region, attempt, _, _, _) =
                fixture_conditional_attempt(TERNARY_VALUES_FIXTURE, name, descriptor);
            assert!(matches!(region, Region::If { .. }), "{name}: {region:?}");
            assert!(
                matches!(attempt, ConditionalValueAttempt::Proved(_)),
                "{name}: {attempt:?}"
            );
        }
    }

    #[test]
    fn a_switch_join_is_not_admitted_as_an_if_conditional_value() {
        let (region, attempt, _, _, _) =
            fixture_conditional_attempt(CONDITIONAL_SWITCH_FIXTURE, "choose", "(I)I");
        assert!(matches!(region, Region::Switch { .. }));
        assert_eq!(
            attempt,
            ConditionalValueAttempt::Refused(ConditionalValueRefusal::NotIf)
        );
    }

    #[test]
    fn shared_false_short_circuit_has_one_region_owner_and_one_field_write() {
        let (region, attempt, _, recovered, report) =
            fixture_conditional_attempt(SHORT_CIRCUIT_BASE_FIXTURE, "<init>", "()V");
        let Region::ShortCircuitValue {
            tests,
            consumer_bci,
            ..
        } = region
        else {
            panic!("the bounded shared-false shape gets its own Region: {region:?}");
        };
        assert_eq!(
            (
                tests.iter().map(|(_, bci)| *bci).collect::<Vec<_>>(),
                consumer_bci
            ),
            (vec![15, 26], 34)
        );
        assert_eq!(
            attempt,
            ConditionalValueAttempt::Refused(ConditionalValueRefusal::NotIf),
            "Region ownership does not claim an SSA value proof"
        );
        let owned: Vec<_> = recovered
            .regions
            .iter()
            .flat_map(Region::blocks)
            .map(|block| block.clone())
            .collect();
        let unique: BTreeSet<_> = owned.iter().cloned().collect();
        assert_eq!(
            owned.len(),
            unique.len(),
            "each physical block has one owner"
        );
        assert_eq!(
            owned.iter().map(CanonicalBlockId::bci).collect::<Vec<_>>(),
            [0, 18, 29, 33, 34]
        );
        assert!(recovered.is_structured());
        assert!(recovered.fallbacks().is_empty());
        assert!(recovered.regions.iter().all(|region| !matches!(
            region,
            Region::Fallback {
                reason: crate::region::FallbackReason::Loop { block_bci: 34 },
                ..
            }
        )));
        assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
        assert_eq!(
            report
                .text
                .matches("capturedVisibleBeforeBaseReturns =")
                .count(),
            1,
            "the shared value writes its field once:\n{}",
            report.text
        );
        assert!(report.regions.iter().any(|region| {
            region.blocks == [0, 18, 29, 33, 34] && region.structured && region.code.is_none()
        }));
        assert!(report.source_map.segments().iter().any(|segment| {
            [15, 26, 29, 30, 33, 34]
                .iter()
                .all(|bci| segment.origin().bcis().contains(bci))
                && segment
                    .text(&report.text)
                    .contains("capturedVisibleBeforeBaseReturns")
        }));
        for bci in [37, 38, 41] {
            assert!(
                report
                    .source_map
                    .segments()
                    .iter()
                    .any(|segment| { segment.origin().bcis().contains(&bci) })
            );
        }
    }

    #[test]
    fn mixed_short_circuit_graphs_have_one_owner_and_one_proved_field_write() {
        for name in ["andOr", "orAnd"] {
            let (region, _, _, recovered, report, attempt) =
                fixture_value_attempts(MIXED_SHORT_CIRCUIT_FIXTURE, name, "(Z)V", None);
            let Region::ShortCircuitValue {
                tests,
                test_edges,
                consumer_bci,
                ..
            } = &region
            else {
                panic!("{name} must have one graph owner: {region:?}");
            };
            assert_eq!(
                tests.iter().map(|(_, bci)| *bci).collect::<Vec<_>>(),
                [1, 7, 13]
            );
            assert_eq!(test_edges.len(), 3);
            assert_eq!(*consumer_bci, 21);
            let owned = recovered
                .regions
                .iter()
                .flat_map(Region::blocks)
                .collect::<Vec<_>>();
            assert_eq!(
                owned.len(),
                owned.iter().copied().collect::<BTreeSet<_>>().len()
            );
            let ShortCircuitValueAttempt::Proved(proof) = attempt else {
                panic!("{name} graph proof failed: {attempt:?}\n{region:?}");
            };
            assert_eq!(proof.test_bcis, [1, 7, 13]);
            assert_eq!(
                proof.consumer,
                ShortCircuitConsumer::Field(
                    "MixedBooleanField".into(),
                    "result".into(),
                    "Z".into()
                )
            );
            assert_eq!(
                report.quality,
                jarde_jvm::ir::Quality::Structured,
                "{name}: {}",
                report.text
            );
            assert_eq!(report.text.matches("MixedBooleanField.result =").count(), 1);
            assert!(
                !report.text.contains("@bytecode"),
                "{name}: {}",
                report.text
            );
            let mapped = report
                .source_map
                .segments()
                .iter()
                .flat_map(|segment| segment.origin().bcis().clone())
                .collect::<BTreeSet<_>>();
            for bci in [0, 1, 4, 7, 10, 13, 16, 17, 20, 21, 24] {
                assert!(
                    mapped.contains(&bci),
                    "{name}: missing BCI {bci}: {}",
                    report.text
                );
            }
        }
    }

    #[test]
    fn mixed_short_circuit_return_reuses_the_graph_and_proves_boolean_consumer() {
        let (region, _, _, recovered, report, attempt) =
            fixture_value_attempts(MIXED_SHORT_CIRCUIT_RETURN_FIXTURE, "value", "(Z)Z", None);
        let Region::ShortCircuitValue {
            tests,
            consumer_bci,
            ..
        } = &region
        else {
            panic!("the return shares the bounded graph owner: {region:?}");
        };
        assert_eq!(
            tests.iter().map(|(_, bci)| *bci).collect::<Vec<_>>(),
            [1, 7, 13]
        );
        assert_eq!(*consumer_bci, 21);
        let owned = recovered
            .regions
            .iter()
            .flat_map(Region::blocks)
            .collect::<Vec<_>>();
        assert_eq!(
            owned.len(),
            owned.iter().copied().collect::<BTreeSet<_>>().len()
        );
        let ShortCircuitValueAttempt::Proved(proof) = attempt else {
            panic!("the return consumer must prove: {attempt:?}");
        };
        assert_eq!(proof.consumer, ShortCircuitConsumer::Return);
        assert_eq!(
            report.quality,
            jarde_jvm::ir::Quality::Structured,
            "{}",
            report.text
        );
        assert_eq!(report.text.matches("return ").count(), 1, "{}", report.text);
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        for bci in [0, 1, 4, 7, 10, 13, 16, 17, 20, 21] {
            assert!(
                !report.source_map.of_bci(bci).is_empty(),
                "unmapped {bci}: {}",
                report.text
            );
        }
    }

    #[test]
    fn the_same_ireturn_in_an_int_method_cannot_claim_boolean_value() {
        let (region, _, _, _, report, attempt) = fixture_value_attempts(
            MIXED_SHORT_CIRCUIT_INT_RETURN_FIXTURE,
            "value",
            "(Z)I",
            None,
        );
        assert!(
            matches!(region, Region::ShortCircuitValue { .. }),
            "{region:?}"
        );
        assert_eq!(
            attempt,
            ShortCircuitValueAttempt::Refused(ShortCircuitValueRefusal::Consumer)
        );
        assert_eq!(
            report.quality,
            jarde_jvm::ir::Quality::Fallback,
            "{}",
            report.text
        );
        assert!(!report.text.contains("return "), "{}", report.text);
        for bci in [0, 1, 4, 7, 10, 13, 16, 17, 20, 21] {
            assert!(
                !report.source_map.of_bci(bci).is_empty(),
                "unmapped {bci}: {}",
                report.text
            );
        }
    }

    #[test]
    fn mixed_short_circuit_argument_proves_one_static_boolean_sink() {
        let (region, _, _, recovered, report, attempt) =
            fixture_value_attempts(MIXED_SHORT_CIRCUIT_ARGUMENT_FIXTURE, "call", "(Z)V", None);
        let Region::ShortCircuitValue {
            tests,
            consumer_bci,
            ..
        } = &region
        else {
            panic!("the call shares the bounded graph owner: {region:?}");
        };
        assert_eq!(
            tests.iter().map(|(_, bci)| *bci).collect::<Vec<_>>(),
            [1, 7, 13]
        );
        assert_eq!(*consumer_bci, 21);
        let owned = recovered
            .regions
            .iter()
            .flat_map(Region::blocks)
            .collect::<Vec<_>>();
        assert_eq!(
            owned.len(),
            owned.iter().copied().collect::<BTreeSet<_>>().len()
        );
        let ShortCircuitValueAttempt::Proved(proof) = attempt else {
            panic!("the invocation consumer must prove: {attempt:?}");
        };
        let ShortCircuitConsumer::Invoke(target) = proof.consumer else {
            panic!("the proof must retain the decoded Methodref");
        };
        assert_eq!(
            (
                target.kind(),
                target.owner(),
                target.name(),
                target.descriptor()
            ),
            (InvokeKind::Static, "MixedBooleanArgument", "sink", "(Z)V")
        );
        assert_eq!(
            report.quality,
            jarde_jvm::ir::Quality::Structured,
            "{}",
            report.text
        );
        assert_eq!(report.text.matches("sink(").count(), 1, "{}", report.text);
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        for bci in [0, 1, 4, 7, 10, 13, 16, 17, 20, 21, 24] {
            assert!(
                !report.source_map.of_bci(bci).is_empty(),
                "unmapped {bci}: {}",
                report.text
            );
        }
    }

    #[test]
    fn mixed_short_circuit_local_has_one_direct_phi_store_and_two_name_reads() {
        let (region, _, _, _, report, attempt) =
            fixture_value_attempts(MIXED_SHORT_CIRCUIT_LOCAL_FIXTURE, "one", "(Z)Z", None);
        assert!(matches!(
            region,
            Region::ShortCircuitValue {
                consumer_bci: 21,
                ..
            }
        ));
        let ShortCircuitValueAttempt::Proved(proof) = attempt else {
            panic!("the local store must be the Phi's sole direct use: {attempt:?}");
        };
        assert!(matches!(proof.consumer, ShortCircuitConsumer::Local(1, _)));
        assert_eq!(
            report.quality,
            jarde_jvm::ir::Quality::Structured,
            "{}",
            report.text
        );
        assert_eq!(report.text.matches("boolean local1 =").count(), 1);
        assert!(report.text.contains("MixedBooleanLocal.result = local1;"));
        assert!(report.text.contains("return local1;"));
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        for bci in [0, 1, 4, 7, 10, 13, 16, 17, 20, 21, 22, 23, 26, 27] {
            assert!(
                !report.source_map.of_bci(bci).is_empty(),
                "unmapped {bci}: {}",
                report.text
            );
        }
    }

    #[test]
    fn instance_field_short_circuit_has_one_physical_owner_and_two_exact_operands() {
        let (region, _, _, recovered, report, attempt) =
            fixture_value_attempts(MIXED_SHORT_CIRCUIT_INSTANCE_FIXTURE, "one", "(ZZ)V", None);
        let Region::ShortCircuitValue {
            tests,
            consumer_bci,
            ..
        } = &region
        else {
            panic!("the instance field graph needs one Region owner: {region:?}");
        };
        assert_eq!(
            tests.iter().map(|(_, bci)| *bci).collect::<Vec<_>>(),
            [5, 11, 17]
        );
        assert_eq!(*consumer_bci, 25);
        let owned = recovered
            .regions
            .iter()
            .flat_map(Region::blocks)
            .collect::<Vec<_>>();
        assert_eq!(
            owned.len(),
            owned.iter().copied().collect::<BTreeSet<_>>().len()
        );
        let ShortCircuitValueAttempt::Proved(proof) = attempt else {
            panic!("the instance field operands must prove: {attempt:?}");
        };
        assert_eq!(proof.stack_depth, 1);
        assert!(matches!(
            proof.consumer,
            ShortCircuitConsumer::InstanceField(ref owner, ref name, ref descriptor, _)
                if owner == "MixedShortCircuitField$Box" && name == "result" && descriptor == "Z"
        ));
        assert_eq!(
            report.quality,
            jarde_jvm::ir::Quality::Structured,
            "{}",
            report.text
        );
        assert_eq!(report.text.matches("target(arg1).result =").count(), 1);
    }

    #[test]
    fn instance_field_near_misses_keep_one_complete_quote() {
        for (name, descriptor) in [
            ("numeric", "(ZZ)V"),
            ("duplicated", "(ZZ)Z"),
            ("compound", "(ZZ)V"),
        ] {
            let (region, _, _, _, report, attempt) = fixture_value_attempts(
                MIXED_SHORT_CIRCUIT_INSTANCE_CONTROLS_FIXTURE,
                name,
                descriptor,
                None,
            );
            assert!(
                matches!(region, Region::ShortCircuitValue { .. }),
                "{name}: {region:?}"
            );
            let expected = if name == "numeric" {
                ShortCircuitValueRefusal::Field
            } else {
                ShortCircuitValueRefusal::Consumer
            };
            assert_eq!(
                attempt,
                ShortCircuitValueAttempt::Refused(expected),
                "{name}"
            );
            assert_eq!(report.quality, jarde_jvm::ir::Quality::Fallback, "{name}");
            assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
        }
    }

    #[test]
    fn instance_receiver_with_an_earlier_static_owner_cannot_be_emitted_twice() {
        let (region, _, _, _, report, attempt) = fixture_value_attempts(
            MIXED_SHORT_CIRCUIT_INSTANCE_CONTROLS_FIXTURE,
            "sharedReceiver",
            "(ZZ)V",
            None,
        );
        assert!(matches!(region, Region::ShortCircuitValue { .. }));
        assert!(
            matches!(attempt, ShortCircuitValueAttempt::Proved(_)),
            "{attempt:?}"
        );
        assert_eq!(
            report.quality,
            jarde_jvm::ir::Quality::Fallback,
            "{}",
            report.text
        );
        assert!(report.text.contains("@bytecode"), "{}", report.text);
    }

    #[test]
    fn inherited_field_owner_cannot_claim_a_derived_receiver_type() {
        let (region, _, _, _, report, attempt) = fixture_value_attempts(
            MIXED_SHORT_CIRCUIT_INSTANCE_CONTROLS_FIXTURE,
            "inheritedOwner",
            "(ZZ)V",
            None,
        );
        assert!(matches!(region, Region::ShortCircuitValue { .. }));
        let ShortCircuitValueAttempt::Proved(proof) = attempt else {
            panic!("the graph and Phi should prove before type presentation: {attempt:?}");
        };
        assert!(matches!(
            proof.consumer,
            ShortCircuitConsumer::InstanceField(ref owner, ref name, ref descriptor, _)
                if owner == "MixedInstanceControls$BaseBox" && name == "result" && descriptor == "Z"
        ));
        assert_eq!(report.quality, jarde_jvm::ir::Quality::Fallback);
        let field = report
            .fields
            .iter()
            .find(|field| field.bci == 25)
            .expect("the actual putfield has a field-plan verdict");
        let refusal = field.refusal.as_ref().expect("the owner mismatch refuses");
        assert!(!field.presented);
        assert_eq!(field.owner, "MixedInstanceControls$BaseBox");
        assert!(refusal.message.contains("MixedInstanceControls$DerivedBox"));
        assert!(refusal.message.contains("MixedInstanceControls$BaseBox"));
        assert!(report.text.contains("@bytecode"), "{}", report.text);
        assert!(!report.text.contains(".result ="), "{}", report.text);
    }

    #[test]
    fn extra_argument_and_instance_receiver_refuse_the_whole_graph() {
        for (name, bcis) in [
            ("many", &[0, 1, 4, 7, 10, 13, 16, 17, 20, 21, 23, 26][..]),
            (
                "instance",
                &[0, 3, 4, 7, 10, 13, 16, 19, 20, 23, 24, 27][..],
            ),
        ] {
            let (region, _, _, _, report, attempt) = fixture_value_attempts(
                MIXED_SHORT_CIRCUIT_ARGUMENT_CONTROLS_FIXTURE,
                name,
                "(Z)V",
                None,
            );
            assert!(
                matches!(region, Region::ShortCircuitValue { .. }),
                "{name}: {region:?}"
            );
            assert!(
                matches!(attempt, ShortCircuitValueAttempt::Refused(_)),
                "{name}: {attempt:?}"
            );
            assert_eq!(
                report.quality,
                jarde_jvm::ir::Quality::Fallback,
                "{name}: {}",
                report.text
            );
            assert!(
                !report.text.contains("two(") && !report.text.contains("take("),
                "{name}: {}",
                report.text
            );
            for bci in bcis {
                assert!(
                    !report.source_map.of_bci(*bci).is_empty(),
                    "{name}: unmapped {bci}: {}",
                    report.text
                );
            }
        }
    }

    #[test]
    fn three_test_shared_true_chain_has_one_owner_proof_and_delayed_write() {
        let (region, _, _, recovered, report, attempt) =
            fixture_value_attempts(SHORT_CIRCUIT_CHAIN_FIXTURE, "assign", "(ZZ)V", None);
        let Region::ShortCircuitValue {
            tests,
            consumer_bci,
            ..
        } = &region
        else {
            panic!("the entire chain must have one owner: {region:?}");
        };
        assert_eq!(
            tests.iter().map(|(_, bci)| *bci).collect::<Vec<_>>(),
            [1, 5, 11]
        );
        assert_eq!(*consumer_bci, 19);
        let owned = recovered
            .regions
            .iter()
            .flat_map(Region::blocks)
            .collect::<Vec<_>>();
        assert_eq!(
            owned.len(),
            owned.iter().copied().collect::<BTreeSet<_>>().len()
        );
        assert_eq!(
            owned.iter().map(|block| block.bci()).collect::<Vec<_>>(),
            [0, 4, 8, 14, 18, 19]
        );
        let ShortCircuitValueAttempt::Proved(proof) = attempt else {
            panic!("all three tests and the one Phi must prove: {attempt:?}");
        };
        assert_eq!(proof.test_bcis, [1, 5, 11]);
        assert_eq!(proof.consumer_bci, 19);
        assert_eq!(
            proof.consumer,
            ShortCircuitConsumer::Field("ChainOrField".into(), "result".into(), "Z".into())
        );
        assert_eq!(
            report.quality,
            jarde_jvm::ir::Quality::Structured,
            "{}\n{:?}",
            report.text,
            report.regions
        );
        assert_eq!(
            report.text.matches("ChainOrField.result =").count(),
            1,
            "{}",
            report.text
        );
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        assert!(report.text.contains("rhs()"), "{}", report.text);
        let mapped = report
            .source_map
            .segments()
            .iter()
            .flat_map(|segment| segment.origin().bcis())
            .collect::<BTreeSet<_>>();
        for bci in [0, 1, 4, 5, 8, 11, 14, 15, 18, 19, 22] {
            assert!(
                mapped.contains(&bci),
                "BCI {bci} missing from source map: {}",
                report.text
            );
        }
    }

    #[test]
    fn three_test_chain_with_non_boolean_producer_quotes_every_instruction() {
        let changed = replace_one_bytecode_sequence(
            SHORT_CIRCUIT_CHAIN_FIXTURE,
            &[0x04, 0xa7, 0x00, 0x04, 0x03, 0xb3],
            &[0x05, 0xa7, 0x00, 0x04, 0x03, 0xb3],
        );
        let (region, _, _, _, report, attempt) =
            fixture_value_attempts(&changed, "assign", "(ZZ)V", None);
        assert!(
            matches!(region, Region::ShortCircuitValue { .. }),
            "{region:?}"
        );
        assert_eq!(
            attempt,
            ShortCircuitValueAttempt::Refused(ShortCircuitValueRefusal::Producer)
        );
        assert_shared_false_refusal_is_quoted(
            &report,
            &[0, 1, 4, 5, 8, 11, 14, 15, 18, 19, 22],
            &[19],
            &["ChainOrField.result ="],
        );
        assert!(!report.text.contains("jre_region_loop"), "{}", report.text);
    }

    #[test]
    fn frozen_shared_false_write_has_a_closed_ssa_value_proof() {
        let (region, _, _, _, report, attempt) =
            fixture_value_attempts(SHORT_CIRCUIT_BASE_FIXTURE, "<init>", "()V", None);
        assert!(matches!(region, Region::ShortCircuitValue { .. }));
        let ShortCircuitValueAttempt::Proved(proof) = attempt else {
            panic!("frozen BCI 12/15, 23/26, 29/33, 34 must prove: {attempt:?}");
        };
        assert_eq!(proof.test_bcis, [15, 26]);
        assert_eq!((proof.stack_depth, proof.consumer_bci), (0, 34));
        assert_ne!(proof.true_producer, proof.false_producer);
        assert_eq!(
            proof.consumer,
            ShortCircuitConsumer::Field(
                "AnonymousSuperDispatch".to_string(),
                "capturedVisibleBeforeBaseReturns".to_string(),
                "Z".to_string(),
            )
        );
        assert!(report.text.contains("capturedVisibleBeforeBaseReturns ="));
    }

    #[test]
    fn short_circuit_write_keeps_a_local_declaration_before_the_field_effect() {
        let (region, _, _, _, report, attempt) =
            fixture_value_attempts(SHORT_CIRCUIT_LOCAL_FIXTURE, "check", "(Z)V", None);
        assert!(matches!(region, Region::ShortCircuitValue { .. }));
        assert!(matches!(attempt, ShortCircuitValueAttempt::Proved(_)));
        let declaration = report
            .text
            .find("local1 = \"captured-value\"")
            .unwrap_or_else(|| panic!("the local is declared:\n{}", report.text));
        let assignment = report
            .text
            .find("LocalShortCircuit.visible =")
            .unwrap_or_else(|| panic!("the field is written:\n{}", report.text));
        assert!(declaration < assignment, "{}", report.text);
        assert_eq!(
            report.text.matches("LocalShortCircuit.visible =").count(),
            1
        );
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        assert!(report.source_map.segments().iter().any(|segment| {
            [4, 14, 17, 18, 21, 22]
                .iter()
                .all(|bci| segment.origin().bcis().contains(bci))
                && segment
                    .text(&report.text)
                    .contains("LocalShortCircuit.visible")
        }));
    }

    fn replace_one_bytecode_sequence(class: &[u8], before: &[u8], after: &[u8]) -> Vec<u8> {
        assert_eq!(before.len(), after.len());
        let matches: Vec<_> = class
            .windows(before.len())
            .enumerate()
            .filter_map(|(offset, bytes)| (bytes == before).then_some(offset))
            .collect();
        let [offset] = matches.as_slice() else {
            panic!("the frozen bytecode sequence must occur once: {before:?}, found {matches:?}");
        };
        let mut changed = class.to_vec();
        changed[*offset..*offset + before.len()].copy_from_slice(after);
        changed
    }

    /// A refused shared-false candidate must remain visible in the artifact from a fresh recovery
    /// of the mutated class. Checking only the proof against the original Region misses ownership
    /// and quality regressions in the actual report path.
    fn assert_shared_false_refusal_is_quoted(
        report: &crate::report::RecoveryReport,
        required_bcis: &[u32],
        putstatic_bcis: &[u32],
        forbidden_assignments: &[&str],
    ) {
        assert_eq!(
            report.quality,
            jarde_jvm::ir::Quality::Fallback,
            "{:?}\n{}",
            report.regions,
            report.text
        );
        assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
        for assignment in forbidden_assignments {
            assert!(
                !report.text.contains(assignment),
                "unexpected structured write `{assignment}`:\n{}",
                report.text
            );
        }

        let quoted_bcis: BTreeSet<u32> = report
            .text
            .lines()
            .filter(|line| line.trim_start().starts_with("// @bytecode "))
            .flat_map(|line| {
                line.split_whitespace()
                    .skip(2)
                    .take_while(|item| item.parse::<u32>().is_ok())
                    .map(|bci| bci.parse().expect("quoted BCI is numeric"))
            })
            .collect();
        let mapped_bcis: BTreeSet<u32> = report
            .source_map
            .segments()
            .iter()
            .flat_map(|segment| segment.origin().bcis().clone())
            .collect();
        for &bci in required_bcis {
            assert!(
                quoted_bcis.contains(&bci),
                "BCI {bci} is absent from the complete refusal quote:\n{}",
                report.text
            );
            assert!(
                mapped_bcis.contains(&bci),
                "BCI {bci} has no source-map origin:\n{}",
                report.text
            );
        }
        for &bci in putstatic_bcis {
            assert!(
                quoted_bcis.contains(&bci),
                "putstatic BCI {bci} must remain quoted on refusal:\n{}",
                report.text
            );
        }
    }

    #[test]
    fn redirected_outer_edge_proves_shared_true_after_fresh_recovery() {
        // Redirecting the outer jump to the true producer creates the other valid edge polarity.
        let shared_true = replace_one_bytecode_sequence(
            SHORT_CIRCUIT_BASE_FIXTURE,
            &[0xb2, 0x00, 0x07, 0x99, 0x00, 0x12],
            &[0xb2, 0x00, 0x07, 0x99, 0x00, 0x0e],
        );
        let (mutated_region, _, _, _, report, attempt) =
            fixture_value_attempts(&shared_true, "<init>", "()V", None);
        assert!(matches!(mutated_region, Region::ShortCircuitValue { .. }));
        assert!(
            matches!(attempt, ShortCircuitValueAttempt::Proved(_)),
            "the redirected edge has a shared-true proof: {attempt:?}"
        );
        assert_eq!(
            report
                .text
                .matches("capturedVisibleBeforeBaseReturns =")
                .count(),
            1
        );
    }

    #[test]
    fn shared_false_proof_rejects_non_boolean_producer_after_fresh_recovery() {
        // BCI 29 still supplies the Phi, but iconst_2 cannot be a boolean arm.
        let non_boolean = replace_one_bytecode_sequence(
            SHORT_CIRCUIT_BASE_FIXTURE,
            &[0x04, 0xa7, 0x00, 0x04, 0x03, 0xb3],
            &[0x05, 0xa7, 0x00, 0x04, 0x03, 0xb3],
        );
        let (mutated_region, _, _, _, report, attempt) =
            fixture_value_attempts(&non_boolean, "<init>", "()V", None);
        assert!(
            matches!(attempt, ShortCircuitValueAttempt::Refused(_)),
            "fresh recovery must refuse the mutated producer: {attempt:?}\n{mutated_region:?}"
        );
        assert_shared_false_refusal_is_quoted(
            &report,
            &[12, 15, 23, 26, 29, 30, 33, 34, 37, 38, 41],
            &[34],
            &["capturedVisibleBeforeBaseReturns ="],
        );
    }

    #[test]
    fn shared_false_proof_rejects_independent_test_effect_after_fresh_recovery() {
        // BCI 12 now pushes 1 followed by two independent nops before the BCI 15 test.
        let extra_instruction = replace_one_bytecode_sequence(
            SHORT_CIRCUIT_BASE_FIXTURE,
            &[0xb2, 0x00, 0x07, 0x99, 0x00, 0x12],
            &[0x04, 0x00, 0x00, 0x99, 0x00, 0x12],
        );
        let (mutated_region, _, _, _, report, attempt) =
            fixture_value_attempts(&extra_instruction, "<init>", "()V", None);
        assert!(
            matches!(attempt, ShortCircuitValueAttempt::Refused(_)),
            "fresh recovery must refuse the independent test effect: {attempt:?}\n{mutated_region:?}"
        );
        assert_shared_false_refusal_is_quoted(
            &report,
            &[12, 13, 14, 15, 23, 26, 29, 30, 33, 34, 37, 38, 41],
            &[34],
            &["capturedVisibleBeforeBaseReturns ="],
        );
    }

    #[test]
    fn shared_false_proof_rejects_second_write_after_fresh_recovery() {
        // javac's chained assignment keeps BCI 12/15, 23/26 and 29/33, then duplicates the
        // Phi at BCI 34 for two putstatic readers. No folding plan may consume this value.
        let (mutated_region, _, _, _, report, attempt) =
            fixture_value_attempts(SHORT_CIRCUIT_DOUBLE_WRITE_FIXTURE, "<init>", "()V", None);
        assert!(
            matches!(attempt, ShortCircuitValueAttempt::Refused(_)),
            "fresh recovery must refuse the second write: {attempt:?}\n{mutated_region:?}"
        );
        assert_shared_false_refusal_is_quoted(
            &report,
            &[12, 15, 18, 20, 23, 26, 29, 30, 33, 34, 35, 38],
            &[35, 38],
            &["Probe.other =", "Probe.other2 ="],
        );
    }

    #[test]
    fn shared_false_proof_requires_the_decoded_static_boolean_field() {
        // Change the class's sole UTF8 field descriptor Z to I. The bytecode and stack shape
        // remain valid, but the target field no longer licenses boolean assignment recovery.
        let integer_field = replace_one_bytecode_sequence(
            SHORT_CIRCUIT_BASE_FIXTURE,
            &[0x01, 0x00, 0x01, b'Z'],
            &[0x01, 0x00, 0x01, b'I'],
        );
        let (mutated_region, _, _, _, report, attempt) =
            fixture_value_attempts(&integer_field, "<init>", "()V", None);
        assert!(matches!(mutated_region, Region::ShortCircuitValue { .. }));
        assert_eq!(
            attempt,
            ShortCircuitValueAttempt::Refused(ShortCircuitValueRefusal::Field)
        );
        assert_shared_false_refusal_is_quoted(
            &report,
            &[12, 15, 23, 26, 29, 30, 33, 34, 37, 38, 41],
            &[34],
            &["capturedVisibleBeforeBaseReturns ="],
        );
    }

    #[test]
    fn frozen_shared_true_write_proves_and_emits_one_delayed_rhs_assignment() {
        let (region, _, _, _, report, attempt) =
            fixture_value_attempts(SHORT_CIRCUIT_SHARED_TRUE_FIXTURE, "assign", "(Z)V", None);
        assert!(matches!(region, Region::ShortCircuitValue { .. }));
        let ShortCircuitValueAttempt::Proved(proof) = attempt else {
            panic!("frozen BCI 1/7, 10/14, 15 must prove: {attempt:?}\n{region:?}");
        };
        assert_eq!(proof.test_bcis, [1, 7]);
        assert_eq!(proof.consumer_bci, 15);
        assert_eq!(
            proof.consumer,
            ShortCircuitConsumer::Field(
                "SharedTrueShortCircuit".to_string(),
                "result".to_string(),
                "Z".to_string(),
            )
        );
        assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
        assert_eq!(report.quality, jarde_jvm::ir::Quality::Structured);
        assert_eq!(
            report
                .text
                .matches("SharedTrueShortCircuit.result =")
                .count(),
            1
        );
        assert!(report.text.contains("rhs()"), "{}", report.text);
        assert!(report.text.contains("?"), "{}", report.text);
        assert!(report.text.contains(": 1"), "{}", report.text);
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        let mapped_bcis: BTreeSet<_> = report
            .source_map
            .segments()
            .iter()
            .flat_map(|segment| segment.origin().bcis().clone())
            .collect();
        for bci in [0, 1, 4, 7, 10, 11, 14, 15] {
            assert!(
                mapped_bcis.contains(&bci),
                "BCI {bci} is unmapped:\n{}",
                report.text
            );
        }
    }

    #[test]
    fn shared_true_duplicate_phi_consumer_is_freshly_quoted() {
        let (region, _, _, _, report, attempt) = fixture_value_attempts(
            SHORT_CIRCUIT_SHARED_TRUE_DUPLICATE_PHI_FIXTURE,
            "assign",
            "(Z)V",
            None,
        );
        assert!(matches!(region, Region::ShortCircuitValue { .. }));
        assert!(
            matches!(attempt, ShortCircuitValueAttempt::Refused(_)),
            "fresh recovery must reject the duplicated Phi consumer: {attempt:?}\n{region:?}"
        );
        assert_shared_false_refusal_is_quoted(
            &report,
            &[1, 7, 10, 11, 14, 15, 16, 19, 22],
            &[16, 19],
            &[
                "SharedTrueDuplicatePhi.other =",
                "SharedTrueDuplicatePhi.result =",
            ],
        );
    }

    #[test]
    fn shared_true_non_boolean_producer_is_freshly_quoted() {
        // iconst_1 at BCI 10 becomes iconst_2. The JVM frame remains the same int type, but
        // interpreting this producer as boolean true would erase its low-bit Z-store semantics.
        let (region, _, _, _, report, attempt) = fixture_value_attempts(
            SHORT_CIRCUIT_SHARED_TRUE_NON_BOOLEAN_FIXTURE,
            "assign",
            "(Z)V",
            None,
        );
        assert!(matches!(region, Region::ShortCircuitValue { .. }));
        assert_eq!(
            attempt,
            ShortCircuitValueAttempt::Refused(ShortCircuitValueRefusal::Producer),
            "fresh recovery must isolate the non-1 producer refusal: {region:?}"
        );
        assert_shared_false_refusal_is_quoted(
            &report,
            &[1, 4, 7, 10, 11, 14, 15, 18],
            &[15],
            &["SharedTrueShortCircuit.result ="],
        );
    }

    #[test]
    fn ordinary_if_and_real_loop_keep_their_region_forms() {
        let (_, _, _, conditional, _) =
            fixture_conditional_attempt(TERNARY_CORE_FIXTURE, "returned", "(Z)I");
        assert!(matches!(
            conditional.regions.first(),
            Some(Region::If { .. })
        ));
        assert!(
            conditional
                .regions
                .iter()
                .all(|region| !matches!(region, Region::ShortCircuitValue { .. }))
        );

        let (_, _, _, looping, _) =
            fixture_conditional_attempt(LOOP_TRANSFERS_FIXTURE, "run", "(I)I");
        assert!(
            looping
                .regions
                .iter()
                .any(|region| matches!(region, Region::Loop { .. }))
        );
        assert!(
            looping
                .regions
                .iter()
                .all(|region| !matches!(region, Region::ShortCircuitValue { .. }))
        );
    }

    #[test]
    fn carried_constructor_argument_keeps_both_conditionals_and_all_origins() {
        let (_, _, _, _, report) = fixture_conditional_attempt(
            include_bytes!(
                "../../../openspec/evidence/java-syntax-2026-09-25/constructor-conditional-delegation/ConstructorPairProbe.class"
            ),
            "<init>",
            "(Ljava/lang/String;I)V",
        );
        assert!(report.produced());
        assert!(
            report
                .text
                .contains("this(arg2 == 1 ? arg1 : \"\", arg2 == 0 ? \"\" : arg1);"),
            "{}",
            report.text
        );
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        for bci in [0, 1, 2, 3, 6, 7, 10, 12, 13, 16, 18, 21, 22, 25] {
            assert!(
                !report.source_map.of_bci(bci).is_empty(),
                "BCI {bci} is absent from the source map"
            );
        }
    }
}
