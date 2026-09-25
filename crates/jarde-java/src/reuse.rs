//! Which variable one local slot holds, and where (P3 3.4, the `source-maps` scenario
//! `Slot reuse across ranges`).
//!
//! A slot defaults to **one** variable. Two proofs can split it: distinct LVT names over disjoint
//! ranges, or a narrow SSA/CFG proof that a reference lifetime ends before an int lifetime begins
//! (or the reverse). The second proof works when only the later variable has an LVT record or the
//! class has no debug table; a name is attached to only the lifetime its reads cover. It requires
//! separate definition/use chains, only trivial phi aliases, no handler/call-context edge and no
//! path from the later lifetime back to an earlier access. BCI order alone is insufficient.
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
//! * a **write** belongs to the record whose in-range reads read the value that write stored — the
//!   store and the loads that consume it are one value chain, so the load's position decides the
//!   store's variable even where the store's own BCI is outside every record.
//!
//! The LVT split is taken only when that assignment is **complete and unambiguous**: every read sits
//! inside exactly one record's range, and every write's value is read by the in-range reads of
//! exactly one record. When neither proof applies, the slot remains **one** variable. The LVT path
//! declines these shapes rather than guessing which record names the storage location:
//!
//! * one record for the slot, or several that all state the *same* name: one variable, that name;
//! * ranges that overlap, a record that states no range, or names this run cannot place: one
//!   variable, **no** name — neither record's spelling is the truth about the whole storage location,
//!   so the ordinal name is written instead (P3 3.1's rule, kept where the evidence is not enough);
//! * a read outside every range, or a write whose value no in-range read consumes: the same. A store
//!   this run cannot trace to a load is not a store it can place, and placing it wrongly would write
//!   one variable's name for another's value.
//!
//! The slot a guarded statement declares in its own header (P3 2.4) is never split: `try (T n = …)`
//! writes that declaration, so the header's slot stays one variable and the guard keeps its own rule
//! for naming it.

use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{CanonicalCfg, CanonicalEdgeKind, Slot, SsaTable, Value, ValueId};
use jarde_reader::budget::{Budget, CountedBudgetDimension};

use crate::names::{DebugLocal, LocalVariable, SlotEvidence};
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
    slots: u16,
    parameters: u16,
    debug: &[DebugLocal],
    resources: &BTreeSet<u16>,
    budget: &mut Budget,
) -> Result<Plan, StopReason> {
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
    if (0..2).any(|part| {
        writes[part].is_empty()
            || reads[part].is_empty()
            || !writes[part].is_subset(&reads[part])
            || !reads[part].is_subset(&writes[part])
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
        let Some(replaced) = ssa.value(phi.value()).replaced_by() else {
            return Ok(None);
        };
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
        if edge.kind() != CanonicalEdgeKind::Normal {
            return Ok(None);
        }
        edges
            .entry(edge.from().clone())
            .or_insert_with(Vec::new)
            .push(edge.to().clone());
    }
    let earlier_blocks: BTreeSet<_> = typed
        .iter()
        .filter(|(_, kind)| *kind == earlier)
        .map(|(access, _)| ssa.blocks()[access.block].block().clone())
        .collect();
    let mut pending: Vec<_> = typed
        .iter()
        .filter(|(_, kind)| *kind == later)
        .map(|(access, _)| ssa.blocks()[access.block].block().clone())
        .collect();
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
