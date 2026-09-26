//! ③ Region recovery: what the method's *structure* is, decided from the normal-flow view and the
//! decoded branch facts, and written down as a tree the AST builder reads (P3 1.2's third row).
//!
//! # What a Region decides and what it does not
//!
//! A Region decides shape: this run of blocks is a straight line, this one ends in a two-way branch
//! whose arms are these two regions meeting at that join, and this one cannot be shown to be a Java
//! structure at all (then and only then: [`region::Region::Fallback`]). It **does not emit text**,
//! **does not decode bytecode** and **does not decide syntax** — the spelling of a condition, the
//! negation that turns a jump sense into a fall-through condition, the names and the statements all
//! belong to [`crate::build`] and [`crate::emit`]. Keeping that line is what lets the same Region
//! tree be presented differently later (2.x patterns) without re-deriving the structure.
//!
//! # The provable subset, stated as preconditions
//!
//! A region is recovered only when all of these hold; each one that fails produces a [`Fallback`]
//! with its own reason, never a guess and never an empty body:
//!
//! 1. **No leaving edge carries structure.** A block with an exception edge is a handler's target
//!    and a block with a `jsr` context entry is a subroutine body: neither is a Java statement, so
//!    the region is unprovable and the bytecode is quoted instead. The exception edge counts only
//!    when an instruction of the block can take it (P3 2.15): a record no `may_throw` instruction
//!    of the block covers is stated by its protected range (P3 2.9), and an edge nothing in the
//!    block can raise from is not a way out of it ([`Walker::exception_edge_takeable`]).
//! 2. **Straight means straight.** A block with no plain successor ends the run; a block with one
//!    continues it; a block with two is an `if`. Three or more successors is a `switch`, which
//!    1.3b recovers and this slice refuses.
//! 3. **The graph is acyclic where it is claimed to be structured.** Reaching a block that is
//!    already part of the recovered structure means a loop or a merge the subset does not model, and
//!    the whole region falls back rather than being printed as a straight line that runs twice.
//! 4. **Both arms meet.** The two successors of a branch must reach one join — the branch's own
//!    immediate post-dominator — with the arms not reaching into each other. A successor that *is*
//!    the join is the one-armed shape the bytecode states: that arm is empty, the other successor's
//!    region is the statement's one body, and the `if` is written without an `else`.
//! 5. **The branch's own facts are there.** The branch instruction needs a decoded
//!    [`Operation::Comparison`] and the arity that operation claims; the *polarity* is not guessed
//!    from the shape (an `if (a)` and an `if (!a)` have the same graph), so a branch whose sense is
//!    undecoded is unprovable.
//! 6. **Uncovered blocks are stated, not dropped.** Every live block the walk did not claim is
//!    reported as a fallback region listing exactly those blocks, which is how a body reachable only
//!    through exception or `jsr` edges stays visible instead of turning into an empty body.

use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{
    CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind, Definition, MethodIr, PhiInput, Slot,
    SsaBlock, SsaTable, ValueId,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{ExceptionHandlerFact, MethodCodeFacts};

use crate::decode::Operations;
use crate::facts::{CompareOp, ConstantValue, Operation};
use crate::normal_flow::NormalFlowView;
use crate::stop::{StopReason, charge, poll};

/// Why one region could only be kept as bytecode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FallbackReason {
    /// An exception edge leaves the region: a handler's shape, which 2.4 recovers.
    ExceptionEdge {
        block_bci: u32,
        handler_ordinal: u32,
    },
    /// A `jsr` context entry leaves the region: a subroutine body is not a Java statement.
    SubroutineEntry { block_bci: u32, call_site: u32 },
    /// A branch with three or more targets whose terminal instruction is not a decoded `switch`.
    BranchTargets { block_bci: u32, successors: usize },
    /// The region can be re-entered and the loop it belongs to is not one this subset proves.
    Loop { block_bci: u32 },
    /// Two positions in the completed Region tree claim the same physical canonical block.
    /// A repeated visit to a join does not by itself prove a loop.
    OwnershipOverlap { block: CanonicalBlockId },
    /// The loop's own shape is not one of the two this subset proves: its test, its exit or its
    /// single latch does not line up with the blocks that iterate.
    LoopShape { block_bci: u32 },
    /// A block inside a loop body leaves the loop by an edge that is not its test — a `break`, or
    /// an arm that jumps past the loop. Presenting the loop would drop that edge.
    LoopLeavesEarly { block_bci: u32 },
    /// The graph itself is not reducible where this region is: a loop entered at more than its
    /// header, or two loops crossing. No Java structure has that shape.
    Irreducible { blocks: Vec<u32> },
    /// Two exception-table records' declared ranges cross: neither nested nor disjoint.
    ///
    /// A crossing pair is the case where "the innermost protecting record" is not an ordering a
    /// presentation could take for granted — a compiler does not emit it, and the priority of the
    /// handlers inside it is a fact of the table rather than of the shape.
    CrossingExceptionRegions {
        record: u32,
        other: u32,
        blocks: Vec<u32>,
    },
    /// The branch's sense was not decoded, so the two arms cannot be told apart.
    UnknownBranchSense { block_bci: u32, branch_bci: u32 },
    /// The branch's decoded arity or target does not match the values it reads and the successors
    /// the graph holds.
    UnrenderableOperand { bci: u32 },
    /// The two arms do not meet at one join.
    ArmsDoNotMeet { block_bci: u32 },
    /// A bounded short-circuit value shape has one physical owner, but its value has not yet been
    /// proved by the SSA consumer rule. The builder must quote the complete shape until that rule
    /// is available.
    ShortCircuitValueUnproved {
        first_branch_bci: u32,
        last_branch_bci: u32,
        consumer_bci: u32,
    },
    /// A pass stated a precondition ([`crate::pass::Precondition`], P3 decision 1) and this run's
    /// evidence does not meet it: the shape was **not** claimed.
    ///
    /// This is the one path a declared precondition fails through. The reason names the pass and
    /// its rule version, the requirement that was not met and where the evidence fell short, so
    /// that a reader can tell "the `loop@1` rule refused this test block" from "nothing here is a
    /// loop"; the variable that carries the *instance* of the refusal (which block, which
    /// instruction) is filled by the check that consulted the declaration, never invented here.
    ///
    /// The live instance of this slice is the loop's test block: a loop's test runs once per
    /// iteration, and a `while (…)`/`do … while (…)` has nowhere to write a statement from the test
    /// block that would run that often — hoisting it out of the loop would run it once, and putting
    /// it in the body would run it after the test. So a loop whose test writes state is quoted
    /// instead of being presented with its effect moved. (An `if`'s or a `switch`'s test block has
    /// no such problem: its effects run exactly once, before the test, and [`crate::build`] writes
    /// them there in order.)
    UnmetPrecondition {
        /// The declared pass whose precondition was checked.
        pass: &'static crate::pass::Pass,
        /// The precondition the pass states and this run does not meet.
        requirement: crate::pass::Precondition,
        /// The block the pass was about to claim (the loop's test block, here).
        block_bci: u32,
        /// The instruction whose evidence fell short, when the requirement is about one.
        at: u32,
    },
    /// Two `switch` arms claim the same block: a case whose code falls through into another case's.
    SwitchArmsOverlap { block_bci: u32 },
    /// The decoded keys and targets of a `switch` do not line up with the successors the graph
    /// holds for it.
    SwitchShape { block_bci: u32 },
    /// A guarded region — a `try`-with-resources, a `synchronized` statement or the `finally` shape
    /// javac copies — that a rule of P3 2.4 examined and did **not** present.
    ///
    /// This is the one reason a guarded shape is refused through: the rule that examined it says
    /// which link of its proof fell short ([`crate::guard::Unproven`]), and the code and the message
    /// are that rule's own, exactly like a site's refusal ([`crate::refusal::Refusal`]). A handler
    /// that is not a guarded region at all keeps this walk's own [`Self::ExceptionEdge`]: "no rule
    /// examined this" and "a rule examined this and refused it" are different facts, and a reader of
    /// the artifact needs to tell them apart.
    Guard {
        /// The declared rule that examined the shape, when one did. The `finally` copy is examined
        /// by *no* rule — this build refuses it rather than presenting it — so a reader that asked
        /// "which rule refused this" gets no name instead of a borrowed one.
        pass: Option<&'static crate::pass::Pass>,
        /// The refusal's diagnostic code.
        code: &'static str,
        /// The instruction the refusal is about.
        at: u32,
        /// What the rule states about it, in one sentence.
        message: String,
    },
    /// Live blocks the walk did not claim, reachable only through edges the projection leaves out.
    UncoveredBlocks { blocks: Vec<u32> },
    /// The decoded body states instruction(s) that **no** canonical block covers and that the graph
    /// does not list as unreachable (P3-R7).
    ///
    /// This is the third fact of the same kind as [`Self::Irreducible`] and
    /// [`Self::CrossingExceptionRegions`]: a premise of the whole presentation that the graph itself
    /// contradicts. A block-or-dead instruction is a body this walk can present or refuse by region,
    /// because every instruction of the body is accounted for by *some* node; an instruction that is
    /// in neither is a body the graph is not an account of at all — the normalization never created a
    /// node for it and never said why not, so no region of this walk covers it and no quote of the
    /// regions would name it either. Presenting the blocks that *are* accounted for would then state
    /// a body while silently dropping the instructions the class file really holds (the ECJ 4.6.1
    /// v52 `finallyPath` is the committed case: its `Exception table` names a handler at BCI 9 and the
    /// decode reads BCIs 9/10/13/14, while the graph's only block is `[0, 9)`).
    ///
    /// The `bcis` are the unaccounted instruction starts, ascending, and they reach the artifact in
    /// one of two ways, depending on what the walk was going to say about the body: as the reason of
    /// a whole-body refusal when every region the walk produced was structured (nothing else was
    /// going to be said, and the body may not be presented as if it were complete), or as the reason
    /// of a region of its own beside the refusals the walk did find (a specific reason a rule
    /// examined is worth more than "the graph has a hole", and the bytes still have to be named).
    /// [`crate::build`] reads [`Self::unaccounted`] in both cases, so the quote names them either way.
    UnaccountedInstructions { bcis: Vec<u32> },
    /// The block's own evidence is incomplete: it branches but the names table states no last
    /// instruction for it, so which instruction branches cannot be stated.
    MissingEvidence { block_bci: u32 },
}

impl FallbackReason {
    /// The diagnostic code this reason is reported under.
    pub fn code(&self) -> &'static str {
        match self {
            Self::ExceptionEdge { .. } => "jre_region_exception_edge",
            Self::SubroutineEntry { .. } => "jre_region_subroutine_entry",
            Self::BranchTargets { .. } => "jre_region_branch_targets",
            Self::Loop { .. } => "jre_region_loop",
            Self::OwnershipOverlap { .. } => "jre_region_ownership_overlap",
            Self::LoopShape { .. } => "jre_region_loop_shape",
            Self::LoopLeavesEarly { .. } => "jre_region_loop_leaves_early",
            Self::Irreducible { .. } => "jre_region_irreducible",
            Self::CrossingExceptionRegions { .. } => "jre_region_crossing_exception_regions",
            Self::UnknownBranchSense { .. } => "jre_region_unknown_branch_sense",
            Self::UnrenderableOperand { .. } => "jre_region_unrenderable_operand",
            Self::ArmsDoNotMeet { .. } => "jre_region_arms_do_not_meet",
            Self::ShortCircuitValueUnproved { .. } => "jre_region_short_circuit_value_unproved",
            Self::UnmetPrecondition { .. } => "jre_region_unmet_precondition",
            Self::SwitchArmsOverlap { .. } => "jre_region_switch_arms_overlap",
            Self::SwitchShape { .. } => "jre_region_switch_shape",
            Self::Guard { code, .. } => code,
            Self::UncoveredBlocks { .. } => "jre_region_uncovered_blocks",
            Self::UnaccountedInstructions { .. } => "jre_region_unaccounted_instruction",
            Self::MissingEvidence { .. } => "jre_region_missing_evidence",
        }
    }

    /// The instruction starts this reason states that **no** canonical block covers, ascending.
    ///
    /// Only [`Self::UnaccountedInstructions`] states any: every other reason, whole-body or by
    /// region, is about blocks the graph holds, and a quote of that region already names them. A
    /// refusal that does list them has to quote them too ([`crate::build`] reads this to do it), or
    /// the artifact would refuse a body while dropping exactly the bytes it refused it for.
    pub fn unaccounted(&self) -> &[u32] {
        match self {
            Self::UnaccountedInstructions { bcis } => bcis,
            _ => &[],
        }
    }

    /// The one way a declared precondition fails.
    ///
    /// The check site states *which* declaration it consulted and what the evidence was, and the
    /// debug assertion keeps the two in step: a refusal can only be stated for a requirement the
    /// pass really declares, so a precondition that was declared and then never checked — or
    /// checked without being declared — fails the build's own tests instead of drifting.
    pub fn unmet(
        pass: &'static crate::pass::Pass,
        requirement: crate::pass::Precondition,
        block_bci: u32,
        at: u32,
    ) -> Self {
        debug_assert!(
            pass.requires(requirement),
            "{} states no {requirement:?} precondition",
            pass.rule()
        );
        Self::UnmetPrecondition {
            pass,
            requirement,
            block_bci,
            at,
        }
    }

    /// The declared pass whose precondition this reason says is unmet, when that is what it says.
    ///
    /// The other reasons are the walk's own statements about shapes (`ArmsDoNotMeet`,
    /// `UncoveredBlocks`, …) or preconditions 2.x will move into declarations; `None` here means
    /// "no registered rule refused this", not "the refusal has no rule".
    pub fn pass(&self) -> Option<&'static crate::pass::Pass> {
        match self {
            Self::UnmetPrecondition { pass, .. } => Some(pass),
            Self::Guard { pass, .. } => *pass,
            _ => None,
        }
    }

    /// What the reason says, in one sentence, for the artifact's own comment and the report.
    pub fn message(&self) -> String {
        match self {
            Self::ExceptionEdge {
                block_bci,
                handler_ordinal,
            } => format!(
                "block at BCI {block_bci} leaves through exception handler {handler_ordinal}: a handler's shape is not part of the recoverable subset"
            ),
            Self::SubroutineEntry {
                block_bci,
                call_site,
            } => format!(
                "block at BCI {block_bci} is entered by the jsr at BCI {call_site}: a subroutine body is not a Java statement"
            ),
            Self::BranchTargets {
                block_bci,
                successors,
            } => format!(
                "block at BCI {block_bci} branches to {successors} targets and its terminal instruction is not a decoded switch"
            ),
            Self::Loop { block_bci } => format!(
                "block at BCI {block_bci} can be re-entered and belongs to no loop this subset proves"
            ),
            Self::OwnershipOverlap { block } => format!(
                "canonical block at BCI {} on jsr path {:?} has more than one owner in the completed Region tree; the whole method is quoted",
                block.bci(),
                block.path()
            ),
            Self::LoopShape { block_bci } => format!(
                "the loop whose header is the block at BCI {block_bci} has a test, an exit or a latch this subset does not prove"
            ),
            Self::LoopLeavesEarly { block_bci } => format!(
                "the block at BCI {block_bci} leaves its loop by an edge that is not the loop's test: presenting the loop would drop it"
            ),
            Self::Irreducible { blocks } => format!(
                "the graph is not reducible over {} block(s) {blocks:?}: a loop is entered at more than its header or two loops cross, which no Java structure spells",
                blocks.len()
            ),
            Self::CrossingExceptionRegions {
                record,
                other,
                blocks,
            } => format!(
                "exception records {record} and {other} have crossing ranges over {} block(s) {blocks:?}: the handler priority inside them is the table's fact, not a shape",
                blocks.len()
            ),
            Self::UnknownBranchSense {
                block_bci,
                branch_bci,
            } => format!(
                "the branch at BCI {branch_bci} in block {block_bci} has no decoded sense, so its two arms cannot be told apart"
            ),
            Self::UnrenderableOperand { bci } => format!(
                "the decoded operands of the instruction at BCI {bci} do not line up with the graph's successors"
            ),
            Self::ArmsDoNotMeet { block_bci } => {
                format!("the arms of the branch in block {block_bci} do not meet at one join")
            }
            Self::ShortCircuitValueUnproved {
                first_branch_bci,
                last_branch_bci,
                consumer_bci,
            } => format!(
                "the short-circuit chain from BCI {first_branch_bci} through {last_branch_bci} reaches a shared value consumer at BCI {consumer_bci}, but this slice has no SSA proof for that value; the complete region is quoted"
            ),
            Self::UnmetPrecondition {
                pass,
                requirement,
                block_bci,
                at,
            } => format!(
                "the {} rule did not claim the block at BCI {block_bci}: it requires {}, and the instruction at BCI {at} is not part of one, so presenting the structure would have moved that effect out of the shape it decides",
                pass.rule(),
                requirement.describe()
            ),
            Self::SwitchArmsOverlap { block_bci } => format!(
                "two switch arms of the block at BCI {block_bci} claim the same block, so one case falls through into another case's code"
            ),
            Self::SwitchShape { block_bci } => format!(
                "the decoded keys and targets of the switch at BCI {block_bci} do not line up with the successors the graph holds"
            ),
            Self::UncoveredBlocks { blocks } => format!(
                "{} live block(s) are reachable only through edges the normal-flow view leaves out: {blocks:?}",
                blocks.len()
            ),
            Self::UnaccountedInstructions { bcis } => format!(
                "the decoded body states {} instruction(s) at BCI {bcis:?} that no canonical block covers and that the graph does not list as unreachable: the graph is not an account of these bytes, so no region of it may be presented as the body",
                bcis.len()
            ),
            Self::MissingEvidence { block_bci } => format!(
                "block at BCI {block_bci} has no last instruction the names table states, so its branch cannot be named"
            ),
            Self::Guard { message, .. } => message.clone(),
        }
    }
}

/// Which way through a loop's test keeps the loop iterating.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Continuation {
    /// Control iterates again when the test's branch **transfers** — the target is the block that
    /// repeats. This is the shape a bottom-tested loop compiles to (`test: if_icmplt body`).
    Taken,
    /// Control iterates again when the test's branch **falls through** — the target is where the
    /// loop leaves. This is the shape a header-tested loop compiles to (`header: ifeq exit`).
    FallThrough,
}

/// Whether a loop tests before its body or after it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoopForm {
    /// `while (cond) { … }`: the test is the header, and the body runs only when it holds.
    While,
    /// `do { … } while (cond);`: the test is the latch, and the body runs once before it is read.
    DoWhile,
}

/// One group of `switch` keys that share a target, with the region that target begins.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwitchGroup {
    /// The keys that select this arm, in the order the decode enumerated them.
    pub keys: Vec<i64>,
    /// Whether the no-match case runs this arm too — either because the default target is this
    /// group's target, or because this group *is* the default alone (then `keys` is empty).
    pub default: bool,
    /// This arm's straight-line code ends at the next case entry, so Java must continue into that
    /// following arm without an intervening `break`.
    pub fall_through: bool,
    /// The arm, which is an empty straight run when the target is the switch's own join — the
    /// `case 0: break;` shape.
    pub arm: Box<Region>,
}

/// A counted loop's two instructions that a `for` header may own. The latch block still belongs
/// to the loop body in the region tree; only its single update statement moves to the header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForHeader {
    pub init_bci: u32,
    pub update_bci: u32,
    pub update_block: CanonicalBlockId,
    pub slot: u16,
}

/// What one region is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Region {
    /// An ordered sequence needed when one branch arm contains multiple child regions.
    Sequence { regions: Vec<Region> },
    /// A run of blocks with at most one plain successor each, ending where the structure changes.
    Straight { blocks: Vec<CanonicalBlockId> },
    /// A straight-line prefix ending in a two-way branch, with both arms recovered.
    ///
    /// `join` is the block both arms meet at, or `None` when both arms leave the method — the two
    /// cases the recovery distinguishes, because the first one continues after the `if` and the
    /// second one does not.
    If {
        prefix: Vec<CanonicalBlockId>,
        branch: CanonicalBlockId,
        branch_bci: u32,
        then_arm: Box<Region>,
        else_arm: Box<Region>,
        join: Option<CanonicalBlockId>,
    },
    /// A bounded short-circuit value graph whose shared producer and consumer cannot be
    /// represented by disjoint `If` arms. It claims each physical block once. The builder may
    /// present its value and consumer only after its separate SSA and type proof succeeds.
    ShortCircuitValue {
        /// Blocks already walked before the outer test; they remain before the quoted shape.
        prefix: Vec<CanonicalBlockId>,
        tests: Vec<(CanonicalBlockId, u32)>,
        /// Decoded fallthrough and taken successor for each test, in `tests` order.
        test_edges: Vec<(CanonicalBlockId, CanonicalBlockId)>,
        /// Pure direct forward transfers, retaining each physical node and its one successor.
        gateways: Vec<(CanonicalBlockId, CanonicalBlockId)>,
        true_producer: CanonicalBlockId,
        false_producer: CanonicalBlockId,
        consumer: CanonicalBlockId,
        consumer_bci: u32,
        reason: FallbackReason,
    },
    /// One bounded, single-entry test DAG ending at two shared boolean returns.
    /// Ownership is committed once; the builder must independently prove its Java expression.
    TwoExitReturn {
        prefix: Vec<CanonicalBlockId>,
        tests: Vec<(CanonicalBlockId, u32)>,
        test_edges: Vec<(CanonicalBlockId, CanonicalBlockId)>,
        gateways: Vec<(CanonicalBlockId, CanonicalBlockId)>,
        true_return: CanonicalBlockId,
        false_return: CanonicalBlockId,
    },
    /// A prefix ending in a `tableswitch`/`lookupswitch`, with one arm per distinct target.
    ///
    /// `groups` holds one arm per distinct target. It retains decode order unless a proven
    /// fallthrough requires the targets' execution order; the no-match case is stated on the group
    /// whose target it shares (or on a group of its own). A target that is the join is an empty
    /// arm, which the emitter writes as a `case` that breaks immediately.
    Switch {
        prefix: Vec<CanonicalBlockId>,
        branch: CanonicalBlockId,
        branch_bci: u32,
        groups: Vec<SwitchGroup>,
        join: Option<CanonicalBlockId>,
    },
    /// One certified javac String dispatch. `dispatch` owns the hash switch, its comparison
    /// branches and the final integer switch; only the final switch's bodies survive as arms.
    StringSwitch {
        dispatch: Vec<CanonicalBlockId>,
        proof: crate::stringswitch::Proof,
        groups: Vec<SwitchGroup>,
        join: Option<CanonicalBlockId>,
    },
    /// A natural loop with an ordered, proved set of test branches.
    ///
    /// A header-tested loop owns its header test and any additional homogeneous short-circuit
    /// tests in execution order. A latch-tested loop currently owns its one proved latch test.
    /// Keeping every test *inside* the region is the point: the values it reads and calls it makes
    /// are written in the loop condition, so each iteration evaluates exactly what the bytecode did.
    Loop {
        header: CanonicalBlockId,
        /// Ordered tests owned by this loop. Each continuation is the branch sense whose
        /// condition is true for this test's role in the proved loop condition.
        tests: Vec<(CanonicalBlockId, u32, Continuation)>,
        /// The short-circuit operator proved by a multi-block header test chain.
        test_operator: Option<crate::ast::BinaryOp>,
        form: LoopForm,
        for_header: Option<ForHeader>,
        /// The regions of one iteration, in normal-flow order. A nested loop, `try`, or `switch`
        /// can end before its enclosing loop does; the following regions are still part of this
        /// body.
        body: Vec<Region>,
        exit: Option<CanonicalBlockId>,
    },
    /// An edge in a loop body whose target is exactly this loop's exit.
    LoopBreak {
        /// The terminal instruction that transfers to the exit.
        source_bci: u32,
        /// The identity of the loop whose exit the edge reaches.
        loop_header: CanonicalBlockId,
    },
    /// A transfer edge to the proved continue target of an enclosing loop: its test for a
    /// `while`, or its unique update latch when a `for` header owns that update.
    LoopContinue {
        source_bci: u32,
        loop_header: CanonicalBlockId,
    },
    /// A run of blocks that could not be shown to be a Java structure; the text quotes it.
    Fallback {
        blocks: Vec<CanonicalBlockId>,
        reason: FallbackReason,
    },
    /// A guarded statement a rule of P3 2.4 proved: a `try`-with-resources or a `synchronized`
    /// block, with the region it guards inside it.
    ///
    /// The region owns **every** block of the statement — the resources' own initialisations, the
    /// body, the closes the normal path performs and the handlers — because none of the others may
    /// be written as statements of its own: the closes would run twice, and the header's text is
    /// where a resource's initialisation belongs. `prefix` is what the walk had already claimed
    /// before the statement's own header block, which is written first, exactly like an `if`'s
    /// prefix.
    Guard {
        /// The blocks written before the statement, in order.
        prefix: Vec<CanonicalBlockId>,
        /// What the rule proved: the shape, the guarded body and every block the statement owns.
        plan: crate::guard::Plan,
    },
    /// `try { … } catch (T n) { … }` — a protected range the exception table states, with one clause
    /// per row that names its `catch` type.
    ///
    /// This is the *structure the table states* and not a shape a rule of P3 2.4 proved: the walk
    /// recovers the protected range and every handler body through the same recursion as any other
    /// region, which is what lets a clause body hold a branch, a loop or a quote of its own. The
    /// region is presented only where no guarded rule owns the block ([`crate::guard::catches`]), so
    /// a `try`-with-resources or a `synchronized` block is never restated as this one.
    Try {
        /// The blocks written before the statement, in order.
        prefix: Vec<CanonicalBlockId>,
        /// The instructions of the statement's own block that are written **before** the `try`
        /// ([`crate::guard::Catches::lead`]): the range from that block's start to the protected
        /// range's start, empty where the two are the same. They are not instructions of the
        /// protected range, so they are written before the statement — and because they live in the
        /// block the body begins in, the body does not write them a second time.
        lead: (u32, u32),
        /// The protected range, recovered as a region of its own.
        body: Box<Region>,
        /// The clauses, in exception-table order.
        catches: Vec<CatchClause>,
    },
}

/// One `catch` clause of a presented `try`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatchClause {
    /// Constant-pool indexes of the `catch` types this clause names, in exception-table order:
    /// [`crate::build`] spells them from the class file's pool, one class for an ordinary clause and
    /// `A | B` for the multi-catch rows that share one handler.
    type_indices: Vec<u16>,
    /// The canonical block the rows' handler entry maps to.
    handler: CanonicalBlockId,
    /// The local slot the handler's own first instruction stores the caught exception into: the
    /// clause's parameter, named by the same table every other local is named by.
    parameter: u16,
    /// The handler's body, from its entry to where the code after the `try` begins.
    body: Box<Region>,
}

impl CatchClause {
    /// Constant-pool indexes of the `catch` types this clause names.
    pub fn type_indices(&self) -> &[u16] {
        &self.type_indices
    }

    /// The block this clause's handler entry is.
    pub fn handler(&self) -> &CanonicalBlockId {
        &self.handler
    }

    /// The slot the handler's first instruction stores the caught exception into.
    pub fn parameter(&self) -> u16 {
        self.parameter
    }

    /// The handler's body.
    pub fn body(&self) -> &Region {
        &self.body
    }
}

impl Region {
    /// The exact instruction starts of cleanup already proved and owned by resource guards.
    pub(crate) fn twr_cleanup_bcis(&self, out: &mut Vec<u32>) {
        match self {
            Self::Sequence { regions } => {
                for region in regions {
                    region.twr_cleanup_bcis(out);
                }
            }
            Self::If {
                then_arm, else_arm, ..
            } => {
                then_arm.twr_cleanup_bcis(out);
                else_arm.twr_cleanup_bcis(out);
            }
            Self::Switch { groups, .. } | Self::StringSwitch { groups, .. } => {
                for group in groups {
                    group.arm.twr_cleanup_bcis(out);
                }
            }
            Self::Loop { body, .. } => {
                for region in body {
                    region.twr_cleanup_bcis(out);
                }
            }
            Self::Guard { plan, .. } => {
                if let crate::guard::Shape::Resources { cleanup, .. } = plan.shape() {
                    out.extend(cleanup.iter().copied());
                }
            }
            Self::Try { body, catches, .. } => {
                body.twr_cleanup_bcis(out);
                for clause in catches {
                    clause.body.twr_cleanup_bcis(out);
                }
            }
            Self::Straight { .. }
            | Self::ShortCircuitValue { .. }
            | Self::TwoExitReturn { .. }
            | Self::LoopBreak { .. }
            | Self::LoopContinue { .. }
            | Self::Fallback { .. } => {}
        }
    }

    /// Every block this region claims, in the order the method runs them.
    pub fn blocks(&self) -> Vec<&CanonicalBlockId> {
        match self {
            Self::Sequence { regions } => regions.iter().flat_map(Self::blocks).collect(),
            Self::Straight { blocks } => blocks.iter().collect(),
            Self::If {
                prefix,
                branch,
                then_arm,
                else_arm,
                ..
            } => {
                let mut blocks: Vec<&CanonicalBlockId> = prefix.iter().collect();
                blocks.push(branch);
                blocks.extend(then_arm.blocks());
                blocks.extend(else_arm.blocks());
                blocks
            }
            Self::ShortCircuitValue {
                prefix,
                tests,
                gateways,
                true_producer,
                false_producer,
                consumer,
                ..
            } => {
                let mut blocks = prefix.iter().collect::<Vec<_>>();
                for (block, _) in tests {
                    if !blocks.contains(&block) {
                        blocks.push(block);
                    }
                }
                for (block, _) in gateways {
                    if !blocks.contains(&block) {
                        blocks.push(block);
                    }
                }
                for block in [true_producer, false_producer, consumer] {
                    if !blocks.contains(&block) {
                        blocks.push(block);
                    }
                }
                blocks
            }
            Self::TwoExitReturn {
                prefix,
                tests,
                gateways,
                true_return,
                false_return,
                ..
            } => {
                let mut blocks = prefix.iter().collect::<Vec<_>>();
                blocks.extend(tests.iter().map(|(block, _)| block));
                blocks.extend(gateways.iter().map(|(block, _)| block));
                blocks.extend([true_return, false_return]);
                blocks
            }
            Self::Switch {
                prefix,
                branch,
                groups,
                ..
            } => {
                let mut blocks: Vec<&CanonicalBlockId> = prefix.iter().collect();
                blocks.push(branch);
                for group in groups {
                    blocks.extend(group.arm.blocks());
                }
                blocks
            }
            Self::StringSwitch {
                dispatch, groups, ..
            } => {
                let mut blocks: Vec<&CanonicalBlockId> = dispatch.iter().collect();
                for group in groups {
                    blocks.extend(group.arm.blocks());
                }
                blocks
            }
            Self::Loop {
                header,
                tests,
                form,
                body,
                ..
            } => {
                let mut blocks = Vec::new();
                // A one-block `do … while` writes that block's statements in its body before
                // testing the branch in the same block. The header/test fields name the shape,
                // while the body is its one physical owner. Keep duplicates *inside* the body
                // visible to the method-level ownership check.
                if *form != LoopForm::DoWhile || !tests.iter().any(|(test, _, _)| test == header) {
                    blocks.push(header);
                }
                blocks.extend(
                    tests
                        .iter()
                        .map(|(test, _, _)| test)
                        .filter(|test| *test != header),
                );
                for region in body {
                    blocks.extend(region.blocks());
                }
                if *form == LoopForm::DoWhile
                    && tests.iter().any(|(test, _, _)| test == header)
                    && !blocks.contains(&header)
                {
                    blocks.insert(0, header);
                }
                blocks
            }
            Self::Fallback { blocks, .. } => blocks.iter().collect(),
            Self::LoopBreak { .. } => Vec::new(),
            Self::LoopContinue { .. } => Vec::new(),
            Self::Guard { prefix, plan } => {
                let mut blocks: Vec<&CanonicalBlockId> = prefix.iter().collect();
                blocks.extend(plan.owned());
                blocks
            }
            Self::Try {
                prefix,
                body,
                catches,
                ..
            } => {
                let mut blocks: Vec<&CanonicalBlockId> = prefix.iter().collect();
                blocks.extend(body.blocks());
                for clause in catches {
                    blocks.extend(clause.body.blocks());
                }
                blocks
            }
        }
    }

    /// Whether this region is presented as Java structure rather than as quoted bytecode.
    pub fn is_structured(&self) -> bool {
        match self {
            Self::Sequence { regions } => regions.iter().all(Self::is_structured),
            Self::Straight { .. } => true,
            Self::If {
                then_arm, else_arm, ..
            } => then_arm.is_structured() && else_arm.is_structured(),
            Self::ShortCircuitValue { reason, .. } => {
                !matches!(reason, FallbackReason::ExceptionEdge { .. })
            }
            Self::TwoExitReturn { .. } => true,
            Self::Switch { groups, .. } | Self::StringSwitch { groups, .. } => {
                groups.iter().all(|group| group.arm.is_structured())
            }
            Self::Loop { body, .. } => body.iter().all(Region::is_structured),
            // A guarded statement is presented when its rule proved it, and every link of that
            // proof is stated by the rule itself: there is no *nested* region inside it that could
            // have been quoted instead, because a body this rule cannot present is refused whole.
            Self::Guard { .. } => true,
            // A `try` is presented when the protected range and every handler body are: a clause
            // whose body is a quote keeps its header and states the quote inside it, which is a
            // weaker presentation and not a structured one.
            Self::Try { body, catches, .. } => {
                body.is_structured() && catches.iter().all(|clause| clause.body.is_structured())
            }
            Self::Fallback { .. } => false,
            Self::LoopBreak { .. } => true,
            Self::LoopContinue { .. } => true,
        }
    }

    /// Every fallback this region holds, in method order.
    pub fn fallbacks(&self) -> Vec<FallbackReason> {
        match self {
            Self::Sequence { regions } => regions.iter().flat_map(Self::fallbacks).collect(),
            Self::Straight { .. } => Vec::new(),
            Self::If {
                then_arm, else_arm, ..
            } => {
                let mut reasons = then_arm.fallbacks();
                reasons.extend(else_arm.fallbacks());
                reasons
            }
            Self::ShortCircuitValue { reason, .. } => match reason {
                FallbackReason::ExceptionEdge { .. } => vec![reason.clone()],
                _ => Vec::new(),
            },
            Self::TwoExitReturn { .. } => Vec::new(),
            Self::Switch { groups, .. } | Self::StringSwitch { groups, .. } => groups
                .iter()
                .flat_map(|group| group.arm.fallbacks())
                .collect(),
            Self::Loop { body, .. } => body.iter().flat_map(Region::fallbacks).collect(),
            Self::Guard { .. } => Vec::new(),
            Self::Try { body, catches, .. } => {
                let mut reasons = body.fallbacks();
                for clause in catches {
                    reasons.extend(clause.body.fallbacks());
                }
                reasons
            }
            Self::Fallback { reason, .. } => vec![reason.clone()],
            Self::LoopBreak { .. } => Vec::new(),
            Self::LoopContinue { .. } => Vec::new(),
        }
    }

    /// The declared rule whose pass produced this region, when a registered rule did.
    ///
    /// This is the traceability P3 decision 1 asks for: every recorded region says which rule, and
    /// which version of it, the shape came from — and a fallback says which rule *refused* it. A
    /// region no rule is answerable for (the whole-body refusals the walk itself states, such as an
    /// irreducible graph) states `None` rather than borrowing a rule's name.
    pub fn rule(&self) -> Option<crate::pass::RuleVersion> {
        match self {
            Self::Sequence { .. } => None,
            Self::Straight { .. } => Some(crate::pass::STRAIGHT.rule()),
            Self::If { .. } => Some(crate::pass::IF.rule()),
            Self::ShortCircuitValue { .. } => None,
            Self::TwoExitReturn { .. } => None,
            Self::Switch { .. } | Self::StringSwitch { .. } => Some(crate::pass::SWITCH.rule()),
            Self::Loop { .. } => Some(crate::pass::LOOP.rule()),
            Self::LoopBreak { .. } => Some(crate::pass::LOOP.rule()),
            Self::LoopContinue { .. } => Some(crate::pass::LOOP.rule()),
            Self::Guard { plan, .. } => Some(plan.pass().rule()),
            // No pass of this build claims a `try`/`catch`: the statement is the exception table's
            // own structure, read here and written by the builder, and naming a rule for it would
            // claim a proof that does not exist.
            Self::Try { .. } => None,
            Self::Fallback { reason, .. } => reason.pass().map(|pass| pass.rule()),
        }
    }
}

fn sequence_region(regions: Vec<Region>) -> Region {
    match regions.len() {
        0 => Region::Straight { blocks: Vec::new() },
        1 => regions.into_iter().next().expect("one region"),
        _ => Region::Sequence { regions },
    }
}

fn same_nodes(left: &[usize], right: &[usize]) -> bool {
    left.len() == right.len() && left.iter().all(|node| right.contains(node))
}

/// The regions of one method, with what the walk claimed and what it could not.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recovered {
    /// The regions in method order; the run continues after each one at the next entry of this
    /// list, which is how an `if` whose arms meet is followed by the join's own region.
    pub regions: Vec<Region>,
    /// Every block the walk claimed, by index into the canonical graph.
    pub claimed: BTreeSet<usize>,
    /// Every block of the canonical graph, live or not.
    pub blocks: usize,
}

impl Recovered {
    /// Whether every region the method has can be presented as Java structure.
    pub fn is_structured(&self) -> bool {
        self.regions.iter().all(Region::is_structured)
    }

    /// Every fallback reason the regions hold, in region order.
    pub fn fallbacks(&self) -> Vec<FallbackReason> {
        self.regions.iter().flat_map(Region::fallbacks).collect()
    }

    /// Every rule that produced a region of this method, each once, in the order it first did.
    ///
    /// The rules a method's output came from — reported beside the profile, so that "which rule
    /// produced this text" is answered by the run's own record rather than by re-reading it.
    pub fn rules(&self) -> Vec<crate::pass::RuleVersion> {
        let mut rules: Vec<crate::pass::RuleVersion> = Vec::new();
        for region in &self.regions {
            if let Some(rule) = region.rule()
                && !rules.contains(&rule)
            {
                rules.push(rule);
            }
        }
        rules
    }
}

/// Publish a two-dispatch String structure only after both ordinary regions and the complete
/// same-method certificate exist. A refusal does not modify either original region.
pub(crate) fn project_string_switches(
    recovered: &mut Recovered,
    ir: &MethodIr,
    budget: &mut Budget,
) -> Result<(), StopReason> {
    fn visit(region: &mut Region, ir: &MethodIr, budget: &mut Budget) -> Result<(), StopReason> {
        match region {
            Region::Sequence { regions } | Region::Loop { body: regions, .. } => {
                project_sequence(regions, ir, budget)?;
            }
            Region::If {
                then_arm, else_arm, ..
            } => {
                visit(then_arm, ir, budget)?;
                visit(else_arm, ir, budget)?;
            }
            Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => {
                for group in groups {
                    visit(&mut group.arm, ir, budget)?;
                }
            }
            Region::Try { body, catches, .. } => {
                visit(body, ir, budget)?;
                for clause in catches {
                    visit(&mut clause.body, ir, budget)?;
                }
            }
            Region::ShortCircuitValue { .. } | Region::TwoExitReturn { .. } => {}
            Region::Straight { .. }
            | Region::Fallback { .. }
            | Region::Guard { .. }
            | Region::LoopBreak { .. }
            | Region::LoopContinue { .. } => {}
        }
        Ok(())
    }

    fn project_sequence(
        regions: &mut Vec<Region>,
        ir: &MethodIr,
        budget: &mut Budget,
    ) -> Result<(), StopReason> {
        for region in regions.iter_mut() {
            visit(region, ir, budget)?;
        }
        let mut index = 0;
        while index + 1 < regions.len() {
            poll(budget, None)?;
            charge(budget, CountedBudgetDimension::AnalysisSteps, 1, None)?;
            if let Some(projected) =
                string_switch_pair(&regions[index], &regions[index + 1], ir, budget)?
            {
                regions.splice(index..index + 2, [projected]);
            } else {
                index += 1;
            }
        }
        Ok(())
    }

    project_sequence(&mut recovered.regions, ir, budget)
}

fn string_switch_pair(
    first: &Region,
    second: &Region,
    ir: &MethodIr,
    budget: &mut Budget,
) -> Result<Option<Region>, StopReason> {
    let (
        Region::Switch {
            branch_bci: hash_bci,
            join: hash_join,
            ..
        },
        Region::Switch {
            prefix,
            branch,
            branch_bci: final_bci,
            groups,
            join,
        },
    ) = (first, second)
    else {
        return Ok(None);
    };
    if !prefix.is_empty()
        || hash_join.as_ref() != Some(branch)
        || !first.is_structured()
        || !second.is_structured()
    {
        return Ok(None);
    }
    let Some(proof) = crate::stringswitch::prove(ir, *hash_bci, *final_bci, budget)? else {
        return Ok(None);
    };
    let (Some(ssa), Some(code)) = (ir.ssa(), ir.code()) else {
        return Ok(None);
    };
    let mut dispatch: Vec<CanonicalBlockId> = first.blocks().into_iter().cloned().collect();
    if !dispatch.contains(branch) {
        dispatch.push(branch.clone());
    }
    let dispatch_set: BTreeSet<_> = dispatch.iter().cloned().collect();
    // The region structure must own precisely the removable interval. A certificate for a
    // different shape is not authority to hide extra instructions in these blocks.
    let actual: BTreeSet<u32> = ssa
        .blocks()
        .iter()
        .filter(|block| dispatch_set.contains(block.block()))
        .flat_map(|block| block.instructions())
        .map(|instruction| instruction.bci())
        .filter(|bci| *bci >= proof.selector_store_bci && *bci <= proof.final_switch_bci)
        .collect();
    if actual != proof.owned_bcis
        || proof.owned_bcis.iter().any(|bci| {
            !code
                .instructions
                .iter()
                .any(|instruction| instruction.bci == *bci)
        })
    {
        return Ok(None);
    }
    let mut mapped = BTreeSet::new();
    let mut defaults = 0;
    for group in groups {
        defaults += usize::from(group.default);
        for key in &group.keys {
            if !mapped.insert(*key) {
                return Ok(None);
            }
            let has_label = proof.labels.iter().any(|(_, label_key)| label_key == key);
            if !has_label && (!group.default || !proof.default_holes.contains(key)) {
                return Ok(None);
            }
        }
    }
    if defaults != 1
        || proof.labels.iter().any(|(_, key)| !mapped.contains(key))
        || proof.default_holes.iter().any(|key| !mapped.contains(key))
    {
        return Ok(None);
    }
    Ok(Some(Region::StringSwitch {
        dispatch,
        proof,
        groups: groups.clone(),
        join: join.clone(),
    }))
}

/// Recovers the region tree of one method from the projection, the decoded operations and the
/// decode facts the same read produced.
///
/// The decode itself travels in `code` rather than piece by piece because the walk reads two
/// things out of it — the declared exception table ([`MethodCodeFacts::exception_handlers`]) and the
/// **instructions** it decoded — and both have to be the same read's: the whole-body check below
/// asks whether the graph accounts for exactly those instructions, which is a question no table
/// rebuilt from a report could answer.
pub(crate) fn recover(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    operations: &Operations,
    sites: &crate::init::Sites,
    code: &MethodCodeFacts,
    method_synchronized: Option<bool>,
    // Whether the method's descriptor states a `boolean` result — the fact the
    // two-terminal-return claim needs before it may commit ownership.
    return_is_boolean: bool,
    profile: &crate::pass::RecoveryProfile,
    budget: &mut Budget,
) -> Result<Recovered, StopReason> {
    let live: Vec<CanonicalBlockId> = canonical
        .blocks()
        .iter()
        .map(|block| block.id().clone())
        .filter(|id| !canonical.unreachable().contains(id))
        .collect();
    // Two facts about the graph refuse the *whole* body before the walk starts, and both are
    // shapes nothing Java could write: a graph that is not reducible over some block (a loop
    // entered twice, or two loops crossing) and an exception table whose records cross. Quoting
    // every live block is the honest answer — presenting the reducible part around a cycle the
    // reader cannot see would hide exactly the fact that made it unprovable. A third fact of the
    // same kind is asked of the decode, and it is answered at the end of this function, because
    // what the body has to become then depends on what the walk found.
    let catch_joins = proved_loop_catch_joins(canonical, view, code, budget)?;
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(canonical.edges().len()).unwrap_or(u64::MAX),
        None,
    )?;
    let excluded_edge_nodes: BTreeSet<_> = canonical
        .edges()
        .iter()
        .filter(|edge| {
            matches!(
                edge.kind(),
                CanonicalEdgeKind::Exception { .. } | CanonicalEdgeKind::Call { .. }
            )
        })
        .flat_map(|edge| [edge.from().clone(), edge.to().clone()])
        .collect();
    let irreducible = view.irreducible_blocks(&catch_joins);
    if !irreducible.is_empty() {
        let bcis = irreducible
            .iter()
            .filter_map(|node| view.id_of(*node).map(CanonicalBlockId::bci))
            .collect();
        return Ok(quoted_whole(
            live,
            FallbackReason::Irreducible { blocks: bcis },
            canonical,
        ));
    }
    if let Some((record, other)) = crossing_records(&code.exception_handlers, canonical) {
        return Ok(quoted_whole(
            live,
            FallbackReason::CrossingExceptionRegions {
                record,
                other,
                blocks: canonical
                    .handler_rows()
                    .iter()
                    .filter(|row| row.ordinal() == record || row.ordinal() == other)
                    .flat_map(|row| row.protected().iter().map(CanonicalBlockId::bci))
                    .collect(),
            },
            canonical,
        ));
    }
    let boundary_return_bci = if method_synchronized == Some(false) {
        code.exception_handlers
            .iter()
            .filter(|row| row.catch_type_index.is_some())
            .find_map(|row| {
                canonical.blocks().iter().find_map(|span| {
                    (row.start_bci < span.end_bci() && span.id().bci() < row.end_bci)
                        .then(|| ssa.block(span.id()))
                        .flatten()
                        .and_then(|names| names.instructions().last())
                        .filter(|last| {
                            last.bci() == row.end_bci
                                && operations.get(last.bci()) == Some(&Operation::Return)
                        })
                        .map(|last| last.bci())
                })
            })
    } else {
        None
    };
    let has_reachable_explicit_monitor = if let Some(at) = boundary_return_bci {
        poll(budget, Some(at))?;
        let scan_items = ssa
            .blocks()
            .iter()
            .map(|block| block.instructions().len().saturating_add(1))
            .sum::<usize>();
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(scan_items).unwrap_or(u64::MAX),
            Some(at),
        )?;
        let mut found = false;
        for entry in ssa.blocks() {
            poll(budget, Some(at))?;
            if canonical.unreachable().contains(entry.block()) {
                continue;
            }
            found |= entry.instructions().iter().any(|instruction| {
                matches!(
                    operations.get(instruction.bci()),
                    Some(Operation::Monitor { .. })
                )
            });
        }
        found
    } else {
        false
    };
    // P3-R7's own check runs *after* the walk, because what the body has to become depends on what
    // the walk was going to say about it (see the end of this function).
    let mut walker = Walker {
        canonical,
        view,
        ssa,
        operations,
        sites,
        code,
        method_synchronized,
        has_reachable_explicit_monitor,
        return_is_boolean,
        handlers: &code.exception_handlers,
        profile,
        budget,
        catch_joins,
        excluded_edge_nodes,
        visited: BTreeSet::new(),
        depth: 0,
    };
    let mut regions = Vec::new();
    // The canonical graph publishes the method entry first: the normalization creates a node for
    // the entry and for every call site it enters, so index 0 is where the body starts.
    let mut current = canonical.blocks().first().map(|block| block.id().clone());
    if current.is_some() && !canonical.unreachable().is_empty() {
        let entry = canonical.blocks()[0].id().clone();
        if canonical.unreachable().contains(&entry) {
            current = None;
        }
    }
    while let Some(node) = current {
        // One walk call may prove two regions: the blocks it proved before a gap and the quote the
        // gap owes ([`Run`]). Both are regions of this method, in that order, and the run continues
        // at the join the gap proved — or stops, with the live blocks it left behind named by the
        // uncovered-blocks scan below.
        let (run, next) = walker.region_at(&node, &Frame::default())?;
        regions.extend(run);
        current = next;
    }
    if std::env::var_os("JRE_PREFIX_PROBE").is_some() {
        eprintln!(
            "P3VISITED blocks={:?} visited={:?}",
            canonical
                .blocks()
                .iter()
                .map(|block| block.id().bci())
                .collect::<Vec<u32>>(),
            walker
                .visited
                .iter()
                .filter_map(|node| canonical.blocks().get(*node).map(|block| block.id().bci()))
                .collect::<Vec<u32>>()
        );
    }
    if std::env::var_os("JRE_PREFIX_PROBE").is_some() {
        let held: BTreeSet<usize> = regions
            .iter()
            .flat_map(Region::blocks)
            .filter_map(|block| walker.view.index_of(block))
            .collect();
        let lost: Vec<u32> = walker
            .visited
            .iter()
            .filter(|node| !held.contains(node))
            .filter_map(|node| canonical.blocks().get(*node).map(|block| block.id().bci()))
            .collect();
        if !lost.is_empty() {
            eprintln!(
                "P3LOST blocks={lost:?} regions={:?}",
                regions
                    .iter()
                    .map(|region| region
                        .blocks()
                        .iter()
                        .map(|block| block.bci())
                        .collect::<Vec<u32>>())
                    .collect::<Vec<Vec<u32>>>()
            );
        }
    }
    // A block a returned quote already names is accounted for: a refused shape's quote (`gap`)
    // commits no ownership, so without this set the scan would name the refused header — and
    // every block behind it — a second time, and two quotes owning one block would fail the
    // ownership check below as a phantom overlap.
    let quoted: BTreeSet<&CanonicalBlockId> = regions.iter().flat_map(Region::blocks).collect();
    let uncovered: Vec<CanonicalBlockId> = canonical
        .blocks()
        .iter()
        .map(|block| block.id().clone())
        .filter(|id| !canonical.unreachable().contains(id))
        .filter(|id| !quoted.contains(id))
        .filter(|id| {
            !walker
                .view
                .index_of(id)
                .is_some_and(|node| walker.visited.contains(&node))
        })
        .collect();
    if !uncovered.is_empty() {
        // Stated, never dropped: a live block the walk could not claim (a handler body, a
        // subroutine clone) becomes a region of its own whose text quotes the bytecode.
        let bcis = uncovered.iter().map(|id| id.bci()).collect();
        regions.push(Region::Fallback {
            blocks: uncovered,
            reason: FallbackReason::UncoveredBlocks { blocks: bcis },
        });
    }
    // The walk's visited set prevents endless traversal, but an already claimed join can still
    // appear in several returned regions. Those local decisions cannot jointly describe a Java
    // body. Refuse the completed tree before the builder sees any of it; keep P3-R7's independent
    // instruction check below so bytes missing from the canonical graph remain named too.
    if let Some(block) = overlapping_owner(&regions, walker.budget)? {
        regions = quoted_whole(
            live.clone(),
            FallbackReason::OwnershipOverlap { block },
            canonical,
        )
        .regions;
        walker.visited.clear();
    }
    // P3-R7, last: the graph has to be an account of the body it stands for **before** any region of
    // it is presented as that body. Every instruction the same read decoded is either covered by a
    // canonical block or named as unreachable; an instruction that is in neither is one the
    // normalization never walked and never stated, so no region of this walk covers it and no quote
    // of the regions would name it either — the ECJ 4.6.1 v52 `finallyPath` is the committed case
    // (its `Exception table` names a handler at BCI 9, the decode reads BCIs 9/10/13/14, and the
    // graph's only node is `[0, 9)` because nothing in `[0, 4)` can throw synchronously).
    //
    // What the body has to become depends on what this walk was going to say about it, and in both
    // cases the artifact ends up **naming** those instructions:
    //
    // * every region is structured: this walk was about to present the body whole, which is the
    //   claim of completeness the graph cannot support. The body is quoted whole, so the text of a
    //   body the graph has no account of is never the body that looks complete;
    // * the walk already refused something: it claims no completeness already, and the refusals it
    //   found are rules that examined a shape and said why (a guarded region's unproven close, a
    //   `jsr` body). Those reasons stay, and the unaccounted instructions are stated beside them as
    //   a quote of their own — replacing a specific reason with "the graph has a hole" would tell a
    //   reader less, and dropping the bytes would be the silence this check exists to undo.
    let unaccounted = unaccounted_instructions(code, canonical);
    if !unaccounted.is_empty() {
        if regions.iter().all(Region::is_structured) {
            return Ok(quoted_whole(
                live,
                FallbackReason::UnaccountedInstructions { bcis: unaccounted },
                canonical,
            ));
        }
        // `blocks` is empty on purpose: these instruction starts belong to **no** node of the graph,
        // which is the whole point of the reason, so the quote is the reason's own list.
        regions.push(Region::Fallback {
            blocks: Vec::new(),
            reason: FallbackReason::UnaccountedInstructions { bcis: unaccounted },
        });
    }
    Ok(Recovered {
        regions,
        claimed: walker.visited,
        blocks: canonical.blocks().len(),
    })
}

/// Find the first repeated physical owner in method order. `Region::blocks` already folds
/// intentional aliases *within* one shape (a loop header that is its test, or a short-circuit
/// producer named by several edges). The identity includes the `jsr` path, so clones at one BCI
/// remain separate owners. Charge each region before flattening it, then its references before
/// checking them; a stop returns no partly validated tree.
fn overlapping_owner(
    regions: &[Region],
    budget: &mut Budget,
) -> Result<Option<CanonicalBlockId>, StopReason> {
    let mut seen = BTreeSet::new();
    for region in regions {
        poll(budget, None)?;
        charge(budget, CountedBudgetDimension::AnalysisSteps, 1, None)?;
        let blocks = region.blocks();
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(blocks.len()).unwrap_or(u64::MAX),
            blocks.first().map(|block| block.bci()),
        )?;
        for block in blocks {
            poll(budget, Some(block.bci()))?;
            if !seen.insert(block) {
                return Ok(Some(block.clone()));
            }
        }
    }
    Ok(None)
}

/// A handler reached only through its stated exception row is not a second ordinary entry when
/// its protected code belongs to one loop and both paths meet again inside that loop. Keep the
/// ordinary edge in the view: the `try` walk still has to claim the handler and the join.
fn proved_loop_catch_joins(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    code: &MethodCodeFacts,
    budget: &mut Budget,
) -> Result<BTreeSet<(usize, usize)>, StopReason> {
    let mut joins = BTreeSet::new();
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(canonical.handler_rows().len()).unwrap_or(u64::MAX),
        None,
    )?;
    let handlers: BTreeSet<usize> = canonical
        .handler_rows()
        .iter()
        .filter_map(|row| row.handler().and_then(|id| view.index_of(id)))
        .collect();
    for handler in handlers {
        let at = view.id_of(handler).map(CanonicalBlockId::bci);
        poll(budget, at)?;
        charge(budget, CountedBudgetDimension::AnalysisSteps, 1, at)?;
        let successors = view.successors(handler);
        let [join] = successors.as_slice() else {
            continue;
        };
        // A second ordinary predecessor means this is no longer an exception-root handler.
        if !view.predecessors(handler).is_empty() {
            continue;
        }
        let rows: Vec<_> = canonical
            .handler_rows()
            .iter()
            .filter(|row| row.handler().and_then(|id| view.index_of(id)) == Some(handler))
            .collect();
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(code.exception_handlers.len()).unwrap_or(u64::MAX),
            at,
        )?;
        // Every declared row targeting this physical entry must have a matching canonical row
        // under this path. An unmapped row cannot silently grant the mapped one an exemption.
        if code.exception_handlers.iter().any(|declared| {
            Some(declared.handler_bci) == at
                && !rows.iter().any(|row| row.ordinal() == declared.ordinal)
        }) {
            continue;
        }
        if rows
            .iter()
            .any(|row| row.catch_type_index().is_none() || row.protected().is_empty())
        {
            continue;
        }
        for header in 0..view.len() {
            let Some(loop_of) = view.loop_entered_at(header) else {
                continue;
            };
            if *join == header || !loop_of.blocks().contains(join) {
                continue;
            }
            let mut compatible = true;
            for row in &rows {
                charge(
                    budget,
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(canonical.blocks().len()).unwrap_or(u64::MAX),
                    at,
                )?;
                let Some(declared) = code
                    .exception_handlers
                    .iter()
                    .find(|declared| declared.ordinal == row.ordinal())
                else {
                    compatible = false;
                    break;
                };
                // `protected()` may list only actual throw-site blocks. The table's full range
                // must also stay inside the loop body; a row covering the loop test or a block
                // outside this loop cannot borrow a sibling row's valid handler entry.
                for block in canonical.blocks().iter().filter(|block| {
                    block.id().bci() < declared.end_bci && block.end_bci() > declared.start_bci
                }) {
                    let Some(node) = view.index_of(block.id()) else {
                        compatible = false;
                        break;
                    };
                    if node == header
                        || !loop_of.blocks().contains(&node)
                        || !view.dominates(header, node)
                    {
                        compatible = false;
                        break;
                    }
                }
                if !compatible {
                    break;
                }
                for protected in row.protected() {
                    let Some(node) = view.index_of(protected) else {
                        compatible = false;
                        break;
                    };
                    if !loop_of.blocks().contains(&node)
                        || !view.dominates(header, node)
                        || !plain_paths_reach_join(view, node, *join, budget)?
                    {
                        compatible = false;
                        break;
                    }
                }
                if !compatible {
                    break;
                }
            }
            if compatible {
                joins.insert((handler, *join));
            }
        }
    }
    Ok(joins)
}

/// Every ordinary path must reach the join before a terminal or a cycle. A cycle that can avoid
/// the join is not a proved rejoin, even if another route reaches it.
fn plain_paths_reach_join(
    view: &NormalFlowView,
    start: usize,
    join: usize,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    let mut state = BTreeMap::new();
    let mut pending = vec![(start, false)];
    while let Some((node, finished)) = pending.pop() {
        let at = view.id_of(node).map(CanonicalBlockId::bci);
        charge(budget, CountedBudgetDimension::AnalysisSteps, 1, at)?;
        if node == join {
            continue;
        }
        if finished {
            state.insert(node, 2u8);
            continue;
        }
        match state.get(&node) {
            Some(1) => return Ok(false),
            Some(2) => continue,
            _ => {}
        }
        let successors = view.successors(node);
        if successors.is_empty() {
            return Ok(false);
        }
        state.insert(node, 1u8);
        pending.push((node, true));
        pending.extend(
            successors
                .into_iter()
                .rev()
                .map(|successor| (successor, false)),
        );
    }
    Ok(true)
}

/// One whole-body fallback: every live block quoted, under one reason.
fn quoted_whole(
    live: Vec<CanonicalBlockId>,
    reason: FallbackReason,
    canonical: &CanonicalCfg,
) -> Recovered {
    Recovered {
        regions: vec![Region::Fallback {
            blocks: live,
            reason,
        }],
        claimed: BTreeSet::new(),
        blocks: canonical.blocks().len(),
    }
}

/// The instruction starts one decode produced that the canonical graph does not account for (P3-R7).
///
/// The question is asked of the **decoded instruction starts**, not of the operations, the frames or
/// the names: the decode is the most basic statement of what the body is, and a graph that disagrees
/// with it about which instructions exist is a graph no presentation may call the body. An
/// instruction is accounted for when
///
/// * some block of the graph covers it — its BCI lies in the half-open range a node spans, from the
///   first original block start it stands for (`CanonicalBlock::blocks`) to its `end_bci` — or
/// * the graph lists the node that holds it as dead ([`CanonicalCfg::unreachable`]), which is a
///   statement about it and not a silence: a dead instruction is a fact of this body with a reason
///   the run can explain, and the ECJ 4.6.1 v45 `finallyPath`'s three dead nodes are that shape.
///
/// Neither is the same as **absent**: a BCI in no range and in no dead node is an instruction the
/// normalization never walked and never stated, which is the third state [`CanonicalCfg::unreachable`]
/// warns about. This walk was written to refuse such a body rather than present the part of it the
/// graph happens to hold.
///
/// The list is ascending — the decode's own order — and deduplicated by construction: one entry per
/// instruction start of the body.
fn unaccounted_instructions(code: &MethodCodeFacts, canonical: &CanonicalCfg) -> Vec<u32> {
    let mut covered = BTreeSet::new();
    for block in canonical.blocks() {
        let start = block
            .blocks()
            .first()
            .copied()
            .unwrap_or_else(|| block.id().bci());
        covered.extend(start..block.end_bci());
    }
    // A node the entry cannot reach is accounted for by the dead list as well, so that a graph which
    // published a dead node *outside* the range its id names — the shape a consumer must not read as
    // reachable — still accounts for it. Both lists are read together; neither alone is a partition.
    for id in canonical.unreachable() {
        covered.insert(id.bci());
        if let Some(block) = canonical.blocks().iter().find(|block| block.id() == id) {
            let start = block.blocks().first().copied().unwrap_or_else(|| id.bci());
            covered.extend(start..block.end_bci());
        }
    }
    code.instructions
        .iter()
        .map(|instruction| instruction.bci)
        .filter(|bci| !covered.contains(bci))
        .collect()
}

/// The first pair of exception-table records whose declared ranges cross, when the graph holds a
/// handler row for at least one of them.
///
/// Ranges cross when they overlap without either containing the other: `[0, 8)` and `[4, 12)` do,
/// while the same range twice (two `catch` clauses of one `try`) and a nested range do not. A pair
/// that protects no block of this graph is not a crossing this run has to refuse — the records are
/// still facts of the class file, but nothing in this body's shape depends on their order.
fn crossing_records(
    handlers: &[ExceptionHandlerFact],
    canonical: &CanonicalCfg,
) -> Option<(u32, u32)> {
    let rows: Vec<u32> = canonical
        .handler_rows()
        .iter()
        .map(|row| row.ordinal())
        .collect();
    for (index, first) in handlers.iter().enumerate() {
        for second in &handlers[index + 1..] {
            let crosses = (first.start_bci < second.start_bci
                && second.start_bci < first.end_bci
                && first.end_bci < second.end_bci)
                || (second.start_bci < first.start_bci
                    && first.start_bci < second.end_bci
                    && second.end_bci < first.end_bci);
            if crosses && (rows.contains(&first.ordinal) || rows.contains(&second.ordinal)) {
                return Some((first.ordinal, second.ordinal));
            }
        }
    }
    None
}

/// Where one region walk may go, and where it ends.
///
/// A frame is how nesting is expressed without the walker keeping a stack of its own: an `if`'s arm
/// ends at its join, and a loop's body ends at the loop's own test and may not leave the blocks that
/// iterate at all.
#[derive(Clone, Debug, Default)]
struct Frame {
    /// The node the region ends at: arriving there (as a successor) ends the run, and the block
    /// itself belongs to the structure that follows.
    boundary: Option<usize>,
    /// The nodes the region may claim, when it is a loop's body. A node outside the scope ends the
    /// run exactly like the boundary does — it is the code after the loop.
    scope: Option<BTreeSet<usize>>,
    /// The header of the loop this frame is the body of. A body walk that arrives back at its own
    /// header is inside the structure it is building, which is a state it has already entered — not
    /// a nested loop — and the walk must not read it as one (see [`Walker::region_at`]).
    own_loop: Option<usize>,
    /// The block of the `try` this frame is the protected range of. The range begins at that block,
    /// so the walk that recovers it starts at it, and reading it as the start of *another* `try`
    /// would be reading the statement it is already building (see [`Walker::try_region`]).
    own_try: Option<usize>,
    /// Other case-entry nodes of a switch arm. They end this arm before the next case claims them.
    case_entries: Option<BTreeSet<usize>>,
    /// The exact normal-flow target of a `break` from the loop whose body this frame walks.
    loop_exit: Option<usize>,
    /// Proven transfer destinations of enclosing loops, outermost first.
    loop_targets: Vec<LoopTarget>,
    /// Source instruction of an incoming branch/switch edge, when this arm is a transfer leaf.
    transfer_source_bci: Option<u32>,
    /// A switch's own join must remain a switch break instead of becoming a loop break.
    switch_join: Option<usize>,
}

#[derive(Clone, Debug)]
struct LoopTarget {
    header: usize,
    exits: BTreeSet<usize>,
    break_target: Option<usize>,
    continue_target: usize,
}

struct HeaderTestChain {
    tests: Vec<(CanonicalBlockId, u32, Continuation)>,
    operator: crate::ast::BinaryOp,
    body: CanonicalBlockId,
    exit: CanonicalBlockId,
}

type HeaderTestFacts = (u32, u32, Vec<CanonicalBlockId>);

impl Frame {
    /// The frame of one loop body: the blocks that iterate, ending where the loop tests.
    fn loop_body(
        &self,
        blocks: &BTreeSet<usize>,
        boundary: usize,
        header: usize,
        continue_target: usize,
        exit: Option<usize>,
        exits: BTreeSet<usize>,
        transfer_sources: &BTreeSet<usize>,
    ) -> Self {
        // A loop inside a loop may not claim a block the enclosing loop's body does not hold: the
        // scope of a body is the intersection, so a nesting cannot widen it.
        let mut scope = match &self.scope {
            Some(outer) => blocks.intersection(outer).copied().collect(),
            None => blocks.clone(),
        };
        // A transfer instruction is part of the loop body even when its one outgoing edge makes
        // its block absent from the natural-loop set. Admit only single-successor predecessors of
        // an exact enclosing-loop exit/continue target; the walker will still verify that edge.
        scope.extend(transfer_sources.iter().copied());
        Self {
            boundary: Some(boundary),
            scope: Some(scope),
            own_loop: Some(header),
            own_try: None,
            case_entries: self.case_entries.clone(),
            loop_exit: exit,
            loop_targets: {
                let mut targets = self.loop_targets.clone();
                targets.push(LoopTarget {
                    header,
                    exits,
                    break_target: exit,
                    continue_target,
                });
                targets
            },
            transfer_source_bci: None,
            switch_join: self.switch_join,
        }
    }

    /// The frame of one arm of a branch: everything the enclosing frame allowed, ending at the join.
    ///
    /// The arm keeps its enclosing loop identity so transfer edges reached through a branch are
    /// attributed to the same loop. Entering a nested loop replaces that identity in its body frame.
    fn arm(&self, join: Option<usize>, source_bci: Option<u32>) -> Self {
        Self {
            // A branch with no normal-flow join still stays inside its enclosing region. This is
            // needed when one arm exits through a caught exception and the other reaches the try's
            // join: the normal-flow graph alone has no post-dominator for that branch.
            boundary: join.or(self.boundary),
            scope: self.scope.clone(),
            own_loop: None,
            // An `if` inside a protected range is still inside that range in either arm. Keep
            // its owner so a throwing arm's exception edges can be matched to this try's catches.
            own_try: self.own_try,
            case_entries: self.case_entries.clone(),
            loop_exit: self.loop_exit,
            loop_targets: self.loop_targets.clone(),
            transfer_source_bci: source_bci.or(self.transfer_source_bci),
            switch_join: self.switch_join,
        }
    }

    /// One switch arm, bounded by the other proven case entries as well as its enclosing join.
    fn switch_arm(&self, join: Option<usize>, entries: &BTreeSet<usize>, source_bci: u32) -> Self {
        let mut case_entries = self.case_entries.clone().unwrap_or_default();
        case_entries.extend(entries.iter().copied());
        Self {
            boundary: join.or(self.boundary),
            scope: self.scope.clone(),
            own_loop: None,
            own_try: self.own_try,
            case_entries: Some(case_entries),
            loop_exit: self.loop_exit,
            loop_targets: self.loop_targets.clone(),
            transfer_source_bci: Some(source_bci),
            switch_join: join.or(self.switch_join),
        }
    }

    /// The frame of one `try`'s protected range: it ends where the code after the `try` begins.
    ///
    /// `start` is the node the statement's own range begins at, which is the node this walk is
    /// entering: the range is *inside* the statement, so it may not present the statement again.
    /// The enclosing frame's scope is kept — what a loop body may claim does not widen under a `try`
    /// — exactly as an arm keeps it.
    fn protected(&self, join: Option<usize>, start: usize) -> Self {
        Self {
            boundary: join,
            scope: self.scope.clone(),
            own_loop: None,
            own_try: Some(start),
            case_entries: self.case_entries.clone(),
            loop_exit: self.loop_exit,
            loop_targets: self.loop_targets.clone(),
            transfer_source_bci: self.transfer_source_bci,
            switch_join: self.switch_join,
        }
    }

    /// Whether a walk must stop at one node: the node it ends at, or one the scope does not hold.
    fn stops_at(&self, node: usize) -> bool {
        self.boundary == Some(node)
            || self
                .case_entries
                .as_ref()
                .is_some_and(|entries| entries.contains(&node))
            || self
                .scope
                .as_ref()
                .is_some_and(|scope| !scope.contains(&node))
    }

    fn stops_at_switch_boundary(&self, node: usize) -> bool {
        self.switch_join == Some(node)
            || self
                .case_entries
                .as_ref()
                .is_some_and(|entries| entries.contains(&node))
    }
}

struct Walker<'a> {
    canonical: &'a CanonicalCfg,
    view: &'a NormalFlowView,
    ssa: &'a SsaTable,
    operations: &'a Operations,
    sites: &'a crate::init::Sites,
    code: &'a MethodCodeFacts,
    /// Whether the declaring method is synchronized, as the same recovery request states it. An
    /// absent flag is not evidence that the JVM's implicit method monitor is absent.
    method_synchronized: Option<bool>,
    /// Whether a reachable instruction anywhere in this method enters or leaves an explicit
    /// monitor. Computed once, only when a range-end return candidate could use the exception.
    has_reachable_explicit_monitor: bool,
    /// Whether the method's own descriptor states a `boolean` result. The two-terminal-return
    /// claim is sound only there: its builder re-checks the descriptor first, so a claim made
    /// anywhere else is ownership committed for a proof that cannot run.
    return_is_boolean: bool,
    /// The exception table the same decode stated: the guarded rules of P3 2.4 read the ranges and
    /// the catch types from it, and the walk reads it for the crossing-range refusal.
    handlers: &'a [ExceptionHandlerFact],
    /// The profile the run is presented under: `twr@1`'s output is Java 7 syntax, so a profile that
    /// presents the artifact as an older release does not admit it ([`crate::pass::Pass::admits`]).
    profile: &'a crate::pass::RecoveryProfile,
    budget: &'a mut Budget,
    catch_joins: BTreeSet<(usize, usize)>,
    /// Endpoints of exception and subroutine edges, computed once so bounded local shape probes do
    /// not rescan the whole canonical edge table.
    excluded_edge_nodes: BTreeSet<CanonicalBlockId>,
    visited: BTreeSet<usize>,
    /// How many [`Walker::region_at`] calls are on the stack: the region walk's own recursion
    /// depth, checked against [`MAX_REGION_DEPTH`] before the walk descends.
    depth: usize,
}

/// The bound on the region walk's own recursion.
///
/// The walk is recursive by structure — an `if`'s arms, a loop's body and every switch arm are
/// regions of their own ([`Walker::region_at`]) — and one cycle in that recursion used to run the
/// process stack out before any stop could be published. This constant is the *evidence-backed*
/// bound that check is taken against, and it is deliberately not the value bound
/// (`build.rs`'s `MAX_VALUE_DEPTH`): that one bounds the nesting of a *value*, not how deep the
/// walk itself may go.
///
/// The number comes from measurements, not from a guess:
///
/// * every recovery in the committed corpus stays at **1–2** levels (the walk's depth is structural
///   nesting, and the corpus' methods are small);
/// * a sweep of a real artifact — every method of every class of `javassist-3.11.0.GA.jar` from
///   `S2-007.war` (347 classes, 3398 methods, 3277 completed walks) — reaches **17** at its
///   deepest, with 20 of those walks at 10 levels or more;
/// * each level costs ~26 KiB of stack in a debug build (the abort's backtrace advances the frame
///   pointer by `0x6620` per three-frame cycle), so 32 levels is ~850 KiB — under the 2 MiB a test
///   thread is given and far under the 8 MiB of the main thread, while release builds are smaller
///   still.
///
/// So the bound is twice the deepest nesting a real artifact was measured to need, and a run that
/// exceeds it is refused before it descends rather than after the stack is gone.
const MAX_REGION_DEPTH: usize = 32;

/// One recovered `try`: the protected range, the instructions of the statement's own block written
/// before it ([`crate::guard::Catches::lead`]), the clauses, where the code after it begins, and
/// the sibling quotes a gap inside it left behind ([`Run`]).
type OwnedTry = (
    Box<Region>,
    (u32, u32),
    Vec<CatchClause>,
    Option<CanonicalBlockId>,
    Vec<Region>,
);

/// What one call of the walk proved, in the order the text writes it, and where the run continues.
///
/// A call normally proves one region. A gap after proved blocks returns a `Straight` prefix
/// followed by a `Fallback`; a proved transfer after blocks similarly returns the prefix followed
/// by `LoopBreak` or `LoopContinue`. A loop body keeps the ordered run directly, and an if/switch
/// arm can keep it in `Sequence` so the transfer remains inside its branch. The try/catch slots
/// still split a gap and report its quote beside their statement. A quote is not a statement; its
/// BCIs are where it really runs. In every case a proved prefix stays outside the quote.
///
/// The successor is where the walk continues afterwards, when the gap itself states one: the join a
/// branch proved is a block every path passes through, so reading it as the next region's start
/// states the code after the gap instead of quoting it. A gap with no proven join stops the run,
/// and the live blocks it left behind are named by the uncovered-blocks scan at the end of
/// [`recover`].
type Run = (Vec<Region>, Option<CanonicalBlockId>);

/// The run one gap leaves behind, and the successor that continues it.
///
/// `prefix` is what the walk proved before the gap, `gap` the blocks the gap must quote — the block
/// whose shape failed, then every block the walk entered and cannot claim, each named once — and
/// `reason` the refusal itself. The invariant this function exists for: a prefix is **never** quoted
/// (its statements stay statements) and no block a prefix holds appears in the quote.
fn gap(
    prefix: Vec<CanonicalBlockId>,
    gap: Vec<CanonicalBlockId>,
    reason: FallbackReason,
    next: Option<CanonicalBlockId>,
) -> Run {
    let fallback = Region::Fallback {
        blocks: gap,
        reason,
    };
    if prefix.is_empty() {
        (vec![fallback], next)
    } else {
        (vec![Region::Straight { blocks: prefix }, fallback], next)
    }
}

/// The blocks one gap's quote names: the block whose shape failed, then every block the walk
/// entered and cannot claim, each once.
///
/// The order is the walk's: the failing block is what the refusal is *about*, and the blocks the
/// walk entered before it follow. A block that appears twice — an arm the walk entered twice, or a
/// block already named as the failing one — is named once, because the quote is a set of
/// instruction starts.
fn gap_blocks(
    current: &CanonicalBlockId,
    entered: impl IntoIterator<Item = CanonicalBlockId>,
) -> Vec<CanonicalBlockId> {
    let mut blocks = vec![current.clone()];
    for block in entered {
        if !blocks.contains(&block) {
            blocks.push(block);
        }
    }
    blocks
}

/// Takes the head of a run for a try/catch slot that holds one region, and gives back the rest.
///
/// The remaining try/catch slots hold one [`Region`]. If a gap stops after a proved prefix, its
/// quote is returned as a sibling beside that statement (see [`Run`]). If/switch arms instead
/// retain an ordered run in `Sequence`; loop bodies hold a region list. An empty run is not a run.
fn split(run: Vec<Region>) -> (Region, Vec<Region>) {
    let mut run = run.into_iter();
    let head = run
        .next()
        .expect("every walk run holds at least one region");
    (head, run.collect())
}

/// A run of one region: the ordinary case, said once.
fn one(region: Region, next: Option<CanonicalBlockId>) -> Run {
    (vec![region], next)
}

impl Walker<'_> {
    /// The region that starts at one block, and the block the run continues at afterwards.
    ///
    /// `frame` says where this region may go: the join it ends at, and — when it is a loop's body —
    /// the blocks that iterate. Arriving anywhere the frame does not allow ends the run instead of
    /// claiming the block, which is what keeps a loop's body from absorbing the code after it.
    ///
    /// This is the whole recursion family's one entry — the arms, the two loop bodies and every
    /// switch arm all come back through it — so the recursion's two bounds are checked here,
    /// **before** the walk descends, and a run that cannot continue ends as a published stop rather
    /// than as a signal:
    ///
    /// * how **deep** the walk may go, one level per entry ([`MAX_REGION_DEPTH`]);
    /// * that it may not **re-enter** the structure it is already inside. A frame whose `own_loop`
    ///   is the block this entry starts at is the body of that loop, and the walk is back at its
    ///   header: the state it is building, already entered. `loop_region` would take the same shape
    ///   decisions for it — none of them reads the frame — and arrive back here, which is the cycle
    ///   that drove the reported abort.
    fn region_at(&mut self, start: &CanonicalBlockId, frame: &Frame) -> Result<Run, StopReason> {
        let reentered = self
            .view
            .index_of(start)
            .is_some_and(|node| frame.own_loop == Some(node));
        if self.depth >= MAX_REGION_DEPTH || reentered {
            // Which of the two refused the entry is part of the diagnosis: "the input nests too
            // deeply" and "the walk is back inside a structure it is already building" are
            // different facts about the run, and a stop must not name the one that did not happen.
            return Err(StopReason::Interrupted {
                code: if reentered {
                    crate::stop::RECURSION_REENTRY_CODE
                } else {
                    crate::stop::RECURSION_BOUND_CODE
                },
                at: Some(start.bci()),
            });
        }
        self.depth += 1;
        let walked = self.region_at_inner(start, frame);
        self.depth -= 1;
        walked
    }

    /// The walk [`Self::region_at`] bounds: one region of the body, from one block on.
    fn region_at_inner(
        &mut self,
        start: &CanonicalBlockId,
        frame: &Frame,
    ) -> Result<Run, StopReason> {
        let mut prefix: Vec<CanonicalBlockId> = Vec::new();
        let mut current = start.clone();
        loop {
            let at = Some(current.bci());
            poll(self.budget, at)?;
            charge(self.budget, CountedBudgetDimension::AnalysisSteps, 1, at)?;
            let Some(node) = self.view.index_of(&current) else {
                // A block the view holds no node for is not a shape this walk can read, and the
                // prefix it reached the block from stays a `Straight` region of its own: the blocks
                // proved so far are statements, and the quote its own reason states is the one
                // beside them.
                let reason = FallbackReason::UncoveredBlocks {
                    blocks: vec![current.bci()],
                };
                return Ok(gap(prefix, vec![current.clone()], reason, None));
            };
            if frame.stops_at_switch_boundary(node) {
                return Ok(one(Region::Straight { blocks: prefix }, None));
            }
            let successors_here = self.view.successors(node);
            let current_loop = frame.loop_targets.last().map(|target| target.header);
            let is_transfer_gateway = frame.loop_targets.iter().any(|target| {
                target.exits.contains(&node)
                    && successors_here.len() == 1
                    && frame
                        .loop_targets
                        .iter()
                        .any(|destination| destination.break_target == Some(successors_here[0]))
            });
            let transfer = (!is_transfer_gateway)
                .then(|| {
                    frame.loop_targets.iter().rev().find_map(|target| {
                        if target.break_target == Some(node) {
                            Some((target.header, false))
                        } else if current_loop != Some(target.header)
                            && target.continue_target == node
                        {
                            Some((target.header, true))
                        } else {
                            None
                        }
                    })
                })
                .flatten();
            if let Some((target_header, is_continue)) = transfer
                && let Some(loop_header) = self.view.id_of(target_header).cloned()
                && let Some(source_bci) = prefix
                    .last()
                    .and_then(|block| self.terminal_bci(block))
                    .or(frame.transfer_source_bci)
            {
                let mut run = Vec::with_capacity(2);
                if !prefix.is_empty() {
                    run.push(Region::Straight { blocks: prefix });
                }
                run.push(if is_continue {
                    Region::LoopContinue {
                        source_bci,
                        loop_header,
                    }
                } else {
                    Region::LoopBreak {
                        source_bci,
                        loop_header,
                    }
                });
                return Ok((run, None));
            }
            if frame.stops_at(node) {
                // The join of the enclosing structure, or the code after the enclosing loop: this
                // region ends where its caller continues.
                return Ok(one(Region::Straight { blocks: prefix }, None));
            }
            // A loop this walk enters from outside is a region of its own, and it *starts* one: the
            // run that led here ends before the loop, because the loop's test is written inside the
            // statement that presents it. Re-entering a block that is not a loop header this subset
            // can prove (an arm that jumps back, an irreducible cycle) stays the stated fallback
            // below.
            if self.view.is_loop_header(node) {
                if prefix.is_empty() {
                    return self.loop_region(&current, node, frame);
                }
                return Ok(one(Region::Straight { blocks: prefix }, Some(current)));
            }
            // A protected range the exception table states with a named `catch` type is the
            // `try`/`catch` statement here, where the guarded rules of P3 2.4 say the region is not
            // theirs. This is read **before** the block is marked visited: the statement's own range
            // starts in this block, and the walk that recovers it starts there too
            // ([`Self::try_region`]).
            if frame.own_try != Some(node)
                && self.starts_catch(&current)
                && let Some((body, lead, catches, join, tails)) =
                    self.try_region(&current, node, frame)?
            {
                let mut run = vec![Region::Try {
                    prefix,
                    lead,
                    body,
                    catches,
                }];
                run.extend(tails);
                return Ok((run, join));
            }
            if !self.visited.insert(node) {
                // The block is already part of the recovered structure: the walk has re-entered one
                // it is building, which is no shape this subset proves. The prefix keeps its
                // statements; the block itself is quoted.
                let reason = FallbackReason::Loop {
                    block_bci: current.bci(),
                };
                return Ok(gap(prefix, vec![current.clone()], reason, None));
            }
            if let Some(reason) = self.leaving_edge(&current) {
                // P3 2.4: the two edges this walk has always refused — an exception edge and a
                // subroutine entry — are where the guarded regions live. A rule that proves one
                // claims every block the statement owns, and the walk continues after it.
                match crate::guard::examine(
                    self.canonical,
                    self.view,
                    self.ssa,
                    self.operations,
                    self.handlers,
                    self.sites,
                    self.profile,
                    &current,
                    self.budget,
                )? {
                    crate::guard::Verdict::Claimed(plan) => {
                        for block in plan.owned() {
                            if let Some(node) = self.view.index_of(block) {
                                self.visited.insert(node);
                            }
                        }
                        let join = plan.join().cloned();
                        return Ok(one(Region::Guard { prefix, plan }, join));
                    }
                    crate::guard::Verdict::Refused { pass, refusal, at } => {
                        let reason = FallbackReason::Guard {
                            pass,
                            code: refusal.code(),
                            at,
                            message: refusal.message().to_string(),
                        };
                        return Ok(gap(prefix, vec![current], reason, None));
                    }
                    crate::guard::Verdict::NotGuarded => {}
                }
                // P3 2.7: a block this walk reached as the protected range of a `try`, whose every
                // exception edge is one a named `catch` row of the table accounts for, is written
                // where it is. The `catch` that owns the edge is the clause this block already
                // stands inside — the frame is that statement's own body — so the instruction that
                // throws has its statement's place, and quoting the block would replace a statement
                // the `catch` around it already owns with the bytecode it was read from. Nothing is
                // moved and no edge is followed: the handler is written by the clause, not here.
                //
                // Every other leaving edge keeps the quote: a block no `try`'s own range reached
                // (`frame.own_try` is `None`, a call outside a `try` among them) is not written
                // under a clause it is not inside, and a catch-all row, a subroutine entry or one
                // unaccounted edge among several is a shape this statement does not state.
                //
                // P3 2.15 excepts one case from all of that: a block that leaves only through edges
                // no instruction of it can take does not leave, and the statement it holds is the
                // method's normal flow. The guarded rules above are asked first, and they are asked
                // about the block and not about the edge being takeable — a rule that refuses a shape
                // states its reason and the quote stays exactly as it was — so what this exception
                // says is that an edge which cannot be taken is not, on its own, a shape this walk
                // must quote the block for. The handler it named is named by the uncovered-blocks
                // scan below, like any other live block no statement reached.
                if !self.leaves_only_through_dead_edges(&current)
                    && (frame.own_try.is_none() || !self.edges_accounted_by_catches(&current))
                {
                    return Ok(gap(prefix, vec![current], reason, None));
                }
            }
            // The successors this frame allows: the ones inside its scope, plus the one node it
            // ends at (a loop's own test), which is kept so the run can stop *on arrival* rather
            // than claim the block. Everything else is an edge the frame drops.
            let all = self.view.successor_ids(&current);
            let successors: Vec<CanonicalBlockId> = all
                .iter()
                .filter(|successor| {
                    self.view.index_of(successor).is_some_and(|node| {
                        frame.boundary == Some(node)
                            || frame.loop_exit == Some(node)
                            || frame
                                .loop_targets
                                .iter()
                                .any(|target| target.exits.contains(&node))
                            || frame.loop_targets.iter().any(|target| {
                                frame.loop_targets.last().map(|current| current.header)
                                    != Some(target.header)
                                    && target.continue_target == node
                            })
                            || frame
                                .scope
                                .as_ref()
                                .is_none_or(|scope| scope.contains(&node))
                    })
                })
                .cloned()
                .collect();
            // A *branch* that loses a target this way is a way out of the structure the subset
            // cannot write (a `break`, an arm that jumps past the loop). Dropping it silently would
            // drop an effect, so the loop is quoted instead. A single-successor block that loses
            // its edge cannot pass the loop's own post-condition, which checks that every block of
            // the loop was walked.
            if successors.len() != all.len() {
                let branched = self
                    .terminal_bci(&current)
                    .and_then(|bci| self.operations.get(bci))
                    .is_some_and(|operation| {
                        operation.comparison().is_some() || operation.switch().is_some()
                    });
                if branched {
                    let reason = FallbackReason::LoopLeavesEarly {
                        block_bci: current.bci(),
                    };
                    return Ok(gap(prefix, vec![current], reason, None));
                }
            }
            // A block whose terminal instruction the decode states as a `switch` is one, however
            // many *distinct* targets the graph kept: two keys that share a target and a default of
            // their own are three targets in the decode and two successors in the graph, and the
            // keys are what make the statement a `switch` rather than an `if`.
            if successors.len() >= 2
                && let Some(branch_bci) = self.terminal_bci(&current)
                && let Some((cases, default)) =
                    self.operations.get(branch_bci).and_then(Operation::switch)
            {
                let cases = cases.to_vec();
                let branch = current.clone();
                return self.switch_region(
                    &prefix,
                    &branch,
                    branch_bci,
                    &successors,
                    &cases,
                    default,
                    node,
                    frame,
                );
            }
            match successors.len() {
                0 => {
                    // The run ends here: the AST builder writes the terminal statement of this
                    // block, and whether it is a `return` or a throw site is its question.
                    prefix.push(current);
                    return Ok(one(Region::Straight { blocks: prefix }, None));
                }
                1 => {
                    prefix.push(current.clone());
                    let next = successors[0].clone();
                    current = next;
                }
                2 => {
                    let branch = current.clone();
                    let branch_bci = match self.terminal_bci(&branch) {
                        Some(bci) => bci,
                        None => {
                            let reason = FallbackReason::MissingEvidence {
                                block_bci: branch.bci(),
                            };
                            return Ok(gap(prefix, vec![branch], reason, None));
                        }
                    };
                    let Some((op, target)) = self
                        .operations
                        .get(branch_bci)
                        .and_then(Operation::comparison)
                    else {
                        let reason = FallbackReason::UnknownBranchSense {
                            block_bci: branch.bci(),
                            branch_bci,
                        };
                        return Ok(gap(prefix, vec![branch], reason, None));
                    };
                    let Some((fall_through, taken)) = self.split_arms(&successors, target) else {
                        let reason = FallbackReason::UnrenderableOperand { bci: branch_bci };
                        return Ok(gap(prefix, vec![branch], reason, None));
                    };
                    if let Some((region, next)) =
                        self.short_circuit_value(&prefix, &branch, branch_bci, frame)?
                    {
                        return Ok(one(region, next));
                    }
                    if let Some(region) =
                        self.two_exit_return(&prefix, &branch, branch_bci, frame)?
                    {
                        return Ok(one(region, None));
                    }
                    let post_join = self
                        .view
                        .immediate_post_dominator(node)
                        .filter(|join| *join != node)
                        .filter(|join| {
                            !frame.loop_targets.iter().any(|target| {
                                frame.loop_targets.last().map(|current| current.header)
                                    != Some(target.header)
                                    && target.continue_target == *join
                            })
                        });
                    // A branch inside a switch arm can finish either at the switch's local
                    // join or at the enclosing loop's exit. The method-wide post-dominator is
                    // then the loop exit, but it is not the join of this Java `if`.
                    let join_node = if post_join.is_some_and(|join| {
                        frame
                            .loop_targets
                            .iter()
                            .any(|target| target.break_target == Some(join))
                    }) {
                        frame.switch_join.or(post_join)
                    } else {
                        post_join
                    };
                    let then_node = self.view.index_of(&fall_through);
                    let else_node = self.view.index_of(&taken);
                    // A successor that *is* the join is the whole arm: the branch arrives at the
                    // place its structure ends at directly, so that arm holds no block of its own —
                    // the block starting there belongs to whatever follows the `if` — and the other
                    // successor's region is the one body the statement has. The builder writes no
                    // `else` for an empty arm, which is exactly the one-armed `if` the bytecode
                    // states. Nothing here is guessed from the addresses; the join is read in two
                    // ways, and both are facts the graph states:
                    //
                    // * the branch's own **immediate post-dominator**, when one successor *is* it:
                    //   the nearest block every path out of the branch passes through;
                    // * the **forward join** of the two successors ([`Self::forward_join`]), when
                    //   the post-dominator is further away because one arm leaves the method before
                    //   the join — `javac` writes `if (a > 0 && b > 0) return 1; return 0;` as two
                    //   forward branches onto one block, and that block is where both arms meet.
                    //
                    // A branch whose **two** successors are both the join states no arm at all and
                    // keeps the refusal below.
                    let forward_then = self.forward_join(node, then_node, else_node, frame);
                    let forward_else = self.forward_join(node, else_node, then_node, frame);
                    if std::env::var_os("JRE_JOIN_PROBE").is_some() {
                        eprintln!(
                            "P3JOIN branch={} ipdom={:?} then={:?} else={:?} ft={forward_then} fe={forward_else} frame_boundary={:?}",
                            branch.bci(),
                            join_node
                                .and_then(|join| self.view.id_of(join).map(CanonicalBlockId::bci)),
                            then_node
                                .and_then(|node| self.view.id_of(node).map(CanonicalBlockId::bci)),
                            else_node
                                .and_then(|node| self.view.id_of(node).map(CanonicalBlockId::bci)),
                            frame
                                .boundary
                                .and_then(|node| self.view.id_of(node).map(CanonicalBlockId::bci)),
                        );
                    }
                    let one_armed = match join_node {
                        Some(join_node)
                            if (then_node == Some(join_node)) != (else_node == Some(join_node)) =>
                        {
                            Some(join_node)
                        }
                        // The post-dominator states no one-armed shape here: exactly one successor
                        // being the forward join of the two does.
                        _ if forward_then != forward_else => {
                            if forward_then {
                                then_node
                            } else {
                                else_node
                            }
                        }
                        _ => None,
                    };
                    let join = one_armed
                        .or(join_node)
                        .and_then(|join| self.view.id_of(join).cloned());
                    if let Some(join_node) = one_armed {
                        let walk = if then_node == Some(join_node) {
                            &taken
                        } else {
                            &fall_through
                        };
                        // The condition's arity is read *before* the arm is walked, so a branch the
                        // builder would refuse claims no block on the way to being refused: an arm
                        // walked and then dropped would leave the blocks it claimed out of the
                        // uncovered-blocks statement as well as out of the text.
                        if let Err(reason) = self.branch_arity_proved(&branch, branch_bci, op) {
                            let next = self.unclaimed_join(join.as_ref());
                            return Ok(gap(prefix, vec![branch], reason, next));
                        }
                        let (arm_run, _) =
                            self.region_at(walk, &frame.arm(Some(join_node), Some(branch_bci)))?;
                        let arm = sequence_region(arm_run);
                        // A one-armed `if` normally has an empty arm because that edge is the
                        // branch's join. Inside a loop, the join can instead be the exact exit of
                        // an enclosing loop. In that case an empty arm drops a real exit edge and
                        // can turn a terminating loop into an infinite one; retain the transfer as
                        // the existing `LoopBreak` leaf.
                        let join_arm = frame
                            .loop_targets
                            .iter()
                            .rev()
                            .find(|target| target.break_target == Some(join_node))
                            .and_then(|target| {
                                self.view.id_of(target.header).cloned().map(|loop_header| {
                                    Region::LoopBreak {
                                        source_bci: branch_bci,
                                        loop_header,
                                    }
                                })
                            })
                            .unwrap_or(Region::Straight { blocks: Vec::new() });
                        let empty = Box::new(join_arm);
                        let (then_arm, else_arm) = if then_node == Some(join_node) {
                            (empty, Box::new(arm))
                        } else {
                            (Box::new(arm), empty)
                        };
                        let run = vec![Region::If {
                            prefix,
                            branch,
                            branch_bci,
                            then_arm,
                            else_arm,
                            join: join.clone(),
                        }];
                        return Ok((run, join));
                    }
                    // Two successors that are both the join state no arm at all: nothing in the
                    // graph says which of them an arm would be, so the branch keeps its refusal
                    // rather than becoming an `if` with two empty arms. The same holds when both
                    // successors are a forward join: the graph states a convergence, but not which
                    // of the two arms arrives empty, so neither is written.
                    if (join_node.is_some() && then_node == join_node && else_node == join_node)
                        || (forward_then && forward_else)
                    {
                        let reason = FallbackReason::ArmsDoNotMeet {
                            block_bci: branch.bci(),
                        };
                        let next = self.unclaimed_join(join.as_ref());
                        return Ok(gap(prefix, vec![branch], reason, next));
                    }
                    let arm_frame = frame.arm(join_node, Some(branch_bci));
                    let (then_run, then_next) = self.region_at(&fall_through, &arm_frame)?;
                    let (else_run, else_next) = self.region_at(&taken, &arm_frame)?;
                    // An arm the branch cannot present is quoted, not dropped and not written as if
                    // the branch had been: both arms' blocks are named by the refusal, each once.
                    let arm_blocks: Vec<CanonicalBlockId> = then_run
                        .iter()
                        .chain(else_run.iter())
                        .flat_map(Region::blocks)
                        .cloned()
                        .collect();
                    // Both arms must meet the join, or leave the method; anything else means the
                    // recovered structure runs into a block the branch did not describe.
                    if let Some(join_node) = join_node {
                        let then_meets =
                            then_node.is_some_and(|node| self.view.reaches(node, join_node));
                        let else_meets =
                            else_node.is_some_and(|node| self.view.reaches(node, join_node));
                        let then_breaks = then_next.is_none()
                            && matches!(then_run.last(), Some(Region::LoopBreak { .. }));
                        let else_breaks = else_next.is_none()
                            && matches!(else_run.last(), Some(Region::LoopBreak { .. }));
                        let both_end = !then_meets && !else_meets;
                        let local_switch_join = frame.switch_join == Some(join_node);
                        if !(then_meets && else_meets)
                            && !both_end
                            && !(local_switch_join
                                && ((then_meets && else_breaks) || (else_meets && then_breaks)))
                        {
                            let reason = FallbackReason::ArmsDoNotMeet {
                                block_bci: branch.bci(),
                            };
                            let next = self.unclaimed_join(join.as_ref());
                            let blocks = gap_blocks(&branch, arm_blocks);
                            return Ok(gap(prefix, blocks, reason, next));
                        }
                    }
                    // The condition's arity is a precondition of the statement the builder writes.
                    if let Err(reason) = self.branch_arity_proved(&branch, branch_bci, op) {
                        let next = self.unclaimed_join(join.as_ref());
                        let blocks = gap_blocks(&branch, arm_blocks);
                        return Ok(gap(prefix, blocks, reason, next));
                    }
                    let run = vec![Region::If {
                        prefix,
                        branch,
                        branch_bci,
                        then_arm: Box::new(sequence_region(then_run)),
                        else_arm: Box::new(sequence_region(else_run)),
                        join: join.clone(),
                    }];
                    return Ok((run, join));
                }
                count => {
                    let branch = current.clone();
                    let branch_bci = match self.terminal_bci(&branch) {
                        Some(bci) => bci,
                        None => {
                            let reason = FallbackReason::MissingEvidence {
                                block_bci: branch.bci(),
                            };
                            return Ok(gap(prefix, vec![branch], reason, None));
                        }
                    };
                    let Some((cases, default)) =
                        self.operations.get(branch_bci).and_then(Operation::switch)
                    else {
                        // Three or more targets whose terminal instruction the decode does not
                        // state as a `switch`: no shape this subset knows.
                        let reason = FallbackReason::BranchTargets {
                            block_bci: branch.bci(),
                            successors: count,
                        };
                        return Ok(gap(prefix, vec![branch], reason, None));
                    };
                    let cases = cases.to_vec();
                    return self.switch_region(
                        &prefix,
                        &branch,
                        branch_bci,
                        &successors,
                        &cases,
                        default,
                        node,
                        frame,
                    );
                }
            }
        }
    }

    /// A join the walk may continue at after a gap: the one the gap proved, when no region has
    /// claimed it yet.
    ///
    /// A branch's immediate post-dominator is where every path out of the branch arrives, so its own
    /// region is written once, after the quote, instead of being named as bytecode the walk never
    /// reached. The block is read **only** when nothing claimed it: a join some region already holds
    /// is that region's, and starting a second walk at it would read the same block twice — the
    /// `visited` check in [`Self::region_at_inner`] would answer [`FallbackReason::Loop`], which is
    /// a statement about a block that is no loop at all.
    fn unclaimed_join(&self, join: Option<&CanonicalBlockId>) -> Option<CanonicalBlockId> {
        join.filter(|join| {
            self.view
                .index_of(join)
                .is_some_and(|node| !self.visited.contains(&node))
        })
        .cloned()
    }

    /// Whether a named row's protected range begins at one block.
    ///
    /// The cheap precondition of the `try`/`catch` shape, read before the shape is examined: a block
    /// no named row begins at is not the head of one, and nothing is paid for asking.
    /// Whether a block holds the head of a `try`: a row that names a `catch` type begins where the
    /// block does, or **inside** it.
    ///
    /// The second half is the same fact as the canonical graph's fusion: `javac` puts a `try` after
    /// a statement the compiler ran in the same straight-line run (`int x = 1; try { … }`), and the
    /// two are one block — so the statement's own range begins inside the block that holds the
    /// instructions before it. The range's start is what the statement is about, and the block's
    /// instructions before it become the statement's [`Region::Try::lead`].
    fn starts_catch(&self, block: &CanonicalBlockId) -> bool {
        self.handlers.iter().any(|row| {
            row.catch_type_index.is_some()
                && row.start_bci >= block.bci()
                && self
                    .terminal_bci(block)
                    .is_some_and(|last| row.start_bci <= last)
        })
    }

    /// The protected range and the clauses of the `try` that begins in one block, when this block
    /// states one.
    ///
    /// Everything the shape's own proof concludes is [`crate::guard::catches`]': which rows name a
    /// `catch` type, which handler each reaches, which local each handler stores the exception into,
    /// where the protected range begins (the statement's lead is what the block holds before it), and
    /// where the code after the statement begins. What is recovered here is the **structure**: the
    /// protected range as the region the plain flow states, and each handler body as a region of its
    /// own, both ending where the code after the `try` begins. A handler body that cannot be
    /// presented is still a clause: the quote inside it says why, and the clause header stays.
    ///
    /// A statement the table **nests** — two protected ranges that begin at one instruction — is
    /// built by [`Self::try_level`] from the inside out: this block states both ranges, and the
    /// returned body is the inner `try` written inside the outer one.
    ///
    /// `None` means this block is not the head of one — and then the walk reads it exactly as it did
    /// before, because every reason `catches` refuses on is a reason this statement has no shape to
    /// write, not a reason to quote the block.
    fn try_region(
        &mut self,
        start: &CanonicalBlockId,
        node: usize,
        frame: &Frame,
    ) -> Result<Option<OwnedTry>, StopReason> {
        let Some(shape) = crate::guard::catches(
            self.canonical,
            self.view,
            self.ssa,
            self.operations,
            self.handlers,
            self.profile,
            start,
            self.budget,
        )?
        else {
            return Ok(None);
        };
        let (body, catches, join, tails) = self.try_level(start, node, frame, &shape)?;
        Ok(Some((Box::new(body), shape.lead, catches, join, tails)))
    }

    /// One level of a `try`/`catch` statement: its body, its clauses, the block the code after the
    /// whole statement begins at, and the sibling quotes a gap inside it left behind ([`Run`]).
    ///
    /// A statement the exception table **nests** — two protected ranges that begin at one instruction
    /// — is built from the inside out: the narrower range's `try` is the wider one's body
    /// ([`crate::guard::Catches::inner`]), so the walk below runs once per level and only the
    /// innermost level's body is a region of a protected range itself.
    ///
    /// The **boundary** every clause body ends at is the code after the whole statement. For a
    /// statement of one range that is the level's own join; for a level whose body is the nested
    /// inner `try` it is the inner statement's join — the code the outer statement runs on
    /// completion, because its body *is* that inner statement — read through the transfer blocks
    /// `javac` may leave between the two ([`Self::after_join`]).
    fn try_level(
        &mut self,
        start: &CanonicalBlockId,
        node: usize,
        frame: &Frame,
        shape: &crate::guard::Catches,
    ) -> Result<
        (
            Region,
            Vec<CatchClause>,
            Option<CanonicalBlockId>,
            Vec<Region>,
        ),
        StopReason,
    > {
        // A protected range or a clause body whose walk met a gap keeps its own statements inside
        // the `try` and reports the quote the gap owes beside the statement ([`Run`]): the region a
        // slot holds is one region, and the sibling quote is written after the `try` in the same
        // method, where the bytecode it names still runs.
        let mut tails: Vec<Region> = Vec::new();
        let (body, boundary) = match &shape.inner {
            Some(inner) => {
                let (body, catches, join, inner_tails) =
                    self.try_level(start, node, frame, inner)?;
                let boundary = join.as_ref().map(|join| self.after_join(join));
                let body = Region::Try {
                    prefix: Vec::new(),
                    lead: inner.lead,
                    body: Box::new(body),
                    catches,
                };
                tails.extend(inner_tails);
                (body, boundary)
            }
            None => {
                let boundary = shape.join.clone();
                let boundary_node = boundary
                    .as_ref()
                    .and_then(|block| self.view.index_of(block));
                let (body, _) = self.region_at(start, &frame.protected(boundary_node, node))?;
                let (body, body_tails) = split(body);
                tails.extend(body_tails);
                (body, boundary)
            }
        };
        let boundary_node = boundary
            .as_ref()
            .and_then(|block| self.view.index_of(block));
        let mut catches = Vec::with_capacity(shape.sites.len());
        for site in &shape.sites {
            let (handler, _) = self.region_at(&site.handler, &frame.arm(boundary_node, None))?;
            let (handler, handler_tails) = split(handler);
            tails.extend(handler_tails);
            catches.push(CatchClause {
                type_indices: site.type_indices.clone(),
                handler: site.handler.clone(),
                parameter: site.parameter,
                body: Box::new(handler),
            });
        }
        Ok((body, catches, shape.join.clone(), tails))
    }

    /// The block the code after one `try` begins at: `join` itself, or the target of the transfer
    /// blocks that stand between it and that code.
    ///
    /// A nested statement's join is the **inner** statement's, and the outer `try`'s own exit comes
    /// after it: `javac` leaves that exit a `goto` (`goto` the code after the outer statement), and
    /// so may a statement that follows. A clause body is written up to the code *after* the
    /// statement — stopping at the transfer instead would let a handler that falls through the
    /// transfer run the statement that follows the `try` as if the clause held it. The walk that
    /// continues after the statement still starts at `join` itself, so the blocks read through here
    /// are claimed by it and not left unread.
    fn after_join(&self, join: &CanonicalBlockId) -> CanonicalBlockId {
        let mut block = join.clone();
        // A transfer cannot be followed forever: the bound is what keeps a `goto` cycle (a
        // statement whose exits pass through the loop of another structure) from spinning here.
        for _ in 0..8 {
            let instructions = self
                .ssa
                .block(&block)
                .map(|entry| entry.instructions())
                .unwrap_or(&[]);
            let [only] = instructions else {
                break;
            };
            if !matches!(self.operations.get(only.bci()), Some(Operation::Transfer)) {
                break;
            }
            let Some(next) = self.view.successor_ids(&block).first().cloned() else {
                break;
            };
            block = next;
        }
        block
    }

    /// Whether a block leaves through an edge the projection does not carry.
    ///
    /// Every edge the graph states is read as the graph states it; whether the block can *take* one
    /// is [`Self::leaves_only_through_dead_edges`]'s question, asked where a quote is decided. The
    /// two are separate on purpose: this edge is what sends a block to the guarded rules of P3 2.4
    /// ([`crate::guard::examine`]), and a rule that refuses the block refuses it whatever an
    /// instruction of it may raise.
    fn leaving_edge(&self, block: &CanonicalBlockId) -> Option<FallbackReason> {
        self.canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == block)
            .find_map(|edge| match edge.kind() {
                CanonicalEdgeKind::Exception { handler_ordinal } => {
                    Some(FallbackReason::ExceptionEdge {
                        block_bci: block.bci(),
                        handler_ordinal,
                    })
                }
                CanonicalEdgeKind::Call { call_site } => Some(FallbackReason::SubroutineEntry {
                    block_bci: block.bci(),
                    call_site,
                }),
                _ => None,
            })
    }

    /// Whether one exception edge is one an instruction of this block can actually take (P3 2.15).
    ///
    /// The graph states the exception table's rows in two ways, and the table is the same statement
    /// either way (P3 2.9): a record a `may_throw` instruction of the body **covers** keeps the edge
    /// its sites feed — the instruction's BCI lies inside the record's range — and a record **no**
    /// such instruction covers is stated by its protected range, one edge per block the range
    /// intersects. The second kind is an edge no instruction of the block can take: nothing in it can
    /// raise, so no run of the block ever enters the handler. `javac --release 8` writes exactly that
    /// row for `try { n = n + 1; } catch (RuntimeException e)` — and for the `try`/`finally` of
    /// `finallyIncrements(I)I`, whose catch-all row `[2, 4)` covers `iload_0; istore_1`.
    ///
    /// `may_throw` is the raw CFG's own classification of an opcode (its site scan in
    /// `jarde_jvm`'s `cfg` module), and [`CanonicalCfg::throw_sites`] is that same scan's published
    /// result: one entry per throwing instruction, holding the block it runs in and the record
    /// ordinals whose ranges cover it. Reading the predicate there rather than restating its opcode
    /// table here is what keeps the two answers one: a second copy of that table in this crate is a
    /// second answer to the same question, and the one the graph was built from would not be the one
    /// this walk read.
    fn exception_edge_takeable(&self, block: &CanonicalBlockId, handler_ordinal: u32) -> bool {
        self.canonical
            .throw_sites()
            .iter()
            .any(|site| site.block() == block && site.handlers().contains(&handler_ordinal))
    }

    /// Whether every edge the block leaves the normal flow through is an exception edge no
    /// instruction of the block can take, and there is at least one (P3 2.15).
    ///
    /// Such a block does not leave the normal flow: the records its edges name are the table's
    /// statement over a block nothing in can throw ([`Self::exception_edge_takeable`]), so quoting it
    /// would replace a statement the bytes hold with bytecode — `finallyIncrements(I)I` was quoted
    /// whole for exactly this reason, though the row it leaves by is stated over `iload_0; istore_1`
    /// and the four statements of its body are the normal flow of the method. The handler the record
    /// reaches is not lost: it is a live block no walk reached, so the uncovered-blocks scan at the
    /// end of [`recover`] names it, exactly as it names a subroutine body.
    ///
    /// A block with a subroutine entry, or with one edge a `may_throw` instruction does feed, keeps
    /// its quote: the reading is "nothing here can throw", never "nothing here leaves". The plain
    /// transfer the block hands control on by ([`CanonicalEdgeKind::Normal`], a `Return`) is not a
    /// way *out* in this sense — the walk follows it — so it is read no more here than
    /// [`Self::leaving_edge`] reads it.
    fn leaves_only_through_dead_edges(&self, block: &CanonicalBlockId) -> bool {
        let mut dead = false;
        for edge in self
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == block)
        {
            match edge.kind() {
                CanonicalEdgeKind::Exception { handler_ordinal } => {
                    if self.exception_edge_takeable(block, handler_ordinal) {
                        return false;
                    }
                    dead = true;
                }
                CanonicalEdgeKind::Call { .. } => return false,
                CanonicalEdgeKind::Normal | CanonicalEdgeKind::Return { .. } => {}
            }
        }
        dead
    }

    /// Whether every edge this block leaves the normal flow through is one a named `catch` row of
    /// the declared table accounts for (P3 2.7).
    ///
    /// The edges are read one by one rather than through [`Self::leaving_edge`]'s first match: a
    /// block under two rows that reach two handlers carries two exception edges, and the statement
    /// is written only when **both** handlers are clauses of the `try` this block is inside. A
    /// block whose only leaving edge is a subroutine entry answers `false` at that edge, so the
    /// caller's own reason ([`FallbackReason::SubroutineEntry`]) is never replaced by "nothing here
    /// is wrong".
    ///
    /// An edge no instruction of the block can take settles nothing and blocks nothing (P3 2.15):
    /// it is skipped, neither asked whether it is a clause's edge nor allowed to answer that the
    /// block stays quoted. What it is *not* is `accounted = true` on its own — a block whose only
    /// edges are dead ones has no clause to be written inside, which is
    /// [`Self::leaves_only_through_dead_edges`]'s answer to give at the quote site.
    fn edges_accounted_by_catches(&self, block: &CanonicalBlockId) -> bool {
        let mut accounted = false;
        for edge in self
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == block)
        {
            match edge.kind() {
                CanonicalEdgeKind::Exception { handler_ordinal } => {
                    if !self.exception_edge_takeable(block, handler_ordinal) {
                        continue;
                    }
                    if !self.exception_edge_accounted(block, edge.to(), handler_ordinal) {
                        return false;
                    }
                    accounted = true;
                }
                // A subroutine entry is no exception path: the `jsr` context is not a clause this
                // statement holds, so the block keeps the quote its own reason states.
                CanonicalEdgeKind::Call { .. } => return false,
                CanonicalEdgeKind::Normal | CanonicalEdgeKind::Return { .. } => {}
            }
        }
        accounted
    }

    /// Whether one exception edge is the one a named `catch` row accounts for: the row the edge's
    /// ordinal names, whose `[start_bci, end_bci)` **intersects** this block's span and **reaches
    /// its end**, and whose handler entry is the block the edge reaches.
    ///
    /// The row is read from the same decode's table ([`Walker::handlers`]) the graph's edges were
    /// built from, and the handler is mapped the way the rest of this file maps a row to a block
    /// ([`CanonicalCfg::handler_rows`]). A `catch_type == 0` row — the catch-all a `finally` copy is
    /// written under — names no clause, so its handler is nobody this statement writes around the
    /// block and the edge is not accounted for.
    ///
    /// P3 2.15 changed what the range has to meet. It used to have to **cover the block's start**
    /// ([`CanonicalBlockId::bci`]), and that test read the canonical graph's fusion rather than the
    /// table: a straight-line run is one node, so the narrower of two nested ranges begins *inside*
    /// the block that holds the code before it — `nested(I)I`'s inner `[8, 11)` begins at the
    /// `bipush -2` of the block `[7, 11)` the outer clause's own binding store opened — and a
    /// statement whose range merely began inside its block was quoted as if no row protected it. The
    /// edge is a graph fact since P3 2.9 and the handler is what the block may enter; where the
    /// range begins is the *statement's* lead ([`crate::guard::Catches::lead`]), which
    /// [`crate::build`] writes before the `try`. So the row's range and the block's span have to
    /// meet, and the graph's own answer to "what does this block span" is the node's
    /// [`jarde_jvm::method_ir::CanonicalBlock::end_bci`] — from its start BCI to just past its last
    /// original block, the same half-open shape the raw pass read the record's range against when it
    /// built the edge.
    ///
    /// Meeting is not the whole of it, because the walk writes **whole blocks**: everything the
    /// block holds after the range end has to be the statement's own exit, or the block is quoted
    /// instead (the check below). A range that stops inside its block and leaves real instructions
    /// behind it — `Held.use`'s failed resource proof, whose close shares the block its row `[2, 7)`
    /// protects only up to BCI 7 — would otherwise be written inside the clause, presenting
    /// exceptions the bytes' own range does not catch.
    fn exception_edge_accounted(
        &self,
        block: &CanonicalBlockId,
        handler: &CanonicalBlockId,
        handler_ordinal: u32,
    ) -> bool {
        let Some(row) = self
            .handlers
            .iter()
            .find(|row| row.ordinal == handler_ordinal)
        else {
            return false;
        };
        let Some(span) = self
            .canonical
            .blocks()
            .iter()
            .find(|entry| entry.id() == block)
        else {
            return false;
        };
        if row.catch_type_index.is_none()
            || row.start_bci >= span.end_bci()
            || block.bci() >= row.end_bci
        {
            return false;
        }
        // ...and what the block holds **after** the range has to be the statement's own exit: the
        // walk writes whole blocks, so a block with real instructions after the range cannot be
        // written inside the clause — they are not protected by the row. `Held.use`'s failed
        // resource proof is that shape: its row `[2, 7)` protects `aload_0; invokevirtual read;
        // istore_2` and the close the compiler fills in after the range shares the block
        // (`aload_1; ifnonnull 15; aload_1; invokevirtual close`), so writing the block inside the
        // `catch` the row is read as would present an exception the bytecode's own range does not
        // catch. A `goto` is the statement's exit and is written by the structure around it
        // (`steps(I)I`'s block `[0, 7)` ends in one after its range `[0, 4)`), so it is allowed.
        let Some(names) = self.ssa.block(block) else {
            return false;
        };
        let after_range: Vec<_> = names
            .instructions()
            .iter()
            .filter(|instruction| instruction.bci() >= row.end_bci)
            .collect();
        let transfers_are_accounted = after_range.iter().all(|instruction| {
            matches!(
                self.operations.get(instruction.bci()),
                Some(Operation::Transfer)
            )
        });
        let boundary_return_is_accounted = self.method_synchronized == Some(false)
            && after_range.len() == 1
            && after_range[0].bci() == row.end_bci
            && names.instructions().last().is_some_and(|last| {
                last.bci() == after_range[0].bci()
                    && self.operations.get(last.bci()) == Some(&Operation::Return)
                    && (0xac..=0xb0).contains(&last.opcode())
                    && last.reads().len() == 1
                    && last.reads().first().is_some_and(|(_, value_id)| {
                        let value = self.ssa.value(*value_id);
                        value.replaced_by().is_none()
                            && matches!(
                                value.def(),
                                Definition::Instruction {
                                    block: producer_block,
                                    bci,
                                } if producer_block == block
                                    && *bci >= row.start_bci
                                    && *bci < row.end_bci
                            )
                    })
            })
            && !self.has_reachable_explicit_monitor;
        if !transfers_are_accounted && !boundary_return_is_accounted {
            return false;
        }
        self.canonical
            .handler_rows()
            .iter()
            .find(|entry| entry.ordinal() == handler_ordinal)
            .and_then(|entry| entry.handler())
            .is_some_and(|mapped| mapped == handler)
    }

    /// The BCI of the last instruction of one block, as the names table states it.
    fn terminal_bci(&self, block: &CanonicalBlockId) -> Option<u32> {
        self.ssa.block(block).and_then(|block| {
            block
                .instructions()
                .last()
                .map(|instruction| instruction.bci())
        })
    }

    /// Whether the branch at one BCI reads exactly the values its decoded sense needs.
    ///
    /// The condition's arity is a precondition of the statement the builder writes: a branch whose
    /// names record states a different number of reads than its operation has operands has no
    /// condition this layer can prove, so its block is quoted instead of becoming an `if`.
    fn branch_arity_proved(
        &self,
        branch: &CanonicalBlockId,
        branch_bci: u32,
        op: CompareOp,
    ) -> Result<(), FallbackReason> {
        let reads = self
            .ssa
            .block(branch)
            .map(|block| {
                block
                    .instructions()
                    .iter()
                    .filter(|instruction| instruction.bci() == branch_bci)
                    .map(|instruction| instruction.reads().len())
                    .next()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        let expected = if op.reads_two() { 2 } else { 1 };
        if reads == expected {
            Ok(())
        } else {
            Err(FallbackReason::UnrenderableOperand { bci: branch_bci })
        }
    }

    /// Whether the test block of a structure holds nothing but the values its test reads.
    ///
    /// The test block's instructions are written *inside* the structure they decide — in a loop's
    /// condition, in an `if`'s condition — and only value-producing instructions have a place
    /// there: a `Push`, a `Load` or an `Arithmetic` becomes the text of an operand, so it runs
    /// exactly once per evaluation and in the same order the bytecode ran it. A store, an
    /// increment, an unused call, a return or an operation this subset does not model is an
    /// **effect** of the test block. A call or field read used by the branch can stay in the
    /// condition expression; an unused call has nowhere to go that keeps its execution count and
    /// order. Hoisting it out would run it once, and putting it in the body would run it after the
    /// test, so the structure is quoted instead.
    fn test_is_pure(&self, block: &CanonicalBlockId, test_bci: u32) -> Result<(), FallbackReason> {
        let Some(names) = self.ssa.block(block) else {
            return Ok(());
        };
        let condition_bcis = self.condition_value_bcis(block, test_bci, names);
        for instruction in names.instructions() {
            if instruction.bci() == test_bci {
                continue;
            }
            let operation = self.operations.get(instruction.bci());
            let value_only = matches!(
                operation,
                Some(
                    Operation::Push(_)
                        | Operation::Load { .. }
                        | Operation::Arithmetic { .. }
                        | Operation::Negate
                        | Operation::NumericComparison { .. },
                )
            ) || (condition_bcis.contains(&instruction.bci())
                && matches!(
                    operation,
                    Some(
                        Operation::Invoke(_)
                            | Operation::Field {
                                access: crate::facts::FieldAccess::Read,
                                ..
                            }
                    )
                ));
            if !value_only {
                // The loop pass declares this precondition (`pass::LOOP.requires(StatementFree)`)
                // and the check is stated through the declaration: the reason carries the rule
                // version, so the text and the report say *which rule* refused, not just that
                // something did.
                return Err(FallbackReason::unmet(
                    &crate::pass::LOOP,
                    crate::pass::Precondition::StatementFree,
                    block.bci(),
                    instruction.bci(),
                ));
            }
        }
        Ok(())
    }

    /// The same-block producers whose values the terminal branch consumes, directly or through
    /// other value producers. Calls and field reads may remain in a loop condition only when this
    /// walk proves that `test_expr` will render them there.
    fn condition_value_bcis(
        &self,
        block: &CanonicalBlockId,
        test_bci: u32,
        names: &SsaBlock,
    ) -> BTreeSet<u32> {
        let Some(test) = names
            .instructions()
            .iter()
            .find(|instruction| instruction.bci() == test_bci)
        else {
            return BTreeSet::new();
        };
        let mut pending: Vec<ValueId> = test.reads().iter().map(|(_, value)| *value).collect();
        let mut seen = BTreeSet::new();
        let mut producers = BTreeSet::new();
        while let Some(value) = pending.pop() {
            if !seen.insert(value) {
                continue;
            }
            let Definition::Instruction {
                block: producer_block,
                bci,
            } = self.ssa.value(value).def()
            else {
                continue;
            };
            if producer_block != block || *bci == test_bci || !producers.insert(*bci) {
                continue;
            }
            if let Some(producer) = names
                .instructions()
                .iter()
                .find(|instruction| instruction.bci() == *bci)
            {
                pending.extend(producer.reads().iter().map(|(_, value)| *value));
            }
        }
        producers
    }

    /// Which successor control falls through to, and which one the branch transfers to.
    ///
    /// The transferred one is the successor the branch's own operand names — a decode fact, not a
    /// guess from the addresses. Reading it is what makes a backward branch splittable at all: a
    /// loop's test jumps *behind* itself, and "the further successor is the target" would put the
    /// two arms the wrong way round on exactly the shape a loop is made of.
    fn split_arms(
        &self,
        successors: &[CanonicalBlockId],
        target: u32,
    ) -> Option<(CanonicalBlockId, CanonicalBlockId)> {
        let taken = successors
            .iter()
            .find(|successor| successor.bci() == target)?;
        let fall_through = successors
            .iter()
            .find(|successor| successor.bci() != target)?;
        if taken.bci() == fall_through.bci() {
            // Both senses reach the same block: there is no arm split to present.
            return None;
        }
        Some((fall_through.clone(), taken.clone()))
    }

    /// Claims a complete forward test DAG with two terminal returns before ordinary `If`
    /// recursion can claim either shared leaf twice. This is ownership only; the builder proves
    /// the return values and every test expression before publishing a statement.
    ///
    /// The claim is made only when the method's own descriptor says the returns are `boolean`:
    /// that is the first fact the builder re-checks, and a claim it can never prove is not
    /// harmless — it commits the whole closure's ownership up front, so the ordinary one-armed
    /// `if` walk (whose join is exactly such a shared leaf, P3 1.4) never runs and an
    /// `int`-returning `if (a > 0) { if (b > 0) { return 1; } } return 0;` loses its statement
    /// to a quote. Leaving the shape to the `If` walk when the fold cannot succeed is the
    /// rejection the shared-terminal-returns change itself calls atomic.
    fn two_exit_return(
        &mut self,
        prefix: &[CanonicalBlockId],
        outer: &CanonicalBlockId,
        outer_bci: u32,
        frame: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        if !self.return_is_boolean
            || frame.scope.is_some()
            || frame.case_entries.is_some()
            || frame.own_loop.is_some()
            || frame.own_try.is_some()
            || frame.boundary.is_some()
            || frame.loop_exit.is_some()
            || !frame.loop_targets.is_empty()
            || frame.switch_join.is_some()
        {
            return Ok(None);
        }
        const MAX_TESTS: usize = 24;
        let mut pending = vec![outer.clone()];
        let mut seen = BTreeSet::new();
        let mut tests = Vec::new();
        let mut test_edges = Vec::new();
        let mut gateways = Vec::new();
        let mut returns = Vec::new();
        let mut decoded = None;
        while let Some(block) = pending.pop() {
            poll(self.budget, Some(block.bci()))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(block.bci()),
            )?;
            let Some(node) = self.view.index_of(&block) else {
                return Ok(None);
            };
            if !seen.insert(node) {
                continue;
            }
            if seen.len() > MAX_TESTS * 2 + 2
                || self.excluded_edge_nodes.contains(&block)
                || block.path() != outer.path()
            {
                return Ok(None);
            }
            let successors = self.view.successor_ids(&block);
            if successors.iter().any(|next| next.bci() <= block.bci()) {
                return Ok(None);
            }
            match successors.len() {
                2 => {
                    let Some(bci) = self.terminal_bci(&block) else {
                        return Ok(None);
                    };
                    let Some((op, target)) =
                        self.operations.get(bci).and_then(Operation::comparison)
                    else {
                        return Ok(None);
                    };
                    let Some((fallthrough, taken)) = self.split_arms(&successors, target) else {
                        return Ok(None);
                    };
                    if self.branch_arity_proved(&block, bci, op).is_err() {
                        return Ok(None);
                    }
                    charge(
                        self.budget,
                        CountedBudgetDimension::AnalysisSteps,
                        u64::try_from(self.code.instructions.len()).unwrap_or(u64::MAX),
                        Some(bci),
                    )?;
                    let Some(last) = self
                        .code
                        .instructions
                        .iter()
                        .position(|instruction| instruction.bci == bci)
                    else {
                        return Ok(None);
                    };
                    if self
                        .code
                        .instructions
                        .get(last + 1)
                        .map(|instruction| instruction.bci)
                        != Some(fallthrough.bci())
                    {
                        return Ok(None);
                    }
                    tests.push((block, bci));
                    test_edges.push((fallthrough.clone(), taken.clone()));
                    pending.extend([fallthrough, taken]);
                }
                1 => {
                    let Some(names) = self.ssa.block(&block) else {
                        return Ok(None);
                    };
                    let [transfer] = names.instructions() else {
                        return Ok(None);
                    };
                    let next = &successors[0];
                    if !matches!(transfer.opcode(), 0xa7 | 0xc8)
                        || !matches!(
                            self.operations.get(transfer.bci()),
                            Some(Operation::Transfer)
                        )
                        || !transfer.reads().is_empty()
                        || !transfer.writes().is_empty()
                    {
                        return Ok(None);
                    }
                    if decoded.is_none() {
                        charge(
                            self.budget,
                            CountedBudgetDimension::AnalysisSteps,
                            u64::try_from(self.code.instructions.len()).unwrap_or(u64::MAX),
                            Some(transfer.bci()),
                        )?;
                        decoded = self.code.control_flow_targets().ok();
                    }
                    if decoded.as_ref().is_none_or(|targets| {
                        targets
                            .iter()
                            .filter(|target| target.instruction_bci == transfer.bci())
                            .filter(|target| {
                                matches!(
                                    target.kind,
                                    jarde_reader::classfile::ControlFlowTargetKind::Branch { .. }
                                )
                            })
                            .map(|target| target.target_bci)
                            .collect::<Vec<_>>()
                            != [next.bci()]
                    }) {
                        return Ok(None);
                    }
                    gateways.push((block, next.clone()));
                    pending.push(next.clone());
                }
                0 => {
                    let Some(names) = self.ssa.block(&block) else {
                        return Ok(None);
                    };
                    if !matches!(names.instructions().last(), Some(instruction) if instruction.opcode() == 0xac && matches!(self.operations.get(instruction.bci()), Some(Operation::Return)))
                    {
                        return Ok(None);
                    }
                    returns.push(block);
                    if returns.len() > 2 {
                        return Ok(None);
                    }
                }
                _ => return Ok(None),
            }
        }
        if tests.len() < 2 || tests.len() > MAX_TESTS || returns.len() != 2 {
            return Ok(None);
        }
        let mut ordered = tests.into_iter().zip(test_edges).collect::<Vec<_>>();
        ordered.sort_by_key(|((block, _), _)| block.bci());
        let (tests, test_edges): (Vec<_>, Vec<_>) = ordered.into_iter().unzip();
        gateways.sort_by_key(|(block, _)| block.bci());
        if tests[0] != (outer.clone(), outer_bci) {
            return Ok(None);
        }
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(self.canonical.edges().len().saturating_mul(seen.len()))
                .unwrap_or(u64::MAX),
            Some(outer_bci),
        )?;
        let mut participants = tests
            .iter()
            .skip(1)
            .map(|(block, _)| block.clone())
            .collect::<Vec<_>>();
        participants.extend(gateways.iter().map(|(block, _)| block.clone()));
        participants.extend(returns.iter().cloned());
        for block in &participants {
            let node = self.view.index_of(block).expect("discovered node");
            if self.visited.contains(&node) {
                return Ok(None);
            }
            let mut expected = tests
                .iter()
                .zip(&test_edges)
                .filter(|(_, (fallthrough, taken))| fallthrough == block || taken == block)
                .map(|((source, _), _)| self.view.index_of(source).expect("discovered test"))
                .collect::<Vec<_>>();
            expected.extend(
                gateways
                    .iter()
                    .filter(|(_, target)| target == block)
                    .map(|(source, _)| self.view.index_of(source).expect("discovered gateway")),
            );
            if expected.is_empty() || !same_nodes(&self.view.predecessors(node), &expected) {
                return Ok(None);
            }
        }
        // No exceptional, subroutine or other edge may enter or leave the closure.
        for block in std::iter::once(outer).chain(participants.iter()) {
            if self.canonical.edges().iter().any(|edge| {
                if edge.from() != block && edge.to() != block {
                    return false;
                }
                match edge.kind() {
                    CanonicalEdgeKind::Normal => false,
                    CanonicalEdgeKind::Return { .. } => {
                        edge.from() != block || !returns.contains(block)
                    }
                    _ => true,
                }
            }) {
                return Ok(None);
            }
        }
        let Some(outer_node) = self.view.index_of(outer) else {
            return Ok(None);
        };
        let predecessor_count = self.view.predecessors(outer_node).len();
        if predecessor_count > 1 || (prefix.is_empty() && predecessor_count != 0) {
            return Ok(None);
        }
        if predecessor_count == 1
            && prefix.last().and_then(|block| self.view.index_of(block))
                != self.view.predecessors(outer_node).first().copied()
        {
            return Ok(None);
        }
        charge(
            self.budget,
            CountedBudgetDimension::IrItems,
            u64::try_from(participants.len().saturating_add(prefix.len())).unwrap_or(u64::MAX),
            Some(outer_bci),
        )?;
        for block in &participants {
            self.visited
                .insert(self.view.index_of(block).expect("discovered node"));
        }
        // This is only a candidate label for the two leaves. The builder separately requires
        // exact `iconst_1; ireturn` and `iconst_0; ireturn` pairs before emitting Java.
        returns.sort_by_key(|block| {
            self.ssa
                .block(block)
                .and_then(|names| names.instructions().first())
                .is_none_or(|instruction| instruction.opcode() != 0x04)
        });
        Ok(Some(Region::TwoExitReturn {
            prefix: prefix.to_vec(),
            tests,
            test_edges,
            gateways,
            true_return: returns[0].clone(),
            false_return: returns[1].clone(),
        }))
    }

    /// Claims the bounded acyclic test graph whose two producers meet at one block with a
    /// field write. This establishes physical ownership only; the value and its consumer
    /// remain unproved and the builder quotes the node whole. Decoded edges and exact predecessor
    /// sets can admit a shared test; the builder still proves the value and its field use.
    /// A candidate at the head of one proven protected try is also claimed when a takeable
    /// exception edge leaves it, so the complete candidate stays one exception-edge refusal.
    fn short_circuit_value(
        &mut self,
        prefix: &[CanonicalBlockId],
        outer_branch: &CanonicalBlockId,
        outer_branch_bci: u32,
        frame: &Frame,
    ) -> Result<Option<(Region, Option<CanonicalBlockId>)>, StopReason> {
        // Method-level candidates keep their original restriction. The only protected-range
        // extension is a candidate that starts at the exact entry of one try body: its boundary
        // and exception-table row then prove the complete local ownership without widening an
        // enclosing if, loop, switch arm, or another try.
        if frame.scope.is_some()
            || frame.case_entries.is_some()
            || frame.own_loop.is_some()
            || frame.loop_exit.is_some()
            || !frame.loop_targets.is_empty()
            || frame.switch_join.is_some()
        {
            return Ok(None);
        }
        let protected_try = match (frame.own_try, frame.boundary) {
            (None, None) => None,
            (Some(start_node), Some(boundary_node)) => {
                if self.view.index_of(outer_branch) != Some(start_node) {
                    return Ok(None);
                }
                let (Some(start), Some(boundary)) =
                    (self.view.id_of(start_node), self.view.id_of(boundary_node))
                else {
                    return Ok(None);
                };
                let Some(start_last_bci) = self.terminal_bci(start) else {
                    return Ok(None);
                };
                let mut ranges = self
                    .handlers
                    .iter()
                    .filter(|row| {
                        row.catch_type_index.is_some()
                            && row.start_bci >= start.bci()
                            && row.start_bci <= start_last_bci
                    })
                    .map(|row| (row.start_bci, row.end_bci))
                    .collect::<Vec<_>>();
                ranges.sort_unstable();
                ranges.dedup();
                let [(range_start, range_end)] = ranges.as_slice() else {
                    return Ok(None);
                };
                // This bounded extension handles one named handler for one range. Multiple rows,
                // nested/overlapping ranges, and a boundary that does not exactly meet the range
                // are ambiguous ownership and stay on the ordinary conservative path.
                let matching_rows = self
                    .handlers
                    .iter()
                    .filter(|row| row.start_bci == *range_start && row.end_bci == *range_end)
                    .collect::<Vec<_>>();
                let [handler_row] = matching_rows.as_slice() else {
                    return Ok(None);
                };
                if handler_row.catch_type_index.is_none()
                    || boundary.bci() < *range_end
                    || self.handlers.iter().any(|row| {
                        row.ordinal != handler_row.ordinal
                            && row.start_bci < *range_end
                            && *range_start < row.end_bci
                    })
                {
                    return Ok(None);
                }
                Some((
                    *range_start,
                    *range_end,
                    handler_row.ordinal,
                    handler_row.handler_bci,
                ))
            }
            _ => return Ok(None),
        };
        const MAX_SHORT_CIRCUIT_TESTS: usize = 24;
        poll(self.budget, Some(outer_branch_bci))?;
        // Collect comparison nodes, bounded pure transfers and two terminal value producers.
        // Every discovered normal edge advances; exact physical predecessors are checked below.
        let mut pending = vec![outer_branch.clone()];
        let mut seen = BTreeSet::new();
        let mut tests = Vec::new();
        let mut test_edges = Vec::new();
        let mut gateways = Vec::new();
        let mut transfer_targets = None;
        let mut producers = Vec::new();
        while let Some(block) = pending.pop() {
            poll(self.budget, Some(block.bci()))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(block.bci()),
            )?;
            let Some(node) = self.view.index_of(&block) else {
                return Ok(None);
            };
            if !seen.insert(node) {
                continue;
            }
            if seen.len() > MAX_SHORT_CIRCUIT_TESTS * 2 + 2 {
                return Ok(None);
            }
            let successors = self.view.successor_ids(&block);
            if successors.len() == 2 {
                let Some(branch_bci) = self.terminal_bci(&block) else {
                    return Ok(None);
                };
                let Some((_, target)) = self
                    .operations
                    .get(branch_bci)
                    .and_then(Operation::comparison)
                else {
                    return Ok(None);
                };
                let Some((fallthrough, taken)) = self.split_arms(&successors, target) else {
                    return Ok(None);
                };
                if successors.iter().any(|next| next.bci() <= block.bci()) {
                    return Ok(None);
                }
                tests.push((block, branch_bci));
                test_edges.push((fallthrough.clone(), taken.clone()));
                pending.extend([fallthrough, taken]);
            } else if successors.len() == 1 {
                let Some(names) = self.ssa.block(&block) else {
                    return Ok(None);
                };
                let Some(first) = names.instructions().first() else {
                    return Ok(None);
                };
                if matches!(self.operations.get(first.bci()), Some(Operation::Push(_))) {
                    producers.push(block);
                    if producers.len() > 2 {
                        return Ok(None);
                    }
                } else {
                    let [transfer] = names.instructions() else {
                        return Ok(None);
                    };
                    let successor = &successors[0];
                    if !matches!(transfer.opcode(), 0xa7 | 0xc8)
                        || !matches!(
                            self.operations.get(transfer.bci()),
                            Some(Operation::Transfer)
                        )
                        || !transfer.reads().is_empty()
                        || !transfer.writes().is_empty()
                        || successor.bci() <= block.bci()
                        || successor.path() != block.path()
                        || self.excluded_edge_nodes.contains(&block)
                    {
                        return Ok(None);
                    }
                    if transfer_targets.is_none() {
                        charge(
                            self.budget,
                            CountedBudgetDimension::AnalysisSteps,
                            u64::try_from(self.code.instructions.len()).unwrap_or(u64::MAX),
                            Some(transfer.bci()),
                        )?;
                        let Ok(targets) = self.code.control_flow_targets() else {
                            return Ok(None);
                        };
                        transfer_targets = Some(targets);
                    }
                    let targets = transfer_targets.as_ref().expect("decoded transfer targets");
                    if targets
                        .iter()
                        .filter(|target| target.instruction_bci == transfer.bci())
                        .filter(|target| {
                            matches!(
                                target.kind,
                                jarde_reader::classfile::ControlFlowTargetKind::Branch { .. }
                            )
                        })
                        .map(|target| target.target_bci)
                        .collect::<Vec<_>>()
                        != [successor.bci()]
                    {
                        return Ok(None);
                    }
                    gateways.push((block, successor.clone()));
                    if gateways.len() > MAX_SHORT_CIRCUIT_TESTS {
                        return Ok(None);
                    }
                    pending.push(successor.clone());
                }
            } else {
                return Ok(None);
            }
        }
        if tests.len() < 2 || tests.len() > MAX_SHORT_CIRCUIT_TESTS || producers.len() != 2 {
            return Ok(None);
        }
        let mut ordered = tests.into_iter().zip(test_edges).collect::<Vec<_>>();
        ordered.sort_by_key(|((block, _), _)| block.bci());
        let (tests, test_edges): (Vec<_>, Vec<_>) = ordered.into_iter().unzip();
        gateways.sort_by_key(|(block, _)| block.bci());
        if tests[0] != (outer_branch.clone(), outer_branch_bci) {
            return Ok(None);
        }
        let (last_fallthrough, last_taken) = test_edges.last().expect("at least two tests");
        if !producers.contains(last_fallthrough) || !producers.contains(last_taken) {
            return Ok(None);
        }
        let literal = |block: &CanonicalBlockId| {
            self.ssa
                .block(block)
                .and_then(|names| names.instructions().first())
                .and_then(|instruction| self.operations.get(instruction.bci()))
                .and_then(|operation| match operation {
                    Operation::Push(ConstantValue::Int(value)) => Some(*value),
                    _ => None,
                })
        };
        // Decode the leaf values before assigning polarity. The last comparison's taken
        // edge can be either 1 or 0 (notably for OR versus AND). A near-miss non-boolean
        // literal still retains one owner so the later value proof can quote it whole.
        let (true_producer, false_producer) = match (literal(last_fallthrough), literal(last_taken))
        {
            (Some(0), _) | (_, Some(1)) => (last_taken.clone(), last_fallthrough.clone()),
            _ => (last_fallthrough.clone(), last_taken.clone()),
        };
        let Some(consumer) = self.view.successor_ids(&true_producer).first().cloned() else {
            return Ok(None);
        };
        let Some(consumer_node) = self.view.index_of(&consumer) else {
            return Ok(None);
        };
        if self.view.successor_ids(&false_producer) != [consumer.clone()]
            || self.view.successors(consumer_node).len() > 1
        {
            return Ok(None);
        }
        let comparisons = tests.len().saturating_mul(tests.len()).saturating_mul(4);
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(comparisons).unwrap_or(u64::MAX),
            Some(outer_branch_bci),
        )?;
        let test_nodes = tests
            .iter()
            .map(|(block, _)| self.view.index_of(block).expect("collected node"))
            .collect::<Vec<_>>();
        let true_node = self
            .view
            .index_of(&true_producer)
            .expect("collected producer");
        let false_node = self
            .view
            .index_of(&false_producer)
            .expect("collected producer");
        let gateway_nodes = gateways
            .iter()
            .map(|(block, _)| self.view.index_of(block).expect("collected gateway"))
            .collect::<Vec<_>>();
        let mut participants = test_nodes[1..].to_vec();
        participants.extend(&gateway_nodes);
        participants.extend([true_node, false_node, consumer_node]);
        if participants.iter().any(|node| self.visited.contains(node))
            || participants.iter().copied().collect::<BTreeSet<_>>().len() != participants.len()
        {
            return Ok(None);
        }
        for node in test_nodes.iter().skip(1) {
            let mut expected = tests
                .iter()
                .zip(&test_edges)
                .filter(|(_, (fallthrough, taken))| {
                    self.view.index_of(fallthrough) == Some(*node)
                        || self.view.index_of(taken) == Some(*node)
                })
                .map(|((block, _), _)| self.view.index_of(block).expect("collected node"))
                .collect::<Vec<_>>();
            expected.extend(
                gateways
                    .iter()
                    .filter(|(_, successor)| self.view.index_of(successor) == Some(*node))
                    .map(|(gateway, _)| self.view.index_of(gateway).expect("collected gateway")),
            );
            if expected.is_empty() || !same_nodes(&self.view.predecessors(*node), &expected) {
                return Ok(None);
            }
        }
        for (gateway, successor) in &gateways {
            let node = self.view.index_of(gateway).expect("collected gateway");
            let mut expected = tests
                .iter()
                .zip(&test_edges)
                .filter(|(_, (fallthrough, taken))| fallthrough == gateway || taken == gateway)
                .map(|((block, _), _)| self.view.index_of(block).expect("collected node"))
                .collect::<Vec<_>>();
            expected.extend(
                gateways
                    .iter()
                    .filter(|(_, target)| target == gateway)
                    .map(|(prior, _)| self.view.index_of(prior).expect("collected gateway")),
            );
            if expected.len() != 1
                || !same_nodes(&self.view.predecessors(node), &expected)
                || self.view.successor_ids(gateway) != [successor.clone()]
                || self.canonical.edges().iter().any(|edge| {
                    (edge.from() == gateway || edge.to() == gateway)
                        && edge.kind() != CanonicalEdgeKind::Normal
                })
            {
                return Ok(None);
            }
        }
        for producer_node in [true_node, false_node] {
            let mut expected = tests
                .iter()
                .zip(&test_edges)
                .filter(|(_, (fallthrough, taken))| {
                    self.view.index_of(fallthrough) == Some(producer_node)
                        || self.view.index_of(taken) == Some(producer_node)
                })
                .map(|((block, _), _)| self.view.index_of(block).expect("collected node"))
                .collect::<Vec<_>>();
            expected.extend(
                gateways
                    .iter()
                    .filter(|(_, successor)| self.view.index_of(successor) == Some(producer_node))
                    .map(|(gateway, _)| self.view.index_of(gateway).expect("collected gateway")),
            );
            if expected.is_empty() || !same_nodes(&self.view.predecessors(producer_node), &expected)
            {
                return Ok(None);
            }
        }
        if !same_nodes(
            &self.view.predecessors(consumer_node),
            &[true_node, false_node],
        ) {
            return Ok(None);
        }
        let mut participant_ids = tests
            .iter()
            .skip(1)
            .map(|(block, _)| block.clone())
            .collect::<Vec<_>>();
        participant_ids.extend(gateways.iter().map(|(block, _)| block.clone()));
        participant_ids.extend([
            true_producer.clone(),
            false_producer.clone(),
            consumer.clone(),
        ]);
        let scan_items = participant_ids
            .iter()
            .filter_map(|block| self.ssa.block(block))
            .map(|block| block.instructions().len().saturating_add(1))
            .sum::<usize>()
            .saturating_add(prefix.len());
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(scan_items).unwrap_or(u64::MAX),
            Some(outer_branch_bci),
        )?;
        charge(
            self.budget,
            CountedBudgetDimension::IrItems,
            u64::try_from(participants.len().saturating_add(prefix.len())).unwrap_or(u64::MAX),
            Some(outer_branch_bci),
        )?;

        let Some(consumer_block) = self.ssa.block(&consumer) else {
            return Ok(None);
        };
        // Keep ownership broad enough to retain near-miss producers and consumers (including a
        // dup before a field write). The builder proves the exact 1/0 values, unique Phi read and
        // consumer type; this recognizer only needs an existing statement consumer anchor.
        let Some(consumer_bci) = consumer_block
            .instructions()
            .iter()
            .find_map(|instruction| {
                (matches!(
                    self.operations.get(instruction.bci()),
                    Some(Operation::Field {
                        access: crate::facts::FieldAccess::Write,
                        ..
                    })
                ) || (instruction.opcode() == 0xac
                    && matches!(
                        self.operations.get(instruction.bci()),
                        Some(Operation::Return)
                    ))
                    || matches!(
                        self.operations.get(instruction.bci()),
                        Some(Operation::Invoke(_))
                    )
                    || matches!(
                        self.operations.get(instruction.bci()),
                        Some(Operation::Store { .. })
                    )
                    || (instruction.opcode() == 0x54
                        && matches!(
                            self.operations.get(instruction.bci()),
                            Some(Operation::ArrayStore { .. })
                        )))
                .then_some(instruction.bci())
            })
        else {
            return Ok(None);
        };
        let Some(consumer_terminal) = consumer_block.instructions().last() else {
            return Ok(None);
        };
        if self
            .operations
            .get(consumer_terminal.bci())
            .is_some_and(|operation| {
                operation.comparison().is_some() || operation.switch().is_some()
            })
        {
            return Ok(None);
        }
        let next = self.view.successor_ids(&consumer).into_iter().next();

        let mut reason = FallbackReason::ShortCircuitValueUnproved {
            first_branch_bci: outer_branch_bci,
            last_branch_bci: tests.last().expect("at least two tests").1,
            consumer_bci,
        };
        let mut owned_blocks = prefix.to_vec();
        owned_blocks.extend(tests.iter().map(|(block, _)| block.clone()));
        owned_blocks.extend(gateways.iter().map(|(block, _)| block.clone()));
        owned_blocks.extend([
            true_producer.clone(),
            false_producer.clone(),
            consumer.clone(),
        ]);
        owned_blocks.sort_by_key(CanonicalBlockId::bci);
        owned_blocks.dedup();
        if let Some((range_start, range_end, handler_ordinal, handler_bci)) = protected_try {
            // Every claimed physical block must be wholly inside this try's declared protected
            // range. Its continuation must reach the try boundary directly or through
            // transfer-only blocks: try_level does not walk the returned continuation.
            let Some(boundary_node) = frame.boundary else {
                return Ok(None);
            };
            for block in &owned_blocks {
                let Some(node) = self.view.index_of(block) else {
                    return Ok(None);
                };
                let Some(names) = self.ssa.block(block) else {
                    return Ok(None);
                };
                if block.bci() < range_start
                    || node == boundary_node
                    || names.instructions().iter().any(|instruction| {
                        instruction.bci() < range_start
                            || (instruction.bci() >= range_end
                                && !matches!(
                                    self.operations.get(instruction.bci()),
                                    Some(Operation::Transfer)
                                ))
                    })
                {
                    return Ok(None);
                }
            }
            let Some(next) = next.as_ref() else {
                return Ok(None);
            };
            if self.view.index_of(next) != Some(boundary_node)
                && self.after_join(next).bci()
                    != self
                        .view
                        .id_of(boundary_node)
                        .map_or(u32::MAX, CanonicalBlockId::bci)
            {
                return Ok(None);
            }

            let Some(handler) = self
                .canonical
                .handler_rows()
                .iter()
                .find(|row| row.ordinal() == handler_ordinal)
                .and_then(|row| row.handler())
            else {
                return Ok(None);
            };
            if handler.bci() != handler_bci {
                return Ok(None);
            }
            let mut takeable_exception = None;
            for block in &owned_blocks {
                for edge in self
                    .canonical
                    .edges()
                    .iter()
                    .filter(|edge| edge.from() == block)
                {
                    match edge.kind() {
                        CanonicalEdgeKind::Exception {
                            handler_ordinal: edge_ordinal,
                        } => {
                            if edge_ordinal != handler_ordinal || edge.to() != handler {
                                return Ok(None);
                            }
                            if self.exception_edge_takeable(block, edge_ordinal) {
                                takeable_exception.get_or_insert((block.bci(), edge_ordinal));
                            }
                        }
                        CanonicalEdgeKind::Call { .. } => return Ok(None),
                        CanonicalEdgeKind::Normal | CanonicalEdgeKind::Return { .. } => {}
                    }
                }
            }
            let Some((block_bci, handler_ordinal)) = takeable_exception else {
                // This change only extends try ownership to preserve a real throwing edge. A
                // protected candidate with no takeable edge retains the prior method-level limit.
                return Ok(None);
            };
            reason = FallbackReason::ExceptionEdge {
                block_bci,
                handler_ordinal,
            };
        } else if owned_blocks
            .iter()
            .any(|block| self.excluded_edge_nodes.contains(block))
        {
            return Ok(None);
        }

        // Commit ownership only after the entire local shape passed. The outer block was inserted
        // by region_at_inner already; every other participant is inserted exactly once here.
        for node in participants {
            self.visited.insert(node);
        }
        Ok(Some((
            Region::ShortCircuitValue {
                prefix: prefix.to_vec(),
                tests,
                test_edges,
                gateways,
                true_producer,
                false_producer,
                consumer,
                consumer_bci,
                reason,
            },
            next,
        )))
    }

    /// Whether one successor of a two-successor branch is the **forward join** of that branch: the
    /// block both successors converge onto by forward edges, which the branch's own immediate
    /// post-dominator does not state.
    ///
    /// `javac --release 8 -g:none` writes `if (a > 0 && b > 0) return 1; return 0;` as
    ///
    /// ```text
    /// 0: iload_0
    /// 1: ifle 10
    /// 4: iload_1
    /// 5: ifle 10
    /// 8: iconst_1
    /// 9: ireturn
    /// 10: iconst_0
    /// 11: ireturn
    /// ```
    ///
    /// so block 10 is reached by the taken edge of both branches — and it is **not** the branch's
    /// immediate post-dominator, because the `return 1` path leaves the method before passing
    /// through it. The one-armed reading above (a successor that *is* the post-dominator) therefore
    /// never fires, the walk claimed the shared block from the inner branch first, and the second
    /// arrival at it was read as a loop ([`FallbackReason::Loop`]) — the `return 0` the bytecode
    /// states disappeared from the text. The join is a fact of the graph all the same, and this is
    /// the reading of it:
    ///
    /// * a **loop header** is never a join: the walk reads a loop as a region of its own before any
    ///   branch arm is examined, and a block a back edge enters is where a structure begins, not
    ///   where a branch's arms meet;
    /// * the **other successor must be a different block** (`other`): two edges onto one block state
    ///   no arm split at all;
    /// * the frame's **own boundary** is where the enclosing structure continues, so a successor
    ///   that *is* it ends this branch's structure without walking anything — the one-armed shape
    ///   read off the frame rather than the graph. This is what makes an inner `if` one-armed once
    ///   the join of the enclosing `if` is the arm's boundary: without it the inner branch would
    ///   walk the shared block a second time;
    /// * otherwise the **convergence** must be stated by the graph: every predecessor of the
    ///   successor is dominated by this branch (every way into it comes through this branch), every
    ///   edge into it is forward (the successor does not dominate the block the edge comes from,
    ///   which is what a back edge is), and at least one predecessor is a block *other* than this
    ///   branch — a successor whose only way in is this branch is the block this branch continues
    ///   at, not a convergence of two arms.
    ///
    /// A branch whose two successors both satisfy this states no arm of its own and keeps
    /// [`FallbackReason::ArmsDoNotMeet`] (see [`Walker::region_at_inner`]).
    fn forward_join(
        &self,
        branch: usize,
        successor: Option<usize>,
        other: Option<usize>,
        frame: &Frame,
    ) -> bool {
        let Some(join) = successor else {
            return false;
        };
        if other.is_none_or(|other| other == join) {
            return false;
        }
        if self.view.is_loop_header(join) {
            return false;
        }
        if frame.boundary == Some(join) {
            return true;
        }
        self.forward_join_predecessors(branch, join)
    }

    /// Whether every incoming edge to `join` comes from this branch's forward region, with at
    /// least one predecessor beyond the branch itself. Switches use this same ownership test after
    /// their N-way arm paths identify a candidate; unlike the binary-branch wrapper above, that
    /// candidate must also pass this check when it is an enclosing boundary.
    fn forward_join_predecessors(&self, branch: usize, join: usize) -> bool {
        if self.view.is_loop_header(join) {
            return false;
        }
        let predecessors = self.view.predecessors(join);
        if predecessors
            .iter()
            .all(|predecessor| *predecessor == branch)
        {
            return false;
        }
        predecessors.iter().all(|predecessor| {
            self.view.dominates(branch, *predecessor) && !self.view.dominates(join, *predecessor)
        })
    }

    /// The loop whose header this walk just entered.
    ///
    /// Two shapes are provable, and both put the test *inside* the loop statement: a header that
    /// tests (`while`/`for`), and a single latch that tests after the body (`do … while`). The
    /// graph facts that make them provable are checked before anything is built — one latch, one
    /// entry, no block of the body leaving the loop, and a test whose block holds no effect of its
    /// own — and every other loop is quoted with its own reason.
    fn loop_region(
        &mut self,
        header: &CanonicalBlockId,
        header_node: usize,
        frame: &Frame,
    ) -> Result<Run, StopReason> {
        let Some(loop_of) = self.view.loop_entered_at(header_node).cloned() else {
            let reason = FallbackReason::LoopShape {
                block_bci: header.bci(),
            };
            return Ok(gap(Vec::new(), vec![header.clone()], reason, None));
        };
        let blocks = loop_of.blocks().clone();
        if self
            .view
            .loop_is_irreducible(header_node, &self.catch_joins)
        {
            // Defensive: `recover` refuses a whole body whose graph is irreducible before the walk
            // starts, so a header that is still irreducible here is a payload this layer did not
            // expect to see — and it is reported, not guessed at.
            let bcis = blocks
                .iter()
                .filter_map(|node| self.view.id_of(*node).map(CanonicalBlockId::bci))
                .collect();
            let reason = FallbackReason::Irreducible { blocks: bcis };
            return Ok(gap(Vec::new(), vec![header.clone()], reason, None));
        }
        if let Some(region) = self.latch_test_chain(header, header_node, &blocks)? {
            return Ok(region);
        }
        if let Some(region) = self.header_tested_loop(header, header_node, &blocks, frame)? {
            return Ok(region);
        }
        if let Some(region) = self.latch_tested_loop(header, header_node, &blocks, frame)? {
            return Ok(region);
        }
        let reason = FallbackReason::LoopShape {
            block_bci: header.bci(),
        };
        Ok(gap(Vec::new(), vec![header.clone()], reason, None))
    }

    /// The `while`/`for` shape: the header's own branch tests and one of its arms leaves the loop.
    fn header_tested_loop(
        &mut self,
        header: &CanonicalBlockId,
        header_node: usize,
        blocks: &BTreeSet<usize>,
        frame: &Frame,
    ) -> Result<Option<Run>, StopReason> {
        match self.header_test_chain(header, header_node, blocks)? {
            Ok(Some(chain)) => {
                let test_nodes: BTreeSet<_> = chain
                    .tests
                    .iter()
                    .filter_map(|(test, _, _)| self.view.index_of(test))
                    .collect();
                let exit_node = self.view.index_of(&chain.exit);
                let exits = self.loop_exit_nodes(blocks);
                let mut all_exits: BTreeSet<usize> = frame
                    .loop_targets
                    .iter()
                    .flat_map(|target| target.exits.iter().copied())
                    .collect();
                all_exits.extend(
                    frame
                        .loop_targets
                        .iter()
                        .map(|target| target.continue_target),
                );
                all_exits.extend(exits.iter().copied());
                let transfer_sources = self.loop_transfer_sources(&all_exits);
                let body_frame = frame.loop_body(
                    blocks,
                    header_node,
                    header_node,
                    header_node,
                    exit_node,
                    exits,
                    &transfer_sources,
                );
                let (body, _) = self.loop_body_sequence(&chain.body, &body_frame, blocks)?;
                self.visited.extend(test_nodes.iter().copied());
                let mut expected = blocks.clone();
                for node in test_nodes {
                    expected.remove(&node);
                }
                if !self.covers(&expected) {
                    return Ok(Some(Self::loop_fallback(
                        header,
                        FallbackReason::LoopShape {
                            block_bci: header.bci(),
                        },
                        body,
                    )));
                }
                return Ok(Some((
                    vec![Region::Loop {
                        header: header.clone(),
                        tests: chain.tests,
                        test_operator: Some(chain.operator),
                        form: LoopForm::While,
                        for_header: None,
                        body,
                        exit: Some(chain.exit.clone()),
                    }],
                    Some(chain.exit),
                )));
            }
            Ok(None) => {}
            Err(reason) => {
                let owned = blocks
                    .iter()
                    .filter_map(|node| self.view.id_of(*node).cloned())
                    .collect();
                return Ok(Some(gap(Vec::new(), owned, reason, None)));
            }
        }
        let successors = self.view.successor_ids(header);
        if successors.len() != 2 {
            return Ok(None);
        }
        let inside = successors
            .iter()
            .find(|successor| {
                self.view
                    .index_of(successor)
                    .is_some_and(|node| node != header_node && blocks.contains(&node))
            })
            .cloned();
        let outside = successors
            .iter()
            .find(|successor| {
                self.view
                    .index_of(successor)
                    .is_some_and(|node| !blocks.contains(&node))
            })
            .cloned();
        let (Some(inside), Some(outside)) = (inside, outside) else {
            return Ok(None);
        };
        let Some(test_bci) = self.terminal_bci(header) else {
            return Ok(None);
        };
        let Some((_, target)) = self
            .operations
            .get(test_bci)
            .and_then(Operation::comparison)
        else {
            return Ok(None);
        };
        if let Some(reason) = self.leaving_edge(header)
            && !self.leaves_only_through_dead_edges(header)
        {
            return Ok(Some(gap(Vec::new(), vec![header.clone()], reason, None)));
        }
        if let Err(reason) = self.test_is_pure(header, test_bci) {
            return Ok(Some(gap(Vec::new(), vec![header.clone()], reason, None)));
        }
        // Which way through the test iterates: the branch's own target says it, and nothing else
        // can — the two successors are its arms, not its polarity.
        let continuation = if inside.bci() == target {
            Continuation::Taken
        } else {
            Continuation::FallThrough
        };
        let exit_node = self.view.index_of(&outside);
        let for_header = self.prove_for_header(header, header_node, blocks)?;
        let exits = self.loop_exit_nodes(blocks);
        let mut all_exits: BTreeSet<usize> = frame
            .loop_targets
            .iter()
            .flat_map(|target| target.exits.iter().copied())
            .collect();
        all_exits.extend(
            frame
                .loop_targets
                .iter()
                .map(|target| target.continue_target),
        );
        all_exits.extend(exits.iter().copied());
        if let Some(proof) = &for_header
            && let Some(update_node) = self.view.index_of(&proof.update_block)
        {
            all_exits.insert(update_node);
        }
        let transfer_sources = self.loop_transfer_sources(&all_exits);
        let body_frame = frame.loop_body(
            blocks,
            header_node,
            header_node,
            for_header
                .as_ref()
                .and_then(|proof| self.view.index_of(&proof.update_block))
                .unwrap_or(header_node),
            exit_node,
            exits,
            &transfer_sources,
        );
        let (body, _) = self.loop_body_sequence(&inside, &body_frame, blocks)?;
        self.visited.insert(header_node);
        let mut expected = blocks.clone();
        expected.remove(&header_node);
        if !self.covers(&expected) {
            // The loop's shape is refused, and every block the body's walk claimed goes into the
            // refusal with it: the body region is dropped here, so naming its blocks is the only
            // thing that keeps them in the artifact at all (see [`Self::loop_fallback`]).
            let reason = FallbackReason::LoopShape {
                block_bci: header.bci(),
            };
            return Ok(Some(Self::loop_fallback(header, reason, body)));
        }
        let run = vec![Region::Loop {
            header: header.clone(),
            tests: vec![(header.clone(), test_bci, continuation)],
            test_operator: None,
            form: LoopForm::While,
            for_header,
            body,
            exit: Some(outside.clone()),
        }];
        Ok(Some((run, Some(outside))))
    }

    /// Proves a homogeneous short-circuit chain of header tests. A candidate chain that fails any
    /// condition, effect or edge check is returned as a local loop refusal so the legacy
    /// single-test path cannot publish a loop that loses one of its exits.
    fn header_test_chain(
        &mut self,
        header: &CanonicalBlockId,
        header_node: usize,
        blocks: &BTreeSet<usize>,
    ) -> Result<Result<Option<HeaderTestChain>, FallbackReason>, StopReason> {
        let Some(first_bci) = self.terminal_bci(header) else {
            return Ok(Ok(None));
        };
        if !self
            .operations
            .get(first_bci)
            .is_some_and(|operation| operation.comparison().is_some())
        {
            return Ok(Ok(None));
        }
        let first_successors = self.view.successor_ids(header);
        if first_successors.len() != 2 {
            return Ok(Ok(None));
        }
        let external: Vec<_> = first_successors
            .iter()
            .filter(|successor| {
                self.view
                    .index_of(successor)
                    .is_some_and(|node| !blocks.contains(&node))
            })
            .cloned()
            .collect();
        if external.len() > 1 {
            return Ok(Err(FallbackReason::LoopShape {
                block_bci: header.bci(),
            }));
        }

        // Do not apply the header-test purity gate to a loop whose apparent "continuation" is
        // simply its body (including a one-block do-while whose header is also its latch). A
        // composite header proof exists only once the first continuing edge reaches a distinct,
        // predecessor-owned comparison that also has the same exact failure destination.
        if let Some(exit) = external.first() {
            let Some(next) = first_successors.iter().find(|successor| *successor != exit) else {
                return Ok(Err(FallbackReason::LoopShape {
                    block_bci: header.bci(),
                }));
            };
            let Some(next_node) = self.view.index_of(next) else {
                return Ok(Err(FallbackReason::LoopShape {
                    block_bci: header.bci(),
                }));
            };
            let Some(exit_node) = self.view.index_of(exit) else {
                return Ok(Err(FallbackReason::LoopShape {
                    block_bci: header.bci(),
                }));
            };
            if !self.header_test_candidate(header_node, next_node, blocks)
                || !self.view.successors(next_node).contains(&exit_node)
            {
                return Ok(Ok(None));
            }
        }

        let (operator, tests, body, exit) = if let Some(exit) = external.first() {
            let mut tests = Vec::new();
            let mut current = header.clone();
            let mut current_node = header_node;
            let mut seen = BTreeSet::new();
            let body = loop {
                if !seen.insert(current_node) {
                    return Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }));
                }
                let (test_bci, target, successors) =
                    match self.proved_header_test(header, &current, current_node)? {
                        Ok(facts) => facts,
                        Err(reason) => return Ok(Err(reason)),
                    };
                if successors.len() != 2 || !successors.contains(exit) {
                    return Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }));
                }
                let Some(next) = successors
                    .iter()
                    .find(|successor| *successor != exit)
                    .cloned()
                else {
                    return Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }));
                };
                let Some(target_node) = successors
                    .iter()
                    .find(|successor| successor.bci() == target)
                else {
                    return Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }));
                };
                tests.push((
                    current.clone(),
                    test_bci,
                    if target_node == &next {
                        Continuation::Taken
                    } else {
                        Continuation::FallThrough
                    },
                ));
                let Some(next_node) = self.view.index_of(&next) else {
                    return Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }));
                };
                if self.header_test_candidate(current_node, next_node, blocks)
                    && self.view.successors(next_node).contains(
                        &self
                            .view
                            .index_of(exit)
                            .expect("the exact loop exit is a node"),
                    )
                {
                    current = next;
                    current_node = next_node;
                } else {
                    break next;
                }
            };
            let body_node = self
                .view
                .index_of(&body)
                .expect("loop body is a normal-flow node");
            let expected: BTreeSet<_> = tests
                .iter()
                .filter_map(|(test, _, _)| self.view.index_of(test))
                .filter(|node| self.view.successors(*node).contains(&body_node))
                .collect();
            if expected != self.view.predecessors(body_node).into_iter().collect() {
                return Ok(Err(FallbackReason::LoopShape {
                    block_bci: header.bci(),
                }));
            }
            (crate::ast::BinaryOp::LogicalAnd, tests, body, exit.clone())
        } else {
            let candidates: Vec<_> = first_successors
                .iter()
                .filter_map(|successor| self.view.index_of(successor))
                .filter(|node| self.header_test_candidate(header_node, *node, blocks))
                .collect();
            let [first_next] = candidates.as_slice() else {
                return if candidates.is_empty() {
                    Ok(Ok(None))
                } else {
                    Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }))
                };
            };
            let (_, first_target, _) = match self.proved_header_test(header, header, header_node)? {
                Ok(facts) => facts,
                Err(reason) => return Ok(Err(reason)),
            };
            let first_next = *first_next;
            let first_next_id = self.view.id_of(first_next).expect("candidate id exists");
            let Some(body) = first_successors
                .iter()
                .find(|successor| self.view.index_of(successor) != Some(first_next))
                .cloned()
            else {
                return Ok(Err(FallbackReason::LoopShape {
                    block_bci: header.bci(),
                }));
            };
            let mut tests = vec![(header.clone(), first_bci, {
                if first_successors
                    .iter()
                    .find(|successor| successor.bci() == first_target)
                    == Some(&body)
                {
                    Continuation::Taken
                } else {
                    Continuation::FallThrough
                }
            })];
            let mut current = first_next_id.clone();
            let mut current_node = first_next;
            let mut seen = BTreeSet::from([header_node]);
            let exit = loop {
                if !seen.insert(current_node) {
                    return Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }));
                }
                let (test_bci, target, successors) =
                    match self.proved_header_test(header, &current, current_node)? {
                        Ok(facts) => facts,
                        Err(reason) => return Ok(Err(reason)),
                    };
                if successors.len() != 2 || !successors.contains(&body) {
                    return Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }));
                }
                let Some(route) = successors
                    .iter()
                    .find(|successor| *successor != &body)
                    .cloned()
                else {
                    return Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }));
                };
                let Some(target_node) = successors
                    .iter()
                    .find(|successor| successor.bci() == target)
                else {
                    return Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }));
                };
                tests.push((
                    current.clone(),
                    test_bci,
                    if target_node == &body {
                        Continuation::Taken
                    } else {
                        Continuation::FallThrough
                    },
                ));
                let Some(route_node) = self.view.index_of(&route) else {
                    return Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }));
                };
                if !blocks.contains(&route_node) {
                    break route;
                }
                if !self.header_test_candidate(current_node, route_node, blocks) {
                    return Ok(Err(FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    }));
                }
                current = route;
                current_node = route_node;
            };
            let body_node = self
                .view
                .index_of(&body)
                .expect("loop body is a normal-flow node");
            let expected: BTreeSet<_> = tests
                .iter()
                .filter_map(|(test, _, _)| self.view.index_of(test))
                .filter(|node| self.view.successors(*node).contains(&body_node))
                .collect();
            if expected != self.view.predecessors(body_node).into_iter().collect() {
                return Ok(Err(FallbackReason::LoopShape {
                    block_bci: header.bci(),
                }));
            }
            (crate::ast::BinaryOp::LogicalOr, tests, body, exit)
        };
        if tests.len() < 2 {
            return Ok(Ok(None));
        }
        Ok(Ok(Some(HeaderTestChain {
            tests,
            operator,
            body,
            exit,
        })))
    }

    fn header_test_candidate(
        &self,
        predecessor: usize,
        candidate: usize,
        blocks: &BTreeSet<usize>,
    ) -> bool {
        if candidate == predecessor || !blocks.contains(&candidate) {
            return false;
        }
        let Some(id) = self.view.id_of(candidate) else {
            return false;
        };
        self.view.predecessors(candidate) == [predecessor]
            && self
                .terminal_bci(id)
                .and_then(|bci| self.operations.get(bci))
                .is_some_and(|operation| operation.comparison().is_some())
    }

    /// Shared evidence gate for one branch that the candidate chain claims as a loop test.
    /// `NormalFlowView` supplies only normal successors; takeable exception edges are rejected
    /// here before those successors can prove the test's loop-condition role.
    fn proved_header_test(
        &mut self,
        loop_header: &CanonicalBlockId,
        block: &CanonicalBlockId,
        node: usize,
    ) -> Result<Result<HeaderTestFacts, FallbackReason>, StopReason> {
        let Some(test_bci) = self.terminal_bci(block) else {
            return Ok(Err(FallbackReason::LoopShape {
                block_bci: loop_header.bci(),
            }));
        };
        poll(self.budget, Some(test_bci))?;
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(test_bci),
        )?;
        let Some((op, target)) = self
            .operations
            .get(test_bci)
            .and_then(Operation::comparison)
        else {
            return Ok(Err(FallbackReason::LoopShape {
                block_bci: loop_header.bci(),
            }));
        };
        if let Err(reason) = self.branch_arity_proved(block, test_bci, op) {
            return Ok(Err(reason));
        }
        if let Some(reason) = self.leaving_edge(block)
            && !self.leaves_only_through_dead_edges(block)
        {
            return Ok(Err(reason));
        }
        if let Err(reason) = self.test_is_pure(block, test_bci) {
            return Ok(Err(reason));
        }
        let successors = self.view.successor_ids(block);
        if successors.len() != 2 || !successors.iter().any(|successor| successor.bci() == target) {
            return Ok(Err(FallbackReason::LoopShape {
                block_bci: loop_header.bci(),
            }));
        }
        debug_assert_eq!(self.view.index_of(block), Some(node));
        Ok(Ok((test_bci, target, successors)))
    }

    /// Proves the narrow indexed-loop spelling before walking nested bodies. The update block is
    /// an actual CFG destination, so a nested `continue` may target it only after this proof has
    /// established that Java's `for` update executes there. Checking every incoming edge prevents
    /// a body effect before the update from being moved into the header with it.
    fn prove_for_header(
        &mut self,
        header: &CanonicalBlockId,
        header_node: usize,
        blocks: &BTreeSet<usize>,
    ) -> Result<Option<ForHeader>, StopReason> {
        let at = Some(header.bci());
        poll(self.budget, at)?;
        let items = self
            .ssa
            .blocks()
            .iter()
            .map(|block| block.instructions().len().saturating_add(1))
            .sum::<usize>()
            .saturating_add(self.ssa.phis().len())
            .saturating_add(self.canonical.edges().len().saturating_mul(3));
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(items).unwrap_or(u64::MAX),
            at,
        )?;
        self.for_header_candidate(header, header_node, blocks)
    }

    fn for_header_candidate(
        &mut self,
        header: &CanonicalBlockId,
        header_node: usize,
        blocks: &BTreeSet<usize>,
    ) -> Result<Option<ForHeader>, StopReason> {
        let Some((proof, preheader, invariants)) = (|| {
            let loop_fact = self.view.loop_entered_at(header_node)?;
            let latch = *loop_fact.latches().iter().next()?;
            if loop_fact.latches().len() != 1
                || latch == header_node
                || self.view.successors(latch) != [header_node]
            {
                return None;
            }
            let update_block = self.view.id_of(latch)?;
            let update_instructions = self.ssa.block(update_block)?.instructions();
            let (slot, update, step, induction_input) = if let Some(
                [induction, step, add, update, transfer],
            ) = update_instructions
                .len()
                .checked_sub(5)
                .and_then(|start| update_instructions.get(start..))
                && let Some(Operation::Load { slot }) = self.operations.get(induction.bci())
                && matches!(self.operations.get(step.bci()), Some(Operation::Load { slot: read }) if read != slot)
                && matches!(
                    self.operations.get(add.bci()),
                    Some(Operation::Arithmetic {
                        op: crate::facts::ArithmeticOp::Add
                    })
                )
                && matches!(self.operations.get(update.bci()), Some(Operation::Store { slot: target }) if target == slot)
                && matches!(
                    self.operations.get(transfer.bci()),
                    Some(Operation::Transfer)
                )
                && matches!(induction.opcode(), 0x15 | 0x1a..=0x1d)
                && matches!(step.opcode(), 0x15 | 0x1a..=0x1d)
                && add.opcode() == 0x60
                && matches!(update.opcode(), 0x36 | 0x3b..=0x3e)
            {
                let [(Slot::Local(read_slot), induction_input)] = induction.reads() else {
                    return None;
                };
                if read_slot != slot {
                    return None;
                }
                let induction_value = induction
                    .writes()
                    .iter()
                    .find_map(|(at, value)| matches!(at, Slot::Stack(_)).then_some(*value))?;
                let step_value = step
                    .writes()
                    .iter()
                    .find_map(|(at, value)| matches!(at, Slot::Stack(_)).then_some(*value))?;
                let sum_value = add
                    .writes()
                    .iter()
                    .find_map(|(at, value)| matches!(at, Slot::Stack(_)).then_some(*value))?;
                let add_inputs = add
                    .reads()
                    .iter()
                    .filter(|(at, _)| matches!(at, Slot::Stack(_)))
                    .map(|(_, value)| *value)
                    .collect::<Vec<_>>();
                if add_inputs.len() != 2
                    || !add_inputs.contains(&induction_value)
                    || !add_inputs.contains(&step_value)
                    || update
                        .reads()
                        .iter()
                        .filter(|(at, _)| matches!(at, Slot::Stack(_)))
                        .map(|(_, value)| *value)
                        .collect::<Vec<_>>()
                        != [sum_value]
                    || self.ssa.value(induction_value).uses().len() != 1
                    || self.ssa.value(step_value).uses().len() != 1
                    || self.ssa.value(sum_value).uses().len() != 1
                {
                    return None;
                }
                let Some(Operation::Load { slot: step_slot }) = self.operations.get(step.bci())
                else {
                    return None;
                };
                (*slot, update, Some(*step_slot), Some(*induction_input))
            } else {
                let update_tail =
                    update_instructions.get(update_instructions.len().checked_sub(2)?..)?;
                let [update, transfer] = update_tail else {
                    return None;
                };
                let Some(Operation::Increment { slot, .. }) = self.operations.get(update.bci())
                else {
                    return None;
                };
                if !matches!(
                    self.operations.get(transfer.bci()),
                    Some(Operation::Transfer)
                ) {
                    return None;
                }
                (*slot, update, None, None)
            };
            if update
                .reads()
                .iter()
                .filter(|(at, _)| *at == Slot::Local(slot))
                .count()
                != usize::from(step.is_none())
                || update
                    .writes()
                    .iter()
                    .filter(|(at, _)| *at == Slot::Local(slot))
                    .count()
                    != 1
            {
                return None;
            }
            let in_edges = self.view.predecessors(latch);
            if in_edges.is_empty() || in_edges.iter().any(|source| !blocks.contains(source)) {
                return None;
            }
            // If the latch also holds body effects, an early edge to its entry must execute those
            // effects before the update. A Java `continue` would skip them, so only the plain
            // header-to-body route may use such a shared block.
            if update_instructions.len() > if step.is_some() { 5 } else { 2 }
                && in_edges != [header_node]
            {
                return None;
            }

            let outside: Vec<_> = self
                .view
                .predecessors(header_node)
                .into_iter()
                .filter(|source| !blocks.contains(source))
                .collect();
            let [preheader] = outside.as_slice() else {
                return None;
            };
            if self.view.successors(*preheader) != [header_node] {
                return None;
            }
            let preheader_block = self.view.id_of(*preheader)?;
            let initial_instructions = self.ssa.block(preheader_block)?.instructions();
            let tail = initial_instructions.get(initial_instructions.len().checked_sub(2)?..)?;
            let [producer, initial] = tail else {
                return None;
            };
            if !matches!(
                self.operations.get(producer.bci()),
                Some(Operation::Push(crate::facts::ConstantValue::Int(_)))
            ) || !matches!(self.operations.get(initial.bci()), Some(Operation::Store { slot: target }) if *target == slot)
            {
                return None;
            }
            let init_value = initial
                .writes()
                .iter()
                .find_map(|(at, value)| (*at == Slot::Local(slot)).then_some(*value))?;
            let update_value = update
                .writes()
                .iter()
                .find_map(|(at, value)| (*at == Slot::Local(slot)).then_some(*value))?;
            let stack_input = initial
                .reads()
                .iter()
                .find_map(|(at, value)| matches!(at, Slot::Stack(_)).then_some(*value))?;
            if !matches!(self.ssa.value(stack_input).def(), Definition::Instruction { bci, .. } if *bci == producer.bci())
            {
                return None;
            }

            let phi = self
                .ssa
                .phis()
                .iter()
                .find(|phi| phi.block() == header && phi.slot() == Slot::Local(slot))?;
            if phi.inputs().len() != 2
                || !phi.inputs().contains(&PhiInput::Value(init_value))
                || !phi.inputs().contains(&PhiInput::Value(update_value))
                || self.ssa.value(init_value).uses().len() != 1
                || self.ssa.value(update_value).uses().len() != 1
            {
                return None;
            }
            // The first load of an add/store latch must read this header's induction value,
            // not another merged local that happens to occupy the same slot.
            if induction_input.is_some_and(|value| value != phi.value()) {
                return None;
            }

            let test = self.ssa.block(header)?.instructions();
            let mut reads_induction = false;
            let mut invariants = BTreeSet::new();
            if let Some(step) = step {
                invariants.insert(step);
            }
            for instruction in test {
                match self.operations.get(instruction.bci()) {
                    Some(Operation::Load { slot: read }) if *read == slot => {
                        reads_induction = true;
                        if !instruction
                            .reads()
                            .iter()
                            .any(|(at, value)| *at == Slot::Local(slot) && *value == phi.value())
                        {
                            return None;
                        }
                    }
                    Some(Operation::Load { slot: read }) => {
                        invariants.insert(*read);
                    }
                    Some(
                        Operation::Push(_)
                        | Operation::Arithmetic { .. }
                        | Operation::Comparison { .. },
                    ) => {}
                    _ => return None,
                }
            }
            if !reads_induction {
                return None;
            }
            Some((
                ForHeader {
                    init_bci: initial.bci(),
                    update_bci: update.bci(),
                    update_block: update_block.clone(),
                    slot,
                },
                *preheader,
                invariants,
            ))
        })() else {
            return Ok(None);
        };
        for names in self.ssa.blocks() {
            poll(self.budget, Some(names.block().bci()))?;
            let Some(node) = self.view.index_of(names.block()) else {
                return Ok(None);
            };
            for instruction in names.instructions() {
                if blocks.contains(&node)
                    && instruction.bci() != proof.update_bci
                    && instruction
                        .writes()
                        .iter()
                        .any(|(at, _)| *at == Slot::Local(proof.slot))
                {
                    return Ok(None);
                }
                if blocks.contains(&node)
                    && instruction.writes().iter().any(
                        |(at, _)| matches!(at, Slot::Local(index) if invariants.contains(index)),
                    )
                {
                    return Ok(None);
                }
                if !blocks.contains(&node)
                    && node != preheader
                    && instruction
                        .reads()
                        .iter()
                        .chain(instruction.writes())
                        .any(|(at, _)| *at == Slot::Local(proof.slot))
                {
                    return Ok(None);
                }
            }
        }
        Ok(Some(proof))
    }

    /// The run a refused loop leaves behind: one refusal naming the loop's own block and everything
    /// the body's walk claimed, then the sibling quotes that walk left behind.
    ///
    /// A refused loop cannot be written — the statement is the loop — so the blocks its body proved
    /// have no place as statements, and the body's region is **dropped** by the caller. A dropped
    /// region is exactly what the exactly-once invariant forbids: those blocks were claimed, so the
    /// uncovered-blocks scan will not name them, and a block named by no region is a silently lost
    /// part of the body. The refusal therefore quotes the header and every block the body's run
    /// holds, each once, in the walk's own order; the quotes the body left behind stay quotes and
    /// are reported beside it ([`Run`]).
    fn loop_fallback(header: &CanonicalBlockId, reason: FallbackReason, body: Vec<Region>) -> Run {
        let blocks = gap_blocks(header, body.iter().flat_map(Region::blocks).cloned());
        let mut run = vec![Region::Fallback { blocks, reason }];
        run.extend(
            body.into_iter()
                .filter(|region| matches!(region, Region::Fallback { .. })),
        );
        (run, None)
    }

    /// Proves the narrow bottom-tested short-circuit shape where the first test shares the body
    /// entry block. That block is written once as the body; only its value-producing suffix belongs
    /// to the test, and later tests are ordinary pure latch blocks.
    fn latch_test_chain(
        &mut self,
        header: &CanonicalBlockId,
        header_node: usize,
        blocks: &BTreeSet<usize>,
    ) -> Result<Option<Run>, StopReason> {
        let Some((first_bci, _, _)) = self.first_latch_test_suffix(header) else {
            return Ok(None);
        };
        if self
            .operations
            .get(first_bci)
            .and_then(Operation::comparison)
            .is_none()
        {
            return Ok(None);
        }
        let first_successors = self.view.successor_ids(header);
        if first_successors.len() != 2 {
            return Ok(None);
        }

        // A branch straight back to the body entry makes this an OR chain. Otherwise the first
        // false edge must already name the one exit shared by every test, making it an AND chain.
        let operator = if first_successors
            .iter()
            .any(|successor| self.view.index_of(successor) == Some(header_node))
        {
            crate::ast::BinaryOp::LogicalOr
        } else {
            let outside: Vec<_> = first_successors
                .iter()
                .filter(|successor| {
                    self.view
                        .index_of(successor)
                        .is_some_and(|node| !blocks.contains(&node))
                })
                .cloned()
                .collect();
            let [_exit] = outside.as_slice() else {
                return Ok(None);
            };
            crate::ast::BinaryOp::LogicalAnd
        };

        let mut tests = Vec::new();
        let mut seen = BTreeSet::new();
        let mut current = header.clone();
        let mut current_node = header_node;
        let mut exit: Option<CanonicalBlockId> = None;
        loop {
            if !seen.insert(current_node) {
                return Ok(None);
            }
            let facts = if current_node == header_node {
                let Some(test_bci) = self.terminal_bci(&current) else {
                    return Ok(None);
                };
                poll(self.budget, Some(test_bci))?;
                charge(
                    self.budget,
                    CountedBudgetDimension::AnalysisSteps,
                    1,
                    Some(test_bci),
                )?;
                let Some((op, target)) = self
                    .operations
                    .get(test_bci)
                    .and_then(Operation::comparison)
                else {
                    return Ok(None);
                };
                if self.branch_arity_proved(&current, test_bci, op).is_err()
                    || (self.leaving_edge(&current).is_some()
                        && !self.leaves_only_through_dead_edges(&current))
                    || !self.latch_test_suffix_is_effect_free(&current, test_bci)
                {
                    return Ok(None);
                }
                (test_bci, target, self.view.successor_ids(&current))
            } else {
                match self.proved_header_test(header, &current, current_node)? {
                    Ok(facts) => facts,
                    Err(_) => return Ok(None),
                }
            };
            let (test_bci, target, successors) = facts;
            if successors.len() != 2 {
                return Ok(None);
            }
            let target_node = successors
                .iter()
                .find(|successor| successor.bci() == target);
            let Some(target_node) = target_node else {
                return Ok(None);
            };

            let branch_to_header = successors
                .iter()
                .find(|successor| self.view.index_of(successor) == Some(header_node));
            let (condition_route, next_route) = match operator {
                crate::ast::BinaryOp::LogicalAnd => {
                    // The condition's true edge advances through the chain; false exits.
                    let Some(exit_edge) = successors.iter().find(|successor| {
                        self.view
                            .index_of(successor)
                            .is_some_and(|node| !blocks.contains(&node))
                    }) else {
                        return Ok(None);
                    };
                    if exit.as_ref().is_some_and(|known| known != exit_edge) {
                        return Ok(None);
                    }
                    exit.get_or_insert_with(|| exit_edge.clone());
                    let Some(condition_route) = successors
                        .iter()
                        .find(|successor| *successor != exit_edge)
                        .cloned()
                    else {
                        return Ok(None);
                    };
                    let route_node = self.view.index_of(&condition_route);
                    if route_node.is_none_or(|node| !blocks.contains(&node)) {
                        return Ok(None);
                    }
                    (
                        condition_route.clone(),
                        (route_node != Some(header_node)).then_some(condition_route),
                    )
                }
                crate::ast::BinaryOp::LogicalOr => {
                    let Some(body_edge) = branch_to_header.cloned() else {
                        return Ok(None);
                    };
                    let Some(route_edge) = successors
                        .iter()
                        .find(|successor| *successor != &body_edge)
                        .cloned()
                    else {
                        return Ok(None);
                    };
                    if self
                        .view
                        .index_of(&route_edge)
                        .is_none_or(|node| !blocks.contains(&node))
                    {
                        if exit.as_ref().is_some_and(|known| known != &route_edge) {
                            return Ok(None);
                        }
                        exit = Some(route_edge.clone());
                    }
                    (
                        body_edge,
                        self.view
                            .index_of(&route_edge)
                            .filter(|node| blocks.contains(node))
                            .map(|_| route_edge),
                    )
                }
                _ => return Ok(None),
            };
            let continuation = if target_node == &condition_route {
                Continuation::Taken
            } else {
                Continuation::FallThrough
            };
            tests.push((current.clone(), test_bci, continuation));

            let Some(next_route) = next_route else {
                break;
            };
            let Some(next_node) = self.view.index_of(&next_route) else {
                return Ok(None);
            };
            if !blocks.contains(&next_node)
                || self.view.predecessors(next_node) != [current_node]
                || self
                    .terminal_bci(&next_route)
                    .and_then(|bci| self.operations.get(bci))
                    .and_then(Operation::comparison)
                    .is_none()
            {
                return Ok(None);
            }
            current = next_route;
            current_node = next_node;
        }
        if tests.len() < 2 {
            return Ok(None);
        }
        let Some(exit) = exit else {
            return Ok(None);
        };
        let test_nodes: BTreeSet<_> = tests
            .iter()
            .filter_map(|(test, _, _)| self.view.index_of(test))
            .collect();
        if test_nodes != *blocks
            || self
                .view
                .predecessors(header_node)
                .into_iter()
                .any(|source| blocks.contains(&source) && !test_nodes.contains(&source))
            || self
                .view
                .predecessors(header_node)
                .into_iter()
                .filter(|source| !blocks.contains(source))
                .count()
                != 1
            || self
                .view
                .loop_entered_at(header_node)
                .is_none_or(|loop_fact| {
                    loop_fact
                        .latches()
                        .iter()
                        .any(|latch| !test_nodes.contains(latch))
                        || loop_fact.latches().is_empty()
                })
        {
            return Ok(None);
        }
        // The initial edge into a do-while body is its one legal non-test predecessor. Every later
        // test block was already checked above for exactly one predecessor: the previous test.
        let internal_header_predecessors: BTreeSet<_> = self
            .view
            .predecessors(header_node)
            .into_iter()
            .filter(|source| blocks.contains(source))
            .collect();
        let expected_header_predecessors: BTreeSet<_> = tests
            .iter()
            .filter_map(|(test, _, _)| self.view.index_of(test))
            .filter(|node| self.view.successors(*node).contains(&header_node))
            .collect();
        if internal_header_predecessors != expected_header_predecessors {
            return Ok(None);
        }

        self.visited.extend(test_nodes.iter().copied());
        let run = vec![Region::Loop {
            header: header.clone(),
            tests,
            test_operator: Some(operator),
            form: LoopForm::DoWhile,
            for_header: None,
            body: vec![Region::Straight {
                blocks: vec![header.clone()],
            }],
            exit: Some(exit.clone()),
        }];
        Ok(Some((run, Some(exit))))
    }

    fn first_latch_test_suffix(
        &self,
        block: &CanonicalBlockId,
    ) -> Option<(u32, usize, BTreeSet<u32>)> {
        let test_bci = self.terminal_bci(block)?;
        self.operations.get(test_bci)?.comparison()?;
        let names = self.ssa.block(block)?;
        let condition_bcis: BTreeSet<_> = self
            .condition_value_bcis(block, test_bci, names)
            .into_iter()
            .filter(|bci| {
                matches!(
                    self.operations.get(*bci),
                    Some(
                        Operation::Push(_)
                            | Operation::Load { .. }
                            | Operation::Arithmetic { .. }
                            | Operation::Negate
                            | Operation::NumericComparison { .. }
                            | Operation::Invoke(_)
                            | Operation::Field {
                                access: crate::facts::FieldAccess::Read,
                                ..
                            }
                    )
                )
            })
            .collect();
        let first_condition = names
            .instructions()
            .iter()
            .position(|instruction| condition_bcis.contains(&instruction.bci()))?;
        Some((test_bci, first_condition, condition_bcis))
    }

    fn latch_test_suffix_is_effect_free(&self, block: &CanonicalBlockId, test_bci: u32) -> bool {
        let Some((_, first_condition, condition_bcis)) = self.first_latch_test_suffix(block) else {
            return false;
        };
        let Some(names) = self.ssa.block(block) else {
            return false;
        };
        if names.instructions()[first_condition..]
            .iter()
            .any(|instruction| {
                instruction.bci() != test_bci && !condition_bcis.contains(&instruction.bci())
            })
        {
            return false;
        }
        names.instructions()[first_condition..]
            .iter()
            .filter(|instruction| instruction.bci() != test_bci)
            .all(|instruction| {
                matches!(
                    self.operations.get(instruction.bci()),
                    Some(
                        Operation::Push(_)
                            | Operation::Load { .. }
                            | Operation::Arithmetic { .. }
                            | Operation::Negate
                            | Operation::NumericComparison { .. }
                            | Operation::Invoke(_)
                            | Operation::Field {
                                access: crate::facts::FieldAccess::Read,
                                ..
                            }
                    )
                )
            })
            && names.instructions()[..first_condition]
                .iter()
                .all(|instruction| {
                    matches!(
                        self.operations.get(instruction.bci()),
                        Some(Operation::Increment { .. })
                    )
                })
    }

    /// The `do … while` shape: one latch, whose own branch tests and jumps back to the header.
    fn latch_tested_loop(
        &mut self,
        header: &CanonicalBlockId,
        header_node: usize,
        blocks: &BTreeSet<usize>,
        frame: &Frame,
    ) -> Result<Option<Run>, StopReason> {
        let latches = self
            .view
            .loop_entered_at(header_node)
            .map(|loop_of| loop_of.latches().clone())
            .unwrap_or_default();
        if latches.len() != 1 {
            return Ok(None);
        }
        let latch_node = *latches.first().expect("one latch");
        let Some(latch) = self.view.id_of(latch_node).cloned() else {
            return Ok(None);
        };
        // Two tests in one loop (the header's and the latch's) is not a shape this subset proves.
        // A block that tests *itself* is not that case: there the header and the latch are one block
        // and its single branch is the loop's whole test.
        if latch_node != header_node
            && let Some(test_bci) = self.terminal_bci(header)
            && self.operations.get(test_bci).is_some_and(|operation| {
                operation.comparison().is_some() || operation.switch().is_some()
            })
        {
            return Ok(None);
        }
        let successors = self.view.successor_ids(&latch);
        if successors.len() != 2 {
            return Ok(None);
        }
        let exit = successors
            .iter()
            .find(|successor| {
                self.view
                    .index_of(successor)
                    .is_some_and(|node| node != header_node && !blocks.contains(&node))
            })
            .cloned();
        let Some(exit) = exit else {
            return Ok(None);
        };
        let Some(test_bci) = self.terminal_bci(&latch) else {
            return Ok(None);
        };
        let Some((_, target)) = self
            .operations
            .get(test_bci)
            .and_then(Operation::comparison)
        else {
            return Ok(None);
        };
        // A block that tests *itself* — the whole loop is one block with a back edge to its own
        // start — is the same `do … while` shape with an empty body region: the block's own
        // statements run before its test on every iteration, and that is exactly what the loop
        // statement writes. Nothing has to move, so the test block's statements are the body here
        // rather than a reason to refuse.
        if latch_node == header_node {
            self.visited.insert(header_node);
            return Ok(Some(one(
                Region::Loop {
                    header: header.clone(),
                    tests: vec![(
                        header.clone(),
                        test_bci,
                        if target == header.bci() {
                            Continuation::Taken
                        } else {
                            Continuation::FallThrough
                        },
                    )],
                    test_operator: None,
                    form: LoopForm::DoWhile,
                    for_header: None,
                    body: vec![Region::Straight {
                        blocks: vec![header.clone()],
                    }],
                    exit: Some(exit.clone()),
                },
                Some(exit),
            )));
        }
        for block in [header, &latch] {
            if let Some(reason) = self.leaving_edge(block)
                && !self.leaves_only_through_dead_edges(block)
            {
                return Ok(Some(gap(Vec::new(), vec![header.clone()], reason, None)));
            }
        }
        if let Err(reason) = self.test_is_pure(&latch, test_bci) {
            return Ok(Some(gap(Vec::new(), vec![header.clone()], reason, None)));
        }
        let continuation = if target == header.bci() {
            Continuation::Taken
        } else {
            Continuation::FallThrough
        };
        let exit_node = self.view.index_of(&exit);
        let exits = self.loop_exit_nodes(blocks);
        let mut all_exits: BTreeSet<usize> = frame
            .loop_targets
            .iter()
            .flat_map(|target| target.exits.iter().copied())
            .collect();
        all_exits.extend(
            frame
                .loop_targets
                .iter()
                .map(|target| target.continue_target),
        );
        all_exits.extend(exits.iter().copied());
        let transfer_sources = self.loop_transfer_sources(&all_exits);
        let body_frame = frame.loop_body(
            blocks,
            latch_node,
            header_node,
            latch_node,
            exit_node,
            exits,
            &transfer_sources,
        );
        let (body, _) = self.loop_body_sequence(header, &body_frame, blocks)?;
        self.visited.insert(latch_node);
        let mut expected = blocks.clone();
        expected.remove(&latch_node);
        if !self.covers(&expected) {
            let reason = FallbackReason::LoopShape {
                block_bci: header.bci(),
            };
            return Ok(Some(Self::loop_fallback(header, reason, body)));
        }
        let run = vec![Region::Loop {
            header: header.clone(),
            tests: vec![(latch, test_bci, continuation)],
            test_operator: None,
            form: LoopForm::DoWhile,
            for_header: None,
            body,
            exit: Some(exit.clone()),
        }];
        Ok(Some((run, Some(exit))))
    }

    /// Collect every sequential region of one loop iteration until it reaches the loop's own
    /// boundary or leaves its natural-loop scope. Nested structures report their unclaimed
    /// successor through `Run`; that successor is still part of this body when the CFG says so.
    fn loop_body_sequence(
        &mut self,
        start: &CanonicalBlockId,
        frame: &Frame,
        blocks: &BTreeSet<usize>,
    ) -> Result<(Vec<Region>, Option<CanonicalBlockId>), StopReason> {
        let mut body = Vec::new();
        let mut current = Some(start.clone());
        let mut starts = BTreeSet::new();
        while let Some(block) = current.take() {
            let Some(node) = self.view.index_of(&block) else {
                return Ok((body, Some(block)));
            };
            if frame.stops_at(node) || !blocks.contains(&node) {
                return Ok((body, Some(block)));
            }
            if !starts.insert(node) {
                return Ok((body, Some(block)));
            }
            let (regions, next) = self.region_at(&block, frame)?;
            body.extend(regions);
            current = next;
        }
        Ok((body, None))
    }

    /// Every real normal-flow destination outside this natural loop.
    fn loop_exit_nodes(&self, blocks: &BTreeSet<usize>) -> BTreeSet<usize> {
        blocks
            .iter()
            .flat_map(|node| self.view.successors(*node))
            .filter(|successor| !blocks.contains(successor))
            .collect()
    }

    /// Exact normal-flow predecessors that consist of a single edge to an enclosing loop target.
    fn loop_transfer_sources(&self, targets: &BTreeSet<usize>) -> BTreeSet<usize> {
        (0..self.view.len())
            .filter(|source| {
                let successors = self.view.successors(*source);
                successors.len() == 1 && targets.contains(&successors[0])
            })
            .collect()
    }

    /// Whether every expected block of a loop was walked.
    ///
    /// A loop is presented only when the walk reached every block that iterates: a block of the
    /// loop the body never claimed would be a block whose execution the presentation dropped.
    fn covers(&self, expected: &BTreeSet<usize>) -> bool {
        expected.iter().all(|node| self.visited.contains(node))
    }

    /// Find the one join of switch paths that stay in the current loop. An arm may instead end
    /// at an exact, already proved loop break target. This is a bounded proof over this switch's
    /// own successor paths; it does not claim blocks or change the normal-flow graph.
    fn switch_loop_join(
        &mut self,
        branch: usize,
        successors: &[CanonicalBlockId],
        frame: &Frame,
        at: u32,
    ) -> Result<Option<usize>, StopReason> {
        let Some(scope) = &frame.scope else {
            return Ok(None);
        };
        let entries: BTreeSet<usize> = successors
            .iter()
            .filter_map(|id| self.view.index_of(id))
            .collect();
        if entries.len() != successors.len() {
            return Ok(None);
        }
        let exits: BTreeSet<usize> = frame
            .loop_targets
            .iter()
            .filter_map(|target| target.break_target)
            .collect();
        let mut proved = Vec::new();
        for candidate in scope {
            if *candidate == branch
                || entries.contains(candidate)
                || frame.boundary == Some(*candidate)
                || self.view.predecessors(*candidate).len() < 2
                || !self.view.dominates(branch, *candidate)
            {
                continue;
            }
            let mut normal_arms = 0;
            let mut valid = true;
            for entry in &entries {
                let mut seen = BTreeSet::new();
                let mut work = vec![*entry];
                let mut reaches_join = false;
                while let Some(current) = work.pop() {
                    poll(self.budget, Some(at))?;
                    charge(
                        self.budget,
                        CountedBudgetDimension::AnalysisSteps,
                        1,
                        Some(at),
                    )?;
                    if current == *candidate {
                        reaches_join = true;
                        continue;
                    }
                    if exits.contains(&current) {
                        continue;
                    }
                    if !scope.contains(&current)
                        || frame.boundary == Some(current)
                        || (current != *entry && entries.contains(&current))
                        || !seen.insert(current)
                    {
                        valid = false;
                        break;
                    }
                    let next = self.view.successors(current);
                    if next.is_empty() {
                        valid = false;
                        break;
                    }
                    work.extend(next);
                }
                if !valid {
                    break;
                }
                normal_arms += usize::from(reaches_join);
            }
            if valid && normal_arms >= 2 {
                proved.push(*candidate);
            }
        }
        Ok(if proved.len() == 1 {
            proved.first().copied()
        } else {
            None
        })
    }

    /// The region of one block whose terminal instruction is a decoded `switch`.
    ///
    /// One group per distinct target, in the order the decode names the keys, with the no-match
    /// case added to the group it shares a target with. A target that *is* the join is an empty arm
    /// — the `case 0: break;` shape — and a default that *is* the join is no label at all, which is
    /// exactly what a `switch` that falls out of itself does.
    #[expect(
        clippy::too_many_arguments,
        reason = "the shape one switch arm list needs, passed as the values the caller already \
                  destructured: grouping them into a struct would only rename them"
    )]
    fn switch_region(
        &mut self,
        prefix: &[CanonicalBlockId],
        branch: &CanonicalBlockId,
        branch_bci: u32,
        successors: &[CanonicalBlockId],
        cases: &[(i64, u32)],
        default: u32,
        node: usize,
        frame: &Frame,
    ) -> Result<Run, StopReason> {
        // A refused `switch` keeps the prefix's statements, and every block the walk entered for it
        // stays named: an arm already walked into a region is dropped with the statement, so its
        // blocks are quoted beside the branch — the same exactly-once rule the body of a refused
        // loop follows ([`Self::loop_fallback`]).
        let quoted = |entered: &[Region], reason: FallbackReason| {
            let blocks = gap_blocks(branch, entered.iter().flat_map(Region::blocks).cloned());
            let (mut run, _) = gap(prefix.to_vec(), blocks, reason, None);
            run.extend(
                entered
                    .iter()
                    .filter(|region| matches!(region, Region::Fallback { .. }))
                    .cloned(),
            );
            (run, None)
        };
        let post_join = self
            .view
            .immediate_post_dominator(node)
            .filter(|join| *join != node);
        let post_is_loop_exit = post_join.is_some_and(|join| {
            frame
                .loop_targets
                .iter()
                .any(|target| target.break_target == Some(join))
        });
        let forward_join = if !post_is_loop_exit && post_join.is_none() {
            self.switch_forward_join(node, successors, frame, branch.bci())?
        } else {
            None
        };
        let join_node = if post_is_loop_exit {
            let Some(local_join) = self.switch_loop_join(node, successors, frame, branch_bci)?
            else {
                // An enclosing loop exit cannot stand in for a switch's local join. Without a
                // unique in-loop meeting point the arm ownership remains unproved.
                return Ok(quoted(
                    &[],
                    FallbackReason::SwitchShape {
                        block_bci: branch.bci(),
                    },
                ));
            };
            Some(local_join)
        } else {
            post_join.or(forward_join)
        };
        let join = join_node.and_then(|join| self.view.id_of(join).cloned());
        let join_bci = join.as_ref().map(CanonicalBlockId::bci);
        // Every target the decode names must be a successor the graph holds, and every successor a
        // target: a `switch` the decode and the graph disagree about is not a shape to present.
        let crosses = |target: u32| successors.iter().any(|successor| successor.bci() == target);
        if !crosses(default) || cases.iter().any(|(_, target)| !crosses(*target)) {
            return Ok(quoted(
                &[],
                FallbackReason::SwitchShape {
                    block_bci: branch.bci(),
                },
            ));
        }
        if successors.iter().any(|successor| {
            !cases.iter().any(|(_, target)| *target == successor.bci())
                && successor.bci() != default
        }) {
            return Ok(quoted(
                &[],
                FallbackReason::SwitchShape {
                    block_bci: branch.bci(),
                },
            ));
        }
        // The groups, in the order the decode first names each target.
        let mut groups: Vec<(Vec<i64>, bool, u32)> = Vec::new();
        for (key, target) in cases {
            match groups.iter_mut().find(|group| group.2 == *target) {
                Some(group) => {
                    if !group.0.contains(key) {
                        group.0.push(*key);
                    }
                }
                None => groups.push((vec![*key], false, *target)),
            }
        }
        if Some(default) != join_bci {
            match groups.iter_mut().find(|group| group.2 == default) {
                Some(group) => group.1 = true,
                None => groups.push((Vec::new(), true, default)),
            }
        }

        // A fallthrough is only a Java label ordering when one arm's complete normal path is a
        // straight route into exactly one other case entry. Probe the unclaimed CFG first: walking
        // arms to find this out would mutate `visited`, and a second walk could mistake that probe
        // for a real re-entry. Ambiguous shapes keep the pre-existing walk and its overlap refusal.
        let targets: BTreeMap<u32, usize> = groups
            .iter()
            .filter(|group| Some(group.2) != join_bci)
            .filter_map(|group| {
                successors
                    .iter()
                    .find(|successor| successor.bci() == group.2)
                    .and_then(|target| self.view.index_of(target))
                    .map(|node| (group.2, node))
            })
            .collect();
        let fall_throughs =
            self.switch_fallthroughs(&groups, &targets, join_node, join_bci, branch.bci())?;
        let mut ordered_groups = groups.clone();
        let case_entries: BTreeSet<usize> = targets.values().copied().collect();
        if let Some(fall_throughs) = &fall_throughs
            && !fall_throughs.is_empty()
        {
            ordered_groups.sort_by_key(|group| group.2);
            let positions: BTreeMap<u32, usize> = ordered_groups
                .iter()
                .enumerate()
                .map(|(index, group)| (group.2, index))
                .collect();
            let valid_order = fall_throughs.iter().all(|(from, to)| {
                from < to
                    && positions
                        .get(to)
                        .zip(positions.get(from))
                        .is_some_and(|(to, from)| *to == from.saturating_add(1))
            });
            let real_entries = ordered_groups
                .iter()
                .filter(|group| Some(group.2) != join_bci)
                .count();
            if !valid_order || targets.len() != real_entries {
                return Ok(quoted(
                    &[],
                    FallbackReason::SwitchArmsOverlap {
                        block_bci: branch.bci(),
                    },
                ));
            }
        }
        let mut built: Vec<SwitchGroup> = Vec::with_capacity(ordered_groups.len());
        let mut claimed: Vec<BTreeSet<usize>> = Vec::with_capacity(groups.len());
        // Every arm the walk entered, in the order it entered them: what a refusal below has to
        // keep naming ([`Self::switch_region`]'s own note on the exactly-once rule).
        let mut entered: Vec<Region> = Vec::new();
        for (keys, is_default, target) in ordered_groups.iter().cloned() {
            if Some(target) == join_bci {
                // The arm is the join itself: the case runs no code and leaves the switch.
                built.push(SwitchGroup {
                    keys,
                    default: is_default,
                    fall_through: false,
                    arm: Box::new(Region::Straight { blocks: Vec::new() }),
                });
                claimed.push(BTreeSet::new());
                continue;
            }
            let Some(start) = successors
                .iter()
                .find(|successor| successor.bci() == target)
                .cloned()
            else {
                return Ok(quoted(
                    &entered,
                    FallbackReason::SwitchShape {
                        block_bci: branch.bci(),
                    },
                ));
            };
            let proven_fallthrough = fall_throughs
                .as_ref()
                .is_some_and(|fall_throughs| !fall_throughs.is_empty());
            let local_loop_join = post_is_loop_exit && join_node != post_join;
            let case_frame = if proven_fallthrough || local_loop_join {
                let current = self.view.index_of(&start);
                let mut other_entries = case_entries.clone();
                if let Some(current) = current {
                    other_entries.remove(&current);
                }
                frame.switch_arm(join_node, &other_entries, branch_bci)
            } else if join_node.is_some() {
                // The switch's shared join belongs to the continuation after the switch. The
                // ordinary branch boundary is checked only after `visited` is changed, so two arms
                // reaching it would be mistaken for a loop re-entry. Keep other case entries
                // unbounded unless fallthrough was proved: otherwise a cross-case route could be
                // silently emitted as an independent arm with an inserted `break`.
                frame.switch_arm(join_node, &BTreeSet::new(), branch_bci)
            } else {
                // With no proven join, preserve the ordinary frame's transfer and ownership rules.
                frame.arm(join_node, Some(branch_bci))
            };
            let (arm_run, _) = self.region_at(&start, &case_frame)?;
            let arm = sequence_region(arm_run);
            let arm_nodes: BTreeSet<usize> = arm
                .blocks()
                .iter()
                .filter_map(|block| self.view.index_of(block))
                .collect();
            // Two arms that claim the same block are a case falling through into another case's
            // code: Java has a way to write that (`case 0: case 1:` shares a *target*, and those
            // are one group), but not two targets whose code overlaps.
            if claimed.iter().any(|other| !other.is_disjoint(&arm_nodes)) {
                entered.push(arm);
                return Ok(quoted(
                    &entered,
                    FallbackReason::SwitchArmsOverlap {
                        block_bci: branch.bci(),
                    },
                ));
            }
            claimed.push(arm_nodes);
            entered.push(arm.clone());
            built.push(SwitchGroup {
                keys,
                default: is_default,
                fall_through: fall_throughs
                    .as_ref()
                    .is_some_and(|fall_throughs| fall_throughs.contains_key(&target)),
                arm: Box::new(arm),
            });
        }
        // The sibling quotes the arms' own walks left behind are written after the statement, in
        // the order the arms were read ([`Run`]); the arms themselves are inside it.
        let tails: Vec<Region> = entered
            .into_iter()
            .filter(|region| matches!(region, Region::Fallback { .. }))
            .collect();
        let mut run = vec![Region::Switch {
            prefix: prefix.to_vec(),
            branch: branch.clone(),
            branch_bci,
            groups: built,
            join: join.clone(),
        }];
        run.extend(tails);
        Ok((run, join))
    }

    /// Find the narrow shared forward join needed when a switch's direct return/throw arm
    /// prevents an immediate post-dominator from existing. Every nonterminal target must be a
    /// straight, acyclic path to the same candidate; other targets must be direct return/throw
    /// blocks. This deliberately leaves nested exits and cross-case paths to the overlap refusal.
    fn switch_forward_join(
        &mut self,
        branch: usize,
        successors: &[CanonicalBlockId],
        frame: &Frame,
        at: u32,
    ) -> Result<Option<usize>, StopReason> {
        let targets: BTreeSet<usize> = successors
            .iter()
            .filter_map(|successor| self.view.index_of(successor))
            .collect();
        if targets.len() < 3 {
            return Ok(None);
        }

        let mut routes = Vec::new();
        let mut terminal_targets = 0usize;
        for start in targets.iter().copied() {
            poll(self.budget, Some(at))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(at),
            )?;
            if self.switch_target_is_terminal(start) {
                terminal_targets += 1;
                continue;
            }

            let mut route = Vec::new();
            let mut seen = BTreeSet::new();
            let mut current = start;
            loop {
                poll(self.budget, Some(at))?;
                charge(
                    self.budget,
                    CountedBudgetDimension::AnalysisSteps,
                    1,
                    Some(at),
                )?;
                if !seen.insert(current)
                    || (current != start && targets.contains(&current))
                    || frame
                        .case_entries
                        .as_ref()
                        .is_some_and(|entries| entries.contains(&current))
                    || frame
                        .scope
                        .as_ref()
                        .is_some_and(|scope| !scope.contains(&current))
                {
                    return Ok(None);
                }
                route.push(current);
                if frame.boundary == Some(current) {
                    break;
                }
                let next = self.view.successors(current);
                match next.as_slice() {
                    [next] => {
                        poll(self.budget, Some(at))?;
                        charge(
                            self.budget,
                            CountedBudgetDimension::AnalysisSteps,
                            1,
                            Some(at),
                        )?;
                        if self.view.dominates(*next, current)
                            || (targets.contains(next) && *next != start)
                            || (frame
                                .scope
                                .as_ref()
                                .is_some_and(|scope| !scope.contains(next))
                                && frame.boundary != Some(*next))
                        {
                            return Ok(None);
                        }
                        current = *next;
                    }
                    [] if self.switch_target_is_terminal(current) => break,
                    _ => return Ok(None),
                }
            }
            routes.push(route);
        }
        if routes.len() < 2 || terminal_targets == 0 {
            return Ok(None);
        }

        // Linear routes can merge only once and then share their suffix. Build position maps once,
        // charging each stored node, then select the first node in the first route shared by every
        // route. No address or path-length score stands in for the graph's unique merge order.
        let mut positions = Vec::with_capacity(routes.len());
        for route in &routes {
            let mut route_positions = BTreeMap::new();
            for (position, node) in route.iter().copied().enumerate() {
                poll(self.budget, Some(at))?;
                charge(
                    self.budget,
                    CountedBudgetDimension::AnalysisSteps,
                    1,
                    Some(at),
                )?;
                route_positions.insert(node, position);
            }
            positions.push(route_positions);
        }
        let Some(first) = routes.first() else {
            return Ok(None);
        };
        // Charge candidate-to-route membership checks before the bounded nested scan.
        poll(self.budget, Some(at))?;
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(first.len().saturating_mul(positions.len())).unwrap_or(u64::MAX),
            Some(at),
        )?;
        let Some(candidate) = first.iter().copied().find(|candidate| {
            !targets.contains(candidate)
                && positions.iter().all(|route| route.contains_key(candidate))
        }) else {
            return Ok(None);
        };

        let predecessors = self.view.predecessors(candidate);
        for _ in &predecessors {
            poll(self.budget, Some(at))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(at),
            )?;
        }
        // Reuse the binary forward-join ownership rule: no outside predecessor and no backedge
        // into the candidate. Do not skip an earlier common node if it fails this ownership test.
        Ok(self
            .forward_join_predecessors(branch, candidate)
            .then_some(candidate))
    }

    /// Whether a case target is a terminal block with an explicit return or throw.
    fn switch_target_is_terminal(&self, target: usize) -> bool {
        if !self.view.successors(target).is_empty() {
            return false;
        }
        self.view
            .id_of(target)
            .and_then(|block| self.terminal_bci(block))
            .and_then(|bci| self.operations.get(bci))
            .is_some_and(|operation| matches!(operation, Operation::Return | Operation::Throw))
    }

    /// Prove the ordinary fallthrough edges that can be represented by ordering case labels.
    ///
    /// This intentionally handles only a straight, single-successor route from one case entry to
    /// another. A branch, cycle, or any other ambiguous route returns `None`; the caller then uses
    /// the ordinary arm walk, which will preserve its existing overlap refusal. `Some(empty)` is a
    /// complete proof that no case entry flows directly into another case entry.
    fn switch_fallthroughs(
        &mut self,
        groups: &[(Vec<i64>, bool, u32)],
        targets: &BTreeMap<u32, usize>,
        join: Option<usize>,
        join_bci: Option<u32>,
        switch_bci: u32,
    ) -> Result<Option<BTreeMap<u32, u32>>, StopReason> {
        let entries: BTreeMap<usize, u32> =
            targets.iter().map(|(bci, node)| (*node, *bci)).collect();
        if entries.len() != targets.len()
            || groups
                .iter()
                .filter(|group| Some(group.2) != join_bci)
                .count()
                != targets.len()
        {
            return Ok(None);
        }

        let mut fallthroughs = BTreeMap::new();
        for (from_bci, start) in targets {
            let mut current = *start;
            let mut seen = BTreeSet::new();
            loop {
                poll(self.budget, Some(switch_bci))?;
                charge(
                    self.budget,
                    CountedBudgetDimension::AnalysisSteps,
                    1,
                    Some(switch_bci),
                )?;
                if !seen.insert(current) {
                    return Ok(None);
                }
                let successors = self.view.successors(current);
                match successors.as_slice() {
                    [] => break,
                    [next] => {
                        if Some(*next) == join {
                            break;
                        }
                        if let Some(to_bci) = entries.get(next) {
                            if to_bci <= from_bci {
                                return Ok(None);
                            }
                            fallthroughs.insert(*from_bci, *to_bci);
                            break;
                        }
                        current = *next;
                    }
                    _ => return Ok(None),
                }
            }
        }
        Ok(Some(fallthroughs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fallback_names_the_code_and_the_place_it_could_not_prove() {
        let reason = FallbackReason::BranchTargets {
            block_bci: 20,
            successors: 3,
        };
        assert_eq!(reason.code(), "jre_region_branch_targets");
        assert!(reason.message().contains("BCI 20"));
        assert!(reason.message().contains('3'));

        let reason = FallbackReason::UncoveredBlocks {
            blocks: vec![44, 45],
        };
        assert_eq!(reason.code(), "jre_region_uncovered_blocks");
        assert!(
            reason.message().contains("[44, 45]"),
            "{}",
            reason.message()
        );
    }
}
