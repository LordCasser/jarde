//! 4.3's independent oracle: a naive reaching-definition data flow over the pass's own inputs, and
//! the comparison that reads the published table through it.
//!
//! This module exists for one reason: the names 4.3 publishes are only as good as the argument that
//! they are the names the artifacts imply, and an argument that audits a table with the code that
//! built it is no audit at all. So the oracle here is a **second, deliberately naive** reading of
//! the same body:
//!
//! - it takes only **inputs**: the canonical graph, the frames 4.1 published, the slot accesses the
//!   frame replay records ([`block_touches`]) and the method's own entry state. It classifies no
//!   opcode, and it never asks the naming half what it decided;
//! - it computes the **reaching definitions** of every block entry, block exit and read by Kleene
//!   iteration from the empty set — one round of the whole graph at a time, until nothing changes —
//!   instead of the demand-driven worklist the pass runs. Nothing is shared with the pass here: no
//!   helper of the naming half is called, and the semantic half's own notion of which input may
//!   take part in a merge is not consulted either (that judgement is the thing under test);
//! - it names a definition by the identity a consumer outside can state — the block and the BCI
//!   that wrote it, the method's own entry, or the throw site that caught it — and never by an SSA
//!   index. The comparison therefore cancels the two things the two algorithms legitimately
//!   disagree about: the numbering of values and how many trivial phis a merge needed.
//!
//! The comparison is what turns the oracle into evidence. For every slot the frames state a value
//! in, it **projects** the published name onto the same set: a phi is expanded into its operands
//! until instructions, the entry state and caught references are reached, and a phi that was
//! removed as trivial is followed through `replaced_by` to the value that took its name. The two
//! sets must be **equal**; anything else is reported with the block, the slot and the definitions
//! that are missing or extra on the published side, because a bare "false" would say nothing about
//! which half is wrong.
//!
//! Two properties decide what that equality may be read as:
//!
//! - **A definition is found, not guessed.** The projection of an instruction definition recovers
//!   its slot from the write record that names it, so a value that reached a slot the oracle never
//!   saw a write for is a disagreement and not a rounding error.
//! - **An exception input is one throw site.** The oracle takes its predecessors from the frames'
//!   logical inputs and, for an exception input, from the state the throwing instruction was at —
//!   never from the source's exit and never from the count of aggregated canonical edges. A body
//!   whose block holds two throw sites that one record catches into one handler therefore reaches
//!   the handler with two definitions of the same slot, exactly as the pass must have found them.
//!
//! The [`Flaws`] knobs are the falsification instruments: each one weakens the oracle or the
//! projection in a way a plausible mistake would, and the tests assert that the comparison turns
//! **red** under it. A comparison that only ever agrees proves nothing; these are the cases that
//! show it can fail.

use std::collections::{BTreeMap, BTreeSet};

use jarde_reader::budget::Budget;
use jarde_reader::classfile::MethodCodeFacts;

use crate::canonical::{CanonicalBlockId, CanonicalCfg};
use crate::frame::{
    BlockFrame, FrameMethod, FrameTable, InstructionTouches, SlotRegion, TouchesOutcome,
    block_touches, entry_slots, starts_value,
};
use crate::ssa::{Definition, PhiInput, Slot, SsaTable, ValueId};

/// One definition of the oracle's own model: the identity a consumer outside can state.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Def {
    /// The method's own entry state: a parameter, `this`, or a value the caller hands the method.
    Param { block: CanonicalBlockId, slot: Slot },
    /// The value one instruction leaves in one slot.
    Store {
        block: CanonicalBlockId,
        bci: u32,
        slot: Slot,
    },
    /// The exception reference one throw site hands one handler.
    Caught { block: CanonicalBlockId, bci: u32 },
}

impl Def {
    /// One line naming the definition, for a report.
    fn describe(&self) -> String {
        match self {
            Self::Param { block, slot } => format!("entry of {block:?} {slot:?}"),
            Self::Store { block, bci, slot } => {
                format!("instruction {block:?}@{bci} into {slot:?}")
            }
            Self::Caught { block, bci } => format!("caught at {block:?}@{bci}"),
        }
    }
}

/// The deliberate weakenings the falsification tests run the oracle and the projection under.
///
/// Every one of them is off in the shape the tests use for their positive assertions, and each is
/// the shape a plausible mistake takes: reading the raw edge instead of the throw site, walking the
/// block list instead of the graph, or parsing one slot less out of the published table.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Flaws {
    /// An exception input contributes the source block's **exit** state — what the aggregated raw
    /// edge carries — instead of the state its named throw site was at.
    aggregate_exception_inputs: bool,
    /// Only edges whose source precedes their destination in the block list are walked, which is
    /// what a naive reading of a "forward" graph does and which drops every loop.
    forward_edges_only: bool,
    /// The projection of the published table forgets the definition of this slot, as a comparator
    /// that parses one slot less would.
    drop_a_written_slot: Option<Slot>,
}

impl Flaws {
    /// The oracle as it is meant to be read.
    fn none() -> Self {
        Self::default()
    }

    /// The oracle that reads an exception input as the aggregated raw edge.
    fn aggregated() -> Self {
        Self {
            aggregate_exception_inputs: true,
            ..Self::none()
        }
    }

    /// The oracle that walks forward edges only.
    fn forward_only() -> Self {
        Self {
            forward_edges_only: true,
            ..Self::none()
        }
    }

    /// The projection that never finds a definition written into one slot.
    fn dropping(slot: Slot) -> Self {
        Self {
            drop_a_written_slot: Some(slot),
            ..Self::none()
        }
    }
}

/// The reaching definitions of one slot at one point of one body.
type Reach = BTreeMap<Slot, BTreeSet<Def>>;

/// One logical predecessor of one block, in the shape the frames publish it.
///
/// A logical input is finer than a canonical edge: one exception record's edge covers every throw
/// site it protects, and each of those sites hands the handler its own state, so the oracle keeps
/// them apart by BCI exactly like the frames do. The ordinal the record has in the exception table
/// is deliberately **not** part of the oracle's model: a caught reference is published as
/// `(block, BCI)` alone, so that is the identity both sides are compared on.
#[derive(Clone, Debug, Eq, PartialEq)]
enum OracleEdge {
    /// The method's own entry state, which participates in the entry block like an edge does.
    Param,
    /// A plain transfer: the source block's exit state.
    Transfer { from: usize },
    /// One throw site of one source block.
    Throw { from: usize, bci: u32 },
}

/// One read the oracle walked, in the order the block's instructions performed it.
#[derive(Clone, Debug, Eq, PartialEq)]
struct OracleRead {
    bci: u32,
    slot: Slot,
    /// The definitions that may be in the slot at that point, in the identity order.
    sources: BTreeSet<Def>,
}

/// Everything the oracle knows about one block at the fixed point.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct OracleState {
    /// The definitions reaching the block's entry, per slot.
    ///
    /// A slot the frames call `Top` may hold definitions here: 4.1 merges a local to `Top` as soon
    /// as **one** contributor has none, so the block's entry state is a conservative reading of the
    /// flows, and the oracle keeps the flows. The comparison only reads the slots the frames state
    /// a value in, which is where all contributors must have one.
    entry: Reach,
    /// The definitions reaching the block's exit, per slot.
    exit: Reach,
    /// The locals reaching the point **before** each of this block's throw sites, by BCI.
    snapshots: BTreeMap<u32, Reach>,
    /// One record per read of the block, in execution order.
    reads: Vec<OracleRead>,
}

/// One block, as the oracle's own graph holds it.
struct OracleBlock {
    id: CanonicalBlockId,
    /// Position in the block list, which is what "a forward edge" is measured against.
    position: usize,
    edges: Vec<OracleEdge>,
    /// The slots the method's own entry state starts values in, for the block that has that input.
    param_slots: Vec<Slot>,
    /// The BCIs of this block's instructions that may raise, ascending.
    sites: Vec<u32>,
    instructions: Vec<InstructionTouches>,
}

/// The oracle's own view of one body: the graph it walks and the fixed point it reaches.
struct Oracle {
    blocks: Vec<OracleBlock>,
    states: Vec<OracleState>,
}

/// The slot one recorded access names.
fn slot_of(region: SlotRegion, slot: u32) -> Slot {
    match region {
        SlotRegion::Local => Slot::Local(u16::try_from(slot).unwrap_or(u16::MAX)),
        SlotRegion::Stack => Slot::Stack(slot),
    }
}

/// The slot above this one, within its own region.
fn above(slot: Slot) -> Option<Slot> {
    match slot {
        Slot::Local(index) => index.checked_add(1).map(Slot::Local),
        Slot::Stack(depth) => depth.checked_add(1).map(Slot::Stack),
    }
}

/// The slot below this one, within its own region.
fn below(slot: Slot) -> Option<Slot> {
    match slot {
        Slot::Local(0) | Slot::Stack(0) => None,
        Slot::Local(index) => Some(Slot::Local(index - 1)),
        Slot::Stack(depth) => Some(Slot::Stack(depth - 1)),
    }
}

/// Adds one definition to one slot's reach.
fn add(reach: &mut Reach, slot: Slot, def: Def) {
    reach.entry(slot).or_default().insert(def);
}

/// Unions one reach into another.
fn merge(reach: &mut Reach, other: &Reach) {
    for (slot, defs) in other {
        reach.entry(*slot).or_default().extend(defs.iter().cloned());
    }
}

/// The locals of one reach, which is all an exception input may hand a handler.
fn locals_only(reach: &Reach) -> Reach {
    reach
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Local(_)))
        .map(|(slot, defs)| (*slot, defs.clone()))
        .collect()
}

/// The slots one block's entry state starts values in, ascending: the domain the frames state and
/// therefore the domain the comparison may read.
fn stated_slots(state: &BlockFrame) -> Vec<Slot> {
    let mut slots = Vec::new();
    for (index, value) in state.locals.iter().enumerate() {
        if starts_value(value) {
            slots.push(Slot::Local(u16::try_from(index).unwrap_or(u16::MAX)));
        }
    }
    for (depth, value) in state.stack.iter().enumerate() {
        if starts_value(value) {
            slots.push(Slot::Stack(u32::try_from(depth).unwrap_or(u32::MAX)));
        }
    }
    slots
}

/// The naive data flow: every block's entry, exit, throw-site snapshots and reads, iterated from
/// the empty reach until the whole graph stops changing.
///
/// The iteration is Kleene's, one whole-graph round at a time, over the least fixpoint: a round
/// reads the state the previous round left, so a loop's carried value grows in from its entry and
/// a cycle that no path defines stays empty instead of inventing a definition. The transfer inside
/// one block is a single forward walk — reads see the reach they were at, writes replace one slot
/// with one definition, and the operand stack above the depth the frame replay measured is dropped —
/// and it is monotone in the entry, which is what makes the iteration terminate.
fn reach_the_fixpoint(
    facts: &MethodCodeFacts,
    canonical: &CanonicalCfg,
    frames: &FrameTable,
    method: &FrameMethod<'_>,
    flaws: &Flaws,
    budget: &mut Budget,
) -> Oracle {
    let index_of: BTreeMap<CanonicalBlockId, usize> = canonical
        .blocks
        .iter()
        .filter(|block| frames.entry(&block.id).is_some())
        .enumerate()
        .map(|(position, block)| (block.id.clone(), position))
        .collect();
    let entry_id = canonical
        .blocks
        .iter()
        .map(|block| block.id.clone())
        .find(|id| id.bci == 0 && id.path.is_empty());
    let param_slots: Vec<Slot> = match entry_slots(method, facts) {
        Ok(Some(slots)) => slots
            .into_iter()
            .map(|slot| slot_of(slot.region, slot.slot))
            .collect(),
        other => panic!("the oracle needs the method's own entry state, got {other:?}"),
    };
    let mut blocks = Vec::new();
    for frame in frames.blocks() {
        let Some(graph_block) = canonical
            .blocks
            .iter()
            .find(|block| block.id == frame.block)
        else {
            panic!(
                "the frames hold {:?}, which the graph does not",
                frame.block
            );
        };
        let position = blocks.len();
        let mut edges = Vec::new();
        if entry_id.as_ref() == Some(&frame.block) && !param_slots.is_empty() {
            edges.push(OracleEdge::Param);
        }
        for input in &frame.inputs {
            let Some(from) = index_of.get(&input.from).copied() else {
                panic!(
                    "an input of {:?} comes from {:?}, which the frames do not hold",
                    frame.block, input.from
                );
            };
            edges.push(match input.throw_site {
                None => OracleEdge::Transfer { from },
                Some(bci) => OracleEdge::Throw { from, bci },
            });
        }
        let instructions = match block_touches(facts, graph_block, frame, method, budget) {
            Ok(TouchesOutcome::Touches(instructions)) => instructions,
            other => panic!(
                "the oracle reads the accesses of {:?}: {other:?}",
                frame.block
            ),
        };
        let sites = canonical
            .throw_sites
            .iter()
            .filter(|site| site.block == frame.block)
            .map(|site| site.bci)
            .collect();
        blocks.push(OracleBlock {
            id: frame.block.clone(),
            position,
            edges,
            param_slots: if entry_id.as_ref() == Some(&frame.block) {
                param_slots.clone()
            } else {
                Vec::new()
            },
            sites,
            instructions,
        });
    }
    assert_eq!(
        blocks.len(),
        frames.blocks().len(),
        "the oracle walked every block the frames hold"
    );
    let rounds = 4 * blocks.len() + 8;
    let mut states = vec![OracleState::default(); blocks.len()];
    for _ in 0..rounds {
        let next: Vec<OracleState> = blocks
            .iter()
            .map(|block| walk(block, &blocks, &states, flaws))
            .collect();
        if next == states {
            return Oracle { blocks, states };
        }
        states = next;
    }
    panic!(
        "the oracle's data flow over {} blocks did not reach a fixed point",
        blocks.len()
    );
}

/// One block's transfer: the entry its edges merge to, then the walk over its instructions.
fn walk(
    block: &OracleBlock,
    blocks: &[OracleBlock],
    states: &[OracleState],
    flaws: &Flaws,
) -> OracleState {
    let mut entry: Reach = BTreeMap::new();
    for edge in &block.edges {
        match edge {
            OracleEdge::Param => {
                for slot in &block.param_slots {
                    add(
                        &mut entry,
                        *slot,
                        Def::Param {
                            block: block.id.clone(),
                            slot: *slot,
                        },
                    );
                }
            }
            OracleEdge::Transfer { from } => {
                if flaws.forward_edges_only && *from >= block.position {
                    continue;
                }
                merge(&mut entry, &states[*from].exit);
            }
            OracleEdge::Throw { from, bci } => {
                if flaws.forward_edges_only && *from >= block.position {
                    continue;
                }
                let source = &states[*from];
                if flaws.aggregate_exception_inputs {
                    // The raw edge's own reading: the source's state at the end of the block, which
                    // is what an oracle that never looked at the throw site would hand the handler.
                    merge(&mut entry, &locals_only(&source.exit));
                } else if let Some(snapshot) = source.snapshots.get(bci) {
                    merge(&mut entry, snapshot);
                }
                add(
                    &mut entry,
                    Slot::Stack(0),
                    Def::Caught {
                        block: blocks[*from].id.clone(),
                        bci: *bci,
                    },
                );
            }
        }
    }
    let mut current = entry.clone();
    let mut snapshots = BTreeMap::new();
    let mut reads = Vec::new();
    let mut wide: BTreeSet<Slot> = BTreeSet::new();
    for instruction in &block.instructions {
        if block.sites.contains(&instruction.bci) {
            // The state **before** the instruction runs, which is what an exception edge out of this
            // site carries: reaching this line first is the whole point of the snapshot.
            snapshots.insert(instruction.bci, locals_only(&current));
        }
        for touch in &instruction.accesses {
            let slot = slot_of(touch.region, touch.slot);
            if touch.write {
                current.remove(&slot);
                if touch.width == 2 {
                    if let Some(upper) = above(slot) {
                        current.remove(&upper);
                    }
                    wide.insert(slot);
                } else {
                    wide.remove(&slot);
                    // The frame pass ends both halves of a half-covered pair in `Top`, so a write
                    // over the lower half of a category-2 value takes that value away too.
                    if let Some(lower) = below(slot)
                        && wide.remove(&lower)
                    {
                        current.remove(&lower);
                    }
                }
                current.insert(
                    slot,
                    BTreeSet::from([Def::Store {
                        block: block.id.clone(),
                        bci: instruction.bci,
                        slot,
                    }]),
                );
            } else {
                reads.push(OracleRead {
                    bci: instruction.bci,
                    slot,
                    sources: current.get(&slot).cloned().unwrap_or_default(),
                });
            }
        }
        // Everything above the depth the frame replay measured for this instruction is gone: the
        // discards an instruction performs without reading anything (`athrow`, `return`) are stated
        // by that depth and not inferred from the accesses.
        current.retain(|slot, _| match slot {
            Slot::Stack(depth) => *depth < instruction.stack_after,
            Slot::Local(_) => true,
        });
        wide.retain(|slot| match slot {
            Slot::Stack(depth) => *depth < instruction.stack_after,
            Slot::Local(_) => true,
        });
    }
    OracleState {
        entry,
        exit: current,
        snapshots,
        reads,
    }
}

/// One place where the published table and the oracle disagree.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Mismatch {
    /// The published table and the frames do not name the same blocks, in the same order.
    Blocks {
        published: Vec<CanonicalBlockId>,
        frames: Vec<CanonicalBlockId>,
    },
    /// The oracle's own block list and the frames' do not name the same blocks.
    Walked {
        oracle: Vec<CanonicalBlockId>,
        frames: Vec<CanonicalBlockId>,
    },
    /// The published table names no entry for a block the frames hold.
    Unnamed { block: CanonicalBlockId },
    /// The published entry of one block names a different set of slots than the frames state.
    EntrySlots {
        block: CanonicalBlockId,
        published: Vec<Slot>,
        frames: Vec<Slot>,
    },
    /// The definitions reaching one block entry and slot differ.
    Entry {
        block: CanonicalBlockId,
        slot: Slot,
        published: BTreeSet<Def>,
        oracle: BTreeSet<Def>,
    },
    /// The definitions reaching one block exit and slot differ.
    Exit {
        block: CanonicalBlockId,
        slot: Slot,
        published: BTreeSet<Def>,
        oracle: BTreeSet<Def>,
    },
    /// The definitions a read may see differ.
    Read {
        block: CanonicalBlockId,
        bci: u32,
        slot: Slot,
        position: usize,
        published: BTreeSet<Def>,
        oracle: BTreeSet<Def>,
    },
    /// The published reads of one block are not the reads the oracle walked, in order and slot.
    ReadSlots {
        block: CanonicalBlockId,
        published: Vec<(u32, Slot)>,
        oracle: Vec<(u32, Slot)>,
    },
    /// A published value states a definition the table cannot be read through.
    Unstated { value: ValueId, message: String },
}

impl Mismatch {
    /// One line per disagreement, with the definitions that are missing or extra on each side.
    fn describe(&self) -> String {
        fn sides(published: &BTreeSet<Def>, oracle: &BTreeSet<Def>) -> String {
            let missing: Vec<String> = oracle.difference(published).map(Def::describe).collect();
            let extra: Vec<String> = published.difference(oracle).map(Def::describe).collect();
            format!(
                "missing on the published side [{}], extra on the published side [{}]",
                missing.join(", "),
                extra.join(", ")
            )
        }
        fn read(set: &BTreeSet<Def>) -> String {
            let named: Vec<String> = set.iter().map(Def::describe).collect();
            format!("[{}]", named.join(", "))
        }
        match self {
            Self::Blocks { published, frames } => format!(
                "the published table names the blocks {published:?} where the frames hold {frames:?}"
            ),
            Self::Walked { oracle, frames } => {
                format!("the oracle walked the blocks {oracle:?} where the frames hold {frames:?}")
            }
            Self::Unnamed { block } => {
                format!("the frames hold {block:?} and the published table names no entry for it")
            }
            Self::EntrySlots {
                block,
                published,
                frames,
            } => format!(
                "the entry of {block:?} publishes the slots {published:?} where the frames state {frames:?}"
            ),
            Self::Entry {
                block,
                slot,
                published,
                oracle,
            } => format!(
                "the entry of {block:?} {slot:?} publishes {}, the oracle reaches {}: {}",
                read(published),
                read(oracle),
                sides(published, oracle)
            ),
            Self::Exit {
                block,
                slot,
                published,
                oracle,
            } => format!(
                "the exit of {block:?} {slot:?} publishes {}, the oracle reaches {}: {}",
                read(published),
                read(oracle),
                sides(published, oracle)
            ),
            Self::Read {
                block,
                bci,
                slot,
                position,
                published,
                oracle,
            } => format!(
                "read {position} at BCI {bci} of {block:?} reads {slot:?}: the table publishes {}, \
                 the oracle reaches {}: {}",
                read(published),
                read(oracle),
                sides(published, oracle)
            ),
            Self::ReadSlots {
                block,
                published,
                oracle,
            } => format!(
                "the reads of {block:?} are {published:?} in the published table and {oracle:?} in \
                 the walk"
            ),
            Self::Unstated { value, message } => {
                format!("the published value {value:?} cannot be read: {message}")
            }
        }
    }
}

/// What one comparison examined, and where the two readings disagreed.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Comparison {
    blocks: usize,
    /// Block entry slots compared.
    entry_slots: usize,
    /// Of those, the ones whose published projection merges two or more definitions.
    merged: usize,
    /// Exit slots compared.
    exits: usize,
    /// Reads compared.
    reads: usize,
    mismatches: Vec<Mismatch>,
}

impl Comparison {
    /// Whether the published table and the oracle said the same thing everywhere they were asked.
    fn agrees(&self) -> bool {
        self.mismatches.is_empty()
    }

    /// The whole disagreement, as the report a failing test prints.
    fn report(&self) -> String {
        let mut lines = vec![format!(
            "{} disagreement(s) over {} entry slot(s) ({} of them merges), {} exit slot(s) and {} \
             read(s) in {} block(s)",
            self.mismatches.len(),
            self.entry_slots,
            self.merged,
            self.exits,
            self.reads,
            self.blocks
        )];
        for mismatch in &self.mismatches {
            lines.push(mismatch.describe());
        }
        lines.join("\n")
    }
}

/// The definitions one published value stands for, following the trivial-phi replacements.
///
/// The projection is the **least fixpoint of the operand graph**, walked from the value outwards:
/// a phi stands for the union of what its operands stand for, an instruction, the method's own
/// entry state and a caught reference stand for themselves, and an operand that names the value it
/// sits in adds nothing, because that input's copy of the slot is the value the merge point already
/// names. Iterating to the least fixpoint instead of recursing is what makes the two readings
/// comparable: the oracle computes the least fixpoint of the reaching-definition equations, so the
/// projection computes the least fixpoint of the published names, and neither invents the other's
/// cycles. Following `replaced_by` first is what cancels the trivial phis: whether a merge kept a
/// phi or moved its name to the one operand it had, the set is the same — which is exactly the
/// licence the removal rests on.
fn project(
    table: &SsaTable,
    value: ValueId,
    flaws: &Flaws,
    mismatches: &mut Vec<Mismatch>,
) -> BTreeSet<Def> {
    let mut leaves = BTreeSet::new();
    let mut expanded: BTreeSet<ValueId> = BTreeSet::new();
    let mut pending = vec![named_value(table, value)];
    while let Some(value) = pending.pop() {
        let named = named_value(table, value);
        if !expanded.insert(named) {
            continue;
        }
        match &table.value(named).def {
            Definition::Phi { block, slot } => {
                match table
                    .phis()
                    .iter()
                    .find(|phi| &phi.block == block && phi.slot == *slot)
                {
                    Some(phi) if phi.value == named => {
                        for input in &phi.inputs {
                            if let PhiInput::Value(operand) = input {
                                pending.push(*operand);
                            }
                        }
                    }
                    Some(phi) => mismatches.push(Mismatch::Unstated {
                        value: named,
                        message: format!(
                            "the table states the phi of {block:?} {slot:?} as {:?} instead",
                            phi.value
                        ),
                    }),
                    None => mismatches.push(Mismatch::Unstated {
                        value: named,
                        message: format!("the table states no phi of {block:?} {slot:?}"),
                    }),
                }
            }
            Definition::Instruction { block, bci } => {
                match written_slot(table, named, block, *bci) {
                    // The instrument: a projection that parsed one slot less never finds this
                    // definition, which is how a comparator without teeth would read the table.
                    Some(slot) if flaws.drop_a_written_slot == Some(slot) => {}
                    Some(slot) => {
                        leaves.insert(Def::Store {
                            block: block.clone(),
                            bci: *bci,
                            slot,
                        });
                    }
                    None => mismatches.push(Mismatch::Unstated {
                        value: named,
                        message: format!(
                            "no write record at BCI {bci} of {block:?} names the value it defines"
                        ),
                    }),
                }
            }
            Definition::Entry { block, slot } => {
                leaves.insert(Def::Param {
                    block: block.clone(),
                    slot: *slot,
                });
            }
            Definition::Caught { block, bci } => {
                leaves.insert(Def::Caught {
                    block: block.clone(),
                    bci: *bci,
                });
            }
        }
    }
    leaves
}

/// The value one name stands for, following the trivial-phi replacements to their end.
fn named_value(table: &SsaTable, value: ValueId) -> ValueId {
    let mut named = value;
    while let Some(next) = table.value(named).replaced_by {
        named = next;
    }
    named
}

/// The slot one published instruction definition wrote, read out of its own write record.
fn written_slot(
    table: &SsaTable,
    value: ValueId,
    block: &CanonicalBlockId,
    bci: u32,
) -> Option<Slot> {
    let named = table.block(block)?;
    named
        .instructions
        .iter()
        .find(|instruction| instruction.bci == bci)?
        .writes
        .iter()
        .find(|(_, written)| *written == value)
        .map(|(slot, _)| *slot)
}

/// The comparison: every slot the frames state a value in, every read and every exit slot, against
/// the reach the oracle computed for the same place.
fn compare(table: &SsaTable, frames: &FrameTable, oracle: &Oracle, flaws: &Flaws) -> Comparison {
    let mut comparison = Comparison::default();
    let published: Vec<CanonicalBlockId> = table
        .blocks()
        .iter()
        .map(|block| block.block.clone())
        .collect();
    let held: Vec<CanonicalBlockId> = frames
        .blocks()
        .iter()
        .map(|block| block.block.clone())
        .collect();
    if published != held {
        comparison.mismatches.push(Mismatch::Blocks {
            published,
            frames: held.clone(),
        });
    }
    let walked: Vec<CanonicalBlockId> =
        oracle.blocks.iter().map(|block| block.id.clone()).collect();
    if walked != held {
        comparison.mismatches.push(Mismatch::Walked {
            oracle: walked,
            frames: held,
        });
        return comparison;
    }
    comparison.blocks = frames.blocks().len();
    for (position, frame) in frames.blocks().iter().enumerate() {
        let state = &oracle.states[position];
        let stated = stated_slots(frame);
        let Some(named) = table.block(&frame.block) else {
            comparison.mismatches.push(Mismatch::Unnamed {
                block: frame.block.clone(),
            });
            continue;
        };
        let entries: Vec<Slot> = named.entry.iter().map(|(slot, _)| *slot).collect();
        if entries != stated {
            comparison.mismatches.push(Mismatch::EntrySlots {
                block: frame.block.clone(),
                published: entries,
                frames: stated,
            });
        }
        for (slot, value) in &named.entry {
            comparison.entry_slots += 1;
            let published_set = project(table, *value, flaws, &mut comparison.mismatches);
            if published_set.len() > 1 {
                comparison.merged += 1;
            }
            let oracle_set = state.entry.get(slot).cloned().unwrap_or_default();
            if published_set != oracle_set {
                comparison.mismatches.push(Mismatch::Entry {
                    block: frame.block.clone(),
                    slot: *slot,
                    published: published_set,
                    oracle: oracle_set,
                });
            }
        }
        let published_reads: Vec<(u32, Slot, ValueId)> = named
            .instructions
            .iter()
            .flat_map(|instruction| {
                instruction
                    .reads
                    .iter()
                    .map(|(slot, value)| (instruction.bci, *slot, *value))
            })
            .collect();
        let walked_reads: Vec<(u32, Slot)> = state
            .reads
            .iter()
            .map(|read| (read.bci, read.slot))
            .collect();
        if published_reads
            .iter()
            .map(|(bci, slot, _)| (*bci, *slot))
            .collect::<Vec<_>>()
            != walked_reads
        {
            comparison.mismatches.push(Mismatch::ReadSlots {
                block: frame.block.clone(),
                published: published_reads
                    .iter()
                    .map(|(bci, slot, _)| (*bci, *slot))
                    .collect(),
                oracle: walked_reads,
            });
        } else {
            for (position, ((bci, slot, value), read)) in
                published_reads.iter().zip(state.reads.iter()).enumerate()
            {
                comparison.reads += 1;
                let published_set = project(table, *value, flaws, &mut comparison.mismatches);
                if published_set != read.sources {
                    comparison.mismatches.push(Mismatch::Read {
                        block: frame.block.clone(),
                        bci: *bci,
                        slot: *slot,
                        position,
                        published: published_set,
                        oracle: read.sources.clone(),
                    });
                }
            }
        }
        for (slot, value) in &named.exit {
            comparison.exits += 1;
            let published_set = project(table, *value, flaws, &mut comparison.mismatches);
            let oracle_set = state.exit.get(slot).cloned().unwrap_or_default();
            if published_set != oracle_set {
                comparison.mismatches.push(Mismatch::Exit {
                    block: frame.block.clone(),
                    slot: *slot,
                    published: published_set,
                    oracle: oracle_set,
                });
            }
        }
    }
    comparison
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call_context::{CallContextOutcome, call_contexts};
    use crate::canonical::{CanonicalEdgeKind, CanonicalOutcome, canonical_cfg};
    use crate::cfg::raw_cfg;
    use crate::frame::{FrameOutcome, frames};
    use crate::ssa::{SsaOutcome, ssa};
    use jarde_reader::budget::Limits;
    use jarde_reader::classfile::{
        CpEntryFacts, class_facts, method_code_facts, test_class::single_method,
    };
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant, SnapshotId,
    };
    use jarde_reader::view::LoaderId;

    /// Access flags of the fixture methods: `ACC_PUBLIC | ACC_STATIC`, like the reader's builder.
    const ACC_STATIC: u16 = 0x0009;
    /// Access flags of a constructor fixture: `ACC_PUBLIC`, and not static.
    const ACC_PUBLIC: u16 = 0x0001;
    /// How many random graphs the second group walks.
    const RANDOM_GRAPHS: usize = 64;

    fn limits() -> Limits {
        Limits {
            input_bytes: 1 << 20,
            archive_entries: 1_000,
            entry_bytes: 1 << 20,
            read_bytes: 1 << 20,
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            output_bytes: 1 << 20,
            class_headers: 10,
            method_bodies: 10,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            nested_depth: 8,
            dependency_depth: 4,
            elapsed_millis: u64::MAX,
        }
    }

    fn budget() -> Budget {
        Budget::new(limits())
    }

    fn method_id() -> PhysicalMethodId {
        PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: SnapshotId("test".to_string()),
                },
                class_bytes: ClassBytesId {
                    digest: Digest("test".to_string()),
                    length: 0,
                },
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"method".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        }
    }

    /// One fixture body: the decoded facts, the class file's own pool, the canonical graph, the
    /// frames 4.1 published for it, and the method view the two readings share.
    struct Body {
        facts: MethodCodeFacts,
        pool: Vec<CpEntryFacts>,
        canonical: CanonicalCfg,
        frames: Box<FrameTable>,
        name: &'static [u8],
        access: u16,
    }

    /// Every layer up to the frames, over one class file.
    fn body_of(bytes: &[u8], name: &'static [u8], access: u16) -> Body {
        let mut budget = budget();
        let header = class_facts(bytes, &mut budget).expect("the fixture is a class file");
        let member = header.methods.first().expect("one method");
        let facts = method_code_facts(bytes, member, &mut budget).expect("the fixture decodes");
        let pool = header.constant_pool.clone();
        let raw = raw_cfg(&facts, &mut budget).expect("the fixture has a raw graph");
        let contexts = match call_contexts(&facts, &raw, 52, &mut budget).expect("the walk runs") {
            CallContextOutcome::Established(contexts) => contexts,
            other => panic!("the fixture must establish contexts, got {other:?}"),
        };
        let canonical = match canonical_cfg(&facts, &raw, &contexts, &method_id(), &mut budget)
            .expect("the budget is ample")
        {
            CanonicalOutcome::Canonical(graph) => *graph,
            CanonicalOutcome::Fallback { message } => {
                panic!("the fixture must normalize: {message}")
            }
        };
        let loader = LoaderId("app".to_string());
        let method = FrameMethod {
            access_flags: access,
            name,
            descriptor: b"()V",
            owner: b"Test",
            super_class: Some(b"java/lang/Object"),
            pool: &pool,
            loader: &loader,
        };
        let frames = match frames(&facts, &canonical, &method, &mut budget)
            .expect("a legal run is answered")
        {
            FrameOutcome::Frames(table) => table,
            other => panic!("the fixture must have frames, got {other:?}"),
        };
        Body {
            facts,
            pool,
            canonical,
            frames,
            name,
            access,
        }
    }

    /// One body built from real bytes by the reader's own class builder.
    fn body_of_code(code: &[u8], max_locals: u16) -> Body {
        body_of(
            &single_method(52, 8, max_locals, code),
            b"method",
            ACC_STATIC,
        )
    }

    /// The block one fixture body's BCI starts, as the canonical graph names it.
    fn block(bci: u32) -> CanonicalBlockId {
        CanonicalBlockId {
            bci,
            path: Vec::new(),
        }
    }

    /// One instruction definition, which is how the oracle names what a write left behind.
    fn store(written_in: u32, bci: u32, slot: Slot) -> Def {
        Def::Store {
            block: block(written_in),
            bci,
            slot,
        }
    }

    /// Both halves over one body, with the pass's own refusal kept as a value instead of a panic.
    ///
    /// A refusal is not a comparison failure: it is the pass declining to publish names for a body.
    /// The groups that must not see one unwrap it through [`audit`]; the random group counts them,
    /// because a refusal on a body this crate's own reader accepted is a finding about the pass and
    /// has to be reported rather than compared away.
    fn may_refuse(
        body: &Body,
        flaws: &Flaws,
    ) -> std::result::Result<(SsaTable, Oracle, Comparison), String> {
        let loader = LoaderId("app".to_string());
        let method = FrameMethod {
            access_flags: body.access,
            name: body.name,
            descriptor: b"()V",
            owner: b"Test",
            super_class: Some(b"java/lang/Object"),
            pool: &body.pool,
            loader: &loader,
        };
        let table = match ssa(
            &body.facts,
            &body.canonical,
            &body.frames,
            &method,
            &mut budget(),
        )
        .expect("the budget is ample")
        {
            SsaOutcome::Ssa(table) => *table,
            SsaOutcome::Inconsistent { message } => return Err(message),
        };
        let oracle = reach_the_fixpoint(
            &body.facts,
            &body.canonical,
            &body.frames,
            &method,
            flaws,
            &mut budget(),
        );
        let comparison = compare(&table, &body.frames, &oracle, flaws);
        Ok((table, oracle, comparison))
    }

    /// Both halves over one body: the published table, the oracle's own fixed point, and what the
    /// comparison found.
    fn audit(body: &Body, flaws: &Flaws) -> (SsaTable, Oracle, Comparison) {
        may_refuse(body, flaws)
            .unwrap_or_else(|message| panic!("the pass reported a contradiction: {message}"))
    }

    /// Runs both halves and insists they agree, which is the positive reading of every fixture.
    fn agrees(body: &Body, flaws: &Flaws) -> (SsaTable, Oracle, Comparison) {
        let (table, oracle, comparison) = audit(body, flaws);
        assert!(comparison.agrees(), "{}", comparison.report());
        (table, oracle, comparison)
    }

    /// Runs both halves and insists the weakened reading is caught.
    fn disagrees(body: &Body, flaws: &Flaws) -> Comparison {
        let (_, _, comparison) = audit(body, flaws);
        assert!(
            !comparison.agrees(),
            "the weakened reading still agreed: {}",
            comparison.report()
        );
        comparison
    }

    /// The position of one block in the oracle's own walk.
    fn position(body: &Body, id: &CanonicalBlockId) -> usize {
        body.frames
            .blocks()
            .iter()
            .position(|frame| &frame.block == id)
            .unwrap_or_else(|| panic!("the frames hold {id:?}"))
    }

    /// The definitions the oracle reaches at one point of one block.
    fn reached(
        oracle: &Oracle,
        body: &Body,
        id: &CanonicalBlockId,
        what: &str,
        slot: Slot,
    ) -> BTreeSet<Def> {
        let state = &oracle.states[position(body, id)];
        let reach = match what {
            "entry" => &state.entry,
            "exit" => &state.exit,
            other => panic!("{other} is not a point of the oracle's walk"),
        };
        reach.get(&slot).cloned().unwrap_or_default()
    }

    /// The operands of the phi one block and slot has, or a panic naming the phis that do exist.
    fn operands(table: &SsaTable, id: &CanonicalBlockId, slot: Slot) -> Vec<PhiInput> {
        table
            .phis()
            .iter()
            .find(|phi| &phi.block == id && phi.slot == slot)
            .unwrap_or_else(|| panic!("{id:?} {slot:?} has a phi: {:#?}", table.phis()))
            .inputs
            .clone()
    }

    /// The definition of each operand of one phi, in operand order: a self-referencing operand is
    /// named by the phi's own identity, which is what it means.
    fn operand_definitions(table: &SsaTable, id: &CanonicalBlockId, slot: Slot) -> Vec<Definition> {
        operands(table, id, slot)
            .iter()
            .map(|input| match input {
                PhiInput::Value(value) => table.value(*value).def.clone(),
                PhiInput::Itself => Definition::Phi {
                    block: id.clone(),
                    slot,
                },
            })
            .collect()
    }

    /// A tiny assembler for the fixture bodies: labels are patched into the branch offsets when the
    /// code array is finished, so a fixture states its shape and never its arithmetic.
    #[derive(Default)]
    struct Asm {
        code: Vec<u8>,
        labels: BTreeMap<&'static str, u32>,
        patches: Vec<(&'static str, u32)>,
    }

    impl Asm {
        fn here(&self) -> u32 {
            u32::try_from(self.code.len()).expect("a fixture is short")
        }

        fn mark(&mut self, label: &'static str) -> &mut Self {
            self.labels.insert(label, self.here());
            self
        }

        fn op(&mut self, bytes: &[u8]) -> &mut Self {
            self.code.extend_from_slice(bytes);
            self
        }

        fn branch(&mut self, opcode: u8, label: &'static str) -> &mut Self {
            self.patches.push((label, self.here()));
            self.code.push(opcode);
            self.code.extend_from_slice(&[0, 0]);
            self
        }

        fn goto(&mut self, label: &'static str) -> &mut Self {
            self.branch(0xa7, label)
        }

        fn ifeq(&mut self, label: &'static str) -> &mut Self {
            self.branch(0x99, label)
        }

        fn done(mut self) -> Vec<u8> {
            for (label, at) in self.patches {
                let target = *self
                    .labels
                    .get(label)
                    .unwrap_or_else(|| panic!("the fixture never marks {label}"));
                let offset = i32::try_from(target).expect("a fixture is short")
                    - i32::try_from(at).expect("a fixture is short");
                let offset = i16::try_from(offset).expect("a fixture's branch fits its offset");
                self.code[usize::try_from(at).expect("a fixture is short") + 1] =
                    offset.to_be_bytes()[0];
                self.code[usize::try_from(at).expect("a fixture is short") + 2] =
                    offset.to_be_bytes()[1];
            }
            self.code
        }
    }

    // ---- Group one: the ordinary edges of a CFG, over real bytes ----

    /// `iconst_1; istore_0; iload_0; istore_1; return`.
    fn straight_line() -> Body {
        body_of_code(&[0x04, 0x3b, 0x1a, 0x3c, 0xb1], 2)
    }

    /// Two paths that write the same local and meet: the join needs one phi per merged slot.
    fn diamond() -> Body {
        let mut asm = Asm::default();
        asm.op(&[0x03]); // 0: iconst_0
        asm.ifeq("other"); // 1: ifeq other
        asm.op(&[0x04, 0x3b]); // 4: iconst_1, 5: istore_0
        asm.goto("join"); // 6
        asm.mark("other"); // 9
        asm.op(&[0x05, 0x3b]); // 9: iconst_2, 10: istore_0
        asm.mark("join"); // 11
        asm.op(&[0x1a, 0x3c, 0xb1]); // 11: iload_0, 12: istore_1, 13: return
        body_of_code(&asm.done(), 2)
    }

    /// A loop with a back edge into its own header, whose carried local is written in the body.
    fn loop_body() -> Body {
        let mut asm = Asm::default();
        asm.op(&[0x03, 0x3b]); // 0: iconst_0, 1: istore_0
        asm.mark("head"); // 2
        asm.op(&[0x1a, 0x06]); // 2: iload_0, 3: iconst_3
        asm.branch(0xa1, "body"); // 4: if_icmplt body
        asm.op(&[0xb1]); // 7: return
        asm.mark("body"); // 8
        asm.op(&[0x84, 0x00, 0x01]); // 8: iinc 0, 1
        asm.goto("head"); // 11
        body_of_code(&asm.done(), 1)
    }

    /// Two entries into one cycle: `C -> D -> C`, entered from the same block at both nodes, so
    /// neither node dominates the other.
    fn irreducible() -> Body {
        let mut asm = Asm::default();
        asm.op(&[0x03]); // 0: iconst_0
        asm.ifeq("c"); // 1: ifeq c
        asm.op(&[0x08, 0x3b]); // 4: iconst_5, 5: istore_0
        asm.goto("d"); // 6
        asm.mark("c"); // 9
        asm.op(&[0x04, 0x3b]); // 9: iconst_1, 10: istore_0
        asm.goto("d"); // 11
        asm.mark("d"); // 14
        asm.op(&[0x05, 0x3b, 0x03]); // 14: iconst_2, 15: istore_0, 16: iconst_0
        asm.ifeq("end"); // 17: ifeq end
        asm.goto("c"); // 20: goto c (the back edge into the cycle)
        asm.mark("end"); // 23
        asm.op(&[0x1a, 0x3c, 0xb1]); // 23: iload_0, 24: istore_1, 25: return
        body_of_code(&asm.done(), 2)
    }

    /// A merge of a category-2 value: one long on the stack, held in two slots and named once.
    fn category_two() -> Body {
        let mut asm = Asm::default();
        asm.op(&[0x03]); // 0: iconst_0
        asm.ifeq("two"); // 1: ifeq two
        asm.op(&[0x09]); // 4: lconst_0
        asm.goto("join"); // 5
        asm.mark("two"); // 8
        asm.op(&[0x0a]); // 8: lconst_1
        asm.mark("join"); // 9
        asm.op(&[0x3f, 0x1e, 0x40, 0xb1]); // 9: lstore_0, 10: lload_0, 11: lstore_1, 12: return
        body_of_code(&asm.done(), 3)
    }

    /// One join with four predecessors, each leaving its own definition in the same local.
    fn fan_out() -> Body {
        let mut asm = Asm::default();
        asm.op(&[0x03]); // 0: iconst_0
        asm.ifeq("one"); // 1
        asm.op(&[0x03]); // 4
        asm.ifeq("two"); // 5
        asm.op(&[0x03]); // 8
        asm.ifeq("three"); // 9
        asm.op(&[0x04, 0x3b]); // 12: iconst_1, 13: istore_0
        asm.goto("join"); // 14
        asm.mark("three"); // 17
        asm.op(&[0x05, 0x3b]); // 17: iconst_2, 18: istore_0
        asm.goto("join"); // 19
        asm.mark("two"); // 22
        asm.op(&[0x06, 0x3b]); // 22: iconst_3, 23: istore_0
        asm.goto("join"); // 24
        asm.mark("one"); // 27
        asm.op(&[0x07, 0x3b]); // 27: iconst_4, 28: istore_0
        asm.goto("join"); // 29
        asm.mark("join"); // 32
        asm.op(&[0x1a, 0x3c, 0xb1]); // 32: iload_0, 33: istore_1, 34: return
        body_of_code(&asm.done(), 2)
    }

    /// A fixed-seed generator, written here because the oracle may not take a dependency: the
    /// random group has to be reproducible from the source alone.
    struct Rng(u64);

    impl Rng {
        fn new(seed: u64) -> Self {
            Self(seed)
        }

        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut value = self.0;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            value ^ (value >> 31)
        }

        fn below(&mut self, bound: u32) -> u32 {
            u32::try_from(self.next() % u64::from(bound)).expect("one remainder of a u64 fits u32")
        }
    }

    /// One random body of real bytes: blocks of a random count, each reading and writing the
    /// caller's three locals and ending in a conditional branch, a `goto`, or the method's return.
    ///
    /// The first block hands every local a value, and no block ever takes one away, so every read
    /// below has a definition on every path into it: the group tests the reaching definitions, not
    /// the frames' refusal of a read that no path defines.
    fn random_code(rng: &mut Rng) -> Vec<u8> {
        const LABELS: [&str; 7] = ["g0", "g1", "g2", "g3", "g4", "g5", "g6"];
        let count = 3 + rng.below(5); // 3..7
        let mut asm = Asm::default();
        for index in 0..count {
            asm.mark(LABELS[usize::try_from(index).expect("a fixture is short")]);
            if index == 0 {
                asm.op(&[0x04, 0x3b]); // iconst_1, istore_0
                asm.op(&[0x05, 0x3c]); // iconst_2, istore_1
                asm.op(&[0x06, 0x3d]); // iconst_3, istore_2
            }
            for _ in 0..=rng.below(2) {
                let read = u8::try_from(rng.below(3)).expect("a local index fits u8");
                let written = u8::try_from(rng.below(3)).expect("a local index fits u8");
                let value = u8::try_from(rng.below(5)).expect("a small constant fits u8");
                asm.op(&[0x1a + read]); // iload_<read>
                asm.op(&[0x04 + value]); // iconst_<value + 1>
                asm.op(&[0x60]); // iadd
                asm.op(&[0x3b + written]); // istore_<written>
            }
            if index + 1 == count {
                asm.op(&[0xb1]); // return
            } else if rng.below(4) == 0 {
                let target = LABELS[usize::try_from(rng.below(count)).expect("a fixture is short")];
                asm.goto(target);
            } else {
                let target = LABELS[usize::try_from(rng.below(count)).expect("a fixture is short")];
                asm.op(&[0x04]); // iconst_1: the condition the branch takes
                asm.ifeq(target);
            }
        }
        asm.done()
    }

    /// Whether one body's canonical graph holds an edge into a block its own list puts earlier.
    fn has_back_edge(body: &Body) -> bool {
        let at =
            |id: &CanonicalBlockId| body.canonical.blocks.iter().position(|held| &held.id == id);
        body.canonical.edges.iter().any(|edge| {
            matches!(
                (at(&edge.from), at(&edge.to)),
                (Some(from), Some(to)) if from > to
            )
        })
    }

    // ---- Group three: the JVM's exception inputs, over real bytes ----

    /// One class file whose pool holds `Test`, `java/lang/Object`, `method`, `()V`, `Code`,
    /// `<init>` and the `Object.<init>()V` reference, with `code` in the named method and
    /// `handlers` as its exception table.
    fn class_file(
        code: &[u8],
        max_locals: u16,
        max_stack: u16,
        handlers: &[(u16, u16, u16, u16)],
        method_name: u16,
        method_access: u16,
    ) -> Vec<u8> {
        fn u16_be(bytes: &mut Vec<u8>, value: u16) {
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        fn utf8(bytes: &mut Vec<u8>, value: &[u8]) {
            bytes.push(1);
            u16_be(
                bytes,
                u16::try_from(value.len()).expect("a fixture name is short"),
            );
            bytes.extend_from_slice(value);
        }
        fn class_entry(bytes: &mut Vec<u8>, name: u16) {
            bytes.push(7);
            u16_be(bytes, name);
        }
        fn name_and_type(bytes: &mut Vec<u8>, name: u16, descriptor: u16) {
            bytes.push(12);
            u16_be(bytes, name);
            u16_be(bytes, descriptor);
        }
        fn method_ref(bytes: &mut Vec<u8>, class: u16, entry: u16) {
            bytes.push(10);
            u16_be(bytes, class);
            u16_be(bytes, entry);
        }
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0xca, 0xfe, 0xba, 0xbe]);
        u16_be(&mut bytes, 0); // minor version
        u16_be(&mut bytes, 52); // major version: Java 8
        u16_be(&mut bytes, 11); // constant_pool_count: ten entries
        utf8(&mut bytes, b"Test"); // 1
        class_entry(&mut bytes, 1); // 2
        utf8(&mut bytes, b"java/lang/Object"); // 3
        class_entry(&mut bytes, 3); // 4
        utf8(&mut bytes, b"method"); // 5
        utf8(&mut bytes, b"()V"); // 6
        utf8(&mut bytes, b"Code"); // 7
        utf8(&mut bytes, b"<init>"); // 8
        name_and_type(&mut bytes, 8, 6); // 9: <init>()V
        method_ref(&mut bytes, 4, 9); // 10: java/lang/Object.<init>()V
        u16_be(&mut bytes, 0x0021); // access flags: public, super
        u16_be(&mut bytes, 2); // this class
        u16_be(&mut bytes, 4); // super class
        u16_be(&mut bytes, 0); // interfaces
        u16_be(&mut bytes, 0); // fields
        u16_be(&mut bytes, 1); // methods
        u16_be(&mut bytes, method_access);
        u16_be(&mut bytes, method_name);
        u16_be(&mut bytes, 6); // descriptor: ()V
        u16_be(&mut bytes, 1); // one attribute
        u16_be(&mut bytes, 7); // "Code"
        let mut content = Vec::new();
        u16_be(&mut content, max_stack);
        u16_be(&mut content, max_locals);
        content.extend_from_slice(
            &u32::try_from(code.len())
                .expect("a fixture is short")
                .to_be_bytes(),
        );
        content.extend_from_slice(code);
        u16_be(
            &mut content,
            u16::try_from(handlers.len()).expect("a fixture has few records"),
        );
        for (start, end, handler, catch_type) in handlers {
            u16_be(&mut content, *start);
            u16_be(&mut content, *end);
            u16_be(&mut content, *handler);
            u16_be(&mut content, *catch_type);
        }
        u16_be(&mut content, 0); // attributes of `Code`
        bytes.extend_from_slice(
            &u32::try_from(content.len())
                .expect("a fixture is short")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(&content);
        u16_be(&mut bytes, 0); // attributes of the class
        bytes
    }

    /// One static `method()` fixture with an exception table.
    fn static_body(
        code: &[u8],
        max_locals: u16,
        max_stack: u16,
        handlers: &[(u16, u16, u16, u16)],
    ) -> Body {
        body_of(
            &class_file(code, max_locals, max_stack, handlers, 5, ACC_STATIC),
            b"method",
            ACC_STATIC,
        )
    }

    /// One `<init>()V` fixture with an exception table, whose local 0 is the uninitialized `this`.
    fn constructor_body(
        code: &[u8],
        max_locals: u16,
        max_stack: u16,
        handlers: &[(u16, u16, u16, u16)],
    ) -> Body {
        body_of(
            &class_file(code, max_locals, max_stack, handlers, 8, ACC_PUBLIC),
            b"<init>",
            ACC_PUBLIC,
        )
    }

    /// One protected block holding **two** throw sites that one record catches into one handler:
    /// local 1 holds a different value at each site and the handler reads it.
    fn two_sites_one_handler() -> Body {
        let code = [
            0x04, // 0: iconst_1
            0x3c, // 1: istore_1
            0x04, // 2: iconst_1
            0x03, // 3: iconst_0
            0x6c, // 4: idiv        (site A)
            0x05, // 5: iconst_2
            0x3c, // 6: istore_1
            0x04, // 7: iconst_1
            0x03, // 8: iconst_0
            0x6c, // 9: idiv        (site B)
            0x57, // 10: pop
            0x57, // 11: pop
            0xb1, // 12: return
            0x1b, // 13: iload_1     (the handler)
            0x57, // 14: pop
            0x57, // 15: pop
            0xb1, // 16: return
        ];
        static_body(&code, 3, 2, &[(0, 13, 13, 0)])
    }

    /// One protected block holding exactly one throw site: the handler's local 0 arrives along a
    /// single input, so nothing merges there.
    fn one_throw_site() -> Body {
        let code = [
            0x04, // 0: iconst_1
            0x3b, // 1: istore_0
            0x04, // 2: iconst_1
            0x03, // 3: iconst_0
            0x6c, // 4: idiv        (the only site)
            0x57, // 5: pop
            0xb1, // 6: return
            0x4c, // 7: astore_1    (the handler)
            0x1a, // 8: iload_0
            0x57, // 9: pop
            0xb1, // 10: return
        ];
        static_body(&code, 3, 2, &[(0, 7, 7, 0)])
    }

    /// One throw site that two exception-table records catch, at two handler entries.
    fn two_handler_records() -> Body {
        let code = [
            0x04, // 0: iconst_1
            0x3b, // 1: istore_0
            0x04, // 2: iconst_1
            0x03, // 3: iconst_0
            0x6c, // 4: idiv        (the only site)
            0x57, // 5: pop
            0xb1, // 6: return
            0x4c, // 7: astore_1    (record 0's handler)
            0xb1, // 8: return
            0x4c, // 9: astore_1    (record 1's handler)
            0xb1, // 10: return
        ];
        static_body(&code, 3, 2, &[(0, 7, 7, 2), (0, 7, 9, 0)])
    }

    /// A handler that holds a second throw site of its own record, so the same handler is entered
    /// from the block it is in: the merge it takes part in refers to its own entry.
    fn handler_throws_again() -> Body {
        let code = [
            0x04, // 0: iconst_1
            0x3b, // 1: istore_0
            0x04, // 2: iconst_1
            0x03, // 3: iconst_0
            0x6c, // 4: idiv        (site A)
            0x57, // 5: pop
            0xb1, // 6: return
            0x4c, // 7: astore_1    (the handler)
            0x04, // 8: iconst_1
            0x03, // 9: iconst_0
            0x6c, // 10: idiv       (site B, inside the handler)
            0x57, // 11: pop
            0xb1, // 12: return
        ];
        static_body(&code, 3, 2, &[(0, 13, 7, 0)])
    }

    /// A constructor whose own constructor call converts `this`: a throw site after the call hands
    /// the handler the converted local, while the block writes it again afterwards, so the site's
    /// state and the block's exit state are two different definitions of the same slot.
    fn constructor_transition() -> Body {
        let mut asm = Asm::default();
        asm.op(&[0x2a]); // 0: aload_0
        asm.op(&[0xb7, 0x00, 0x0a]); // 1: invokespecial #10 (Object.<init>()V)
        asm.op(&[0x04]); // 4: iconst_1
        asm.ifeq("skip"); // 5: ifeq skip
        asm.mark("site"); // 8
        asm.op(&[0x04, 0x03, 0x6c]); // 8: iconst_1, 9: iconst_0, 10: idiv (the site)
        asm.op(&[0x57]); // 11: pop
        asm.op(&[0x01]); // 12: aconst_null
        asm.op(&[0x4b]); // 13: astore_0
        asm.op(&[0xb1]); // 14: return
        asm.mark("handler"); // 15
        asm.op(&[0x4c]); // 15: astore_1 (the handler)
        asm.op(&[0x2a]); // 16: aload_0
        asm.op(&[0x57]); // 17: pop
        asm.op(&[0xb1]); // 18: return
        asm.mark("skip"); // 19
        asm.op(&[0xb1]); // 19: return
        let code = asm.done();
        // The record covers the site's own block, and neither the constructor call nor the handler:
        // a record over the call itself would enter the same handler with the *token*, and 4.1
        // merges a token and an initialized reference to `Top` — the pass's honest answer, but not
        // the state this fixture is about. The record's two ends are both block starts, which is
        // what the canonical graph maps a protected range onto.
        assert_eq!(&code[8..11], &[0x04, 0x03, 0x6c]);
        constructor_body(&code, 2, 2, &[(8, 15, 15, 0)])
    }

    // ---- The groups, each declared and asserted on its own ----

    #[test]
    fn the_oracle_states_the_reach_of_a_straight_line_by_hand() {
        let body = straight_line();
        let (_, oracle, comparison) = agrees(&body, &Flaws::none());
        assert!(comparison.agrees(), "{}", comparison.report());
        let state = &oracle.states[0];
        assert!(
            state.entry.is_empty(),
            "a static `()V` method starts with no value in any slot: {:#?}",
            state.entry
        );
        let reads: Vec<(u32, Slot, BTreeSet<Def>)> = state
            .reads
            .iter()
            .map(|read| (read.bci, read.slot, read.sources.clone()))
            .collect();
        assert_eq!(
            reads,
            vec![
                (
                    1,
                    Slot::Stack(0),
                    BTreeSet::from([store(0, 0, Slot::Stack(0))])
                ),
                (
                    2,
                    Slot::Local(0),
                    BTreeSet::from([store(0, 1, Slot::Local(0))])
                ),
                (
                    3,
                    Slot::Stack(0),
                    BTreeSet::from([store(0, 2, Slot::Stack(0))])
                ),
            ],
            "`istore_0` reads the push, `iload_0` reads the store, `istore_1` reads the load"
        );
        assert_eq!(
            state.exit.get(&Slot::Local(0)).cloned().unwrap_or_default(),
            BTreeSet::from([store(0, 1, Slot::Local(0))])
        );
        assert_eq!(
            state.exit.get(&Slot::Local(1)).cloned().unwrap_or_default(),
            BTreeSet::from([store(0, 3, Slot::Local(1))])
        );
        assert!(
            !state.exit.contains_key(&Slot::Stack(0)),
            "`return` ends the block on an empty stack: {:#?}",
            state.exit
        );
    }

    #[test]
    fn group_one_the_ordinary_edges_agree() {
        let fixtures = [
            ("straight line", straight_line()),
            ("diamond", diamond()),
            ("loop", loop_body()),
            ("irreducible", irreducible()),
            ("category-2", category_two()),
            ("high fan-out", fan_out()),
        ];
        let mut entry_slots = 0usize;
        let mut merged = 0usize;
        let mut reads = 0usize;
        let mut tables = Vec::new();
        for (what, body) in &fixtures {
            let (table, _, comparison) = agrees(body, &Flaws::none());
            assert!(
                comparison.blocks > 0 && comparison.reads > 0,
                "{what} was compared but nothing was at stake: {}",
                comparison.report()
            );
            entry_slots += comparison.entry_slots;
            merged += comparison.merged;
            reads += comparison.reads;
            tables.push(table);
        }
        println!(
            "group one: {} fixtures, {} entry slots ({merged} merges), {} reads, {} blocks",
            fixtures.len(),
            entry_slots,
            reads,
            tables
                .iter()
                .map(|table| table.blocks().len())
                .sum::<usize>()
        );
        assert_eq!(fixtures.len(), 6);
        assert!(
            entry_slots >= 6 && merged >= 4,
            "the six shapes must merge: {entry_slots} entry slots, {merged} merges"
        );
        // The shape behind the agreement: one operand per logical input of the slot, and a merge of
        // a category-2 value is one phi for the value and none for its upper half.
        assert_eq!(operands(&tables[1], &block(11), Slot::Local(0)).len(), 2);
        assert_eq!(operands(&tables[2], &block(2), Slot::Local(0)).len(), 2);
        // The cycle of the irreducible fixture is entered at both of its nodes, and the node both
        // entries reach merges the two definitions those entries left in the local — while the
        // other node's entry merges a definition with a `Top`, which is no value and no phi.
        assert!(has_back_edge(&fixtures[3].1));
        assert_eq!(operands(&tables[3], &block(14), Slot::Local(0)).len(), 2);
        assert_eq!(
            operand_definitions(&tables[3], &block(14), Slot::Local(0)),
            vec![
                Definition::Instruction {
                    block: block(4),
                    bci: 5
                },
                Definition::Instruction {
                    block: block(9),
                    bci: 10
                },
            ]
        );
        assert_eq!(operands(&tables[4], &block(9), Slot::Stack(0)).len(), 2);
        assert_eq!(operands(&tables[5], &block(32), Slot::Local(0)).len(), 4);
    }

    #[test]
    fn group_two_the_random_graphs_agree() {
        let mut rng = Rng::new(0x5eed_2026_0403);
        let mut graphs = 0usize;
        let mut entry_slots = 0usize;
        let mut merged = 0usize;
        let mut exits = 0usize;
        let mut reads = 0usize;
        let mut loopy = 0usize;
        let mut refused: Vec<(Vec<u8>, String)> = Vec::new();
        for _ in 0..RANDOM_GRAPHS {
            let code = random_code(&mut rng);
            let body = body_of_code(&code, 3);
            let comparison = match may_refuse(&body, &Flaws::none()) {
                Ok((_, _, comparison)) => comparison,
                Err(message) => {
                    refused.push((code, message));
                    continue;
                }
            };
            graphs += 1;
            entry_slots += comparison.entry_slots;
            merged += comparison.merged;
            exits += comparison.exits;
            reads += comparison.reads;
            if has_back_edge(&body) {
                loopy += 1;
            }
        }
        println!(
            "group two: {graphs} random graphs compared, {entry_slots} entry slots ({merged} \
             merges), {exits} exit slots, {reads} reads, {loopy} with a back edge, {} refused",
            refused.len()
        );
        for (code, message) in &refused {
            println!("group two: refused {code:02x?}: {message}");
        }
        // A refusal is a **finding**, not a comparison result: every shape the generator builds is
        // a body this crate's own reader decoded and the frames accepted, so the pass declining to
        // publish names for one of them is a defect of the pass and nothing here may compare it
        // away. Three of the 64 used to be refused — `ir_ssa_inconsistent` reading "a phi ... is
        // named as a use of one value without holding it as an operand" — for a reason that was in
        // the pass and not in those bytes: a value's use records are a multiset of **occurrences**,
        // and the replacement asked every record to rewrite one, which the first record of a place
        // holding two had already done. Zero is the count now, and this is where a return to
        // anything else shows; the bytes of any refusal are printed above.
        assert!(
            refused.is_empty(),
            "{} of {RANDOM_GRAPHS} random graphs were refused, and every one of them is a body \
             this crate's reader decoded and 4.1 framed: {refused:#?}",
            refused.len()
        );
        assert_eq!(graphs, RANDOM_GRAPHS, "every graph ran");
        assert!(
            entry_slots >= graphs,
            "the graphs must merge, not just walk: {entry_slots} entry slots over {graphs} graphs"
        );
        assert!(
            merged >= graphs / 2,
            "at least half the graphs must hold a merge of two or more definitions: {merged}"
        );
        assert!(
            loopy >= 8,
            "the generator must produce loops for the back-edge rule to matter: {loopy}"
        );
    }

    /// The shape the random group's refusals all had, reduced by hand to the smallest body that
    /// holds it — one value handed to **one phi as two operands**.
    ///
    /// The first case is ten bytes, `0 iconst_1, 1 iconst_0, 2 ifeq +7 (to 9), 5 iconst_0,
    /// 6 ifeq -5 (back to 1), 9 return`: four blocks, the entry (0), a header (1) reached from the
    /// entry and from the block at 5, that block, and the join (9) that the header and the block at
    /// 5 both enter. The header takes its stack slot from the entry and, through the back edge,
    /// from its own value, so its phi has one usable operand and the replacement pass names the
    /// slot after that operand. The join holds the replaced phi's value as **two** of its operands
    /// — one per participant — so that value carries two use records of the phi-operand kind. The
    /// second case is the same shape with a **local** instead of the stack slot the entry leaves
    /// (eleven bytes, the local's value stored at BCI 1 and named after it), so that one merged
    /// slot of each region carries the finding.
    ///
    /// The replacement used to ask every use record to rewrite at least one occurrence, and the
    /// first record's pass already rewrote both of the join's operands, so the second record found
    /// nothing and the run reported `ir_ssa_inconsistent` reading "a phi ... is named as a use of
    /// one value without holding it as an operand" — about the pass's own bookkeeping, on bodies
    /// this crate's reader decoded and 4.1 framed. The records are a multiset of **occurrences**,
    /// so the check is now between the occurrences rewritten and the records held, and both bodies
    /// complete.
    ///
    /// Every body the random group refused used to be one of these shapes: the group's own report
    /// named the same message on 3 of its 64 graphs, and this is the case it pinned.
    #[test]
    fn one_value_held_as_two_phi_operands_is_not_a_contradiction() {
        // 0 iconst_1, 1 iconst_0, 2 ifeq +7 (to 9), 5 iconst_0, 6 ifeq -5 (back to 1), 9 return.
        let stack_case = (
            &[0x04, 0x03, 0x99, 0x00, 0x07, 0x03, 0x99, 0xff, 0xfb, 0xb1][..],
            1u16,
            Slot::Stack(0),
            block(9),
            Definition::Instruction {
                block: block(0),
                bci: 0,
            },
        );
        // 0 iconst_1, 1 istore_0, 2 iconst_0, 3 ifeq +7 (to 10), 6 iconst_0, 7 ifeq -5 (back to
        // 2), 10 return.
        let local_case = (
            &[
                0x04, 0x3b, 0x03, 0x99, 0x00, 0x07, 0x03, 0x99, 0xff, 0xfb, 0xb1,
            ][..],
            1u16,
            Slot::Local(0),
            block(10),
            Definition::Instruction {
                block: block(0),
                bci: 1,
            },
        );
        for (code, locals, slot, join_block, handed) in [stack_case, local_case] {
            let body = body_of_code(code, locals);
            let (table, _, comparison) = agrees(&body, &Flaws::none());
            assert!(
                comparison.entry_slots > 0 && comparison.reads > 0,
                "the body must be compared, not just walked: {}",
                comparison.report()
            );
            assert_eq!(
                table.blocks().len(),
                4,
                "the entry, the header, the block the header enters and the join: {:#?}",
                table.blocks()
            );
            let join = operands(&table, &join_block, slot);
            assert_eq!(
                join.len(),
                2,
                "the join takes one operand per participant: {:#?}",
                table.phis()
            );
            assert_eq!(
                join[0],
                join[1],
                "both participants hand the join the same value, which is what made the records \
                 two: {:#?}",
                table.phis()
            );
            let PhiInput::Value(operand) = join[0] else {
                panic!("the join's operands name values: {join:?}");
            };
            assert_eq!(
                table.value(operand).def,
                handed,
                "the header's phi is named after the value the entry hands it, so both operands \
                 stand for that value: {:#?}",
                table.phis()
            );
        }
    }

    #[test]
    fn group_three_the_exception_inputs_agree() {
        let fixtures = [
            ("one throw site", one_throw_site()),
            (
                "two sites, one record, one handler",
                two_sites_one_handler(),
            ),
            ("two records, two handlers", two_handler_records()),
            ("a handler that throws again", handler_throws_again()),
            (
                "a constructor call that converts local 0",
                constructor_transition(),
            ),
        ];
        let mut entry_slots = 0usize;
        let mut reads = 0usize;
        for (what, body) in &fixtures {
            let (_, _, comparison) = agrees(body, &Flaws::none());
            assert!(
                comparison.entry_slots > 0 && comparison.reads > 0,
                "{what} was compared but nothing was at stake: {}",
                comparison.report()
            );
            entry_slots += comparison.entry_slots;
            reads += comparison.reads;
        }
        println!(
            "group three: {} exception fixtures, {} entry slots, {} reads",
            fixtures.len(),
            entry_slots,
            reads
        );
        assert_eq!(fixtures.len(), 5);
    }

    #[test]
    fn group_three_two_sites_under_one_raw_edge_reach_the_handler_as_two_definitions() {
        let body = two_sites_one_handler();
        let source = block(0);
        let handler = block(13);
        assert_eq!(
            body.canonical
                .edges
                .iter()
                .filter(
                    |edge| matches!(edge.kind, CanonicalEdgeKind::Exception { .. })
                        && edge.from == source
                        && edge.to == handler
                )
                .count(),
            1,
            "the canonical graph aggregates both throw sites into one exception edge"
        );
        assert_eq!(
            body.canonical
                .throw_sites
                .iter()
                .filter(|site| site.block == source)
                .count(),
            2,
            "the canonical graph keeps the two sites"
        );
        let state = body.frames.entry(&handler).expect("the handler is reached");
        assert_eq!(state.inputs.len(), 2, "one logical input per throw site");
        assert_eq!(state.inputs[0].throw_site, Some(4));
        assert_eq!(state.inputs[1].throw_site, Some(9));
        let (table, oracle, comparison) = agrees(&body, &Flaws::none());
        assert_eq!(
            reached(&oracle, &body, &handler, "entry", Slot::Local(1)),
            BTreeSet::from([store(0, 1, Slot::Local(1)), store(0, 6, Slot::Local(1))]),
            "the handler's local is reached by the value each site left in it"
        );
        assert_eq!(operands(&table, &handler, Slot::Local(1)).len(), 2);
        assert_eq!(
            reached(&oracle, &body, &handler, "entry", Slot::Stack(0)),
            BTreeSet::from([
                Def::Caught {
                    block: source.clone(),
                    bci: 4
                },
                Def::Caught {
                    block: source.clone(),
                    bci: 9
                }
            ]),
            "the caught reference is one definition per site"
        );
        assert!(comparison.agrees(), "{}", comparison.report());
    }

    #[test]
    fn group_three_each_handler_record_brings_its_own_catch_type() {
        let body = two_handler_records();
        let first = body.frames.entry(&block(7)).expect("record 0's handler");
        let second = body.frames.entry(&block(9)).expect("record 1's handler");
        assert_ne!(
            first.stack.first(),
            second.stack.first(),
            "the two records enter their handlers with their own reference"
        );
        let (_, oracle, comparison) = agrees(&body, &Flaws::none());
        for handler in [block(7), block(9)] {
            assert_eq!(
                reached(&oracle, &body, &handler, "entry", Slot::Stack(0)),
                BTreeSet::from([Def::Caught {
                    block: block(0),
                    bci: 4
                }]),
                "both handlers are entered by the same site's reference"
            );
        }
        assert!(comparison.agrees(), "{}", comparison.report());
    }

    #[test]
    fn group_three_a_handler_that_throws_again_merges_with_itself() {
        let body = handler_throws_again();
        let handler = block(7);
        let (table, oracle, comparison) = agrees(&body, &Flaws::none());
        assert_eq!(
            reached(&oracle, &body, &handler, "entry", Slot::Local(0)),
            BTreeSet::from([store(0, 1, Slot::Local(0))]),
            "the local arrives from the first site, and the handler's own input adds nothing"
        );
        assert_eq!(
            reached(&oracle, &body, &handler, "entry", Slot::Stack(0)),
            BTreeSet::from([
                Def::Caught {
                    block: block(0),
                    bci: 4
                },
                Def::Caught {
                    block: handler.clone(),
                    bci: 10
                }
            ]),
            "the handler is entered by its own throw site as well"
        );
        // The self-referential input makes the local's phi trivial: its name moves to the
        // instruction that wrote the local, and the operands stay one per throw site.
        let phi = table
            .phis()
            .iter()
            .find(|phi| phi.block == handler && phi.slot == Slot::Local(0))
            .expect("the handler's local 0 merges two inputs");
        let named = table
            .block(&handler)
            .expect("the handler is named")
            .entry
            .iter()
            .find(|(slot, _)| *slot == Slot::Local(0))
            .expect("the handler's local 0 has an entry definition")
            .1;
        assert_eq!(phi.inputs.len(), 2, "one operand per throw site");
        assert_eq!(
            operand_definitions(&table, &handler, Slot::Local(0)),
            vec![
                Definition::Instruction {
                    block: block(0),
                    bci: 1
                },
                Definition::Phi {
                    block: handler.clone(),
                    slot: Slot::Local(0)
                },
            ]
        );
        assert_ne!(
            named, phi.value,
            "the removed phi's name is not the value the slot is called by"
        );
        assert_eq!(
            table.value(phi.value).replaced_by,
            Some(named),
            "the phi states the value that took its name"
        );
        assert!(comparison.agrees(), "{}", comparison.report());
    }

    #[test]
    fn group_three_a_constructor_call_hands_the_handler_the_converted_local() {
        let body = constructor_transition();
        let handler = block(15);
        let (_, oracle, comparison) = agrees(&body, &Flaws::none());
        assert_eq!(
            reached(&oracle, &body, &handler, "entry", Slot::Local(0)),
            BTreeSet::from([store(0, 1, Slot::Local(0))]),
            "the handler's local 0 is the value the constructor call converted it to, not the \
             uninitialized `this` and not the value the site's block writes afterwards"
        );
        assert_eq!(
            reached(&oracle, &body, &handler, "entry", Slot::Stack(0)),
            BTreeSet::from([Def::Caught {
                block: block(8),
                bci: 10
            }]),
            "the handler is entered by the site's own reference"
        );
        // And the site's block writes local 0 again after the site, so the block's own exit is a
        // different definition of the same slot: reading the site's state is a real reading and not
        // the block's end.
        assert_eq!(
            reached(&oracle, &body, &block(8), "exit", Slot::Local(0)),
            BTreeSet::from([store(8, 13, Slot::Local(0))]),
            "the site's block writes local 0 again after the site"
        );
        assert!(comparison.agrees(), "{}", comparison.report());
    }

    // ---- The falsification instruments: a weakened reading must turn red ----

    #[test]
    fn flaw_one_reading_the_aggregated_edge_turns_the_two_site_case_red() {
        let body = two_sites_one_handler();
        agrees(&body, &Flaws::none());
        let comparison = disagrees(&body, &Flaws::aggregated());
        assert!(
            comparison.mismatches.iter().any(|mismatch| matches!(
                mismatch,
                Mismatch::Entry {
                    slot: Slot::Local(1),
                    ..
                }
            )),
            "the core case must fail at the handler's merged local: {}",
            comparison.report()
        );
        assert!(
            comparison
                .report()
                .contains("missing on the published side"),
            "the report names what the weakened reading lost: {}",
            comparison.report()
        );
    }

    #[test]
    fn flaw_one_also_turns_the_constructor_transition_red() {
        let body = constructor_transition();
        agrees(&body, &Flaws::none());
        let comparison = disagrees(&body, &Flaws::aggregated());
        assert!(
            comparison
                .report()
                .contains("the entry of CanonicalBlockId { bci: 15, path: [] } Local(0)"),
            "the handler's own local is where the aggregated reading goes wrong: {}",
            comparison.report()
        );
    }

    #[test]
    fn flaw_two_walking_forward_edges_only_turns_the_loop_red() {
        let body = loop_body();
        agrees(&body, &Flaws::none());
        let comparison = disagrees(&body, &Flaws::forward_only());
        assert!(
            comparison.mismatches.iter().any(|mismatch| matches!(
                mismatch,
                Mismatch::Entry {
                    slot: Slot::Local(0),
                    ..
                }
            )),
            "the loop's carried local loses the back edge's definition: {}",
            comparison.report()
        );
        assert!(
            comparison
                .report()
                .contains("instruction CanonicalBlockId { bci: 8, path: [] }@8"),
            "the definition the back edge carries is the one that goes missing: {}",
            comparison.report()
        );
    }

    #[test]
    fn flaw_three_a_projection_that_parses_one_slot_less_is_caught() {
        let body = straight_line();
        agrees(&body, &Flaws::none());
        let comparison = disagrees(&body, &Flaws::dropping(Slot::Local(1)));
        assert!(
            comparison.mismatches.iter().any(|mismatch| matches!(
                mismatch,
                Mismatch::Exit {
                    slot: Slot::Local(1),
                    ..
                }
            )),
            "the exit of the block is where the missing definition shows: {}",
            comparison.report()
        );
        assert!(
            comparison
                .report()
                .contains("instruction CanonicalBlockId { bci: 0, path: [] }@3 into Local(1)"),
            "the report names exactly which definition the published side lost: {}",
            comparison.report()
        );
    }

    #[test]
    fn the_oracle_never_names_a_helper_of_the_naming_half() {
        let source = include_str!("ssa_oracle.rs");
        for token in [
            concat!("flow", "_facts"),
            concat!("Assign", "er"),
            concat!("Block", "Flow"),
            concat!("Site", "Flow"),
            concat!("Instruction", "Flow"),
            concat!("Flow", "Facts"),
            concat!("Flow", "Input"),
            concat!("Input", "Kind"),
            concat!("simpl", "ify"),
            concat!("record", "_use"),
            concat!("resolve", "("),
            concat!("target", "("),
            concat!("Slot", "Resolution"),
        ] {
            assert!(
                !source.contains(token),
                "the oracle names the production helper {token:?}"
            );
        }
        let oracle_half = source.split("#[cfg(test)]").next().unwrap_or_default();
        assert!(
            !oracle_half.contains("ssa("),
            "the oracle and the comparison are handed the published table; the tests are what run \
             the pass"
        );
    }
}
