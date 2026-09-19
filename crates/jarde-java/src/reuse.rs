//! Which variable one local slot holds, and where (P3 3.4, the `source-maps` scenario
//! `Slot reuse across ranges`).
//!
//! A slot is **one** variable unless the `LocalVariableTable` names it over two disjoint ranges with
//! two different names — what a compiler writes when it reuses one storage location for two source
//! variables whose scopes do not overlap. A record states two things about such a variable: its
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
//! The split is taken only when that assignment is **complete and unambiguous**: every read sits
//! inside exactly one record's range, and every write's value is read by the in-range reads of
//! exactly one record. Everything else keeps the slot **one** variable — the same answer a body with
//! no debug metadata gets, and the answer [`crate::names`]'s rule 1 requires when the evidence is
//! missing rather than merely inconvenient:
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

use jarde_jvm::method_ir::{Slot, SsaTable, ValueId};

use crate::names::{DebugLocal, LocalVariable, SlotEvidence};

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
    slots: u16,
    debug: &[DebugLocal],
    resources: &BTreeSet<u16>,
) -> Plan {
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
    for (slot, slot_records) in &records {
        let index = usize::from(*slot);
        let names: BTreeSet<&str> = slot_records.iter().map(|record| record.name()).collect();
        if names.len() <= 1 {
            plan.evidence[index] = SlotEvidence::Whole(slot_records[0].name().to_string());
            continue;
        }
        match split(ssa, *slot, slot_records, &accesses, resources) {
            Some((names, split)) => {
                plan.evidence[index] = SlotEvidence::Split(names);
                plan.splits.insert(*slot, split);
            }
            None => plan.evidence[index] = SlotEvidence::Unnamed,
        }
    }
    plan
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
