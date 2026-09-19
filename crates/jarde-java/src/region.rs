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

use crate::facts::{Operation, RecoveryFacts};
use crate::normal_flow::NormalFlowView;
use crate::stop::{StopReason, charge, poll};

/// Why one region could only be kept as bytecode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FallbackReason {
    /// An exception edge leaves the region: a handler's shape, which 1.3b/2.4 recovers.
    ExceptionEdge {
        block_bci: u32,
        handler_ordinal: u32,
    },
    /// A `jsr` context entry leaves the region: a subroutine body is not a Java statement.
    SubroutineEntry { block_bci: u32, call_site: u32 },
    /// A branch with three or more targets: a `switch`, which 1.3b recovers.
    BranchTargets { block_bci: u32, successors: usize },
    /// The region can be re-entered: a loop, which 1.3b recovers.
    Loop { block_bci: u32 },
    /// The branch's sense was not decoded, so the two arms cannot be told apart.
    UnknownBranchSense { block_bci: u32, branch_bci: u32 },
    /// The branch's decoded arity does not match the values it reads.
    UnrenderableOperand { bci: u32 },
    /// The two arms do not meet at one join.
    ArmsDoNotMeet { block_bci: u32 },
    /// Live blocks the walk did not claim, reachable only through edges the projection leaves out.
    UncoveredBlocks { blocks: Vec<u32> },
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
            Self::UnknownBranchSense { .. } => "jre_region_unknown_branch_sense",
            Self::UnrenderableOperand { .. } => "jre_region_unrenderable_operand",
            Self::ArmsDoNotMeet { .. } => "jre_region_arms_do_not_meet",
            Self::UncoveredBlocks { .. } => "jre_region_uncovered_blocks",
            Self::MissingEvidence { .. } => "jre_region_missing_evidence",
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
                "block at BCI {block_bci} branches to {successors} targets: a switch is not part of the recoverable subset"
            ),
            Self::Loop { block_bci } => format!(
                "block at BCI {block_bci} can be re-entered: a loop is not part of the recoverable subset yet"
            ),
            Self::UnknownBranchSense {
                block_bci,
                branch_bci,
            } => format!(
                "the branch at BCI {branch_bci} in block {block_bci} has no decoded sense, so its two arms cannot be told apart"
            ),
            Self::UnrenderableOperand { bci } => {
                format!("the instruction at BCI {bci} reads values this subset cannot render")
            }
            Self::ArmsDoNotMeet { block_bci } => {
                format!("the two arms of the branch in block {block_bci} do not meet at one join")
            }
            Self::UncoveredBlocks { blocks } => format!(
                "{} live block(s) are reachable only through edges the normal-flow view leaves out: {blocks:?}",
                blocks.len()
            ),
            Self::MissingEvidence { block_bci } => format!(
                "block at BCI {block_bci} has no last instruction the names table states, so its branch cannot be named"
            ),
        }
    }
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
    /// A run of blocks that could not be shown to be a Java structure; the text quotes it.
    Fallback {
        blocks: Vec<CanonicalBlockId>,
        reason: FallbackReason,
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
            Self::Fallback { blocks, .. } => blocks.iter().collect(),
        }
    }

    /// Whether this region is presented as Java structure rather than as quoted bytecode.
    pub fn is_structured(&self) -> bool {
        match self {
            Self::Straight { .. } => true,
            Self::If {
                then_arm, else_arm, ..
            } => then_arm.is_structured() && else_arm.is_structured(),
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
            Self::Fallback { reason, .. } => vec![reason.clone()],
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
}

/// Recovers the region tree of one method from the projection and the decoded facts.
pub(crate) fn recover(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    facts: &RecoveryFacts,
    budget: &mut Budget,
) -> Result<Recovered, StopReason> {
    let mut walker = Walker {
        canonical,
        view,
        ssa,
        facts,
        budget,
        visited: BTreeSet::new(),
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
        let (region, next) = walker.region_at(&node, None)?;
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
    Ok(Recovered {
        regions,
        claimed: walker.visited,
        blocks: canonical.blocks().len(),
    })
}

struct Walker<'a> {
    canonical: &'a CanonicalCfg,
    view: &'a NormalFlowView,
    ssa: &'a SsaTable,
    facts: &'a RecoveryFacts,
    budget: &'a mut Budget,
    visited: BTreeSet<usize>,
}

impl Walker<'_> {
    /// The region that starts at one block, and the block the run continues at afterwards.
    fn region_at(
        &mut self,
        start: &CanonicalBlockId,
        boundary: Option<usize>,
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
            if Some(node) == boundary {
                // The join of the enclosing `if`: this region ends where its caller continues.
                return Ok((Region::Straight { blocks: prefix }, None));
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
                let mut blocks = prefix;
                blocks.push(current);
                return Ok((Region::Fallback { blocks, reason }, None));
            }
            let successors = self.view.successor_ids(&current);
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
                    if let Some(boundary) = boundary
                        && self.view.index_of(&next) == Some(boundary)
                    {
                        return Ok((Region::Straight { blocks: prefix }, None));
                    }
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
                    let Some(Operation::Comparison { op }) = self.facts.operation(branch_bci)
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
                    let op = *op;
                    let (fall_through, taken) =
                        match self.split_arms(&branch, branch_bci, &successors) {
                            Some(split) => split,
                            None => {
                                let mut blocks = prefix;
                                blocks.push(branch.clone());
                                return Ok((
                                    Region::Fallback {
                                        blocks,
                                        reason: FallbackReason::Loop {
                                            block_bci: branch.bci(),
                                        },
                                    },
                                    None,
                                ));
                            }
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
                    let arm_boundary = join_node;
                    let (then_arm, _) = self.region_at(&fall_through, arm_boundary)?;
                    let (else_arm, _) = self.region_at(&taken, arm_boundary)?;
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
                    let mut blocks = prefix;
                    blocks.push(current.clone());
                    return Ok((
                        Region::Fallback {
                            blocks,
                            reason: FallbackReason::BranchTargets {
                                block_bci: current.bci(),
                                successors: count,
                            },
                        },
                        None,
                    ));
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

    /// Which successor control falls through to, and which one the branch transfers to.
    ///
    /// The fall-through arm is the successor that *follows* the branch instruction in the body: a
    /// two-way branch either continues at the next instruction or transfers to its operand's
    /// target, and the target of a forward branch is always further than the fall-through. A branch
    /// whose two successors are both behind it is a back edge — a loop, which this subset refuses.
    fn split_arms(
        &self,
        branch: &CanonicalBlockId,
        branch_bci: u32,
        successors: &[CanonicalBlockId],
    ) -> Option<(CanonicalBlockId, CanonicalBlockId)> {
        let mut forward: Vec<&CanonicalBlockId> = successors
            .iter()
            .filter(|successor| successor.bci() > branch_bci && *successor != branch)
            .collect();
        if forward.len() != 2 {
            return None;
        }
        forward.sort_by_key(|successor| successor.bci());
        Some((forward[0].clone(), forward[1].clone()))
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
