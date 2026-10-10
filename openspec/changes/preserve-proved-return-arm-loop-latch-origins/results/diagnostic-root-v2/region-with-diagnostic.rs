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
    SsaBlock, SsaInstruction, SsaTable, ValueId,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{CpEntryFacts, ExceptionHandlerFact, MethodCodeFacts};

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
    /// A condition chain would present a postfix position outside the bound this slice states
    /// (`recover-postfix-condition-positions`): one position, at one end of the chain.
    ///
    /// The middle of a short-circuit chain and a second variable's position are recorded and left
    /// to a later slice, so a chain that would present them keeps the refusal it had instead of
    /// being presented as a shape this slice never stated. The position named is the one the bound
    /// refuses: the second position of a chain, or the one that is neither its first nor its last
    /// test.
    ChainPositionBound { block_bci: u32, at: u32 },
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
            Self::ChainPositionBound { .. } => "jre_region_chain_position_bound",
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
            Self::ChainPositionBound { block_bci, at } => format!(
                "the chain at block BCI {block_bci} would present the postfix condition position at BCI {at} outside the bound this slice states: one position, at one end of a condition chain"
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

/// Where the loop's test is presented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoopForm {
    /// `while (cond) { … }`: the test is the header, and the body runs only when it holds.
    While,
    /// `do { … } while (cond);`: the test is the latch, and the body runs once before it is read.
    DoWhile,
    /// `while (true) { … }`: both exits are explicit, proved breaks in the body.
    Endless,
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
        /// The continuation the consumer block's own trailing branch begins, recognized from that
        /// block at claim time. A consumer whose terminal instruction is the next structure's
        /// first test stores this region's value **and** branches again, so no disjoint join
        /// block exists for a sibling region to start at; the continuation is composed here and
        /// renders at this region's own lexical path, after the consumer block's suffix. The
        /// continuation never starts behind this region's back: the walk committed this region's
        /// participants before the nested dispatch, so the continuation's first test block is
        /// exactly `consumer`, and every block it holds is a block only it owns.
        tail: Vec<Region>,
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
    /// Two pure tests share one early return; their other path reaches the enclosing branch's
    /// proved tail. The tail is deliberately absent from this region's owners.
    SharedTailEarlyReturn {
        prefix: Vec<CanonicalBlockId>,
        tests: [(CanonicalBlockId, u32); 2],
        test_edges: [(CanonicalBlockId, CanonicalBlockId); 2],
        return_block: CanonicalBlockId,
        join: CanonicalBlockId,
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
    /// A natural loop with an ordered, proved set of header/latch tests, or an empty set
    /// when an Endless body's `If` regions own both exit tests.
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
        /// Hidden transfers represented by this loop's structure rather than a statement.
        gateway_origins: Vec<u32>,
    },
    /// An edge in a loop body whose target is this loop's proved Java break destination.
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
        /// An internally structured protected body. The enclosing plan remains its sole physical
        /// owner; this tree supplies lexical structure to the Java writer.
        body: Option<Box<Region>>,
        /// A separately bounded conditional cleanup, present only when the finally certificate
        /// proves both physical copies and this tree owns the normal copy's lexical `if`.
        finally_body: Option<Box<Region>>,
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
        /// A certified terminal goto whose normal continuation is represented by this try's
        /// lexical sequence, rather than by a Java transfer statement.
        normal_exit_bci: Option<u32>,
        /// The clauses, in exception-table order.
        catches: Vec<CatchClause>,
        /// Effect-free physical rethrow owned by this ordinary try, when proved.
        transparent: Option<crate::guard::TransparentHandler>,
    },
}

/// One `catch` clause of a presented `try`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatchClause {
    /// Named constant-pool types, or the Throwable type established by the single catch-all proof.
    /// [`crate::build`] spells named types from the pool in exception-table order.
    types: crate::guard::CatchTypes,
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
    pub(crate) fn types(&self) -> &crate::guard::CatchTypes {
        &self.types
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
    /// The exact instruction starts of physical cleanup replaced by a guard's source construct.
    pub(crate) fn elided_cleanup_bcis(&self, out: &mut Vec<u32>) {
        match self {
            Self::Sequence { regions } => {
                for region in regions {
                    region.elided_cleanup_bcis(out);
                }
            }
            Self::If {
                then_arm, else_arm, ..
            } => {
                then_arm.elided_cleanup_bcis(out);
                else_arm.elided_cleanup_bcis(out);
            }
            Self::Switch { groups, .. } | Self::StringSwitch { groups, .. } => {
                for group in groups {
                    group.arm.elided_cleanup_bcis(out);
                }
            }
            Self::Loop { body, .. } => {
                for region in body {
                    region.elided_cleanup_bcis(out);
                }
            }
            Self::Guard { plan, .. } => {
                if let crate::guard::Shape::NullableResourceFinally { normal_cleanup, .. } =
                    plan.shape()
                {
                    out.push(normal_cleanup.1);
                }
                if let crate::guard::Shape::Resources { cleanup, .. } = plan.shape() {
                    out.extend(cleanup.iter().copied());
                }
                if matches!(plan.shape(), crate::guard::Shape::LoopFinally { .. }) {
                    out.extend(
                        plan.facts()
                            .iter()
                            .copied()
                            .filter(|bci| (38..79).contains(bci)),
                    );
                }
            }
            Self::Try {
                body,
                catches,
                transparent,
                ..
            } => {
                body.elided_cleanup_bcis(out);
                for clause in catches {
                    clause.body.elided_cleanup_bcis(out);
                }
                if let Some(proof) = transparent {
                    out.extend(proof.bcis);
                }
            }
            Self::Straight { .. }
            | Self::TwoExitReturn { .. }
            | Self::SharedTailEarlyReturn { .. }
            | Self::LoopBreak { .. }
            | Self::LoopContinue { .. }
            | Self::Fallback { .. } => {}
            Self::ShortCircuitValue { tail, .. } => {
                for region in tail {
                    region.elided_cleanup_bcis(out);
                }
            }
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
                tail,
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
                for block in [true_producer, false_producer] {
                    if !blocks.contains(&block) {
                        blocks.push(block);
                    }
                }
                // A composed continuation starts **at** the consumer block: its own first test
                // lists the block, so listing it here too would name one block twice and the
                // completed tree would refuse as an ownership overlap.
                if tail.is_empty() && !blocks.contains(&consumer) {
                    blocks.push(consumer);
                }
                for region in tail {
                    blocks.extend(region.blocks());
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
            Self::SharedTailEarlyReturn {
                prefix,
                tests,
                return_block,
                ..
            } => {
                let mut blocks = prefix.iter().collect::<Vec<_>>();
                blocks.extend(tests.iter().map(|(block, _)| block));
                blocks.push(return_block);
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
                // A do-while body can own its entry or the effectful prefix of its latch.
                // An endless body owns its header. These fields name the shape while the
                // body remains the physical owner.
                // Keep duplicates inside the body visible to the method-level ownership check.
                let body_owns = |block: &CanonicalBlockId| {
                    *form != LoopForm::While
                        && body
                            .iter()
                            .flat_map(Region::blocks)
                            .any(|owned| owned == block)
                };
                if !body_owns(header) {
                    blocks.push(header);
                }
                blocks.extend(
                    tests
                        .iter()
                        .map(|(test, _, _)| test)
                        .filter(|test| *test != header && !body_owns(test)),
                );
                for region in body {
                    blocks.extend(region.blocks());
                }
                blocks
            }
            Self::Fallback { blocks, .. } => blocks.iter().collect(),
            Self::LoopBreak { .. } => Vec::new(),
            Self::LoopContinue { .. } => Vec::new(),
            Self::Guard { prefix, plan, .. } => {
                let mut blocks: Vec<&CanonicalBlockId> = prefix.iter().collect();
                blocks.extend(plan.owned());
                blocks
            }
            Self::Try {
                prefix,
                body,
                catches,
                transparent,
                ..
            } => {
                let mut blocks: Vec<&CanonicalBlockId> = prefix.iter().collect();
                blocks.extend(body.blocks());
                for clause in catches {
                    blocks.extend(clause.body.blocks());
                }
                if let Some(proof) = transparent {
                    blocks.push(&proof.block);
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
            Self::SharedTailEarlyReturn { .. } => true,
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
            Self::SharedTailEarlyReturn { .. } => Vec::new(),
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
            Self::SharedTailEarlyReturn { .. } => None,
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
            Region::Guard {
                body: Some(body),
                finally_body,
                ..
            } => {
                visit(body, ir, budget)?;
                if let Some(cleanup) = finally_body {
                    visit(cleanup, ir, budget)?;
                }
            }
            Region::ShortCircuitValue { tail, .. } => {
                for region in tail.iter_mut() {
                    visit(region, ir, budget)?;
                }
            }
            Region::TwoExitReturn { .. } | Region::SharedTailEarlyReturn { .. } => {}
            Region::Straight { .. }
            | Region::Fallback { .. }
            | Region::Guard { body: None, .. }
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
    pool: &[CpEntryFacts],
    chains: &crate::concat::Plan,
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
    let fragmented = crate::fragmented_catch::prove(canonical, view, ssa, code, budget)?;
    let mut catch_joins = proved_loop_catch_joins(canonical, view, code, budget)?;
    if let Some(proof) = &fragmented {
        catch_joins.extend(proof.loop_entries.iter().copied());
    }
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
        pool,
        chains,
        sites,
        code,
        method_synchronized,
        has_reachable_explicit_monitor,
        return_is_boolean,
        handlers: &code.exception_handlers,
        profile,
        budget,
        catch_joins,
        fragmented,
        excluded_edge_nodes,
        visited: BTreeSet::new(),
        depth: 0,
        own_monitor: None,
        unclosed_tail_at: None,
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
    if let Some(block_bci) = walker.unclosed_tail_at {
        return Ok(quoted_whole(
            live,
            FallbackReason::ArmsDoNotMeet { block_bci },
            canonical,
        ));
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
    // The failure-closure invariant runs at the quote landing point in [`crate::build`], where
    // the built statement tree — not the region tree — can answer whether a body that dropped
    // the quote's comment lines would still compile (`quoted_control_flow_exit` below is the
    // classification it consults). P3-R7's unaccounted check stays last, so bytes the graph
    // does not account for keep their own naming beside whatever that refusal states.
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

/// One arm's strictly forward routes, as [`Walker::arm_forward_routes`] read them.
struct ArmRoutes {
    /// Every block the arm holds before the stop block or the frame's boundary.
    blocks: BTreeSet<usize>,
    /// Whether a route reached the stop block the caller named.
    reached_stop: bool,
    /// Whether a route reached the frame's boundary — a route that would run past the join.
    reached_boundary: bool,
}

/// How many branches a route from a ladder's arm to its join may pass: one, the `else if` step
/// itself. The bound is what keeps the reading to the single-level ladder this change presents.
const LADDER_MAX_BRANCHES: usize = 1;

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

/// The first control-flow-changing exit a quoted fallback region holds, when the completed tree
/// also presents structure the quote is reachable from — the failure-closure invariant's
/// classification, consulted by [`crate::build`] at the quote landing point
/// (`recover-return-in-do-while-false`).
///
/// A partial body publishes statements for the regions it proved and quotes the rest. That is
/// acceptable only while nothing the quote holds can change what the method does: a reader who
/// strips the quote's comment lines must not be left with a method that still compiles while
/// silently dropping an edge's execution. The exits that change control flow are exactly
///
/// * a `return` — the method exit,
/// * an `athrow` — the exceptional method exit, and
/// * a decoded transfer whose one normal destination no region of the completed tree owns —
///   the `break`/`continue` escape a quoted loop-jump would spell.
///
/// The classification runs over the completed tree, over every fallback region at once — never
/// per region kind: whichever refusal produced the quote, the edge it holds is the same fact.
/// Two guard rails keep it from naming edges the invariant does not protect:
///
/// * a tree that presents **no** structure is already a whole-body refusal — there is no
///   partial body left whose stripped quote could compile, so nothing is classified;
/// * the quote must be reached from the presented structure through a **normal** edge. A
///   handler body its exception row alone enters is not on any path the presentation states;
///   the partial body claims nothing about the exceptional path, so its quote stays.
///
/// Whether the method is then refused is **not** this function's question: the answer depends
/// on whether the built body without the quote would still compile, and that is the statement
/// layer's fact (a `try`/`catch` whose empty clause falls off the end does not, an explicit
/// `return` at the tail does). [`crate::build`] holds both halves and refuses there.
pub(crate) fn quoted_control_flow_exit(
    regions: &[Region],
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    operations: &Operations,
    budget: &mut Budget,
) -> Result<Option<(u32, u32)>, StopReason> {
    if !regions.iter().any(Region::is_structured) {
        return Ok(None);
    }
    let presented: BTreeSet<CanonicalBlockId> = regions
        .iter()
        .filter(|region| region.is_structured())
        .flat_map(Region::blocks)
        .cloned()
        .collect();
    let quoted: BTreeSet<CanonicalBlockId> = regions
        .iter()
        .filter(|region| !region.is_structured())
        .flat_map(Region::blocks)
        .cloned()
        .collect();
    if presented.is_empty() || quoted.is_empty() {
        return Ok(None);
    }
    let owned: BTreeSet<CanonicalBlockId> =
        regions.iter().flat_map(Region::blocks).cloned().collect();
    poll(budget, quoted.iter().next().map(|block| block.bci()))?;
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(canonical.edges().len()).unwrap_or(u64::MAX),
        quoted.iter().next().map(|block| block.bci()),
    )?;
    // The seed: a normal edge from a presented block into a quoted one. A branch arm, a
    // fall-through, a transfer — the paths the published statements state — are the only ways
    // the executed method can arrive inside the quote, so they are the only ways stripping it
    // can silently change what those statements do.
    let reached_from_presented = canonical.edges().iter().any(|edge| {
        edge.kind() == CanonicalEdgeKind::Normal
            && presented.contains(edge.from())
            && quoted.contains(edge.to())
    });
    if !reached_from_presented {
        return Ok(None);
    }
    for block in &quoted {
        poll(budget, Some(block.bci()))?;
        let Some(at) = ssa
            .block(block)
            .and_then(|block| block.instructions().last().map(|last| last.bci()))
        else {
            continue;
        };
        match operations.get(at) {
            Some(Operation::Return | Operation::Throw) => return Ok(Some((block.bci(), at))),
            Some(Operation::Transfer) => {
                // A decoded `goto` whose one normal destination leaves both the presented
                // structure and this quote's own blocks: a `break`/`continue` escape whose
                // destination no presented region owns. A transfer back into the presented
                // structure is a plain continuation the quote spells, and a transfer that
                // stays inside its own quote (a refused loop's own latch) is internal to the
                // system the quote names — neither is an edge that leaves what the
                // presentation states.
                let destinations: Vec<_> = canonical
                    .edges()
                    .iter()
                    .filter(|edge| edge.from() == block && edge.kind() == CanonicalEdgeKind::Normal)
                    .map(|edge| edge.to())
                    .collect();
                if destinations.len() == 1 && !owned.contains(destinations[0]) {
                    return Ok(Some((block.bci(), at)));
                }
            }
            _ => {}
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
    /// Direct entry of one conditional arm, with its owning branch. The optional third node is
    /// the boundary of one enclosing candidate arm; both joins are checked before publication.
    /// No deeper arm inherits this candidate.
    if_arm: Option<(usize, usize, Option<usize>)>,
    /// A proved shared tail of an enclosing branch. Nested local joins may change `boundary`,
    /// but no descendant may claim this block before the enclosing continuation does.
    shared_tail: Option<usize>,
    /// The nodes the region may claim, when it is a loop's body. A node outside the scope ends the
    /// run exactly like the boundary does — it is the code after the loop.
    scope: Option<BTreeSet<usize>>,
    /// The header of the loop this frame is the body of. A body walk that arrives back at its own
    /// header is inside the structure it is building, not a nested loop. Only a separately proved
    /// first entry may walk that block as body code (see [`Walker::region_at`]).
    own_loop: Option<usize>,
    /// A proved do-while body branch or endless body may enter its own header once.
    /// Other shapes retain the re-entry stop until their body ownership is proved separately.
    allow_own_loop_entry: bool,
    /// The block of the `try` this frame is the protected range of. The range begins at that block,
    /// so the walk that recovers it starts at it, and reading it as the start of *another* `try`
    /// would be reading the statement it is already building (see [`Walker::try_region`]).
    own_try: Option<usize>,
    /// The rows consumed by the enclosing finally certificate in this bounded body. The named
    /// catch and try catch-all are both present in the protected body; the catch body owns its
    /// separate catch-all row.
    own_finally: Option<((u32, (u32, u32)), Option<(u32, (u32, u32))>)>,
    /// The four protected rows of the one five-row segmented finally certificate.
    segmented_finally_rows: Option<[(u32, (u32, u32)); 4]>,
    /// The two disjoint body rows of the fixed two-return loop certificate.
    multi_return_finally_rows: Option<[(u32, (u32, u32)); 2]>,
    /// The proved two-row void finally whose protected body holds ordinary loops: a loop body
    /// frame keeps the certificate's own row so its covered exception edges stay accounted.
    void_loop_finally: bool,
    /// The proved lock guard's or resource guard's protected body holds ordinary loops: a loop body
    /// frame keeps the certificate's own row (the resource guard's row set) so its covered exception
    /// edges stay accounted, exactly as the two-row void finally's does.
    lock_guard_finally: bool,
    /// The sole inner named row admitted by a proved two-copy outer finally body.
    nested_finally_row: Option<u32>,
    /// Other case-entry nodes of a switch arm. They end this arm before the next case claims them.
    case_entries: Option<BTreeSet<usize>>,
    /// The physical normal-flow exit of the loop whose body this frame walks.
    loop_exit: Option<usize>,
    /// Proven transfer destinations of enclosing loops, outermost first.
    loop_targets: Vec<LoopTarget>,
    /// Source instruction of an incoming branch/switch edge, when this arm is a transfer leaf.
    transfer_source_bci: Option<u32>,
    /// A switch's own join must remain a switch break instead of becoming a loop break.
    switch_join: Option<usize>,
    /// Only a proved switch arm may treat the current loop's update as a continue transfer.
    switch_continue: Option<usize>,
}

#[derive(Clone, Debug)]
struct LoopTarget {
    header: usize,
    exits: BTreeSet<usize>,
    break_target: Option<usize>,
    continue_target: usize,
}

/// The nested synchronized body a frame is the walk of: every block the enclosing monitor plan
/// owns — its own handler and the inner pair's included, whose exception rows account the body's
/// throwing edges while the walk writes the statements it protects.
#[derive(Clone, Debug)]
struct MonitorBody {
    owned: Vec<CanonicalBlockId>,
}

struct HeaderTestChain {
    tests: Vec<(CanonicalBlockId, u32, Continuation)>,
    operator: crate::ast::BinaryOp,
    body: CanonicalBlockId,
    exit: CanonicalBlockId,
}

/// Where a switch inside a loop meets, decided by [`Walker::switch_loop_join`].
enum SwitchLoopJoin {
    /// One proved in-loop join the arms' own routes share.
    Local(usize),
    /// No block inside the loop is met by two arms: every arm's route ends at the current
    /// loop's continue target, so that target is the join and the arms fall out of the switch.
    ContinueTarget,
    /// The arm routes cannot be classified, or they share an in-loop block this proof cannot
    /// elect as the one join. The switch is refused.
    Refused,
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
        break_target: Option<usize>,
        exits: BTreeSet<usize>,
        transfer_sources: &BTreeSet<usize>,
        terminal_returns: &BTreeSet<usize>,
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
        // A proved return leaf is reached recursively from the branch in this body, rather than
        // from the natural-loop walk's top-level queue.
        scope.extend(terminal_returns.iter().copied());
        Self {
            boundary: Some(boundary),
            if_arm: None,
            shared_tail: self.shared_tail,
            scope: Some(scope),
            own_loop: Some(header),
            allow_own_loop_entry: false,
            own_try: None,
            own_finally: None,
            segmented_finally_rows: None,
            multi_return_finally_rows: None,
            void_loop_finally: self.void_loop_finally,
            lock_guard_finally: self.lock_guard_finally,
            nested_finally_row: None,
            case_entries: self.case_entries.clone(),
            loop_exit: exit,
            loop_targets: {
                let mut targets = self.loop_targets.clone();
                targets.push(LoopTarget {
                    header,
                    exits,
                    break_target,
                    continue_target,
                });
                targets
            },
            transfer_source_bci: None,
            switch_join: self.switch_join,
            switch_continue: None,
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
            if_arm: None,
            shared_tail: self.shared_tail,
            scope: self.scope.clone(),
            own_loop: None,
            allow_own_loop_entry: false,
            // An `if` inside a protected range is still inside that range in either arm. Keep
            // its owner so a throwing arm's exception edges can be matched to this try's catches.
            own_try: self.own_try,
            own_finally: self.own_finally,
            segmented_finally_rows: self.segmented_finally_rows,
            multi_return_finally_rows: self.multi_return_finally_rows,
            void_loop_finally: self.void_loop_finally,
            lock_guard_finally: self.lock_guard_finally,
            nested_finally_row: self.nested_finally_row,
            case_entries: self.case_entries.clone(),
            loop_exit: self.loop_exit,
            loop_targets: self.loop_targets.clone(),
            transfer_source_bci: source_bci.or(self.transfer_source_bci),
            switch_join: self.switch_join,
            switch_continue: self.switch_continue,
        }
    }

    /// One switch arm, bounded by the other proven case entries as well as its enclosing join.
    fn switch_arm(
        &self,
        join: Option<usize>,
        entries: &BTreeSet<usize>,
        source_bci: u32,
        continue_target: Option<usize>,
    ) -> Self {
        let mut case_entries = self.case_entries.clone().unwrap_or_default();
        case_entries.extend(entries.iter().copied());
        Self {
            boundary: join.or(self.boundary),
            if_arm: None,
            shared_tail: self.shared_tail,
            scope: self.scope.clone(),
            own_loop: None,
            allow_own_loop_entry: false,
            own_try: self.own_try,
            own_finally: self.own_finally,
            segmented_finally_rows: self.segmented_finally_rows,
            multi_return_finally_rows: self.multi_return_finally_rows,
            void_loop_finally: self.void_loop_finally,
            lock_guard_finally: self.lock_guard_finally,
            nested_finally_row: self.nested_finally_row,
            case_entries: Some(case_entries),
            loop_exit: self.loop_exit,
            loop_targets: self.loop_targets.clone(),
            transfer_source_bci: Some(source_bci),
            switch_join: join.or(self.switch_join),
            switch_continue: continue_target,
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
            if_arm: None,
            shared_tail: self.shared_tail,
            scope: self.scope.clone(),
            own_loop: None,
            allow_own_loop_entry: false,
            own_try: Some(start),
            own_finally: self.nested_finally_row.and(self.own_finally),
            segmented_finally_rows: self.segmented_finally_rows,
            multi_return_finally_rows: self.multi_return_finally_rows,
            void_loop_finally: self.void_loop_finally,
            lock_guard_finally: self.lock_guard_finally,
            nested_finally_row: self.nested_finally_row,
            case_entries: self.case_entries.clone(),
            loop_exit: self.loop_exit,
            loop_targets: self.loop_targets.clone(),
            transfer_source_bci: self.transfer_source_bci,
            switch_join: self.switch_join,
            switch_continue: self.switch_continue,
        }
    }

    /// Whether a walk must stop at one node: the node it ends at, or one the scope does not hold.
    fn stops_at(&self, node: usize) -> bool {
        self.boundary == Some(node)
            || self.shared_tail == Some(node)
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
    pool: &'a [CpEntryFacts],
    chains: &'a crate::concat::Plan,
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
    fragmented: Option<crate::fragmented_catch::FragmentedCatch>,
    /// Endpoints of exception and subroutine edges, computed once so bounded local shape probes do
    /// not rescan the whole canonical edge table.
    excluded_edge_nodes: BTreeSet<CanonicalBlockId>,
    visited: BTreeSet<usize>,
    /// A reachable arm successor that could not be owned. Refuse the whole method so a later
    /// unconditional return cannot turn an incomplete quote into executable wrong behavior.
    unclosed_tail_at: Option<u32>,
    /// The nested synchronized body the walk is currently inside, if any: set for the bounded
    /// walk of a claimed monitor plan's structured body ([`Walker::monitor_nested_body`]), where
    /// no guarded rule is examined and the body's throwing edges are accounted by the monitor
    /// statement's own rows. The walker field — and not a frame's — because the state belongs to
    /// exactly one bounded walk, saved and restored around it.
    own_monitor: Option<MonitorBody>,
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
    Option<u32>,
    Option<crate::guard::TransparentHandler>,
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

/// Every physical edge touching a lexical transfer must agree with the protected body's sole
/// normal exit. In particular, the normal-flow projection alone cannot detect a competing
/// exception or subroutine entry because it deliberately excludes those edges.
fn closed_transfer_edges<Id: Ord>(
    edges: impl IntoIterator<Item = (CanonicalEdgeKind, Id, Id)>,
    bridge: &Id,
    join: &Id,
    owned: &BTreeSet<Id>,
) -> bool {
    let (mut incoming, mut outgoing) = (0usize, 0usize);
    for (kind, from, to) in edges {
        if &to == bridge {
            if kind != CanonicalEdgeKind::Normal || !owned.contains(&from) {
                return false;
            }
            incoming += 1;
        }
        if &from == bridge {
            if kind != CanonicalEdgeKind::Normal || &to != join {
                return false;
            }
            outgoing += 1;
        }
    }
    incoming > 0 && outgoing == 1
}

/// A run of one region: the ordinary case, said once.
fn one(region: Region, next: Option<CanonicalBlockId>) -> Run {
    (vec![region], next)
}

/// The region kinds a nested synchronized body may hold: the ordinary statement structure the
/// walk proves — straight runs, branches and loops. No guarded shape of its own (a `try`, a
/// `finally` or another monitor inside a monitor body is a compound this slice does not present)
/// and no fallback quote: a quoted block inside the braces would present a `synchronized` whose
/// body is not the one the bytecode runs.
fn monitor_body_supported(region: &Region) -> bool {
    match region {
        Region::Straight { .. } => true,
        Region::Sequence { regions } => regions.iter().all(monitor_body_supported),
        Region::If {
            then_arm, else_arm, ..
        } => monitor_body_supported(then_arm) && monitor_body_supported(else_arm),
        Region::Loop { body, .. } => body.iter().all(monitor_body_supported),
        _ => false,
    }
}

fn finally_body_supported(region: &Region, nested_catch: bool) -> bool {
    match region {
        Region::Straight { .. } => true,
        Region::Sequence { regions } => regions
            .iter()
            .all(|region| finally_body_supported(region, nested_catch)),
        Region::If {
            then_arm, else_arm, ..
        } => {
            finally_body_supported(then_arm, nested_catch)
                && finally_body_supported(else_arm, nested_catch)
        }
        Region::Try { body, catches, .. } if nested_catch => {
            catches.len() == 1
                && finally_body_supported(body, false)
                && finally_body_supported(catches[0].body(), false)
        }
        _ => false,
    }
}

fn shared_join_body_supported(region: &Region, loop_allowed: bool) -> bool {
    match region {
        // A loop body may hold a loop of its own where the enclosing loop is admitted: the
        // fixed body-loop certificates prove every physical block either way, so the nesting
        // depth is the source's own, not a relaxation of the walk.
        Region::Loop { body, .. } if loop_allowed => body
            .iter()
            .all(|part| shared_join_body_supported(part, loop_allowed)),
        Region::Sequence { regions } => regions
            .iter()
            .all(|part| shared_join_body_supported(part, loop_allowed)),
        Region::If {
            then_arm, else_arm, ..
        } => {
            shared_join_body_supported(then_arm, loop_allowed)
                && shared_join_body_supported(else_arm, loop_allowed)
        }
        _ => finally_body_supported(region, false),
    }
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
    ///   is the block this entry starts at is the body of that loop. The proved body-branch path
    ///   permits exactly its first entry; every other such entry stops before descending.
    fn region_at(&mut self, start: &CanonicalBlockId, frame: &Frame) -> Result<Run, StopReason> {
        let reentered = self.view.index_of(start).is_some_and(|node| {
            frame.own_loop == Some(node)
                && (!frame.allow_own_loop_entry || self.visited.contains(&node))
        });
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
                        } else if (current_loop != Some(target.header)
                            || frame.switch_continue == Some(node)
                            || self.fragmented.as_ref().is_some_and(|proof| {
                                frame.own_try.is_some()
                                    && proof.loop_header == target.header
                                    && self.view.index_of(&proof.update) == Some(node)
                            }))
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
            // A protected range the exception table states with a named `catch` type is the
            // `try`/`catch` statement here, where the guarded rules of P3 2.4 say the region is not
            // theirs. This is read **before** the block is marked visited: the statement's own range
            // starts in this block, and the walk that recovers it starts there too
            // ([`Self::try_region`]).
            // A shared catch-all has to be selected before the ordinary named-catch reader:
            // that reader owns only its named row and cannot account for the second protected
            // range or either return copy. The private guard proof is complete before any child
            // walk may mark a block visited. The resource guard's row set is asked here for the
            // same reason and one more: its protected range begins where its own block does (the
            // object's construction leads it), and the *loop* its body holds is a block of its
            // own — the loop reader below would otherwise present the protected body as a loop of
            // its own before the statement was claimed.
            if frame.own_try != Some(node)
                && frame.own_finally.is_none()
                && (self.starts_catch(&current) || self.starts_resource_guard(&current))
                && let Some(plan) = crate::guard::shared_finally_candidate(
                    self.canonical,
                    self.view,
                    self.ssa,
                    self.operations,
                    self.pool,
                    self.chains,
                    self.handlers,
                    self.profile,
                    &current,
                    self.budget,
                )?
            {
                let recovered = match plan.shape() {
                    crate::guard::Shape::Finally { .. } => self
                        .finally_body(&current, &plan, frame)?
                        .map(|body| (body, None)),
                    crate::guard::Shape::LockGuardFinally { .. } => self
                        .lock_guard_finally_body(&current, &plan, frame)?
                        .map(|body| (body, None)),
                    crate::guard::Shape::LoopFinally { .. } => self
                        .loop_finally_regions(&current, &plan, frame)?
                        .map(|(body, cleanup)| (body, Some(cleanup))),
                    crate::guard::Shape::ConditionalFinally { .. } => self
                        .conditional_finally_regions(&current, &plan, frame)?
                        .map(|(body, cleanup)| (body, Some(cleanup))),
                    crate::guard::Shape::NullableResourceFinally { .. } => self
                        .nullable_resource_finally_regions(&current, &plan, frame)?
                        .map(|(body, cleanup)| (body, Some(cleanup))),
                    crate::guard::Shape::FlagConditionalFinally { .. } => self
                        .flag_conditional_finally_regions(&current, &plan, frame)?
                        .map(|(body, cleanup)| (body, Some(cleanup))),
                    crate::guard::Shape::LocalNullConditionalFinally { .. } => self
                        .local_null_conditional_finally_regions(&current, &plan, frame)?
                        .map(|(body, cleanup)| (body, Some(cleanup))),
                    crate::guard::Shape::SharedFinally { .. } => self
                        .shared_finally_body(&current, &plan, frame)?
                        .map(|body| (body, None)),
                    crate::guard::Shape::EmptyCatchCallFinally { .. } => self
                        .empty_catch_call_finally_body(&current, &plan, frame)?
                        .map(|body| (body, None)),
                    crate::guard::Shape::TwoCatchReturnFinally { .. } => self
                        .two_catch_return_finally_body(&current, &plan, frame)?
                        .map(|body| (body, None)),
                    crate::guard::Shape::NestedCleanupFinally { .. } => self
                        .nested_cleanup_finally_regions(&current, &plan, frame)?
                        .map(|(body, cleanup)| (body, Some(cleanup))),
                    crate::guard::Shape::SegmentedFinally { .. } => self
                        .segmented_finally_body(&current, &plan, frame)?
                        .map(|body| (body, None)),
                    crate::guard::Shape::MultiReturnLoopFinally { .. } => self
                        .multi_return_loop_finally_body(&current, &plan, frame)?
                        .map(|body| (body, None)),
                    crate::guard::Shape::ResourceGuardFinally { .. } => self
                        .resource_guard_finally_body(&current, &plan, frame)?
                        .map(|body| (body, None)),
                    _ => None,
                };
                if let Some((body, finally_body)) = recovered {
                    for block in plan.owned() {
                        if let Some(index) = self.view.index_of(block) {
                            self.visited.insert(index);
                        }
                    }
                    let join = plan.join().cloned();
                    return Ok(one(
                        Region::Guard {
                            prefix,
                            plan,
                            body: Some(Box::new(body)),
                            finally_body: finally_body.map(Box::new),
                        },
                        join,
                    ));
                }
                let reason = FallbackReason::Guard {
                    pass: None,
                    code: "jre_guard_finally_copy",
                    at: current.bci(),
                    message:
                        "the shared catch-all finally has no complete bounded try and catch bodies"
                            .into(),
                };
                return Ok(gap(prefix, plan.owned().to_vec(), reason, None));
            }
            if frame.own_try != Some(node)
                && (frame.own_finally.is_none() || frame.nested_finally_row.is_some())
                && self.starts_catch(&current)
                && let Some((body, lead, catches, join, tails, exit_bci, transparent)) =
                    self.try_region(&current, node, frame)?
            {
                let mut run = vec![Region::Try {
                    prefix,
                    lead,
                    body,
                    catches,
                    transparent,
                    normal_exit_bci: exit_bci.or_else(|| {
                        self.fragmented.as_ref().and_then(|proof| {
                            (proof.outer_start != proof.inner_start
                                && current.bci() == proof.inner_start)
                                .then_some(proof.inner_exit_bci)
                        })
                    }),
                }];
                run.extend(tails);
                if self.fragmented.as_ref().is_some_and(|proof| {
                    proof.outer_start != proof.inner_start
                        && current.bci() == proof.inner_start
                        && frame.own_try.is_some()
                }) && let Some(tail_start) = join.as_ref()
                    && frame.boundary != self.view.index_of(tail_start)
                {
                    let mut next = Some(tail_start.clone());
                    for _ in 0..self.canonical.blocks().len() {
                        let Some(at) = next.as_ref() else { break };
                        if frame.boundary == self.view.index_of(at) {
                            break;
                        }
                        let previous = at.clone();
                        let (tail, following) = self.region_at(&previous, frame)?;
                        run.extend(tail);
                        next = following;
                        if next.as_ref() == Some(&previous) {
                            break;
                        }
                    }
                    return Ok((run, next));
                }
                return Ok((run, join));
            }
            // A loop this walk enters from outside is a region of its own, and it *starts* one: the
            // run that led here ends before the loop, because the loop's test is written inside the
            // statement that presents it. Re-entering a block that is not a loop header this subset
            // can prove (an arm that jumps back, an irreducible cycle) stays the stated fallback
            // below.
            if self.view.is_loop_header(node) && frame.own_loop != Some(node) {
                // Arriving at the header of the loop this frame is inside — an arm walk, a
                // protected body or any nested frame following the loop's own latch edge — is
                // that back edge (`continue`'s target), not a nested loop entry: the run ends
                // here and the caller continues at the header. Entering the loop *again* would
                // build a second region over the same blocks and the completed tree would own
                // them twice. A header no enclosing loop target names stays a fresh entry.
                let latch_edge_of_own_loop = frame
                    .loop_targets
                    .last()
                    .is_some_and(|target| target.header == node);
                if prefix.is_empty() && !latch_edge_of_own_loop {
                    return self.loop_region(&current, node, frame);
                }
                return Ok(one(Region::Straight { blocks: prefix }, Some(current)));
            }
            let leaving = self.leaving_edge(&current);
            // Examine once, before ownership changes. A structured finally consumes the verdict
            // here; every other guard keeps it for the ordinary branch below.
            let monitor_entry = self.ssa.block(&current).is_some_and(|block| {
                block.instructions().iter().any(|instruction| {
                    matches!(
                        self.operations.get(instruction.bci()),
                        Some(Operation::Monitor { enter: true })
                    )
                })
            });
            let mut guard_verdict = if (leaving.is_some() || monitor_entry)
                && frame.own_finally.is_none()
                && self.own_monitor.is_none()
                && !self.visited.contains(&node)
                && !(frame.own_try == Some(node)
                    && self.handlers.len() == 1
                    && self.handlers[0].catch_type_index.is_none())
            {
                Some(crate::guard::examine(
                    self.canonical,
                    self.view,
                    self.ssa,
                    self.operations,
                    self.handlers,
                    self.sites,
                    self.profile,
                    &current,
                    self.budget,
                )?)
            } else {
                None
            };
            // A finally certificate with control flow needs the same pre-visited recursive entry
            // as a named try. Nothing is claimed until the child has covered the exact protected
            // block set and every edge has been accounted for.
            if matches!(guard_verdict.as_ref(), Some(crate::guard::Verdict::Claimed(plan))
                if matches!(plan.shape(), crate::guard::Shape::Finally { structured: true, .. }
                    | crate::guard::Shape::LoopFinally { .. }
                    | crate::guard::Shape::NullableResourceFinally { .. }
                    | crate::guard::Shape::FlagConditionalFinally { .. }
                    | crate::guard::Shape::LocalNullConditionalFinally { .. }
                    | crate::guard::Shape::SegmentedNullLeadFinally { .. }))
            {
                let Some(crate::guard::Verdict::Claimed(plan)) = guard_verdict.take() else {
                    unreachable!("the structured finally verdict was just matched")
                };
                let recovered = match plan.shape() {
                    crate::guard::Shape::LoopFinally { .. } => self
                        .loop_finally_regions(&current, &plan, frame)?
                        .map(|(body, cleanup)| (body, Some(cleanup))),
                    crate::guard::Shape::NullableResourceFinally { .. } => self
                        .nullable_resource_finally_regions(&current, &plan, frame)?
                        .map(|(body, cleanup)| (body, Some(cleanup))),
                    crate::guard::Shape::FlagConditionalFinally { .. } => self
                        .flag_conditional_finally_regions(&current, &plan, frame)?
                        .map(|(body, cleanup)| (body, Some(cleanup))),
                    crate::guard::Shape::LocalNullConditionalFinally { .. } => self
                        .local_null_conditional_finally_regions(&current, &plan, frame)?
                        .map(|(body, cleanup)| (body, Some(cleanup))),
                    crate::guard::Shape::SegmentedNullLeadFinally { .. } => self
                        .segmented_null_lead_finally_regions(&current, &plan, frame)?
                        .map(|(body, cleanup)| (body, Some(cleanup))),
                    crate::guard::Shape::Finally {
                        completion: crate::guard::FinallyCompletion::Void { .. },
                        row_ordinal,
                        ..
                    } => self
                        .bounded_shared_finally_body(
                            &current,
                            plan.body(),
                            None,
                            ((*row_ordinal, plan.body()), None),
                            &plan,
                            frame,
                            None,
                        )?
                        .map(|body| (body, None)),
                    _ => self
                        .finally_body(&current, &plan, frame)?
                        .map(|body| (body, None)),
                };
                if let Some((body, cleanup)) = recovered {
                    let join = plan.join().cloned();
                    for block in plan.owned() {
                        if let Some(node) = self.view.index_of(block) {
                            self.visited.insert(node);
                        }
                    }
                    return Ok(one(
                        Region::Guard {
                            prefix,
                            plan,
                            body: Some(Box::new(body)),
                            finally_body: cleanup.map(Box::new),
                        },
                        join,
                    ));
                }
                let reason = FallbackReason::Guard {
                    pass: None,
                    code: "jre_guard_finally_copy",
                    at: current.bci(),
                    message: "the proved finally body has no complete bounded structure".into(),
                };
                return Ok(gap(prefix, plan.owned().to_vec(), reason, None));
            }
            if matches!(guard_verdict.as_ref(), Some(crate::guard::Verdict::Claimed(plan))
                if matches!(plan.shape(), crate::guard::Shape::MonitorBranches { .. }))
            {
                let Some(crate::guard::Verdict::Claimed(plan)) = guard_verdict.take() else {
                    unreachable!("the multi-exit monitor verdict was just matched")
                };
                for block in plan.owned() {
                    if let Some(node) = self.view.index_of(block) {
                        self.visited.insert(node);
                    }
                }
                return Ok(one(
                    Region::Guard {
                        prefix,
                        plan,
                        body: None,
                        finally_body: None,
                    },
                    None,
                ));
            }
            // A monitor whose body holds a proved inner pair is a **structured** body: the walk
            // builds its region tree — the loop or branch around the inner block included — the
            // same way a proved `finally` body's is, and nothing is claimed until that tree
            // covered the exact blocks the body's own instructions occupy. The inner pair's
            // braces are placed where its span sits in that tree, by the builder.
            if matches!(guard_verdict.as_ref(), Some(crate::guard::Verdict::Claimed(plan))
                if matches!(plan.shape(), crate::guard::Shape::Monitor { nested: Some(_), .. }))
            {
                let Some(crate::guard::Verdict::Claimed(plan)) = guard_verdict.take() else {
                    unreachable!("the nested monitor verdict was just matched")
                };
                if let Some(body) = self.monitor_nested_body(&plan, frame)? {
                    for block in plan.owned() {
                        if let Some(node) = self.view.index_of(block) {
                            self.visited.insert(node);
                        }
                    }
                    let join = plan.join().cloned();
                    return Ok(one(
                        Region::Guard {
                            prefix,
                            plan,
                            body: Some(Box::new(body)),
                            finally_body: None,
                        },
                        join,
                    ));
                }
                let reason = FallbackReason::Guard {
                    pass: None,
                    code: "jre_guard_body",
                    at: current.bci(),
                    message: "the nested synchronized body has no complete bounded structure"
                        .into(),
                };
                return Ok(gap(prefix, plan.owned().to_vec(), reason, None));
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
            if let Some(reason) = leaving {
                // P3 2.4: the two edges this walk has always refused — an exception edge and a
                // subroutine entry — are where the guarded regions live. A rule that proves one
                // claims every block the statement owns, and the walk continues after it.
                if let Some(verdict) = guard_verdict.take() {
                    match verdict {
                        crate::guard::Verdict::Claimed(plan) => {
                            for block in plan.owned() {
                                if let Some(node) = self.view.index_of(block) {
                                    self.visited.insert(node);
                                }
                            }
                            let join = plan.join().cloned();
                            return Ok(one(
                                Region::Guard {
                                    prefix,
                                    plan,
                                    body: None,
                                    finally_body: None,
                                },
                                join,
                            ));
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
                    && frame
                        .own_finally
                        .is_none_or(|row| !self.finally_edges_accounted(&current, row))
                    && frame
                        .segmented_finally_rows
                        .is_none_or(|rows| !self.segmented_finally_edges_accounted(&current, rows))
                    && frame.multi_return_finally_rows.is_none_or(|rows| {
                        !self.multi_return_finally_edges_accounted(&current, rows)
                    })
                    && self
                        .own_monitor
                        .as_ref()
                        .is_none_or(|body| !self.monitor_edges_accounted(&current, &body.owned))
                    && (frame.own_try.is_none()
                        || !self.edges_accounted_by_catches(&current, frame))
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
                            || frame.shared_tail == Some(node)
                            || frame.loop_exit == Some(node)
                            || frame
                                .loop_targets
                                .iter()
                                .any(|target| target.exits.contains(&node))
                            || frame
                                .loop_targets
                                .iter()
                                .any(|target| target.break_target == Some(node))
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
                    if let Some(region) =
                        self.shared_tail_early_return(&prefix, &branch, branch_bci, frame)?
                    {
                        let next = frame
                            .shared_tail
                            .and_then(|tail| self.view.id_of(tail).cloned());
                        return Ok(one(region, next));
                    }
                    if let Some((regions, next)) =
                        self.short_circuit_value(&prefix, &branch, branch_bci, frame)?
                    {
                        return Ok((regions, next));
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
                    let then_node = self.view.index_of(&fall_through);
                    let else_node = self.view.index_of(&taken);
                    if !frame.loop_targets.is_empty() {
                        charge(
                            self.budget,
                            CountedBudgetDimension::AnalysisSteps,
                            u64::try_from(self.canonical.edges().len()).unwrap_or(u64::MAX),
                            Some(branch_bci),
                        )?;
                    }
                    let loop_bridge_join = frame.loop_targets.last().and_then(|target| {
                        let boundary = frame.boundary?;
                        let exit = self.view.id_of(target.break_target?)?;
                        let blocks = self.view.loop_entered_at(target.header)?.blocks();
                        let through_bridge = [then_node, else_node]
                            .into_iter()
                            .flatten()
                            .any(|arm| self.loop_exit_bridge(node, arm, exit, blocks));
                        let through_latch = [then_node, else_node]
                            .into_iter()
                            .flatten()
                            .any(|arm| blocks.contains(&arm) && self.view.reaches(arm, boundary));
                        (frame.switch_join.is_none() && through_bridge && through_latch).then(
                            || {
                                // A body that holds two loop-jump edges — one breaking out, one
                                // keeping the loop — has no single graph join: the post-dominator
                                // was an enclosing loop's continue target and the bridge proof is
                                // what classified the arms. When a proved for-header owns an
                                // update block apart from the header, the loop-side arm's
                                // statements end there and the join is that block, so the walk
                                // stops at the update instead of absorbing it. A loop whose
                                // continue target *is* its header — a `while`/endless loop, where
                                // the update is the body's own last statement — keeps the
                                // boundary: the header is where the loop's arms regroup, and a
                                // join placed on it would end the body walk early.
                                if target.continue_target != target.header
                                    && target.continue_target != boundary
                                    && blocks.contains(&target.continue_target)
                                {
                                    target.continue_target
                                } else {
                                    boundary
                                }
                            },
                        )
                    });
                    let fragmented_handler_join = self.fragmented.as_ref().and_then(|proof| {
                        let boundary = frame.boundary?;
                        let [a, b] = [then_node?, else_node?];
                        (frame.own_try.is_some()
                            && proof.exceptional_blocks.contains(&node)
                            && proof.loop_header == frame.loop_targets.last()?.header
                            && (self.view.reaches(a, boundary) != self.view.reaches(b, boundary))
                            && self.view.reaches(a, self.view.index_of(&proof.update)?)
                            && self.view.reaches(b, self.view.index_of(&proof.update)?))
                        .then_some(boundary)
                    });
                    let join_node =
                        if let Some(boundary) = fragmented_handler_join.or(loop_bridge_join) {
                            Some(boundary)
                        } else if post_join.is_some_and(|join| {
                            frame
                                .loop_targets
                                .iter()
                                .any(|target| target.break_target == Some(join))
                        }) {
                            frame.switch_join.or(post_join)
                        } else {
                            post_join
                        };
                    let shared_join = if join_node.is_none() {
                        if self.return_is_boolean
                            && frame.shared_tail.is_none()
                            && frame.boundary.is_none()
                            && frame.scope.is_none()
                            && frame.case_entries.is_none()
                            && frame.own_try.is_none()
                            && frame.own_finally.is_none()
                            && frame.loop_exit.is_none()
                            && frame.switch_join.is_none()
                            && frame.loop_targets.is_empty()
                        {
                            self.shared_forward_join(node, then_node, else_node, branch_bci)?
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let join_node = join_node.or(shared_join);
                    // A branch inside a loop body whose join none of the readings above states
                    // still states its own edges: the second loop jump pulled the arms' meeting
                    // point past an enclosing loop's continue target (the post-dominator was
                    // filtered) and no exit bridge exists. One arm that is a single `goto` onto
                    // *this* loop's own continue target is a `continue` edge, and the other arm
                    // staying a block of the loop is the loop-side continuation: the continue
                    // target is the join both arms route to — the same reading the one-jump body
                    // takes from its post-dominator, stated here from the edge itself.
                    let continue_target_join = join_node
                        .is_none()
                        .then(|| {
                            frame.loop_targets.last().and_then(|target| {
                                let blocks =
                                    self.view.loop_entered_at(target.header)?.blocks().clone();
                                [then_node, else_node]
                                    .into_iter()
                                    .flatten()
                                    .find(|arm| {
                                        self.loop_continue_bridge(
                                            node,
                                            *arm,
                                            target.continue_target,
                                            &blocks,
                                        ) || frame.loop_targets[..frame.loop_targets.len() - 1]
                                            .iter()
                                            .any(|enclosing| {
                                                [
                                                    enclosing.break_target,
                                                    Some(enclosing.continue_target),
                                                ]
                                                .into_iter()
                                                .flatten()
                                                .any(|destination| {
                                                    // A bare transfer onto an *enclosing*
                                                    // loop's break or continue destination —
                                                    // a labeled `continue outer`/`break outer`
                                                    // edge of this two-edge body, whose
                                                    // destination leaves this loop entirely.
                                                    !blocks.contains(&destination)
                                                        && self.loop_break_transfer(
                                                            node,
                                                            *arm,
                                                            destination,
                                                            &blocks,
                                                        )
                                                })
                                            })
                                    })
                                    .filter(|arm| {
                                        // The sibling arm stays the loop's continuation: each of
                                        // its routes ends at a destination this frame's loops own
                                        // — this loop's continue target, or the break/continue
                                        // destination of an enclosing one (the second jump of a
                                        // two-edge body) — and none runs off the method, which is
                                        // a `return`'s shape and not this dispatch's.
                                        let welcome: BTreeSet<usize> = frame
                                            .loop_targets
                                            .iter()
                                            .flat_map(|target| {
                                                target
                                                    .break_target
                                                    .into_iter()
                                                    .chain([target.continue_target])
                                            })
                                            .chain([target.continue_target])
                                            .collect();
                                        [then_node, else_node].into_iter().flatten().any(|other| {
                                            other != *arm
                                                && blocks.contains(&other)
                                                && self.loop_side_routes(other, &welcome)
                                        })
                                    })
                                    .map(|_| target.continue_target)
                            })
                        })
                        .flatten();
                    let join_node = join_node.or(continue_target_join);
                    // An `if`/`else if` ladder whose one arm terminates the method has **no**
                    // post-dominator: the returning arm never passes the block the other arms
                    // regroup on, so none of the readings above states a join and both arm walks
                    // would run on to the frame's own boundary. Inside a loop that boundary is the
                    // header, and the block the arms really meet at — the loop's own latch — is
                    // then claimed by whichever arm walks first while the second re-enters it
                    // ([`FallbackReason::Loop`]); the completed tree owns that block twice and the
                    // whole method is refused (`jre_region_ownership_overlap`). The join is a fact
                    // of the graph all the same, and [`Self::ladder_join`] is the reading of it.
                    let ladder_join = if join_node.is_none() {
                        self.ladder_join(node, then_node, else_node, frame, branch_bci)?
                    } else {
                        None
                    };
                    let ladder_elected = ladder_join.is_some();
                    let join_node = join_node.or(ladder_join);
                    // The last two-edge reading: the graph's join already left the loop (the
                    // breaking arm's route exits it), one successor is the proved exit transfer
                    // of a loop this frame knows, and the other successor stays in the loop and
                    // its continuation reaches a *nested* loop header before this loop's own
                    // header. The in-loop successor is then where the body's statements continue
                    // — the one-armed shape the existing join reading below writes when a
                    // successor *is* the join — and electing it keeps the nested loop and the
                    // code around it inside the body walk instead of dropping them after a join
                    // no block of the loop holds.
                    let in_loop_successor_join = (|| {
                        let target = frame.loop_targets.last()?;
                        let join_leaves_loop = join_node.is_none_or(|join| {
                            frame
                                .scope
                                .as_ref()
                                .is_some_and(|scope| !scope.contains(&join))
                        });
                        if !join_leaves_loop {
                            return None;
                        }
                        let blocks = self.view.loop_entered_at(target.header)?.blocks().clone();
                        [then_node, else_node]
                            .into_iter()
                            .flatten()
                            .find(|stay| {
                                blocks.contains(stay)
                                    && self.nested_loop_ahead(*stay, target.header, &blocks)
                            })
                            .and_then(|stay| {
                                [then_node, else_node]
                                    .into_iter()
                                    .flatten()
                                    .find(|exit| {
                                        *exit != stay
                                            && frame.loop_targets.iter().any(|known| {
                                                known.break_target.is_some_and(|destination| {
                                                    self.loop_break_transfer(
                                                        node,
                                                        *exit,
                                                        destination,
                                                        &blocks,
                                                    )
                                                })
                                            })
                                    })
                                    .map(|_| stay)
                            })
                    })();
                    // This reading **overrides** a join the graph stated outside the loop: a
                    // join no block of the body holds cannot close a statement inside it, while
                    // the stay successor is a block the body walk continues at. The readings
                    // that elected a join *inside* the loop (an exit bridge's boundary, a
                    // continue target) return `None` above and keep their join.
                    let join_node = in_loop_successor_join.or(join_node);
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
                    let forward_then = shared_join.is_none()
                        && self.forward_join(node, then_node, else_node, frame);
                    let forward_else = shared_join.is_none()
                        && self.forward_join(node, else_node, then_node, frame);
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
                        let mut arm_frame = frame.arm(Some(join_node), Some(branch_bci));
                        if shared_join.is_some() && frame.shared_tail.is_none() {
                            arm_frame.shared_tail = Some(join_node);
                        }
                        let arm_before = self.visited.clone();
                        let (mut arm_run, arm_next) = self.region_at(walk, &arm_frame)?;
                        if let Some(next) = arm_next.as_ref()
                            && self.view.index_of(next) != Some(join_node)
                        {
                            let mut continued =
                                self.continue_early_return_arm(&mut arm_run, next, &arm_frame)?;
                            if !continued {
                                continued = self.continue_prefixed_loop_arm(
                                    &mut arm_run,
                                    next,
                                    walk,
                                    &branch,
                                    &arm_frame,
                                    &arm_before,
                                )?;
                            }
                            if !continued {
                                self.unclosed_tail_at.get_or_insert(branch.bci());
                                let reason = FallbackReason::ArmsDoNotMeet {
                                    block_bci: branch.bci(),
                                };
                                return Ok(gap(prefix, vec![branch], reason, None));
                            }
                        }
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
                    let mut arm_frame = frame.arm(join_node, Some(branch_bci));
                    if let Some(tail) = shared_join.filter(|_| frame.shared_tail.is_none()) {
                        arm_frame.shared_tail = Some(tail);
                    }
                    let before_arms = self.visited.clone();
                    let mut then_frame = arm_frame.clone();
                    let mut else_frame = arm_frame.clone();
                    let direct_arm = frame.boundary.is_none() && frame.if_arm.is_none();
                    let second_level = frame.if_arm.is_some_and(|(_, _, outer)| outer.is_none())
                        && frame.boundary.is_some();
                    if (direct_arm || second_level)
                        && frame.scope.is_none()
                        && frame.loop_targets.is_empty()
                        && frame.own_try.is_none()
                        && frame.own_finally.is_none()
                        && frame.case_entries.is_none()
                    {
                        let outer = if second_level { frame.boundary } else { None };
                        if let Some(entry) = then_node {
                            then_frame.if_arm = Some((node, entry, outer));
                        }
                        if let Some(entry) = else_node {
                            else_frame.if_arm = Some((node, entry, outer));
                        }
                    }
                    let (mut then_run, mut then_next) =
                        self.region_at(&fall_through, &then_frame)?;
                    let (mut else_run, mut else_next) = self.region_at(&taken, &else_frame)?;
                    // Test5's non-returning arm reaches an ordinary do-while header
                    // after its list-allocation block. The generic arm walk stops at
                    // that header, so finish this certified arm within the bounded
                    // two-segment finally scope before comparing the arms.
                    if frame.multi_return_finally_rows.is_some() {
                        self.continue_multi_return_loop_arm(
                            &mut then_run,
                            &mut then_next,
                            &then_frame,
                        )?;
                        self.continue_multi_return_loop_arm(
                            &mut else_run,
                            &mut else_next,
                            &else_frame,
                        )?;
                    }
                    let then_loop = self.continue_effectful_loop_arm(
                        &mut then_run,
                        &mut then_next,
                        &then_frame,
                    )?;
                    let else_loop = if then_loop {
                        self.continue_effectful_loop_arm(
                            &mut else_run,
                            &mut else_next,
                            &else_frame,
                        )?
                    } else {
                        false
                    };
                    // A two-edge body branch whose elected join is the loop's own header or
                    // continue target: the jump arm never returns there, and the continuing
                    // arm's own nested `if` already ended at its join — a block of this loop
                    // the body walk resumes at. Electing that block keeps the rest of the body
                    // (the nested loop, and the update after it) inside the walk instead of
                    // ending it at the loop's header with those blocks unclaimed.
                    let continuing_join = (|| {
                        let then_jump = then_next.is_none()
                            && matches!(then_run.last(), Some(Region::LoopBreak { .. }));
                        let else_jump = else_next.is_none()
                            && matches!(else_run.last(), Some(Region::LoopBreak { .. }));
                        if then_jump == else_jump {
                            return None;
                        }
                        let (run, next) = if then_jump {
                            (&else_run, &else_next)
                        } else {
                            (&then_run, &then_next)
                        };
                        let elected = join_node?;
                        let last = frame.loop_targets.last()?;
                        if elected != last.header && elected != last.continue_target {
                            return None;
                        }
                        let Region::If {
                            join: Some(nested), ..
                        } = run.last()?
                        else {
                            return None;
                        };
                        if next.as_ref() != Some(nested) {
                            return None;
                        }
                        let nested_node = self.view.index_of(nested)?;
                        if nested_node == elected
                            || nested_node == last.header
                            || self.visited.contains(&nested_node)
                            || frame.boundary == Some(nested_node)
                        {
                            return None;
                        }
                        let blocks = self.view.loop_entered_at(last.header)?.blocks();
                        if !blocks.contains(&nested_node)
                            || !self.nested_loop_ahead(nested_node, last.header, blocks)
                        {
                            return None;
                        }
                        Some(nested.clone())
                    })();
                    // The shared-latch two-edge body: this loop's own `continue` edge and the
                    // nested loop's exit edge route onto the same back-edge destination, so
                    // the arms' post-dominator is that destination and electing it ends the
                    // body walk with the nested loop's blocks unclaimed. The structural facts
                    // — each read from the graph, none from the addresses — that admit
                    // re-electing the join to where the continuing arm's own statements end:
                    // the elected join is this loop's continue destination (the update block
                    // a proved for-header owns, or the header of a loop none does); one arm
                    // is a goto bridge onto that destination (the edge a `continue` spells,
                    // exclusively owned by this branch); the other arm's walk ended at a
                    // nested loop header whose blocks this loop's body holds; and every exit
                    // of that nested loop lands on one of this loop's own latch blocks — the
                    // two edges' routes onto the one back edge, whether they name the
                    // destination itself (an update block both edges share) or a latch whose
                    // own transfer is the header. The jump arm then carries its own edge as
                    // the loop continue — the attribution by edge semantics the readings
                    // above use, with the same exclusive-ownership classifier, not a third
                    // one.
                    let shared_latch_join = (|| {
                        let last = frame.loop_targets.last()?;
                        let elected = join_node?;
                        let destination = last.continue_target;
                        if elected != destination {
                            return None;
                        }
                        let loop_of = self.view.loop_entered_at(last.header)?;
                        let latches = loop_of.latches();
                        let blocks = loop_of.blocks();
                        let (bridge, jump_is_then) = [then_node, else_node]
                            .into_iter()
                            .flatten()
                            .find_map(|entry| {
                            self.loop_continue_bridge(node, entry, destination, blocks)
                                .then_some((entry, then_node == Some(entry)))
                        })?;
                        let (run, next) = if jump_is_then {
                            (&then_run, &else_next)
                        } else {
                            (&else_run, &then_next)
                        };
                        // The jumping arm is nothing but its bridge block: the one-transfer
                        // run the re-elected join no longer stands for.
                        let [Region::Straight { blocks: bridge_run }] = run.as_slice() else {
                            return None;
                        };
                        if bridge_run.len() != 1
                            || self.view.index_of(bridge_run.first()?) != Some(bridge)
                        {
                            return None;
                        }
                        let source_bci = self.terminal_bci(bridge_run.first()?)?;
                        let nested = next.as_ref()?;
                        let nested_node = self.view.index_of(nested)?;
                        if self.visited.contains(&nested_node)
                            || frame.boundary == Some(nested_node)
                        {
                            return None;
                        }
                        let Some(nested_of) = self.view.loop_entered_at(nested_node) else {
                            return None;
                        };
                        let nested_blocks = nested_of.blocks();
                        if !nested_blocks.is_subset(blocks) {
                            return None;
                        }
                        let nested_exits: BTreeSet<usize> = nested_blocks
                            .iter()
                            .flat_map(|member| self.view.successors(*member))
                            .filter(|successor| !nested_blocks.contains(successor))
                            .collect();
                        // Every exit lands on a latch of this loop — directly (the test
                        // exit onto a shared update block, or onto the back-edge transfer a
                        // loop whose continue target is its header ends in) or through its
                        // own one transfer (an inner `break` goto onto that same block).
                        // Anything else leaves the loop past its back edge, and no exit may:
                        // this shape's post-dominator is the continue destination only
                        // because both routes regroup there.
                        let mut landings: BTreeSet<usize> = BTreeSet::new();
                        for exit in &nested_exits {
                            let landing = if latches.contains(exit) {
                                *exit
                            } else {
                                let successors = self.view.successors(*exit);
                                if successors.len() != 1 || !latches.contains(&successors[0]) {
                                    return None;
                                }
                                successors[0]
                            };
                            landings.insert(landing);
                        }
                        // The loop's back edges are exactly the routes this shape accounts
                        // for: the landings, plus the bridge itself when a `continue` in a
                        // loop whose continue target is its header is a back edge of its own.
                        // Any other back edge is a jump this reading does not classify.
                        let mut accounted: BTreeSet<usize> = landings;
                        if self.view.successors(bridge) == [last.header] {
                            accounted.insert(bridge);
                        }
                        let held: BTreeSet<usize> = latches.iter().copied().collect();
                        if held != accounted {
                            return None;
                        }
                        Some((jump_is_then, source_bci))
                    })();
                    let continuing_join_elected = continuing_join.is_some();
                    let shared_latch_elected = shared_latch_join.is_some();
                    let shared_latch = shared_latch_join.and_then(|(jump_is_then, source_bci)| {
                        let loop_header =
                            self.view.id_of(frame.loop_targets.last()?.header)?.clone();
                        let run = if jump_is_then {
                            &mut then_run
                        } else {
                            &mut else_run
                        };
                        *run = vec![Region::LoopContinue {
                            source_bci,
                            loop_header,
                        }];
                        // The continuing arm's trailing straight run is this loop's own
                        // continuation, and it goes back to the body walk: a counted loop's
                        // header clause reads its initialisation from a preceding sibling
                        // statement, not from inside an arm. The run's blocks leave the arm's
                        // ownership with it — the walk re-claims them at the re-elected join,
                        // and a claimed block the walk re-enters would be quoted instead.
                        let continuing = if jump_is_then {
                            &mut else_run
                        } else {
                            &mut then_run
                        };
                        let [Region::Straight { blocks: lead }] = continuing.as_slice() else {
                            return None;
                        };
                        let join_block = lead.first()?.clone();
                        for block in lead {
                            let node = self.view.index_of(block)?;
                            if !self.visited.remove(&node) {
                                return None;
                            }
                        }
                        *continuing = Vec::new();
                        Some(join_block)
                    });
                    let join = continuing_join.or(shared_latch).or(join);
                    let join_node = join.as_ref().and_then(|join| self.view.index_of(join));
                    // A nested value can meet at its own join before this arm meets the outer
                    // join. Keep that intervening straight run inside the *same* arm and frame.
                    // The helper refuses a partial or multiply entered continuation before any
                    // of its visited nodes can become a published owner.
                    let then_at_join =
                        continuing_join_elected && then_next.as_ref() == join.as_ref();
                    let then_continued = then_at_join
                        || self.continue_inner_join_arm(
                            &mut then_run,
                            then_next.as_ref(),
                            &arm_frame,
                        )?;
                    let else_at_join =
                        continuing_join_elected && else_next.as_ref() == join.as_ref();
                    let else_continued = if then_continued {
                        else_at_join
                            || self.continue_inner_join_arm(
                                &mut else_run,
                                else_next.as_ref(),
                                &arm_frame,
                            )?
                    } else {
                        false
                    };
                    if !then_loop || !else_loop || !then_continued || !else_continued {
                        let entered: Vec<_> = self
                            .visited
                            .difference(&before_arms)
                            .filter_map(|node| self.view.id_of(*node).cloned())
                            .chain(then_next)
                            .chain(else_next)
                            .collect();
                        self.visited = before_arms;
                        let reason = FallbackReason::ArmsDoNotMeet {
                            block_bci: branch.bci(),
                        };
                        let next = self.unclaimed_join(join.as_ref());
                        return Ok(gap(prefix, gap_blocks(&branch, entered), reason, next));
                    }
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
                        let certified_continue = fragmented_handler_join == Some(join_node)
                            && ((then_meets
                                && else_next.is_none()
                                && matches!(else_run.last(), Some(Region::LoopContinue { .. })))
                                || (else_meets
                                    && then_next.is_none()
                                    && matches!(
                                        then_run.last(),
                                        Some(Region::LoopContinue { .. })
                                    )));
                        let both_end = !then_meets && !else_meets;
                        let local_switch_join = frame.switch_join == Some(join_node);
                        // The arm that took a loop-jump edge outright: its walk ended (no
                        // continuation block) in a proved `LoopBreak`, or in nothing but the
                        // `goto` block whose one transfer is the join — the `continue` edge the
                        // goto spells when the join is this loop's own continue target.
                        let takes_loop_edge = |run: &[Region]| -> bool {
                            if matches!(run.last(), Some(Region::LoopBreak { .. })) {
                                return true;
                            }
                            let [Region::Straight { blocks }] = run else {
                                return false;
                            };
                            blocks.len() == 1
                                && self.view.index_of(&blocks[0]).is_some_and(|bridge| {
                                    self.view.successors(bridge) == [join_node]
                                })
                        };
                        let then_breaks = then_next.is_none()
                            && matches!(then_run.last(), Some(Region::LoopBreak { .. }));
                        let else_breaks = else_next.is_none()
                            && matches!(else_run.last(), Some(Region::LoopBreak { .. }));
                        // A proved `continue` transfer ends its arm the same way a `break` does:
                        // the edge left for its target loop's destination and never comes back to
                        // this join. It is the loop-side jump of a two-edge body whose other arm
                        // is the one that meets the join.
                        let then_jumps = then_next.is_none()
                            && matches!(
                                then_run.last(),
                                Some(Region::LoopBreak { .. } | Region::LoopContinue { .. })
                            );
                        let else_jumps = else_next.is_none()
                            && matches!(
                                else_run.last(),
                                Some(Region::LoopBreak { .. } | Region::LoopContinue { .. })
                            );
                        let then_edge = then_next.is_none() && takes_loop_edge(&then_run);
                        let else_edge = else_next.is_none() && takes_loop_edge(&else_run);
                        if !(then_meets && else_meets)
                            && !certified_continue
                            && !both_end
                            && !(frame.loop_targets.last().is_some_and(|target| {
                                // The join is this loop's own continue target and one arm left by
                                // a proved loop-jump edge while the other meets it — the two-edge
                                // body's shape. The continue target is the join whether the loop
                                // spells `while` (it *is* the header this frame ends at) or `for`
                                // (a proved update block the arms stop at instead), and the
                                // meeting arm may end *at* the join rather than routing through
                                // it, its own walk having stopped exactly there.
                                let then_at_join = then_next.as_ref().is_some_and(|next| {
                                    self.view.index_of(next) == Some(join_node)
                                });
                                let else_at_join = else_next.as_ref().is_some_and(|next| {
                                    self.view.index_of(next) == Some(join_node)
                                });
                                target.continue_target == join_node
                                    && (((then_meets && else_jumps) || (else_meets && then_jumps))
                                        || (then_edge && else_at_join)
                                        || (else_edge && then_at_join))
                            }))
                            && !(local_switch_join
                                && ((then_meets && else_breaks) || (else_meets && then_breaks)))
                            // The shared-latch re-election proved both arms' edges from the graph
                            // already — the continue bridge by exclusive ownership, the nested
                            // loop by containment and its exit set — and the one arm ends at
                            // this loop's own latch, a destination the forward reachability of
                            // the other arm's continuation never states. The re-election, not a
                            // reaches reading, is what the join stands on.
                            && !shared_latch_elected
                            // The ladder's own election proved the same question from the arms'
                            // routes: each arm reaches the join, and no route of either reaches
                            // the frame's boundary without passing through it, so the reading the
                            // refusal below states ("both arms must meet the join, or leave the
                            // method") is already established — the arm that does not meet it
                            // leaves the method, which is the early-return leaf the tree states.
                            && !ladder_elected
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

    /// A nested boolean diamond may finish at its own `ireturn` after the enclosing guard's
    /// early-return join. Admit only that single, closed return block: both producer arms must be
    /// its exact normal predecessors, and the recursive walk must claim precisely that block.
    fn continue_early_return_arm(
        &mut self,
        run: &mut Vec<Region>,
        next: &CanonicalBlockId,
        frame: &Frame,
    ) -> Result<bool, StopReason> {
        let [
            Region::If {
                branch,
                then_arm,
                else_arm,
                join: Some(inner_join),
                ..
            },
        ] = run.as_slice()
        else {
            return Ok(false);
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
            return Ok(false);
        };
        let (Some(then_end), Some(else_end), Some(node)) = (
            then_blocks.last(),
            else_blocks.last(),
            self.view.index_of(next),
        ) else {
            return Ok(false);
        };
        if inner_join != next
            || next.path() != branch.path()
            || next.bci() <= branch.bci()
            || !self.return_is_boolean
            || self.visited.contains(&node)
            || frame.stops_at(node)
            || !self.view.successors(node).is_empty()
            || self.leaving_edge(next).is_some()
            || !self.ssa.block(next).is_some_and(|block| {
                block.instructions().last().is_some_and(|instruction| {
                    instruction.opcode() == 0xac
                        && matches!(
                            self.operations.get(instruction.bci()),
                            Some(Operation::Return)
                        )
                })
            })
        {
            return Ok(false);
        }
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(self.canonical.edges().len()).unwrap_or(u64::MAX),
            Some(next.bci()),
        )?;
        let incoming = self
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.to() == next)
            .map(|edge| (edge.kind(), edge.from().clone()))
            .collect::<Vec<_>>();
        if !exact_normal_predecessors(&incoming, &[then_end.clone(), else_end.clone()]) {
            return Ok(false);
        }
        let before = self.visited.clone();
        let (tail, tail_next) = self.region_at(next, frame)?;
        let newly_visited = self
            .visited
            .difference(&before)
            .filter_map(|node| self.view.id_of(*node).cloned())
            .collect::<BTreeSet<_>>();
        if tail_next.is_some()
            || !matches!(tail.as_slice(), [Region::Straight { blocks }] if blocks == &[next.clone()])
            || !continuation_claims_are_exact(&[next.clone()], &newly_visited)
        {
            return Ok(false);
        }
        run.extend(tail);
        Ok(true)
    }

    /// The one physical edge by which a closed loop arm reaches its enclosing `If`'s join.
    /// A region alone does not prove this: its declared exit must agree with every canonical
    /// edge and with the natural loop's complete set of owners.
    fn loop_arm_join_source(
        &mut self,
        arm: &Region,
        branch: &CanonicalBlockId,
        entry_source: &CanonicalBlockId,
        join: &CanonicalBlockId,
        frame: &Frame,
    ) -> Result<Option<CanonicalBlockId>, StopReason> {
        fn closed_body(region: &Region) -> bool {
            match region {
                Region::Straight { .. } => true,
                Region::Sequence { regions } => regions.iter().all(closed_body),
                Region::If {
                    then_arm,
                    else_arm,
                    join: Some(_),
                    ..
                } => closed_body(then_arm) && closed_body(else_arm),
                _ => false,
            }
        }

        let Region::Loop {
            header,
            body,
            exit: Some(exit),
            gateway_origins,
            form,
            for_header,
            ..
        } = arm
        else {
            return Ok(None);
        };
        if exit != join || !body.iter().all(closed_body) {
            return Ok(None);
        }
        let Some(header_node) = self.view.index_of(header) else {
            return Ok(None);
        };
        let Some(natural_loop) = self.view.loop_entered_at(header_node) else {
            return Ok(None);
        };
        let blocks = arm.blocks();
        let owners: BTreeSet<_> = blocks.iter().map(|block| (*block).clone()).collect();
        let owner_nodes: BTreeSet<_> = owners
            .iter()
            .filter_map(|block| self.view.index_of(block))
            .collect();
        if owners.len() != blocks.len()
            || owner_nodes != *natural_loop.blocks()
            || owners.iter().any(|block| {
                block.path() != branch.path()
                    || (entry_source == branch
                        && (block.bci() <= branch.bci() || block.bci() >= join.bci()))
                    || self.view.index_of(block).is_none_or(|node| {
                        !self.visited.contains(&node)
                            || frame
                                .scope
                                .as_ref()
                                .is_some_and(|scope| !scope.contains(&node))
                    })
            })
        {
            return Ok(None);
        }
        let allowed_latch_origin = if gateway_origins.is_empty() {
            true
        } else if *form == LoopForm::While && gateway_origins.len() == 1 {
            self.implicit_tail_latch_origin(header_node, body, for_header.as_ref())?
                == gateway_origins.first().copied()
        } else {
            false
        };
        if !allowed_latch_origin {
            return Ok(None);
        }
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(self.canonical.edges().len().saturating_add(owners.len()))
                .unwrap_or(u64::MAX),
            Some(header.bci()),
        )?;
        let mut entry_count = 0;
        let mut exit_source = None;
        for edge in self.canonical.edges() {
            poll(self.budget, Some(header.bci()))?;
            let from_loop = owners.contains(edge.from());
            let to_loop = owners.contains(edge.to());
            if !from_loop && !to_loop {
                continue;
            }
            if edge.kind() != CanonicalEdgeKind::Normal {
                return Ok(None);
            }
            if !from_loop {
                if edge.from() != entry_source || edge.to() != header {
                    return Ok(None);
                }
                entry_count += 1;
            } else if !to_loop {
                if edge.to() != join || exit_source.replace(edge.from().clone()).is_some() {
                    return Ok(None);
                }
            }
        }
        if entry_count != 1 {
            return Ok(None);
        }
        for block in &owners {
            poll(self.budget, Some(block.bci()))?;
            if self.view.successor_ids(block).is_empty() {
                return Ok(None);
            }
        }
        Ok(exit_source)
    }

    /// Resume the direct arm's straight prefix at one fresh natural-loop header. The loop may
    /// leave directly at the arm join or through one exclusive straight tail. Every participant
    /// is checked against the canonical normal edges before it is attached to the branch.
    fn continue_prefixed_loop_arm(
        &mut self,
        run: &mut Vec<Region>,
        next: &CanonicalBlockId,
        entry: &CanonicalBlockId,
        branch: &CanonicalBlockId,
        frame: &Frame,
        visited_before: &BTreeSet<usize>,
    ) -> Result<bool, StopReason> {
        let [Region::Straight { blocks: prefix }] = run.as_slice() else {
            return Ok(false);
        };
        let (Some(join_node), Some(header_node)) = (frame.boundary, self.view.index_of(next))
        else {
            return Ok(false);
        };
        if prefix.is_empty()
            || prefix.first() != Some(entry)
            || !self.view.is_loop_header(header_node)
            || !prefixed_loop_header_is_fresh(header_node, frame, &self.visited)
            || prefix.iter().any(|block| block.path() != branch.path())
        {
            return Ok(false);
        }
        let prefix = prefix.clone();
        let boundary = self.view.id_of(join_node).cloned();
        let Some(boundary) = boundary else {
            return Ok(false);
        };

        if !self.closed_straight_chain(&prefix, branch, next, branch, frame)? {
            return Ok(false);
        }
        let (loop_run, loop_next) = self.region_at(next, frame)?;
        let [
            loop_region @ Region::Loop {
                exit: Some(exit), ..
            },
        ] = loop_run.as_slice()
        else {
            return Ok(false);
        };
        if loop_next.as_ref() != Some(exit) {
            return Ok(false);
        }
        let Some(exit_source) = self.loop_arm_join_source(
            loop_region,
            branch,
            prefix.last().expect("nonempty straight prefix"),
            exit,
            frame,
        )?
        else {
            return Ok(false);
        };
        let mut tail = Vec::new();
        let mut participants = prefix.clone();
        participants.extend(loop_region.blocks().into_iter().cloned());
        if exit != &boundary {
            let before_tail = self.visited.clone();
            let (tail_run, tail_next) = self.region_at(exit, frame)?;
            let [
                Region::Straight {
                    blocks: tail_blocks,
                },
            ] = tail_run.as_slice()
            else {
                return Ok(false);
            };
            if tail_blocks.is_empty()
                || tail_next.is_some()
                || !self.closed_straight_chain(
                    tail_blocks,
                    &exit_source,
                    &boundary,
                    branch,
                    frame,
                )?
            {
                return Ok(false);
            }
            let newly_visited = self
                .visited
                .difference(&before_tail)
                .copied()
                .collect::<BTreeSet<_>>();
            let tail_nodes = tail_blocks
                .iter()
                .filter_map(|block| self.view.index_of(block))
                .collect::<Vec<_>>();
            if !continuation_claims_are_exact(&tail_nodes, &newly_visited) {
                return Ok(false);
            }
            participants.extend(tail_blocks.iter().cloned());
            tail = tail_run;
        }
        let newly_visited = self
            .visited
            .difference(visited_before)
            .copied()
            .collect::<BTreeSet<_>>();
        let participant_nodes = participants
            .iter()
            .filter_map(|block| self.view.index_of(block))
            .collect::<Vec<_>>();
        if !continuation_claims_are_exact(&participant_nodes, &newly_visited) {
            return Ok(false);
        }
        run.extend(loop_run);
        run.extend(tail);
        Ok(true)
    }

    /// Prove one straight chain from an exact predecessor to an exact successor. This is a
    /// bounded local edge scan: every inspected canonical edge and owned block is charged, and
    /// any side entry, exit, exceptional edge, or repeated owner refuses the chain.
    fn closed_straight_chain(
        &mut self,
        blocks: &[CanonicalBlockId],
        entry_source: &CanonicalBlockId,
        exit_target: &CanonicalBlockId,
        branch: &CanonicalBlockId,
        frame: &Frame,
    ) -> Result<bool, StopReason> {
        let Some(first) = blocks.first() else {
            return Ok(false);
        };
        let edges = self
            .canonical
            .edges()
            .iter()
            .map(|edge| (edge.kind(), edge.from().clone(), edge.to().clone()));
        if !exact_straight_chain_edges(
            blocks,
            entry_source,
            exit_target,
            edges,
            self.budget,
            Some(first.bci()),
        )? {
            return Ok(false);
        }
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(blocks.len()).unwrap_or(u64::MAX),
            Some(first.bci()),
        )?;
        for block in blocks {
            poll(self.budget, Some(block.bci()))?;
            if block.path() != branch.path()
                || self
                    .view
                    .index_of(block)
                    .is_none_or(|node| !self.visited.contains(&node) || frame.stops_at(node))
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn continue_multi_return_loop_arm(
        &mut self,
        run: &mut Vec<Region>,
        next: &mut Option<CanonicalBlockId>,
        frame: &Frame,
    ) -> Result<(), StopReason> {
        let Some(header) = next.as_ref() else {
            return Ok(());
        };
        let Some(node) = self.view.index_of(header) else {
            return Ok(());
        };
        if !self.view.is_loop_header(node) {
            return Ok(());
        }
        for _ in 0..2 {
            let Some(at) = next.as_ref() else { break };
            let previous = at.clone();
            let (part, following) = self.region_at(&previous, frame)?;
            if part.is_empty() || following.as_ref() == Some(&previous) {
                break;
            }
            run.extend(part);
            *next = following;
        }
        Ok(())
    }

    /// The first straight block of a direct `if` arm can stop on arrival at a loop header.
    /// Resume only the certified effectful loop and its one straight internal join, ending at
    /// the parent's boundary. The caller's visited snapshot rolls back a failed continuation.
    fn continue_effectful_loop_arm(
        &mut self,
        run: &mut Vec<Region>,
        next: &mut Option<CanonicalBlockId>,
        frame: &Frame,
    ) -> Result<bool, StopReason> {
        let (Some((_, entry, outer_boundary)), Some(header), Some(boundary)) =
            (frame.if_arm, next.as_ref(), frame.boundary)
        else {
            return Ok(true);
        };
        let Some(header_node) = self.view.index_of(header) else {
            return Ok(false);
        };
        if !self.view.is_loop_header(header_node) {
            return Ok(true);
        }
        let [Region::Straight { blocks }] = run.as_slice() else {
            return Ok(false);
        };
        if blocks.len() != 1 || self.view.index_of(&blocks[0]) != Some(entry) {
            return Ok(false);
        }
        let Some(loop_of) = self.view.loop_entered_at(header_node).cloned() else {
            return Ok(false);
        };
        if self
            .view
            .loop_is_irreducible(header_node, &self.catch_joins)
        {
            return Ok(false);
        }
        if self.depth >= MAX_REGION_DEPTH {
            return Err(StopReason::Interrupted {
                code: crate::stop::RECURSION_BOUND_CODE,
                at: Some(header.bci()),
            });
        }
        self.depth += 1;
        let certified = self.effectful_dual_exit_loop(header, header_node, &loop_of, frame);
        self.depth -= 1;
        let Some((loop_run, loop_next)) = certified? else {
            return Ok(false);
        };
        let [
            Region::Loop {
                form: LoopForm::Endless,
                exit: Some(join),
                ..
            },
        ] = loop_run.as_slice()
        else {
            return Ok(false);
        };
        if loop_next.as_ref() != Some(join) {
            return Ok(false);
        }
        if outer_boundary.is_some() && self.view.index_of(join) == Some(boundary) {
            let join = join.clone();
            run.extend(loop_run);
            *next = Some(join);
            return Ok(true);
        }
        let (tail, tail_next) = self.region_at(join, frame)?;
        let [
            Region::Straight {
                blocks: tail_blocks,
            },
        ] = tail.as_slice()
        else {
            return Ok(false);
        };
        let Some(join_node) = self.view.index_of(join) else {
            return Ok(false);
        };
        if tail_blocks != &[join.clone()]
            || tail_next.is_some()
            || self.view.successors(join_node) != [boundary]
        {
            return Ok(false);
        }
        run.extend(loop_run);
        run.extend(tail);
        *next = self.view.id_of(boundary).cloned();
        Ok(true)
    }

    /// Continue one nested `If`'s unclaimed forward join inside its enclosing arm. The only
    /// admitted tail is a closed straight run ending at the arm's original boundary. The visited
    /// snapshot is checked against its physical blocks before the run is attached to the arm.
    fn continue_inner_join_arm(
        &mut self,
        run: &mut Vec<Region>,
        next: Option<&CanonicalBlockId>,
        frame: &Frame,
    ) -> Result<bool, StopReason> {
        let Some(next) = next else { return Ok(true) };
        let Some(boundary) = frame.boundary else {
            return Ok(true);
        };
        if self.view.index_of(next) == Some(boundary) {
            return Ok(true);
        }
        // The inner `if` finished at a nested loop's header — or at the header or continue
        // target of the loop this frame walks a body of. The continuation is a region of its
        // own, never the straight tail this helper attaches: the arm keeps exactly the run it
        // walked and the caller resumes at that block, the loop's own walk claiming it next.
        if let Some(node) = self.view.index_of(next)
            && !self.visited.contains(&node)
            && (self.view.is_loop_header(node)
                || frame
                    .loop_targets
                    .last()
                    .is_some_and(|target| target.continue_target == node))
            && frame
                .scope
                .as_ref()
                .is_none_or(|scope| scope.contains(&node))
        {
            return Ok(true);
        }
        let Some(Region::If {
            branch,
            then_arm,
            else_arm,
            join: Some(inner_join),
            ..
        }) = run.last()
        else {
            return Ok(true);
        };
        if inner_join != next || next.path() != branch.path() || next.bci() <= branch.bci() {
            return Ok(false);
        }
        let straight_end = |arm: &Region| match arm {
            Region::Straight { blocks } => blocks.last().cloned(),
            _ => None,
        };
        // An arm that ends in a proved loop transfer never enters the join: its own edge left
        // for the target loop's break destination, so only the sibling arm's tail block is a
        // predecessor of the continuation.
        fn ends_in_loop_break(arm: &Region) -> bool {
            match arm {
                Region::LoopBreak { .. } => true,
                Region::Sequence { regions } => {
                    matches!(regions.last(), Some(Region::LoopBreak { .. }))
                }
                _ => false,
            }
        }
        let sources = match (then_arm.as_ref(), else_arm.as_ref()) {
            (Region::Straight { .. }, Region::Straight { .. }) => {
                vec![straight_end(then_arm), straight_end(else_arm)]
            }
            (Region::LoopBreak { .. }, Region::Straight { .. }) => vec![straight_end(else_arm)],
            (Region::Straight { .. }, Region::LoopBreak { .. }) => vec![straight_end(then_arm)],
            (arm, Region::Straight { .. }) if ends_in_loop_break(arm) => {
                vec![straight_end(else_arm)]
            }
            (Region::Straight { .. }, arm) if ends_in_loop_break(arm) => {
                vec![straight_end(then_arm)]
            }
            (Region::Loop { .. }, Region::Straight { .. }) => vec![
                self.loop_arm_join_source(then_arm, branch, branch, next, frame)?,
                straight_end(else_arm),
            ],
            (Region::Straight { .. }, Region::Loop { .. }) => vec![
                straight_end(then_arm),
                self.loop_arm_join_source(else_arm, branch, branch, next, frame)?,
            ],
            (Region::Sequence { .. }, Region::Straight { .. }) => {
                let Some(sources) = self.two_level_loop_join_sources(then_arm, branch, next)?
                else {
                    return Ok(false);
                };
                sources
                    .into_iter()
                    .map(Some)
                    .chain([straight_end(else_arm)])
                    .collect()
            }
            (Region::Straight { .. }, Region::Sequence { .. }) => {
                let Some(sources) = self.two_level_loop_join_sources(else_arm, branch, next)?
                else {
                    return Ok(false);
                };
                sources
                    .into_iter()
                    .map(Some)
                    .chain([straight_end(then_arm)])
                    .collect()
            }
            _ => return Ok(false),
        };
        let Some(sources) = sources.into_iter().collect::<Option<Vec<_>>>() else {
            return Ok(false);
        };
        let Some(next_node) = self.view.index_of(next) else {
            return Ok(false);
        };
        if self.visited.contains(&next_node)
            || frame
                .scope
                .as_ref()
                .is_some_and(|scope| !scope.contains(&next_node))
            || frame.stops_at_switch_boundary(next_node)
        {
            return Ok(false);
        }
        let already_visited = self.visited.clone();
        let (tail, tail_next) = self.region_at(next, frame)?;
        let [Region::Straight { blocks }] = tail.as_slice() else {
            return Ok(false);
        };
        if tail_next.is_some() || blocks.first() != Some(next) {
            return Ok(false);
        }
        let newly_visited: BTreeSet<_> = self
            .visited
            .difference(&already_visited)
            .filter_map(|node| self.view.id_of(*node).cloned())
            .collect();
        if !continuation_claims_are_exact(blocks, &newly_visited) {
            return Ok(false);
        }
        let owned: BTreeSet<_> = blocks.iter().cloned().collect();
        let Some(boundary_id) = self.view.id_of(boundary) else {
            return Ok(false);
        };
        if blocks.iter().any(|block| {
            block.path() != branch.path()
                || block.bci() >= boundary_id.bci()
                || self.view.index_of(block).is_none_or(|node| {
                    frame
                        .scope
                        .as_ref()
                        .is_some_and(|scope| !scope.contains(&node))
                })
        }) {
            return Ok(false);
        }
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(self.canonical.edges().len().saturating_add(blocks.len()))
                .unwrap_or(u64::MAX),
            Some(next.bci()),
        )?;
        let mut incoming: BTreeMap<CanonicalBlockId, Vec<(CanonicalEdgeKind, CanonicalBlockId)>> =
            BTreeMap::new();
        let mut outgoing: BTreeMap<CanonicalBlockId, Vec<(CanonicalEdgeKind, CanonicalBlockId)>> =
            BTreeMap::new();
        for edge in self.canonical.edges() {
            if owned.contains(edge.to()) {
                incoming
                    .entry(edge.to().clone())
                    .or_default()
                    .push((edge.kind(), edge.from().clone()));
            }
            if owned.contains(edge.from()) {
                outgoing
                    .entry(edge.from().clone())
                    .or_default()
                    .push((edge.kind(), edge.to().clone()));
            }
        }
        for (index, block) in blocks.iter().enumerate() {
            poll(self.budget, Some(block.bci()))?;
            let expected_in: Vec<_> = if index == 0 {
                sources.clone()
            } else {
                vec![blocks[index - 1].clone()]
            };
            let expected_out = blocks.get(index + 1).unwrap_or(boundary_id);
            let actual_in = incoming.get(block).map(Vec::as_slice).unwrap_or(&[]);
            let actual_out = outgoing.get(block).map(Vec::as_slice).unwrap_or(&[]);
            if !exact_normal_predecessors(actual_in, &expected_in)
                || actual_out.len() != 1
                || actual_out[0].0 != CanonicalEdgeKind::Normal
                || &actual_out[0].1 != expected_out
                || self.view.successor_ids(block) != [expected_out.clone()]
            {
                return Ok(false);
            }
        }
        run.extend(tail);
        Ok(true)
    }

    /// The only three-input continuation admitted here is a proved effectful two-exit loop
    /// preceded by its single direct `if`-arm entry. Its two physical exit blocks and the sibling
    /// arm, checked by the caller, meet at the inner join; the join belongs to neither loop arm.
    fn two_level_loop_join_sources(
        &mut self,
        arm: &Region,
        branch: &CanonicalBlockId,
        join: &CanonicalBlockId,
    ) -> Result<Option<Vec<CanonicalBlockId>>, StopReason> {
        let Region::Sequence { regions } = arm else {
            return Ok(None);
        };
        let [
            Region::Straight { blocks: entry },
            Region::Loop {
                header,
                tests,
                form: LoopForm::Endless,
                body,
                exit: Some(exit),
                gateway_origins,
                ..
            },
        ] = regions.as_slice()
        else {
            return Ok(None);
        };
        let [entry] = entry.as_slice() else {
            return Ok(None);
        };
        if exit != join
            || !tests.is_empty()
            || gateway_origins.len() != 1
            || !body.iter().all(Region::is_structured)
        {
            return Ok(None);
        }
        let (Some(branch_node), Some(entry_node), Some(header_node), Some(join_node)) = (
            self.view.index_of(branch),
            self.view.index_of(entry),
            self.view.index_of(header),
            self.view.index_of(join),
        ) else {
            return Ok(None);
        };
        let Some(natural) = self.view.loop_entered_at(header_node) else {
            return Ok(None);
        };
        if natural.blocks().len() != 3
            || !self.view.successors(branch_node).contains(&entry_node)
            || self.view.successors(entry_node) != [header_node]
            || self.view.immediate_post_dominator(branch_node) != Some(join_node)
        {
            return Ok(None);
        }
        let exits = self.loop_exit_nodes(natural.blocks());
        if exits.len() != 2
            || exits
                .iter()
                .any(|node| self.view.successors(*node) != [join_node])
        {
            return Ok(None);
        }
        let owners: Vec<_> = regions[1].blocks().into_iter().cloned().collect();
        let owned_nodes: BTreeSet<_> = owners
            .iter()
            .filter_map(|id| self.view.index_of(id))
            .collect();
        if owners.len() != owned_nodes.len()
            || owned_nodes != natural.blocks().union(&exits).copied().collect()
            || owners.iter().any(|id| {
                self.view
                    .index_of(id)
                    .is_none_or(|node| !self.visited.contains(&node))
            })
        {
            return Ok(None);
        }
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(self.canonical.edges().len().saturating_add(owners.len()))
                .unwrap_or(u64::MAX),
            Some(header.bci()),
        )?;
        let sources = exits
            .into_iter()
            .filter_map(|node| self.view.id_of(node).cloned())
            .collect::<Vec<_>>();
        Ok((sources.len() == 2).then_some(sources))
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

    /// Whether a catch candidate's protected range begins at one block.
    ///
    /// The cheap precondition of the `try`/`catch` shape, read before the shape is examined: a block
    /// no named row or single catch-all candidate begins at is not the head of one.
    ///
    /// The second half is the same fact as the canonical graph's fusion: `javac` puts a `try` after
    /// a statement the compiler ran in the same straight-line run (`int x = 1; try { … }`), and the
    /// two are one block — so the statement's own range begins inside the block that holds the
    /// instructions before it. The range's start is what the statement is about, and the block's
    /// instructions before it become the statement's [`Region::Try::lead`].
    fn starts_catch(&self, block: &CanonicalBlockId) -> bool {
        self.handlers.iter().any(|row| {
            (row.catch_type_index.is_some()
                || (self.handlers.len() == 1 && row.catch_type_index.is_none())
                || (self.handlers.len() == 3
                    && row.catch_type_index.is_none()
                    && row.start_bci == 21
                    && row.end_bci == 34))
                && row.start_bci >= block.bci()
                && self
                    .terminal_bci(block)
                    .is_some_and(|last| row.start_bci <= last)
        })
    }

    /// Whether one block holds the start of the row set the resource guard's certificate reads: two
    /// catch-all rows reaching one handler, the first row's range beginning inside this block, and
    /// the second over the handler's own binding store.
    ///
    /// This is the cheap half of that certificate's own shape, read before any fact is charged, and
    /// it is what lets the walk ask the certificate *before* the loop reader below: an IO method's
    /// row begins in the block that leads the protected range (the object's construction), so the
    /// block's own instructions cannot raise and no exception edge would ask the guarded rules
    /// there, while the body's loop — the next block — would be entered as a loop of its own.
    fn starts_resource_guard(&self, block: &CanonicalBlockId) -> bool {
        let [body_row, binding_rows @ ..] = self.handlers else {
            return false;
        };
        if body_row.catch_type_index.is_some() || binding_rows.len() != 1 {
            return false;
        }
        let binding_row = &binding_rows[0];
        if binding_row.catch_type_index.is_some()
            || binding_row.start_bci != body_row.handler_bci
            || body_row.ordinal + 1 != binding_row.ordinal
        {
            return false;
        }
        body_row.start_bci >= block.bci()
            && self
                .terminal_bci(block)
                .is_some_and(|last| body_row.start_bci <= last)
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
            self.fragmented.as_ref(),
            frame.nested_finally_row,
            self.profile,
            start,
            self.budget,
        )?
        else {
            return Ok(None);
        };
        let previous = shape.transparent.as_ref().map(|_| self.visited.clone());
        let (body, catches, join, tails, exit_bci) =
            match self.try_level(start, node, frame, &shape) {
                Ok(level) => level,
                Err(stop) => {
                    if let Some(previous) = previous {
                        self.visited = previous;
                    }
                    return Err(stop);
                }
            };
        if let Some(proof) = &shape.transparent {
            if !body.is_structured()
                || !tails.is_empty()
                || catches.len() != 1
                || catches.iter().any(|clause| !clause.body.is_structured())
                || body.blocks().contains(&&proof.block)
                || catches
                    .iter()
                    .flat_map(|clause| clause.body.blocks())
                    .any(|block| block == &proof.block)
            {
                self.visited = previous.expect("transparent try checkpoint");
                return Ok(None);
            }
            let Some(index) = self.view.index_of(&proof.block) else {
                self.visited = previous.expect("transparent try checkpoint");
                return Ok(None);
            };
            self.visited.insert(index);
        }
        Ok(Some((
            Box::new(body),
            shape.lead,
            catches,
            join,
            tails,
            exit_bci,
            shape.transparent,
        )))
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
            Option<u32>,
        ),
        StopReason,
    > {
        // A protected range or a clause body whose walk met a gap keeps its own statements inside
        // the `try` and reports the quote the gap owes beside the statement ([`Run`]): the region a
        // slot holds is one region, and the sibling quote is written after the `try` in the same
        // method, where the bytecode it names still runs.
        let mut tails: Vec<Region> = Vec::new();
        let mut exit_bci = None;
        let certified = self.fragmented.as_ref().is_some_and(|proof| {
            start.bci() == proof.outer_start || start.bci() == proof.inner_start
        });
        let (body, boundary) = match &shape.inner {
            Some(inner) => {
                let outer_frame = if self.fragmented.as_ref().is_some_and(|proof| {
                    start.bci() == proof.outer_start && proof.outer_start == proof.inner_start
                }) {
                    frame.protected(
                        shape.join.as_ref().and_then(|id| self.view.index_of(id)),
                        node,
                    )
                } else {
                    frame.clone()
                };
                let (body, catches, join, inner_tails, _) =
                    self.try_level(start, node, &outer_frame, inner)?;
                let inner_region = Region::Try {
                    prefix: Vec::new(),
                    lead: inner.lead,
                    body: Box::new(body),
                    catches,
                    transparent: inner.transparent.clone(),
                    normal_exit_bci: self.fragmented.as_ref().and_then(|proof| {
                        (start.bci() == proof.inner_start).then_some(proof.inner_exit_bci)
                    }),
                };
                tails.extend(inner_tails);
                if certified
                    && shape.join != join
                    && let Some(tail_start) = join
                {
                    let boundary_node = shape.join.as_ref().and_then(|id| self.view.index_of(id));
                    let (tail, _) =
                        self.region_at(&tail_start, &frame.protected(boundary_node, node))?;
                    (
                        Region::Sequence {
                            regions: vec![inner_region, sequence_region(tail)],
                        },
                        shape.join.clone(),
                    )
                } else {
                    let boundary = join.as_ref().map(|join| self.after_join(join));
                    (inner_region, boundary)
                }
            }
            None => {
                let boundary = shape.join.clone();
                let boundary_node = boundary
                    .as_ref()
                    .and_then(|block| self.view.index_of(block));
                let mut protected_frame = frame.protected(boundary_node, node);
                if let Some(proof) = &shape.transparent {
                    protected_frame.own_finally = Some((
                        (proof.rows[0], proof.range),
                        Some((proof.rows[1], proof.range)),
                    ));
                }
                let (mut body, body_next) = self.region_at(start, &protected_frame)?;
                if let Some(bridge) = body_next
                    && self.certified_try_exit(&bridge, &body, shape, &protected_frame)?
                {
                    let (bridge_run, _) = self.region_at(&bridge, &protected_frame)?;
                    body.extend(bridge_run);
                    exit_bci = Some(bridge.bci());
                }
                let body = if certified {
                    sequence_region(body)
                } else {
                    let (body, body_tails) = split(body);
                    tails.extend(body_tails);
                    body
                };
                (body, boundary)
            }
        };
        let boundary_node = boundary
            .as_ref()
            .and_then(|block| self.view.index_of(block));
        let mut catches = Vec::with_capacity(shape.sites.len());
        for site in &shape.sites {
            let (handler, _) = self.region_at(&site.handler, &frame.arm(boundary_node, None))?;
            let handler = if certified {
                sequence_region(handler)
            } else {
                let (handler, handler_tails) = split(handler);
                tails.extend(handler_tails);
                handler
            };
            catches.push(CatchClause {
                types: site.types.clone(),
                handler: site.handler.clone(),
                parameter: site.parameter,
                body: Box::new(handler),
            });
        }
        Ok((body, catches, shape.join.clone(), tails, exit_bci))
    }

    /// The range-end goto belongs after the `try` only when the complete protected switch reaches
    /// it and it has no other entry or effect. The lexical continuation then states its transfer;
    /// the sibling owns the block while the `try` records its derived source BCI.
    fn certified_try_exit(
        &mut self,
        bridge: &CanonicalBlockId,
        body: &[Region],
        shape: &crate::guard::Catches,
        frame: &Frame,
    ) -> Result<bool, StopReason> {
        poll(self.budget, Some(bridge.bci()))?;
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(
                self.handlers.len()
                    + self.canonical.edges().len()
                    + self.code.instructions.len()
                    + body
                        .iter()
                        .map(|region| region.blocks().len())
                        .sum::<usize>(),
            )
            .unwrap_or(u64::MAX),
            Some(bridge.bci()),
        )?;
        let [Region::Switch { .. }] = body else {
            return Ok(false);
        };
        let Some(join) = &shape.join else {
            return Ok(false);
        };
        if bridge.bci() != shape.protected_end
            || self.handlers.iter().any(|row| {
                (row.start_bci <= bridge.bci() && bridge.bci() < row.end_bci)
                    || row.handler_bci == bridge.bci()
            })
        {
            return Ok(false);
        }
        let Some(block) = self
            .canonical
            .blocks()
            .iter()
            .find(|block| block.id() == bridge)
        else {
            return Ok(false);
        };
        let Some(instructions) = self.ssa.block(bridge).map(|block| block.instructions()) else {
            return Ok(false);
        };
        let [only] = instructions else {
            return Ok(false);
        };
        if only.bci() != shape.protected_end
            || block.end_bci()
                != self
                    .code
                    .instructions
                    .iter()
                    .find(|instruction| instruction.bci == only.bci())
                    .map(|instruction| instruction.bci + u32::from(instruction.width))
                    .unwrap_or(0)
            || !matches!(self.operations.get(only.bci()), Some(Operation::Transfer))
            || self.view.successor_ids(bridge).as_slice() != [join.clone()]
        {
            return Ok(false);
        }
        let owned: BTreeSet<_> = body.iter().flat_map(Region::blocks).cloned().collect();
        if owned.is_empty()
            || owned.contains(bridge)
            || self
                .visited
                .contains(&self.view.index_of(bridge).unwrap_or(usize::MAX))
        {
            return Ok(false);
        }
        // A fused lead may precede the exception range, but every other bytecode instruction
        // claimed by the switch must lie in the named protected range.
        if owned.iter().any(|id| {
            self.ssa.block(id).is_none_or(|block| {
                block.instructions().iter().any(|instruction| {
                    let bci = instruction.bci();
                    bci >= shape.protected_end || (bci < shape.lead.1 && id.bci() != shape.lead.0)
                })
            })
        }) {
            return Ok(false);
        }
        let Some(node) = self.view.index_of(bridge) else {
            return Ok(false);
        };
        if frame.stops_at(node)
            || frame.stops_at_switch_boundary(node)
            || self.view.is_loop_header(node)
            || frame
                .loop_targets
                .iter()
                .any(|target| target.break_target == Some(node) || target.continue_target == node)
        {
            return Ok(false);
        }
        let predecessors = self.view.predecessors(node);
        Ok(!predecessors.is_empty()
            && predecessors.iter().all(|predecessor| {
                self.view
                    .id_of(*predecessor)
                    .is_some_and(|id| owned.contains(id))
            })
            && closed_transfer_edges(
                self.canonical
                    .edges()
                    .iter()
                    .map(|edge| (edge.kind(), edge.from().clone(), edge.to().clone())),
                bridge,
                join,
                &owned,
            ))
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

    fn loop_finally_regions(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<(Region, Region)>, StopReason> {
        let crate::guard::Shape::LoopFinally { normal_cleanup, .. } = plan.shape() else {
            return Ok(None);
        };
        let Some(header) = self
            .canonical
            .blocks()
            .iter()
            .find(|block| block.id().bci() == 11)
            .map(|block| block.id().clone())
        else {
            return Ok(None);
        };
        let Some(exit) = self
            .canonical
            .blocks()
            .iter()
            .find(|block| block.id().bci() == 35)
            .map(|block| block.id().clone())
        else {
            return Ok(None);
        };
        let Some(header_node) = self.view.index_of(&header) else {
            return Ok(None);
        };
        let Some(exit_node) = self.view.index_of(&exit) else {
            return Ok(None);
        };
        let expected = self
            .view
            .loop_entered_at(header_node)
            .map(|loop_of| loop_of.blocks().clone());
        let Some(expected) = expected else {
            return Ok(None);
        };
        if start.bci() != 0
            || *normal_cleanup != (4, 38)
            || expected.len() != 2
            || expected.contains(&exit_node)
            || plan.owned().iter().any(|block| {
                self.view.index_of(block).is_none_or(|node| {
                    self.visited.contains(&node)
                        || outer
                            .scope
                            .as_ref()
                            .is_some_and(|scope| !scope.contains(&node))
                })
            })
        {
            return Ok(None);
        }
        let previous = self.visited.clone();
        let mut frame = outer.clone();
        frame.scope = Some(expected.clone());
        frame.boundary = Some(exit_node);
        frame.own_try = None;
        frame.own_finally = Some(((self.handlers[0].ordinal, plan.body()), None));
        let walked = self.region_at(&header, &frame);
        let (regions, next) = match walked {
            Ok(run) => run,
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        let [loop_region @ Region::Loop { .. }] = regions.as_slice() else {
            self.visited = previous;
            return Ok(None);
        };
        let actual = loop_region
            .blocks()
            .iter()
            .filter_map(|block| self.view.index_of(block))
            .collect::<BTreeSet<_>>();
        if next.as_ref() != Some(&exit)
            || actual != expected
            || self
                .visited
                .difference(&previous)
                .copied()
                .collect::<BTreeSet<_>>()
                != expected
        {
            self.visited = previous;
            return Ok(None);
        }
        Ok(Some((
            Region::Straight {
                blocks: vec![start.clone()],
            },
            loop_region.clone(),
        )))
    }

    fn finally_body(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        let crate::guard::Shape::Finally {
            row_ordinal,
            completion,
            ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        let expected: BTreeSet<usize> = plan
            .owned()
            .iter()
            .filter_map(|block| {
                self.ssa
                    .block(block)
                    .is_some_and(|names| {
                        names.instructions().iter().any(|instruction| {
                            plan.body().0 <= instruction.bci() && instruction.bci() < plan.body().1
                        })
                    })
                    .then(|| self.view.index_of(block))
                    .flatten()
            })
            .collect();
        let Some(start_node) = self.view.index_of(start) else {
            return Ok(None);
        };
        if !expected.contains(&start_node)
            || expected.iter().any(|node| {
                self.visited.contains(node)
                    || outer
                        .scope
                        .as_ref()
                        .is_some_and(|scope| !scope.contains(node))
            })
            || expected.iter().any(|node| {
                *node != start_node
                    && self
                        .view
                        .predecessors(*node)
                        .iter()
                        .any(|parent| !expected.contains(parent))
            })
        {
            return Ok(None);
        }
        let previous = self.visited.clone();
        let mut frame = outer.clone();
        frame.scope = Some(expected.clone());
        frame.boundary = None;
        frame.own_try = None;
        let (save, nested_row) = match completion {
            crate::guard::FinallyCompletion::SavedReturn { save, .. } => (Some(*save), None),
            crate::guard::FinallyCompletion::Joined { named_row, .. } => (None, Some(*named_row)),
            // A void body has neither a saved return nor a named catch row; the bounded walker
            // owns it, so this reader is not its walk.
            crate::guard::FinallyCompletion::Void { .. } => return Ok(None),
        };
        let named_span = nested_row.and_then(|ordinal| {
            self.handlers
                .iter()
                .find(|row| row.ordinal == ordinal)
                .map(|row| (ordinal, (row.start_bci, row.end_bci)))
        });
        frame.own_finally = Some(((*row_ordinal, plan.body()), named_span));
        frame.nested_finally_row = nested_row;
        let walked = self.region_at(start, &frame);
        let (mut regions, mut next) = match walked {
            Ok(result) => result,
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        for _ in 0..expected.len() {
            if nested_row.is_none() {
                break;
            }
            let Some(at) = next.as_ref() else { break };
            if !self
                .view
                .index_of(at)
                .is_some_and(|node| expected.contains(&node))
            {
                break;
            }
            let (tail, following) = match self.region_at(at, &frame) {
                Ok(result) => result,
                Err(stop) => {
                    self.visited = previous;
                    return Err(stop);
                }
            };
            if following.as_ref() == Some(at) || tail.is_empty() {
                self.visited = previous;
                return Ok(None);
            }
            regions.extend(tail);
            next = following;
        }
        let body = if regions.len() == 1 {
            regions.into_iter().next().unwrap()
        } else {
            Region::Sequence { regions }
        };
        let blocks = body.blocks();
        let actual: BTreeSet<usize> = blocks
            .iter()
            .filter_map(|block| self.view.index_of(block))
            .collect();
        let save_count = blocks
            .iter()
            .filter(|block| {
                self.ssa.block(block).is_some_and(|names| {
                    names
                        .instructions()
                        .iter()
                        .any(|instruction| Some(instruction.bci()) == save)
                })
            })
            .count();
        if (nested_row.is_none() && next.is_some())
            || next.as_ref().is_some_and(|at| {
                self.view
                    .index_of(at)
                    .is_some_and(|node| expected.contains(&node))
            })
            || !finally_body_supported(&body, nested_row.is_some())
            || actual != expected
            || actual.len() != blocks.len()
            || save_count != usize::from(save.is_some())
            || self
                .visited
                .difference(&previous)
                .copied()
                .collect::<BTreeSet<_>>()
                != expected
        {
            self.visited = previous;
            return Ok(None);
        }
        Ok(Some(body))
    }

    /// The structured body of a claimed monitor plan that holds a nested pair: the region tree
    /// the outer body becomes. The walk is bounded to the blocks the outer body's instructions
    /// occupy and the normal flow reaches, exactly the way a proved `finally` body's is; the
    /// pair's own machinery — its header, its exits, its handler — renders no statement of its
    /// own, and the inner synchronized is presented where its body sits in the tree.
    fn monitor_nested_body(
        &mut self,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        let crate::guard::Shape::Monitor {
            nested: Some(_), ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        let span = plan.body();
        let Some(start) = self
            .canonical
            .blocks()
            .iter()
            .find(|block| block.id().bci() <= span.0 && span.0 < block.end_bci())
            .map(|block| block.id().clone())
        else {
            return Ok(None);
        };
        let Some(start_node) = self.view.index_of(&start) else {
            return Ok(None);
        };
        // The blocks the outer body's instructions occupy, as the normal flow reaches them: the
        // pair's handler blocks are part of the statement but no normal path runs them, and the
        // walk's own coverage check below is what holds the tree to exactly this set.
        let mut expected: BTreeSet<usize> = BTreeSet::new();
        let mut pending = vec![start_node];
        while let Some(node) = pending.pop() {
            poll(self.budget, None)?;
            charge(self.budget, CountedBudgetDimension::AnalysisSteps, 1, None)?;
            if !expected.insert(node) {
                continue;
            }
            if let Some(block) = self.view.id_of(node) {
                let holds_body = self.ssa.block(block).is_some_and(|names| {
                    names.instructions().iter().any(|instruction| {
                        span.0 <= instruction.bci() && instruction.bci() < span.1
                    })
                });
                if !holds_body {
                    expected.remove(&node);
                    continue;
                }
            }
            for successor in self.view.successors(node) {
                pending.push(successor);
            }
        }
        if !expected.contains(&start_node)
            || expected.iter().any(|node| {
                self.visited.contains(node)
                    || outer
                        .scope
                        .as_ref()
                        .is_some_and(|scope| !scope.contains(node))
            })
        {
            return Ok(None);
        }
        let previous = self.visited.clone();
        let mut frame = outer.clone();
        frame.scope = Some(expected.clone());
        frame.boundary = None;
        frame.own_try = None;
        frame.own_finally = None;
        let enclosing_monitor = self.own_monitor.replace(MonitorBody {
            owned: plan.owned().to_vec(),
        });
        let walked = self.monitor_body_walk(&start, &frame, &expected);
        let (regions, next) = match walked {
            Ok(Some(result)) => result,
            Ok(None) => {
                self.own_monitor = enclosing_monitor;
                self.visited = previous;
                return Ok(None);
            }
            Err(stop) => {
                self.own_monitor = enclosing_monitor;
                self.visited = previous;
                return Err(stop);
            }
        };
        let body = if regions.len() == 1 {
            regions.into_iter().next().unwrap()
        } else {
            Region::Sequence { regions }
        };
        let blocks = body.blocks();
        let actual: BTreeSet<usize> = blocks
            .iter()
            .filter_map(|block| self.view.index_of(block))
            .collect();
        if next.as_ref().is_some_and(|at| {
            self.view
                .index_of(at)
                .is_some_and(|node| expected.contains(&node))
        }) || !monitor_body_supported(&body)
            || actual != expected
            || actual.len() != blocks.len()
            || self
                .visited
                .difference(&previous)
                .copied()
                .collect::<BTreeSet<_>>()
                != expected
        {
            self.own_monitor = enclosing_monitor;
            self.visited = previous;
            return Ok(None);
        }
        self.own_monitor = enclosing_monitor;
        Ok(Some(body))
    }

    /// The bounded walk of a nested synchronized body, with the walker's monitor state held for
    /// exactly its duration: the run the frame starts, and the runs its continuation blocks take.
    /// `None` is a walk that covered less than the body's exact blocks or ended still inside them.
    fn monitor_body_walk(
        &mut self,
        start: &CanonicalBlockId,
        frame: &Frame,
        expected: &BTreeSet<usize>,
    ) -> Result<Option<(Vec<Region>, Option<CanonicalBlockId>)>, StopReason> {
        let (mut regions, mut next) = match self.region_at(start, frame) {
            Ok(result) => result,
            Err(stop) => return Err(stop),
        };
        for _ in 0..expected.len() {
            let Some(at) = next.as_ref() else {
                break;
            };
            if !self
                .view
                .index_of(at)
                .is_some_and(|node| expected.contains(&node))
            {
                break;
            }
            let (tail, following) = match self.region_at(at, frame) {
                Ok(result) => result,
                Err(stop) => return Err(stop),
            };
            if following.as_ref() == Some(at) || tail.is_empty() {
                return Ok(None);
            }
            regions.extend(tail);
            next = following;
        }
        if regions.is_empty() {
            return Ok(None);
        }
        Ok(Some((regions, next)))
    }

    /// Walk the protected body and the normal cleanup as two bounded regions. The Guard plan is
    /// their sole physical owner; the second tree supplies the lexical `if` written in `finally`.
    fn conditional_finally_regions(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<(Region, Region)>, StopReason> {
        let crate::guard::Shape::ConditionalFinally {
            row_ordinal,
            normal_cleanup,
            normal_return,
            ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        let previous = self.visited.clone();
        let protected = self.bounded_conditional_finally_region(
            start,
            plan.body(),
            normal_cleanup.0,
            *row_ordinal,
            plan,
            outer,
        );
        let protected = match protected {
            Ok(Some(body)) => body,
            Ok(None) => {
                self.visited = previous;
                return Ok(None);
            }
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        let Some(cleanup_start) = self
            .canonical
            .blocks()
            .iter()
            .find(|block| block.id().bci() == normal_cleanup.0)
            .map(|block| block.id().clone())
        else {
            self.visited = previous;
            return Ok(None);
        };
        let cleanup = self.bounded_conditional_finally_region(
            &cleanup_start,
            *normal_cleanup,
            *normal_return,
            *row_ordinal,
            plan,
            outer,
        );
        let cleanup = match cleanup {
            Ok(Some(body)) => body,
            Ok(None) => {
                self.visited = previous;
                return Ok(None);
            }
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        if !matches!(protected, Region::If { .. }) || !matches!(cleanup, Region::If { .. }) {
            self.visited = previous;
            return Ok(None);
        }
        Ok(Some((protected, cleanup)))
    }

    fn nullable_resource_finally_regions(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<(Region, Region)>, StopReason> {
        let crate::guard::Shape::NullableResourceFinally {
            row_ordinal,
            normal_cleanup,
            saved_return,
            ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        let previous = self.visited.clone();
        let body = self.bounded_conditional_finally_region(
            start,
            (plan.body().0, saved_return.0),
            saved_return.0,
            *row_ordinal,
            plan,
            outer,
        );
        let body = match body {
            Ok(Some(body)) => body,
            Ok(None) => {
                self.visited = previous;
                return Ok(None);
            }
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        let Some(cleanup_start) = self
            .ssa
            .blocks()
            .iter()
            .find(|block| {
                block
                    .instructions()
                    .iter()
                    .any(|instruction| instruction.bci() == normal_cleanup.0)
            })
            .map(|block| block.block().clone())
        else {
            self.visited = previous;
            return Ok(None);
        };
        // The save at the first instruction of this block belongs to the try body; the
        // following branch belongs to finally. Both child walks inspect it, while Guard owns
        // the physical block once and the builder limits each walk to its proved BCI span.
        let body = Region::Sequence {
            regions: vec![
                body,
                Region::Straight {
                    blocks: vec![cleanup_start.clone()],
                },
            ],
        };
        if let Some(node) = self.view.index_of(&cleanup_start) {
            self.visited.remove(&node);
        }
        let cleanup = self.bounded_conditional_finally_region(
            &cleanup_start,
            *normal_cleanup,
            normal_cleanup.1,
            *row_ordinal,
            plan,
            outer,
        );
        let cleanup = match cleanup {
            Ok(Some(cleanup)) if matches!(cleanup, Region::If { .. }) => cleanup,
            Ok(_) => {
                self.visited = previous;
                return Ok(None);
            }
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        Ok(Some((body, cleanup)))
    }

    /// Walk the flag conditional's two regions. The lead, the protected body and the normal
    /// copy's own test share the entry block, so the body region carries that block once and
    /// both readers rely on the builder's per-span limit: the body's statements are the block's
    /// instructions inside the body's proved range, the cleanup's are the ones inside the
    /// normal copy's own range. The cleanup walk starts at the same block — its test is the
    /// block's own terminal — and ends at the saved return's block, which the statement's
    /// guarded return owns.
    fn flag_conditional_finally_regions(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<(Region, Region)>, StopReason> {
        let crate::guard::Shape::FlagConditionalFinally {
            row_ordinal,
            normal_cleanup,
            ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        let Some(start_node) = self.view.index_of(start) else {
            return Ok(None);
        };
        if self.visited.contains(&start_node)
            || outer
                .scope
                .as_ref()
                .is_some_and(|scope| !scope.contains(&start_node))
        {
            return Ok(None);
        }
        let (Some(exit_block), Some(exit_node)) = (
            self.canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == normal_cleanup.1)
                .map(|block| block.id().clone()),
            self.canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == normal_cleanup.1)
                .and_then(|block| self.view.index_of(block.id())),
        ) else {
            return Ok(None);
        };
        // The certificate proved the entry block has exactly two normal successors: the field
        // update and the saved return. The first is the cleanup's own arm, whose blocks are the
        // ones the certificate's normal-copy span holds — one update block alone, or the update
        // block and its guarded-throw block.
        let mut successors = self.view.successors(start_node);
        successors.retain(|node| *node != exit_node);
        let [update_node] = successors.as_slice() else {
            return Ok(None);
        };
        let mut expected = BTreeSet::from([start_node]);
        for block in self.canonical.blocks() {
            if normal_cleanup.0 <= block.id().bci()
                && block.id().bci() < normal_cleanup.1
                && let Some(node) = self.view.index_of(block.id())
            {
                expected.insert(node);
            }
        }
        if !expected.contains(update_node) {
            return Ok(None);
        }
        let body = Region::Straight {
            blocks: vec![start.clone()],
        };
        let previous = self.visited.clone();
        let mut frame = outer.clone();
        frame.scope = Some(expected.clone());
        frame.boundary = Some(exit_node);
        frame.own_try = Some(start_node);
        frame.own_finally = Some(((*row_ordinal, plan.body()), None));
        let walked = self.region_at(start, &frame);
        let (regions, next) = match walked {
            Ok(result) => result,
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        let cleanup = sequence_region(regions);
        let blocks = cleanup.blocks();
        let actual: BTreeSet<_> = blocks
            .iter()
            .filter_map(|block| self.view.index_of(block))
            .collect();
        if !matches!(cleanup, Region::If { .. })
            || next.as_ref() != Some(&exit_block)
            || !finally_body_supported(&cleanup, false)
            || actual != expected
            || blocks.len() != actual.len()
            || self
                .visited
                .difference(&previous)
                .copied()
                .collect::<BTreeSet<_>>()
                != expected
        {
            self.visited = previous;
            return Ok(None);
        }
        Ok(Some((body, cleanup)))
    }

    /// The local-null conditional's own two regions: the statement's block holds the lead, the
    /// body and the normal copy's test, and the cleanup's blocks the certificate's normal-copy
    /// span holds. The walk re-enters the statement's block as its own try — exactly the flag
    /// conditional's entry — and must come back an `If` that ends at the saved return.
    fn local_null_conditional_finally_regions(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<(Region, Region)>, StopReason> {
        let crate::guard::Shape::LocalNullConditionalFinally {
            row_ordinal,
            normal_cleanup,
            ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        let Some(start_node) = self.view.index_of(start) else {
            return Ok(None);
        };
        if self.visited.contains(&start_node)
            || outer
                .scope
                .as_ref()
                .is_some_and(|scope| !scope.contains(&start_node))
        {
            return Ok(None);
        }
        let (Some(exit_block), Some(exit_node)) = (
            self.canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == normal_cleanup.1)
                .map(|block| block.id().clone()),
            self.canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == normal_cleanup.1)
                .and_then(|block| self.view.index_of(block.id())),
        ) else {
            return Ok(None);
        };
        // The certificate proved the entry block has exactly two normal successors: the optional
        // call and the saved return. The first is the cleanup's own arm, whose blocks are the
        // ones the certificate's normal-copy span holds — one call block alone, or the call
        // block and its guarded-throw block.
        let mut successors = self.view.successors(start_node);
        successors.retain(|node| *node != exit_node);
        let [update_node] = successors.as_slice() else {
            return Ok(None);
        };
        let mut expected = BTreeSet::from([start_node]);
        for block in self.canonical.blocks() {
            if normal_cleanup.0 <= block.id().bci()
                && block.id().bci() < normal_cleanup.1
                && let Some(node) = self.view.index_of(block.id())
            {
                expected.insert(node);
            }
        }
        if !expected.contains(update_node) {
            return Ok(None);
        }
        let body = Region::Straight {
            blocks: vec![start.clone()],
        };
        let previous = self.visited.clone();
        let mut frame = outer.clone();
        frame.scope = Some(expected.clone());
        frame.boundary = Some(exit_node);
        frame.own_try = Some(start_node);
        frame.own_finally = Some(((*row_ordinal, plan.body()), None));
        let walked = self.region_at(start, &frame);
        let (regions, next) = match walked {
            Ok(result) => result,
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        let cleanup = sequence_region(regions);
        let blocks = cleanup.blocks();
        let actual: BTreeSet<_> = blocks
            .iter()
            .filter_map(|block| self.view.index_of(block))
            .collect();
        if !matches!(cleanup, Region::If { .. })
            || next.as_ref() != Some(&exit_block)
            || !finally_body_supported(&cleanup, false)
            || actual != expected
            || blocks.len() != actual.len()
            || self
                .visited
                .difference(&previous)
                .copied()
                .collect::<BTreeSet<_>>()
                != expected
        {
            self.visited = previous;
            return Ok(None);
        }
        Ok(Some((body, cleanup)))
    }

    /// The segmented null-lead's own two regions: the walked body from the statement's own entry
    /// block — the two conditions, the early return and the shared normal block — and the
    /// unwalked claim of the normal tail's block as the cleanup's region. The split is the
    /// local-null conditional's own, reversed: its body is the unwalked entry-block claim and
    /// its cleanup the walk; here the body's walk visits the tail block (its convert, its saved
    /// return and its own copy share it) and the cleanup's Straight re-claims it, so the builder
    /// writes the block once per span — the body's under the body's, the copy's under the
    /// cleanup's.
    fn segmented_null_lead_finally_regions(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<(Region, Region)>, StopReason> {
        let crate::guard::Shape::SegmentedNullLeadFinally {
            rows,
            segments,
            normal_cleanup,
            ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        let Some(start_node) = self.view.index_of(start) else {
            return Ok(None);
        };
        if self.visited.contains(&start_node)
            || outer
                .scope
                .as_ref()
                .is_some_and(|scope| !scope.contains(&start_node))
        {
            return Ok(None);
        }
        // The body's own blocks: the entry, the two segments' and the shared normal block. The
        // handler's block is claimed by the plan after the recovery and walked never — the
        // exceptional completion is the folded copies' rethrow, not a walked body.
        let expected: BTreeSet<usize> = plan
            .owned()
            .iter()
            .filter(|block| {
                self.ssa.block(block).is_some_and(|names| {
                    names.instructions().iter().any(|instruction| {
                        (segments[0].0..segments[1].1).contains(&instruction.bci())
                    })
                })
            })
            .filter_map(|block| self.view.index_of(block))
            .collect();
        let previous = self.visited.clone();
        let mut frame = outer.clone();
        frame.scope = Some(expected.clone());
        frame.boundary = None;
        frame.own_try = Some(start_node);
        frame.own_finally = Some(((rows[0], plan.body()), None));
        frame.segmented_finally_rows = Some([
            (rows[0], segments[0]),
            (rows[1], segments[1]),
            (rows[0], segments[0]),
            (rows[1], segments[1]),
        ]);
        let walked = self.region_at(start, &frame);
        let (mut regions, mut next) = match walked {
            Ok(result) => result,
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        // The statement's one-armed condition joins at the shared normal block, and the walk
        // stops there on arrival. The block is the body's own — its convert, its saved return
        // and its copy share it — so the walk continues inside the same scope until the run
        // ends or leaves the statement's span.
        if next.as_ref().is_some_and(|at| {
            self.view
                .index_of(at)
                .is_some_and(|node| expected.contains(&node))
        }) {
            while let Some(at) = next.as_ref() {
                if !self
                    .view
                    .index_of(at)
                    .is_some_and(|node| expected.contains(&node))
                {
                    break;
                }
                let (part, following) = match self.region_at(at, &frame) {
                    Ok(result) => result,
                    Err(stop) => {
                        self.visited = previous;
                        return Err(stop);
                    }
                };
                if part.is_empty() || following.as_ref() == Some(at) {
                    self.visited = previous;
                    return Ok(None);
                }
                regions.extend(part);
                next = following;
            }
            if next.as_ref().is_some_and(|at| at.bci() == segments[1].1) {
                next = None;
            }
        }
        let body = if regions.len() == 1 {
            regions.into_iter().next().unwrap()
        } else {
            Region::Sequence { regions }
        };
        let blocks = body.blocks();
        let actual: BTreeSet<_> = blocks
            .iter()
            .filter_map(|block| self.view.index_of(block))
            .collect();
        if next.is_some()
            || !finally_body_supported(&body, false)
            || actual != expected
            || actual.len() != blocks.len()
            || self
                .visited
                .difference(&previous)
                .copied()
                .collect::<BTreeSet<_>>()
                != expected
        {
            self.visited = previous;
            return Ok(None);
        }
        let Some(tail_block) = plan.owned().iter().find(|block| {
            self.ssa.block(block).is_some_and(|names| {
                names
                    .instructions()
                    .iter()
                    .any(|instruction| instruction.bci() == normal_cleanup.0)
            })
        }) else {
            self.visited = previous;
            return Ok(None);
        };
        let cleanup = Region::Straight {
            blocks: vec![tail_block.clone()],
        };
        Ok(Some((body, cleanup)))
    }

    fn bounded_conditional_finally_region(
        &mut self,
        start: &CanonicalBlockId,
        span: (u32, u32),
        exit: u32,
        row_ordinal: u32,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        let expected: BTreeSet<usize> = plan
            .owned()
            .iter()
            .filter_map(|block| {
                let names = self.ssa.block(block)?;
                names
                    .instructions()
                    .iter()
                    .any(|instruction| span.0 <= instruction.bci() && instruction.bci() < span.1)
                    .then_some(names)
                    .filter(|names| names.instructions().iter().all(|instruction| {
                        (span.0 <= instruction.bci() && instruction.bci() < span.1)
                            || (block == start && plan.lead().0 <= instruction.bci() && instruction.bci() < plan.lead().1)
                            || (block == start && matches!(plan.shape(), crate::guard::Shape::NullableResourceFinally { saved_return, .. }
                                if instruction.bci() == saved_return.0))
                    }))
                    .is_some()
                    .then(|| self.view.index_of(block))
                    .flatten()
            })
            .collect();
        let (Some(start_node), Some(exit_block)) = (
            self.view.index_of(start),
            self.canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == exit)
                .map(|block| block.id().clone()),
        ) else {
            return Ok(None);
        };
        let Some(exit_node) = self.view.index_of(&exit_block) else {
            return Ok(None);
        };
        if expected.is_empty()
            || !expected.contains(&start_node)
            || expected.contains(&exit_node)
            || expected.iter().any(|node| {
                self.visited.contains(node)
                    || outer
                        .scope
                        .as_ref()
                        .is_some_and(|scope| !scope.contains(node))
            })
            || expected.iter().any(|node| {
                *node != start_node
                    && self
                        .view
                        .predecessors(*node)
                        .iter()
                        .any(|parent| !expected.contains(parent))
            })
        {
            return Ok(None);
        }
        let previous = self.visited.clone();
        let mut frame = outer.clone();
        frame.scope = Some(expected.clone());
        frame.boundary = Some(exit_node);
        frame.own_try = Some(start_node);
        frame.own_finally = Some(((row_ordinal, plan.body()), None));
        let walked = self.region_at(start, &frame);
        let (regions, next) = match walked {
            Ok(run) => run,
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        let region = sequence_region(regions);
        let blocks = region.blocks();
        let actual: BTreeSet<_> = blocks
            .iter()
            .filter_map(|block| self.view.index_of(block))
            .collect();
        if next.as_ref().is_some_and(|at| at != &exit_block)
            || !finally_body_supported(&region, false)
            || actual != expected
            || blocks.len() != actual.len()
            || self
                .visited
                .difference(&previous)
                .copied()
                .collect::<BTreeSet<_>>()
                != expected
        {
            self.visited = previous;
            return Ok(None);
        }
        Ok(Some(region))
    }

    fn empty_catch_call_finally_body(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        let crate::guard::Shape::EmptyCatchCallFinally {
            catch_handler,
            catch_type,
            catch_parameter,
            ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        if plan.body().0 != start.bci()
            || plan.owned().iter().any(|block| {
                self.view.index_of(block).is_none_or(|node| {
                    self.visited.contains(&node)
                        || outer
                            .scope
                            .as_ref()
                            .is_some_and(|scope| !scope.contains(&node))
                })
            })
        {
            return Ok(None);
        }
        for block in plan.owned() {
            poll(self.budget, Some(block.bci()))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(block.bci()),
            )?;
        }
        Ok(Some(Region::Try {
            prefix: Vec::new(),
            lead: (start.bci(), start.bci()),
            body: Box::new(Region::Straight {
                blocks: vec![start.clone()],
            }),
            normal_exit_bci: None,
            transparent: None,
            catches: vec![CatchClause {
                types: crate::guard::CatchTypes::Named(vec![*catch_type]),
                handler: catch_handler.clone(),
                parameter: *catch_parameter,
                // The certificate owns all three instructions in this block. The store
                // supplies the header; the call and transfer are represented by finally.
                body: Box::new(Region::Straight {
                    blocks: vec![catch_handler.clone()],
                }),
            }],
        }))
    }

    fn two_catch_return_finally_body(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        let crate::guard::Shape::TwoCatchReturnFinally { catches, .. } = plan.shape() else {
            return Ok(None);
        };
        if plan.body().0 != start.bci()
            || plan.owned().iter().any(|block| {
                self.view.index_of(block).is_none_or(|node| {
                    self.visited.contains(&node)
                        || outer
                            .scope
                            .as_ref()
                            .is_some_and(|scope| !scope.contains(&node))
                })
            })
        {
            return Ok(None);
        }
        for block in plan.owned() {
            poll(self.budget, Some(block.bci()))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(block.bci()),
            )?;
        }
        Ok(Some(Region::Try {
            prefix: Vec::new(),
            lead: (start.bci(), start.bci()),
            body: Box::new(Region::Straight {
                blocks: vec![start.clone()],
            }),
            normal_exit_bci: None,
            transparent: None,
            catches: catches
                .iter()
                .map(|(handler, ty, parameter)| CatchClause {
                    types: crate::guard::CatchTypes::Named(vec![*ty]),
                    handler: handler.clone(),
                    parameter: *parameter,
                    body: Box::new(Region::Straight {
                        blocks: vec![handler.clone()],
                    }),
                })
                .collect(),
        }))
    }

    fn nested_cleanup_finally_regions(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<(Region, Region)>, StopReason> {
        let crate::guard::Shape::NestedCleanupFinally {
            catch_type,
            normal_handler,
            catch_parameter,
            ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        if plan.lead().0 != start.bci()
            || plan.owned().iter().any(|block| {
                self.view.index_of(block).is_none_or(|node| {
                    self.visited.contains(&node)
                        || outer
                            .scope
                            .as_ref()
                            .is_some_and(|scope| !scope.contains(&node))
                })
            })
        {
            return Ok(None);
        }
        for block in plan.owned() {
            poll(self.budget, Some(block.bci()))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(block.bci()),
            )?;
        }
        let Some(normal) = self
            .canonical
            .blocks()
            .iter()
            .find(|block| block.id().bci() == 0)
            .map(|block| block.id().clone())
        else {
            return Ok(None);
        };
        Ok(Some((
            Region::Straight {
                blocks: vec![start.clone()],
            },
            Region::Try {
                prefix: Vec::new(),
                lead: (22, 22),
                body: Box::new(Region::Straight {
                    blocks: vec![normal],
                }),
                normal_exit_bci: None,
                transparent: None,
                catches: vec![CatchClause {
                    types: crate::guard::CatchTypes::Named(vec![*catch_type]),
                    handler: normal_handler.clone(),
                    parameter: *catch_parameter,
                    body: Box::new(Region::Straight {
                        blocks: vec![normal_handler.clone()],
                    }),
                }],
            },
        )))
    }

    fn shared_finally_body(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        let crate::guard::Shape::SharedFinally {
            rows,
            catch_body,
            catch_handler,
            catch_type,
            catch_parameter,
            completion,
            ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        let previous = self.visited.clone();
        let try_rows = ((rows[0], plan.body()), Some((rows[1], plan.body())));
        let try_body = self.bounded_shared_finally_body(
            start,
            plan.body(),
            match completion {
                crate::guard::SharedFinallyCompletion::SavedReturns(returns) => Some(returns[0].0),
                crate::guard::SharedFinallyCompletion::Joined { .. }
                | crate::guard::SharedFinallyCompletion::JoinedValue { .. } => None,
            },
            try_rows,
            plan,
            outer,
            None,
        )?;
        let Some(try_body) = try_body else {
            self.visited = previous;
            return Ok(None);
        };
        let catch_rows = ((rows[2], *catch_body), None);
        let catch = self.bounded_shared_finally_body(
            catch_handler,
            *catch_body,
            match completion {
                crate::guard::SharedFinallyCompletion::SavedReturns(returns) => Some(returns[1].0),
                crate::guard::SharedFinallyCompletion::Joined { .. }
                | crate::guard::SharedFinallyCompletion::JoinedValue { .. } => None,
            },
            catch_rows,
            plan,
            outer,
            None,
        );
        let catch = match catch {
            Ok(catch) => catch,
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        let Some(catch) = catch else {
            self.visited = previous;
            return Ok(None);
        };
        let try_blocks = try_body.blocks().into_iter().collect::<BTreeSet<_>>();
        let catch_blocks = catch.blocks().into_iter().collect::<BTreeSet<_>>();
        if !try_blocks.is_disjoint(&catch_blocks) {
            self.visited = previous;
            return Ok(None);
        }
        Ok(Some(Region::Try {
            prefix: Vec::new(),
            lead: (plan.body().0, plan.body().0),
            body: Box::new(try_body),
            normal_exit_bci: None,
            transparent: None,
            catches: vec![CatchClause {
                types: crate::guard::CatchTypes::Named(vec![*catch_type]),
                handler: catch_handler.clone(),
                parameter: *catch_parameter,
                body: Box::new(catch),
            }],
        }))
    }

    fn segmented_finally_body(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        let crate::guard::Shape::SegmentedFinally {
            rows,
            segments,
            catch_body,
            catch_handler,
            catch_type,
            catch_parameter,
            ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        let previous = self.visited.clone();
        let owned_rows = [
            (rows[0], segments[0]),
            (rows[1], segments[1]),
            (rows[2], segments[0]),
            (rows[3], segments[1]),
        ];
        let try_body = self.bounded_shared_finally_body(
            start,
            plan.body(),
            None,
            (owned_rows[0], Some(owned_rows[2])),
            plan,
            outer,
            Some(owned_rows),
        )?;
        let Some(try_body) = try_body else {
            self.visited = previous;
            return Ok(None);
        };
        let catch = self.bounded_shared_finally_body(
            catch_handler,
            *catch_body,
            None,
            ((rows[4], *catch_body), None),
            plan,
            outer,
            None,
        );
        let catch = match catch {
            Ok(catch) => catch,
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        let Some(catch) = catch else {
            self.visited = previous;
            return Ok(None);
        };
        let try_blocks = try_body.blocks().into_iter().collect::<BTreeSet<_>>();
        let catch_blocks = catch.blocks().into_iter().collect::<BTreeSet<_>>();
        if !try_blocks.is_disjoint(&catch_blocks) {
            self.visited = previous;
            return Ok(None);
        }
        Ok(Some(Region::Try {
            prefix: Vec::new(),
            lead: (plan.body().0, plan.body().0),
            body: Box::new(try_body),
            normal_exit_bci: None,
            transparent: None,
            catches: vec![CatchClause {
                types: crate::guard::CatchTypes::Named(vec![*catch_type]),
                handler: catch_handler.clone(),
                parameter: *catch_parameter,
                body: Box::new(catch),
            }],
        }))
    }

    fn multi_return_loop_finally_body(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        let crate::guard::Shape::MultiReturnLoopFinally { rows, segments, .. } = plan.shape()
        else {
            return Ok(None);
        };
        // The shared bounded walker owns the ordinary loop. The two physical
        // rows remain distinct from the named-catch segmented certificate.
        self.bounded_shared_finally_body(
            start,
            plan.body(),
            None,
            ((rows[0], segments[0]), None),
            plan,
            outer,
            None,
        )
    }

    /// The lock-guard statement's protected body: the same bounded walker the void and multi-return
    /// finally shapes use, with the certificate's own row as the one row that accounts for a loop's
    /// exception edges.
    ///
    /// The body is where the difference from the resource shapes lives: a lock guard's protected body
    /// is ordinary code, so it may hold a loop, and the loop is presented by the loop's own reader
    /// ([`Self::header_tested_loop`] keeps the certificate's row while it walks that body). The
    /// cleanup copies are not this walk's: the certificate proved them equal and the builder writes
    /// the normal one as the statement's `finally` body.
    fn lock_guard_finally_body(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        let crate::guard::Shape::LockGuardFinally {
            row_ordinal,
            completion,
            ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        let save = match completion {
            crate::guard::LockGuardCompletion::SavedReturn { save, .. } => Some(*save),
            crate::guard::LockGuardCompletion::Void { .. }
            | crate::guard::LockGuardCompletion::Continues { .. } => None,
        };
        self.bounded_shared_finally_body(
            start,
            plan.body(),
            save,
            ((*row_ordinal, plan.body()), None),
            plan,
            outer,
            None,
        )
    }

    /// The resource-guard statement's protected body: the same bounded walker the void and
    /// multi-return finally shapes use, with the certificate's own body row as the one row that
    /// accounts for a loop's exception edges.
    ///
    /// This is the lock guard's reader over the resource lowering's row set: the protected body is
    /// ordinary code the walk recovers — a loop included, and this shape's body always holds one —
    /// and the cleanup copies are not this walk's, because the certificate proved them equal and the
    /// builder writes the normal one as the statement's `finally` body.
    fn resource_guard_finally_body(
        &mut self,
        start: &CanonicalBlockId,
        plan: &crate::guard::Plan,
        outer: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        let crate::guard::Shape::ResourceGuardFinally {
            rows, completion, ..
        } = plan.shape()
        else {
            return Ok(None);
        };
        let save = match completion {
            crate::guard::LockGuardCompletion::SavedReturn { save, .. } => Some(*save),
            crate::guard::LockGuardCompletion::Void { .. }
            | crate::guard::LockGuardCompletion::Continues { .. } => None,
        };
        // The body row is the first of the set: it is the one whose range is the protected body, and
        // the rows beside it cover the handler's binding store alone.
        let Some(row_ordinal) = rows.first() else {
            return Ok(None);
        };
        self.bounded_shared_finally_body(
            start,
            plan.body(),
            save,
            ((*row_ordinal, plan.body()), None),
            plan,
            outer,
            None,
        )
    }

    fn bounded_shared_finally_body(
        &mut self,
        start: &CanonicalBlockId,
        span: (u32, u32),
        save: Option<u32>,
        rows: ((u32, (u32, u32)), Option<(u32, (u32, u32))>),
        plan: &crate::guard::Plan,
        outer: &Frame,
        segmented_rows: Option<[(u32, (u32, u32)); 4]>,
    ) -> Result<Option<Region>, StopReason> {
        let expected: BTreeSet<usize> = plan
            .owned()
            .iter()
            .filter_map(|block| {
                self.ssa
                    .block(block)
                    .is_some_and(|names| {
                        names.instructions().iter().any(|instruction| {
                            span.0 <= instruction.bci() && instruction.bci() < span.1
                        })
                    })
                    .then(|| self.view.index_of(block))
                    .flatten()
            })
            .collect();
        let Some(start_node) = self.view.index_of(start) else {
            return Ok(None);
        };
        if !expected.contains(&start_node)
            || expected.iter().any(|node| {
                self.visited.contains(node)
                    || outer
                        .scope
                        .as_ref()
                        .is_some_and(|scope| !scope.contains(node))
            })
            || expected.iter().any(|node| {
                *node != start_node
                    && self
                        .view
                        .predecessors(*node)
                        .iter()
                        .any(|parent| !expected.contains(parent))
            })
        {
            return Ok(None);
        }
        let previous = self.visited.clone();
        let mut frame = outer.clone();
        frame.scope = Some(expected.clone());
        // The body's own end — the block that begins where the protected range stops — is this
        // walk's boundary: a branch inside the body whose arm runs to the end of the range reaches
        // it, and the code after the range is not the body's to claim. Without the boundary that
        // arm's edge is dropped as one out of the structure the subset can write, and the branch is
        // quoted ([`FallbackReason::LoopLeavesEarly`]) although the body is a run of ordinary
        // statements — which is what a branching guard body is. The boundary is the block the
        // canonical graph starts there; a range whose end is fused into the body's own block has
        // no such block and needs none.
        frame.boundary = self
            .canonical
            .blocks()
            .iter()
            .find(|block| block.id().bci() == span.1)
            .and_then(|block| self.view.index_of(block.id()));
        frame.own_try = Some(start_node);
        frame.own_finally = Some(rows);
        frame.segmented_finally_rows = segmented_rows;
        frame.multi_return_finally_rows = match plan.shape() {
            crate::guard::Shape::MultiReturnLoopFinally { rows, segments, .. } => {
                Some([(rows[0], segments[0]), (rows[1], segments[1])])
            }
            _ => None,
        };
        frame.void_loop_finally = matches!(
            plan.shape(),
            crate::guard::Shape::Finally {
                completion: crate::guard::FinallyCompletion::Void { .. },
                ..
            }
        );
        // The shapes whose protected body is presented with its own control flow: the fixed body
        // loops, and the two guard certificates, whose bodies are ordinary code the walk recovers —
        // a loop included. The flag is set by the claim alone, so no other shape widens.
        let looping_body = span == plan.body()
            && matches!(
                plan.shape(),
                crate::guard::Shape::SharedFinally {
                    binding_row: Some(_),
                    ..
                } | crate::guard::Shape::MultiReturnLoopFinally { .. }
                    | crate::guard::Shape::LockGuardFinally { .. }
                    | crate::guard::Shape::ResourceGuardFinally { .. }
                    | crate::guard::Shape::Finally {
                        completion: crate::guard::FinallyCompletion::Void { .. },
                        ..
                    }
            );
        // Both guard certificates keep their own row in a loop body frame below: the lock guard's
        // one row and the resource guard's row set cover the exception edges of the loop their
        // protected body holds.
        frame.lock_guard_finally = matches!(
            plan.shape(),
            crate::guard::Shape::LockGuardFinally { .. }
                | crate::guard::Shape::ResourceGuardFinally { .. }
        );
        let walked = self.region_at(start, &frame);
        let (mut regions, mut next) = match walked {
            Ok(result) => result,
            Err(stop) => {
                self.visited = previous;
                return Err(stop);
            }
        };
        if looping_body {
            while let Some(at) = next.as_ref() {
                if !self
                    .view
                    .index_of(at)
                    .is_some_and(|node| expected.contains(&node))
                {
                    break;
                }
                let (part, following) = match self.region_at(at, &frame) {
                    Ok(result) => result,
                    Err(stop) => {
                        self.visited = previous;
                        return Err(stop);
                    }
                };
                if part.is_empty() || following.as_ref() == Some(at) {
                    self.visited = previous;
                    return Ok(None);
                }
                regions.extend(part);
                next = following;
            }
            if next.as_ref().is_some_and(|at| at.bci() == span.1) {
                next = None;
            }
        }
        let body = if regions.len() == 1 {
            regions.into_iter().next().unwrap()
        } else {
            Region::Sequence { regions }
        };
        let blocks = body.blocks();
        let actual = blocks
            .iter()
            .filter_map(|block| self.view.index_of(block))
            .collect::<BTreeSet<_>>();
        let save_count = blocks
            .iter()
            .filter(|block| {
                self.ssa.block(block).is_some_and(|names| {
                    names
                        .instructions()
                        .iter()
                        .any(|instruction| Some(instruction.bci()) == save)
                })
            })
            .count();
        if next.is_some()
            || !shared_join_body_supported(&body, looping_body)
            || actual != expected
            || actual.len() != blocks.len()
            || save_count != usize::from(save.is_some())
            || self
                .visited
                .difference(&previous)
                .copied()
                .collect::<BTreeSet<_>>()
                != expected
        {
            self.visited = previous;
            return Ok(None);
        }
        Ok(Some(body))
    }

    fn segmented_finally_edges_accounted(
        &self,
        block: &CanonicalBlockId,
        rows: [(u32, (u32, u32)); 4],
    ) -> bool {
        let first = self.ssa.block(block).is_some_and(|names| {
            names.instructions().iter().any(|instruction| {
                rows[0].1.0 <= instruction.bci() && instruction.bci() < rows[0].1.1
            })
        });
        let second = self.ssa.block(block).is_some_and(|names| {
            names.instructions().iter().any(|instruction| {
                rows[1].1.0 <= instruction.bci() && instruction.bci() < rows[1].1.1
            })
        });
        match (first, second) {
            (true, false) => self.finally_edges_accounted(block, (rows[0], Some(rows[2]))),
            (false, true) => self.finally_edges_accounted(block, (rows[1], Some(rows[3]))),
            (false, false) | (true, true) => false,
        }
    }

    fn multi_return_finally_edges_accounted(
        &self,
        block: &CanonicalBlockId,
        rows: [(u32, (u32, u32)); 2],
    ) -> bool {
        let matches = rows
            .iter()
            .copied()
            .filter(|(_, span)| {
                self.ssa.block(block).is_some_and(|names| {
                    names
                        .instructions()
                        .iter()
                        .any(|step| span.0 <= step.bci() && step.bci() < span.1)
                })
            })
            .collect::<Vec<_>>();
        let [row] = matches.as_slice() else {
            return false;
        };
        self.finally_edges_accounted(block, (*row, None))
    }

    fn finally_edges_accounted(
        &self,
        block: &CanonicalBlockId,
        rows: ((u32, (u32, u32)), Option<(u32, (u32, u32))>),
    ) -> bool {
        let mut accounted = false;
        for edge in self
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == block)
        {
            match edge.kind() {
                CanonicalEdgeKind::Exception { handler_ordinal } => {
                    let matches =
                        [Some(rows.0), rows.1]
                            .into_iter()
                            .flatten()
                            .any(|(ordinal, span)| {
                                handler_ordinal == ordinal
                                    && self.canonical.handler_rows().iter().any(|row| {
                                        row.ordinal() == ordinal && row.handler() == Some(edge.to())
                                    })
                                    && self.ssa.block(block).is_some_and(|names| {
                                        names.instructions().iter().any(|instruction| {
                                            span.0 <= instruction.bci()
                                                && instruction.bci() < span.1
                                        })
                                    })
                            });
                    if !matches {
                        return false;
                    }
                    accounted = true;
                }
                CanonicalEdgeKind::Call { .. } => return false,
                CanonicalEdgeKind::Normal | CanonicalEdgeKind::Return { .. } => {}
            }
        }
        accounted
    }

    /// Whether every edge a block of a nested synchronized body leaves the normal flow through is
    /// an exception edge the monitor statement's own table states: a row that covers an
    /// instruction of the block and names a handler the enclosing monitor plan owns. Those edges
    /// are the statement's own cleanup machinery — the outer row's, and the inner pair's — so the
    /// walk writes the statements the body protects without quoting them for handlers the plan
    /// already owns. A subroutine entry edge is never accounted.
    fn monitor_edges_accounted(
        &self,
        block: &CanonicalBlockId,
        owned: &[CanonicalBlockId],
    ) -> bool {
        let mut accounted = false;
        for edge in self
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == block)
        {
            match edge.kind() {
                CanonicalEdgeKind::Exception { handler_ordinal } => {
                    let matches = self.handlers.iter().any(|row| {
                        row.ordinal == handler_ordinal
                            && owned.contains(edge.to())
                            && self.ssa.block(block).is_some_and(|names| {
                                names.instructions().iter().any(|instruction| {
                                    row.start_bci <= instruction.bci()
                                        && instruction.bci() < row.end_bci
                                })
                            })
                    });
                    if !matches {
                        return false;
                    }
                    accounted = true;
                }
                CanonicalEdgeKind::Call { .. } => return false,
                CanonicalEdgeKind::Normal | CanonicalEdgeKind::Return { .. } => {}
            }
        }
        accounted
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
    fn edges_accounted_by_catches(&self, block: &CanonicalBlockId, frame: &Frame) -> bool {
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
                    if !self.exception_edge_accounted(block, edge.to(), handler_ordinal, frame) {
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
        frame: &Frame,
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
        if (row.catch_type_index.is_none()
            && !(self.handlers.len() == 1 && frame.own_try == self.view.index_of(block)))
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
    /// order. For header-tested loops, a *throwing* read — an array length, or an array element
    /// (`recover-postfix-condition-positions`) — can stay there only when its SSA value has exactly
    /// one consumer, so the expression tree cannot evaluate the throwing read twice. Hoisting it
    /// out would run it once, and putting it in the body would run it after the test, so the
    /// structure is quoted instead. The two effects that are their expression's own machinery — the
    /// copy-and-store dance, and the `iload; iinc` pair of a postfix condition position — are
    /// stated in `test_expression_instruction`.
    fn test_is_pure(
        &self,
        block: &CanonicalBlockId,
        test_bci: u32,
        allow_array_length: bool,
    ) -> Result<(), FallbackReason> {
        let Some(names) = self.ssa.block(block) else {
            return Ok(());
        };
        let condition_bcis = self.condition_value_bcis(block, test_bci, names);
        let reads_per_value = block_reads_per_value(names);
        for instruction in names.instructions() {
            if instruction.bci() == test_bci {
                continue;
            }
            if !self.test_expression_instruction(
                block,
                test_bci,
                instruction,
                &condition_bcis,
                &reads_per_value,
                allow_array_length,
            ) {
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

    /// Whether one instruction of a test block belongs to the value expression the block's test
    /// writes.
    ///
    /// A `Push`, a `Load`, an `Arithmetic`, a `Negate` or a numeric comparison is an operand's own
    /// text. A value the test reads through the block's own producers may stay there when it is a
    /// call or a field read the branch consumes; a *throwing* read — an array length, or an array
    /// element (`recover-postfix-condition-positions`) — may stay only when its value has exactly
    /// one reader in the block, so the expression tree cannot evaluate the read twice and the read
    /// still runs once per evaluation, where the bytecode ran it. The copy-and-store dance a loop
    /// test may hold, and the `iload slot; iinc slot, ±1` pair a proved snapshot absorbs, are the
    /// two effects that are their expression's own machinery.
    fn test_expression_instruction(
        &self,
        block: &CanonicalBlockId,
        test_bci: u32,
        instruction: &SsaInstruction,
        condition_bcis: &BTreeSet<u32>,
        reads_per_value: &BTreeMap<ValueId, usize>,
        allow_array_length: bool,
    ) -> bool {
        let operation = self.operations.get(instruction.bci());
        if matches!(
            operation,
            Some(
                Operation::Push(_)
                    | Operation::Load { .. }
                    | Operation::Arithmetic { .. }
                    | Operation::Negate
                    | Operation::NumericComparison { .. },
            )
        ) {
            return true;
        }
        // A copy and the store it feeds, when the value that store writes is one no reader can
        // observe: the assignment dance of `recover-dup-store-conditional`, whose eliminated form
        // writes neither of them and whose store is the one write the bytecode's own program cannot
        // tell was run. Both halves are required together — a copy that feeds an observable store,
        // or a store anything reads, is the effect this precondition exists for and is refused, at
        // its own BCI.
        if self.unobservable_store_dance_part(block, instruction) {
            return true;
        }
        // The same dance with a store the program *can* observe: the copy hands one value to the
        // store and the other to the test, so both instructions are still the expression's own
        // machinery — the assignment is written where the bytecode ran it, at the test's own
        // operand position (`recover-loop-test-copy-store`). Whether the store's target is read
        // later decides *which* presentation the builder writes, and the builder's own proof is
        // what proves the dance; this precondition only states that the block holds no effect the
        // test's text has nowhere to put.
        if self.store_dance_part(block, test_bci, instruction) {
            return true;
        }
        // The increment and the load of the slot it updates, when the old value the load took is
        // read once after the update by the condition itself (`recover-postfix-condition-
        // positions`): the test writes the postfix expression where the bytecode read the value,
        // and the increment is absorbed into it instead of writing a statement of its own.
        if self.snapshot_condition_part(block, test_bci, instruction, condition_bcis) {
            return true;
        }
        if !condition_bcis.contains(&instruction.bci()) {
            return false;
        }
        let single_reader = instruction.writes().len() == 1
            && instruction
                .writes()
                .iter()
                .all(|(_, value)| reads_per_value.get(value) == Some(&1));
        match operation {
            Some(
                Operation::Invoke(_)
                | Operation::Field {
                    access: crate::facts::FieldAccess::Read,
                    ..
                },
            ) => true,
            Some(Operation::ArrayLength) => allow_array_length && single_reader,
            Some(Operation::ArrayLoad | Operation::ArrayElementLoad { .. }) => single_reader,
            _ => false,
        }
    }

    /// Whether one instruction is a half of the `iload slot; iinc slot, ±1` pair whose old value
    /// the condition reads after the update — the postfix condition position
    /// `recover-postfix-condition-positions` presents.
    ///
    /// The pair is the increment and the load of the very slot it updates, and the increment must
    /// read the value that load read: that is what says the increment is that load's own update and
    /// not a second one. The value the load pushed must have exactly one consumer, in this block,
    /// after the update, and that consumer must be part of the condition (or the branch itself);
    /// nothing may read the value the update wrote in between; and the consumer must not be a store
    /// back into the same slot — `i = i++` keeps the refusal the Non-Goal states. Both halves are
    /// checked together, so a load that feeds anything else, and an increment whose old value has a
    /// second reader, keep the `StatementFree` refusal they have always had, at their own BCI.
    fn snapshot_condition_part(
        &self,
        block: &CanonicalBlockId,
        test_bci: u32,
        instruction: &SsaInstruction,
        condition_bcis: &BTreeSet<u32>,
    ) -> bool {
        let Some(names) = self.ssa.block(block) else {
            return false;
        };
        let Some(position) = names
            .instructions()
            .iter()
            .position(|candidate| candidate.bci() == instruction.bci())
        else {
            return false;
        };
        let (load, increment) = match self.operations.get(instruction.bci()) {
            Some(Operation::Increment { .. }) => {
                let Some(load) = position
                    .checked_sub(1)
                    .and_then(|position| names.instructions().get(position))
                else {
                    return false;
                };
                (load, instruction)
            }
            Some(Operation::Load { .. }) => {
                let Some(increment) = names.instructions().get(position + 1) else {
                    return false;
                };
                (instruction, increment)
            }
            _ => return false,
        };
        let (
            Some(Operation::Load { slot: read }),
            Some(Operation::Increment {
                slot: written,
                amount,
            }),
        ) = (
            self.operations.get(load.bci()),
            self.operations.get(increment.bci()),
        )
        else {
            return false;
        };
        if read != written || !matches!(amount, 1 | -1) {
            return false;
        }
        let (Some(old), Some(loaded)) = (local_read(load, *read), stack_output(load)) else {
            return false;
        };
        if local_read(increment, *written) != Some(old) {
            return false;
        }
        let Some(updated) = local_write(increment, *written) else {
            return false;
        };
        let uses = self.ssa.value(loaded).uses();
        let [consumer] = uses else {
            return false;
        };
        let Some(consumer_bci) = consumer.bci() else {
            return false;
        };
        if consumer.block() != block || consumer_bci <= increment.bci() {
            return false;
        }
        if consumer_bci != test_bci && !condition_bcis.contains(&consumer_bci) {
            return false;
        }
        // A text between the update and the consumer that reads the updated value would see the
        // increment where the bytecode had not run it yet.
        if self.ssa.value(updated).uses().iter().any(|use_| {
            use_.block() == block
                && use_
                    .bci()
                    .is_some_and(|bci| bci > increment.bci() && bci < consumer_bci)
        }) {
            return false;
        }
        !matches!(
            self.operations.get(consumer_bci),
            Some(Operation::Store { slot: written }) if *written == *read
        )
    }

    /// The test blocks of one condition chain that present a postfix condition position — a half of
    /// an `iload slot; iinc slot, ±1` pair whose old value the test reads
    /// (`recover-postfix-condition-positions`) — with the pair's own instruction. The chain rules
    /// read this to state the slice's own bound: one position, at one end of the chain.
    fn chain_snapshot_positions(
        &mut self,
        tests: &[(CanonicalBlockId, u32, Continuation)],
    ) -> Result<Vec<(usize, u32)>, StopReason> {
        let mut positions = Vec::new();
        for (index, (block, test_bci, _)) in tests.iter().enumerate() {
            let Some(names) = self.ssa.block(block) else {
                continue;
            };
            poll(self.budget, Some(*test_bci))?;
            charge(
                self.budget,
                CountedBudgetDimension::IrItems,
                u64::try_from(names.instructions().len()).unwrap_or(u64::MAX),
                Some(*test_bci),
            )?;
            let condition_bcis = self.condition_value_bcis(block, *test_bci, names);
            if let Some(instruction) = names.instructions().iter().find(|instruction| {
                instruction.bci() != *test_bci
                    && self.snapshot_condition_part(block, *test_bci, instruction, &condition_bcis)
            }) {
                positions.push((index, instruction.bci()));
            }
        }
        Ok(positions)
    }

    /// Whether one instruction is a half of a `dup; store` pair whose store writes a value no
    /// reader observes — the one store a loop's own test block may hold.
    ///
    /// The pair is the assignment dance `recover-dup-store-conditional` presents: the copy hands one
    /// value to the store and the other to the test, and the store target has no reader, so the
    /// eliminated form writes neither instruction and the value flows into the condition. Both
    /// halves are checked together, so a copy that feeds an *observable* store, and a store anything
    /// reads, keep the `StatementFree` refusal they have always had (and at the BCI they named).
    fn unobservable_store_dance_part(
        &self,
        block: &CanonicalBlockId,
        instruction: &SsaInstruction,
    ) -> bool {
        let Some(names) = self.ssa.block(block) else {
            return false;
        };
        let Some(position) = names
            .instructions()
            .iter()
            .position(|candidate| candidate.bci() == instruction.bci())
        else {
            return false;
        };
        let (duplicate, store) = match self.operations.get(instruction.bci()) {
            Some(Operation::Duplicate) => {
                let Some(store) = names.instructions().get(position + 1) else {
                    return false;
                };
                (instruction, store)
            }
            Some(Operation::Store { .. }) => {
                let Some(duplicate) = position
                    .checked_sub(1)
                    .and_then(|position| names.instructions().get(position))
                else {
                    return false;
                };
                (duplicate, instruction)
            }
            _ => return false,
        };
        duplicate.opcode() == 0x59
            && matches!(
                self.operations.get(store.bci()),
                Some(Operation::Store { .. })
            )
            && store
                .writes()
                .iter()
                .all(|(_, value)| self.ssa.value(*value).uses().is_empty())
    }

    /// Whether one instruction is a half of the `dup; store` pair whose stored value the test
    /// consumes — the copy-and-store dance at a test block's own position
    /// (`recover-dup-store-conditional`, and its loop-test position
    /// `recover-loop-test-copy-store`).
    ///
    /// The pair is the assignment dance: the copy produces two values, the store takes one into a
    /// local slot and the test reads the other, so the assignment's text is the test's own operand
    /// and no instruction of the pair writes an effect of its own. Both halves are required
    /// together: a copy whose values anything else consumes, and a store whose value no test reads,
    /// keep the `StatementFree` refusal they have always had, at the BCI they named.
    fn store_dance_part(
        &self,
        block: &CanonicalBlockId,
        test_bci: u32,
        instruction: &SsaInstruction,
    ) -> bool {
        let Some(names) = self.ssa.block(block) else {
            return false;
        };
        let Some(position) = names
            .instructions()
            .iter()
            .position(|candidate| candidate.bci() == instruction.bci())
        else {
            return false;
        };
        let (duplicate, store) = match self.operations.get(instruction.bci()) {
            Some(Operation::Duplicate) => {
                let Some(store) = names.instructions().get(position + 1) else {
                    return false;
                };
                (instruction, store)
            }
            Some(Operation::Store { .. }) => {
                let Some(duplicate) = position
                    .checked_sub(1)
                    .and_then(|position| names.instructions().get(position))
                else {
                    return false;
                };
                (duplicate, instruction)
            }
            _ => return false,
        };
        if duplicate.opcode() != 0x59
            || !matches!(
                self.operations.get(store.bci()),
                Some(Operation::Store { .. })
            )
        {
            return false;
        }
        // The store writes one local slot the **body** declares and takes one of the copy's two
        // values: a parameter's declaration is the signature, and the in-place assignment
        // expression this position writes is the copy family's local form — a parameter target
        // keeps the refusal the loop's own test has always stated for it.
        let [(Slot::Local(slot), _)] = store.writes() else {
            return false;
        };
        if self.slot_is_parameter(*slot) {
            return false;
        }
        let Some((_, stored)) = crate::build::stack_operands(store).last().copied() else {
            return false;
        };
        // The test reads the copy's other value: the two copies have exactly these consumers.
        let copies: Vec<ValueId> = duplicate
            .writes()
            .iter()
            .filter_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value))
            .collect();
        let [first, second] = copies.as_slice() else {
            return false;
        };
        let Some(test) = names
            .instructions()
            .iter()
            .find(|candidate| candidate.bci() == test_bci)
        else {
            return false;
        };
        let tested = |value: &ValueId| {
            test.reads()
                .iter()
                .any(|(slot, read)| matches!(slot, Slot::Stack(_)) && read == value)
        };
        (stored == *first && tested(second)) || (stored == *second && tested(first))
    }

    /// Whether one local slot is a parameter (or the receiver) of this method.
    ///
    /// The method's entry block's own entry state defines exactly the slots the signature declares,
    /// so a slot it defines is one the body does not declare — the same fact the builder's own
    /// `parameters` count states, read from the graph rather than from the descriptor.
    fn slot_is_parameter(&self, slot: u16) -> bool {
        self.ssa.blocks().first().is_some_and(|block| {
            block
                .entry()
                .iter()
                .any(|(entry, _)| matches!(entry, Slot::Local(entry) if *entry == slot))
        })
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
    ) -> Result<Option<(Vec<Region>, Option<CanonicalBlockId>)>, StopReason> {
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
        // Two successors mean the consumer block itself branches once more: the anchor this
        // region consumes lives there and the branch is the continuation's own first test, so
        // the block is claimed here and the continuation composed from it. Anything with more
        // than two successors is no two-way shape this slice reads.
        let consumer_successors = self.view.successors(consumer_node);
        let consumer_two_way = consumer_successors.len() == 2;
        if self.view.successor_ids(&false_producer) != [consumer.clone()]
            || consumer_successors.len() > 2
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
        // A consumer block that ends in a branch stores this region's value **and** starts the
        // next structure: the two-way comparison case composes the continuation from this very
        // block (below), and everything else — a `switch` there, a two-way exit of a
        // protected-range candidate — keeps the whole-shape refusal this slice has always taken.
        let consumer_terminal_branches =
            self.operations
                .get(consumer_terminal.bci())
                .is_some_and(|operation| {
                    operation.comparison().is_some() || operation.switch().is_some()
                });
        if consumer_terminal_branches && !(consumer_two_way && protected_try.is_none()) {
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
        if consumer_two_way && consumer_terminal_branches {
            // The consumer block stores this region's value and then branches into the next
            // structure, whose first test is that very branch: no disjoint join block exists for
            // a sibling to start at, so the continuation is recognized from the consumer block
            // itself — the nested walk marks the block visited as its dispatch — and the whole
            // continuation composes into one statement sequence at one lexical path. A
            // continuation that does not close rolls the whole claim back, which is the decline
            // this shape has always taken; a stop (budget, cancellation, recursion bound) stops
            // the run as any other.
            let checkpoint = self.visited.clone();
            let mut tail = Vec::new();
            let mut continuation = Some(consumer.clone());
            let tail_next = loop {
                let Some(start) = continuation.clone() else {
                    break None;
                };
                match self.region_at(&start, frame) {
                    Ok((run, next)) => {
                        tail.extend(run);
                        continuation = next;
                    }
                    Err(stop) => return Err(stop),
                }
            };
            if tail.is_empty() {
                self.visited = checkpoint;
                return Ok(None);
            }
            for node in participants {
                self.visited.insert(node);
            }
            let mut run = Vec::with_capacity(tail.len() + 1);
            run.push(Region::ShortCircuitValue {
                prefix: prefix.to_vec(),
                tests,
                test_edges,
                gateways,
                true_producer,
                false_producer,
                consumer,
                consumer_bci,
                reason,
                tail,
            });
            return Ok(Some((run, tail_next)));
        }
        for node in participants {
            self.visited.insert(node);
        }
        Ok(Some((
            vec![Region::ShortCircuitValue {
                prefix: prefix.to_vec(),
                tests,
                test_edges,
                gateways,
                true_producer,
                false_producer,
                consumer,
                consumer_bci,
                reason,
                tail: Vec::new(),
            }],
            next,
        )))
    }

    /// Claim one two-test arm whose paths end either at its one return or the enclosing shared
    /// tail. The return is a single physical owner even though both tests can enter it.
    fn shared_tail_early_return(
        &mut self,
        prefix: &[CanonicalBlockId],
        outer: &CanonicalBlockId,
        outer_bci: u32,
        frame: &Frame,
    ) -> Result<Option<Region>, StopReason> {
        let Some(tail_node) = frame
            .shared_tail
            .filter(|tail| frame.boundary == Some(*tail))
        else {
            return Ok(None);
        };
        if !self.return_is_boolean || frame.scope.is_some() || frame.own_try.is_some() {
            return Ok(None);
        }
        let Some(outer_node) = self.view.index_of(outer) else {
            return Ok(None);
        };
        let outer_successors = self.view.successor_ids(outer);
        let [first, second] = outer_successors.as_slice() else {
            return Ok(None);
        };
        let mut shape = None;
        for (test, returned) in [(first, second), (second, first)] {
            let Some(test_node) = self.view.index_of(test) else {
                continue;
            };
            let Some(return_node) = self.view.index_of(returned) else {
                continue;
            };
            let test_successors = self.view.successor_ids(test);
            if test_successors.len() == 2
                && test_successors.contains(returned)
                && test_successors
                    .iter()
                    .any(|block| self.view.index_of(block) == Some(tail_node))
                && self.view.successors(return_node).is_empty()
                && self
                    .terminal_bci(returned)
                    .and_then(|bci| self.operations.get(bci))
                    .is_some_and(|operation| matches!(operation, Operation::Return))
            {
                if shape
                    .replace((test.clone(), returned.clone(), test_node, return_node))
                    .is_some()
                {
                    return Ok(None);
                }
            }
        }
        let Some((inner, returned, inner_node, return_node)) = shape else {
            return Ok(None);
        };
        if self.visited.contains(&inner_node)
            || self.visited.contains(&return_node)
            || outer.path() != inner.path()
            || outer.path() != returned.path()
            || !self.view.dominates(outer_node, inner_node)
            || !self.view.dominates(outer_node, return_node)
        {
            return Ok(None);
        }
        let Some(tail) = self.view.id_of(tail_node).cloned() else {
            return Ok(None);
        };
        let Some(paths) = self.proved_forward_paths(
            outer_node,
            &[self.view.index_of(first), self.view.index_of(second)],
            Some(tail_node),
            outer_bci,
        )?
        else {
            return Ok(None);
        };
        if !paths
            .iter()
            .all(|path| path.contains(&return_node) || path.contains(&tail_node))
            || !paths.iter().any(|path| path.contains(&tail_node))
        {
            return Ok(None);
        }
        let mut incoming_inner = Vec::new();
        let mut incoming_return = Vec::new();
        for edge in self.canonical.edges() {
            poll(self.budget, Some(outer_bci))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(outer_bci),
            )?;
            if edge.to() == &inner {
                incoming_inner.push((edge.kind(), edge.from().clone()));
            }
            if edge.to() == &returned {
                incoming_return.push((edge.kind(), edge.from().clone()));
            }
        }
        if !exact_normal_predecessors(&incoming_inner, &[outer.clone()])
            || !exact_normal_predecessors(&incoming_return, &[outer.clone(), inner.clone()])
        {
            return Ok(None);
        }
        let Some(inner_bci) = self.terminal_bci(&inner) else {
            return Ok(None);
        };
        let Some((outer_op, outer_target)) = self
            .operations
            .get(outer_bci)
            .and_then(Operation::comparison)
        else {
            return Ok(None);
        };
        let Some((inner_op, inner_target)) = self
            .operations
            .get(inner_bci)
            .and_then(Operation::comparison)
        else {
            return Ok(None);
        };
        if self
            .branch_arity_proved(outer, outer_bci, outer_op)
            .is_err()
            || self
                .branch_arity_proved(&inner, inner_bci, inner_op)
                .is_err()
        {
            return Ok(None);
        }
        let Some(outer_edges) = self.split_arms(&outer_successors, outer_target) else {
            return Ok(None);
        };
        let Some(inner_edges) = self.split_arms(&self.view.successor_ids(&inner), inner_target)
        else {
            return Ok(None);
        };
        charge(
            self.budget,
            CountedBudgetDimension::IrItems,
            2,
            Some(outer_bci),
        )?;
        self.visited.extend([inner_node, return_node]);
        Ok(Some(Region::SharedTailEarlyReturn {
            prefix: prefix.to_vec(),
            tests: [(outer.clone(), outer_bci), (inner, inner_bci)],
            test_edges: [outer_edges, inner_edges],
            return_block: returned,
            join: tail,
        }))
    }

    /// Enumerate a branch's strictly forward normal paths. A path may end only at an explicit
    /// return or at the inherited tail. Each edge and block examined is charged before use.
    fn proved_forward_paths(
        &mut self,
        branch: usize,
        starts: &[Option<usize>; 2],
        tail: Option<usize>,
        at: u32,
    ) -> Result<Option<[BTreeSet<usize>; 2]>, StopReason> {
        let mut adjacency: BTreeMap<usize, Vec<(CanonicalEdgeKind, usize)>> = BTreeMap::new();
        for edge in self.canonical.edges() {
            poll(self.budget, Some(at))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(at),
            )?;
            let (Some(from), Some(to)) = (
                self.view.index_of(edge.from()),
                self.view.index_of(edge.to()),
            ) else {
                return Ok(None);
            };
            adjacency.entry(from).or_default().push((edge.kind(), to));
        }
        let mut paths = [BTreeSet::new(), BTreeSet::new()];
        for (index, start) in starts.iter().enumerate() {
            let Some(start) = start else { return Ok(None) };
            let mut pending = vec![*start];
            while let Some(node) = pending.pop() {
                poll(self.budget, Some(at))?;
                charge(
                    self.budget,
                    CountedBudgetDimension::AnalysisSteps,
                    1,
                    Some(at),
                )?;
                if !paths[index].insert(node) {
                    continue;
                }
                let Some(block) = self.view.id_of(node) else {
                    return Ok(None);
                };
                let Some(origin) = self.view.id_of(branch) else {
                    return Ok(None);
                };
                if block.path() != origin.path()
                    || (Some(node) != tail
                        && (block.bci() <= origin.bci()
                            || self.view.is_loop_header(node)
                            || (tail.is_none() && !self.view.dominates(branch, node))))
                {
                    return Ok(None);
                }
                if Some(node) == tail {
                    continue;
                }
                let mut successors = Vec::new();
                for &(kind, next) in adjacency.get(&node).into_iter().flatten() {
                    poll(self.budget, Some(at))?;
                    charge(
                        self.budget,
                        CountedBudgetDimension::AnalysisSteps,
                        1,
                        Some(at),
                    )?;
                    if kind != CanonicalEdgeKind::Normal {
                        return Ok(None);
                    }
                    let Some(destination) = self.view.id_of(next) else {
                        return Ok(None);
                    };
                    if destination.path() != block.path() || destination.bci() <= block.bci() {
                        return Ok(None);
                    }
                    successors.push(next);
                }
                if successors.is_empty() {
                    if !self
                        .terminal_bci(block)
                        .and_then(|bci| self.operations.get(bci))
                        .is_some_and(|operation| matches!(operation, Operation::Return))
                    {
                        return Ok(None);
                    }
                } else {
                    pending.extend(successors);
                }
            }
        }
        Ok(Some(paths))
    }

    /// A unique first block shared by both normal forward paths is the enclosing continuation.
    /// The predecessor certificate excludes side entries and back edges before arm ownership moves.
    fn shared_forward_join(
        &mut self,
        branch: usize,
        then_node: Option<usize>,
        else_node: Option<usize>,
        at: u32,
    ) -> Result<Option<usize>, StopReason> {
        let Some([then_path, else_path]) =
            self.proved_forward_paths(branch, &[then_node, else_node], None, at)?
        else {
            return Ok(None);
        };
        let entered: BTreeSet<_> = then_path.union(&else_path).copied().collect();
        for edge in self.canonical.edges() {
            poll(self.budget, Some(at))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(at),
            )?;
            let Some(target) = self.view.index_of(edge.to()) else {
                return Ok(None);
            };
            if !entered.contains(&target) {
                continue;
            }
            let Some(source) = self.view.index_of(edge.from()) else {
                return Ok(None);
            };
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(self.view.len().saturating_mul(2)).unwrap_or(u64::MAX),
                Some(at),
            )?;
            if edge.kind() != CanonicalEdgeKind::Normal
                || !self.view.dominates(branch, source)
                || self.view.dominates(target, source)
            {
                return Ok(None);
            }
        }
        let common: BTreeSet<_> = then_path.intersection(&else_path).copied().collect();
        if common.is_empty() {
            return Ok(None);
        }
        poll(self.budget, Some(at))?;
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(
                common
                    .len()
                    .saturating_mul(self.canonical.edges().len().saturating_add(self.view.len())),
            )
            .unwrap_or(u64::MAX),
            Some(at),
        )?;
        let Some(candidate) = unique_first_common(&common, |node| self.view.reachable(node)) else {
            return Ok(None);
        };
        let Some(join) = self.view.id_of(candidate) else {
            return Ok(None);
        };
        if !self.forward_join_predecessors(branch, candidate)
            || self
                .canonical
                .edges()
                .iter()
                .any(|edge| edge.to() == join && edge.kind() != CanonicalEdgeKind::Normal)
        {
            return Ok(None);
        }
        Ok(Some(candidate))
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

    /// The one block an `if`/`else if` ladder's arms regroup on when one of them terminates the
    /// method.
    ///
    /// A branch whose arm returns has **no** post-dominator: the returning route never passes the
    /// block the other arms meet at, so no block lies on every path out of the branch and every
    /// join reading above states nothing. The walk then keeps the frame's own boundary for both
    /// arms, and inside a loop that boundary is the header: the first arm runs through the loop's
    /// latch block and claims it, the second arm reaches the same block and re-enters it, which the
    /// walk quotes as a loop ([`FallbackReason::Loop`]) and the completed tree owns twice. The
    /// ladder's join is a fact of the graph all the same, and this is the reading of it:
    ///
    /// * the branch is inside a loop and the frame's boundary is that loop's own continuation
    ///   (its header, or the update block a proved for-header owns) — a ladder outside a loop is
    ///   read by [`Self::shared_forward_join`] and the readings above, and this one stays out of
    ///   their way;
    /// * both arms reach one and the same block by strictly forward normal edges inside the
    ///   loop's own scope, and that block is the first one both of them hold
    ///   ([`unique_first_common`]) — the arms' own regrouping point, not an address;
    /// * no route of either arm reaches the frame's boundary without passing through it: every
    ///   path that continues the loop runs the statements after the `if`, which is what makes the
    ///   join's own statements exactly the ones the bytecode runs;
    /// * every way into the candidate comes through this branch ([`Self::forward_join_predecessors`]);
    /// * at most one branch lies on any route from an arm to the candidate: this reading presents
    ///   a **single-level** ladder (one `else if` step), which is the scope this change states. A
    ///   deeper chain's routes regroup on a block its own levels have not each been proved to
    ///   reach, so it keeps its refusal.
    fn ladder_join(
        &mut self,
        branch: usize,
        then_node: Option<usize>,
        else_node: Option<usize>,
        frame: &Frame,
        at: u32,
    ) -> Result<Option<usize>, StopReason> {
        let Some(target) = frame.loop_targets.last() else {
            return Ok(None);
        };
        let Some(boundary) = frame.boundary else {
            return Ok(None);
        };
        let Some(loop_of) = self.view.loop_entered_at(target.header) else {
            return Ok(None);
        };
        let (Some(then_node), Some(else_node)) = (then_node, else_node) else {
            return Ok(None);
        };
        if then_node == else_node
            || (boundary != target.header && boundary != target.continue_target)
        {
            // Two edges onto one block state no arm split at all, and a frame that ends anywhere
            // but where this loop continues is not the loop body's own frame.
            return Ok(None);
        }
        let blocks = loop_of.blocks();
        let (Some(then_routes), Some(else_routes)) = (
            self.arm_forward_routes(then_node, None, boundary, frame, at)?,
            self.arm_forward_routes(else_node, None, boundary, frame, at)?,
        ) else {
            return Ok(None);
        };
        let common: BTreeSet<usize> = then_routes
            .blocks
            .intersection(&else_routes.blocks)
            .copied()
            .collect();
        let Some(candidate) = unique_first_common(&common, |node| self.view.reachable(node)) else {
            return Ok(None);
        };
        if candidate == boundary || !blocks.contains(&candidate) {
            return Ok(None);
        }
        let (Some(then_stop), Some(else_stop)) = (
            self.arm_forward_routes(then_node, Some(candidate), boundary, frame, at)?,
            self.arm_forward_routes(else_node, Some(candidate), boundary, frame, at)?,
        ) else {
            return Ok(None);
        };
        if !then_stop.reached_stop
            || !else_stop.reached_stop
            || then_stop.reached_boundary
            || else_stop.reached_boundary
        {
            return Ok(None);
        }
        if !self.forward_join_predecessors(branch, candidate) {
            return Ok(None);
        }
        let Some(join) = self.view.id_of(candidate) else {
            return Ok(None);
        };
        if self
            .canonical
            .edges()
            .iter()
            .any(|edge| edge.to() == join && edge.kind() != CanonicalEdgeKind::Normal)
        {
            return Ok(None);
        }
        Ok(Some(candidate))
    }

    /// One arm's strictly forward routes inside the enclosing loop, up to the block the ladder's
    /// arms regroup on (`stop`), or to the frame's boundary when no candidate is named yet.
    ///
    /// The walk is the arm's own: it moves forward in one body's BCI order, and a route ends where
    /// the arm's walk ends — at a terminal block (the early-return arm's `return`), at the frame's
    /// boundary, or at the block the caller named. `reached_boundary` is the answer the election
    /// needs: a route that arrives at the frame's boundary **without** the stop block would run
    /// past the join the `if` presents.
    ///
    /// Anything this reading cannot state is a refusal, never a guess: a route that leaves the
    /// frame's scope, enters a nested loop's header, or passes more than one branch keeps the
    /// shape out of the single-level ladder this change presents. Every block and edge examined is
    /// charged before use.
    fn arm_forward_routes(
        &mut self,
        start: usize,
        stop: Option<usize>,
        boundary: usize,
        frame: &Frame,
        at: u32,
    ) -> Result<Option<ArmRoutes>, StopReason> {
        let mut routes = ArmRoutes {
            blocks: BTreeSet::new(),
            reached_stop: false,
            reached_boundary: false,
        };
        let mut seen: BTreeMap<usize, usize> = BTreeMap::new();
        let mut pending = vec![(start, 0usize)];
        while let Some((node, branches)) = pending.pop() {
            poll(self.budget, Some(at))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(at),
            )?;
            if Some(node) == stop {
                routes.reached_stop = true;
                routes.blocks.insert(node);
                continue;
            }
            if node == boundary {
                routes.reached_boundary = true;
                continue;
            }
            if self.view.is_loop_header(node)
                || frame
                    .scope
                    .as_ref()
                    .is_some_and(|scope| !scope.contains(&node))
            {
                // A nested loop is a structure of its own, and a block the loop's body does not
                // hold is one the arm's own walk stops at: neither is a route of this ladder.
                return Ok(None);
            }
            match seen.get(&node) {
                Some(previous) if *previous <= branches => continue,
                _ => {
                    seen.insert(node, branches);
                }
            }
            routes.blocks.insert(node);
            let successors = self.view.successors(node);
            if successors.is_empty() {
                // The route leaves the method: the early-return arm's `return` is where the
                // ladder's other arm terminates, and nothing after the `if` runs on it.
                continue;
            }
            let step = usize::from(successors.len() > 1);
            if branches + step > LADDER_MAX_BRANCHES {
                return Ok(None);
            }
            let Some(block) = self.view.id_of(node) else {
                return Ok(None);
            };
            for successor in successors {
                poll(self.budget, Some(at))?;
                charge(
                    self.budget,
                    CountedBudgetDimension::AnalysisSteps,
                    1,
                    Some(at),
                )?;
                if Some(successor) == stop || successor == boundary {
                    // The block a route ends at is where the walk stops, whichever way the
                    // address runs: the loop's own back edge is one of those ends.
                    pending.push((successor, branches + step));
                    continue;
                }
                let Some(destination) = self.view.id_of(successor) else {
                    return Ok(None);
                };
                if destination.path() != block.path() || destination.bci() <= block.bci() {
                    // A route that does not move forward in this body is no route of a ladder: a
                    // back edge is the loop's own, and a `jsr` clone is another body.
                    return Ok(None);
                }
                pending.push((successor, branches + step));
            }
        }
        Ok(Some(routes))
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
    /// Header and latch tests remain inside their loop statement (`while`/`for` or `do … while`).
    /// The narrow effectful dual-exit shape instead keeps its header test as the body's first
    /// `If` inside `while (true)`. Each shape checks its entry, latch, edges and ownership before
    /// building; a loop without that proof is quoted with its reason.
    fn include_fragmented_catch_scope(&self, frame: &mut Frame, header: usize) {
        if let Some(extra) = self
            .fragmented
            .as_ref()
            .and_then(|proof| proof.supplemental_scope(header))
            && let Some(scope) = frame.scope.as_mut()
        {
            scope.extend(extra.iter().copied());
        }
    }

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
        let header_successors = self.view.successors(header_node);
        let latch_exit = loop_of.latches().iter().find_map(|latch| {
            self.view
                .successors(*latch)
                .into_iter()
                .find(|successor| !blocks.contains(successor))
                .and_then(|node| self.view.id_of(node).cloned())
        });
        if header_successors.len() == 2 {
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                u64::try_from(self.canonical.edges().len()).unwrap_or(u64::MAX),
                Some(header.bci()),
            )?;
        }
        let body_branch = header_successors.len() == 2
            && self
                .terminal_bci(header)
                .and_then(|bci| self.operations.get(bci))
                .is_some_and(|operation| operation.comparison().is_some())
            && header_successors.iter().all(|successor| {
                blocks.contains(successor)
                    || latch_exit.as_ref().is_some_and(|exit| {
                        self.loop_exit_bridge(header_node, *successor, exit, &blocks)
                    })
            });
        if body_branch {
            if let Some(region) =
                self.latch_tested_loop(header, header_node, &blocks, frame, true)?
            {
                return Ok(region);
            }
        }
        if let Some(region) = self.effectful_dual_exit_loop(header, header_node, &loop_of, frame)? {
            return Ok(region);
        }
        if let Some(region) = self.header_tested_loop(header, header_node, &blocks, frame)? {
            return Ok(region);
        }
        if !body_branch
            && let Some(region) =
                self.latch_tested_loop(header, header_node, &blocks, frame, false)?
        {
            return Ok(region);
        }
        let reason = FallbackReason::LoopShape {
            block_bci: header.bci(),
        };
        Ok(gap(Vec::new(), vec![header.clone()], reason, None))
    }

    /// Only the two-break shape whose header's losing arm writes a local through one call.
    /// All edges and the join value are proved before changing the body frame's scope.
    fn effectful_dual_exit_loop(
        &mut self,
        header: &CanonicalBlockId,
        header_node: usize,
        loop_of: &crate::normal_flow::NaturalLoop,
        frame: &Frame,
    ) -> Result<Option<Run>, StopReason> {
        let blocks = loop_of.blocks();
        let nested_arm = match (frame.boundary, frame.if_arm) {
            (None, None) => None,
            (Some(boundary), Some((parent, entry, outer))) => {
                Some((parent, entry, boundary, outer))
            }
            _ => return Ok(None),
        };
        if frame.scope.is_some()
            || frame.shared_tail.is_some()
            || !frame.loop_targets.is_empty()
            || frame.case_entries.is_some()
            || frame.own_try.is_some()
            || frame.own_finally.is_some()
            || !self.handlers.is_empty()
            || blocks.len() != 3
        {
            return Ok(None);
        }
        let latches: Vec<_> = loop_of.latches().iter().copied().collect();
        let [latch] = latches.as_slice() else {
            return Ok(None);
        };
        let header_successors = self.view.successors(header_node);
        let [first, second] = header_successors.as_slice() else {
            return Ok(None);
        };
        let (body, effect) = match (blocks.contains(first), blocks.contains(second)) {
            (true, false) => (*first, *second),
            (false, true) => (*second, *first),
            _ => return Ok(None),
        };
        if body == header_node || body == *latch || self.view.successors(*latch) != [header_node] {
            return Ok(None);
        }
        let body_successors = self.view.successors(body);
        let [body_first, body_second] = body_successors.as_slice() else {
            return Ok(None);
        };
        let bridge = if *body_first == *latch {
            *body_second
        } else if *body_second == *latch {
            *body_first
        } else {
            return Ok(None);
        };
        if blocks.contains(&bridge)
            || bridge == effect
            || self.loop_exit_nodes(blocks) != BTreeSet::from([effect, bridge])
        {
            return Ok(None);
        }
        let effect_successors = self.view.successors(effect);
        let [join] = effect_successors.as_slice() else {
            return Ok(None);
        };
        let join = *join;
        if self.view.successors(bridge) != [join] || blocks.contains(&join) || join == effect {
            return Ok(None);
        }
        let Some((body_id, effect_id, bridge_id, latch_id, join_id)) = (|| {
            Some((
                self.view.id_of(body)?.clone(),
                self.view.id_of(effect)?.clone(),
                self.view.id_of(bridge)?.clone(),
                self.view.id_of(*latch)?.clone(),
                self.view.id_of(join)?.clone(),
            ))
        })() else {
            return Ok(None);
        };
        let Some(header_test) = self.terminal_bci(header) else {
            return Ok(None);
        };
        let Some(body_test) = self.terminal_bci(&body_id) else {
            return Ok(None);
        };
        if !self
            .operations
            .get(header_test)
            .is_some_and(|op| op.comparison().is_some())
            || !self
                .operations
                .get(body_test)
                .is_some_and(|op| op.comparison().is_some())
            || self.test_is_pure(header, header_test, true).is_err()
        {
            return Ok(None);
        }
        let scan = self
            .canonical
            .edges()
            .len()
            .saturating_mul(16)
            .saturating_add(self.code.instructions.len().saturating_mul(8))
            .saturating_add(self.ssa.phis().len().saturating_mul(2))
            .saturating_add(
                self.ssa
                    .blocks()
                    .iter()
                    .map(|block| block.instructions().len())
                    .sum::<usize>(),
            );
        poll(self.budget, Some(header.bci()))?;
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(scan).unwrap_or(u64::MAX),
            Some(header.bci()),
        )?;
        // The shape is a single-entry natural loop with one pure latch. The two exit
        // blocks have exactly one owner each, and the join has no third normal input.
        let incoming = |to: &CanonicalBlockId| {
            self.canonical
                .edges()
                .iter()
                .filter(|edge| edge.to() == to)
                .map(|edge| (edge.kind(), edge.from().clone()))
                .collect::<Vec<_>>()
        };
        let outgoing = |from: &CanonicalBlockId| {
            self.canonical
                .edges()
                .iter()
                .filter(|edge| edge.from() == from)
                .map(|edge| (edge.kind(), edge.to().clone()))
                .collect::<Vec<_>>()
        };
        let three_way =
            nested_arm.is_some_and(|(_, _, boundary, outer)| outer.is_some() && boundary == join);
        let mut empty_id = None;
        if let Some((parent, entry, boundary, outer)) = nested_arm {
            let (Some(parent_id), Some(entry_id), Some(boundary_id)) = (
                self.view.id_of(parent),
                self.view.id_of(entry),
                self.view.id_of(boundary),
            ) else {
                return Ok(None);
            };
            let parent_successors = self.view.successors(parent);
            let [first_arm, second_arm] = parent_successors.as_slice() else {
                return Ok(None);
            };
            let other = if *first_arm == entry {
                *second_arm
            } else if *second_arm == entry {
                *first_arm
            } else {
                return Ok(None);
            };
            let Some(other_id) = self.view.id_of(other) else {
                return Ok(None);
            };
            // The second-level form has no loop-private tail: the empty sibling joins the two
            // exits at the inner boundary, which then continues to the outer boundary once.
            // The original one-level form retains its separate two-input loop join.
            if parent == entry
                || entry == boundary
                || other == boundary
                || (outer.is_some() != (join == boundary))
                || blocks.contains(&entry)
                || blocks.contains(&other)
                || self.view.immediate_post_dominator(parent) != Some(boundary)
                || !self.view.dominates(parent, header_node)
                || !self.view.dominates(entry, header_node)
                || (!three_way && !self.view.dominates(entry, join))
                || self.view.successors(entry) != [header_node]
                || (three_way && outer.is_none_or(|outer| self.view.successors(join) != [outer]))
                || (!three_way && self.view.successors(join) != [boundary])
                || self.view.successors(other) != [boundary]
                || !exact_normal_predecessors(&incoming(entry_id), &[parent_id.clone()])
                || !exact_normal_predecessors(&incoming(other_id), &[parent_id.clone()])
                || (!three_way
                    && !exact_normal_predecessors(
                        &incoming(boundary_id),
                        &[join_id.clone(), other_id.clone()],
                    ))
                || (three_way
                    && !exact_normal_predecessors(
                        &incoming(boundary_id),
                        &[effect_id.clone(), bridge_id.clone(), other_id.clone()],
                    ))
                || (!three_way
                    && !exact_normal_predecessors(&outgoing(&join_id), &[boundary_id.clone()]))
                || (three_way
                    && outer.is_none_or(|outer| {
                        self.view.id_of(outer).is_none_or(|outer_id| {
                            !exact_normal_predecessors(&outgoing(&join_id), &[outer_id.clone()])
                        })
                    }))
                || !exact_normal_predecessors(&outgoing(other_id), &[boundary_id.clone()])
                || self.leaving_edge(parent_id).is_some()
                || self.leaving_edge(entry_id).is_some()
                || self.leaving_edge(other_id).is_some()
                || self.leaving_edge(boundary_id).is_some()
            {
                return Ok(None);
            }
            if three_way {
                empty_id = Some(other_id.clone());
            }
        }
        let header_in = incoming(header);
        if header_in.len() != 2
            || nested_arm.is_some_and(|(_, entry, _, _)| {
                self.view
                    .id_of(entry)
                    .is_none_or(|entry_id| !header_in.iter().any(|(_, source)| source == entry_id))
            })
            || header_in
                .iter()
                .any(|(kind, _)| *kind != CanonicalEdgeKind::Normal)
            || header_in
                .iter()
                .filter(|(_, from)| from == &latch_id)
                .count()
                != 1
            || header_in
                .iter()
                .filter(|(_, from)| from != &latch_id)
                .any(|(_, from)| {
                    self.view
                        .index_of(from)
                        .is_none_or(|node| blocks.contains(&node))
                })
            || !exact_normal_predecessors(&incoming(&body_id), &[header.clone()])
            || !exact_normal_predecessors(&incoming(&effect_id), &[header.clone()])
            || !exact_normal_predecessors(&incoming(&bridge_id), &[body_id.clone()])
            || !exact_normal_predecessors(&incoming(&latch_id), &[body_id.clone()])
            || !exact_normal_predecessors(
                &incoming(&join_id),
                &if let Some(empty_id) = &empty_id {
                    vec![effect_id.clone(), bridge_id.clone(), empty_id.clone()]
                } else {
                    vec![effect_id.clone(), bridge_id.clone()]
                },
            )
            || !exact_normal_predecessors(&outgoing(header), &[body_id.clone(), effect_id.clone()])
            || !exact_normal_predecessors(
                &outgoing(&body_id),
                &[bridge_id.clone(), latch_id.clone()],
            )
            || !exact_normal_predecessors(&outgoing(&effect_id), &[join_id.clone()])
            || !exact_normal_predecessors(&outgoing(&bridge_id), &[join_id.clone()])
            || !exact_normal_predecessors(&outgoing(&latch_id), &[header.clone()])
            || self.view.successors(body) != [bridge, *latch]
                && self.view.successors(body) != [*latch, bridge]
            || self.leaving_edge(header).is_some()
            || [
                body_id.clone(),
                effect_id.clone(),
                bridge_id.clone(),
                latch_id.clone(),
                join_id.clone(),
            ]
            .iter()
            .any(|id| self.leaving_edge(id).is_some())
        {
            return Ok(None);
        }
        let selected: Vec<_> = [
            Some(header),
            Some(&body_id),
            Some(&effect_id),
            Some(&bridge_id),
            Some(&latch_id),
            Some(&join_id),
            empty_id.as_ref(),
        ]
        .into_iter()
        .flatten()
        .collect();
        for id in selected {
            let Some(block) = self
                .canonical
                .blocks()
                .iter()
                .find(|block| block.id() == id)
            else {
                return Ok(None);
            };
            let Some(names) = self.ssa.block(id) else {
                return Ok(None);
            };
            let decoded: Vec<_> = self
                .code
                .instructions
                .iter()
                .filter(|instruction| {
                    instruction.bci >= id.bci() && instruction.bci < block.end_bci()
                })
                .map(|instruction| instruction.bci)
                .collect();
            if id.is_clone()
                || block.blocks() != [id.bci()]
                || decoded
                    != names
                        .instructions()
                        .iter()
                        .map(|instruction| instruction.bci())
                        .collect::<Vec<_>>()
                || names
                    .instructions()
                    .iter()
                    .any(|instruction| self.operations.get(instruction.bci()).is_none())
            {
                return Ok(None);
            }
        }
        let (
            Some(effect_names),
            Some(body_names),
            Some(bridge_names),
            Some(latch_names),
            Some(join_names),
        ) = (
            self.ssa.block(&effect_id),
            self.ssa.block(&body_id),
            self.ssa.block(&bridge_id),
            self.ssa.block(&latch_id),
            self.ssa.block(&join_id),
        )
        else {
            return Ok(None);
        };
        let single_use_at_block =
            |ssa: &SsaTable, value: ValueId, block: &CanonicalBlockId, bci| {
                let [use_] = ssa.value(value).uses() else {
                    return false;
                };
                use_.block() == block && use_.bci() == Some(bci)
            };
        let effect_instructions = effect_names.instructions();
        let (store, transfer, constructor_exit) = match effect_instructions {
            [push, call, store, transfer]
                if matches!(self.operations.get(push.bci()), Some(Operation::Push(_)))
                    && matches!(self.operations.get(call.bci()), Some(Operation::Invoke(_)))
                    && call.writes().len() == 1
                    && matches!(call.writes()[0].0, Slot::Stack(_))
                    && store.reads() == call.writes()
                    && push.writes().len() == 1
                    && call.reads().contains(&push.writes()[0]) =>
            {
                (store, transfer, false)
            }
            [allocate, duplicate, argument, call, store, transfer] => {
                let (
                    Some(Operation::Allocate { ty }),
                    Some(Operation::Duplicate),
                    Some(Operation::Push(ConstantValue::String(_))),
                    Some(Operation::Invoke(target)),
                ) = (
                    self.operations.get(allocate.bci()),
                    self.operations.get(duplicate.bci()),
                    self.operations.get(argument.bci()),
                    self.operations.get(call.bci()),
                )
                else {
                    return Ok(None);
                };
                let [(Slot::Stack(0), allocated)] = allocate.writes() else {
                    return Ok(None);
                };
                let [(Slot::Stack(0), first), (Slot::Stack(1), receiver)] = duplicate.writes()
                else {
                    return Ok(None);
                };
                let [(Slot::Stack(2), parameter)] = argument.writes() else {
                    return Ok(None);
                };
                let [(Slot::Stack(0), initialized)] = call.writes() else {
                    return Ok(None);
                };
                if ty != "java/io/File"
                    || target.kind() != crate::facts::InvokeKind::Special
                    || target.owner() != ty
                    || target.name() != "<init>"
                    || target.descriptor() != "(Ljava/lang/String;)V"
                    || allocate.opcode() != 0xbb
                    || duplicate.opcode() != 0x59
                    || !matches!(argument.opcode(), 0x12 | 0x13)
                    || call.opcode() != 0xb7
                    || allocate.reads() != []
                    || duplicate.reads() != [(Slot::Stack(0), *allocated)]
                    || argument.reads() != []
                    || call.reads() != [(Slot::Stack(2), *parameter), (Slot::Stack(1), *receiver)]
                    || store.reads() != [(Slot::Stack(0), *initialized)]
                    || !self.ssa.value(*first).uses().is_empty()
                    || !single_use_at_block(self.ssa, *allocated, &effect_id, duplicate.bci())
                    || !single_use_at_block(self.ssa, *receiver, &effect_id, call.bci())
                    || !single_use_at_block(self.ssa, *parameter, &effect_id, call.bci())
                    || !single_use_at_block(self.ssa, *initialized, &effect_id, store.bci())
                    || [(*allocated, allocate.bci()), (*first, duplicate.bci()),
                        (*receiver, duplicate.bci()), (*parameter, argument.bci()),
                        (*initialized, call.bci())].iter().any(|(value, bci)| {
                        self.ssa.value(*value).replaced_by().is_some()
                            || !matches!(self.ssa.value(*value).def(), Definition::Instruction { block, bci: at } if block == &effect_id && at == bci)
                    })
                {
                    return Ok(None);
                }
                (store, transfer, true)
            }
            _ => return Ok(None),
        };
        let [bridge_transfer] = bridge_names.instructions() else {
            return Ok(None);
        };
        let [increment, latch_transfer] = latch_names.instructions() else {
            return Ok(None);
        };
        let Some((Slot::Local(slot), effect_value)) = store.writes().first().copied() else {
            return Ok(None);
        };
        let empty_value = if let Some(empty_id) = &empty_id {
            let Some(empty_names) = self.ssa.block(empty_id) else {
                return Ok(None);
            };
            let [push, empty_store, transfer] = empty_names.instructions() else {
                return Ok(None);
            };
            let Some((Slot::Local(written), value)) = empty_store.writes().first().copied() else {
                return Ok(None);
            };
            if written != slot
                || !matches!(self.operations.get(push.bci()), Some(Operation::Push(_)))
                || !matches!(self.operations.get(empty_store.bci()), Some(Operation::Store { slot: written }) if *written == slot)
                || !matches!(
                    self.operations.get(transfer.bci()),
                    Some(Operation::Transfer)
                )
                || !matches!(transfer.opcode(), 0xa7 | 0xc8)
                || empty_store.writes() != [(Slot::Local(slot), value)]
                || empty_store.reads() != push.writes()
                || self.ssa.value(value).uses().len() != 1
                || self.ssa.value(value).uses()[0].block() != &join_id
                || self.ssa.value(value).uses()[0].bci().is_some()
            {
                return Ok(None);
            }
            Some(value)
        } else {
            None
        };
        let Some(body_store) = body_names.instructions().iter().find(|instruction| {
            matches!(self.operations.get(instruction.bci()), Some(Operation::Store { slot: written }) if *written == slot)
        }) else {
            return Ok(None);
        };
        let Some((Slot::Local(_), body_value)) = body_store.writes().first().copied() else {
            return Ok(None);
        };
        if !matches!(self.operations.get(store.bci()), Some(Operation::Store { slot: written }) if *written == slot)
            || !matches!(
                self.operations.get(transfer.bci()),
                Some(Operation::Transfer)
            )
            || !matches!(
                self.operations.get(bridge_transfer.bci()),
                Some(Operation::Transfer)
            )
            || !matches!(
                self.operations.get(increment.bci()),
                Some(Operation::Increment { .. })
            )
            || !matches!(
                self.operations.get(latch_transfer.bci()),
                Some(Operation::Transfer)
            )
            || !matches!(transfer.opcode(), 0xa7 | 0xc8)
            || !matches!(bridge_transfer.opcode(), 0xa7 | 0xc8)
            || !matches!(latch_transfer.opcode(), 0xa7 | 0xc8)
            || body_names
                .instructions()
                .last()
                .is_none_or(|instruction| instruction.bci() != body_test)
            || body_names
                .instructions()
                .iter()
                .filter(|instruction| {
                    matches!(
                        self.operations.get(instruction.bci()),
                        Some(Operation::Store { .. })
                    )
                })
                .count()
                != 1
            || (body_names.instructions().iter().any(|instruction| {
                matches!(
                    self.operations.get(instruction.bci()),
                    Some(Operation::Invoke(_) | Operation::InvokeDynamic(_))
                )
            }) && !constructor_exit)
            || store.writes() != [(Slot::Local(slot), effect_value)]
            || body_store.writes() != [(Slot::Local(slot), body_value)]
            || self.ssa.value(effect_value).uses().len() != 1
            || self.ssa.value(effect_value).uses()[0].block() != &join_id
            || self.ssa.value(effect_value).uses()[0].bci().is_some()
        {
            return Ok(None);
        }
        if constructor_exit {
            let [
                array,
                index,
                element,
                saved,
                reload,
                name,
                literal,
                equals,
                test,
            ] = body_names.instructions()
            else {
                return Ok(None);
            };
            let (
                [(Slot::Stack(0), array_value)],
                [(Slot::Stack(1), index_value)],
                [(Slot::Stack(0), element_value)],
                [(Slot::Stack(0), reload_value)],
                [(Slot::Stack(0), name_value)],
                [(Slot::Stack(1), literal_value)],
                [(Slot::Stack(0), result_value)],
            ) = (
                array.writes(),
                index.writes(),
                element.writes(),
                reload.writes(),
                name.writes(),
                literal.writes(),
                equals.writes(),
            )
            else {
                return Ok(None);
            };
            if !matches!(
                self.operations.get(array.bci()),
                Some(Operation::Load { slot: 1 })
            ) || !matches!(
                self.operations.get(index.bci()),
                Some(Operation::Load { .. })
            ) || !matches!(
                self.operations.get(element.bci()),
                Some(Operation::ArrayElementLoad { .. })
            ) || saved.bci() != body_store.bci()
                || !matches!(self.operations.get(reload.bci()), Some(Operation::Load { slot: read }) if *read == slot)
                || !matches!(self.operations.get(name.bci()), Some(Operation::Invoke(target))
                    if target.kind() == crate::facts::InvokeKind::Virtual
                        && target.owner() == "java/io/File" && target.name() == "getName"
                        && target.descriptor() == "()Ljava/lang/String;")
                || !matches!(
                    self.operations.get(literal.bci()),
                    Some(Operation::Push(ConstantValue::String(_)))
                )
                || !matches!(self.operations.get(equals.bci()), Some(Operation::Invoke(target))
                    if target.kind() == crate::facts::InvokeKind::Virtual
                        && target.owner() == "java/lang/String" && target.name() == "equals"
                        && target.descriptor() == "(Ljava/lang/Object;)Z")
                || test.bci() != body_test
                || array.opcode() != 0x2b
                || index.opcode() != 0x15
                || element.opcode() != 0x32
                || reload.opcode() != 0x2c
                || name.opcode() != 0xb6
                || !matches!(literal.opcode(), 0x12 | 0x13)
                || equals.opcode() != 0xb6
                || test.opcode() != 0x99
                || element.reads()
                    != [
                        (Slot::Stack(1), *index_value),
                        (Slot::Stack(0), *array_value),
                    ]
                || saved.reads() != [(Slot::Stack(0), *element_value)]
                || reload.reads() != [(Slot::Local(slot), body_value)]
                || name.reads() != [(Slot::Stack(0), *reload_value)]
                || equals.reads()
                    != [
                        (Slot::Stack(1), *literal_value),
                        (Slot::Stack(0), *name_value),
                    ]
                || test.reads() != [(Slot::Stack(0), *result_value)]
                || [
                    (*array_value, element.bci()),
                    (*index_value, element.bci()),
                    (*element_value, saved.bci()),
                    (*reload_value, name.bci()),
                    (*name_value, equals.bci()),
                    (*literal_value, equals.bci()),
                    (*result_value, test.bci()),
                ]
                .iter()
                .any(|(value, bci)| !single_use_at_block(self.ssa, *value, &body_id, *bci))
            {
                return Ok(None);
            }
        }
        let body_value_uses = self.ssa.value(body_value).uses();
        if body_value_uses.len() != 2
            || body_value_uses.iter().filter(|usage| usage.block() == &join_id && usage.bci().is_none()).count() != 1
            || body_value_uses.iter().filter(|usage| {
                usage.block() == &body_id && usage.bci().is_some_and(|bci| {
                    body_names.instructions().iter().any(|instruction| {
                        instruction.bci() == bci
                            && matches!(self.operations.get(bci), Some(Operation::Load { slot: read }) if *read == slot)
                            && instruction.reads().contains(&(Slot::Local(slot), body_value))
                    })
                })
            }).count() != 1
        {
            return Ok(None);
        }
        let phis: Vec<_> = self
            .ssa
            .phis()
            .iter()
            .filter(|phi| phi.block() == &join_id && phi.slot() == Slot::Local(slot))
            .collect();
        let [phi] = phis.as_slice() else {
            return Ok(None);
        };
        if phi.inputs().len() != 2 + usize::from(empty_value.is_some())
            || !phi.inputs().contains(&PhiInput::Value(effect_value))
            || !phi.inputs().contains(&PhiInput::Value(body_value))
            || empty_value.is_some_and(|value| !phi.inputs().contains(&PhiInput::Value(value)))
            || self.ssa.value(phi.value()).replaced_by().is_some()
            || join_names
                .entry()
                .iter()
                .filter(|(at, _)| *at == Slot::Local(slot))
                .count()
                != 1
            || !join_names
                .entry()
                .contains(&(Slot::Local(slot), phi.value()))
        {
            return Ok(None);
        }
        let [use_] = self.ssa.value(phi.value()).uses() else {
            return Ok(None);
        };
        let join_value_closed = if constructor_exit {
            let Some((_, _, _, Some(outer))) = nested_arm else {
                return Ok(None);
            };
            let Some(outer_id) = self.view.id_of(outer) else {
                return Ok(None);
            };
            let [join_transfer] = join_names.instructions() else {
                return Ok(None);
            };
            let outer_inputs = incoming(outer_id);
            let [(_, first), (_, second)] = outer_inputs.as_slice() else {
                return Ok(None);
            };
            let empty_outer = if first == &join_id {
                second
            } else if second == &join_id {
                first
            } else {
                return Ok(None);
            };
            let (Some(empty_names), Some(outer_names)) =
                (self.ssa.block(empty_outer), self.ssa.block(outer_id))
            else {
                return Ok(None);
            };
            let [null, null_store] = empty_names.instructions() else {
                return Ok(None);
            };
            let [(Slot::Local(written), outer_null)] = null_store.writes() else {
                return Ok(None);
            };
            let outer_phis: Vec<_> = self
                .ssa
                .phis()
                .iter()
                .filter(|candidate| {
                    candidate.block() == outer_id && candidate.slot() == Slot::Local(slot)
                })
                .collect();
            let [outer_phi] = outer_phis.as_slice() else {
                return Ok(None);
            };
            let [check_load, check] = outer_names.instructions() else {
                return Ok(None);
            };
            let Some([call_node, return_node]) = (|| {
                let successors = self.view.successors(outer);
                let [first, second] = successors.as_slice() else {
                    return None;
                };
                let (call, returned) = if self.view.successors(*first) == [*second] {
                    (*first, *second)
                } else if self.view.successors(*second) == [*first] {
                    (*second, *first)
                } else {
                    return None;
                };
                Some([call, returned])
            })() else {
                return Ok(None);
            };
            let (Some(call_id), Some(return_id)) =
                (self.view.id_of(call_node), self.view.id_of(return_node))
            else {
                return Ok(None);
            };
            let (Some(call_names), Some(return_names)) =
                (self.ssa.block(call_id), self.ssa.block(return_id))
            else {
                return Ok(None);
            };
            let [call_load, delete] = call_names.instructions() else {
                return Ok(None);
            };
            let [return_load, returned] = return_names.instructions() else {
                return Ok(None);
            };
            let outer_value = outer_phi.value();
            let direct_uses: BTreeSet<_> = self
                .ssa
                .value(outer_value)
                .uses()
                .iter()
                .filter_map(|usage| usage.bci())
                .collect();
            join_transfer.opcode() == 0xa7
                && matches!(
                    self.operations.get(join_transfer.bci()),
                    Some(Operation::Transfer)
                )
                && join_transfer.reads().is_empty()
                && join_transfer.writes().is_empty()
                && self.view.successors(join) == [outer]
                && outer_inputs
                    .iter()
                    .all(|(kind, _)| *kind == CanonicalEdgeKind::Normal)
                && *written == slot
                && matches!(
                    self.operations.get(null.bci()),
                    Some(Operation::Push(ConstantValue::Null))
                )
                && matches!(self.operations.get(null_store.bci()), Some(Operation::Store { slot: written }) if *written == slot)
                && null_store.reads() == null.writes()
                && outer_phi.inputs().len() == 2
                && outer_phi.inputs().contains(&PhiInput::Value(phi.value()))
                && outer_phi.inputs().contains(&PhiInput::Value(*outer_null))
                && self.ssa.value(outer_value).replaced_by().is_none()
                && outer_names
                    .entry()
                    .contains(&(Slot::Local(slot), outer_value))
                && self.ssa.value(*outer_null).uses().len() == 1
                && self.ssa.value(*outer_null).uses()[0].block() == outer_id
                && self.ssa.value(*outer_null).uses()[0].bci().is_none()
                && use_.block() == outer_id
                && use_.bci().is_none()
                && matches!(self.operations.get(check_load.bci()), Some(Operation::Load { slot: read }) if *read == slot)
                && matches!(
                    self.operations.get(check.bci()),
                    Some(Operation::Comparison { .. })
                )
                && check_load.reads() == [(Slot::Local(slot), outer_value)]
                && check.reads() == check_load.writes()
                && matches!(self.operations.get(call_load.bci()), Some(Operation::Load { slot: read }) if *read == slot)
                && call_load.reads() == [(Slot::Local(slot), outer_value)]
                && matches!(self.operations.get(delete.bci()), Some(Operation::Invoke(target))
                    if target.kind() == crate::facts::InvokeKind::Virtual
                        && target.owner() == "java/io/File" && target.name() == "deleteOnExit"
                        && target.descriptor() == "()V")
                && delete.reads() == call_load.writes()
                && delete.writes().is_empty()
                && matches!(self.operations.get(return_load.bci()), Some(Operation::Load { slot: read }) if *read == slot)
                && return_load.reads() == [(Slot::Local(slot), outer_value)]
                && matches!(self.operations.get(returned.bci()), Some(Operation::Return))
                && returned.reads() == return_load.writes()
                && direct_uses
                    == BTreeSet::from([check_load.bci(), call_load.bci(), return_load.bci()])
                && self.ssa.value(outer_value).uses().len() == 5
                && self
                    .ssa
                    .value(outer_value)
                    .uses()
                    .iter()
                    .filter(|usage| usage.bci().is_none() && usage.block() == return_id)
                    .count()
                    == 2
        } else {
            use_.block() == &join_id
                && join_names.instructions().iter().any(|instruction| {
                    use_.bci() == Some(instruction.bci())
                        && instruction.reads().contains(&(Slot::Local(slot), phi.value()))
                        && matches!(self.operations.get(instruction.bci()), Some(Operation::Load { slot: read }) if *read == slot)
                })
        };
        if !join_value_closed || !self.loop_exit_bridge(body, bridge, &join_id, blocks) {
            return Ok(None);
        }
        let mut scope = blocks.clone();
        scope.extend([effect, bridge]);
        let mut body_frame = frame.loop_body(
            &scope,
            join,
            header_node,
            header_node,
            Some(join),
            Some(join),
            BTreeSet::from([join]),
            &BTreeSet::new(),
            &BTreeSet::new(),
        );
        self.include_fragmented_catch_scope(&mut body_frame, header_node);
        body_frame.allow_own_loop_entry = true;
        let (body_regions, next) = self.loop_body_sequence(header, &body_frame, &scope)?;
        let owners: Vec<_> = body_regions
            .iter()
            .flat_map(Region::blocks)
            .cloned()
            .collect();
        let owned_nodes: BTreeSet<_> = owners
            .iter()
            .filter_map(|id| self.view.index_of(id))
            .collect();
        let exits_through = |arm: &Region, block: &CanonicalBlockId, at: u32| {
            matches!(arm, Region::Sequence { regions }
                if matches!(regions.as_slice(),
                    [Region::Straight { blocks }, Region::LoopBreak { source_bci, loop_header }]
                    if blocks == &[block.clone()] && *source_bci == at && loop_header == header))
        };
        let body_shape = if let [
            Region::If {
                branch,
                branch_bci,
                then_arm,
                else_arm,
                ..
            },
        ] = body_regions.as_slice()
        {
            let arms = [then_arm.as_ref(), else_arm.as_ref()];
            branch == header
                && *branch_bci == header_test
                && arms.iter().any(|arm| exits_through(arm, &effect_id, transfer.bci()))
                && arms.iter().any(|arm| {
                    if let Region::If {
                        branch,
                        branch_bci,
                        then_arm,
                        else_arm,
                        ..
                    } = arm
                    {
                        let inner = [then_arm.as_ref(), else_arm.as_ref()];
                        branch == &body_id
                            && *branch_bci == body_test
                            && inner.iter().any(|arm| {
                                exits_through(arm, &bridge_id, bridge_transfer.bci())
                            })
                            && inner.iter().any(|arm| {
                                matches!(arm, Region::Straight { blocks } if blocks == &[latch_id.clone()])
                            })
                    } else {
                        false
                    }
                })
        } else {
            false
        };
        if !body_shape
            || next.as_ref() != Some(&join_id)
            || owners.len() != scope.len()
            || owned_nodes != scope
            || !body_regions.iter().all(Region::is_structured)
        {
            return Ok(Some(Self::loop_fallback(
                header,
                FallbackReason::LoopShape {
                    block_bci: header.bci(),
                },
                body_regions,
            )));
        }
        Ok(Some((
            vec![Region::Loop {
                header: header.clone(),
                tests: Vec::new(),
                test_operator: None,
                form: LoopForm::Endless,
                for_header: None,
                body: body_regions,
                exit: Some(join_id.clone()),
                gateway_origins: vec![latch_transfer.bci()],
            }],
            Some(join_id),
        )))
    }

    /// The hidden transfer a final straight run may assign to a header-tested loop: a `while`
    /// latch, or the exact update block already proved by `ForHeader`, whose terminal instruction
    /// transfers back to this header and whose canonical block has no other outgoing edge.
    fn implicit_tail_latch_origin(
        &mut self,
        header_node: usize,
        body: &[Region],
        for_header: Option<&ForHeader>,
    ) -> Result<Option<u32>, StopReason> {
        let Some(Region::Straight { blocks: tail }) = body.last() else {
            return Ok(None);
        };
        let Some(latch_id) = tail.last() else {
            return Ok(None);
        };
        let Some(latch_node) = self.view.index_of(latch_id) else {
            return Ok(None);
        };
        let Some(natural_loop) = self.view.loop_entered_at(header_node) else {
            return Ok(None);
        };
        let Some(header_id) = self.view.id_of(header_node) else {
            return Ok(None);
        };
        let Some(latch_bci) = self.terminal_bci(latch_id) else {
            return Ok(None);
        };
        poll(self.budget, Some(latch_bci))?;
        if let Some(proof) = for_header {
            let Some(instructions) = self.ssa.block(latch_id).map(|block| block.instructions())
            else {
                return Ok(None);
            };
            let Some([update, transfer]) = instructions.get(instructions.len().saturating_sub(2)..)
            else {
                return Ok(None);
            };
            let update_matches_slot = match self.operations.get(proof.update_bci) {
                Some(Operation::Increment { slot, .. }) => *slot == proof.slot,
                Some(Operation::Store { slot }) => *slot == proof.slot,
                _ => false,
            };
            if &proof.update_block != latch_id
                || update.bci() != proof.update_bci
                || transfer.bci() != latch_bci
                || !update_matches_slot
            {
                return Ok(None);
            }
        }
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(natural_loop.latches().len()).unwrap_or(u64::MAX),
            Some(latch_bci),
        )?;
        if natural_loop.header() != header_node
            || natural_loop.is_irreducible()
            || natural_loop.latches().len() != 1
            || !natural_loop.latches().contains(&latch_node)
        {
            return Ok(None);
        }
        if !self.ssa.block(latch_id).is_some_and(|block| {
            block.instructions().last().is_some_and(|instruction| {
                instruction.bci() == latch_bci
                    && matches!(instruction.opcode(), 0xa7 | 0xc8)
                    && matches!(self.operations.get(latch_bci), Some(Operation::Transfer))
            })
        }) || self.view.successors(latch_node) != [header_node]
        {
            return Ok(None);
        }
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(self.canonical.edges().len()).unwrap_or(u64::MAX),
            Some(latch_bci),
        )?;
        let mut outgoing = 0usize;
        for edge in self.canonical.edges() {
            poll(self.budget, Some(latch_bci))?;
            if edge.from() != latch_id {
                continue;
            }
            outgoing += 1;
            if edge.kind() != CanonicalEdgeKind::Normal || edge.to() != header_id {
                return Ok(None);
            }
        }
        if outgoing != 1 {
            return Ok(None);
        }
        Ok(Some(latch_bci))
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
                let terminal_returns = self.loop_terminal_returns(blocks, exit_node, frame)?;
                let mut body_frame = frame.loop_body(
                    blocks,
                    header_node,
                    header_node,
                    header_node,
                    exit_node,
                    exit_node,
                    exits,
                    &transfer_sources,
                    &terminal_returns,
                );
                self.include_fragmented_catch_scope(&mut body_frame, header_node);
                let (body, _) = self.loop_body_sequence(&chain.body, &body_frame, blocks)?;
                self.visited.extend(test_nodes.iter().copied());
                let mut expected = blocks.clone();
                for node in test_nodes {
                    expected.remove(&node);
                }
                expected.extend(terminal_returns);
                if !self.covers(&expected) {
                    return Ok(Some(Self::loop_fallback(
                        header,
                        FallbackReason::LoopShape {
                            block_bci: header.bci(),
                        },
                        body,
                    )));
                }
                let gateway_origins = self
                    .implicit_tail_latch_origin(header_node, &body, None)?
                    .into_iter()
                    .collect();
                return Ok(Some((
                    vec![Region::Loop {
                        header: header.clone(),
                        tests: chain.tests,
                        test_operator: Some(chain.operator),
                        form: LoopForm::While,
                        for_header: None,
                        body,
                        exit: Some(chain.exit.clone()),
                        gateway_origins,
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
            && frame
                .own_finally
                .is_none_or(|rows| !self.finally_edges_accounted(header, rows))
            && frame
                .segmented_finally_rows
                .is_none_or(|rows| !self.segmented_finally_edges_accounted(header, rows))
            && frame
                .multi_return_finally_rows
                .is_none_or(|rows| !self.multi_return_finally_edges_accounted(header, rows))
        {
            return Ok(Some(gap(Vec::new(), vec![header.clone()], reason, None)));
        }
        if let Err(reason) = self.test_is_pure(header, test_bci, true) {
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
        let exit_gateway = self.loop_exit_gateway_pair(header_node, exit_node, blocks, frame)?;
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
        let mut transfer_sources = self.loop_transfer_sources(&all_exits);
        if let Some((gateway, _, _)) = exit_gateway {
            transfer_sources.insert(gateway);
        }
        let terminal_returns = self.loop_terminal_returns(blocks, exit_node, frame)?;
        let mut body_frame = frame.loop_body(
            blocks,
            header_node,
            header_node,
            for_header
                .as_ref()
                .and_then(|proof| self.view.index_of(&proof.update_block))
                .unwrap_or(header_node),
            exit_node,
            exit_gateway.map(|(_, target, _)| target).or(exit_node),
            exits,
            &transfer_sources,
            &terminal_returns,
        );
        if matches!(self.handlers.len(), 3 | 4)
            && frame.own_finally.is_some()
            && frame.own_try.is_some()
        {
            body_frame.own_finally = frame.own_finally;
            body_frame.own_try = frame.own_try;
            body_frame.segmented_finally_rows = frame.segmented_finally_rows;
            body_frame.multi_return_finally_rows = frame.multi_return_finally_rows;
        }
        // The proved two-row void finally keeps its own row for the same reason: a loop inside the
        // protected body leaves through the certificate's handler, and the row is what accounts
        // for that edge. The flag is set by the claim alone, so no other two-row shape widens.
        if self.handlers.len() == 2 && frame.void_loop_finally {
            body_frame.own_finally = frame.own_finally;
            body_frame.own_try = frame.own_try;
        }
        // The proved lock guard's and resource guard's protected bodies are ordinary code, so their
        // loops are walked the same way and leave through the same certificate handler: the rows
        // the claim proved are what account for those edges. Like the flag above, this one is set by
        // the claim alone.
        if frame.lock_guard_finally && frame.own_finally.is_some() && frame.own_try.is_some() {
            body_frame.own_finally = frame.own_finally;
            body_frame.own_try = frame.own_try;
        }
        self.include_fragmented_catch_scope(&mut body_frame, header_node);
        let (body, _) = self.loop_body_sequence(&inside, &body_frame, blocks)?;
        self.visited.insert(header_node);
        let mut expected = blocks.clone();
        expected.remove(&header_node);
        if let Some((gateway, _, _)) = exit_gateway {
            expected.insert(gateway);
        }
        expected.extend(terminal_returns);
        if !self.covers(&expected) {
            // The loop's shape is refused, and every block the body's walk claimed goes into the
            // refusal with it: the body region is dropped here, so naming its blocks is the only
            // thing that keeps them in the artifact at all (see [`Self::loop_fallback`]).
            let reason = FallbackReason::LoopShape {
                block_bci: header.bci(),
            };
            return Ok(Some(Self::loop_fallback(header, reason, body)));
        }
        let mut gateway_origins = exit_gateway
            .map(|(_, _, origins)| origins.to_vec())
            .unwrap_or_default();
        if let Some(proof) = self.fragmented.as_ref()
            && proof.loop_header == header_node
            && for_header
                .as_ref()
                .is_some_and(|candidate| candidate.update_block == proof.update)
        {
            gateway_origins.push(proof.update_transfer_bci);
        }
        if let Some(latch_bci) =
            self.implicit_tail_latch_origin(header_node, &body, for_header.as_ref())?
            && !gateway_origins.contains(&latch_bci)
        {
            gateway_origins.push(latch_bci);
        }
        let run = vec![Region::Loop {
            header: header.clone(),
            tests: vec![(header.clone(), test_bci, continuation)],
            test_operator: None,
            form: LoopForm::While,
            for_header,
            body,
            exit: Some(outside.clone()),
            gateway_origins,
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
        // The MVP admits **one** postfix condition position per chain, at one of its ends
        // (`recover-postfix-condition-positions`): the middle of a short-circuit chain and a second
        // variable's position are recorded and left to a later slice, so a chain that would present
        // them keeps the refusal it had.
        let positions = self.chain_snapshot_positions(&tests)?;
        if let Some(at) = chain_position_outside_bound(&positions, tests.len()) {
            return Ok(Err(FallbackReason::ChainPositionBound {
                block_bci: header.bci(),
                at,
            }));
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
        if let Err(reason) = self.test_is_pure(block, test_bci, true) {
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
            if in_edges.is_empty()
                || in_edges.iter().any(|source| {
                    !blocks.contains(source)
                        && !self.fragmented.as_ref().is_some_and(|proof| {
                            proof.loop_header == header_node
                                && proof.loop_entries.contains(&(*source, latch))
                        })
                })
            {
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

    /// The run a refused loop leaves behind: the quotes the body's walk left behind first (they
    /// are the walk's own first failures), then one refusal naming the loop's own block and every
    /// block nothing earlier claimed.
    ///
    /// A refused loop cannot be written — the statement is the loop — so the blocks its body proved
    /// have no place as statements, and the body's region is **dropped** by the caller. A dropped
    /// region is exactly what the exactly-once invariant forbids: those blocks were claimed, so the
    /// uncovered-blocks scan will not name them, and a block named by no region is a silently lost
    /// part of the body. The refusal therefore quotes the header and every body block still
    /// unclaimed, each once, in the walk's own order; the quotes the body left behind stay quotes
    /// and are reported beside it ([`Run`]) — before the refusal, because the body's quote is the
    /// failure the loop's refusal is about, and no block may be claimed by both. A refusal whose
    /// every block an earlier quote already holds states nothing of its own and is not emitted.
    fn loop_fallback(header: &CanonicalBlockId, reason: FallbackReason, body: Vec<Region>) -> Run {
        let mut run: Vec<Region> = body
            .iter()
            .filter(|region| matches!(region, Region::Fallback { .. }))
            .cloned()
            .collect();
        let claimed: BTreeSet<&CanonicalBlockId> = run.iter().flat_map(Region::blocks).collect();
        let blocks: Vec<CanonicalBlockId> =
            gap_blocks(header, body.iter().flat_map(Region::blocks).cloned())
                .into_iter()
                .filter(|block| !claimed.contains(block))
                .collect();
        if !blocks.is_empty() {
            run.push(Region::Fallback { blocks, reason });
        }
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
        // The MVP admits **one** postfix condition position per chain, at one of its ends
        // (`recover-postfix-condition-positions`): the middle of a short-circuit chain and a second
        // variable's position are recorded and left to a later slice, so a chain that would present
        // them keeps the refusal it had.
        let positions = self.chain_snapshot_positions(&tests)?;
        if let Some(at) = chain_position_outside_bound(&positions, tests.len()) {
            let reason = FallbackReason::ChainPositionBound {
                block_bci: header.bci(),
                at,
            };
            return Ok(Some(gap(Vec::new(), vec![header.clone()], reason, None)));
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
            gateway_origins: Vec::new(),
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

    /// Whether the first test's suffix of one bottom-tested chain holds nothing but the values
    /// that test reads.
    ///
    /// The block is written once as the loop's body, and the build writes the instructions before
    /// the condition's first value as the body's own statements (`recover-postfix-condition-
    /// positions`: `last = xs[i];` shares the block with the `xs[i++] != 0` the do-while's
    /// condition starts at). Everything from that first value on belongs to the condition, and the
    /// per-instruction rule is the one `test_is_pure` states — a value the test writes has exactly
    /// one place where it is admitted — so an effect the test would have to run has no place there
    /// and the chain is declined.
    fn latch_test_suffix_is_effect_free(&self, block: &CanonicalBlockId, test_bci: u32) -> bool {
        let Some((_, first_condition, _)) = self.first_latch_test_suffix(block) else {
            return false;
        };
        let Some(names) = self.ssa.block(block) else {
            return false;
        };
        // The condition's own values are the *unfiltered* set the branch reads through the block's
        // producers; the per-instruction rule is the same one `test_is_pure` states, so a value the
        // test writes has one place where it is admitted (`recover-postfix-condition-positions`
        // admits the array read and the `iload; iinc` pair here too).
        let condition_bcis = self.condition_value_bcis(block, test_bci, names);
        let reads_per_value = block_reads_per_value(names);
        if names.instructions()[first_condition..]
            .iter()
            .any(|instruction| {
                instruction.bci() != test_bci
                    && !self.test_expression_instruction(
                        block,
                        test_bci,
                        instruction,
                        &condition_bcis,
                        &reads_per_value,
                        true,
                    )
            })
        {
            return false;
        }
        // Everything before the condition's first value is the body's own lead: the block is the
        // loop's body written once, and the build writes those instructions as the statements they
        // are (`recover-postfix-condition-positions`: `last = xs[i];` shares the block with the
        // `xs[i++] != 0` the do-while's condition starts at).
        true
    }

    /// The `do … while` shape: one latch, whose own branch tests and jumps back to the header.
    fn latch_tested_loop(
        &mut self,
        header: &CanonicalBlockId,
        header_node: usize,
        blocks: &BTreeSet<usize>,
        frame: &Frame,
        allow_header_entry: bool,
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
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(self.canonical.edges().len().saturating_mul(blocks.len()))
                .unwrap_or(u64::MAX),
            Some(header.bci()),
        )?;
        // A comparison in the header is a body `if` when its paths remain in this iteration
        // or one path uses an owned, effect-free bridge to this latch's exact exit. Its position
        // alone does not make it a second loop test.
        if latch_node != header_node
            && let Some(header_bci) = self.terminal_bci(header)
            && self
                .operations
                .get(header_bci)
                .is_some_and(|operation| operation.switch().is_some())
        {
            return Ok(None);
        }
        if latch_node != header_node
            && let Some(header_bci) = self.terminal_bci(header)
            && self
                .operations
                .get(header_bci)
                .is_some_and(|operation| operation.comparison().is_some())
            && self.view.successors(header_node).iter().any(|successor| {
                !blocks.contains(successor)
                    && !self.loop_exit_bridge(header_node, *successor, &exit, blocks)
            })
        {
            return Ok(None);
        }
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
                    gateway_origins: Vec::new(),
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
        let latch_body_prefix = if let Err(reason) = self.test_is_pure(&latch, test_bci, false) {
            let Some((_, first_condition, condition_bcis)) = self.first_latch_test_suffix(&latch)
            else {
                return Ok(Some(gap(Vec::new(), vec![header.clone()], reason, None)));
            };
            let Some(names) = self.ssa.block(&latch) else {
                return Ok(Some(gap(Vec::new(), vec![header.clone()], reason, None)));
            };
            if first_condition == 0
                || names.instructions()[first_condition..]
                    .iter()
                    .any(|instruction| {
                        instruction.bci() != test_bci
                            && !condition_bcis.contains(&instruction.bci())
                    })
            {
                return Ok(Some(gap(Vec::new(), vec![header.clone()], reason, None)));
            }
            true
        } else {
            false
        };
        let continuation = if target == header.bci() {
            Continuation::Taken
        } else {
            Continuation::FallThrough
        };
        let exit_node = self.view.index_of(&exit);
        let exits = self.loop_exit_nodes(blocks);
        if blocks.iter().any(|source| {
            self.view.successors(*source).iter().any(|successor| {
                !blocks.contains(successor)
                    && !(*source == latch_node && Some(*successor) == exit_node)
                    && !self.loop_exit_bridge(*source, *successor, &exit, blocks)
            })
        }) {
            return Ok(None);
        }
        if exits.iter().any(|candidate| Some(*candidate) != exit_node)
            && blocks.iter().any(|node| {
                self.view
                    .id_of(*node)
                    .and_then(|block| self.terminal_bci(block))
                    .and_then(|bci| self.operations.get(bci))
                    .is_some_and(|operation| operation.switch().is_some())
            })
        {
            // A switch arm can intercept an unlabelled break. This one-loop proof does not
            // establish the enclosing switch's transfer ownership.
            return Ok(None);
        }
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
        let terminal_returns = self.loop_terminal_returns(blocks, exit_node, frame)?;
        let mut body_frame = frame.loop_body(
            blocks,
            latch_node,
            header_node,
            latch_node,
            exit_node,
            exit_node,
            exits.clone(),
            &transfer_sources,
            &terminal_returns,
        );
        self.include_fragmented_catch_scope(&mut body_frame, header_node);
        body_frame.allow_own_loop_entry = allow_header_entry;
        let (mut body, _) = self.loop_body_sequence(header, &body_frame, blocks)?;
        if latch_body_prefix {
            body.push(Region::Straight {
                blocks: vec![latch.clone()],
            });
        }
        self.visited.insert(latch_node);
        let mut expected = blocks.clone();
        expected.remove(&latch_node);
        expected.extend(terminal_returns);
        expected.extend(
            exits
                .iter()
                .copied()
                .filter(|candidate| Some(*candidate) != exit_node),
        );
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
            gateway_origins: Vec::new(),
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

    /// A body branch exclusively owns a one-instruction normal transfer to the stated destination.
    /// The bridge is outside the natural loop because it never takes the back edge.
    fn loop_exit_bridge(
        &self,
        source: usize,
        bridge: usize,
        exit: &CanonicalBlockId,
        blocks: &BTreeSet<usize>,
    ) -> bool {
        if !blocks.contains(&source)
            || blocks.contains(&bridge)
            || self.view.successors(bridge)
                != self.view.index_of(exit).into_iter().collect::<Vec<_>>()
            || !self.view.successors(source).contains(&bridge)
            || self.view.successors(source).len() != 2
        {
            return false;
        }
        let (Some(id), Some(source_id)) = (self.view.id_of(bridge), self.view.id_of(source)) else {
            return false;
        };
        if !self
            .terminal_bci(source_id)
            .and_then(|bci| self.operations.get(bci))
            .is_some_and(|operation| operation.comparison().is_some())
        {
            return false;
        }
        let incoming: Vec<_> = self
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.to() == id)
            .map(|edge| (edge.kind(), edge.from().clone()))
            .collect();
        let outgoing: Vec<_> = self
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == id)
            .map(|edge| (edge.kind(), edge.to().clone()))
            .collect();
        if !exact_normal_predecessors(&incoming, &[source_id.clone()])
            || outgoing != [(CanonicalEdgeKind::Normal, exit.clone())]
        {
            return false;
        }
        self.leaving_edge(id).is_none()
            && self.ssa.block(id).is_some_and(|block| {
                matches!(block.instructions(), [instruction]
                    if matches!(instruction.opcode(), 0xa7 | 0xc8)
                        && matches!(self.operations.get(instruction.bci()), Some(Operation::Transfer)))
            })
    }

    /// Whether every route leaving one block ends at a destination `welcome` names — this loop's
    /// own continue target or the break/continue destination of an enclosing one — before any of
    /// them reaches a block with no successors: a `return`'s or an `athrow`'s leaf, a route the
    /// frame's loops do not own. The walk expands each block once, like [`NormalFlowView::reaches`]
    /// whose bounded-work question this answers for loop transfers instead of one join.
    fn loop_side_routes(&self, from: usize, welcome: &BTreeSet<usize>) -> bool {
        let mut seen = BTreeSet::new();
        let mut worklist = vec![from];
        while let Some(current) = worklist.pop() {
            if welcome.contains(&current) {
                continue;
            }
            if !seen.insert(current) {
                continue;
            }
            let successors = self.view.successors(current);
            if successors.is_empty() {
                return false;
            }
            worklist.extend(successors);
        }
        true
    }

    /// Whether one block's continuation inside this loop reaches the header of a **nested** loop
    /// before this loop's own header: the body still holds a region of its own after the block,
    /// and a join placed anywhere but that continuation would leave it unclaimed. The walk stays
    /// inside `blocks` and stops at this loop's header — the latch edge back to it is the loop's
    /// own, not a continuation.
    fn nested_loop_ahead(&self, from: usize, header: usize, blocks: &BTreeSet<usize>) -> bool {
        let mut seen = BTreeSet::new();
        let mut worklist = vec![from];
        while let Some(current) = worklist.pop() {
            if current == header || !seen.insert(current) {
                continue;
            }
            if self.view.is_loop_header(current) {
                return true;
            }
            worklist.extend(
                self.view
                    .successors(current)
                    .into_iter()
                    .filter(|successor| blocks.contains(successor)),
            );
        }
        false
    }

    /// A body branch exclusively owns a one-instruction normal transfer onto **this loop's own
    /// continue target** — the edge a `continue` spells. The mirror of [`Self::loop_exit_bridge`]
    /// for the loop-side target: the transfer block is *inside* the natural loop (it reaches the
    /// latch by taking the update), and its one successor is the update block a proved for-header
    /// owns, or the header of a loop no such proof covers. The same exclusive-ownership checks
    /// apply: exact normal predecessors from the branch alone, one normal successor, no leaving
    /// edge, and a decoded `goto` as the block's only instruction.
    fn loop_continue_bridge(
        &self,
        source: usize,
        bridge: usize,
        continue_target: usize,
        blocks: &BTreeSet<usize>,
    ) -> bool {
        let Some(target_id) = self.view.id_of(continue_target).cloned() else {
            return false;
        };
        if !blocks.contains(&source)
            || !blocks.contains(&bridge)
            || bridge == continue_target
            || self.view.successors(bridge) != [continue_target]
            || !self.view.successors(source).contains(&bridge)
            || self.view.successors(source).len() != 2
        {
            return false;
        }
        let (Some(id), Some(source_id)) = (self.view.id_of(bridge), self.view.id_of(source)) else {
            return false;
        };
        if !self
            .terminal_bci(source_id)
            .and_then(|bci| self.operations.get(bci))
            .is_some_and(|operation| operation.comparison().is_some())
        {
            return false;
        }
        let incoming: Vec<_> = self
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.to() == id)
            .map(|edge| (edge.kind(), edge.from().clone()))
            .collect();
        let outgoing: Vec<_> = self
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == id)
            .map(|edge| (edge.kind(), edge.to().clone()))
            .collect();
        if !exact_normal_predecessors(&incoming, &[source_id.clone()])
            || outgoing != [(CanonicalEdgeKind::Normal, target_id)]
        {
            return false;
        }
        self.leaving_edge(id).is_none()
            && self.ssa.block(id).is_some_and(|block| {
                matches!(block.instructions(), [instruction]
                    if matches!(instruction.opcode(), 0xa7 | 0xc8)
                        && matches!(self.operations.get(instruction.bci()), Some(Operation::Transfer)))
            })
    }

    /// A body branch's successor that is nothing but the `break` edge itself: a block **inside**
    /// this loop whose single decoded `goto` transfers to the break destination of an enclosing
    /// loop, with the branch as its only normal predecessor. Unlike [`Self::loop_exit_bridge`]
    /// the transfer block stays inside the natural loop — a labeled break of an *enclosing*
    /// loop never crosses this loop's header, so the natural-loop walk keeps the block — and
    /// what classifies the edge is the destination it names, not the block's position.
    fn loop_break_transfer(
        &self,
        source: usize,
        exit: usize,
        destination: usize,
        blocks: &BTreeSet<usize>,
    ) -> bool {
        // The transfer block itself may sit **outside** this loop's natural set: a labeled
        // break of an enclosing loop never comes back to this loop's latch, so the
        // natural-loop walk does not hold it. What classifies the edge is the destination
        // it names and its exclusive ownership by the branch — not the block's membership.
        if !blocks.contains(&source)
            || self.view.successors(exit) != [destination]
            || !self.view.successors(source).contains(&exit)
            || self.view.successors(source).len() != 2
        {
            return false;
        }
        let (Some(id), Some(source_id)) = (self.view.id_of(exit), self.view.id_of(source)) else {
            return false;
        };
        if !self
            .terminal_bci(source_id)
            .and_then(|bci| self.operations.get(bci))
            .is_some_and(|operation| operation.comparison().is_some())
        {
            return false;
        }
        let incoming: Vec<_> = self
            .canonical
            .edges()
            .iter()
            .filter(|edge| edge.to() == id)
            .map(|edge| (edge.kind(), edge.from().clone()))
            .collect();
        if !exact_normal_predecessors(&incoming, &[source_id.clone()]) {
            return false;
        }
        self.leaving_edge(id).is_none()
            && self.ssa.block(id).is_some_and(|block| {
                matches!(block.instructions(), [instruction]
                    if matches!(instruction.opcode(), 0xa7 | 0xc8)
                        && matches!(self.operations.get(instruction.bci()), Some(Operation::Transfer)))
            })
    }

    /// The header's physical failure gateway remains the loop's `exit`; a second proved gateway
    /// may use their shared successor as this loop's Java `break` target. Both bridges are checked
    /// against the canonical edges before either is admitted to the body frame.
    fn loop_exit_gateway_pair(
        &mut self,
        header: usize,
        header_exit: Option<usize>,
        blocks: &BTreeSet<usize>,
        frame: &Frame,
    ) -> Result<Option<(usize, usize, [u32; 2])>, StopReason> {
        let Some(header_exit) = header_exit else {
            return Ok(None);
        };
        let successors = self.view.successors(header_exit);
        let [target] = successors.as_slice() else {
            return Ok(None);
        };
        let Some(target_id) = self.view.id_of(*target).cloned() else {
            return Ok(None);
        };
        let Some(header_id) = self.view.id_of(header) else {
            return Ok(None);
        };
        poll(self.budget, Some(header_id.bci()))?;
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(blocks.len()).unwrap_or(u64::MAX),
            Some(header_id.bci()),
        )?;
        let exits = self.loop_exit_nodes(blocks);
        if exits.len() != 2 || !exits.contains(&header_exit) {
            return Ok(None);
        }
        let Some(gateway) = exits.into_iter().find(|node| *node != header_exit) else {
            return Ok(None);
        };
        if frame
            .scope
            .as_ref()
            .is_some_and(|scope| !scope.contains(&gateway))
        {
            return Ok(None);
        }
        let edges = u64::try_from(self.canonical.edges().len())
            .unwrap_or(u64::MAX)
            .saturating_mul(2);
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            edges,
            Some(header_id.bci()),
        )?;
        if !self.loop_exit_bridge(header, header_exit, &target_id, blocks) {
            return Ok(None);
        }
        let Some(loop_of) = self.view.loop_entered_at(header) else {
            return Ok(None);
        };
        let latches: Vec<_> = loop_of.latches().iter().copied().collect();
        let [latch] = latches.as_slice() else {
            return Ok(None);
        };
        let Some(latch_id) = self.view.id_of(*latch) else {
            return Ok(None);
        };
        let Some(latch_bci) = self.terminal_bci(latch_id) else {
            return Ok(None);
        };
        if self.view.successors(*latch) != [header]
            || self.leaving_edge(latch_id).is_some()
            || !self.ssa.block(latch_id).is_some_and(|block| {
                block.instructions().last().is_some_and(|instruction| {
                    instruction.bci() == latch_bci
                        && matches!(instruction.opcode(), 0xa7 | 0xc8)
                        && matches!(self.operations.get(latch_bci), Some(Operation::Transfer))
                })
            })
        {
            return Ok(None);
        }
        let mut owner = None;
        for source in blocks {
            let Some(source_id) = self.view.id_of(*source) else {
                continue;
            };
            poll(self.budget, Some(source_id.bci()))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(source_id.bci()),
            )?;
            if *source == header || !self.view.successors(*source).contains(&gateway) {
                continue;
            }
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                edges,
                Some(source_id.bci()),
            )?;
            if !self.loop_exit_bridge(*source, gateway, &target_id, blocks)
                || owner.replace(*source).is_some()
            {
                return Ok(None);
            }
        }
        Ok(owner.map(|_| {
            (
                gateway,
                *target,
                [
                    self.view.id_of(header_exit).expect("proved gateway").bci(),
                    latch_bci,
                ],
            )
        }))
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

    /// A terminal return leaf can belong to a loop despite having no back edge. Its only entry
    /// must be one comparison in the natural loop, and its only exit must return from this
    /// method (`do { if (…) return …; … } while (false);` compiles to exactly that: the constant
    /// false test leaves no loop, so the body's statement set — the conditional return's leaf
    /// included — is what the region has to own). The leaf's terminal instruction is checked
    /// against the decode before widening the body scope: a `return` edge is a method exit, not
    /// a loop edge, which keeps this classification orthogonal to the `break`/`continue`
    /// transfer proofs.
    fn loop_terminal_returns(
        &mut self,
        blocks: &BTreeSet<usize>,
        normal_exit: Option<usize>,
        frame: &Frame,
    ) -> Result<BTreeSet<usize>, StopReason> {
        let mut leaves = BTreeSet::new();
        for source in blocks {
            let Some(source_id) = self.view.id_of(*source) else {
                continue;
            };
            poll(self.budget, Some(source_id.bci()))?;
            charge(
                self.budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(source_id.bci()),
            )?;
            let successors = self.view.successors(*source);
            if successors.len() != 2
                || !self
                    .terminal_bci(source_id)
                    .and_then(|bci| self.operations.get(bci))
                    .is_some_and(|operation| operation.comparison().is_some())
            {
                continue;
            }
            for candidate in successors {
                if blocks.contains(&candidate)
                    || Some(candidate) == normal_exit
                    || frame
                        .scope
                        .as_ref()
                        .is_some_and(|scope| !scope.contains(&candidate))
                    || !self.view.successors(candidate).is_empty()
                {
                    continue;
                }
                let Some(id) = self.view.id_of(candidate) else {
                    continue;
                };
                charge(
                    self.budget,
                    CountedBudgetDimension::AnalysisSteps,
                    u64::try_from(self.canonical.edges().len())
                        .unwrap_or(u64::MAX)
                        .saturating_mul(2),
                    Some(id.bci()),
                )?;
                let incoming: Vec<_> = self
                    .canonical
                    .edges()
                    .iter()
                    .filter(|edge| edge.to() == id)
                    .map(|edge| (edge.kind(), edge.from().clone()))
                    .collect();
                let outgoing: Vec<_> = self
                    .canonical
                    .edges()
                    .iter()
                    .filter(|edge| edge.from() == id)
                    .map(|edge| edge.kind())
                    .collect();
                if !exact_normal_predecessors(&incoming, &[source_id.clone()])
                    || !outgoing.is_empty()
                {
                    continue;
                }
                let Some(ssa_block) = self.ssa.block(id) else {
                    continue;
                };
                // The leaf's terminal instruction must be a decoded `return` — any of the seven
                // return opcodes, with any value shape the statement layer can present (`return
                // local`, `return "literal"`, `return "a" + f`, …). The CF-07 slice first proved
                // the `iload; ireturn` pair alone; a leaf whose value is computed before it
                // returns is the same edge — the method exit — and its statements are the
                // branch arm's own, so the value's shape is the statement layer's question, not
                // this classification's. A leaf that ends in anything else (an `athrow` site, a
                // transfer) is not this edge: the guard rules and the loop-jump classifications
                // own those.
                if !ssa_block.instructions().last().is_some_and(|returned| {
                    matches!(self.operations.get(returned.bci()), Some(Operation::Return))
                }) {
                    continue;
                }
                leaves.insert(candidate);
            }
        }
        if !leaves.is_empty() {
            Ok(leaves)
        } else {
            Ok(BTreeSet::new())
        }
    }

    /// Whether every expected block of a loop was walked.
    ///
    /// A loop is presented only when the walk reached every block that iterates: a block of the
    /// loop the body never claimed would be a block whose execution the presentation dropped.
    fn covers(&self, expected: &BTreeSet<usize>) -> bool {
        expected.iter().all(|node| self.visited.contains(node))
    }

    /// Find the one join of switch paths that stay in the current loop. An arm may instead end
    /// at an exact, already proved loop break target, or end at the loop's own continue target
    /// while its other route still completes at the shared join (the arm's `if (…) continue;`).
    /// For the current loop's update, a case must end in its own explicit transfer; a normal
    /// path through the shared join is not a continue. When no block inside the loop is met by
    /// two arms' own routes, every arm's meeting point is the continue target itself, and the
    /// answer says so ([`SwitchLoopJoin::ContinueTarget`]). This bounded proof does not claim
    /// blocks or change the normal-flow graph.
    fn switch_loop_join(
        &mut self,
        branch: usize,
        successors: &[CanonicalBlockId],
        frame: &Frame,
        at: u32,
        continue_target: Option<usize>,
    ) -> Result<SwitchLoopJoin, StopReason> {
        let Some(scope) = &frame.scope else {
            return Ok(SwitchLoopJoin::Refused);
        };
        let entries: BTreeSet<usize> = successors
            .iter()
            .filter_map(|id| self.view.index_of(id))
            .collect();
        if entries.len() != successors.len() {
            return Ok(SwitchLoopJoin::Refused);
        }
        let exits: BTreeSet<usize> = frame
            .loop_targets
            .iter()
            .filter_map(|target| target.break_target)
            .collect();
        let mut proved = Vec::new();
        // Whether any block inside the loop is met by two or more arms' own routes. When no such
        // block exists and no local join was proved, every arm's meeting point is the loop's own
        // continue target, and that target is the join (the arms fall out of the switch into it).
        let mut shared_tail = false;
        for candidate in scope {
            if *candidate == branch
                || entries.contains(candidate)
                || continue_target == Some(*candidate)
                || frame.boundary == Some(*candidate)
                || self.view.predecessors(*candidate).len() < 2
                || !self.view.dominates(branch, *candidate)
                || (continue_target.is_some()
                    && !self.forward_join_predecessors(branch, *candidate))
            {
                continue;
            }
            let mut normal_arms = 0;
            let mut continue_arms = 0;
            let mut valid = true;
            for entry in &entries {
                let mut seen = BTreeSet::new();
                let mut work = vec![(*entry, None)];
                let mut reaches_join = false;
                let mut reaches_continue = false;
                let mut arm_valid = true;
                while let Some((current, predecessor)) = work.pop() {
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
                    if continue_target == Some(current) {
                        let explicit_transfer = predecessor.is_some_and(|source| {
                            self.view.successors(source) == [current]
                                && self
                                    .view
                                    .id_of(source)
                                    .and_then(|block| self.terminal_bci(block))
                                    .and_then(|bci| self.operations.get(bci))
                                    .is_some_and(|operation| {
                                        matches!(operation, Operation::Transfer)
                                    })
                        });
                        if !explicit_transfer {
                            arm_valid = false;
                            break;
                        }
                        reaches_continue = true;
                        continue;
                    }
                    if exits.contains(&current) {
                        continue;
                    }
                    if !scope.contains(&current)
                        || frame.boundary == Some(current)
                        || (current != *entry && entries.contains(&current))
                        || (continue_target.is_some()
                            && self
                                .view
                                .id_of(current)
                                .and_then(|block| self.terminal_bci(block))
                                .and_then(|bci| self.operations.get(bci))
                                .is_some_and(|operation| operation.switch().is_some()))
                        || !seen.insert(current)
                    {
                        arm_valid = false;
                        break;
                    }
                    let next = self.view.successors(current);
                    if next.is_empty() {
                        arm_valid = false;
                        break;
                    }
                    work.extend(next.into_iter().map(|successor| (successor, Some(current))));
                }
                // One arm's unclassifiable route (across another case entry, out of the scope)
                // keeps this candidate from being *proved*, but the other arms still say whether
                // the candidate is a real meeting point of the switch — which is what decides
                // between refusing and letting the loop's own continue target be the join.
                if !arm_valid {
                    valid = false;
                    continue;
                }
                normal_arms += usize::from(reaches_join);
                continue_arms += usize::from(reaches_continue);
            }
            shared_tail |= normal_arms >= 2;
            if valid && normal_arms >= 2 && (continue_target.is_none() || continue_arms > 0) {
                proved.push(*candidate);
            }
        }
        Ok(if proved.len() == 1 {
            SwitchLoopJoin::Local(proved.first().copied().expect("one proved candidate"))
        } else if proved.is_empty() && !shared_tail {
            SwitchLoopJoin::ContinueTarget
        } else {
            SwitchLoopJoin::Refused
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
        // loop follows ([`Self::loop_fallback`]). The quotes the arms' own walks left behind come
        // first (they are the walk's own first failures) and own their blocks; the refusal quotes
        // the branch and only the blocks nothing earlier claimed, and says nothing of its own when
        // that set is empty.
        let quoted = |entered: &[Region], reason: FallbackReason| {
            let retained: Vec<Region> = entered
                .iter()
                .filter(|region| matches!(region, Region::Fallback { .. }))
                .cloned()
                .collect();
            let claimed: BTreeSet<&CanonicalBlockId> =
                retained.iter().flat_map(Region::blocks).collect();
            let blocks: Vec<CanonicalBlockId> =
                gap_blocks(branch, entered.iter().flat_map(Region::blocks).cloned())
                    .into_iter()
                    .filter(|block| !claimed.contains(block))
                    .collect();
            let mut run = Vec::new();
            if !prefix.is_empty() {
                run.push(Region::Straight {
                    blocks: prefix.to_vec(),
                });
            }
            run.extend(retained);
            if !blocks.is_empty() {
                run.push(Region::Fallback { blocks, reason });
            }
            (run, None)
        };
        let post_join = self
            .view
            .immediate_post_dominator(node)
            .filter(|join| *join != node);
        let post_is_loop_break = post_join.is_some_and(|join| {
            frame
                .loop_targets
                .iter()
                .any(|target| target.break_target == Some(join))
        });
        let switch_continue = post_join.filter(|join| {
            frame
                .loop_targets
                .last()
                .is_some_and(|target| target.continue_target == *join)
        });
        let post_is_loop_exit = post_is_loop_break || switch_continue.is_some();
        let forward_join = if !post_is_loop_exit && post_join.is_none() {
            self.switch_forward_join(node, successors, frame, branch.bci())?
        } else {
            None
        };
        let join_node = if post_is_loop_exit {
            match self.switch_loop_join(node, successors, frame, branch_bci, switch_continue)? {
                SwitchLoopJoin::Local(local_join) => Some(local_join),
                // Every arm's route ends at the loop's own continue target: the join is that
                // target and the arms fall out of the switch into it. `switch_continue` is
                // `Some` exactly when the post-dominator *is* that target, so a switch whose
                // post-dominator is a loop break still refuses below.
                SwitchLoopJoin::ContinueTarget => switch_continue,
                SwitchLoopJoin::Refused => {
                    // An enclosing loop exit cannot stand in for a switch's local join. Without a
                    // unique in-loop meeting point the arm ownership remains unproved.
                    return Ok(quoted(
                        &[],
                        FallbackReason::SwitchShape {
                            block_bci: branch.bci(),
                        },
                    ));
                }
            }
        } else {
            post_join.or(forward_join)
        };
        if post_is_loop_exit && join_node.is_none() {
            return Ok(quoted(
                &[],
                FallbackReason::SwitchShape {
                    block_bci: branch.bci(),
                },
            ));
        }
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
                frame.switch_arm(join_node, &other_entries, branch_bci, switch_continue)
            } else if join_node.is_some() {
                // The switch's shared join belongs to the continuation after the switch. The
                // ordinary branch boundary is checked only after `visited` is changed, so two arms
                // reaching it would be mistaken for a loop re-entry. Keep other case entries
                // unbounded unless fallthrough was proved: otherwise a cross-case route could be
                // silently emitted as an independent arm with an inserted `break`.
                frame.switch_arm(join_node, &BTreeSet::new(), branch_bci, None)
            } else {
                // With no proven join, preserve the ordinary frame's transfer and ownership rules.
                frame.arm(join_node, Some(branch_bci))
            };
            let (mut arm_run, arm_next) = self.region_at(&start, &case_frame)?;
            if let Some(next) = arm_next.as_ref()
                && matches!(arm_run.as_slice(), [Region::Switch { .. }])
                && self.view.index_of(next) != case_frame.boundary
                && !self.continue_switch_arm(
                    &mut arm_run,
                    next,
                    &start,
                    &case_frame,
                    &case_entries,
                )?
            {
                // A child switch left a continuation that this arm cannot own. Refuse the
                // method rather than publish its first dispatch with the tail elsewhere.
                self.unclosed_tail_at.get_or_insert(branch.bci());
                entered.push(sequence_region(arm_run));
                return Ok(quoted(
                    &entered,
                    FallbackReason::SwitchShape {
                        block_bci: branch.bci(),
                    },
                ));
            }
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

    /// Keep one child switch's proved join inside its enclosing case. This is the single
    /// hash-dispatch → final-dispatch continuation used by nested javac String switches; the
    /// ordinary sequence projection still decides whether the pair is a String switch.
    fn continue_switch_arm(
        &mut self,
        run: &mut Vec<Region>,
        next: &CanonicalBlockId,
        start: &CanonicalBlockId,
        frame: &Frame,
        case_entries: &BTreeSet<usize>,
    ) -> Result<bool, StopReason> {
        let [
            Region::Switch {
                branch,
                join: Some(join),
                ..
            },
        ] = run.as_slice()
        else {
            return Ok(false);
        };
        let (Some(start_node), Some(branch_node), Some(next_node)) = (
            self.view.index_of(start),
            self.view.index_of(branch),
            self.view.index_of(next),
        ) else {
            return Ok(false);
        };
        if join != next
            || next.path() != start.path()
            || next.bci() <= branch.bci()
            || frame.stops_at(next_node)
            || case_entries.contains(&next_node)
            || self.visited.contains(&next_node)
            || self.excluded_edge_nodes.contains(next)
            || !self.view.dominates(start_node, next_node)
            || !self.forward_join_predecessors(branch_node, next_node)
            || !run[0].is_structured()
        {
            return Ok(false);
        }
        let first_owned: BTreeSet<_> = run[0].blocks().into_iter().cloned().collect();
        poll(self.budget, Some(next.bci()))?;
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(self.canonical.edges().len()).unwrap_or(u64::MAX),
            Some(next.bci()),
        )?;
        if self.canonical.edges().iter().any(|edge| {
            edge.to() == next
                && (edge.kind() != CanonicalEdgeKind::Normal || !first_owned.contains(edge.from()))
        }) {
            return Ok(false);
        }

        let before = self.visited.clone();
        let (tail, tail_next) = self.region_at(next, frame)?;
        let [Region::Switch { .. }] = tail.as_slice() else {
            return Ok(false);
        };
        if tail_next.is_some() || !tail[0].is_structured() {
            return Ok(false);
        }
        let tail_owned: BTreeSet<_> = tail[0].blocks().into_iter().cloned().collect();
        let newly_visited: BTreeSet<_> = self
            .visited
            .difference(&before)
            .filter_map(|node| self.view.id_of(*node).cloned())
            .collect();
        if tail_owned != newly_visited
            || tail_owned.is_empty()
            || !tail_owned.is_disjoint(&first_owned)
            || tail_owned.iter().any(|block| {
                block.path() != start.path()
                    || self.excluded_edge_nodes.contains(block)
                    || self.view.index_of(block).is_none_or(|node| {
                        !self.view.dominates(start_node, node)
                            || case_entries.contains(&node)
                            || frame.stops_at(node)
                            || self.view.is_loop_header(node)
                    })
            })
        {
            return Ok(false);
        }
        poll(self.budget, Some(next.bci()))?;
        charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(self.canonical.edges().len()).unwrap_or(u64::MAX),
            Some(next.bci()),
        )?;
        if self.canonical.edges().iter().any(|edge| {
            (tail_owned.contains(edge.to())
                && (edge.kind() != CanonicalEdgeKind::Normal
                    || !(first_owned.contains(edge.from()) || tail_owned.contains(edge.from()))))
                || (tail_owned.contains(edge.from())
                    && edge.kind() == CanonicalEdgeKind::Normal
                    && !tail_owned.contains(edge.to()))
        }) {
            return Ok(false);
        }
        run.extend(tail);
        Ok(true)
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

/// The postfix position of one condition chain that this slice's bound refuses, when the chain
/// holds one: the second position of a chain, or the single position that is neither the chain's
/// first nor its last test (`recover-postfix-condition-positions`). The value answered is the
/// pair's own instruction — the place the refusal names.
fn chain_position_outside_bound(positions: &[(usize, u32)], tests: usize) -> Option<u32> {
    if let Some((_, at)) = positions.get(1) {
        return Some(*at);
    }
    let [(index, at)] = positions else {
        return None;
    };
    (*index != 0 && *index + 1 != tests).then_some(*at)
}

/// The value one instruction reads from one local slot, when it reads that slot.
fn local_read(instruction: &SsaInstruction, slot: u16) -> Option<ValueId> {
    instruction
        .reads()
        .iter()
        .find_map(|(read, value)| match read {
            Slot::Local(read) if *read == slot => Some(*value),
            _ => None,
        })
}

/// The value one instruction writes into one local slot, when it writes that slot.
fn local_write(instruction: &SsaInstruction, slot: u16) -> Option<ValueId> {
    instruction
        .writes()
        .iter()
        .find_map(|(written, value)| match written {
            Slot::Local(written) if *written == slot => Some(*value),
            _ => None,
        })
}

/// The one value one instruction pushes onto the operand stack, when it pushes exactly one.
fn stack_output(instruction: &SsaInstruction) -> Option<ValueId> {
    let mut outputs = instruction
        .writes()
        .iter()
        .filter_map(|(slot, value)| matches!(slot, Slot::Stack(_)).then_some(*value));
    let output = outputs.next()?;
    outputs.next().is_none().then_some(output)
}

/// How many times each value is read by the instructions of one block: the single-reader proof of
/// a throwing condition read (`test_expression_instruction`) is taken over this count.
fn block_reads_per_value(names: &SsaBlock) -> BTreeMap<ValueId, usize> {
    let mut reads_per_value = BTreeMap::new();
    for (_, value) in names
        .instructions()
        .iter()
        .flat_map(|instruction| instruction.reads())
    {
        let uses = reads_per_value.entry(*value).or_insert(0_usize);
        *uses = uses.saturating_add(1);
    }
    reads_per_value
}

/// Both inner arms must actually enter the continuation. Cardinality plus membership alone
/// would let two identical incoming edges stand in for the missing second predecessor.
/// The earliest common node must lead to every other common node. Two incomparable minima do
/// not define one continuation, regardless of their physical address ordering.
fn unique_first_common(
    common: &BTreeSet<usize>,
    reachable: impl Fn(usize) -> BTreeSet<usize>,
) -> Option<usize> {
    let mut first = None;
    for &candidate in common {
        if common.is_subset(&reachable(candidate)) && first.replace(candidate).is_some() {
            return None;
        }
    }
    first
}

fn prefixed_loop_header_is_fresh(header: usize, frame: &Frame, visited: &BTreeSet<usize>) -> bool {
    !visited.contains(&header)
        && !frame.stops_at(header)
        && !frame
            .loop_targets
            .iter()
            .any(|target| target.header == header || target.continue_target == header)
}

fn exact_straight_chain_edges<T, I>(
    blocks: &[T],
    entry_source: &T,
    exit_target: &T,
    edges: I,
    budget: &mut Budget,
    at: Option<u32>,
) -> Result<bool, StopReason>
where
    T: Ord + Clone,
    I: IntoIterator<Item = (CanonicalEdgeKind, T, T)>,
    I::IntoIter: ExactSizeIterator,
{
    if blocks.is_empty() {
        return Ok(false);
    }
    let edges = edges.into_iter();
    let edge_count = edges.len();
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(edge_count.saturating_add(blocks.len())).unwrap_or(u64::MAX),
        at,
    )?;

    let mut owners = BTreeSet::new();
    let mut expected = BTreeSet::new();
    if !expected.insert((entry_source.clone(), blocks[0].clone())) {
        return Ok(false);
    }
    for (index, block) in blocks.iter().enumerate() {
        poll(budget, at)?;
        if !owners.insert(block.clone()) {
            return Ok(false);
        }
        let target = blocks.get(index + 1).unwrap_or(exit_target);
        if !expected.insert((block.clone(), target.clone())) {
            return Ok(false);
        }
    }
    if owners.contains(entry_source) || owners.contains(exit_target) {
        return Ok(false);
    }

    for (kind, from, to) in edges {
        poll(budget, at)?;
        if !owners.contains(&from) && !owners.contains(&to) {
            continue;
        }
        if kind != CanonicalEdgeKind::Normal || !expected.remove(&(from, to)) {
            return Ok(false);
        }
    }
    Ok(expected.is_empty())
}

fn exact_normal_predecessors<T: Ord>(actual: &[(CanonicalEdgeKind, T)], expected: &[T]) -> bool {
    actual.len() == expected.len()
        && expected.iter().collect::<BTreeSet<_>>().len() == expected.len()
        && actual
            .iter()
            .all(|(kind, _)| *kind == CanonicalEdgeKind::Normal)
        && actual.iter().map(|(_, from)| from).collect::<BTreeSet<_>>()
            == expected.iter().collect::<BTreeSet<_>>()
}

fn continuation_claims_are_exact<T: Ord + Clone>(blocks: &[T], visited: &BTreeSet<T>) -> bool {
    let owners: BTreeSet<_> = blocks.iter().cloned().collect();
    owners.len() == blocks.len() && &owners == visited
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_end_transfer_rejects_competing_entries_and_exits() {
        let owned = BTreeSet::from([0_u32, 40, 48, 56]);
        let ordinary = vec![
            (CanonicalEdgeKind::Normal, 0, 61),
            (CanonicalEdgeKind::Normal, 40, 61),
            (CanonicalEdgeKind::Normal, 48, 61),
            (CanonicalEdgeKind::Normal, 56, 61),
            (CanonicalEdgeKind::Normal, 61, 79),
        ];
        assert!(closed_transfer_edges(ordinary.clone(), &61, &79, &owned));
        for extra in [
            (CanonicalEdgeKind::Normal, 90, 61),
            (CanonicalEdgeKind::Exception { handler_ordinal: 1 }, 90, 61),
            (CanonicalEdgeKind::Normal, 61, 90),
            (CanonicalEdgeKind::Normal, 61, 79),
            (CanonicalEdgeKind::Call { call_site: 90 }, 61, 79),
        ] {
            let mut changed = ordinary.clone();
            changed.push(extra);
            assert!(!closed_transfer_edges(changed, &61, &79, &owned));
        }
    }

    #[test]
    fn shared_tail_needs_one_comparable_first_common_block() {
        let joined = BTreeSet::from([24, 32]);
        assert_eq!(
            unique_first_common(&joined, |node| {
                if node == 24 {
                    joined.clone()
                } else {
                    BTreeSet::from([32])
                }
            }),
            Some(24)
        );
        let incomparable = BTreeSet::from([23, 30, 34]);
        assert_eq!(
            unique_first_common(&incomparable, |node| BTreeSet::from([node])),
            None
        );
    }

    #[test]
    fn prefixed_loop_chain_certificate_rejects_extra_entries_exits_and_owners() {
        use jarde_reader::budget::Limits;

        let test_budget = || {
            Budget::new(Limits {
                analysis_steps: 16,
                elapsed_millis: u64::MAX,
                ..Limits::default()
            })
        };
        let chain = [2_u32, 3];
        let valid = vec![
            (CanonicalEdgeKind::Normal, 1, 2),
            (CanonicalEdgeKind::Normal, 2, 3),
            (CanonicalEdgeKind::Normal, 3, 4),
            (CanonicalEdgeKind::Normal, 40, 41),
        ];
        assert!(
            exact_straight_chain_edges(&chain, &1, &4, valid, &mut test_budget(), Some(2),)
                .unwrap()
        );

        assert!(matches!(
            exact_straight_chain_edges(
                &chain,
                &1,
                &4,
                vec![
                    (CanonicalEdgeKind::Normal, 1, 2),
                    (CanonicalEdgeKind::Normal, 2, 3),
                    (CanonicalEdgeKind::Normal, 3, 4)
                ],
                &mut Budget::new(Limits {
                    analysis_steps: 0,
                    elapsed_millis: u64::MAX,
                    ..Limits::default()
                }),
                Some(2),
            ),
            Err(StopReason::Budget {
                dimension: CountedBudgetDimension::AnalysisSteps,
                ..
            })
        ));
        for edges in [
            vec![
                (CanonicalEdgeKind::Normal, 1, 2),
                (CanonicalEdgeKind::Normal, 9, 2),
                (CanonicalEdgeKind::Normal, 2, 3),
                (CanonicalEdgeKind::Normal, 3, 4),
            ],
            vec![
                (CanonicalEdgeKind::Normal, 1, 2),
                (CanonicalEdgeKind::Normal, 2, 3),
                (CanonicalEdgeKind::Normal, 3, 4),
                (CanonicalEdgeKind::Normal, 3, 8),
            ],
            vec![
                (CanonicalEdgeKind::Normal, 1, 2),
                (CanonicalEdgeKind::Exception { handler_ordinal: 0 }, 2, 3),
                (CanonicalEdgeKind::Normal, 3, 4),
            ],
        ] {
            assert!(
                !exact_straight_chain_edges(&chain, &1, &4, edges, &mut test_budget(), Some(2),)
                    .unwrap()
            );
        }
        assert!(
            !exact_straight_chain_edges(
                &[2_u32, 2],
                &1,
                &4,
                vec![
                    (CanonicalEdgeKind::Normal, 1, 2),
                    (CanonicalEdgeKind::Normal, 2, 4),
                ],
                &mut test_budget(),
                Some(2),
            )
            .unwrap()
        );
        assert!(!continuation_claims_are_exact(
            &[2_u32, 3],
            &BTreeSet::from([2, 3, 5]),
        ));
        assert!(!continuation_claims_are_exact(
            &[2_u32, 2],
            &BTreeSet::from([2]),
        ));
    }

    #[test]
    fn prefixed_loop_header_cannot_reopen_claimed_outer_target_or_parent_scope() {
        let header = 7;
        let empty = BTreeSet::new();
        assert!(prefixed_loop_header_is_fresh(
            header,
            &Frame::default(),
            &empty
        ));
        assert!(!prefixed_loop_header_is_fresh(
            header,
            &Frame::default(),
            &BTreeSet::from([header]),
        ));

        let outer_header = Frame {
            loop_targets: vec![LoopTarget {
                header,
                exits: BTreeSet::new(),
                break_target: None,
                continue_target: 2,
            }],
            ..Frame::default()
        };
        assert!(!prefixed_loop_header_is_fresh(
            header,
            &outer_header,
            &empty
        ));
        let outer_continue = Frame {
            loop_targets: vec![LoopTarget {
                header: 2,
                exits: BTreeSet::new(),
                break_target: None,
                continue_target: header,
            }],
            ..Frame::default()
        };
        assert!(!prefixed_loop_header_is_fresh(
            header,
            &outer_continue,
            &empty
        ));

        let parent_scope = Frame {
            scope: Some(BTreeSet::from([3, 5])),
            ..Frame::default()
        };
        assert!(!prefixed_loop_header_is_fresh(
            header,
            &parent_scope,
            &empty
        ));
        let parent_boundary = Frame {
            boundary: Some(header),
            ..Frame::default()
        };
        assert!(!prefixed_loop_header_is_fresh(
            header,
            &parent_boundary,
            &empty
        ));
    }

    #[test]
    fn cf02_tail_requires_both_distinct_normal_predecessors() {
        // CF-02's inner value joins at BCI 28 after producers 23 and 27. A path from
        // the outer early-return join (11) would make that tail ambiguous.
        let expected = [23_u32, 27];
        assert!(exact_normal_predecessors(
            &[
                (CanonicalEdgeKind::Normal, 23),
                (CanonicalEdgeKind::Normal, 27),
            ],
            &expected,
        ));
        assert!(!exact_normal_predecessors(
            &[
                (CanonicalEdgeKind::Normal, 23),
                (CanonicalEdgeKind::Normal, 23),
            ],
            &expected,
        ));
        assert!(!exact_normal_predecessors(
            &[
                (CanonicalEdgeKind::Normal, 23),
                (CanonicalEdgeKind::Normal, 23),
            ],
            &[23, 23],
        ));
        for kind in [
            CanonicalEdgeKind::Call { call_site: 12 },
            CanonicalEdgeKind::Exception { handler_ordinal: 0 },
            CanonicalEdgeKind::Return { call_site: 12 },
        ] {
            assert!(!exact_normal_predecessors(
                &[(CanonicalEdgeKind::Normal, 23), (kind, 27)],
                &expected,
            ));
        }
        assert!(!exact_normal_predecessors(
            &[
                (CanonicalEdgeKind::Normal, 23),
                (CanonicalEdgeKind::Normal, 27),
                (CanonicalEdgeKind::Normal, 11),
            ],
            &expected,
        ));
    }

    #[test]
    fn intermediate_join_requires_one_new_claim_per_tail_block() {
        assert!(continuation_claims_are_exact(
            &[18_u32, 20],
            &BTreeSet::from([18, 20])
        ));
        assert!(!continuation_claims_are_exact(
            &[18_u32, 18],
            &BTreeSet::from([18])
        ));
        assert!(!continuation_claims_are_exact(
            &[18_u32, 20],
            &BTreeSet::from([18])
        ));
        assert!(!continuation_claims_are_exact(
            &[18_u32],
            &BTreeSet::from([18, 20])
        ));
    }

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

    #[test]
    fn diagnose_cf07_last_index_latch_region_ownership_from_real_class_ir() {
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

        const CLASS: &[u8] = include_bytes!(
            "../../../openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/cases/javac23-original/classes/cf07/LoopCases.class"
        );

        fn find_loop(region: &Region, bci: u32) -> Option<&Region> {
            match region {
                Region::Loop { header, .. } if header.bci() == bci => Some(region),
                Region::Loop { body, .. } | Region::Sequence { regions: body } => {
                    body.iter().find_map(|child| find_loop(child, bci))
                }
                Region::If {
                    then_arm, else_arm, ..
                } => find_loop(then_arm, bci).or_else(|| find_loop(else_arm, bci)),
                _ => None,
            }
        }

        fn find_if(region: &Region, bci: u32) -> Option<&Region> {
            match region {
                Region::If { branch_bci, .. } if *branch_bci == bci => Some(region),
                Region::If {
                    then_arm, else_arm, ..
                } => find_if(then_arm, bci).or_else(|| find_if(else_arm, bci)),
                Region::Loop { body, .. } | Region::Sequence { regions: body } => {
                    body.iter().find_map(|child| find_if(child, bci))
                }
                _ => None,
            }
        }

        fn owner_count(region: &Region, block: &CanonicalBlockId) -> usize {
            region
                .blocks()
                .iter()
                .filter(|owner| **owner == block)
                .count()
        }

        fn continue_count(region: &Region, source_bci: u32, header: &CanonicalBlockId) -> usize {
            match region {
                Region::LoopContinue {
                    source_bci: source,
                    loop_header,
                } => {
                    if *source == source_bci && loop_header == header {
                        1
                    } else {
                        0
                    }
                }
                Region::Sequence { regions } | Region::Loop { body: regions, .. } => regions
                    .iter()
                    .map(|region| continue_count(region, source_bci, header))
                    .sum(),
                Region::If {
                    then_arm, else_arm, ..
                } => {
                    continue_count(then_arm, source_bci, header)
                        + continue_count(else_arm, source_bci, header)
                }
                _ => 0,
            }
        }

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
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
            .expect("frozen CF07 class opens");
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(CLASS).to_hex().to_string()),
                length: u64::try_from(CLASS.len()).expect("fixture length fits u64"),
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
        let request = MethodAnalysisRequest {
            environment: ResolutionEnvironment {
                runtime: RuntimeView {
                    physical: PhysicalView {
                        snapshot: snapshot.id().clone(),
                        scope: PhysicalScope::SnapshotAll,
                    },
                    profile: RuntimeProfile {
                        java_release: 23,
                        multi_release: MultiReleasePolicy::Disabled,
                        layout: LayoutMode::Generic,
                    },
                    load_domain: domain.clone(),
                },
                domains: vec![domain],
                providers: Vec::new(),
            },
            method: PhysicalMethodId {
                owner: definition,
                name: JvmBytes(b"lastIndexOf".to_vec()),
                descriptor: JvmBytes(b"([IIII)I".to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        };
        let analysis = analyze_method_ir(&[snapshot], &request, &mut budget)
            .expect("frozen lastIndexOf completes one real JVM analysis");
        let ir = analysis.ir();
        let canonical = ir.canonical().expect("canonical CFG from that analysis");
        let ssa = ir.ssa().expect("SSA from that analysis");
        let code = ir.code().expect("decoded code from that analysis");
        let operations = Operations::of(code, ir.constant_pool());
        let chains = crate::concat::plan_four_conditional_strings(
            ssa,
            canonical,
            &operations,
            &crate::build::FieldCopies::default(),
            &mut budget,
        )
        .expect("the real method's existing concat plan completes");
        let view = crate::normal_flow::NormalFlowView::build(canonical, &mut budget)
            .expect("normal-flow view from that canonical CFG");
        let recovered = recover(
            canonical,
            &view,
            ssa,
            &operations,
            ir.constant_pool(),
            &chains,
            &crate::init::Sites::empty(),
            code,
            Some(false),
            false,
            &crate::pass::JAVA_8,
            &mut budget,
        )
        .expect("the real method region walk completes");

        eprintln!("lastIndexOf recovered regions: {:#?}", recovered.regions);
        let loop_region = recovered
            .regions
            .iter()
            .find_map(|region| find_loop(region, 5))
            .expect("real recovered loop with javap header BCI 5");
        let Region::Loop {
            header,
            body,
            form,
            for_header,
            gateway_origins,
            ..
        } = loop_region
        else {
            unreachable!();
        };
        let inner_if = body
            .iter()
            .find_map(|region| find_if(region, 16))
            .expect("real nested If with javap branch BCI 16");
        let Region::If {
            branch,
            branch_bci,
            then_arm,
            else_arm,
            join,
            ..
        } = inner_if
        else {
            unreachable!();
        };
        let goto_fact = code
            .instructions
            .iter()
            .find(|instruction| instruction.bci == 25)
            .expect("javap's physical latch goto BCI 25 exists");
        assert_eq!(goto_fact.opcode, 0xa7);
        assert_eq!(operations.get(25), Some(&Operation::Transfer));
        let latch_block = canonical
            .blocks()
            .iter()
            .find(|block| {
                block.id().path().is_empty()
                    && block.id().bci() <= goto_fact.bci
                    && goto_fact.bci < block.end_bci()
            })
            .expect("the real canonical span owns physical BCI 25");
        let latch_id = latch_block.id();
        let latch_ssa = ssa.block(latch_id).expect("the real latch has SSA");
        let latch_bcis: Vec<_> = latch_ssa
            .instructions()
            .iter()
            .map(SsaInstruction::bci)
            .collect();
        let latch_edges: Vec<_> = canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == latch_id)
            .map(|edge| (edge.from().bci(), edge.kind(), edge.to().bci()))
            .collect();
        let all_edges: Vec<_> = canonical
            .edges()
            .iter()
            .map(|edge| (edge.from().bci(), edge.kind(), edge.to().bci()))
            .collect();
        let header_node = view.index_of(header).expect("the header is in normal flow");
        let natural = view
            .loop_entered_at(header_node)
            .expect("the actual header owns a natural loop");
        let natural_blocks: Vec<_> = natural
            .blocks()
            .iter()
            .filter_map(|node| view.id_of(*node).cloned())
            .collect();
        let latch_nodes: Vec<_> = natural
            .latches()
            .iter()
            .filter_map(|node| view.id_of(*node).cloned())
            .collect();
        let method_owner_count: usize = recovered
            .regions
            .iter()
            .map(|region| owner_count(region, latch_id))
            .sum();
        let loop_owner_count = owner_count(loop_region, latch_id);
        let if_owner_count = owner_count(inner_if, latch_id);
        let then_owner_count = owner_count(then_arm, latch_id);
        let else_owner_count = owner_count(else_arm, latch_id);
        let physical_continue_count = recovered
            .regions
            .iter()
            .map(|region| continue_count(region, 25, header))
            .sum::<usize>();

        eprintln!(
            "outer_loop header={header:?} form={form:?} for_header={for_header:?} gateway_origins={gateway_origins:?}"
        );
        eprintln!(
            "nested_if branch_block={branch:?} branch_bci={branch_bci} join={join:?} then={then_arm:#?} else={else_arm:#?}"
        );
        eprintln!(
            "latch_physical=(bci={}, opcode=0x{:02x}, width={}) canonical={latch_id:?} span={}..{} ssa_bcis={latch_bcis:?}",
            goto_fact.bci,
            goto_fact.opcode,
            goto_fact.width,
            latch_id.bci(),
            latch_block.end_bci()
        );
        eprintln!(
            "normal_flow_natural_loop header={} blocks={natural_blocks:?} latches={latch_nodes:?} irreducible={}",
            natural.header(),
            natural.is_irreducible()
        );
        eprintln!("latch_canonical_outgoing={latch_edges:?}");
        eprintln!("canonical_all_edges={all_edges:?}");
        eprintln!(
            "latch_block_owner_counts: then={then_owner_count} else={else_owner_count} if={if_owner_count} loop={loop_owner_count} method={method_owner_count} continue_leaf_mentions={physical_continue_count}"
        );

        assert_eq!(header.bci(), 5);
        assert_eq!(*branch_bci, 16);
        assert_eq!(goto_fact.bci, 25);
        assert_eq!(latch_id.bci(), 22);
        assert!(latch_edges.iter().any(|(_, kind, target)| {
            *kind == CanonicalEdgeKind::Normal && *target == header.bci()
        }));
        assert!(
            natural
                .latches()
                .contains(&view.index_of(latch_id).expect("latch index"))
        );
    }
}
