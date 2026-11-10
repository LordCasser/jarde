//! The per-candidate consolidation of the facts a recovered [`Region`] tree states (A4.1).
//!
//! [`RegionSummary`] is a **derived view**, not a second ownership ledger. Its owned set is
//! [`Region::blocks`]' own folded semantics, its boundaries are the continuation fields the
//! variants already carry (`If`'s join, `Loop`'s exit, `Guard`'s plan, `SwitchBreak`'s join), and
//! the method-level account stays the walk's own enforced check ([`super::overlapping_owner`] plus
//! the uncovered-blocks closure of [`super::recover`]). Nothing here refuses: the view is computed
//! and then *proved* against the canonical CFG/SSA the same run produced, and the proof's failures
//! come back as [`RegionSummaryMismatch`] values for the caller to assert on. The composition
//! contracts of the next increments consume the view; until one of them wires a production call
//! site, the module is compiled for the tests that pin the proof only.

use jarde_jvm::method_ir::{CanonicalBlockId, PhiInput, Slot, SsaTable, ValueId};
use jarde_reader::budget::{Budget, CountedBudgetDimension};

use crate::decode::Operations;
use crate::facts::Operation;
use crate::stop::{StopReason, charge, poll};

pub(crate) use super::composition::RegionTransfer;

#[cfg(test)]
pub(crate) use super::composition::RegionTransferKind;

use super::Region;

#[cfg(test)]
use std::collections::BTreeSet;

#[cfg(test)]
use jarde_jvm::method_ir::{CanonicalCfg, CanonicalEdgeKind, Definition};

#[cfg(test)]
use super::{LoopForm, Recovered};

/// What one candidate region consolidates, as its tree states it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionSummary {
    /// The blocks the candidate claims, in the order the method runs them — exactly
    /// [`Region::blocks`]' own semantics, aliases folded the way the completed tree is checked.
    pub(crate) owned: Vec<CanonicalBlockId>,
    /// Where control enters the candidate: the first owned block. `None` for a pure-edge
    /// candidate (`LoopBreak`/`LoopContinue`/`SwitchBreak`) or an empty run.
    pub(crate) entry: Option<CanonicalBlockId>,
    /// The normal continuations the candidate's own variant states — an `if`'s or switch's join,
    /// a loop's exit, a guard's plan join. The per-edge handoff between sibling candidates is
    /// composition, which the next increments own; this is the boundary the tree itself names.
    pub(crate) exits: Vec<CanonicalBlockId>,
    /// Every structure transfer the subtree holds, with the destination its edge names.
    pub(crate) transfers: Vec<RegionTransfer>,
    /// Every owned block whose last instruction leaves the method or raises, in method order.
    pub(crate) terminals: Vec<RegionTerminal>,
    /// Every exception clause the subtree binds, in binding order: a `try`'s clauses in
    /// exception-table order, a guard's own whole-statement clause.
    pub(crate) exceptions: Vec<RegionExceptionBoundary>,
    /// What the SSA states at the candidate's own boundary.
    pub(crate) boundary: RegionBoundaryValues,
    /// The effects the candidate's own presentation performs implicitly, as its variant states
    /// them.
    pub(crate) effects: RegionEffects,
}

/// One terminal block and how it leaves the method.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionTerminal {
    pub(crate) block: CanonicalBlockId,
    pub(crate) kind: RegionTerminalKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegionTerminalKind {
    /// The block's last instruction is a `return`: the method exits here.
    Return,
    /// The block's last instruction is an `athrow`: control raises here.
    Throw,
}

/// One exception clause the tree binds, as its own statement records it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionExceptionBoundary {
    /// The canonical block the clause's handler entry is.
    pub(crate) handler: CanonicalBlockId,
    /// The types the clause names, or the proved single catch-all.
    pub(crate) types: crate::guard::CatchTypes,
    /// The slot the handler's first instruction stores the caught exception into.
    pub(crate) parameter: u16,
}

/// The SSA facts at one candidate's boundary, read from the same run's table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionBoundaryValues {
    /// The bindings control arrives at the entry block with — the declarations that must exist
    /// when the candidate begins.
    pub(crate) entry: Vec<(Slot, ValueId)>,
    /// Per stated exit target: what the target is entered with, and the entry phis that merge
    /// the boundary's edges.
    pub(crate) exits: Vec<RegionExitValues>,
}

/// The SSA state at one stated exit target, outside the candidate's ownership.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionExitValues {
    pub(crate) target: CanonicalBlockId,
    /// The bindings control arrives at the target with.
    pub(crate) arrival: Vec<(Slot, ValueId)>,
    /// The target's entry phis: the merges this boundary performs, one operand per participating
    /// predecessor — the copies the boundary's edges must hand over.
    pub(crate) merges: Vec<RegionPhiMerge>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionPhiMerge {
    pub(crate) slot: Slot,
    pub(crate) value: ValueId,
    pub(crate) inputs: Vec<PhiInput>,
}

/// The effects the candidate's own presentation performs implicitly, as its variant states them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionEffects {
    /// The physical cleanup a guard's source construct replaced: the statement performs these
    /// instructions' effects (the closes, the monitor exit) by being what it is.
    pub(crate) elided_cleanup_bcis: Vec<u32>,
    /// The transfers the structure itself represents instead of a statement (a loop's gateways).
    pub(crate) gateway_origin_bcis: Vec<u32>,
}

/// One way the derived view failed to match the canonical CFG/SSA or the walk's own accounts.
///
/// The A4.1 reconciliation is a **stage proof**, not a per-run self-check: it is run by the fixture
/// suite that pins the view, never by an ordinary recovery. So the proof and this vocabulary are
/// compiled for the tests that assert on them, while the view itself ([`RegionSummary`] and its
/// [`RegionSummary::derive`]) is production code the composition contracts consume.
#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionSummaryMismatch {
    /// One candidate's `owned` names the same physical block twice.
    DuplicateOwned { block: CanonicalBlockId },
    /// Two top-level candidates claim one physical block.
    OverlappingOwned { block: CanonicalBlockId },
    /// A live canonical block no candidate owns.
    MissingOwnership { block: CanonicalBlockId },
    /// The walk's claim record names an index no candidate owns.
    ClaimedOutsideOwned { index: usize },
    /// The candidate owner counts disagree with the derived owned sets.
    OwnerCountsDisagree,
    /// A stated exit is not a live canonical block.
    ExitNotLive { exit: CanonicalBlockId },
    /// A stated exit is inside the candidate's own ownership.
    ExitOwnedByCandidate { exit: CanonicalBlockId },
    /// No owned block transfers normally to the stated exit.
    ExitUnreached { exit: CanonicalBlockId },
    /// The transfer's source instruction is not the origin of exactly one owned block.
    TransferSourceUnresolved { source_bci: u32 },
    /// No normal edge leaves the transfer's source block.
    TransferEdgeMissing { source_bci: u32 },
    /// The transfer names a loop the tree does not hold.
    TransferLoopMissing { loop_header: CanonicalBlockId },
    /// The transfer's stated destination is not among its source block's normal successors.
    TransferDestinationMissing {
        source_bci: u32,
        destination: CanonicalBlockId,
    },
    /// A terminal block's outgoing edges contradict how it leaves: a `return` or a raise whose
    /// edges are not exception edges.
    ///
    /// Neither a `return` nor an `athrow` can itself raise, so no *normal* edge may leave the
    /// block. An exception edge still can: the block's last instruction ends it, but the block
    /// stands for a contiguous run of instructions whose earlier members may throw, so a `return`
    /// inside a protected range owns exactly the handler edge those instructions take
    /// (`TestBreakInLoop5.test`'s `lreturn` at BCI 123 in the range `[0, 123)`).
    TerminalLeavesUnexpectedly { block: CanonicalBlockId },
    /// A variant-stated return block the terminals do not record as a return.
    StatedReturnUnrecorded { block: CanonicalBlockId },
    /// The canonical table does not state the clause's rows as the boundary records them.
    ExceptionRowsMismatch { handler: CanonicalBlockId },
    /// The clauses are not in exception-table order.
    ///
    /// The JVM dispatches in table order, so that is the order the text must write and the order a
    /// combination decision must keep when it matches a clause against the row owning it. Each
    /// clause's rows still match one-for-one in a reordered view, which is why the row agreement
    /// above cannot catch it: the order has to be proved on its own.
    ExceptionClauseOrderMismatch { handler: CanonicalBlockId },
    /// A row of this boundary protects a block the candidate does not own.
    ExceptionRowProtectsUnowned {
        handler: CanonicalBlockId,
        block: CanonicalBlockId,
    },
    /// The target's arrival state does not name the boundary's recorded merge.
    MergeArrivalMismatch { block: CanonicalBlockId, slot: Slot },
    /// Along one normal edge, the successor's entry state disagrees with the predecessor's exit
    /// state for a slot neither merges.
    BoundaryEdgeDisagreement {
        from: CanonicalBlockId,
        to: CanonicalBlockId,
        slot: Slot,
    },
    /// One edge's outgoing value is not among the merge operands of the target it enters.
    MergeOperandMissing {
        to: CanonicalBlockId,
        slot: Slot,
        value: ValueId,
    },
    /// An effect the presentation performs implicitly is not an instruction of an owned block.
    EffectBciUnowned { bci: u32 },
}

impl RegionSummary {
    /// Derive one candidate's summary from its tree, the SSA and the decode of the same run.
    pub(crate) fn derive(
        region: &Region,
        ssa: &SsaTable,
        operations: &Operations,
        budget: &mut Budget,
    ) -> Result<Self, StopReason> {
        poll(budget, None)?;
        charge(budget, CountedBudgetDimension::AnalysisSteps, 1, None)?;
        let owned: Vec<CanonicalBlockId> = region.blocks().into_iter().cloned().collect();
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(owned.len()).unwrap_or(u64::MAX),
            owned.first().map(|block| block.bci()),
        )?;

        let mut transfers = Vec::new();
        collect_transfers(region, &mut transfers);
        let mut exceptions = Vec::new();
        collect_exceptions(region, &mut exceptions);
        let mut gateway_origin_bcis = Vec::new();
        collect_gateway_origins(region, &mut gateway_origin_bcis);
        let mut elided_cleanup_bcis = Vec::new();
        region.elided_cleanup_bcis(&mut elided_cleanup_bcis);

        let mut terminals = Vec::new();
        for block in &owned {
            poll(budget, Some(block.bci()))?;
            charge(budget, CountedBudgetDimension::AnalysisSteps, 1, Some(block.bci()))?;
            let names = ssa
                .block(block)
                .expect("an owned block is live and has SSA names");
            let Some(last) = names.instructions().last() else {
                continue;
            };
            let kind = match operations.get(last.bci()) {
                Some(Operation::Return) => RegionTerminalKind::Return,
                Some(Operation::Throw) => RegionTerminalKind::Throw,
                _ => continue,
            };
            terminals.push(RegionTerminal {
                block: block.clone(),
                kind,
            });
        }

        let exits = stated_exits(region);
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(exits.len()).unwrap_or(u64::MAX),
            exits.first().map(|exit| exit.bci()),
        )?;
        // Where control enters: the variant's own entry fact — a loop's header — or the first
        // block the run reaches. The listing order is not the answer for a do-while: when the
        // body owns its header, the latch test is listed before the body's blocks.
        let entry = match region {
            Region::Loop { header, .. } => Some(header.clone()),
            Region::EntrySplitLoop { entry, .. } => Some(entry.clone()),
            _ => owned.first().cloned(),
        };
        let boundary = RegionBoundaryValues {
            entry: entry
                .as_ref()
                .map(|block| {
                    ssa.block(block)
                        .expect("the entry is live and has SSA names")
                        .entry()
                        .to_vec()
                })
                .unwrap_or_default(),
            exits: exits
                .iter()
                .map(|target| exit_values(target, ssa))
                .collect(),
        };
        Ok(Self {
            owned,
            entry,
            exits,
            transfers,
            terminals,
            exceptions,
            boundary,
            effects: RegionEffects {
                elided_cleanup_bcis,
                gateway_origin_bcis,
            },
        })
    }
}

/// Derive the view of every top-level candidate of one completed walk, in method order.
///
/// This is the production entry the composition contracts read ([`super::Recovered::summaries`]).
/// It derives from the tree the walk published and from the SSA/decode of the same run; it
/// decides nothing and refuses nothing, so a candidate the walk kept stays exactly as walked.
pub(crate) fn summaries_of(
    regions: &[Region],
    ssa: &SsaTable,
    operations: &Operations,
    budget: &mut Budget,
) -> Result<Vec<RegionSummary>, StopReason> {
    let mut summaries = Vec::with_capacity(regions.len());
    for region in regions {
        poll(budget, None)?;
        summaries.push(RegionSummary::derive(region, ssa, operations, budget)?);
    }
    Ok(summaries)
}

/// The normal continuation targets the candidate's own variant states.
fn stated_exits(region: &Region) -> Vec<CanonicalBlockId> {
    match region {
        Region::If { join, .. } | Region::CompoundIf { join, .. } => {
            join.iter().cloned().collect()
        }
        Region::SharedTailEarlyReturn { join, .. } => vec![join.clone()],
        Region::Switch { join, .. } | Region::StringSwitch { join, .. } => {
            join.iter().cloned().collect()
        }
        Region::Loop { exit, .. } => exit.iter().cloned().collect(),
        Region::EntrySplitLoop { exit, .. } => vec![exit.clone()],
        Region::Guard { plan, .. } => plan.join().cloned().into_iter().collect(),
        // A sequence's and a try's normal continuation is their lexical successor, a straight
        // run's is the structure that ends it, a value graph's boundary is its own consumer
        // block, and a fallback quotes — none of these names a target block of its own.
        Region::Sequence { .. }
        | Region::Straight { .. }
        | Region::ShortCircuitValue { .. }
        | Region::TwoExitReturn { .. }
        | Region::LoopBreak { .. }
        | Region::LoopContinue { .. }
        | Region::SwitchBreak { .. }
        | Region::Fallback { .. }
        | Region::Try { .. } => Vec::new(),
    }
}

fn exit_values(target: &CanonicalBlockId, ssa: &SsaTable) -> RegionExitValues {
    let names = ssa
        .block(target)
        .expect("a stated exit is a live block and has SSA names");
    RegionExitValues {
        target: target.clone(),
        arrival: names.entry().to_vec(),
        merges: ssa
            .phis()
            .iter()
            .filter(|phi| phi.block() == target)
            .map(|phi| RegionPhiMerge {
                slot: phi.slot(),
                value: phi.value(),
                inputs: phi.inputs().to_vec(),
            })
            .collect(),
    }
}

/// Every region the tree holds under this one, in method order.
fn for_each_child<'a>(region: &'a Region, visit: &mut impl FnMut(&'a Region)) {
    for child in region.composed_children() {
        visit(child);
    }
}

fn collect_transfers(region: &Region, out: &mut Vec<RegionTransfer>) {
    if let Some(transfer) = region.transfer_leaf() {
        out.push(transfer);
    }
    for_each_child(region, &mut |child| collect_transfers(child, out));
}

fn collect_exceptions(region: &Region, out: &mut Vec<RegionExceptionBoundary>) {
    // The clauses, in exception-table order; the walk presents them in the table's own order and a
    // guard's whole-statement clause is the plan's one record. Both are read once, here, from the
    // owner of that reading ([`Region::exception_clauses`]).
    for clause in region.exception_clauses() {
        out.push(RegionExceptionBoundary {
            handler: clause.handler,
            types: clause.types,
            parameter: clause.parameter,
        });
    }
    for_each_child(region, &mut |child| collect_exceptions(child, out));
}

fn collect_gateway_origins(region: &Region, out: &mut Vec<u32>) {
    if let Region::Loop {
        gateway_origins, ..
    } = region
    {
        out.extend(gateway_origins.iter().copied());
    }
    for_each_child(region, &mut |child| collect_gateway_origins(child, out));
}

/// Every loop the tree holds, outermost first.
#[cfg(test)]
fn collect_loops<'a>(region: &'a Region, out: &mut Vec<&'a Region>) {
    if matches!(region, Region::Loop { .. }) {
        out.push(region);
    }
    for_each_child(region, &mut |child| collect_loops(child, out));
}

/// Every return block a variant states itself (`TwoExitReturn`'s two exits).
#[cfg(test)]
fn collect_stated_returns(region: &Region, out: &mut Vec<CanonicalBlockId>) {
    if let Region::TwoExitReturn {
        true_return,
        false_return,
        ..
    } = region
    {
        out.push(true_return.clone());
        out.push(false_return.clone());
    }
    for_each_child(region, &mut |child| collect_stated_returns(child, out));
}

/// The value a state names, followed to the end of its replacement chain: a phi the pass removed
/// as trivial names the merge it stood for through its own `replaced_by`.
#[cfg(test)]
fn resolved(ssa: &SsaTable, value: ValueId) -> ValueId {
    let mut current = value;
    while let Some(next) = ssa.value(current).replaced_by() {
        current = next;
    }
    current
}

/// Whether one canonical block's physical span holds an instruction: a block stands for the
/// contiguous original runs from its own start to its end, so a terminal transfer or an elided
/// close at a mid-block BCI belongs to exactly the block whose span covers it.
#[cfg(test)]
fn block_holds(canonical: &CanonicalCfg, id: &CanonicalBlockId, bci: u32) -> bool {
    canonical
        .blocks()
        .iter()
        .find(|entry| entry.id() == id)
        .is_some_and(|entry| entry.id().bci() <= bci && bci < entry.end_bci())
}

/// Derive every top-level candidate's summary and prove the whole method's view against the
/// canonical CFG/SSA and the walk's own accounts. An empty mismatch list is the A4.1 proof: no
/// missing or overlapping ownership, every stated exit reached from inside its candidate, every
/// transfer's destination resolved and edged, every boundary's rows and SSA handoffs consistent.
#[cfg(test)]
pub(crate) fn reconcile_method_summaries(
    recovered: &Recovered,
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    operations: &Operations,
    budget: &mut Budget,
) -> Result<Vec<RegionSummaryMismatch>, StopReason> {
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(canonical.edges().len()).unwrap_or(u64::MAX),
        None,
    )?;
    let mut mismatches = Vec::new();
    let mut summaries = Vec::new();
    for region in &recovered.regions {
        summaries.push(RegionSummary::derive(region, ssa, operations, budget)?);
    }

    reconcile_ownership(recovered, canonical, &summaries, budget, &mut mismatches)?;
    reconcile_exits(canonical, &summaries, &mut mismatches);
    reconcile_transfers(recovered, canonical, &summaries, &mut mismatches);
    reconcile_terminals(canonical, &summaries, &mut mismatches);
    reconcile_stated_returns(&summaries, recovered, &mut mismatches);
    reconcile_exceptions(canonical, &summaries, &mut mismatches);
    reconcile_exception_order(canonical, recovered, &mut mismatches);
    reconcile_boundary_edges(canonical, ssa, &summaries, &mut mismatches);
    reconcile_effects(canonical, &summaries, &mut mismatches);
    Ok(mismatches)
}

/// The ownership proof: each candidate's owned set is duplicate-free, the candidates partition the
/// live canonical blocks exactly, the walk's claim record names only owned blocks, and the
/// candidate owner counts agree where the candidate collector applies. This re-derives the
/// enforced check ([`super::overlapping_owner`] and the uncovered closure) as an A4.1 fact; it
/// replaces no enforced check.
#[cfg(test)]
fn reconcile_ownership(
    recovered: &Recovered,
    canonical: &CanonicalCfg,
    summaries: &[RegionSummary],
    budget: &mut Budget,
    mismatches: &mut Vec<RegionSummaryMismatch>,
) -> Result<(), StopReason> {
    let mut union = BTreeSet::new();
    for summary in summaries {
        let mut owned = BTreeSet::new();
        for block in &summary.owned {
            if !owned.insert(block) {
                mismatches.push(RegionSummaryMismatch::DuplicateOwned {
                    block: block.clone(),
                });
            }
            if !union.insert(block) {
                mismatches.push(RegionSummaryMismatch::OverlappingOwned {
                    block: block.clone(),
                });
            }
        }
    }
    for block in canonical
        .blocks()
        .iter()
        .map(|block| block.id())
        .filter(|id| !canonical.unreachable().contains(id))
    {
        if !union.contains(block) {
            mismatches.push(RegionSummaryMismatch::MissingOwnership {
                block: block.clone(),
            });
        }
    }
    for index in &recovered.claimed {
        let Some(block) = canonical.blocks().get(*index) else {
            mismatches.push(RegionSummaryMismatch::ClaimedOutsideOwned { index: *index });
            continue;
        };
        if !union.contains(block.id()) {
            mismatches.push(RegionSummaryMismatch::ClaimedOutsideOwned { index: *index });
        }
    }
    // The candidate collector's own rule (`candidate_region_owner_counts`): where it applies, its
    // counts must agree with the derived sets. A tree holding a guard or a fallback has no
    // candidate inventory — `Region::blocks` is the single ownership rule there.
    if let Some(counts) =
        super::candidate_region_owner_counts(&recovered.regions, budget, true, true)?
    {
        if counts.values().any(|count| *count != 1)
            || counts.len() != union.len()
            || !counts.keys().all(|block| union.contains(block))
        {
            mismatches.push(RegionSummaryMismatch::OwnerCountsDisagree);
        }
    }
    Ok(())
}

#[cfg(test)]
fn reconcile_exits(
    canonical: &CanonicalCfg,
    summaries: &[RegionSummary],
    mismatches: &mut Vec<RegionSummaryMismatch>,
) {
    let live: BTreeSet<&CanonicalBlockId> = canonical
        .blocks()
        .iter()
        .map(|block| block.id())
        .filter(|id| !canonical.unreachable().contains(id))
        .collect();
    for summary in summaries {
        for exit in &summary.exits {
            if !live.contains(exit) {
                mismatches.push(RegionSummaryMismatch::ExitNotLive { exit: exit.clone() });
                continue;
            }
            if summary.owned.contains(exit) {
                mismatches.push(RegionSummaryMismatch::ExitOwnedByCandidate {
                    exit: exit.clone(),
                });
            }
            let reached = canonical.edges().iter().any(|edge| {
                edge.kind() == CanonicalEdgeKind::Normal
                    && edge.to() == exit
                    && summary.owned.contains(edge.from())
            });
            if !reached {
                mismatches.push(RegionSummaryMismatch::ExitUnreached { exit: exit.clone() });
            }
        }
    }
}

#[cfg(test)]
fn reconcile_transfers(
    recovered: &Recovered,
    canonical: &CanonicalCfg,
    summaries: &[RegionSummary],
    mismatches: &mut Vec<RegionSummaryMismatch>,
) {
    let mut loops = Vec::new();
    for region in &recovered.regions {
        collect_loops(region, &mut loops);
    }
    let loop_headers: BTreeSet<&CanonicalBlockId> = loops
        .iter()
        .filter_map(|region| match region {
            Region::Loop { header, .. } => Some(header),
            _ => None,
        })
        .collect();
    for summary in summaries {
        for transfer in &summary.transfers {
            let sources: Vec<&CanonicalBlockId> = summary
                .owned
                .iter()
                .filter(|block| block_holds(canonical, block, transfer.source_bci))
                .collect();
            let Some(source) = sources
                .first()
                .copied()
                .filter(|_| sources.len() == 1)
            else {
                mismatches.push(RegionSummaryMismatch::TransferSourceUnresolved {
                    source_bci: transfer.source_bci,
                });
                continue;
            };
            let successors: Vec<&CanonicalBlockId> = canonical
                .edges()
                .iter()
                .filter(|edge| {
                    edge.kind() == CanonicalEdgeKind::Normal && edge.from() == source
                })
                .map(|edge| edge.to())
                .collect();
            if successors.is_empty() {
                mismatches.push(RegionSummaryMismatch::TransferEdgeMissing {
                    source_bci: transfer.source_bci,
                });
                continue;
            }
            let named_loop = |header: &CanonicalBlockId| {
                loops
                    .iter()
                    .copied()
                    .find(|region| matches!(region, Region::Loop { header: own, .. } if own == header))
            };
            match &transfer.kind {
                RegionTransferKind::SwitchBreak { join, .. } => {
                    if !successors.contains(&join) {
                        mismatches.push(RegionSummaryMismatch::TransferDestinationMissing {
                            source_bci: transfer.source_bci,
                            destination: join.clone(),
                        });
                    }
                }
                RegionTransferKind::Break { loop_header } => {
                    if !loop_headers.contains(loop_header) {
                        mismatches.push(RegionSummaryMismatch::TransferLoopMissing {
                            loop_header: loop_header.clone(),
                        });
                        continue;
                    }
                    // A break's destination is the named loop's exit when the loop states one;
                    // an endless body's breaks are its exits, which no field names.
                    if let Some(Region::Loop {
                        exit: Some(exit), ..
                    }) = named_loop(loop_header)
                    {
                        if !successors.contains(&exit) {
                            mismatches.push(RegionSummaryMismatch::TransferDestinationMissing {
                                source_bci: transfer.source_bci,
                                destination: exit.clone(),
                            });
                        }
                    }
                }
                RegionTransferKind::Continue { loop_header } => {
                    if !loop_headers.contains(loop_header) {
                        mismatches.push(RegionSummaryMismatch::TransferLoopMissing {
                            loop_header: loop_header.clone(),
                        });
                        continue;
                    }
                    if let Some(Region::Loop {
                        header,
                        tests,
                        form,
                        for_header,
                        ..
                    }) = named_loop(loop_header)
                    {
                        // The proved continue target: a `for` header's own update when it owns
                        // one, otherwise the latch test of a do-while or the header itself.
                        let destination = for_header
                            .as_ref()
                            .map(|header| &header.update_block)
                            .unwrap_or(match form {
                                LoopForm::DoWhile => &tests[0].0,
                                LoopForm::While | LoopForm::Endless => header,
                            });
                        if !successors.contains(&destination) {
                            mismatches.push(RegionSummaryMismatch::TransferDestinationMissing {
                                source_bci: transfer.source_bci,
                                destination: destination.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
fn reconcile_terminals(
    canonical: &CanonicalCfg,
    summaries: &[RegionSummary],
    mismatches: &mut Vec<RegionSummaryMismatch>,
) {
    for summary in summaries {
        for terminal in &summary.terminals {
            let contradicts = match terminal.kind {
                // A `return` leaves the method, so nothing may continue *normally* after it. The
                // exception edges an earlier instruction of the same block takes are not a
                // continuation: the block's own span is what those instructions protect.
                RegionTerminalKind::Return | RegionTerminalKind::Throw => {
                    canonical.edges().iter().any(|edge| {
                        edge.from() == &terminal.block
                            && !matches!(edge.kind(), CanonicalEdgeKind::Exception { .. })
                    })
                }
            };
            if contradicts {
                mismatches.push(RegionSummaryMismatch::TerminalLeavesUnexpectedly {
                    block: terminal.block.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
fn reconcile_stated_returns(
    summaries: &[RegionSummary],
    recovered: &Recovered,
    mismatches: &mut Vec<RegionSummaryMismatch>,
) {
    let mut stated = Vec::new();
    for region in &recovered.regions {
        collect_stated_returns(region, &mut stated);
    }
    for block in stated {
        let recorded = summaries.iter().any(|summary| {
            summary.terminals.iter().any(|terminal| {
                terminal.block == block && terminal.kind == RegionTerminalKind::Return
            })
        });
        if !recorded {
            mismatches.push(RegionSummaryMismatch::StatedReturnUnrecorded {
                block: block.clone(),
            });
        }
    }
}

#[cfg(test)]
fn reconcile_exceptions(
    canonical: &CanonicalCfg,
    summaries: &[RegionSummary],
    mismatches: &mut Vec<RegionSummaryMismatch>,
) {
    for summary in summaries {
        for boundary in &summary.exceptions {
            let rows: Vec<_> = canonical
                .handler_rows()
                .iter()
                .filter(|row| row.handler() == Some(&boundary.handler))
                .collect();
            let rows_match = match &boundary.types {
                crate::guard::CatchTypes::Named(types) => {
                    rows.len() == types.len()
                        && types.iter().all(|named| {
                            rows.iter()
                                .filter(|row| row.catch_type_index() == Some(*named))
                                .count()
                                == 1
                        })
                }
                crate::guard::CatchTypes::ProvenThrowable => {
                    rows.len() == 1 && rows[0].catch_type_index().is_none()
                }
            };
            if !rows_match {
                mismatches.push(RegionSummaryMismatch::ExceptionRowsMismatch {
                    handler: boundary.handler.clone(),
                });
            }
            for row in &rows {
                for block in row.protected() {
                    if !summary.owned.contains(block) {
                        mismatches.push(RegionSummaryMismatch::ExceptionRowProtectsUnowned {
                            handler: boundary.handler.clone(),
                            block: block.clone(),
                        });
                    }
                }
            }
        }
    }
}

/// The clause-order proof (A4.2): every statement states its clauses in exception-table order.
///
/// The row agreement above matches each clause against the rows reaching its handler one for one,
/// and a reordered clause list still matches: the same rows, the same handlers, the same types. So
/// the order has to be proved on its own. It is the order the JVM dispatches in, which is
/// therefore the order the text must write — and the order a combination decision must preserve
/// when it reads a clause against the row that owns it.
///
/// The order is read from the tree, and the table is what it must agree with. A statement holds at
/// least two clauses for the fact to have content, and a clause whose rows the table does not
/// resolve is the row agreement's business, not this proof's.
#[cfg(test)]
fn reconcile_exception_order(
    canonical: &CanonicalCfg,
    recovered: &Recovered,
    mismatches: &mut Vec<RegionSummaryMismatch>,
) {
    fn visit(
        region: &Region,
        canonical: &CanonicalCfg,
        mismatches: &mut Vec<RegionSummaryMismatch>,
    ) {
        let clauses = region.exception_clauses();
        if clauses.len() > 1 {
            let first_ordinal = |handler: &CanonicalBlockId| {
                canonical
                    .handler_rows()
                    .iter()
                    .filter(|row| row.handler() == Some(handler))
                    .map(|row| row.ordinal())
                    .min()
            };
            let mut previous: Option<u32> = None;
            for clause in &clauses {
                let ordinal = first_ordinal(&clause.handler);
                if let (Some(previous), Some(ordinal)) = (previous, ordinal)
                    && ordinal <= previous
                {
                    mismatches.push(RegionSummaryMismatch::ExceptionClauseOrderMismatch {
                        handler: clause.handler.clone(),
                    });
                }
                if ordinal.is_some() {
                    previous = ordinal;
                }
            }
        }
        for child in region.composed_children() {
            visit(child, canonical, mismatches);
        }
    }
    for region in &recovered.regions {
        visit(region, canonical, mismatches);
    }
}

/// The SSA boundary proof: every merge a stated exit records is what the target's arrival state
/// names, and along every normal edge the successor's entry state is what the predecessor's exit
/// state hands over — the target's own phi naming the merge whose operands carry the edge's copy.
#[cfg(test)]
fn reconcile_boundary_edges(
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    summaries: &[RegionSummary],
    mismatches: &mut Vec<RegionSummaryMismatch>,
) {
    for summary in summaries {
        for exit in &summary.boundary.exits {
            for merge in &exit.merges {
                let arrival_names_merge = exit.arrival.iter().any(|(slot, value)| {
                    slot == &merge.slot && resolved(ssa, *value) == resolved(ssa, merge.value)
                });
                if !arrival_names_merge {
                    mismatches.push(RegionSummaryMismatch::MergeArrivalMismatch {
                        block: exit.target.clone(),
                        slot: merge.slot,
                    });
                }
            }
        }
    }
    for edge in canonical.edges() {
        if edge.kind() != CanonicalEdgeKind::Normal {
            continue;
        }
        let (Some(from), Some(to)) = (ssa.block(edge.from()), ssa.block(edge.to())) else {
            // An endpoint the entry cannot reach has no SSA names; the live boundary agreement
            // is what this proof is about.
            continue;
        };
        for (slot, out_value) in from.exit() {
            let Some((_, in_value)) = to.entry().iter().find(|(entry_slot, _)| entry_slot == slot)
            else {
                continue;
            };
            if resolved(ssa, *out_value) == resolved(ssa, *in_value) {
                continue;
            }
            // The target's own merge for this slot: the edge's copy must be one of its operands.
            let merge = to
                .entry()
                .iter()
                .find(|(entry_slot, value)| {
                    entry_slot == slot
                        && matches!(
                            ssa.value(*value).def(),
                            Definition::Phi { block, .. } if block == edge.to()
                        )
                })
                .and_then(|(_, value)| {
                    ssa.phis()
                        .iter()
                        .find(|phi| phi.block() == edge.to() && phi.value() == *value)
                });
            match merge {
                Some(phi) if phi.inputs().iter().any(|input| match input {
                    PhiInput::Value(operand) => {
                        resolved(ssa, *operand) == resolved(ssa, *out_value)
                    }
                    // The edge hands the merge's own value back, which this disagreement
                    // branch cannot be: the arrival would already name it.
                    PhiInput::Itself => false,
                }) => {}
                Some(_) => mismatches.push(RegionSummaryMismatch::MergeOperandMissing {
                    to: edge.to().clone(),
                    slot: *slot,
                    value: *out_value,
                }),
                None => mismatches.push(RegionSummaryMismatch::BoundaryEdgeDisagreement {
                    from: edge.from().clone(),
                    to: edge.to().clone(),
                    slot: *slot,
                }),
            }
        }
    }
}

#[cfg(test)]
fn reconcile_effects(
    canonical: &CanonicalCfg,
    summaries: &[RegionSummary],
    mismatches: &mut Vec<RegionSummaryMismatch>,
) {
    for summary in summaries {
        for bci in summary
            .effects
            .elided_cleanup_bcis
            .iter()
            .chain(summary.effects.gateway_origin_bcis.iter())
        {
            if !summary
                .owned
                .iter()
                .any(|block| block_holds(canonical, block, *bci))
            {
                mismatches.push(RegionSummaryMismatch::EffectBciUnowned { bci: *bci });
            }
        }
    }
}
