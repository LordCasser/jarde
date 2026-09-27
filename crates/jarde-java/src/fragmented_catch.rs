//! A local proof for one nested named catch inside a loop's fragmented outer catch.
//! It reads the decode, Canonical CFG and SSA of the same method analysis. No graph is rewritten.

use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{
    CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind, Definition, Slot, SsaTable, ValueId,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::MethodCodeFacts;

use crate::normal_flow::NormalFlowView;
use crate::stop::{StopReason, charge, poll};

#[derive(Clone, Debug)]
pub(crate) struct FragmentedCatch {
    pub(crate) outer_rows: Vec<u32>,
    pub(crate) inner_row: u32,
    pub(crate) outer_start: u32,
    pub(crate) inner_start: u32,
    /// The proved terminal goto from the inner protected body to its normal continuation.
    pub(crate) inner_exit_bci: u32,
    pub(crate) outer_join: CanonicalBlockId,
    pub(crate) update: CanonicalBlockId,
    /// The proved terminal goto from the update block back to this loop's header.
    pub(crate) update_transfer_bci: u32,
    pub(crate) loop_header: usize,
    /// Ordinary successors of exception-only blocks which enter the natural loop.
    pub(crate) loop_entries: BTreeSet<(usize, usize)>,
    /// Exception-only blocks owned by the lexical loop, never by its natural loop fact.
    pub(crate) exceptional_blocks: BTreeSet<usize>,
}

impl FragmentedCatch {
    pub(crate) fn is_outer_row(&self, ordinal: u32) -> bool {
        self.outer_rows.contains(&ordinal)
    }

    pub(crate) fn supplemental_scope(&self, header: usize) -> Option<&BTreeSet<usize>> {
        (self.loop_header == header).then_some(&self.exceptional_blocks)
    }
}

/// `None` is a refusal, including a row or value that the current run cannot map. The caller
/// retains its existing conservative fallback. Every scan is charged or polled before use.
pub(crate) fn prove(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    code: &MethodCodeFacts,
    budget: &mut Budget,
) -> Result<Option<FragmentedCatch>, StopReason> {
    let rows = &code.exception_handlers;
    if rows.len() < 3 || rows.len() > 12 || canonical.blocks().len() > 128 {
        return Ok(None);
    }
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        rows.len() as u64,
        None,
    )?;
    if rows.iter().any(|row| row.catch_type_index.is_none())
        || canonical
            .blocks()
            .iter()
            .any(|block| !block.id().path().is_empty())
        || !canonical.unreachable().is_empty()
    {
        return Ok(None);
    }
    let mut groups: BTreeMap<(u32, u16), Vec<_>> = BTreeMap::new();
    for row in rows {
        groups
            .entry((row.handler_bci, row.catch_type_index.expect("named row")))
            .or_default()
            .push(row);
    }
    let Some((outer_key, mut outer)) = groups
        .iter()
        .find(|(_, rows)| rows.len() >= 2)
        .map(|(key, rows)| (*key, rows.clone()))
    else {
        return Ok(None);
    };
    if groups.len() != 2 || groups.values().filter(|rows| rows.len() >= 2).count() != 1 {
        return Ok(None);
    }
    let Some(inner) = rows.iter().find(|row| row.handler_bci != outer_key.0) else {
        return Ok(None);
    };
    if groups
        .get(&(
            inner.handler_bci,
            inner.catch_type_index.expect("named row"),
        ))
        .is_none_or(|rows| rows.len() != 1)
    {
        return Ok(None);
    }
    outer.sort_by_key(|row| row.start_bci);
    if outer.iter().any(|row| row.start_bci >= row.end_bci)
        || outer
            .windows(2)
            .any(|pair| pair[0].end_bci > pair[1].start_bci)
        || inner.start_bci >= inner.end_bci
    {
        return Ok(None);
    }
    let first = outer[0];
    let last = *outer.last().expect("fragmented group");
    if outer_key.0 < last.end_bci
        || inner.handler_bci >= outer_key.0
        || !outer.iter().any(|row| {
            row.start_bci <= inner.start_bci
                && inner.end_bci <= inner.handler_bci
                && inner.handler_bci < row.end_bci
        })
        || inner.ordinal
            >= outer
                .iter()
                .filter(|row| row.start_bci <= inner.start_bci && inner.start_bci < row.end_bci)
                .map(|row| row.ordinal)
                .min()
                .unwrap_or(u32::MAX)
    {
        return Ok(None);
    }
    let row_by_ordinal: BTreeMap<_, _> = rows.iter().map(|row| (row.ordinal, row)).collect();
    if row_by_ordinal.len() != rows.len() || canonical.handler_rows().len() != rows.len() {
        return Ok(None);
    }
    // Each physical record must have precisely the exception edges the canonical pass gave it.
    // A row without a throw site still owns its complete protected block range.
    for declared in rows {
        poll(budget, Some(declared.start_bci))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            (canonical.blocks().len() + canonical.edges().len() + canonical.throw_sites().len())
                as u64,
            Some(declared.start_bci),
        )?;
        let Some(mapped) = canonical
            .handler_rows()
            .iter()
            .find(|row| row.ordinal() == declared.ordinal)
        else {
            return Ok(None);
        };
        let Some(handler) = mapped.handler() else {
            return Ok(None);
        };
        if mapped.handler_bci() != declared.handler_bci
            || mapped.catch_type_index() != declared.catch_type_index
            || handler.bci() != declared.handler_bci
        {
            return Ok(None);
        }
        let sites: BTreeSet<_> = canonical
            .throw_sites()
            .iter()
            .filter(|site| declared.start_bci <= site.bci() && site.bci() < declared.end_bci)
            .map(|site| site.block().clone())
            .collect();
        let expected: BTreeSet<_> = if sites.is_empty() {
            canonical
                .blocks()
                .iter()
                .filter(|block| {
                    block.id().bci() < declared.end_bci && block.end_bci() > declared.start_bci
                })
                .map(|block| block.id().clone())
                .collect()
        } else {
            sites
        };
        if expected.is_empty() || expected != mapped.protected().iter().cloned().collect() {
            return Ok(None);
        }
        let edges: BTreeSet<_> = canonical
            .edges()
            .iter()
            .filter(|edge| {
                edge.kind()
                    == CanonicalEdgeKind::Exception {
                        handler_ordinal: declared.ordinal,
                    }
            })
            .map(|edge| (edge.from().clone(), edge.to().clone()))
            .collect();
        if edges
            != expected
                .iter()
                .map(|block| (block.clone(), handler.clone()))
                .collect()
        {
            return Ok(None);
        }
    }
    // An outer Java catch protects every operation that can raise in its lexical body. This
    // proves the gaps between physical rows contain only operations the JVM cannot dispatch.
    for site in canonical.throw_sites() {
        poll(budget, Some(site.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            rows.len() as u64,
            Some(site.bci()),
        )?;
        let declared: Vec<_> = rows
            .iter()
            .filter(|row| row.start_bci <= site.bci() && site.bci() < row.end_bci)
            .map(|row| row.ordinal)
            .collect();
        if site.handlers() != declared
            || (first.start_bci <= site.bci()
                && site.bci() < last.end_bci
                && declared
                    .iter()
                    .filter(|ordinal| outer.iter().any(|row| row.ordinal == **ordinal))
                    .count()
                    != 1)
        {
            return Ok(None);
        }
    }
    let Some(inner_id) = canonical
        .handler_rows()
        .iter()
        .find(|row| row.ordinal() == inner.ordinal)
        .and_then(|row| row.handler())
    else {
        return Ok(None);
    };
    let Some(outer_id) = canonical
        .handler_rows()
        .iter()
        .find(|row| row.ordinal() == first.ordinal)
        .and_then(|row| row.handler())
    else {
        return Ok(None);
    };
    let (Some(inner_node), Some(outer_node)) = (view.index_of(inner_id), view.index_of(outer_id))
    else {
        return Ok(None);
    };
    if !view.predecessors(inner_node).is_empty() || !view.predecessors(outer_node).is_empty() {
        return Ok(None);
    }
    // Both handlers have a single finite normal continuation. It is the loop update, not an
    // arbitrary point in the loop. The closure may join ordinary body blocks on the way.
    for header in 0..view.len() {
        let Some(loop_of) = view.loop_entered_at(header) else {
            continue;
        };
        let updates: Vec<_> = loop_of.latches().iter().copied().collect();
        let [update] = updates.as_slice() else {
            continue;
        };
        let update = *update;
        let Some(update_id) = view.id_of(update) else {
            continue;
        };
        let core: BTreeSet<_> = loop_of
            .blocks()
            .iter()
            .copied()
            .filter(|node| view.dominates(header, *node))
            .collect();
        let first_node = canonical
            .blocks()
            .iter()
            .find(|block| block.id().bci() == first.start_bci)
            .and_then(|block| view.index_of(block.id()));
        if view.successors(update) != [header]
            || first_node.is_none_or(|node| !core.contains(&node))
        {
            continue;
        }
        let Some(inner_closure) = normal_closure(view, inner_node, update, budget)? else {
            continue;
        };
        let Some(outer_closure) = normal_closure(view, outer_node, update, budget)? else {
            continue;
        };
        let mut exceptional_blocks: BTreeSet<_> = inner_closure
            .union(&outer_closure)
            .copied()
            .filter(|node| !core.contains(node))
            .collect();
        if !exceptional_blocks.contains(&inner_node)
            || !exceptional_blocks.contains(&outer_node)
            || exceptional_blocks.iter().any(|node| {
                view.predecessors(*node)
                    .iter()
                    .any(|pred| !exceptional_blocks.contains(pred))
            })
        {
            continue;
        }
        let loop_entries: BTreeSet<_> = exceptional_blocks
            .iter()
            .flat_map(|node| {
                view.successors(*node)
                    .into_iter()
                    .filter(|next| core.contains(next))
                    .map(|next| (*node, next))
            })
            .collect();
        // The inner protected body's ordinary exit is the only permitted join into its
        // shared normal tail. An exceptional arm may also bypass that tail to the update.
        // In particular, a handler branch directly into some other body block is not a
        // presentation-preserving catch continuation.
        let inner_exit = canonical
            .blocks()
            .iter()
            .find(|block| block.id().bci() <= inner.end_bci && inner.end_bci < block.end_bci())
            .and_then(|block| {
                let last = ssa.block(block.id())?.instructions().last()?;
                let successors = view.successor_ids(block.id());
                (last.bci() == inner.end_bci
                    && matches!(last.opcode(), 0xa7 | 0xc8)
                    && successors.len() == 1)
                    .then(|| view.index_of(&successors[0]))
                    .flatten()
            });
        if loop_entries.is_empty()
            || inner_exit.is_none_or(|node| !core.contains(&node))
            || loop_entries
                .iter()
                .any(|(_, next)| Some(*next) != inner_exit && *next != update)
        {
            continue;
        }
        // The update's SSA consumes the loop-header induction value, with no alternate local
        // definition on either handler continuation. The accumulator may legitimately acquire
        // new definitions; its scope/read proof belongs to the statement builder.
        let Some(update_ssa) = ssa.block(update_id) else {
            continue;
        };
        let Some(update_transfer) = update_ssa.instructions().last() else {
            continue;
        };
        if !matches!(update_transfer.opcode(), 0xa7 | 0xc8) {
            continue;
        }
        let Some(update_step) = update_ssa
            .instructions()
            .iter()
            .find(|step| step.opcode() == 0x84)
        else {
            continue;
        };
        let [(Slot::Local(index), _)] = update_step.reads() else {
            continue;
        };
        if update_step.writes().len() != 1 || update_step.writes()[0].0 != Slot::Local(*index) {
            continue;
        }
        let Some(header_ssa) = view.id_of(header).and_then(|id| ssa.block(id)) else {
            continue;
        };
        let header_index = header_ssa
            .entry()
            .iter()
            .find(|(slot, _)| *slot == Slot::Local(*index))
            .and_then(|(_, value)| resolved_value(ssa, *value));
        if header_index.is_none()
            || resolved_value(ssa, update_step.reads()[0].1) != header_index
            || inner_closure
                .union(&outer_closure)
                .copied()
                .chain(std::iter::once(update))
                .any(|node| {
                    let block = view.id_of(node).and_then(|id| ssa.block(id));
                    block
                        .and_then(|block| {
                            block
                                .entry()
                                .iter()
                                .find(|(slot, _)| *slot == Slot::Local(*index))
                        })
                        .and_then(|(_, value)| resolved_value(ssa, *value))
                        != header_index
                        || (node != update
                            && block
                                .and_then(|block| {
                                    block
                                        .exit()
                                        .iter()
                                        .find(|(slot, _)| *slot == Slot::Local(*index))
                                })
                                .and_then(|(_, value)| resolved_value(ssa, *value))
                                != header_index)
                })
            || [inner_id, outer_id].iter().any(|id| {
                let Some(block) = ssa.block(id) else {
                    return true;
                };
                let Some(store) = block.instructions().first() else {
                    return true;
                };
                store.opcode() != 0x3a && !(0x4b..=0x4e).contains(&store.opcode())
                    || store.reads().len() != 1
                    || !matches!(
                        ssa.value(store.reads()[0].1).def(),
                        Definition::Caught { .. }
                            | Definition::Phi {
                                slot: Slot::Stack(_),
                                ..
                            }
                    )
                    || !header_ssa
                        .entry()
                        .iter()
                        .filter(|(slot, _)| matches!(slot, Slot::Local(_)))
                        .all(|(slot, _)| block.entry().iter().any(|(entry, _)| entry == slot))
            })
        {
            continue;
        }
        let mut exits = Vec::new();
        for row in &outer {
            let matching: Vec<_> = canonical
                .blocks()
                .iter()
                .filter(|block| block.id().bci() <= row.end_bci && row.end_bci < block.end_bci())
                .collect();
            let [block] = matching.as_slice() else {
                exits.clear();
                break;
            };
            let Some(last) = ssa
                .block(block.id())
                .and_then(|entry| entry.instructions().last())
            else {
                exits.clear();
                break;
            };
            let successors = view.successor_ids(block.id());
            let [successor] = successors.as_slice() else {
                exits.clear();
                break;
            };
            if last.bci() != row.end_bci
                || !matches!(last.opcode(), 0xa7 | 0xc8)
                || view.index_of(successor) != Some(update)
            {
                exits.clear();
                break;
            }
            exits.push((*successor).clone());
        }
        if exits.len() != outer.len() {
            continue;
        }
        let outer_exit = exits[0].clone();
        exceptional_blocks.remove(&update);
        return Ok(Some(FragmentedCatch {
            outer_rows: outer.iter().map(|row| row.ordinal).collect(),
            inner_row: inner.ordinal,
            outer_start: first.start_bci,
            inner_start: inner.start_bci,
            inner_exit_bci: inner.end_bci,
            outer_join: outer_exit,
            update: update_id.clone(),
            update_transfer_bci: update_transfer.bci(),
            loop_header: header,
            loop_entries,
            exceptional_blocks,
        }));
    }
    Ok(None)
}

fn resolved_value(ssa: &SsaTable, mut value: ValueId) -> Option<ValueId> {
    for _ in 0..=ssa.values().len() {
        match ssa.value(value).replaced_by() {
            Some(next) => value = next,
            None => return Some(value),
        }
    }
    None
}

fn normal_closure(
    view: &NormalFlowView,
    start: usize,
    join: usize,
    budget: &mut Budget,
) -> Result<Option<BTreeSet<usize>>, StopReason> {
    let mut pending = vec![(start, false)];
    let mut state = BTreeMap::new();
    while let Some((node, finishing)) = pending.pop() {
        let at = view.id_of(node).map(CanonicalBlockId::bci);
        poll(budget, at)?;
        charge(budget, CountedBudgetDimension::AnalysisSteps, 1, at)?;
        if node == join {
            continue;
        }
        if finishing {
            state.insert(node, 2u8);
            continue;
        }
        match state.get(&node) {
            Some(1) => return Ok(None),
            Some(2) => continue,
            _ => {}
        }
        let successors = view.successors(node);
        if successors.is_empty() {
            return Ok(None);
        }
        state.insert(node, 1u8);
        pending.push((node, true));
        pending.extend(successors.into_iter().rev().map(|next| (next, false)));
    }
    Ok(Some(state.into_keys().collect()))
}
