//! The facts a loop/branch composition decision reads (A4.1), owned once.
//!
//! A recovered [`Region`] tree states how each subtree leaves: one arm of a two-edge body takes a
//! proved `break`, another continues its loop, a switch arm breaks to the switch's join. The walk's
//! *composition* decisions — which arm of a branch never arrives at the join, whether a sibling still
//! enters it, which arm may be reached from a switch join — all ask that same question. This module
//! is the one place that answers it: [`RegionTransfer`] names the transfer and its proved
//! destination, and [`Region::transfer_leaf`] reads it off the variant that states it.
//!
//! It re-derives nothing and owns no block. The destination a transfer names is the *variant's own*
//! fact (`LoopBreak`/`LoopContinue`'s loop identity, `SwitchBreak`'s join); resolving that identity
//! to a physical block, and proving it against the CFG/SSA, is [`super::summary`]'s reconciliation.
//! Keeping the reading here is what lets the composition sites stop matching the variants field by
//! field: they consume the fact, and only this module states which variants state facts at all.
//!
//! This module also owns the other question a combination decision asks of a subtree: **what
//! exception clauses does it bind, and what does it compose?** A `try` states its clauses in
//! exception-table order — the order the JVM dispatches in, and therefore the order the text must
//! write — and a guard states the one whole-construct clause its plan proves. Reading that off the
//! variant that states it, once ([`Region::exception_clauses`], [`Region::composed_children`],
//! [`Region::protected_body`]), is what lets the derived view and the named-catch combination
//! decisions share one reading instead of each destructuring `Try`/`Guard` in its own way.
//!
//! Everything here is a pure read of a tree the walk already built, so consuming it changes neither
//! what the walk decides nor what the run is charged.

use jarde_jvm::method_ir::CanonicalBlockId;

use super::Region;

/// One exception clause a subtree binds, as the variant that states it records it.
///
/// A `try` states one per handler entry, in exception-table order — the order the JVM dispatches
/// in, so the order the text must write, and the order a combination decision must keep when it
/// compares a clause with the table row that owns it. A guard states the one whole-construct clause
/// its plan proved. Both are the *variant's own* fact: the types it names, the handler entry it
/// reaches, the slot its binding store fills, and the body its braces hold.
pub(crate) struct RegionExceptionClause<'a> {
    /// The types the clause names, or the proved single catch-all.
    pub(crate) types: crate::guard::CatchTypes,
    /// The canonical block the clause's handler entry is.
    pub(crate) handler: CanonicalBlockId,
    /// The slot the handler's first instruction stores the caught exception into.
    pub(crate) parameter: u16,
    /// The clause's body, when the variant states it as a region of its own.
    ///
    /// A `try` recovers every handler body through the same recursion as any other region, so its
    /// clause states one. A guard's whole-construct clause is the plan's own instruction range
    /// inside one handler block — it has no region of its own, and the statement writes it.
    pub(crate) body: Option<&'a Region>,
}

/// One structure transfer the tree states, with the destination its own variant names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionTransfer {
    /// BCI of the terminal instruction that transfers.
    pub(crate) source_bci: u32,
    pub(crate) kind: RegionTransferKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionTransferKind {
    /// An edge to the proved break destination of the named loop.
    Break { loop_header: CanonicalBlockId },
    /// An edge to the proved continue target of the named loop.
    Continue { loop_header: CanonicalBlockId },
    /// An edge out of a switch arm to that switch's proved join.
    SwitchBreak {
        switch_bci: u32,
        join: CanonicalBlockId,
    },
}

impl RegionTransferKind {
    /// Whether this transfer leaves its loop for the loop's break destination, so the edge never
    /// arrives at a join the arm's sibling meets.
    pub(crate) fn is_break(&self) -> bool {
        matches!(self, Self::Break { .. })
    }

    /// Whether this transfer goes back to its loop's continue target instead of arriving at a join.
    pub(crate) fn is_continue(&self) -> bool {
        matches!(self, Self::Continue { .. })
    }

    /// Whether this transfer is a switch arm's break, with the switch and join it names.
    pub(crate) fn switch_break(&self) -> Option<(u32, &CanonicalBlockId)> {
        match self {
            Self::SwitchBreak { switch_bci, join } => Some((*switch_bci, join)),
            _ => None,
        }
    }

    /// Whether this transfer leaves its loop by either proved loop edge — the break destination or
    /// the continue target. Both leave the arm without arriving at a join its sibling meets.
    pub(crate) fn is_loop_jump(&self) -> bool {
        matches!(self, Self::Break { .. } | Self::Continue { .. })
    }

    /// The loop this transfer names, when it is a loop transfer. A switch arm's break names a
    /// switch and a join instead, so a consumer asking about loops must refuse it rather than
    /// borrow one.
    pub(crate) fn loop_header(&self) -> Option<&CanonicalBlockId> {
        match self {
            Self::Break { loop_header } | Self::Continue { loop_header } => Some(loop_header),
            Self::SwitchBreak { .. } => None,
        }
    }
}

impl Region {
    /// The proved structure transfer this region is, when it is one.
    ///
    /// It never looks inside a [`Region::Sequence`]: the region asked about is a run's own tail, and
    /// whether a *nested* run is closed by a break is [`Region::ends_in_loop_break`]'s separate
    /// question.
    pub(crate) fn transfer_leaf(&self) -> Option<RegionTransfer> {
        let transfer = |source_bci, kind| Some(RegionTransfer { source_bci, kind });
        match self {
            Self::LoopBreak {
                source_bci,
                loop_header,
            } => transfer(
                *source_bci,
                RegionTransferKind::Break {
                    loop_header: loop_header.clone(),
                },
            ),
            Self::LoopContinue {
                source_bci,
                loop_header,
            } => transfer(
                *source_bci,
                RegionTransferKind::Continue {
                    loop_header: loop_header.clone(),
                },
            ),
            Self::SwitchBreak {
                source_bci,
                switch_bci,
                join,
            } => transfer(
                *source_bci,
                RegionTransferKind::SwitchBreak {
                    switch_bci: *switch_bci,
                    join: join.clone(),
                },
            ),
            _ => None,
        }
    }

    /// Whether this subtree is closed by a break out of its loop: the region itself, or a
    /// [`Region::Sequence`] whose last child is one. An arm that ends this way never arrives at the
    /// join, so only its sibling's tail block is a predecessor of the continuation.
    pub(crate) fn ends_in_loop_break(&self) -> bool {
        match self {
            Self::LoopBreak { .. } => true,
            Self::Sequence { regions } => matches!(regions.last(), Some(Self::LoopBreak { .. })),
            _ => false,
        }
    }

    /// The exit this loop's own variant states, when this region is a loop and it names one.
    ///
    /// An endless body states no exit, so an exit-vs-join reading answers `None` — never a
    /// destination borrowed from whatever block it was being compared with. Every composition
    /// decision that asks *does this arm leave where the join is* reads the exit here instead of
    /// destructuring the `Loop` variant's field at each site.
    pub(crate) fn loop_exit(&self) -> Option<&CanonicalBlockId> {
        match self {
            Self::Loop { exit, .. } => exit.as_ref(),
            _ => None,
        }
    }

    /// Whether this region is a loop whose own stated exit is `target`.
    ///
    /// The naked question behind it is [`Region::loop_exit`]; this spells the composition reading
    /// so a caller that only needs the answer cannot accidentally treat "no stated exit" as
    /// "matches the join" through an `Option` comparison.
    pub(crate) fn loop_exits_at(&self, target: &CanonicalBlockId) -> bool {
        self.loop_exit() == Some(target)
    }

    /// The exception clauses this subtree binds **directly**, in exception-table order.
    ///
    /// A `try` states one clause per handler entry — the order the JVM dispatches in, so the order
    /// the text must write, and the order every combination decision that compares a clause with
    /// the table row owning it must keep. A guard states the one whole-construct clause its plan
    /// proved, which has no region of its own: its body is the plan's instruction range and the
    /// statement writes it, so [`RegionExceptionClause::body`] is `None` there.
    ///
    /// The subtrees a clause holds are [`Region::composed_children`], not this: a clause body is
    /// recovered through the same recursion as any other region and is a child like any other.
    pub(crate) fn exception_clauses(&self) -> Vec<RegionExceptionClause<'_>> {
        match self {
            Self::Try { catches, .. } => catches
                .iter()
                .map(|clause| RegionExceptionClause {
                    types: clause.types().clone(),
                    handler: clause.handler().clone(),
                    parameter: clause.parameter(),
                    body: Some(clause.body()),
                })
                .collect(),
            Self::Guard { plan, .. } => plan
                .enclosing()
                .map(|enclosing| RegionExceptionClause {
                    types: crate::guard::CatchTypes::Named(vec![enclosing.catch_type()]),
                    handler: enclosing.handler().clone(),
                    parameter: enclosing.parameter(),
                    body: None,
                })
                .into_iter()
                .collect(),
            _ => Vec::new(),
        }
    }

    /// The protected body of this statement, when it is one that protects a range.
    ///
    /// A `try`'s body is the range its clauses cover; a guard's is the body the rule proved. Both
    /// are the same question a composition decision asks before it hands the range to a child walk.
    pub(crate) fn protected_body(&self) -> Option<&Region> {
        match self {
            Self::Try { body, .. } => Some(body),
            Self::Guard { body, .. } => body.as_deref(),
            _ => None,
        }
    }

    /// This region as a loop, when it is one — directly, or behind the single-child `Sequence` a
    /// proved loop body is wrapped in.
    ///
    /// Two spellings of one fact: a body that is a loop walks out as the loop itself, and one whose
    /// walk proved a loop and nothing else is kept in a `Sequence` so the transfer stays inside it.
    /// A composition decision asking *is the protected range a loop* reads this instead of matching
    /// both shapes itself, and gets `None` — never a loop borrowed from a sibling — for anything
    /// that is not one.
    pub(crate) fn loop_leaf(&self) -> Option<&Region> {
        match self {
            Self::Loop { .. } => Some(self),
            Self::Sequence { regions } => match regions.as_slice() {
                [only @ Self::Loop { .. }] => Some(only),
                _ => None,
            },
            _ => None,
        }
    }

    /// The subtrees this region composes, in the order the text writes them.
    ///
    /// A sequence's members, a loop's body, a branch's two arms, a switch's arms, a try's protected
    /// range and then its clauses in table order, a guard's body and then its separately bounded
    /// cleanup. Nothing else holds a child: a straight run and a value graph state blocks, and a
    /// transfer states an edge. Every walk of the tree — collecting facts, validating a subtree,
    /// reading guards — enumerates children here instead of destructuring the variants again.
    pub(crate) fn composed_children(&self) -> Vec<&Region> {
        match self {
            Self::Sequence { regions } | Self::Loop { body: regions, .. } => {
                regions.iter().collect()
            }
            Self::If {
                then_arm, else_arm, ..
            }
            | Self::CompoundIf {
                then_arm, else_arm, ..
            } => vec![then_arm, else_arm],
            Self::ShortCircuitValue { tail, .. } => tail.iter().collect(),
            Self::Switch { groups, .. } | Self::StringSwitch { groups, .. } => {
                groups.iter().map(|group| group.arm.as_ref()).collect()
            }
            Self::Try { body, catches, .. } => {
                let mut children: Vec<&Region> = vec![body];
                children.extend(catches.iter().map(|clause| clause.body()));
                children
            }
            Self::Guard {
                body, finally_body, ..
            } => {
                let mut children: Vec<&Region> = Vec::new();
                if let Some(body) = body {
                    children.push(body);
                }
                if let Some(cleanup) = finally_body {
                    children.push(cleanup);
                }
                children
            }
            Self::Straight { .. }
            | Self::TwoExitReturn { .. }
            | Self::SharedTailEarlyReturn { .. }
            | Self::EntrySplitLoop { .. }
            | Self::LoopBreak { .. }
            | Self::LoopContinue { .. }
            | Self::SwitchBreak { .. }
            | Self::Fallback { .. } => Vec::new(),
        }
    }

    /// Whether this subtree completes inside its own structure: a straight run, a sequence of
    /// closed runs, or an `if` whose arms both meet a stated join.
    ///
    /// It is deliberately stronger than [`Region::is_structured`]: an `if` whose `join` is `None`
    /// is structured but not *closed*, because both its arms leave the method and there is no
    /// continuation for the enclosing arm to fall into. A loop arm joining a sibling's join needs
    /// the stronger answer.
    pub(crate) fn is_closed_run(&self) -> bool {
        match self {
            Self::Straight { .. } => true,
            Self::Sequence { regions } => regions.iter().all(Self::is_closed_run),
            Self::If {
                then_arm,
                else_arm,
                join: Some(_),
                ..
            }
            | Self::CompoundIf {
                then_arm,
                else_arm,
                join: Some(_),
                ..
            } => then_arm.is_closed_run() && else_arm.is_closed_run(),
            _ => false,
        }
    }
}

/// One walk frame's switch-arm scope: the join node no arm of it may claim, the branch
/// instruction that owns that join when this frame's proof named one, and whether the switch's
/// fallthrough proof made this frame one of its arms.
///
/// The three frame fields are one fact — *which switch, if any, owns this arm's join* — but the
/// questions asked of it are not the same question, so the facts are exposed separately rather
/// than folded into one conjunction: an arm may ask only *is this block my join* (the case-entry
/// and loop-jump readings) or also *did the switch's own proof make me an arm* (the break and
/// gateway readings). Keeping the raw facts here and composing them at each site is what lets the
/// three-field destructuring disappear without changing any site's condition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SwitchArm {
    /// The walk's node index of the join every arm of this switch breaks to.
    join_node: Option<usize>,
    /// The switch's branch instruction, when this frame's proof named one.
    branch_bci: Option<u32>,
    /// Whether the switch's fallthrough proof made this frame one of its arms.
    fallthrough_proved: bool,
}

impl SwitchArm {
    /// The scope a frame states. It exists whenever the frame names any part of one, so a caller
    /// asking only the join question sees the same answer it saw before the fact had an owner.
    pub(crate) fn of(
        join_node: Option<usize>,
        branch_bci: Option<u32>,
        fallthrough_proved: bool,
    ) -> Option<Self> {
        (join_node.is_some() || branch_bci.is_some() || fallthrough_proved).then_some(Self {
            join_node,
            branch_bci,
            fallthrough_proved,
        })
    }

    /// Whether this arm's join is the block at `node` — the plain join question, which a frame
    /// answers whether or not the switch's fallthrough proof named it.
    pub(crate) fn joins_at(self, node: usize) -> bool {
        self.join_node == Some(node)
    }

    /// Whether the switch's fallthrough proof made this frame an arm **and** its join is `node`.
    ///
    /// This is the stronger reading the switch-break and gateway sites need: a join alone is not
    /// a switch arm, because the enclosing structure may hold that block.
    pub(crate) fn is_proved_arm_joining_at(self, node: usize) -> bool {
        self.fallthrough_proved && self.joins_at(node)
    }

    /// The switch's branch instruction, when this frame's proof named one.
    pub(crate) fn branch_bci(self) -> Option<u32> {
        self.branch_bci
    }

    /// Whether the switch's fallthrough proof made this frame an arm of one.
    pub(crate) fn fallthrough_proved(self) -> bool {
        self.fallthrough_proved
    }

    /// The join node this arm breaks to, when the frame states one.
    pub(crate) fn join_node(self) -> Option<usize> {
        self.join_node
    }
}

/// The transfer that closes one recovered run, when its last region is a proved transfer.
///
/// A run is the ordered regions one walk call proved ([`super::Run`]); the composition decisions
/// read its *tail* because that is where a two-edge body's jump leaves. Reading it here keeps the
/// question identical at every site that asks it.
pub(crate) fn run_transfer(run: &[Region]) -> Option<RegionTransfer> {
    run.last().and_then(Region::transfer_leaf)
}

/// The single block of a run that is nothing but that one block: `[Straight { blocks: [b] }]`.
///
/// An arm whose whole walk proved one `goto` block is the bridge edge a re-elected join (or a
/// gateway break) no longer stands for. Both the continue-ladder election and the shared-latch
/// election ask this, and the answer is one reading of the tree rather than a pattern match at
/// each site.
pub(crate) fn run_single_block(run: &[Region]) -> Option<&CanonicalBlockId> {
    let [Region::Straight { blocks }] = run else {
        return None;
    };
    match blocks.as_slice() {
        [block] => Some(block),
        _ => None,
    }
}

/// Whether one adjacent pair of a recovered run is the exact gateway break: the single `gateway`
/// block, immediately followed by a break out of `parent`.
///
/// A nested-loop side exit states its jump as the gateway's own block and then the transfer that
/// leaves the parent. Both the proof and the verification read that same pair, so the reading is
/// stated once here rather than spelled twice.
pub(crate) fn is_gateway_break_pair(
    pair: &[Region],
    gateway: &CanonicalBlockId,
    parent: &CanonicalBlockId,
) -> bool {
    let [owner, transfer] = pair else {
        return false;
    };
    matches!(owner, Region::Straight { blocks } if blocks == std::slice::from_ref(gateway))
        && transfer
            .transfer_leaf()
            .is_some_and(|transfer| transfer.kind.is_break() && transfer.kind.loop_header() == Some(parent))
}