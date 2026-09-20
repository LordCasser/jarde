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
//!    the region is unprovable and the bytecode is quoted instead.
//! 2. **Straight means straight.** A block with no plain successor ends the run; a block with one
//!    continues it; a block with two is an `if`. Three or more successors is a `switch`, which
//!    1.3b recovers and this slice refuses.
//! 3. **The graph is acyclic where it is claimed to be structured.** Reaching a block that is
//!    already part of the recovered structure means a loop or a merge the subset does not model, and
//!    the whole region falls back rather than being printed as a straight line that runs twice.
//! 4. **Both arms meet.** The two successors of a branch must reach one join — the nearest block
//!    every path out of the branch passes through — with the arms not reaching into each other.
//! 5. **The branch's own facts are there.** The branch instruction needs a decoded
//!    [`Operation::Comparison`] and the arity that operation claims; the *polarity* is not guessed
//!    from the shape (an `if (a)` and an `if (!a)` have the same graph), so a branch whose sense is
//!    undecoded is unprovable.
//! 6. **Uncovered blocks are stated, not dropped.** Every live block the walk did not claim is
//!    reported as a fallback region listing exactly those blocks, which is how a body reachable only
//!    through exception or `jsr` edges stays visible instead of turning into an empty body.

use std::collections::BTreeSet;

use jarde_jvm::method_ir::{CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind, SsaTable};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{ExceptionHandlerFact, MethodCodeFacts};

use crate::decode::Operations;
use crate::facts::Operation;
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
            Self::LoopShape { .. } => "jre_region_loop_shape",
            Self::LoopLeavesEarly { .. } => "jre_region_loop_leaves_early",
            Self::Irreducible { .. } => "jre_region_irreducible",
            Self::CrossingExceptionRegions { .. } => "jre_region_crossing_exception_regions",
            Self::UnknownBranchSense { .. } => "jre_region_unknown_branch_sense",
            Self::UnrenderableOperand { .. } => "jre_region_unrenderable_operand",
            Self::ArmsDoNotMeet { .. } => "jre_region_arms_do_not_meet",
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
    /// The arm, which is an empty straight run when the target is the switch's own join — the
    /// `case 0: break;` shape.
    pub arm: Box<Region>,
}

/// What one region is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Region {
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
    /// A prefix ending in a `tableswitch`/`lookupswitch`, with one arm per distinct target.
    ///
    /// `groups` holds the arms in the order the decode first names each target, with the no-match
    /// case stated on the group whose target it shares (or on a group of its own). A target that is
    /// the join is an empty arm, which the emitter writes as a `case` that breaks immediately.
    Switch {
        prefix: Vec<CanonicalBlockId>,
        branch: CanonicalBlockId,
        branch_bci: u32,
        groups: Vec<SwitchGroup>,
        join: Option<CanonicalBlockId>,
    },
    /// A loop the graph proves is natural — one header, one entry, one latch — and whose test is
    /// one branch.
    ///
    /// `test` is the block whose branch decides whether another iteration runs and `test_bci` that
    /// branch: the header for [`LoopForm::While`], the latch for [`LoopForm::DoWhile`]. Keeping the
    /// test *inside* the region is the point: the values it reads and the calls it makes are
    /// written in the loop's condition, so each iteration reads and calls exactly what one
    /// iteration of the bytecode did.
    Loop {
        header: CanonicalBlockId,
        test: CanonicalBlockId,
        test_bci: u32,
        form: LoopForm,
        continuation: Continuation,
        body: Box<Region>,
        exit: Option<CanonicalBlockId>,
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
}

impl Region {
    /// Every block this region claims, in the order the method runs them.
    pub fn blocks(&self) -> Vec<&CanonicalBlockId> {
        match self {
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
            Self::Loop {
                header, test, body, ..
            } => {
                let mut blocks: Vec<&CanonicalBlockId> = vec![header];
                if test != header {
                    blocks.push(test);
                }
                blocks.extend(body.blocks());
                blocks
            }
            Self::Fallback { blocks, .. } => blocks.iter().collect(),
            Self::Guard { prefix, plan } => {
                let mut blocks: Vec<&CanonicalBlockId> = prefix.iter().collect();
                blocks.extend(plan.owned());
                blocks
            }
        }
    }

    /// Whether this region is presented as Java structure rather than as quoted bytecode.
    pub fn is_structured(&self) -> bool {
        match self {
            Self::Straight { .. } => true,
            Self::If {
                then_arm, else_arm, ..
            } => then_arm.is_structured() && else_arm.is_structured(),
            Self::Switch { groups, .. } => groups.iter().all(|group| group.arm.is_structured()),
            Self::Loop { body, .. } => body.is_structured(),
            // A guarded statement is presented when its rule proved it, and every link of that
            // proof is stated by the rule itself: there is no *nested* region inside it that could
            // have been quoted instead, because a body this rule cannot present is refused whole.
            Self::Guard { .. } => true,
            Self::Fallback { .. } => false,
        }
    }

    /// Every fallback this region holds, in method order.
    pub fn fallbacks(&self) -> Vec<FallbackReason> {
        match self {
            Self::Straight { .. } => Vec::new(),
            Self::If {
                then_arm, else_arm, ..
            } => {
                let mut reasons = then_arm.fallbacks();
                reasons.extend(else_arm.fallbacks());
                reasons
            }
            Self::Switch { groups, .. } => groups
                .iter()
                .flat_map(|group| group.arm.fallbacks())
                .collect(),
            Self::Loop { body, .. } => body.fallbacks(),
            Self::Guard { .. } => Vec::new(),
            Self::Fallback { reason, .. } => vec![reason.clone()],
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
            Self::Straight { .. } => Some(crate::pass::STRAIGHT.rule()),
            Self::If { .. } => Some(crate::pass::IF.rule()),
            Self::Switch { .. } => Some(crate::pass::SWITCH.rule()),
            Self::Loop { .. } => Some(crate::pass::LOOP.rule()),
            Self::Guard { plan, .. } => Some(plan.pass().rule()),
            Self::Fallback { reason, .. } => reason.pass().map(|pass| pass.rule()),
        }
    }
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
    code: &MethodCodeFacts,
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
    let irreducible = view.irreducible_blocks();
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
    // P3-R7's own check runs *after* the walk, because what the body has to become depends on what
    // the walk was going to say about it (see the end of this function).
    let mut walker = Walker {
        canonical,
        view,
        ssa,
        operations,
        handlers: &code.exception_handlers,
        profile,
        budget,
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
        let (region, next) = walker.region_at(&node, &Frame::default())?;
        regions.push(region);
        current = next;
    }
    let uncovered: Vec<CanonicalBlockId> = canonical
        .blocks()
        .iter()
        .map(|block| block.id().clone())
        .filter(|id| !canonical.unreachable().contains(id))
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
}

impl Frame {
    /// The frame of one loop body: the blocks that iterate, ending where the loop tests.
    fn loop_body(&self, blocks: &BTreeSet<usize>, boundary: usize, header: usize) -> Self {
        // A loop inside a loop may not claim a block the enclosing loop's body does not hold: the
        // scope of a body is the intersection, so a nesting cannot widen it.
        let scope = match &self.scope {
            Some(outer) => blocks.intersection(outer).copied().collect(),
            None => blocks.clone(),
        };
        Self {
            boundary: Some(boundary),
            scope: Some(scope),
            own_loop: Some(header),
        }
    }

    /// The frame of one arm of a branch: everything the enclosing frame allowed, ending at the join.
    ///
    /// The arm is no longer the loop body's own walk: an arm that jumps back to the enclosing loop's
    /// test is an ordinary continuation of the body (a `continue`), and reading it as "the walk
    /// re-entered the loop it is building" would refuse a shape this layer has always answered.
    fn arm(&self, join: Option<usize>) -> Self {
        Self {
            boundary: join,
            scope: self.scope.clone(),
            own_loop: None,
        }
    }

    /// Whether a walk must stop at one node: the node it ends at, or one the scope does not hold.
    fn stops_at(&self, node: usize) -> bool {
        self.boundary == Some(node)
            || self
                .scope
                .as_ref()
                .is_some_and(|scope| !scope.contains(&node))
    }
}

struct Walker<'a> {
    canonical: &'a CanonicalCfg,
    view: &'a NormalFlowView,
    ssa: &'a SsaTable,
    operations: &'a Operations,
    /// The exception table the same decode stated: the guarded rules of P3 2.4 read the ranges and
    /// the catch types from it, and the walk reads it for the crossing-range refusal.
    handlers: &'a [ExceptionHandlerFact],
    /// The profile the run is presented under: `twr@1`'s output is Java 7 syntax, so a profile that
    /// presents the artifact as an older release does not admit it ([`crate::pass::Pass::admits`]).
    profile: &'a crate::pass::RecoveryProfile,
    budget: &'a mut Budget,
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
    fn region_at(
        &mut self,
        start: &CanonicalBlockId,
        frame: &Frame,
    ) -> Result<(Region, Option<CanonicalBlockId>), StopReason> {
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
    ) -> Result<(Region, Option<CanonicalBlockId>), StopReason> {
        let mut prefix: Vec<CanonicalBlockId> = Vec::new();
        let mut current = start.clone();
        loop {
            let at = Some(current.bci());
            poll(self.budget, at)?;
            charge(self.budget, CountedBudgetDimension::AnalysisSteps, 1, at)?;
            let Some(node) = self.view.index_of(&current) else {
                return Ok((
                    Region::Fallback {
                        blocks: vec![current.clone()],
                        reason: FallbackReason::UncoveredBlocks {
                            blocks: vec![current.bci()],
                        },
                    },
                    None,
                ));
            };
            if frame.stops_at(node) {
                // The join of the enclosing structure, or the code after the enclosing loop: this
                // region ends where its caller continues.
                return Ok((Region::Straight { blocks: prefix }, None));
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
                return Ok((Region::Straight { blocks: prefix }, Some(current)));
            }
            if !self.visited.insert(node) {
                return Ok((
                    Region::Fallback {
                        blocks: vec![current.clone()],
                        reason: FallbackReason::Loop {
                            block_bci: current.bci(),
                        },
                    },
                    None,
                ));
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
                        return Ok((Region::Guard { prefix, plan }, join));
                    }
                    crate::guard::Verdict::Refused { pass, refusal, at } => {
                        let mut blocks = prefix;
                        blocks.push(current);
                        return Ok((
                            Region::Fallback {
                                blocks,
                                reason: FallbackReason::Guard {
                                    pass,
                                    code: refusal.code(),
                                    at,
                                    message: refusal.message().to_string(),
                                },
                            },
                            None,
                        ));
                    }
                    crate::guard::Verdict::NotGuarded => {}
                }
                let mut blocks = prefix;
                blocks.push(current);
                return Ok((Region::Fallback { blocks, reason }, None));
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
                    let mut blocks = prefix;
                    blocks.push(current.clone());
                    return Ok((
                        Region::Fallback {
                            blocks,
                            reason: FallbackReason::LoopLeavesEarly {
                                block_bci: current.bci(),
                            },
                        },
                        None,
                    ));
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
                    return Ok((Region::Straight { blocks: prefix }, None));
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
                            let mut blocks = prefix;
                            blocks.push(branch.clone());
                            return Ok((
                                Region::Fallback {
                                    blocks,
                                    reason: FallbackReason::MissingEvidence {
                                        block_bci: branch.bci(),
                                    },
                                },
                                None,
                            ));
                        }
                    };
                    let Some((op, target)) = self
                        .operations
                        .get(branch_bci)
                        .and_then(Operation::comparison)
                    else {
                        let mut blocks = prefix;
                        blocks.push(branch.clone());
                        return Ok((
                            Region::Fallback {
                                blocks,
                                reason: FallbackReason::UnknownBranchSense {
                                    block_bci: branch.bci(),
                                    branch_bci,
                                },
                            },
                            None,
                        ));
                    };
                    let Some((fall_through, taken)) = self.split_arms(&successors, target) else {
                        let mut blocks = prefix;
                        blocks.push(branch.clone());
                        return Ok((
                            Region::Fallback {
                                blocks,
                                reason: FallbackReason::UnrenderableOperand { bci: branch_bci },
                            },
                            None,
                        ));
                    };
                    let join_node = self
                        .view
                        .immediate_post_dominator(node)
                        .filter(|join| *join != node);
                    let join = join_node.and_then(|join| self.view.id_of(join).cloned());
                    let then_node = self.view.index_of(&fall_through);
                    let else_node = self.view.index_of(&taken);
                    if join_node.is_some() && (then_node == join_node || else_node == join_node) {
                        // An arm that *is* the join is a one-armed branch. The subset models two
                        // arms, and printing an empty `else` for it would claim a structure the
                        // bytecode does not have, so it falls back instead.
                        let mut blocks = prefix;
                        blocks.push(branch.clone());
                        return Ok((
                            Region::Fallback {
                                blocks,
                                reason: FallbackReason::ArmsDoNotMeet {
                                    block_bci: branch.bci(),
                                },
                            },
                            None,
                        ));
                    }
                    let arm_frame = frame.arm(join_node);
                    let (then_arm, _) = self.region_at(&fall_through, &arm_frame)?;
                    let (else_arm, _) = self.region_at(&taken, &arm_frame)?;
                    // Both arms must meet the join, or leave the method; anything else means the
                    // recovered structure runs into a block the branch did not describe.
                    if let Some(join_node) = join_node {
                        let then_meets =
                            then_node.is_some_and(|node| self.view.reaches(node, join_node));
                        let else_meets =
                            else_node.is_some_and(|node| self.view.reaches(node, join_node));
                        let both_end = !then_meets && !else_meets;
                        if !(then_meets && else_meets) && !both_end {
                            let mut blocks = prefix;
                            blocks.push(branch.clone());
                            return Ok((
                                Region::Fallback {
                                    blocks,
                                    reason: FallbackReason::ArmsDoNotMeet {
                                        block_bci: branch.bci(),
                                    },
                                },
                                None,
                            ));
                        }
                    }
                    // The condition's arity is a precondition of the statement the builder writes.
                    let reads = self
                        .ssa
                        .block(&branch)
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
                    if reads != expected {
                        let mut blocks = prefix;
                        blocks.push(branch.clone());
                        return Ok((
                            Region::Fallback {
                                blocks,
                                reason: FallbackReason::UnrenderableOperand { bci: branch_bci },
                            },
                            None,
                        ));
                    }
                    let region = Region::If {
                        prefix,
                        branch,
                        branch_bci,
                        then_arm: Box::new(then_arm),
                        else_arm: Box::new(else_arm),
                        join: join.clone(),
                    };
                    return Ok((region, join));
                }
                count => {
                    let branch = current.clone();
                    let branch_bci = match self.terminal_bci(&branch) {
                        Some(bci) => bci,
                        None => {
                            let mut blocks = prefix;
                            blocks.push(branch.clone());
                            return Ok((
                                Region::Fallback {
                                    blocks,
                                    reason: FallbackReason::MissingEvidence {
                                        block_bci: branch.bci(),
                                    },
                                },
                                None,
                            ));
                        }
                    };
                    let Some((cases, default)) =
                        self.operations.get(branch_bci).and_then(Operation::switch)
                    else {
                        // Three or more targets whose terminal instruction the decode does not
                        // state as a `switch`: no shape this subset knows.
                        let mut blocks = prefix;
                        blocks.push(branch.clone());
                        return Ok((
                            Region::Fallback {
                                blocks,
                                reason: FallbackReason::BranchTargets {
                                    block_bci: branch.bci(),
                                    successors: count,
                                },
                            },
                            None,
                        ));
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

    /// Whether a block leaves through an edge the projection does not carry.
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

    /// The BCI of the last instruction of one block, as the names table states it.
    fn terminal_bci(&self, block: &CanonicalBlockId) -> Option<u32> {
        self.ssa.block(block).and_then(|block| {
            block
                .instructions()
                .last()
                .map(|instruction| instruction.bci())
        })
    }

    /// Whether the test block of a structure holds nothing but the values its test reads.
    ///
    /// The test block's instructions are written *inside* the structure they decide — in a loop's
    /// condition, in an `if`'s condition — and only value-producing instructions have a place
    /// there: a `Push`, a `Load` or an `Arithmetic` becomes the text of an operand, so it runs
    /// exactly once per evaluation and in the same order the bytecode ran it. A store, an
    /// increment, a call, a return or an operation this subset does not model is an **effect** of
    /// the test block, and there is nowhere to write it that keeps its execution count and its
    /// order relative to the structure it belongs to: hoisting it out of a loop would run it once,
    /// and putting it in the body would run it after the test. So the structure is quoted instead.
    fn test_is_pure(&self, block: &CanonicalBlockId, test_bci: u32) -> Result<(), FallbackReason> {
        let Some(names) = self.ssa.block(block) else {
            return Ok(());
        };
        for instruction in names.instructions() {
            if instruction.bci() == test_bci {
                continue;
            }
            let value_only = matches!(
                self.operations.get(instruction.bci()),
                Some(Operation::Push(_) | Operation::Load { .. } | Operation::Arithmetic { .. })
            );
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
    ) -> Result<(Region, Option<CanonicalBlockId>), StopReason> {
        let Some(loop_of) = self.view.loop_entered_at(header_node).cloned() else {
            return Ok((
                Region::Fallback {
                    blocks: vec![header.clone()],
                    reason: FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    },
                },
                None,
            ));
        };
        let blocks = loop_of.blocks().clone();
        if loop_of.is_irreducible() {
            // Defensive: `recover` refuses a whole body whose graph is irreducible before the walk
            // starts, so a header that is still irreducible here is a payload this layer did not
            // expect to see — and it is reported, not guessed at.
            let bcis = blocks
                .iter()
                .filter_map(|node| self.view.id_of(*node).map(CanonicalBlockId::bci))
                .collect();
            return Ok((
                Region::Fallback {
                    blocks: vec![header.clone()],
                    reason: FallbackReason::Irreducible { blocks: bcis },
                },
                None,
            ));
        }
        if let Some(region) = self.header_tested_loop(header, header_node, &blocks, frame)? {
            return Ok(region);
        }
        if let Some(region) = self.latch_tested_loop(header, header_node, &blocks, frame)? {
            return Ok(region);
        }
        Ok((
            Region::Fallback {
                blocks: vec![header.clone()],
                reason: FallbackReason::LoopShape {
                    block_bci: header.bci(),
                },
            },
            None,
        ))
    }

    /// The `while`/`for` shape: the header's own branch tests and one of its arms leaves the loop.
    fn header_tested_loop(
        &mut self,
        header: &CanonicalBlockId,
        header_node: usize,
        blocks: &BTreeSet<usize>,
        frame: &Frame,
    ) -> Result<Option<(Region, Option<CanonicalBlockId>)>, StopReason> {
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
        if let Some(reason) = self.leaving_edge(header) {
            return Ok(Some((
                Region::Fallback {
                    blocks: vec![header.clone()],
                    reason,
                },
                None,
            )));
        }
        if let Err(reason) = self.test_is_pure(header, test_bci) {
            return Ok(Some((
                Region::Fallback {
                    blocks: vec![header.clone()],
                    reason,
                },
                None,
            )));
        }
        // Which way through the test iterates: the branch's own target says it, and nothing else
        // can — the two successors are its arms, not its polarity.
        let continuation = if inside.bci() == target {
            Continuation::Taken
        } else {
            Continuation::FallThrough
        };
        let body_frame = frame.loop_body(blocks, header_node, header_node);
        let (body, _) = self.region_at(&inside, &body_frame)?;
        self.visited.insert(header_node);
        let mut expected = blocks.clone();
        expected.remove(&header_node);
        if !self.covers(&expected) {
            return Ok(Some((
                Region::Fallback {
                    blocks: vec![header.clone()],
                    reason: FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    },
                },
                None,
            )));
        }
        Ok(Some((
            Region::Loop {
                header: header.clone(),
                test: header.clone(),
                test_bci,
                form: LoopForm::While,
                continuation,
                body: Box::new(body),
                exit: Some(outside.clone()),
            },
            Some(outside),
        )))
    }

    /// The `do … while` shape: one latch, whose own branch tests and jumps back to the header.
    fn latch_tested_loop(
        &mut self,
        header: &CanonicalBlockId,
        header_node: usize,
        blocks: &BTreeSet<usize>,
        frame: &Frame,
    ) -> Result<Option<(Region, Option<CanonicalBlockId>)>, StopReason> {
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
            return Ok(Some((
                Region::Loop {
                    header: header.clone(),
                    test: header.clone(),
                    test_bci,
                    form: LoopForm::DoWhile,
                    continuation: if target == header.bci() {
                        Continuation::Taken
                    } else {
                        Continuation::FallThrough
                    },
                    body: Box::new(Region::Straight {
                        blocks: vec![header.clone()],
                    }),
                    exit: Some(exit.clone()),
                },
                Some(exit),
            )));
        }
        for block in [header, &latch] {
            if let Some(reason) = self.leaving_edge(block) {
                return Ok(Some((
                    Region::Fallback {
                        blocks: vec![header.clone()],
                        reason,
                    },
                    None,
                )));
            }
        }
        if let Err(reason) = self.test_is_pure(&latch, test_bci) {
            return Ok(Some((
                Region::Fallback {
                    blocks: vec![header.clone()],
                    reason,
                },
                None,
            )));
        }
        let continuation = if target == header.bci() {
            Continuation::Taken
        } else {
            Continuation::FallThrough
        };
        let body_frame = frame.loop_body(blocks, latch_node, header_node);
        let (body, _) = self.region_at(header, &body_frame)?;
        self.visited.insert(latch_node);
        let mut expected = blocks.clone();
        expected.remove(&latch_node);
        if !self.covers(&expected) {
            return Ok(Some((
                Region::Fallback {
                    blocks: vec![header.clone()],
                    reason: FallbackReason::LoopShape {
                        block_bci: header.bci(),
                    },
                },
                None,
            )));
        }
        Ok(Some((
            Region::Loop {
                header: header.clone(),
                test: latch,
                test_bci,
                form: LoopForm::DoWhile,
                continuation,
                body: Box::new(body),
                exit: Some(exit.clone()),
            },
            Some(exit),
        )))
    }

    /// Whether every expected block of a loop was walked.
    ///
    /// A loop is presented only when the walk reached every block that iterates: a block of the
    /// loop the body never claimed would be a block whose execution the presentation dropped.
    fn covers(&self, expected: &BTreeSet<usize>) -> bool {
        expected.iter().all(|node| self.visited.contains(node))
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
    ) -> Result<(Region, Option<CanonicalBlockId>), StopReason> {
        let quoted = |reason: FallbackReason| {
            let mut blocks: Vec<CanonicalBlockId> = prefix.to_vec();
            blocks.push(branch.clone());
            (Region::Fallback { blocks, reason }, None)
        };
        let join_node = self
            .view
            .immediate_post_dominator(node)
            .filter(|join| *join != node);
        let join = join_node.and_then(|join| self.view.id_of(join).cloned());
        let join_bci = join.as_ref().map(CanonicalBlockId::bci);
        // Every target the decode names must be a successor the graph holds, and every successor a
        // target: a `switch` the decode and the graph disagree about is not a shape to present.
        let crosses = |target: u32| successors.iter().any(|successor| successor.bci() == target);
        if !crosses(default) || cases.iter().any(|(_, target)| !crosses(*target)) {
            return Ok(quoted(FallbackReason::SwitchShape {
                block_bci: branch.bci(),
            }));
        }
        if successors.iter().any(|successor| {
            !cases.iter().any(|(_, target)| *target == successor.bci())
                && successor.bci() != default
        }) {
            return Ok(quoted(FallbackReason::SwitchShape {
                block_bci: branch.bci(),
            }));
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
        let arm_frame = frame.arm(join_node);
        let mut built: Vec<SwitchGroup> = Vec::with_capacity(groups.len());
        let mut claimed: Vec<BTreeSet<usize>> = Vec::with_capacity(groups.len());
        for (keys, is_default, target) in groups {
            if Some(target) == join_bci {
                // The arm is the join itself: the case runs no code and leaves the switch.
                built.push(SwitchGroup {
                    keys,
                    default: is_default,
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
                return Ok(quoted(FallbackReason::SwitchShape {
                    block_bci: branch.bci(),
                }));
            };
            let (arm, _) = self.region_at(&start, &arm_frame)?;
            let arm_nodes: BTreeSet<usize> = arm
                .blocks()
                .iter()
                .filter_map(|block| self.view.index_of(block))
                .collect();
            // Two arms that claim the same block are a case falling through into another case's
            // code: Java has a way to write that (`case 0: case 1:` shares a *target*, and those
            // are one group), but not two targets whose code overlaps.
            if claimed.iter().any(|other| !other.is_disjoint(&arm_nodes)) {
                return Ok(quoted(FallbackReason::SwitchArmsOverlap {
                    block_bci: branch.bci(),
                }));
            }
            claimed.push(arm_nodes);
            built.push(SwitchGroup {
                keys,
                default: is_default,
                arm: Box::new(arm),
            });
        }
        Ok((
            Region::Switch {
                prefix: prefix.to_vec(),
                branch: branch.clone(),
                branch_bci,
                groups: built,
                join: join.clone(),
            },
            join,
        ))
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
