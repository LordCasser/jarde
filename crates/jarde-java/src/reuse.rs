//! Which variable one local slot holds, and where (P3 3.4, the `source-maps` scenario
//! `Slot reuse across ranges`).
//!
//! A slot defaults to **one** variable. Two proofs can split it: distinct LVT names over disjoint
//! ranges, or a narrow SSA/CFG proof that a reference lifetime ends before an int lifetime begins
//! (or the reverse). The second proof works when only the later variable has an LVT record or the
//! class has no debug table; a name is attached to only the lifetime its reads cover. It requires
//! separate definition/use chains, only proved phi inputs, and no path from the later lifetime
//! back to an earlier access. A proved monitor cleanup can account for its own exception edges;
//! other handler and call-context edges still refuse. BCI order alone is insufficient.
//!
//! In the LVT path, a record states two things about a source variable: its
//! **name**, and the range of bytecode over which the source could see it. It does *not* state which
//! instructions are that variable's, and the store that initialises a variable is routinely outside
//! its record: javac 23.0.1's `reuse(ZI)I` states `c` over `[8, 10)` while the `istore_3` that fills
//! it sits at BCI 7, and states `a` over `[10, 13)` and `[19, 21)` while its stores are at 9 and 18.
//! So this module groups the slot's uses in the direction the evidence does support:
//!
//! * a **read** of the slot belongs to the record whose range contains the read's own BCI — a read
//!   happens while the variable is in scope, which is exactly what the range states;
//! * a **write** belongs to the identity its SSA value owner already proves. A read usually ties that
//!   identity to its record; an identity with no local read can be named only when its unique Store
//!   is immediately followed by the row's start in the decoded instruction table.
//!
//! The LVT split is taken only when that assignment is **complete and unambiguous**: every read sits
//! inside exactly one sorted row, and every row fits the corresponding proven identity range. An
//! unread identity also needs the exact Store boundary above. When neither proof applies, the slot
//! remains **one** variable. The LVT path declines these shapes rather than guessing which record
//! names the storage location:
//!
//! * one record for the slot, or several that all state the *same* name: one variable, that name;
//! * ranges that overlap, a record that states no range, or names this run cannot place: one
//!   variable, **no** name — neither record's spelling is the truth about the whole storage location,
//!   so the ordinal name is written instead (P3 3.1's rule, kept where the evidence is not enough);
//! * a read outside every row, a row that crosses its identity range, or an unread identity whose
//!   unique Store is not immediately before the row start: the same. The row cannot name a value this
//!   run cannot place, and placing it wrongly would write one variable's name for another's value.
//!
//! The slot a guarded statement declares in its own header (P3 2.4) is never split: `try (T n = …)`
//! writes that declaration, so the header's slot stays one variable and the guard keeps its own rule
//! for naming it.

use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{
    CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind, Slot, SsaTable, Value, ValueId,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};

use crate::ast::Type;
use crate::decode::Operations;
use crate::facts::Operation;
use crate::names::{DebugLocal, LocalVariable, SlotEvidence};
use crate::region::Region;
use crate::stop::{StopReason, charge, poll};

/// What this run decided the body's local slots hold (P3 3.4).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Plan {
    /// The variables of each slot, as [`crate::names::NameTable::build`] takes them, at the slot's
    /// own index. Slots the evidence says nothing about are [`SlotEvidence::Unnamed`].
    evidence: Vec<SlotEvidence>,
    /// The split slots, and which variable each of their uses belongs to. A slot that is one
    /// variable is absent: every use of it belongs to that one variable.
    splits: BTreeMap<u16, Split>,
}

/// The uses of one split slot, grouped by the variable they belong to.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Split {
    /// The variable each read and write of the slot belongs to, by BCI.
    by_bci: BTreeMap<u32, u16>,
    /// The bytecode range of each variable, in variable order. A use point the map does not list —
    /// the instruction that *consumes* a load, which is where a slot's value reaches the text —
    /// still names the variable whose range covers it.
    ranges: Vec<(u32, u32)>,
}

impl Plan {
    /// The evidence each slot's names are decided from.
    pub(crate) fn evidence(&self) -> &[SlotEvidence] {
        &self.evidence
    }

    /// The variable `slot` holds at the use point `at`, when this run states one.
    ///
    /// `None` means the slot is split and this run cannot place that use in either variable: the
    /// caller states a fallback rather than writing one variable's name where the other's value is
    /// in use. A slot that is one variable always answers, whatever the BCI.
    pub(crate) fn variable_at(&self, slot: u16, at: u32) -> Option<LocalVariable> {
        let Some(split) = self.splits.get(&slot) else {
            return Some(LocalVariable::whole(slot));
        };
        let index = split.by_bci.get(&at).copied().or_else(|| {
            let found = split
                .ranges
                .iter()
                .position(|(start, end)| *start <= at && at < *end)?;
            u16::try_from(found).ok()
        })?;
        Some(LocalVariable::new(slot, index))
    }
}

/// Decides, from the body's own uses and the debug records it carries, which variables each local
/// slot holds.
///
/// `slots` is how many local slots the body has, and `resources` the slots a guarded statement
/// declares in its own header (P3 2.4), which are never split.
pub(crate) fn plan(
    ssa: &SsaTable,
    canonical: &CanonicalCfg,
    operations: &Operations,
    regions: &[Region],
    slots: u16,
    parameters: u16,
    debug: &[DebugLocal],
    resources: &BTreeSet<u16>,
    budget: &mut Budget,
) -> Result<Plan, StopReason> {
    let mut monitor_guards = None;
    let mut records: BTreeMap<u16, Vec<&DebugLocal>> = BTreeMap::new();
    for record in debug {
        records.entry(record.slot()).or_default().push(record);
    }
    // A record for a slot the body's frames do not declare is still evidence about a *name*; the
    // evidence array covers it, so that no slot the table names is left out of the table.
    let covered = records
        .keys()
        .next_back()
        .map_or(slots, |last| slots.max(last.saturating_add(1)));
    let mut plan = Plan {
        evidence: vec![SlotEvidence::Unnamed; usize::from(covered)],
        splits: BTreeMap::new(),
    };
    let accesses = accesses(ssa);
    for slot in 0..covered {
        let index = usize::from(slot);
        let slot_records = records.get(&slot).map(Vec::as_slice).unwrap_or(&[]);
        if let Some((names, split)) = typed_split(
            ssa,
            canonical,
            regions,
            &mut monitor_guards,
            slot,
            parameters,
            slot_records,
            accesses.get(&slot),
            resources,
            budget,
        )? {
            plan.evidence[index] = SlotEvidence::Split(names);
            plan.splits.insert(slot, split);
            continue;
        }
        if let Some((segments, split)) = array_retype_split(
            ssa,
            canonical,
            operations,
            slot,
            parameters,
            slot_records,
            accesses.get(&slot),
            resources,
            budget,
        )? {
            plan.evidence[index] = SlotEvidence::Split(vec![None; segments]);
            plan.splits.insert(slot, split);
            continue;
        }
        if let Some((segments, split)) = reference_retype_split(
            ssa,
            canonical,
            operations,
            slot,
            parameters,
            slot_records,
            accesses.get(&slot),
            resources,
            budget,
        )? {
            plan.evidence[index] = SlotEvidence::Split(segments);
            plan.splits.insert(slot, split);
            continue;
        }
        if slot_records.is_empty() {
            continue;
        }
        let names: BTreeSet<&str> = slot_records.iter().map(|record| record.name()).collect();
        if names.len() <= 1 {
            plan.evidence[index] = SlotEvidence::Whole(slot_records[0].name().to_string());
            continue;
        }
        match split(ssa, slot, slot_records, &accesses, resources) {
            Some((names, split)) => {
                plan.evidence[index] = SlotEvidence::Split(names.into_iter().map(Some).collect());
                plan.splits.insert(slot, split);
            }
            None => plan.evidence[index] = SlotEvidence::Unnamed,
        }
    }
    Ok(plan)
}

/// Only a recovered monitor guard can account for its synthetic cleanup exception edges.
/// Region recovery has already proved and claimed every block in each plan.
fn monitor_guard_blocks(
    regions: &[Region],
    budget: &mut Budget,
) -> Result<Vec<(BTreeSet<CanonicalBlockId>, BTreeSet<u32>)>, StopReason> {
    let mut pending: Vec<&Region> = regions.iter().rev().collect();
    let mut guards = Vec::new();
    while let Some(region) = pending.pop() {
        poll(budget, None)?;
        match region {
            Region::Guard {
                plan,
                body,
                finally_body,
                ..
            } => {
                if let crate::guard::Shape::Monitor { cleanup_rows, .. } = plan.shape() {
                    let mut owned = BTreeSet::new();
                    for block in plan.owned() {
                        poll(budget, Some(block.bci()))?;
                        charge(
                            budget,
                            CountedBudgetDimension::IrItems,
                            1,
                            Some(block.bci()),
                        )?;
                        owned.insert(block.clone());
                    }
                    guards.push((owned, cleanup_rows.clone()));
                }
                if let Some(body) = body {
                    pending.push(body);
                }
                if let Some(cleanup) = finally_body {
                    pending.push(cleanup);
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
            _ => {}
        }
    }
    Ok(guards)
}

/// Every read and write of every local slot of the body, by slot.
///
/// Each access carries the block it sits in, so that the store a read's value came from is looked
/// for where the read is — the same block-local scan the builder's own
/// `slot_name_denotes_the_same_value` performs.
fn accesses(ssa: &SsaTable) -> BTreeMap<u16, Accesses> {
    let mut accesses: BTreeMap<u16, Accesses> = BTreeMap::new();
    for (block, entry) in ssa.blocks().iter().enumerate() {
        for instruction in entry.instructions() {
            let bci = instruction.bci();
            for (slot, value) in instruction.reads() {
                if let Slot::Local(slot) = slot {
                    accesses.entry(*slot).or_default().reads.push(Access {
                        slot: *slot,
                        block,
                        bci,
                        value: *value,
                    });
                }
            }
            for (slot, value) in instruction.writes() {
                if let Slot::Local(slot) = slot {
                    accesses.entry(*slot).or_default().writes.push(Access {
                        slot: *slot,
                        block,
                        bci,
                        value: *value,
                    });
                }
            }
        }
    }
    accesses
}

/// The first deliberately narrow inference beyond two LVT names: a reference lifetime followed
/// by an int lifetime, or the reverse. Both categories are stated by the frames, not by a debug
/// spelling. A value in any other category leaves the old plan in force.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Category {
    Reference,
    Int,
}

fn category(value: &Value) -> Option<Category> {
    match value {
        Value::Ref(_) => Some(Category::Reference),
        Value::Int => Some(Category::Int),
        _ => None,
    }
}

/// A trivial phi is an alias of the value that replaced it. A nontrivial phi is never used as a
/// reason to cut a lifetime; following the published replacement chain is bounded by the SSA's
/// own value count, so malformed cycles cannot make this proof loop forever.
fn representative(ssa: &SsaTable, mut value: ValueId) -> Option<ValueId> {
    for _ in 0..=ssa.values().len() {
        let Some(next) = ssa.value(value).replaced_by() else {
            return Some(value);
        };
        value = next;
    }
    None
}

/// Infer two locals only when every access has a typed value chain, the two categories occupy
/// ordered, non-overlapping parts of the CFG, and no later block can reach an earlier access.
/// This admits a loop within the first part (including its trivial self phi) but not a loop from
/// the later part back into it. Handler and call-context edges are left for a separate proof.
#[allow(clippy::too_many_arguments)]
fn typed_split(
    ssa: &SsaTable,
    canonical: &CanonicalCfg,
    regions: &[Region],
    monitor_guards: &mut Option<Vec<(BTreeSet<CanonicalBlockId>, BTreeSet<u32>)>>,
    slot: u16,
    parameters: u16,
    records: &[&DebugLocal],
    accesses: Option<&Accesses>,
    resources: &BTreeSet<u16>,
    budget: &mut Budget,
) -> Result<Option<(Vec<Option<String>>, Split)>, StopReason> {
    if slot < parameters || resources.contains(&slot) || !canonical.unreachable().is_empty() {
        return Ok(None);
    }
    let Some(accesses) = accesses else {
        return Ok(None);
    };
    if accesses.reads.is_empty() || accesses.writes.len() < 2 {
        return Ok(None);
    }
    let mut typed = Vec::with_capacity(accesses.reads.len() + accesses.writes.len());
    for access in accesses.reads.iter().chain(&accesses.writes) {
        poll(budget, Some(access.bci))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(access.bci),
        )?;
        let Some(kind) = category(ssa.value(access.value).ty()) else {
            return Ok(None);
        };
        if ssa.blocks()[access.block].block().is_clone() {
            return Ok(None);
        }
        typed.push((*access, kind));
    }
    let bounds = |kind| {
        let mut bcis = typed
            .iter()
            .filter(|(_, found)| *found == kind)
            .map(|(access, _)| access.bci);
        let first = bcis.next()?;
        Some(bcis.fold((first, first), |(lo, hi), bci| (lo.min(bci), hi.max(bci))))
    };
    let (Some(reference), Some(int)) = (bounds(Category::Reference), bounds(Category::Int)) else {
        return Ok(None);
    };
    let (earlier, later, boundary) = if reference.1 < int.0 {
        (Category::Reference, Category::Int, int.0)
    } else if int.1 < reference.0 {
        (Category::Int, Category::Reference, reference.0)
    } else {
        return Ok(None);
    };
    let first_bci = reference.0.min(int.0);
    if !accesses.writes.iter().any(|write| write.bci == first_bci)
        || !accesses.writes.iter().any(|write| write.bci == boundary)
    {
        return Ok(None);
    }

    let earlier_blocks: BTreeSet<_> = typed
        .iter()
        .filter(|(_, kind)| *kind == earlier)
        .map(|(access, _)| ssa.blocks()[access.block].block().clone())
        .collect();
    let later_blocks: BTreeSet<_> = typed
        .iter()
        .filter(|(_, kind)| *kind == later)
        .map(|(access, _)| ssa.blocks()[access.block].block().clone())
        .collect();
    let mut monitor = None;
    if earlier == Category::Reference
        && later == Category::Int
        && !canonical.handler_rows().is_empty()
    {
        // A monitor proof needs its cleanup exception rows. Ordinary methods need no region
        // scan, and walking already-billed non-monitor regions must not alter their ledger.
        if monitor_guards.is_none() {
            *monitor_guards = Some(monitor_guard_blocks(regions, budget)?);
        }
        for guard in monitor_guards
            .as_ref()
            .expect("monitor guards were just collected")
        {
            poll(budget, Some(boundary))?;
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(boundary),
            )?;
            if earlier_blocks.is_subset(&guard.0) && later_blocks.is_disjoint(&guard.0) {
                monitor = Some(guard);
                break;
            }
        }
    }

    let index = |kind| if kind == earlier { 0u16 } else { 1u16 };
    let mut by_bci = BTreeMap::new();
    for (access, kind) in &typed {
        let assigned = index(*kind);
        if by_bci
            .insert(access.bci, assigned)
            .is_some_and(|old| old != assigned)
        {
            return Ok(None);
        }
    }
    let mut writes: [BTreeSet<ValueId>; 2] = std::array::from_fn(|_| BTreeSet::new());
    let mut reads: [BTreeSet<ValueId>; 2] = std::array::from_fn(|_| BTreeSet::new());
    for access in &accesses.writes {
        let kind = category(ssa.value(access.value).ty()).expect("all accesses were classified");
        let Some(value) = representative(ssa, access.value) else {
            return Ok(None);
        };
        writes[usize::from(index(kind))].insert(value);
    }
    for access in &accesses.reads {
        let kind = category(ssa.value(access.value).ty()).expect("all accesses were classified");
        let Some(value) = representative(ssa, access.value) else {
            return Ok(None);
        };
        reads[usize::from(index(kind))].insert(value);
    }
    let mut phi_inputs = BTreeSet::new();
    let mut phi_values = BTreeSet::new();
    let mut phi_blocks = BTreeSet::new();
    for phi in ssa
        .phis()
        .iter()
        .filter(|phi| phi.slot() == Slot::Local(slot))
    {
        poll(budget, Some(phi.block().bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(phi.block().bci()),
        )?;
        if let Some(replaced) = ssa.value(phi.value()).replaced_by() {
            let Some(replaced) = representative(ssa, replaced) else {
                return Ok(None);
            };
            if phi.inputs().iter().any(|input| match input {
                jarde_jvm::method_ir::PhiInput::Value(value) => {
                    representative(ssa, *value) != Some(replaced)
                }
                jarde_jvm::method_ir::PhiInput::Itself => false,
            }) {
                return Ok(None);
            }
            continue;
        }
        // A loop-carried phi is admitted only for the integer lifetime after this proved
        // monitor guard. Its every incoming value must be one of that lifetime's writes.
        if monitor.is_none_or(|(owned, _)| owned.contains(phi.block()))
            || phi.block().bci() < boundary
            || !reads[1].contains(&phi.value())
            || phi.inputs().is_empty()
        {
            return Ok(None);
        }
        for input in phi.inputs() {
            poll(budget, Some(phi.block().bci()))?;
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(phi.block().bci()),
            )?;
            let jarde_jvm::method_ir::PhiInput::Value(value) = input else {
                return Ok(None);
            };
            let Some(value) = representative(ssa, *value) else {
                return Ok(None);
            };
            if !writes[1].contains(&value) {
                return Ok(None);
            }
            phi_inputs.insert(value);
        }
        phi_values.insert(phi.value());
        phi_blocks.insert(phi.block().clone());
    }
    if (0..2).any(|part| {
        writes[part].is_empty()
            || reads[part].is_empty()
            || writes[part].iter().any(|value| {
                !reads[part].contains(value) && (part != 1 || !phi_inputs.contains(value))
            })
            || reads[part].iter().any(|value| {
                !writes[part].contains(value) && (part != 1 || !phi_values.contains(value))
            })
    }) {
        return Ok(None);
    }
    // A local value loaded in the first lifetime can survive on the operand stack after the
    // slot is overwritten. The split would then have to name that old value at a later source
    // position; this first slice does not prove such a scope, so it refuses that shape.
    for value in &writes[0] {
        for use_ in ssa.value(*value).uses() {
            let at = use_.bci().unwrap_or_else(|| use_.block().bci());
            poll(budget, Some(at))?;
            charge(budget, CountedBudgetDimension::AnalysisSteps, 1, Some(at))?;
            if at >= boundary {
                return Ok(None);
            }
        }
    }
    let mut edges = BTreeMap::new();
    for edge in canonical.edges() {
        poll(budget, Some(edge.from().bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(edge.from().bci()),
        )?;
        match edge.kind() {
            CanonicalEdgeKind::Normal => {}
            CanonicalEdgeKind::Exception { handler_ordinal }
                if monitor.is_some_and(|(owned, rows)| {
                    rows.contains(&handler_ordinal)
                        && owned.contains(edge.from())
                        && owned.contains(edge.to())
                }) => {}
            _ => return Ok(None),
        }
        edges
            .entry(edge.from().clone())
            .or_insert_with(Vec::new)
            .push(edge.to().clone());
    }
    let mut pending: Vec<_> = later_blocks.into_iter().collect();
    pending.extend(phi_blocks);
    let mut visited = BTreeSet::new();
    while let Some(block) = pending.pop() {
        poll(budget, Some(block.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(block.bci()),
        )?;
        if earlier_blocks.contains(&block) {
            return Ok(None);
        }
        if visited.insert(block.clone()) {
            pending.extend(edges.get(&block).into_iter().flatten().cloned());
        }
    }

    let mut names = vec![None, None];
    for record in records {
        let Some((start, end)) = record.range() else {
            return Ok(None);
        };
        let covered: BTreeSet<_> = accesses
            .reads
            .iter()
            .filter(|access| start <= access.bci && access.bci < end)
            .filter_map(|access| category(ssa.value(access.value).ty()).map(index))
            .collect();
        if covered.len() > 1 {
            return Ok(None);
        }
        if let Some(part) = covered.iter().next() {
            let name = &mut names[usize::from(*part)];
            if name.as_ref().is_some_and(|old| old != record.name()) {
                return Ok(None);
            }
            *name = Some(record.name().to_string());
        }
    }
    Ok(Some((
        names,
        Split {
            by_bci,
            ranges: vec![(0, boundary), (boundary, u32::MAX)],
        },
    )))
}

/// The third deliberately narrow inference: an array lifetime whose element type differs from a
/// later array lifetime over the same slot (EM-17/18 A2: the `int[]` a computed fill holds, then
/// the `boolean[]` the same slot holds once the first is dead). The spellings come from the array
/// channel the frames and the creating instructions already state —
/// [`crate::build::array_of_value`] — so no new proof fact is read.
///
/// Which definition owns a read is the SSA's own answer: the read's value is the one store that
/// wrote it, so the reads two definitions own cannot overlap, and the reads that name no single
/// definition refuse the split — above all a phi that merges two definitions' values (the join or
/// loop-carried confluence of a true alias), and beside it a caught reference or an entry value.
/// The refused slot keeps the one variable it has today, exactly as a slot whose writes all state
/// the same spelling does: one declaration already presents that.
///
/// Each definition is one segment, and one segment is one variable with exactly one write — which
/// is also why a definition's value surviving on the operand stack past the next store refuses
/// here: the name a consumer of that stack value spells is placed by the segment ranges, and a
/// consumer at or past the next definition's store would read the next segment's name.
#[allow(clippy::too_many_arguments)]
fn array_retype_split(
    ssa: &SsaTable,
    canonical: &CanonicalCfg,
    operations: &Operations,
    slot: u16,
    parameters: u16,
    records: &[&DebugLocal],
    accesses: Option<&Accesses>,
    resources: &BTreeSet<u16>,
    budget: &mut Budget,
) -> Result<Option<(usize, Split)>, StopReason> {
    // The same scope rules the other narrow proof keeps: a parameter's declaration is the
    // signature's, a guard header's slot is the header's own rule, a slot the debug table records
    // is the records' decision, and unreachable code states no store order at all.
    if slot < parameters
        || resources.contains(&slot)
        || !records.is_empty()
        || !canonical.unreachable().is_empty()
    {
        return Ok(None);
    }
    let Some(accesses) = accesses else {
        return Ok(None);
    };
    if accesses.reads.is_empty() || accesses.writes.len() < 2 {
        return Ok(None);
    }
    // Every write states a full array spelling, and at least two spellings differ. The writes are
    // the segment boundaries in store order; a definition whose value nothing reads still bounds
    // the segment before it, and its own statement is presented as the declaration it is.
    let mut definitions: Vec<(u32, ValueId, (Type, u32))> =
        Vec::with_capacity(accesses.writes.len());
    for write in &accesses.writes {
        poll(budget, Some(write.bci))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(write.bci),
        )?;
        let Some(shape) = crate::build::array_of_value(ssa, operations, write.value, 0) else {
            return Ok(None);
        };
        definitions.push((write.bci, write.value, shape));
    }
    let Some(split) = owned_definition_split(ssa, accesses, definitions, None, budget)? else {
        return Ok(None);
    };
    Ok(Some((split.ranges.len(), split)))
}

/// Split reference lifetimes whose exact source types differ, reusing the same definition owner
/// proof as array retyping. LVT rows may name the proven identities only after every read maps
/// one-to-one to a row in definition order. The complete normal CFG proof remains authoritative.
#[allow(clippy::too_many_arguments)]
fn reference_retype_split(
    ssa: &SsaTable,
    canonical: &CanonicalCfg,
    operations: &Operations,
    slot: u16,
    parameters: u16,
    records: &[&DebugLocal],
    accesses: Option<&Accesses>,
    resources: &BTreeSet<u16>,
    budget: &mut Budget,
) -> Result<Option<(Vec<Option<String>>, Split)>, StopReason> {
    if slot < parameters || resources.contains(&slot) || !canonical.unreachable().is_empty() {
        return Ok(None);
    }
    let Some(accesses) = accesses else {
        return Ok(None);
    };
    if accesses.reads.is_empty() || accesses.writes.len() < 2 {
        return Ok(None);
    }
    // The owner proof creates exactly one segment per distinct write BCI. Rows that cannot
    // correspond one-to-one are known refusals and should retain the old no-scan cost.
    if !records.is_empty() && records.len() != accesses.writes.len() {
        return Ok(None);
    }
    for block in ssa.blocks() {
        poll(budget, Some(block.block().bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(block.block().bci()),
        )?;
        if block.block().is_clone() {
            return Ok(None);
        }
    }
    let mut definitions = Vec::with_capacity(accesses.writes.len());
    let mut distinct_type = false;
    for write in &accesses.writes {
        poll(budget, Some(write.bci))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(write.bci),
        )?;
        let Some(ty) = crate::build::reference_lifetime_type(ssa, operations, write.value) else {
            return Ok(None);
        };
        if definitions
            .first()
            .is_some_and(|(_, _, first_type)| first_type != &ty)
        {
            distinct_type = true;
        }
        definitions.push((write.bci, write.value, ty));
    }
    if !distinct_type {
        return Ok(None);
    }
    if !reference_values_stay_before_boundaries(ssa, operations, &definitions, budget)? {
        return Ok(None);
    }
    let Some(split) = owned_definition_split(ssa, accesses, definitions, Some(canonical), budget)?
    else {
        return Ok(None);
    };
    if records.is_empty() {
        return Ok(Some((vec![None; split.ranges.len()], split)));
    }
    let Some(names) = reference_split_names(
        records,
        accesses.reads.iter().map(|read| read.bci),
        &accesses.writes,
        Some(operations),
        &split,
        budget,
    )?
    else {
        return Ok(None);
    };
    Ok(Some((names, split)))
}

/// Attach LVT names only when rows and proven SSA identities agree exactly. Every read must be
/// inside its SSA owner's row, and every row must fit that identity's range. An unread identity is
/// named only when its one proven Store is the immediately preceding decoded instruction.
fn reference_split_names(
    records: &[&DebugLocal],
    read_bcis: impl IntoIterator<Item = u32>,
    writes: &[Access],
    operations: Option<&Operations>,
    split: &Split,
    budget: &mut Budget,
) -> Result<Option<Vec<Option<String>>>, StopReason> {
    let segments = split.ranges.len();
    if records.is_empty() {
        return Ok(Some(vec![None; segments]));
    }
    if records.len() != segments {
        return Ok(None);
    }

    // The BTreeMap gives a deterministic (start, end) order without an extra sorting pass. Charge
    // each insertion and ordered visit; its tree work follows the same per-item accounting used for
    // existing BTreeMap-backed analysis. Duplicate intervals cannot identify distinct owners.
    let mut row_map = BTreeMap::new();
    for record in records {
        let at = record.range().map(|(start, _)| start);
        poll(budget, at)?;
        charge(budget, CountedBudgetDimension::AnalysisSteps, 1, at)?;
        let Some((start, end)) = record.range() else {
            return Ok(None);
        };
        if start >= end
            || row_map
                .insert((start, end), record.name().to_string())
                .is_some()
        {
            return Ok(None);
        }
    }

    let mut rows = Vec::with_capacity(segments);
    let mut previous_end = None;
    for ((start, end), name) in row_map {
        poll(budget, Some(start))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(start),
        )?;
        if previous_end.is_some_and(|end| end > start) {
            return Ok(None);
        }
        previous_end = Some(end);
        rows.push((start, end, name));
    }

    let mut rows_with_reads = vec![false; segments];
    for read_bci in read_bcis {
        poll(budget, Some(read_bci))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(read_bci),
        )?;
        let Some(owner) = split.by_bci.get(&read_bci).copied() else {
            return Ok(None);
        };
        let index = usize::from(owner);
        let Some((start, end, _)) = rows.get(index) else {
            return Ok(None);
        };
        if read_bci < *start || read_bci >= *end {
            return Ok(None);
        }
        rows_with_reads[index] = true;
    }
    // A missing read cannot establish the row's identity. The narrow exception uses the already
    // proved store owner and the decoded instruction table: no opcode-width estimate or LVT-only
    // guess can make a different store qualify. Do not scan stores on the ordinary all-read path.
    let writes_per_owner = if rows_with_reads.iter().any(|has_read| !has_read) {
        let mut writes_per_owner = vec![Vec::new(); segments];
        for write in writes {
            poll(budget, Some(write.bci))?;
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(write.bci),
            )?;
            let Some(owner) = split.by_bci.get(&write.bci).copied() else {
                return Ok(None);
            };
            let index = usize::from(owner);
            let Some(owner_writes) = writes_per_owner.get_mut(index) else {
                return Ok(None);
            };
            owner_writes.push(write);
        }
        Some(writes_per_owner)
    } else {
        None
    };

    let mut names = Vec::with_capacity(segments);
    for (index, ((start, end, name), has_read)) in rows.into_iter().zip(rows_with_reads).enumerate()
    {
        poll(budget, Some(start))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(start),
        )?;
        let Some((identity_start, identity_end)) = split.ranges.get(index).copied() else {
            return Ok(None);
        };
        if start < identity_start || end > identity_end {
            return Ok(None);
        }
        if !has_read {
            let Some(operations) = operations else {
                return Ok(None);
            };
            let Some(writes_per_owner) = writes_per_owner.as_ref() else {
                return Ok(None);
            };
            let owner_writes = writes_per_owner[index].as_slice();
            if owner_writes.len() != 1 {
                return Ok(None);
            }
            let write = owner_writes[0];
            let is_matching_store = matches!(
                operations.get(write.bci),
                Some(Operation::Store { slot }) if *slot == write.slot
            );
            if !is_matching_store || operations.next_bci(write.bci) != Some(start) {
                return Ok(None);
            }
        }
        names.push(Some(name));
    }
    Ok(Some(names))
}

/// A local read can move an older reference onto the operand stack, where a later local write
/// cannot kill it. Follow only operations whose decoded meaning preserves that same reference:
/// local load/store, dup, and checkcast. Ordinary computations and calls create values with their
/// own meaning and are not treated as aliases of their inputs.
fn reference_values_stay_before_boundaries(
    ssa: &SsaTable,
    operations: &Operations,
    definitions: &[(u32, ValueId, Type)],
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    for index in 0..definitions.len().saturating_sub(1) {
        let (write_bci, value, _) = &definitions[index];
        let boundary = definitions[index + 1].0;
        poll(budget, Some(*write_bci))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(*write_bci),
        )?;
        let Some(value) = representative(ssa, *value) else {
            return Ok(false);
        };
        let mut pending = vec![value];
        let mut visited = BTreeSet::new();
        while let Some(value) = pending.pop() {
            poll(budget, Some(*write_bci))?;
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(*write_bci),
            )?;
            let Some(value) = representative(ssa, value) else {
                return Ok(false);
            };
            if !visited.insert(value) {
                continue;
            }
            for use_ in ssa.value(value).uses() {
                let at = use_.bci().unwrap_or_else(|| use_.block().bci());
                poll(budget, Some(at))?;
                charge(budget, CountedBudgetDimension::AnalysisSteps, 1, Some(at))?;
                if at >= boundary {
                    return Ok(false);
                }
                if use_.bci().is_none() {
                    let Some(forwarded) = trivial_phi_outputs(ssa, use_.block(), value, budget)?
                    else {
                        return Ok(false);
                    };
                    pending.extend(forwarded);
                    continue;
                }
                let Some(instruction) = instruction_at_use(ssa, use_.block(), at, budget)? else {
                    return Ok(false);
                };
                if unsupported_stack_copy_uses_source(ssa, operations, instruction, value, budget)?
                {
                    return Ok(false);
                }
                let Some(forwarded) =
                    reference_forwarded_values(ssa, operations, instruction, value, budget)?
                else {
                    continue;
                };
                if forwarded.is_empty() {
                    return Ok(false);
                }
                pending.extend(forwarded);
            }
        }
    }
    Ok(true)
}

fn trivial_phi_outputs(
    ssa: &SsaTable,
    block: &CanonicalBlockId,
    source: ValueId,
    budget: &mut Budget,
) -> Result<Option<Vec<ValueId>>, StopReason> {
    let mut outputs = Vec::new();
    for phi in ssa.phis() {
        poll(budget, Some(block.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(block.bci()),
        )?;
        if phi.block() != block {
            continue;
        }
        let Some(output) = representative(ssa, phi.value()) else {
            return Ok(None);
        };
        let mut contains_source = false;
        let mut all_same = true;
        for input in phi.inputs() {
            poll(budget, Some(block.bci()))?;
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(block.bci()),
            )?;
            let input = match input {
                jarde_jvm::method_ir::PhiInput::Value(value) => {
                    let Some(value) = representative(ssa, *value) else {
                        return Ok(None);
                    };
                    value
                }
                jarde_jvm::method_ir::PhiInput::Itself => output,
            };
            contains_source |= input == source;
            all_same &= input == source;
        }
        if contains_source {
            if !all_same || ssa.value(phi.value()).replaced_by().is_none() {
                return Ok(None);
            }
            outputs.push(output);
        }
    }
    if outputs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(outputs))
    }
}

fn instruction_at_use<'a>(
    ssa: &'a SsaTable,
    block_id: &CanonicalBlockId,
    bci: u32,
    budget: &mut Budget,
) -> Result<Option<&'a jarde_jvm::method_ir::SsaInstruction>, StopReason> {
    let Some(block) = ssa.block(block_id) else {
        return Ok(None);
    };
    poll(budget, Some(block_id.bci()))?;
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        1,
        Some(block_id.bci()),
    )?;
    for instruction in block.instructions() {
        poll(budget, Some(instruction.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(instruction.bci()),
        )?;
        if instruction.bci() == bci {
            return Ok(Some(instruction));
        }
    }
    Ok(None)
}

fn unsupported_stack_copy_uses_source(
    ssa: &SsaTable,
    operations: &Operations,
    instruction: &jarde_jvm::method_ir::SsaInstruction,
    source: ValueId,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    if !matches!(operations.get(instruction.bci()), Some(Operation::Other))
        || !(0x5a..=0x5f).contains(&instruction.opcode())
    {
        return Ok(false);
    }
    for (slot, value) in instruction.reads() {
        poll(budget, Some(instruction.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(instruction.bci()),
        )?;
        if matches!(slot, Slot::Stack(_)) && representative(ssa, *value) == Some(source) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn reference_forwarded_values(
    ssa: &SsaTable,
    operations: &Operations,
    instruction: &jarde_jvm::method_ir::SsaInstruction,
    source: ValueId,
    budget: &mut Budget,
) -> Result<Option<Vec<ValueId>>, StopReason> {
    let Some(operation) = operations.get(instruction.bci()) else {
        return Ok(None);
    };
    let reads = instruction.reads();
    let writes = instruction.writes();
    let has_source_read = |slot: Slot, budget: &mut Budget| -> Result<bool, StopReason> {
        for (read_slot, value) in reads {
            poll(budget, Some(instruction.bci()))?;
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(instruction.bci()),
            )?;
            if *read_slot == slot && representative(ssa, *value) == Some(source) {
                return Ok(true);
            }
        }
        Ok(false)
    };
    let has_source_stack_read = |budget: &mut Budget| -> Result<bool, StopReason> {
        for (read_slot, value) in reads {
            poll(budget, Some(instruction.bci()))?;
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(instruction.bci()),
            )?;
            if matches!(read_slot, Slot::Stack(_)) && representative(ssa, *value) == Some(source) {
                return Ok(true);
            }
        }
        Ok(false)
    };
    let output_slots = match operation {
        Operation::Load { slot } if has_source_read(Slot::Local(*slot), budget)? => {
            Some((None, true))
        }
        Operation::Store { slot } if has_source_stack_read(budget)? => {
            Some((Some(Slot::Local(*slot)), false))
        }
        Operation::Duplicate if has_source_stack_read(budget)? => Some((None, true)),
        Operation::CheckCast { .. } if has_source_stack_read(budget)? => Some((None, true)),
        _ => None,
    };
    let Some((required_slot, stack_outputs)) = output_slots else {
        return Ok(None);
    };
    let mut forwarded = Vec::new();
    for (slot, value) in writes {
        poll(budget, Some(instruction.bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(instruction.bci()),
        )?;
        let selected = required_slot.map_or_else(
            || stack_outputs && matches!(slot, Slot::Stack(_)),
            |required| *slot == required,
        );
        if selected
            && matches!(ssa.value(*value).ty(), Value::Ref(_))
            && matches!(
                ssa.value(*value).def(),
                jarde_jvm::method_ir::Definition::Instruction { bci, .. }
                    if *bci == instruction.bci()
            )
        {
            if let Some(value) = representative(ssa, *value) {
                forwarded.push(value);
            } else {
                return Ok(None);
            }
        }
    }
    Ok(Some(forwarded))
}

/// Assign each read to the unique slot definition whose SSA value it reads. Both array retyping
/// and ordinary reference retyping use this one owner/liveness implementation; only the latter
/// supplies the normal-CFG proof required before the identities may be published.
fn owned_definition_split<T: Eq>(
    ssa: &SsaTable,
    accesses: &Accesses,
    mut definitions: Vec<(u32, ValueId, T)>,
    canonical: Option<&CanonicalCfg>,
    budget: &mut Budget,
) -> Result<Option<Split>, StopReason> {
    definitions.sort_by_key(|(bci, _, _)| *bci);
    if definitions
        .windows(2)
        .any(|window| window[0].0 == window[1].0)
        || definitions
            .windows(2)
            .all(|window| window[0].2 == window[1].2)
    {
        return Ok(None);
    }
    // A definition's own value outliving the next definition's store would place a read of the
    // earlier segment's name at a statement the later segment's ranges cover. The reads' own
    // positions cannot do this (a read past the next store reads the next store's value), but the
    // value can survive on the operand stack or feed a merge; both refuse.
    for (index, (_, value, _)) in definitions.iter().enumerate() {
        let Some(boundary) = definitions.get(index + 1).map(|(bci, _, _)| *bci) else {
            break;
        };
        let Some(value) = representative(ssa, *value) else {
            return Ok(None);
        };
        for use_ in ssa.value(value).uses() {
            let at = use_.bci().unwrap_or_else(|| use_.block().bci());
            poll(budget, Some(at))?;
            charge(budget, CountedBudgetDimension::AnalysisSteps, 1, Some(at))?;
            if at >= boundary {
                return Ok(None);
            }
        }
    }
    // The variable each access belongs to: a write its own segment, a read the segment of the
    // definition whose value it reads. A read whose value names no definition of this slot keeps
    // the slot one variable, and so does a definition two reads would claim.
    let mut by_bci: BTreeMap<u32, u16> = BTreeMap::new();
    for (index, (bci, _, _)) in definitions.iter().enumerate() {
        let index = u16::try_from(index).unwrap_or(u16::MAX);
        by_bci.insert(*bci, index);
    }
    for read in &accesses.reads {
        poll(budget, Some(read.bci))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(read.bci),
        )?;
        let Some(value) = representative(ssa, read.value) else {
            return Ok(None);
        };
        let mut owners = definitions
            .iter()
            .enumerate()
            .filter(|(_, (_, defined, _))| {
                representative(ssa, *defined).is_some_and(|found| found == value)
            });
        let Some((index, _)) = owners.next() else {
            return Ok(None);
        };
        if owners.next().is_some() {
            return Ok(None);
        }
        let index = u16::try_from(index).unwrap_or(u16::MAX);
        match by_bci.insert(read.bci, index) {
            // A read and a write at one BCI have to agree on the segment they touch.
            Some(assigned) if assigned != index => return Ok(None),
            _ => {}
        }
    }
    if let Some(canonical) = canonical
        && !normal_flow_stays_after_boundaries(ssa, canonical, accesses, &by_bci, budget)?
    {
        return Ok(None);
    }
    let mut ranges = Vec::with_capacity(definitions.len());
    let mut start = 0;
    for (bci, _, _) in &definitions[1..] {
        ranges.push((start, *bci));
        start = *bci;
    }
    ranges.push((start, u32::MAX));
    Ok(Some(Split { by_bci, ranges }))
}

/// Every later-segment access must remain outside all earlier-segment blocks on every normal
/// path. Starting at outgoing edges makes an ordered pair of accesses in one block legal, while a
/// real edge cycle back to that block is still found. Any exceptional/call-context edge refuses.
fn normal_flow_stays_after_boundaries(
    ssa: &SsaTable,
    canonical: &CanonicalCfg,
    accesses: &Accesses,
    by_bci: &BTreeMap<u32, u16>,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    let mut edges: BTreeMap<CanonicalBlockId, Vec<CanonicalBlockId>> = BTreeMap::new();
    for edge in canonical.edges() {
        poll(budget, Some(edge.from().bci()))?;
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            1,
            Some(edge.from().bci()),
        )?;
        if edge.kind() != CanonicalEdgeKind::Normal {
            return Ok(false);
        }
        edges
            .entry(edge.from().clone())
            .or_default()
            .push(edge.to().clone());
    }
    let max_segment = by_bci.values().copied().max().unwrap_or(0);
    for boundary in 1..=max_segment {
        let mut earlier_blocks = BTreeSet::new();
        let mut later_blocks = BTreeSet::new();
        for access in accesses.reads.iter().chain(&accesses.writes) {
            let at = Some(access.bci);
            poll(budget, at)?;
            charge(budget, CountedBudgetDimension::AnalysisSteps, 1, at)?;
            let Some(block) = ssa
                .blocks()
                .get(access.block)
                .map(|block| block.block().clone())
            else {
                return Ok(false);
            };
            match by_bci.get(&access.bci) {
                Some(part) if *part < boundary => {
                    earlier_blocks.insert(block);
                }
                Some(_) => {
                    later_blocks.insert(block);
                }
                None => return Ok(false),
            }
        }
        let mut pending = Vec::new();
        for block in later_blocks {
            pending.extend(edges.get(&block).into_iter().flatten().cloned());
        }
        let mut visited = BTreeSet::new();
        while let Some(block) = pending.pop() {
            poll(budget, Some(block.bci()))?;
            charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                1,
                Some(block.bci()),
            )?;
            if earlier_blocks.contains(&block) {
                return Ok(false);
            }
            if visited.insert(block.clone()) {
                pending.extend(edges.get(&block).into_iter().flatten().cloned());
            }
        }
    }
    Ok(true)
}

/// The accesses of one local slot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Accesses {
    reads: Vec<Access>,
    writes: Vec<Access>,
}

/// One access of one local slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Access {
    /// The slot the access touches.
    slot: u16,
    /// The block the accessing instruction sits in.
    block: usize,
    bci: u32,
    value: ValueId,
}

/// The split of one slot: its variables' names in range order, and the variable of each use.
///
/// `None` means this run does not split the slot — see the module's own rules for the shapes that
/// keep a reused slot one variable.
fn split(
    ssa: &SsaTable,
    slot: u16,
    records: &[&DebugLocal],
    accesses: &BTreeMap<u16, Accesses>,
    resources: &BTreeSet<u16>,
) -> Option<(Vec<String>, Split)> {
    if resources.contains(&slot) {
        return None;
    }
    let mut sorted: Vec<(u32, u32, String)> = records
        .iter()
        .map(|record| {
            let (start, end) = record.range()?;
            Some((start, end, record.name().to_string()))
        })
        .collect::<Option<Vec<_>>>()?;
    sorted.sort();
    // Two ranges that touch (`end == next.start`) are disjoint; two that overlap are not, and a
    // record that covers no BCI states no range a variable is visible over.
    if sorted.windows(2).any(|window| window[0].1 > window[1].0)
        || sorted.iter().any(|(start, end, _)| start >= end)
    {
        return None;
    }
    let empty = Accesses::default();
    let accesses = accesses.get(&slot).unwrap_or(&empty);
    let mut by_bci: BTreeMap<u32, u16> = BTreeMap::new();
    let mut owner: BTreeMap<u32, BTreeSet<u16>> = BTreeMap::new();
    for read in &accesses.reads {
        let index = sorted
            .iter()
            .position(|(start, end, _)| *start <= read.bci && read.bci < *end)?;
        let index = u16::try_from(index).ok()?;
        let store = store_producing(ssa, read)?;
        owner.entry(store).or_default().insert(index);
        by_bci.insert(read.bci, index);
    }
    for write in &accesses.writes {
        let owners = owner.get(&write.bci)?;
        // Two variables' reads consume one store's value: the store is not one variable's alone, so
        // neither of the two names is the whole truth about it.
        if owners.len() != 1 {
            return None;
        }
        let index = *owners.iter().next()?;
        match by_bci.insert(write.bci, index) {
            // A read and a write at one BCI (`iinc`) have to agree on the variable they touch.
            Some(assigned) if assigned != index => return None,
            _ => {}
        }
    }
    let ranges: Vec<(u32, u32)> = sorted
        .iter()
        .map(|(start, end, _)| (*start, *end))
        .collect();
    let names: Vec<String> = sorted.into_iter().map(|(_, _, name)| name).collect();
    Some((names, Split { by_bci, ranges }))
}

/// The BCI of the store that wrote the value a read reads, when that store is in the read's own
/// block.
///
/// The scan is the builder's own: the block's entry state for the slot, then every write before the
/// read. A read whose value nothing in its block wrote came from a predecessor, so this run cannot
/// say which store filled the variable the read names — and a store it cannot place is a split it
/// does not take.
fn store_producing(ssa: &SsaTable, read: &Access) -> Option<u32> {
    let block = ssa.blocks().get(read.block)?;
    let mut in_use = block.entry().iter().find_map(|(slot, value)| match slot {
        Slot::Local(slot) if *slot == read.slot => Some((*value, None)),
        _ => None,
    });
    for instruction in block.instructions() {
        if instruction.bci() >= read.bci {
            break;
        }
        for (slot, value) in instruction.writes() {
            if matches!(slot, Slot::Local(slot) if *slot == read.slot) {
                in_use = Some((*value, Some(instruction.bci())));
            }
        }
    }
    match in_use {
        Some((held, Some(bci))) if held == read.value => Some(bci),
        _ => None,
    }
}

#[cfg(test)]
mod reference_split_name_tests {
    use super::*;

    fn fixture() -> (Vec<DebugLocal>, Vec<u32>, Split) {
        let records = vec![
            DebugLocal::over(5, "value", 32, 41),
            DebugLocal::over(5, "value", 74, 82),
        ];
        let read_bcis = vec![35, 77];
        let split = Split {
            by_bci: BTreeMap::from([(30, 0), (35, 0), (72, 1), (77, 1)]),
            ranges: vec![(0, 72), (72, u32::MAX)],
        };
        (records, read_bcis, split)
    }

    fn test_budget(analysis_steps: u64) -> Budget {
        Budget::new(jarde_reader::budget::Limits {
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
            analysis_steps,
            normalization_clones: 1 << 20,
            nested_depth: 32,
            dependency_depth: 32,
            elapsed_millis: u64::MAX,
        })
    }

    const REFERENCE_ALIAS_BOUNDARY_CLASS: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-reference-alias-boundary/ReferenceAliasBoundary.class"
    );
    const RAW_ALIAS_ESCAPE_CLASS: &[u8] = include_bytes!(
        "../../../tests/fixtures/cf11-structured-list-controls/UpstreamNestedLoops.class"
    );

    fn analyze_reference_method(
        class: &[u8],
        name: &[u8],
        descriptor: &[u8],
        java_release: u16,
    ) -> jarde_jvm::method_ir::MethodIrAnalysis {
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

        let mut budget = test_budget(1 << 20);
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
            .expect("actual alias-boundary class opens");
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class).to_hex().to_string()),
                length: u64::try_from(class.len()).expect("class length fits u64"),
            },
            variant: PhysicalVariant::Base,
        };
        let method = PhysicalMethodId {
            owner: definition,
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        };
        let domain = LoadDomain {
            loader: LoaderId("app".to_owned()),
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
                        java_release,
                        multi_release: MultiReleasePolicy::Disabled,
                        layout: LayoutMode::Generic,
                    },
                    load_domain: domain.clone(),
                },
                domains: vec![domain],
                providers: Vec::new(),
            },
            method,
            stages: AnalysisStage::ALL.to_vec(),
        };
        analyze_method_ir(&[snapshot], &request, &mut budget)
            .expect("actual method completes SSA analysis")
    }

    fn analyze_reference_alias_boundary() -> jarde_jvm::method_ir::MethodIrAnalysis {
        analyze_reference_method(
            REFERENCE_ALIAS_BOUNDARY_CLASS,
            b"aliasSurvivesLaterStore",
            b"()V",
            8,
        )
    }

    fn reference_alias_debug_locals(
        code: &jarde_reader::classfile::MethodCodeFacts,
    ) -> Vec<DebugLocal> {
        let jarde_reader::classfile::LocalDebugTable::Read(records) = code.debug() else {
            return Vec::new();
        };
        records
            .iter()
            .map(|record| {
                DebugLocal::over(
                    record.slot,
                    record.name_lossy(),
                    record.start_bci,
                    record.end_bci,
                )
            })
            .collect()
    }

    #[test]
    fn actual_alias_surviving_a_later_store_is_refused_by_the_ssa_gate_and_plan() {
        use crate::ast::Type;

        let analysis = analyze_reference_alias_boundary();
        let ir = analysis.ir();
        let canonical = ir.canonical().expect("actual canonical graph");
        let ssa = ir.ssa().expect("actual SSA");
        let code = ir.code().expect("actual bytecode facts");
        let operations = Operations::of(code, ir.constant_pool());
        let slot = accesses(ssa).remove(&0).expect("slot 0 accesses");
        assert_eq!(
            slot.writes
                .iter()
                .map(|write| write.bci)
                .collect::<Vec<_>>(),
            vec![4, 12],
            "javac emits the two reference stores in bytecode order"
        );
        assert_eq!(
            slot.reads.iter().map(|read| read.bci).collect::<Vec<_>>(),
            vec![5],
            "the first value is loaded before the second store and consumed by the call"
        );
        let definitions = slot
            .writes
            .iter()
            .map(|write| {
                (
                    write.bci,
                    write.value,
                    crate::build::reference_lifetime_type(ssa, &operations, write.value)
                        .expect("each actual store has an exact reference type"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            definitions
                .iter()
                .map(|(_, _, ty)| ty.clone())
                .collect::<Vec<_>>(),
            vec![
                Type::Reference("java.lang.Integer".to_owned()),
                Type::Reference("java.lang.String".to_owned()),
            ]
        );
        let mut alias_budget = test_budget(1 << 20);
        assert!(
            !reference_values_stay_before_boundaries(
                ssa,
                &operations,
                &definitions,
                &mut alias_budget,
            )
            .unwrap(),
            "the value loaded at BCI 5 remains on the operand stack for the call at BCI 13"
        );

        let declaration = ir.declaration().expect("actual method declaration");
        let mut region_budget = test_budget(1 << 20);
        let normal = crate::normal_flow::NormalFlowView::build(canonical, &mut region_budget)
            .expect("actual normal-flow view");
        let regions = crate::region::recover(
            canonical,
            &normal,
            ssa,
            &operations,
            ir.constant_pool(),
            &crate::concat::Plan::empty(),
            &crate::init::Sites::empty(),
            code,
            Some(declaration.access_flags() & 0x0020 != 0),
            false,
            &crate::pass::JAVA_8,
            &mut region_budget,
        )
        .expect("actual method regions")
        .regions;
        let debug = reference_alias_debug_locals(code);
        assert_eq!(
            debug
                .iter()
                .filter(|record| record.slot() == 0)
                .map(DebugLocal::range)
                .collect::<Vec<_>>(),
            vec![Some((5, 17))],
            "one actual LVT row deliberately makes a two-row candidate impossible"
        );
        let resources = BTreeSet::new();
        let plan_for = |debug: &[DebugLocal]| {
            let mut plan_budget = test_budget(1 << 20);
            plan(
                ssa,
                canonical,
                &operations,
                &regions,
                code.max_locals,
                declaration.parameter_slots(),
                debug,
                &resources,
                &mut plan_budget,
            )
            .expect("the complete reuse plan refuses safely")
        };
        let actual_lvt_plan = plan_for(&debug);
        assert!(
            !actual_lvt_plan.splits.contains_key(&0),
            "the actual one-row table must not publish a split identity"
        );

        let no_lvt_plan = plan_for(&[]);
        assert!(
            !no_lvt_plan.splits.contains_key(&0),
            "the complete no-LVT path must still reject the surviving alias"
        );

        let matching_lvt = [
            DebugLocal::over(0, "first", 5, 12),
            DebugLocal::over(0, "second", 13, 17),
        ];
        assert_eq!(matching_lvt.len(), slot.writes.len());
        let matching_lvt_plan = plan_for(&matching_lvt);
        assert!(
            !matching_lvt_plan.splits.contains_key(&0),
            "matching non-overlapping rows must not bypass the actual alias refusal"
        );
    }

    #[test]
    fn actual_unused_raw_alias_identity_requires_exact_store_boundary_and_owner() {
        let analysis = analyze_reference_method(
            RAW_ALIAS_ESCAPE_CLASS,
            b"rawAliasEscape",
            b"(Ljava/util/List;Ljava/util/List;)V",
            23,
        );
        let ir = analysis.ir();
        let canonical = ir.canonical().expect("actual canonical graph");
        let ssa = ir.ssa().expect("actual SSA");
        let code = ir.code().expect("actual bytecode facts");
        let operations = Operations::of(code, ir.constant_pool());
        let slot = accesses(ssa).remove(&5).expect("actual slot-5 accesses");
        assert_eq!(
            slot.reads.iter().map(|read| read.bci).collect::<Vec<_>>(),
            vec![33],
            "the actual second foreach identity has no local load"
        );
        assert_eq!(
            slot.writes
                .iter()
                .map(|write| write.bci)
                .collect::<Vec<_>>(),
            vec![30, 72]
        );
        let definitions = slot
            .writes
            .iter()
            .map(|write| {
                (
                    write.bci,
                    write.value,
                    crate::build::reference_lifetime_type(ssa, &operations, write.value)
                        .expect("actual store has an exact reference type"),
                )
            })
            .collect::<Vec<_>>();
        let mut split_budget = test_budget(1 << 20);
        let split =
            owned_definition_split(ssa, &slot, definitions, Some(canonical), &mut split_budget)
                .expect("actual ownership proof completes")
                .expect("actual owner split is proven");
        assert_eq!(split.ranges, vec![(0, 72), (72, u32::MAX)]);

        let records = reference_alias_debug_locals(code)
            .into_iter()
            .filter(|record| record.slot() == 5)
            .collect::<Vec<_>>();
        assert_eq!(
            records.iter().map(DebugLocal::range).collect::<Vec<_>>(),
            vec![Some((32, 41)), Some((74, 82))]
        );
        let run_names = |rows: &[DebugLocal], reads: &[Access], writes: &[Access]| {
            let rows = rows.iter().collect::<Vec<_>>();
            let mut budget = test_budget(1 << 20);
            reference_split_names(
                &rows,
                reads.iter().map(|read| read.bci),
                writes,
                Some(&operations),
                &split,
                &mut budget,
            )
        };
        assert_eq!(
            run_names(&records, &slot.reads, &slot.writes).unwrap(),
            Some(vec![Some("value".to_owned()), Some("value".to_owned())])
        );

        let non_adjacent = [records[0].clone(), DebugLocal::over(5, "value", 75, 82)];
        assert_eq!(
            run_names(&non_adjacent, &slot.reads, &slot.writes).unwrap(),
            None,
            "the unread identity cannot be named by a nearby but non-adjacent row"
        );

        let crosses_identity_range = [DebugLocal::over(5, "value", 32, 73), records[1].clone()];
        assert_eq!(
            run_names(&crosses_identity_range, &[], &slot.writes).unwrap(),
            None,
            "an unread row cannot extend across the next proven identity boundary"
        );

        let mut wrong_slot = slot.writes.clone();
        wrong_slot[1].slot = 6;
        assert_eq!(
            run_names(&records, &slot.reads, &wrong_slot).unwrap(),
            None,
            "the decoded Store must write the same local slot as the proof input"
        );

        let multiple_owner_writes = [slot.writes[0], slot.writes[1], slot.writes[1]];
        assert_eq!(
            run_names(&records, &slot.reads, &multiple_owner_writes).unwrap(),
            None,
            "an unread identity with multiple candidate owner writes is ambiguous"
        );
    }

    #[test]
    fn same_name_rows_match_proven_read_identities_and_leave_initial_stores_to_ssa() {
        let (records, read_bcis, split) = fixture();
        let records = records.iter().collect::<Vec<_>>();
        let mut budget = test_budget(100);
        assert_eq!(
            reference_split_names(
                &records,
                read_bcis.iter().copied(),
                &[],
                None,
                &split,
                &mut budget,
            )
            .unwrap(),
            Some(vec![Some("value".to_owned()), Some("value".to_owned())])
        );
        assert_eq!(split.by_bci.get(&30), Some(&0));
        assert_eq!(split.by_bci.get(&72), Some(&1));
        assert!(records[0].range().unwrap().0 > 30);
        assert!(records[1].range().unwrap().0 > 72);
    }

    #[test]
    fn no_lvt_keeps_the_existing_unnamed_identity_split() {
        let (_, read_bcis, split) = fixture();
        let mut budget = test_budget(0);
        assert_eq!(
            reference_split_names(
                &[],
                read_bcis.iter().copied(),
                &[],
                None,
                &split,
                &mut budget,
            )
            .unwrap(),
            Some(vec![None, None])
        );
        assert_eq!(budget.usage().analysis_steps, 0);
    }

    #[test]
    fn malformed_or_incomplete_lvt_mapping_refuses_without_guessing() {
        let (mut records, read_bcis, split) = fixture();
        let reject = |rows: &[DebugLocal]| {
            let rows = rows.iter().collect::<Vec<_>>();
            let mut budget = test_budget(100);
            assert_eq!(
                reference_split_names(
                    &rows,
                    read_bcis.iter().copied(),
                    &[],
                    None,
                    &split,
                    &mut budget,
                )
                .unwrap(),
                None
            );
        };
        reject(&[records[0].clone(), DebugLocal::over(5, "value", 40, 82)]);
        reject(&[records[0].clone(), DebugLocal::over(5, "value", 74, 74)]);
        reject(&[
            DebugLocal::over(5, "first", 32, 41),
            DebugLocal::over(5, "second", 32, 41),
        ]);
        reject(&records[..1]);
        records.push(DebugLocal::over(5, "extra", 90, 100));
        reject(&records);
        reject(&[
            DebugLocal::over(5, "value", 0, 20),
            DebugLocal::over(5, "value", 20, 30),
        ]);
        let no_second_read = read_bcis[..1].iter().copied();
        let rows = records[..2].iter().collect::<Vec<_>>();
        let mut budget = test_budget(100);
        assert_eq!(
            reference_split_names(&rows, no_second_read, &[], None, &split, &mut budget).unwrap(),
            None
        );
    }

    #[test]
    fn valid_rows_and_reads_are_accepted_when_input_order_is_reversed() {
        let (records, read_bcis, split) = fixture();
        let records = records.iter().rev().collect::<Vec<_>>();
        let read_bcis = read_bcis.iter().rev().copied();
        let mut budget = test_budget(100);
        assert_eq!(
            reference_split_names(&records, read_bcis, &[], None, &split, &mut budget).unwrap(),
            Some(vec![Some("value".to_owned()), Some("value".to_owned())])
        );
    }

    #[test]
    fn a_debug_row_cannot_override_the_proven_read_owner() {
        let (records, read_bcis, mut split) = fixture();
        let records = records.iter().collect::<Vec<_>>();
        split.by_bci.insert(read_bcis[1], 0);
        let mut budget = test_budget(100);
        assert_eq!(
            reference_split_names(
                &records,
                read_bcis.iter().copied(),
                &[],
                None,
                &split,
                &mut budget,
            )
            .unwrap(),
            None,
            "the debug row cannot override the owner already proved for the read"
        );
    }

    #[test]
    fn matching_budget_stop_and_cancellation_publish_no_partial_names() {
        let (records, read_bcis, split) = fixture();
        let records = records.iter().collect::<Vec<_>>();
        let mut stopped = test_budget(3);
        assert!(matches!(
            reference_split_names(
                &records,
                read_bcis.iter().copied(),
                &[],
                None,
                &split,
                &mut stopped,
            ),
            Err(StopReason::Budget {
                dimension: CountedBudgetDimension::AnalysisSteps,
                ..
            })
        ));
        assert_eq!(stopped.usage().analysis_steps, 3);

        let mut cancelled = test_budget(100);
        cancelled.cancellation_token().cancel();
        assert!(matches!(
            reference_split_names(
                &records,
                read_bcis.iter().copied(),
                &[],
                None,
                &split,
                &mut cancelled,
            ),
            Err(StopReason::Cancelled { .. })
        ));
        assert_eq!(cancelled.usage().analysis_steps, 0);
    }
}
