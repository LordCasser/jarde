//! 4.3: the local and operand-stack SSA of one decoded body, over the frames of 4.1/4.2.
//!
//! The pass has two halves, kept apart on purpose.
//!
//! **The semantic half** ([`flow_facts`]) reads the published frames, the canonical graph and the
//! shared operand facts, and turns one block into the slot accesses of each of its instructions
//! plus the logical inputs its entry state is merged from ([`BlockFlow`]). It classifies nothing a
//! second time: it replays the block through 4.1's own dense table, which is the one place the
//! shapes of `pop2`, `dup2_x2` and their family — shapes that follow the *values* on the stack —
//! are decided, and it takes the classes and the exception inputs from the frame pass as they are.
//! A category-2 value is one access at its first slot and never two, and the alias conversion of a
//! constructor call is the write of a new value at every slot it converts, because that is what it
//! is.
//!
//! **The naming half** ([`Assigner`]) owns slot identity, definitions, uses and the phis of the
//! logical predecessors, and nothing else. It walks an explicit worklist — never the call stack —
//! and it resolves a block's entry state as follows:
//!
//! - every slot the entry state starts a value in gets one entry definition, and the participants
//!   of that definition are decidable before any of them has been resolved: a seed input, a plain
//!   transfer or an exception transfer always hands the slot its state at *its* position, and an
//!   exception transfer hands the operand stack's slot 0 the caught reference. The frame pass's
//!   own merge is why this is decidable: `merge_local` answers `Top` as soon as one contributor
//!   has no value, so a slot with a class in the entry state has a value on **every** contributing
//!   path;
//! - a slot with exactly one participant *is* that participant's value — a passthrough, no phi;
//! - a slot with two or more participants gets a phi, and the phi is created **before** its
//!   operands are collected. That is what makes a cycle through one of the block's own inputs
//!   resolvable: the block runs on the placeholder and the operand is completed when its source
//!   publishes its state, which for a back edge is the block's own exit;
//! - an operand whose source has not reached its position yet stays **pending**: the relation is
//!   kept (the requesting block is woken when that source publishes) and *nothing* is cached for
//!   it. "Not processed yet" and "this path has no value" are two different answers, and the
//!   second — a contributor that states no value for a slot its own merge gave one — is a
//!   contradiction of this build's artifacts, not a value a consumer may hold;
//! - a `Top` slot never starts a value: no definition, no phi, no pseudo-value. Neither does the
//!   upper half of a category-2 pair, whose two slots are one value throughout.
//!
//! The phi's class is the class 4.1 computed for that slot of that block, which *is* the merge of
//! its operands by 4.1's own rule. It is not re-folded here on purpose: 4.1's reference merge is
//! not associative (a `null`, a named reference and another named reference fold to different
//! classes in different orders), so a second fold could disagree with the state the frames
//! published while both are faithful — one merge rule, applied once, in the frame pass.
//!
//! **Trivial phis** (one usable operand, distinct from the phi itself) are replaced by that
//! operand in this same stage: every use of the phi and every definition record that named it move
//! to the operand, so the slot's name becomes the operand's. The phi's operand list is **not**
//! rewritten with it: it keeps one operand per logical predecessor, `Itself` operands included,
//! because that list is the record of what the merge took part in and the use records were written
//! per operand. Folding it to the one usable operand would break both halves of the contract the
//! published table owes — the phi would state a merge point with fewer predecessors than the slot
//! was actually merged over, and the target's use records would count operand occurrences that the
//! published operands no longer hold — while trimming the use records to match the folded operand
//! would falsify the same fact from the other side and lose "this slot merged N paths". An
//! `Itself` operand of a replaced phi still means what it means everywhere else, the value the
//! merge point already names, which `replaced_by` now names. A phi that only refers to itself is
//! left alone: it is a value no path defines, and replacing it would invent one. The replacement
//! touches neither the control nor the effect facts, and it introduces no optimization framework.
//!
//! **A self-reference has one spelling.** An operand that names the value its own phi defines is
//! `Itself`, never `Value(own)`, and it carries no use record — a self-reference is not a use of a
//! value. Collection writes it that way already, and a replacement that hands one phi the value
//! another phi's operand list was naming keeps it that way: the occurrence becomes `Itself` and
//! its record is dropped instead of moving to the target. That is what keeps "the occurrences
//! rewritten are the records held" an exact count — the one check [`Assigner::replace`] makes —
//! rather than a count with an exception for the phi a replacement left holding its own value.
//!
//! **Dropping those records is a by-block decision, not a positional one.** Which record answers
//! for which occurrence is not a fact either order of the table states: an entry record holds the
//! block whose phi carries the occurrence and nothing else, and the orders the two sides are
//! *written* in differ. [`Assigner::complete`] pushes one entry record per participant while the
//! block that holds the phi resolves them — so a value's entry records are in the order its
//! consumer blocks were completed, with a block a missing source woke later pushing again at the
//! end — while [`Assigner::replace`] rewrites the phi operands in [`Assigner::phis`] table order,
//! which is the order the phis were created in. Nothing ties those two traversals together, so
//! "the first N entry records" can take the records of blocks whose occurrences were not the
//! rewritten ones while the totals still add up, and both blocks would then publish a def-use edge
//! the other holds. The drop is therefore counted per block, and a block whose occurrences ask for
//! more records than it holds is a refusal rather than a silent misattribution.
//!
//! **The replaced phi's origins are not merged into the target.** The target is still defined
//! exactly once — by an instruction, by the entry state, by a caught reference or by a phi of its
//! own — and its `OriginSet` states where that one definition came from; the replaced value stays
//! in `values` with the origin of the merge point it stood for, and `replaced_by` is the one place
//! the two are tied together. Folding the merge point's origins into the definition would claim
//! the one BCI that defines the target also defines every BCI the merge stood for.
//!
//! **Origins and effects.** A value an instruction defines carries one `MethodPoint` at that
//! instruction's BCI; a phi carries its block's own origin set, which is where a normalized clone
//! keeps every original BCI it stands for; a caught reference carries its throw site's origin; and
//! an entry value carries none, because no instruction produces it. The `Effects` fact of this pass
//! is per **instruction-level throw site**: every instruction carries its own handlers, so an
//! exception effect belongs to the BCI that raises and never to the end of a block. Its shape is
//! [`CanonicalEffectFacts`], the canonical counterpart of 3.3's `EffectFacts`; the raw facts are
//! neither replaced nor rewritten by it, and no second IR is introduced.
//!
//! **Charging.** `IrItems`: one per stored entry/exit slot record, per SSA value, per origin
//! member, per phi operand, per use record, per replayed access and per effect record. `IrEdges`:
//! one per def-use edge, phi operands included. `AnalysisSteps`: one per worklist iteration and one
//! per instruction the replay examines. A budget stop or a cancellation is the ordinary `Error` the
//! rest of the crate reports, and a stopped run publishes neither `Ssa` nor `Effects`: the driver
//! keeps the phases before it and reports the stop under its own code.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::MethodCodeFacts;
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{OriginMember, OriginSet, PhysicalMethodId};

use crate::canonical::{CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind};
use crate::frame::{
    BlockFrame, FrameMethod, FrameTable, InstructionTouches, SlotRegion, SlotTouch, TouchesOutcome,
    Value, block_touches, caught_reference, entry_slots, starts_value,
};

/// Code of an SSA run whose input contradicts itself.
pub(crate) const IR_SSA_INCONSISTENT: &str = "ir_ssa_inconsistent";

/// What one run of the `ssa` pass produced.
#[derive(Debug)]
pub(crate) enum SsaOutcome {
    /// The names of every reached block: one definition per value, one use per read, the phis of
    /// the merge points, and the effect facts of the same instructions.
    Ssa(Box<SsaTable>),
    /// The frames, the graph and the instructions of this body disagree about a slot: a value is
    /// read from a slot no definition reaches, an entry state no input of it provides, or a
    /// definition that depends on itself around a cycle no path into it can break. The message
    /// names the slot and the instruction, and the run publishes neither `Ssa` nor `Effects`.
    Inconsistent { message: String },
}

/// One slot a value flows through.
///
/// Locals are named by their index, the operand stack by the depth of the value's **first** slot: a
/// category-2 value is one slot identity and never two, so its two frame entries are one SSA value.
/// The stack is aligned at block boundaries by that depth, which the frame pass has already proven
/// to agree between every pair of blocks that meet.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum Slot {
    Local(u16),
    Stack(u32),
}

impl Slot {
    /// The slot one recorded access names.
    fn of(region: SlotRegion, slot: u32) -> Self {
        match region {
            SlotRegion::Local => Self::Local(u16::try_from(slot).unwrap_or(u16::MAX)),
            SlotRegion::Stack => Self::Stack(slot),
        }
    }

    /// The slot above this one, within its own region.
    fn upper(self) -> Option<Self> {
        match self {
            Self::Local(index) => index.checked_add(1).map(Self::Local),
            Self::Stack(depth) => depth.checked_add(1).map(Self::Stack),
        }
    }

    /// The slot below this one, within its own region.
    fn lower(self) -> Option<Self> {
        match self {
            Self::Local(0) | Self::Stack(0) => None,
            Self::Local(index) => Some(Self::Local(index - 1)),
            Self::Stack(depth) => Some(Self::Stack(depth - 1)),
        }
    }
}

/// Identity of one SSA value: an index into the table's own list.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct ValueId(u32);

impl ValueId {
    fn index(self) -> usize {
        usize::try_from(self.0).unwrap_or(usize::MAX)
    }
}

/// Where one SSA value comes into existence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Definition {
    /// The method's own entry state: a parameter, `this`, or a value the caller hands the method.
    /// No instruction produces it, so it carries no origin.
    Entry { block: CanonicalBlockId, slot: Slot },
    /// The value one instruction leaves in one slot.
    Instruction { block: CanonicalBlockId, bci: u32 },
    /// The value the entry phi of one block and slot names.
    Phi { block: CanonicalBlockId, slot: Slot },
    /// The exception reference one throw site hands the handler it feeds.
    Caught { block: CanonicalBlockId, bci: u32 },
}

/// One use of one value.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct SsaUse {
    /// The block the use sits in.
    pub(crate) block: CanonicalBlockId,
    /// BCI of the using instruction, or `None` for the operand of a phi, which no instruction
    /// produces.
    pub(crate) bci: Option<u32>,
}

/// One SSA value: its class, its definition, its origin and its uses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SsaValue {
    /// The class 4.1 established for the slot, or the one it merged for a phi.
    pub(crate) ty: Value,
    pub(crate) def: Definition,
    pub(crate) origin: OriginSet,
    pub(crate) uses: Vec<SsaUse>,
    /// The value that replaced this one, for a phi this stage removed as trivial. A replaced phi
    /// is no longer a definition: the target is, and every use of the phi was moved to it. Its own
    /// operands and origin stay as they were — they record the merge point it stood for, and
    /// merging them into the target would state that the target's one definition is theirs.
    pub(crate) replaced_by: Option<ValueId>,
}

/// One entry phi: the definition one block's entry state gives one slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SsaPhi {
    pub(crate) block: CanonicalBlockId,
    pub(crate) slot: Slot,
    /// The value this phi defines.
    pub(crate) value: ValueId,
    /// One operand per participating logical predecessor, in the block's own input order — for a
    /// phi that was removed as trivial as well: its operands are the merge it stood for, and the
    /// value that replaced it is named by its own [`SsaValue::replaced_by`].
    pub(crate) inputs: Vec<PhiInput>,
}

/// One operand of an entry phi.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PhiInput {
    /// The value the slot arrives with along this input.
    Value(ValueId),
    /// The phi itself: this input's copy of the slot is the value the merge point already names,
    /// which is what a back edge into the same block hands over. For a phi that was replaced as
    /// trivial, that value is the one [`SsaValue::replaced_by`] names.
    Itself,
}

/// What one instruction of one block reads, writes and may raise.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SsaInstruction {
    pub(crate) bci: u32,
    pub(crate) opcode: u8,
    /// One record per read, in the order the instruction performed them.
    pub(crate) reads: Vec<(Slot, ValueId)>,
    /// One record per write, in the order the instruction performed them; a category-2 write is
    /// one record at the value's first slot.
    pub(crate) writes: Vec<(Slot, ValueId)>,
}

/// The SSA of one block: its entry definitions, its instructions and its exit definitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SsaBlock {
    pub(crate) block: CanonicalBlockId,
    /// One record per slot the entry state starts a value in, ascending.
    pub(crate) entry: Vec<(Slot, ValueId)>,
    /// One record per slot the block's exit state starts a value in, ascending. It is what a plain
    /// successor reads, and the frame pass has already proven it agrees with the slots that
    /// successor is entered with.
    pub(crate) exit: Vec<(Slot, ValueId)>,
    pub(crate) instructions: Vec<SsaInstruction>,
}

/// The effect facts of one canonical instruction.
///
/// The shape mirrors 3.3's raw [`crate::cfg::InstructionEffect`] — the same question about what an
/// instruction does to its frame — over the canonical graph, with the one difference a value-flow
/// consumer needs: the handlers of an exception effect belong to the **instruction** at `bci` and
/// never to the end of its block, and the block is named beside it. The raw facts are neither
/// replaced nor rewritten by this one; each describes its own graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CanonicalInstructionEffect {
    pub(crate) block: CanonicalBlockId,
    pub(crate) bci: u32,
    pub(crate) opcode: u8,
    /// Locals this instruction reads, ascending, deduplicated.
    pub(crate) locals_read: Vec<u16>,
    /// Locals this instruction writes, ascending, deduplicated.
    pub(crate) locals_written: Vec<u16>,
    /// Change of the operand-stack depth, in slots, exactly as the frame pass measured it on this
    /// instruction: a discard that reads nothing — `athrow` clearing the stack, a `return` ending
    /// the block with it — is included, so the delta is not inferred from the reads and writes.
    pub(crate) stack_delta: i32,
    /// Whether this instruction may raise.
    pub(crate) may_throw: bool,
    /// The handler records this throw site feeds, in declaration order, at this BCI.
    pub(crate) handlers: Vec<u32>,
    pub(crate) origin: OriginSet,
}

/// The `Effects` fact of the canonical graph: one record per instruction, in block order and then
/// in execution order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CanonicalEffectFacts {
    pub(crate) instructions: Vec<CanonicalInstructionEffect>,
}

/// The published artifact: every value, every phi and every block's names.
///
/// The table is derived storage whose consumer in this build is the next slice, so it stays
/// crate-private exactly like the frames and the graph it is derived from.
#[derive(Debug)]
pub(crate) struct SsaTable {
    values: Vec<SsaValue>,
    phis: Vec<SsaPhi>,
    blocks: Vec<SsaBlock>,
    effects: CanonicalEffectFacts,
}

#[allow(
    dead_code,
    reason = "the next slice consumes this payload; 5.1 decides what becomes public"
)]
impl SsaTable {
    /// Every value, by identity.
    pub(crate) fn values(&self) -> &[SsaValue] {
        &self.values
    }

    /// One value.
    pub(crate) fn value(&self, id: ValueId) -> &SsaValue {
        &self.values[id.index()]
    }

    /// Every entry phi, in creation order.
    pub(crate) fn phis(&self) -> &[SsaPhi] {
        &self.phis
    }

    /// Every block the frames hold, in the canonical graph's order.
    pub(crate) fn blocks(&self) -> &[SsaBlock] {
        &self.blocks
    }

    /// The effect facts of the same instructions.
    pub(crate) fn effects(&self) -> &CanonicalEffectFacts {
        &self.effects
    }

    /// One block's names, or `None` when the entry cannot reach it.
    pub(crate) fn block(&self, block: &CanonicalBlockId) -> Option<&SsaBlock> {
        self.blocks.iter().find(|entry| &entry.block == block)
    }
}

/// Builds the SSA of one canonical graph, over the frames 4.1 published for it.
///
/// See the module documentation for the protocol, the invariants and the charges. The stop
/// conditions are the ordinary ones of this crate: a budget stop or a cancellation is an `Err`, and
/// a contradiction of the artifacts is the [`SsaOutcome::Inconsistent`] value that carries the
/// message the driver reports under [`IR_SSA_INCONSISTENT`].
pub(crate) fn ssa(
    facts: &MethodCodeFacts,
    canonical: &CanonicalCfg,
    frames: &FrameTable,
    method: &FrameMethod<'_>,
    budget: &mut Budget,
) -> Result<SsaOutcome> {
    match run(facts, canonical, frames, method, budget) {
        Ok(table) => Ok(SsaOutcome::Ssa(Box::new(table))),
        Err(Problem::Budget(error)) => Err(error),
        Err(Problem::Inconsistent(message)) => Ok(SsaOutcome::Inconsistent { message }),
    }
}

/// The two ways one run can end before it publishes: a stop of the budget layer, and a
/// contradiction of the artifacts this pass reads.
#[derive(Debug)]
enum Problem {
    Budget(Error),
    Inconsistent(String),
}

type Norm<T> = std::result::Result<T, Problem>;

/// The budget layer's own stop, kept apart from a contradiction of the artifacts: the first is the
/// run's stop condition, the second is a result about the input.
impl From<Error> for Problem {
    fn from(error: Error) -> Self {
        Self::Budget(error)
    }
}

fn inconsistent<T>(message: String) -> Norm<T> {
    Err(Problem::Inconsistent(message))
}

/// The fixpoint itself, as [`ssa`] documents it.
fn run(
    facts: &MethodCodeFacts,
    canonical: &CanonicalCfg,
    frames: &FrameTable,
    method: &FrameMethod<'_>,
    budget: &mut Budget,
) -> Norm<SsaTable> {
    budget.poll()?;
    let Some(anchor) = graph_method(canonical) else {
        return inconsistent(
            "the canonical graph states no method point, so no SSA value can be anchored to one"
                .to_string(),
        );
    };
    let seed = match entry_slots(method, facts) {
        Ok(Some(slots)) => slots
            .into_iter()
            .map(|slot| (Slot::of(slot.region, slot.slot), slot.value))
            .collect::<BTreeMap<Slot, Value>>(),
        Ok(None) => {
            return inconsistent(
                "the frames of this body hold no state for its own entry block".to_string(),
            );
        }
        Err(error) => return Err(Problem::Budget(error)),
    };
    let flows = flow_facts(facts, canonical, frames, &seed, method, budget)?;
    let mut assigner = Assigner::new(flows, anchor, seed, budget)?;
    assigner.assign(canonical, method, budget)?;
    assigner.simplify(budget)?;
    assigner.publish(budget)
}

/// The method the canonical graph's own origins name.
///
/// The graph belongs to exactly one method and states it in every origin member, which is what lets
/// this pass anchor `MethodPoint` origins without a parameter it would otherwise not use.
fn graph_method(canonical: &CanonicalCfg) -> Option<PhysicalMethodId> {
    canonical.blocks.iter().find_map(|block| {
        block.origin.members.iter().find_map(|member| match member {
            OriginMember::MethodPoint { method, .. } => Some(method.clone()),
            _ => None,
        })
    })
}

/// One logical input of a block, as the name assignment sees it.
#[derive(Clone, Debug, Eq, PartialEq)]
struct FlowInput {
    kind: InputKind,
    /// The block the state comes from; `None` for the method's own entry state.
    from: Option<CanonicalBlockId>,
    /// BCI of the throwing instruction, for an exception input.
    throw_site: Option<u32>,
}

/// Where one logical input arrives from.
#[derive(Clone, Debug, Eq, PartialEq)]
enum InputKind {
    /// The method's own entry state: the caller's contribution, which participates in the entry
    /// block's phis exactly like an incoming edge does.
    Seed,
    /// A plain transfer: the source block's exit state.
    Transfer,
    /// An exception transfer of one throw site: the source's state **before** that instruction,
    /// with the caught reference on the operand stack.
    Exception { handler_ordinal: u32 },
}

/// The throw site one instruction is, when the canonical graph holds one for it.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SiteFlow {
    handler_ordinals: Vec<u32>,
    /// The local slots the handlers of this site read out of the source's pre-instruction state.
    /// The operand stack contributes nothing here: an exception edge carries the caught reference
    /// and not the stack the throwing instruction was about to use.
    slots: Vec<Slot>,
    origin: OriginSet,
}

/// One instruction of one block: what it touches, and the throw site it is.
#[derive(Clone, Debug, Eq, PartialEq)]
struct InstructionFlow {
    bci: u32,
    opcode: u8,
    stack_after: u32,
    accesses: Vec<SlotTouch>,
    site: Option<SiteFlow>,
}

/// The semantic half: what one block *is*, before any name is assigned.
///
/// It holds the classes the entry state starts values in, the logical inputs that state is merged
/// from and the accesses of every instruction — all of it read out of the frame pass and the shared
/// operand facts, with no value identity in sight.
#[derive(Clone, Debug)]
struct BlockFlow {
    id: CanonicalBlockId,
    /// The origin set of the canonical block: every original BCI a normalized clone stands for,
    /// which is the origin a phi of this block carries.
    origin: OriginSet,
    entry_values: BTreeMap<Slot, Value>,
    inputs: Vec<FlowInput>,
    instructions: Vec<InstructionFlow>,
}

impl BlockFlow {
    /// The participants of one entry slot, as indices into [`Self::inputs`].
    ///
    /// A seed and a plain transfer hand the slot their state at *their* position. An exception
    /// transfer hands the handler the source's state **before** the throwing instruction for every
    /// local, and the caught reference for the operand stack's slot 0 — and nothing for any deeper
    /// stack slot, because an exception edge carries one value and not the stack the instruction
    /// was about to use.
    ///
    /// That the slot has a class in the entry state at all is what makes every one of these a
    /// contributor: the frame pass merges a local to `Top` as soon as one contributor has no value
    /// for it, and it refuses two operand stacks of different depths, so a slot with a class has a
    /// value on every path that reaches the block.
    fn participants(&self, slot: Slot) -> Vec<usize> {
        self.inputs
            .iter()
            .enumerate()
            .filter(|(_, input)| match input.kind {
                InputKind::Seed | InputKind::Transfer => true,
                InputKind::Exception { .. } => {
                    matches!(slot, Slot::Local(_) | Slot::Stack(0))
                }
            })
            .map(|(index, _)| index)
            .collect()
    }
}

/// The semantic half of the pass, for every block the frames hold.
///
/// Blocks the frames do not hold are blocks the entry cannot reach — the canonical graph keeps them
/// in its own `unreachable` list instead of inventing a state for them — and they carry no names
/// either.
fn flow_facts(
    facts: &MethodCodeFacts,
    canonical: &CanonicalCfg,
    frames: &FrameTable,
    seed: &BTreeMap<Slot, Value>,
    method: &FrameMethod<'_>,
    budget: &mut Budget,
) -> Norm<Vec<BlockFlow>> {
    budget.poll()?;
    let entry_id = CanonicalBlockId {
        bci: 0,
        path: Vec::new(),
    };
    // Every block's entry classes first: a throw site names the slots its handlers read out of the
    // source, and those handlers are other blocks of the same graph.
    let mut entry_values: BTreeMap<CanonicalBlockId, BTreeMap<Slot, Value>> = BTreeMap::new();
    for block in &canonical.blocks {
        let Some(state) = frames.entry(&block.id) else {
            continue;
        };
        entry_values.insert(block.id.clone(), entry_classes(state, budget)?);
    }
    let mut flows = Vec::new();
    for block in &canonical.blocks {
        let Some(state) = frames.entry(&block.id) else {
            continue;
        };
        let mut inputs = Vec::new();
        if block.id == entry_id && !seed.is_empty() {
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            inputs.push(FlowInput {
                kind: InputKind::Seed,
                from: None,
                throw_site: None,
            });
        }
        for logical in &state.inputs {
            let kind = match logical.throw_site {
                None => InputKind::Transfer,
                Some(bci) => {
                    let Some(site) = canonical
                        .throw_sites
                        .iter()
                        .find(|site| site.block == logical.from && site.bci == bci)
                    else {
                        return inconsistent(format!(
                            "block {:?} states an exception input at BCI {bci} of block {:?}, which \
                             the graph holds no throw site for: the two are built from one fact and \
                             cannot disagree",
                            block.id, logical.from
                        ));
                    };
                    let Some(ordinal) =
                        exception_ordinal(canonical, &logical.from, &block.id, &site.handlers)
                    else {
                        return inconsistent(format!(
                            "the exception input of block {:?} from BCI {bci} of block {:?} names no \
                             exception edge of the graph",
                            block.id, logical.from
                        ));
                    };
                    InputKind::Exception {
                        handler_ordinal: ordinal,
                    }
                }
            };
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            inputs.push(FlowInput {
                kind,
                from: Some(logical.from.clone()),
                throw_site: logical.throw_site,
            });
        }
        let instructions = match block_touches(facts, block, state, method, budget) {
            Ok(TouchesOutcome::Touches(touches)) => touches,
            Ok(TouchesOutcome::Inconsistent { message }) => {
                return inconsistent(format!(
                    "the instructions of block {:?} disagree with the frames published for it: \
                     {message}",
                    block.id
                ));
            }
            Ok(TouchesOutcome::Unsupported { message }) => {
                return inconsistent(format!(
                    "the instructions of block {:?} reach a state this build does not prove: \
                     {message}",
                    block.id
                ));
            }
            Err(error) => return Err(Problem::Budget(error)),
        };
        let mut flows_here = Vec::with_capacity(instructions.len());
        for instruction in instructions {
            let site = site_flow(canonical, &entry_values, &block.id, &instruction, budget)?;
            flows_here.push(InstructionFlow {
                bci: instruction.bci,
                opcode: instruction.opcode,
                stack_after: instruction.stack_after,
                accesses: instruction.accesses,
                site,
            });
        }
        flows.push(BlockFlow {
            id: block.id.clone(),
            origin: block.origin.clone(),
            entry_values: entry_values.get(&block.id).cloned().unwrap_or_default(),
            inputs,
            instructions: flows_here,
        });
    }
    Ok(flows)
}

/// The classes one published entry state starts values in.
///
/// `Top` starts none and neither does the upper half of a category-2 pair, whose value is the one
/// below it: taking either for a value would invent a definition no instruction produced.
fn entry_classes(state: &BlockFrame, budget: &mut Budget) -> Norm<BTreeMap<Slot, Value>> {
    let mut values = BTreeMap::new();
    for (index, value) in state.locals.iter().enumerate() {
        if !starts_value(value) {
            continue;
        }
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        values.insert(
            Slot::Local(u16::try_from(index).unwrap_or(u16::MAX)),
            value.clone(),
        );
    }
    for (depth, value) in state.stack.iter().enumerate() {
        if !starts_value(value) {
            continue;
        }
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        values.insert(
            Slot::Stack(u32::try_from(depth).unwrap_or(u32::MAX)),
            value.clone(),
        );
    }
    Ok(values)
}

/// The throw site of one replayed instruction, with the local slots its handlers read.
///
/// Charges one `IrItems` per slot the record keeps, **before** the set holds it, and only for a
/// slot it does not hold yet: the entry value of a slot reached along two handlers is one slot of
/// this record, and counting it twice would charge for storage that does not exist.
fn site_flow(
    canonical: &CanonicalCfg,
    entry_values: &BTreeMap<CanonicalBlockId, BTreeMap<Slot, Value>>,
    block: &CanonicalBlockId,
    instruction: &InstructionTouches,
    budget: &mut Budget,
) -> Norm<Option<SiteFlow>> {
    let Some(site) = canonical
        .throw_sites
        .iter()
        .find(|site| &site.block == block && site.bci == instruction.bci)
    else {
        return Ok(None);
    };
    let mut slots: BTreeSet<Slot> = BTreeSet::new();
    for ordinal in &site.handlers {
        for edge in &canonical.edges {
            let CanonicalEdgeKind::Exception { handler_ordinal } = edge.kind else {
                continue;
            };
            if &edge.from != block || handler_ordinal != *ordinal {
                continue;
            }
            if let Some(values) = entry_values.get(&edge.to) {
                for slot in values.keys().filter(|slot| matches!(slot, Slot::Local(_))) {
                    if slots.contains(slot) {
                        continue;
                    }
                    budget.charge(CountedBudgetDimension::IrItems, 1)?;
                    slots.insert(*slot);
                }
            }
        }
    }
    Ok(Some(SiteFlow {
        handler_ordinals: site.handlers.clone(),
        slots: slots.into_iter().collect(),
        origin: site.origin.clone(),
    }))
}

/// The exception edge that carries one throw site's state into one handler block.
fn exception_ordinal(
    canonical: &CanonicalCfg,
    from: &CanonicalBlockId,
    to: &CanonicalBlockId,
    handlers: &[u32],
) -> Option<u32> {
    canonical.edges.iter().find_map(|edge| match edge.kind {
        CanonicalEdgeKind::Exception { handler_ordinal }
            if &edge.from == from && &edge.to == to && handlers.contains(&handler_ordinal) =>
        {
            Some(handler_ordinal)
        }
        _ => None,
    })
}

/// What one attempt to resolve one entry slot learned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Resolution {
    /// The value the slot holds along this input.
    Ready(ValueId),
    /// This input states no value for the slot while the block's own entry state states a class for
    /// it: the frames and this resolution contradict each other.
    NoValue,
    /// The source of this input has not reached the position the input is taken at yet. Nothing is
    /// decided for the slot: the relation is kept and the block is woken when that source
    /// publishes its state.
    Pending(usize),
}

/// One entry slot of one block, as far as it is resolved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlotResolution {
    /// The value the slot holds at the block's entry.
    Ready(ValueId),
    /// The index of the phi that names it; its operands may still be incomplete, and the block can
    /// already run on the placeholder.
    Phi(usize),
}

/// One worklist item: a block whose entry state is being resolved, or one that is ready to run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Task {
    Enter(usize),
    Run(usize),
}

/// One block, as the name assignment sees it.
struct BlockState {
    /// The semantic half this state assigns names to.
    flow: BlockFlow,
    /// The entry slot resolutions, once each slot has one.
    slots: BTreeMap<Slot, SlotResolution>,
    /// The phis of this block whose operands are not all collected yet, by phi index.
    open: BTreeSet<usize>,
    /// One entry per participating input of each phi still being collected; `None` where its source
    /// has not answered.
    operands: BTreeMap<usize, Vec<Option<PhiInput>>>,
    /// The definition of each slot, at the position this block has run to.
    defs: BTreeMap<Slot, (ValueId, u32)>,
    /// The exit definitions, published when the block has run to its end.
    exit: Option<BTreeMap<Slot, ValueId>>,
    /// The instructions this block has recorded, in execution order.
    instructions: Vec<SsaInstruction>,
    entered: bool,
    done: bool,
    /// Whether an `Enter` task of this block is already queued. The two task kinds are tracked
    /// apart on purpose: a block that entered and is queued to *run* still has to be re-asked for
    /// its open phi operands when a source publishes, and one flag for both would drop that wake.
    enter_queued: bool,
    run_queued: bool,
}

/// The naming half of the pass: slot identity, definitions, uses and the phis of the logical
/// predecessors, over one explicit worklist.
struct Assigner {
    anchor: PhysicalMethodId,
    blocks: Vec<BlockState>,
    index_of: BTreeMap<CanonicalBlockId, usize>,
    values: Vec<SsaValue>,
    phis: Vec<SsaPhi>,
    effects: Vec<CanonicalInstructionEffect>,
    /// The caller's own contribution to the entry block, one definition per slot.
    seed: BTreeMap<Slot, ValueId>,
    /// The caught reference of one throw site, one value per `(source, BCI, handler record)`.
    caught: BTreeMap<(usize, u32, u32), ValueId>,
    /// The state of a source **before** one throwing instruction, for the slots its handlers read.
    sites: BTreeMap<(usize, u32), BTreeMap<Slot, Option<ValueId>>>,
    /// The blocks waiting for one source to publish its state or one of its site snapshots. A block
    /// may wait for itself here: a back edge into it is completed from its own exit.
    waiters: BTreeMap<usize, BTreeSet<usize>>,
    worklist: VecDeque<Task>,
}

impl Assigner {
    fn new(
        flows: Vec<BlockFlow>,
        anchor: PhysicalMethodId,
        seed: BTreeMap<Slot, Value>,
        budget: &mut Budget,
    ) -> Norm<Self> {
        let mut index_of = BTreeMap::new();
        for (position, flow) in flows.iter().enumerate() {
            index_of.insert(flow.id.clone(), position);
        }
        let blocks = flows
            .into_iter()
            .map(|flow| BlockState {
                flow,
                slots: BTreeMap::new(),
                open: BTreeSet::new(),
                operands: BTreeMap::new(),
                defs: BTreeMap::new(),
                exit: None,
                instructions: Vec::new(),
                entered: false,
                done: false,
                enter_queued: false,
                run_queued: false,
            })
            .collect::<Vec<BlockState>>();
        let entry = CanonicalBlockId {
            bci: 0,
            path: Vec::new(),
        };
        let Some(&start) = index_of.get(&entry) else {
            return inconsistent(
                "the canonical graph holds no entry block BCI 0 that a frame of this body covers"
                    .to_string(),
            );
        };
        let mut assigner = Self {
            anchor,
            blocks,
            index_of,
            values: Vec::new(),
            phis: Vec::new(),
            effects: Vec::new(),
            seed: BTreeMap::new(),
            caught: BTreeMap::new(),
            sites: BTreeMap::new(),
            waiters: BTreeMap::new(),
            worklist: VecDeque::new(),
        };
        // The caller's own definitions: one value per slot its state starts, none for a slot it
        // leaves unreadable.
        for (slot, ty) in &seed {
            let value = assigner.new_value(
                ty.clone(),
                Definition::Entry {
                    block: entry.clone(),
                    slot: *slot,
                },
                OriginSet::default(),
                budget,
            )?;
            assigner.seed.insert(*slot, value);
        }
        // Every block the frames hold asks for its entry state, the entry block first: a block is
        // resolved from its inputs, and a block no source has published for yet registers itself as
        // a waiter of exactly those sources instead of caching anything for them. Seeding the
        // worklist with all of them is what makes the walk independent of the order the blocks are
        // stored in — the dependency, not the position, decides when a block runs.
        assigner.enqueue(Task::Enter(start));
        for position in 0..assigner.blocks.len() {
            if position != start {
                assigner.enqueue(Task::Enter(position));
            }
        }
        Ok(assigner)
    }

    /// Queues one task, once per block and kind.
    fn enqueue(&mut self, task: Task) {
        let queued = match task {
            Task::Enter(position) => &mut self.blocks[position].enter_queued,
            Task::Run(position) => &mut self.blocks[position].run_queued,
        };
        if *queued {
            return;
        }
        *queued = true;
        self.worklist.push_back(task);
    }

    /// One fresh value, with its origin, charged before it is stored.
    fn new_value(
        &mut self,
        ty: Value,
        def: Definition,
        origin: OriginSet,
        budget: &mut Budget,
    ) -> Norm<ValueId> {
        let members = u64::try_from(origin.members.len()).unwrap_or(u64::MAX);
        budget.charge(CountedBudgetDimension::IrItems, 1 + members)?;
        let id = ValueId(u32::try_from(self.values.len()).unwrap_or(u32::MAX));
        self.values.push(SsaValue {
            ty,
            def,
            origin,
            uses: Vec::new(),
            replaced_by: None,
        });
        Ok(id)
    }

    /// The origin of one instruction-level definition.
    fn point(&self, bci: u32) -> OriginSet {
        let mut origin = OriginSet::default();
        origin.insert(OriginMember::MethodPoint {
            method: self.anchor.clone(),
            bci,
        });
        origin
    }

    /// Runs the whole name assignment, until nothing is left to resolve.
    fn assign(
        &mut self,
        canonical: &CanonicalCfg,
        method: &FrameMethod<'_>,
        budget: &mut Budget,
    ) -> Norm<()> {
        while let Some(task) = self.worklist.pop_front() {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            match task {
                Task::Enter(position) => {
                    self.blocks[position].enter_queued = false;
                    self.enter(position, canonical, method, budget)?;
                }
                Task::Run(position) => {
                    self.blocks[position].run_queued = false;
                    self.run_block(position, budget)?;
                }
            }
        }
        // The worklist is drained. A block that never entered, a block that entered and never ran,
        // and a phi whose operands never all arrived are one defect: a definition that depends on
        // itself around a cycle no path into it can break. Nothing is published for such a body,
        // and the message names the block, what it waits for and which of its slots are still open.
        let mut waiting_on: BTreeMap<usize, Vec<String>> = BTreeMap::new();
        for (source, waiters) in &self.waiters {
            for waiter in waiters {
                waiting_on
                    .entry(*waiter)
                    .or_default()
                    .push(format!("{:?}", self.blocks[*source].flow.id));
            }
        }
        let stalled: Vec<String> = self
            .blocks
            .iter()
            .enumerate()
            .filter(|(_, block)| !block.entered || !block.done)
            .map(|(position, block)| {
                let open: Vec<String> = block
                    .flow
                    .entry_values
                    .keys()
                    .filter(|slot| !block.slots.contains_key(slot))
                    .map(|slot| format!("{slot:?}"))
                    .collect();
                format!(
                    "{:?} (entered {}, ran {}, open entry slots [{}], waiting on [{}])",
                    block.flow.id,
                    block.entered,
                    block.done,
                    open.join(", "),
                    waiting_on
                        .get(&position)
                        .map(|sources| sources.join(", "))
                        .unwrap_or_default()
                )
            })
            .collect();
        if !stalled.is_empty() {
            return inconsistent(format!(
                "the names of the blocks {} could not be resolved: their definitions depend on \
                 themselves and no path into the cycle states a value",
                stalled.join("; ")
            ));
        }
        let incomplete: Vec<String> =
            self.blocks
                .iter()
                .flat_map(|block| {
                    block.open.iter().map(|phi| {
                        format!("{:?} {:?}", self.phis[*phi].block, self.phis[*phi].slot)
                    })
                })
                .collect();
        if !incomplete.is_empty() {
            return inconsistent(format!(
                "the entry phis {} were left without an operand",
                incomplete.join(", ")
            ));
        }
        Ok(())
    }

    /// Resolves, as far as this attempt can, the entry state of one block, and runs it once every
    /// slot of it is decided. A later attempt — the block was woken by a source — only completes
    /// the phi operands that were still open.
    fn enter(
        &mut self,
        position: usize,
        canonical: &CanonicalCfg,
        method: &FrameMethod<'_>,
        budget: &mut Budget,
    ) -> Norm<()> {
        if self.blocks[position].entered {
            return self.complete(position, canonical, method, budget);
        }
        let slots: Vec<(Slot, Value)> = self.blocks[position]
            .flow
            .entry_values
            .iter()
            .map(|(slot, ty)| (*slot, ty.clone()))
            .collect();
        let mut waiting: BTreeSet<usize> = BTreeSet::new();
        for (slot, ty) in slots {
            if self.blocks[position].slots.contains_key(&slot) {
                continue;
            }
            let participants = self.blocks[position].flow.participants(slot);
            if participants.is_empty() {
                return inconsistent(format!(
                    "slot {slot:?} of block {:?} holds the class {ty:?} while no logical input of \
                     that block contributes a value to it",
                    self.blocks[position].flow.id
                ));
            }
            if participants.len() == 1 {
                match self.resolve(position, slot, participants[0], canonical, method, budget)? {
                    Resolution::Ready(value) => {
                        self.check_class(position, slot, value)?;
                        self.blocks[position]
                            .slots
                            .insert(slot, SlotResolution::Ready(value));
                    }
                    Resolution::NoValue => {
                        return inconsistent(format!(
                            "slot {slot:?} of block {:?} holds the class {ty:?} while its only \
                             input states no value for it",
                            self.blocks[position].flow.id
                        ));
                    }
                    Resolution::Pending(source) => {
                        waiting.insert(source);
                    }
                }
                continue;
            }
            // Two or more participants: a phi, created **before** its operands are collected, so
            // that a cycle through one of this block's own inputs has a definition to resolve
            // against.
            let block = self.blocks[position].flow.id.clone();
            let origin = self.blocks[position].flow.origin.clone();
            let value = self.new_value(
                ty,
                Definition::Phi {
                    block: block.clone(),
                    slot,
                },
                origin,
                budget,
            )?;
            budget.charge(
                CountedBudgetDimension::IrItems,
                u64::try_from(participants.len() + 1).unwrap_or(u64::MAX),
            )?;
            let phi = self.phis.len();
            self.phis.push(SsaPhi {
                block,
                slot,
                value,
                inputs: Vec::new(),
            });
            self.blocks[position]
                .slots
                .insert(slot, SlotResolution::Phi(phi));
            self.blocks[position]
                .operands
                .insert(phi, vec![None; participants.len()]);
            self.blocks[position].open.insert(phi);
        }
        if !waiting.is_empty() {
            for source in waiting {
                self.waiters.entry(source).or_default().insert(position);
            }
            return Ok(());
        }
        let defs: Vec<(Slot, ValueId, u32)> = self.blocks[position]
            .slots
            .iter()
            .map(|(slot, resolution)| {
                let (value, ty) = match resolution {
                    SlotResolution::Ready(value) => (*value, self.values[value.index()].ty.clone()),
                    SlotResolution::Phi(phi) => {
                        let phi = &self.phis[*phi];
                        (phi.value, self.values[phi.value.index()].ty.clone())
                    }
                };
                let width = u32::try_from(ty.slots()).unwrap_or(u32::MAX);
                (*slot, value, width)
            })
            .collect();
        for (slot, value, width) in defs {
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            self.blocks[position].defs.insert(slot, (value, width));
        }
        self.blocks[position].entered = true;
        self.complete(position, canonical, method, budget)?;
        self.enqueue(Task::Run(position));
        Ok(())
    }

    /// Collects the operands of the phis of one block that are still open, and registers the
    /// blocks to wake once a missing source publishes.
    fn complete(
        &mut self,
        position: usize,
        canonical: &CanonicalCfg,
        method: &FrameMethod<'_>,
        budget: &mut Budget,
    ) -> Norm<()> {
        let open: Vec<(usize, Slot, Vec<Option<PhiInput>>)> = self.blocks[position]
            .open
            .iter()
            .map(|phi| {
                (
                    *phi,
                    self.phis[*phi].slot,
                    self.blocks[position]
                        .operands
                        .get(phi)
                        .cloned()
                        .unwrap_or_default(),
                )
            })
            .collect();
        for (phi, slot, mut operands) in open {
            let participants = self.blocks[position].flow.participants(slot);
            let value = self.phis[phi].value;
            let block = self.blocks[position].flow.id.clone();
            for (index, input) in participants.into_iter().enumerate() {
                if operands.get(index).and_then(Option::as_ref).is_some() {
                    continue;
                }
                match self.resolve(position, slot, input, canonical, method, budget)? {
                    Resolution::Ready(operand) if operand == value => {
                        operands[index] = Some(PhiInput::Itself);
                    }
                    Resolution::Ready(operand) => {
                        self.record_use(operand, &block, None, budget)?;
                        operands[index] = Some(PhiInput::Value(operand));
                    }
                    Resolution::NoValue => {
                        return inconsistent(format!(
                            "slot {slot:?} of block {block:?} holds a class while one of its \
                             logical inputs states no value for it"
                        ));
                    }
                    Resolution::Pending(source) => {
                        self.waiters.entry(source).or_default().insert(position);
                    }
                }
            }
            if operands.iter().all(Option::is_some) {
                self.phis[phi].inputs = operands.iter().flatten().cloned().collect();
                self.blocks[position].open.remove(&phi);
                self.blocks[position].operands.remove(&phi);
            } else {
                self.blocks[position].operands.insert(phi, operands);
            }
        }
        Ok(())
    }

    /// One input's answer for one slot.
    fn resolve(
        &mut self,
        position: usize,
        slot: Slot,
        input: usize,
        canonical: &CanonicalCfg,
        method: &FrameMethod<'_>,
        budget: &mut Budget,
    ) -> Norm<Resolution> {
        let FlowInput {
            kind,
            from,
            throw_site,
        } = self.blocks[position].flow.inputs[input].clone();
        match kind {
            InputKind::Seed => match self.seed.get(&slot) {
                Some(value) => Ok(Resolution::Ready(*value)),
                None => Ok(Resolution::NoValue),
            },
            InputKind::Transfer => {
                let source = self.source_of(position, &from)?;
                match &self.blocks[source].exit {
                    None => Ok(Resolution::Pending(source)),
                    Some(exit) => match exit.get(&slot) {
                        Some(value) => Ok(Resolution::Ready(*value)),
                        // The frame pass merges a local to `Top` as soon as one contributor has no
                        // value for it, so a slot the destination states a class for has a value
                        // on every contributing path.
                        None => Ok(Resolution::NoValue),
                    },
                }
            }
            InputKind::Exception { handler_ordinal } => {
                let source = self.source_of(position, &from)?;
                let Some(bci) = throw_site else {
                    return inconsistent(format!(
                        "an exception input of block {:?} names no throw site",
                        self.blocks[position].flow.id
                    ));
                };
                if slot == Slot::Stack(0) {
                    let value =
                        self.caught(source, bci, handler_ordinal, canonical, method, budget)?;
                    return Ok(Resolution::Ready(value));
                }
                let Some(snapshot) = self.sites.get(&(source, bci)) else {
                    return Ok(Resolution::Pending(source));
                };
                match snapshot.get(&slot) {
                    Some(Some(value)) => Ok(Resolution::Ready(*value)),
                    Some(None) => Ok(Resolution::NoValue),
                    None => inconsistent(format!(
                        "slot {slot:?} of block {:?} is read through the throw site at BCI {bci} of \
                         block {:?}, which its own snapshot does not cover",
                        self.blocks[position].flow.id, self.blocks[source].flow.id
                    )),
                }
            }
        }
    }

    /// The block one logical input comes from, as an index of this run's own block list.
    fn source_of(&self, position: usize, from: &Option<CanonicalBlockId>) -> Norm<usize> {
        let Some(from) = from else {
            return inconsistent(format!(
                "an incoming input of block {:?} names no source block",
                self.blocks[position].flow.id
            ));
        };
        match self.index_of.get(from) {
            Some(source) => Ok(*source),
            None => inconsistent(format!(
                "an input of block {:?} comes from {from:?}, which the frames of this body do not \
                 hold",
                self.blocks[position].flow.id
            )),
        }
    }

    /// The exception reference one throw site hands one handler.
    fn caught(
        &mut self,
        source: usize,
        bci: u32,
        ordinal: u32,
        canonical: &CanonicalCfg,
        method: &FrameMethod<'_>,
        budget: &mut Budget,
    ) -> Norm<ValueId> {
        if let Some(value) = self.caught.get(&(source, bci, ordinal)) {
            return Ok(*value);
        }
        let Some(row) = canonical
            .handler_rows
            .iter()
            .find(|row| row.ordinal == ordinal)
        else {
            return inconsistent(format!(
                "the throw site at BCI {bci} of block {:?} feeds handler record {ordinal}, which no \
                 row of the graph states",
                self.blocks[source].flow.id
            ));
        };
        let ty = match caught_reference(method, row) {
            Ok(ty) => ty,
            Err(crate::frame::Problem::Budget(error)) => return Err(Problem::Budget(error)),
            Err(crate::frame::Problem::Inconsistent(message)) => {
                return inconsistent(format!(
                    "the handler reference of record {ordinal} at BCI {bci} contradicts the class \
                     file: {message}"
                ));
            }
            Err(crate::frame::Problem::Unproven(message)) => {
                return inconsistent(format!(
                    "the handler reference of record {ordinal} at BCI {bci} cannot be stated: \
                     {message}"
                ));
            }
        };
        let block = self.blocks[source].flow.id.clone();
        let origin = self.blocks[source]
            .flow
            .instructions
            .iter()
            .find(|instruction| instruction.bci == bci)
            .and_then(|instruction| instruction.site.as_ref())
            .map(|site| site.origin.clone())
            .unwrap_or_else(|| self.point(bci));
        let value = self.new_value(ty, Definition::Caught { block, bci }, origin, budget)?;
        self.caught.insert((source, bci, ordinal), value);
        Ok(value)
    }

    /// Whether one resolved value carries the class the frames state for the slot.
    ///
    /// A passthrough is the contributor's own value, so 4.1's merge of one contribution is its
    /// state, and this is where the two artifacts that must agree are compared.
    fn check_class(&self, position: usize, slot: Slot, value: ValueId) -> Norm<()> {
        let stated = &self.blocks[position].flow.entry_values[&slot];
        let value_ty = &self.values[value.index()].ty;
        if value_ty != stated {
            return inconsistent(format!(
                "slot {slot:?} of block {:?} is entered with {stated:?} while its only input defines \
                 {value_ty:?} there",
                self.blocks[position].flow.id
            ));
        }
        Ok(())
    }

    /// Records one def-use edge.
    fn record_use(
        &mut self,
        value: ValueId,
        block: &CanonicalBlockId,
        bci: Option<u32>,
        budget: &mut Budget,
    ) -> Norm<()> {
        budget.charge(CountedBudgetDimension::IrEdges, 1)?;
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        self.values[value.index()].uses.push(SsaUse {
            block: block.clone(),
            bci,
        });
        Ok(())
    }

    /// Runs one block to its end: its instructions, its throw-site snapshots and its exit state.
    fn run_block(&mut self, position: usize, budget: &mut Budget) -> Norm<()> {
        if self.blocks[position].done {
            return Ok(());
        }
        let block = self.blocks[position].flow.id.clone();
        let count = self.blocks[position].flow.instructions.len();
        for index in 0..count {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            let instruction = self.blocks[position].flow.instructions[index].clone();
            let depth_before = stack_depth(&self.blocks[position].defs);
            if let Some(site) = &instruction.site {
                self.publish_site(position, instruction.bci, site, budget)?;
            }
            let mut reads: Vec<(Slot, ValueId)> = Vec::new();
            let mut writes: Vec<(Slot, ValueId)> = Vec::new();
            for touch in instruction.accesses {
                let slot = Slot::of(touch.region, touch.slot);
                if touch.write {
                    let Some(ty) = touch.value.clone() else {
                        return inconsistent(format!(
                            "the instruction at BCI {} of block {block:?} writes {slot:?} without \
                             stating the class it writes",
                            instruction.bci
                        ));
                    };
                    if !matches!(touch.width, 1 | 2) {
                        return inconsistent(format!(
                            "the instruction at BCI {} of block {block:?} names the width {} for \
                             {slot:?}: a value occupies one slot or two",
                            instruction.bci, touch.width
                        ));
                    }
                    if touch.region == SlotRegion::Stack
                        && touch.slot.saturating_add(touch.width) > instruction.stack_after
                    {
                        return inconsistent(format!(
                            "the instruction at BCI {} of block {block:?} writes {slot:?} where the \
                             frames hold only {} slots after it",
                            instruction.bci, instruction.stack_after
                        ));
                    }
                    let width = u32::try_from(ty.slots()).unwrap_or(u32::MAX);
                    let value = self.new_value(
                        ty,
                        Definition::Instruction {
                            block: block.clone(),
                            bci: instruction.bci,
                        },
                        self.point(instruction.bci),
                        budget,
                    )?;
                    budget.charge(CountedBudgetDimension::IrItems, 1)?;
                    write_slot(&mut self.blocks[position].defs, slot, value, width);
                    writes.push((slot, value));
                } else {
                    if touch.region == SlotRegion::Stack
                        && touch.slot.saturating_add(touch.width) > depth_before
                    {
                        return inconsistent(format!(
                            "the instruction at BCI {} of block {block:?} reads {slot:?} where the \
                             frames hold only {depth_before} slots before it",
                            instruction.bci
                        ));
                    }
                    let Some((value, _)) = self.blocks[position].defs.get(&slot).copied() else {
                        return inconsistent(format!(
                            "the instruction at BCI {} of block {block:?} reads {slot:?}, which \
                             holds no definition: the frames state a value there and the value flow \
                             does not",
                            instruction.bci
                        ));
                    };
                    self.record_use(value, &block, Some(instruction.bci), budget)?;
                    reads.push((slot, value));
                }
            }
            // The frame's own depth after this instruction: everything above it is gone, and a
            // discard performs no access of its own (`athrow` clears the stack, a `return` ends the
            // block with it).
            self.blocks[position].defs.retain(|slot, _| match slot {
                Slot::Stack(depth) => *depth < instruction.stack_after,
                Slot::Local(_) => true,
            });
            if stack_depth(&self.blocks[position].defs) > instruction.stack_after {
                return inconsistent(format!(
                    "the instruction at BCI {} of block {block:?} leaves the operand stack \
                     deeper than the frames state after it",
                    instruction.bci
                ));
            }
            let stack_delta = i32::try_from(instruction.stack_after)
                .unwrap_or(i32::MAX)
                .saturating_sub(i32::try_from(depth_before).unwrap_or(i32::MAX));
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            self.effects.push(CanonicalInstructionEffect {
                block: block.clone(),
                bci: instruction.bci,
                opcode: instruction.opcode,
                locals_read: sorted_local_reads(&reads),
                locals_written: sorted_local_writes(&writes),
                stack_delta,
                may_throw: instruction.site.is_some(),
                handlers: instruction
                    .site
                    .as_ref()
                    .map(|site| site.handler_ordinals.clone())
                    .unwrap_or_default(),
                origin: instruction
                    .site
                    .as_ref()
                    .map(|site| site.origin.clone())
                    .unwrap_or_default(),
            });
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            self.blocks[position].instructions.push(SsaInstruction {
                bci: instruction.bci,
                opcode: instruction.opcode,
                reads,
                writes,
            });
        }
        let exit: BTreeMap<Slot, ValueId> = self.blocks[position]
            .defs
            .iter()
            .map(|(slot, (value, _))| (*slot, *value))
            .collect();
        budget.charge(
            CountedBudgetDimension::IrItems,
            u64::try_from(exit.len()).unwrap_or(u64::MAX),
        )?;
        self.blocks[position].exit = Some(exit);
        self.blocks[position].done = true;
        // A plain successor reads this exit; a handler reads the site snapshots. Both were
        // registered as waiters before this run, so one wake serves both.
        let waiting = self.waiters.remove(&position).unwrap_or_default();
        for waiter in waiting {
            self.enqueue(Task::Enter(waiter));
        }
        Ok(())
    }

    /// Records the state of a source **before** one throwing instruction, for the slots its handlers
    /// read, and wakes whoever was waiting for it.
    fn publish_site(
        &mut self,
        position: usize,
        bci: u32,
        site: &SiteFlow,
        budget: &mut Budget,
    ) -> Norm<()> {
        let mut snapshot: BTreeMap<Slot, Option<ValueId>> = BTreeMap::new();
        for slot in &site.slots {
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            snapshot.insert(
                *slot,
                self.blocks[position]
                    .defs
                    .get(slot)
                    .map(|(value, _)| *value),
            );
        }
        self.sites.insert((position, bci), snapshot);
        let waiting = self.waiters.remove(&position).unwrap_or_default();
        for waiter in waiting {
            self.enqueue(Task::Enter(waiter));
        }
        Ok(())
    }

    /// Removes every phi whose only usable operand is one other value.
    ///
    /// The replacement is local to this stage: every use of the phi and every definition record
    /// that named it moves to the operand, so the operand becomes the slot's name. A phi that
    /// only refers to itself is left alone — it is a value no path defines, and replacing it would
    /// invent one.
    ///
    /// The phi's **operand list is left as it was**: one operand per logical predecessor, exactly
    /// as [`Assigner::complete`] collected it. That list is not a second name for the replaced
    /// value, it is the record of the merge the phi *was*, and folding it to the one usable
    /// operand would falsify two invariants of the published table at once — the phi would claim
    /// a merge point entered once, while the use records (one per operand, written when each
    /// operand was resolved) still count every operand that took part, and neither number could be
    /// recovered from the other. The consumer follows [`SsaValue::replaced_by`] for the value's
    /// name; the operands and the uses stay readable as what they always were.
    fn simplify(&mut self, budget: &mut Budget) -> Norm<()> {
        loop {
            let mut replaced = None;
            for phi in self.phis.iter() {
                if phi.inputs.is_empty() || self.values[phi.value.index()].replaced_by.is_some() {
                    continue;
                }
                let mut usable: Option<ValueId> = None;
                let mut unique = true;
                for input in &phi.inputs {
                    let value = match input {
                        PhiInput::Itself => phi.value,
                        PhiInput::Value(value) => *value,
                    };
                    if value == phi.value {
                        continue;
                    }
                    match usable {
                        None => usable = Some(value),
                        Some(seen) if seen == value => {}
                        Some(_) => unique = false,
                    }
                }
                if !unique {
                    continue;
                }
                if let Some(target) = usable {
                    replaced = Some((phi.value, target));
                    break;
                }
            }
            let Some((phi, target)) = replaced else {
                return Ok(());
            };
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            self.replace(phi, target, budget)?;
        }
    }

    /// Moves every record that named one value — its uses and every definition of it — to another,
    /// in this same stage.
    ///
    /// What the replaced value keeps is what it *is*: its own operands, which are the merge point
    /// it stood for, and its origin, which is that merge point's. Only the name of the slot and
    /// the def-use edges move.
    ///
    /// The use records are a **multiset of occurrences**, not one record per place: a read touch
    /// records one for every operand of an instruction that holds the value, and an entry records
    /// one for every participant that hands it to a phi. So the same instruction can name one
    /// value twice, and one phi can hold it as two operands. What the two checks below count is
    /// therefore the occurrences rewritten against the records held, because "every record rewrote
    /// at least one occurrence" is false of a multiset: the first record of one place's two
    /// occurrences would answer for the second record as well and the second would find nothing.
    ///
    /// One occurrence is deliberately *not* rewritten: the one that would name its own holder —
    /// the phi being replaced, or a phi whose own value is `to`. It becomes
    /// [`PhiInput::Itself`], the one spelling a self-reference has, and its record is dropped with
    /// it, so the count above stays the exact equality it states and no phi ever publishes its own
    /// value as a `Value` operand. See the module documentation, "a self-reference has one
    /// spelling".
    ///
    /// Which records those are is decided **by block**, never by position in the list. A record of
    /// an entry carries nothing but the block whose phi holds the occurrence — that is the whole
    /// identity the table states for it — while the list order is another fact entirely: an entry
    /// record is pushed by [`Assigner::complete`] as that block resolves its participants, so the
    /// entries of one value are in the order its consumer blocks were completed, and a block a
    /// missing source woke later pushes again at the end. This function, by contrast, walks
    /// [`Assigner::phis`] in table order — the order the phis were created in, which is the order
    /// their blocks were entered. The two are orders of two different traversals of the same
    /// occurrences, and nothing ties them together: "the first `self_named` entry records" would
    /// drop the records of blocks whose occurrences were *not* the rewritten ones while the totals
    /// still added up, and the wrong def-use edge would then be published for both blocks. So the
    /// records are dropped block by block, and a block whose occurrences ask for more records than
    /// it holds is refused below: that is the two readings of one fact disagreeing, not a case to
    /// paper over.
    fn replace(&mut self, from: ValueId, to: ValueId, budget: &mut Budget) -> Norm<()> {
        let uses = std::mem::take(&mut self.values[from.index()].uses);
        self.values[from.index()].replaced_by = Some(to);
        let mut reads_named = 0u64;
        let mut reads_rewritten = 0u64;
        let mut operands_named = 0u64;
        for use_record in &uses {
            let Some(bci) = use_record.bci else {
                operands_named += 1;
                continue;
            };
            reads_named += 1;
            let Some(position) = self.index_of.get(&use_record.block).copied() else {
                return inconsistent(format!(
                    "a use of one value names the block {:?}, which the frames do not hold",
                    use_record.block
                ));
            };
            for instruction in self.blocks[position].instructions.iter_mut() {
                if instruction.bci != bci {
                    continue;
                }
                for read in instruction.reads.iter_mut() {
                    if read.1 == from {
                        read.1 = to;
                        reads_rewritten += 1;
                    }
                }
            }
        }
        if reads_rewritten != reads_named {
            return inconsistent(format!(
                "one value is named as a read {reads_named} time(s) while the instructions at the \
                 named BCIs hold {reads_rewritten} read(s) of it"
            ));
        }
        // Rewriting the phis is one global, idempotent pass: every operand occurrence of the value
        // is answered by exactly one record, wherever the phis that hold it are. An occurrence
        // whose answer would name the **holder's own** value — the phi that is being replaced,
        // whose `replaced_by` now names `to`, and any phi whose own value *is* `to` — is a
        // self-reference and becomes [`PhiInput::Itself`], the one spelling this table gives to
        // "this input's copy is the value the merge point already names". A self-reference is not
        // a use of a value, which is why its record does not move to the target and is dropped
        // below, exactly like the `Itself` operands [`Assigner::complete`] writes directly.
        // Without this, a replacement whose target one phi already defines would leave that phi
        // holding its own value as a *value* operand, and the count below would then find a record
        // for an occurrence the next replacement has to leave alone.
        let mut operands_rewritten = 0u64;
        // The occurrences that became self-references, counted per block: one record of an entry
        // is identified by its block alone, so this is the count each block's records have to
        // answer for. See the remark on dropping below.
        let mut self_named: BTreeMap<CanonicalBlockId, u64> = BTreeMap::new();
        for phi in self.phis.iter_mut() {
            let own = phi.value == from || phi.value == to;
            for input in phi.inputs.iter_mut() {
                if matches!(input, PhiInput::Value(value) if *value == from) {
                    if own {
                        *input = PhiInput::Itself;
                        *self_named.entry(phi.block.clone()).or_default() += 1;
                    } else {
                        *input = PhiInput::Value(to);
                    }
                    operands_rewritten += 1;
                }
            }
        }
        if operands_rewritten != operands_named {
            return inconsistent(format!(
                "one value is named as a phi operand {operands_named} time(s) while the phis hold \
                 it as an operand {operands_rewritten} time(s)"
            ));
        }
        // The records of the occurrences that became self-references, taken **by block**: the
        // number of entry records a block hands over is the number of its phi operands that were
        // rewritten into `Itself`, so a block holding fewer records than that is a record list
        // that does not describe these phis, and it is refused rather than dropped from wherever
        // the list happens to hold them. The list order is not a fact this function may lean on —
        // see its documentation — and the counts below are what make the two readings comparable.
        let self_named_total: u64 = self_named.values().copied().sum();
        let mut left = self_named;
        let mut dropped = 0u64;
        for use_record in uses {
            let is_a_self_reference = use_record.bci.is_none()
                && match left.get_mut(&use_record.block) {
                    Some(count) if *count > 0 => {
                        *count -= 1;
                        true
                    }
                    _ => false,
                };
            if is_a_self_reference {
                dropped += 1;
                continue;
            }
            self.record_use(to, &use_record.block, use_record.bci, budget)?;
        }
        if dropped != self_named_total {
            return inconsistent(format!(
                "one replacement turns {self_named_total} occurrence(s) of one value into \
                 self-references while the records of the blocks holding them answer for {dropped} \
                 of them: the records and the phis are two readings of one fact and cannot disagree"
            ));
        }
        for block in self.blocks.iter_mut() {
            for (value, _) in block.defs.values_mut() {
                if *value == from {
                    *value = to;
                }
            }
            if let Some(exit) = block.exit.as_mut() {
                for value in exit.values_mut() {
                    if *value == from {
                        *value = to;
                    }
                }
            }
            for instruction in block.instructions.iter_mut() {
                for read in instruction.reads.iter_mut() {
                    if read.1 == from {
                        read.1 = to;
                    }
                }
                for write in instruction.writes.iter_mut() {
                    if write.1 == from {
                        write.1 = to;
                    }
                }
            }
        }
        Ok(())
    }

    /// The value a name stands for, following the trivial-phi replacements.
    fn target(&self, value: ValueId) -> ValueId {
        let mut value = value;
        while let Some(next) = self.values[value.index()].replaced_by {
            value = next;
        }
        value
    }

    /// Assembles the published table.
    ///
    /// Charges, per block and before the storage is allocated: one item for the block record, one
    /// per slot the entry state starts a value in, one per slot of the exit state, and one per
    /// element of the instruction copies — the record itself and each of its read and write
    /// records — and one per phi record and per operand of it. The shadow state of the walk and
    /// the published table are both alive while the copies are built, so the copies are storage of
    /// their own and are charged like the records they are copied from. What *moves* — the values
    /// and the canonical effect facts — is not charged again.
    fn publish(self, budget: &mut Budget) -> Norm<SsaTable> {
        let mut blocks = Vec::with_capacity(self.blocks.len());
        for state in &self.blocks {
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            // One published entry record per slot resolution, charged before the vector that
            // holds them is collected.
            budget.charge(
                CountedBudgetDimension::IrItems,
                u64::try_from(state.slots.len()).unwrap_or(u64::MAX),
            )?;
            let entry: Vec<(Slot, ValueId)> = state
                .slots
                .iter()
                .map(|(slot, resolution)| {
                    let value = match resolution {
                        SlotResolution::Ready(value) => *value,
                        SlotResolution::Phi(phi) => self.phis[*phi].value,
                    };
                    (*slot, self.target(value))
                })
                .collect();
            // The exit copy's slots are charged before it is cloned, not after it is sorted.
            budget.charge(
                CountedBudgetDimension::IrItems,
                u64::try_from(state.exit.as_ref().map_or(0, BTreeMap::len)).unwrap_or(u64::MAX),
            )?;
            let mut exit: Vec<(Slot, ValueId)> = state
                .exit
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|(slot, value)| (slot, self.target(value)))
                .collect();
            exit.sort();
            // One item per published instruction record and per read and write record it holds,
            // charged before the copies are made.
            for instruction in &state.instructions {
                let held = instruction
                    .reads
                    .len()
                    .saturating_add(instruction.writes.len())
                    .saturating_add(1);
                budget.charge(
                    CountedBudgetDimension::IrItems,
                    u64::try_from(held).unwrap_or(u64::MAX),
                )?;
            }
            let instructions = state
                .instructions
                .iter()
                .map(|instruction| {
                    let mut renamed = instruction.clone();
                    for read in renamed.reads.iter_mut() {
                        read.1 = self.target(read.1);
                    }
                    for write in renamed.writes.iter_mut() {
                        write.1 = self.target(write.1);
                    }
                    renamed
                })
                .collect::<Vec<SsaInstruction>>();
            blocks.push(SsaBlock {
                block: state.flow.id.clone(),
                entry,
                exit,
                instructions,
            });
        }
        // One item per published phi record and per operand it carries, charged before the copies.
        for phi in &self.phis {
            budget.charge(
                CountedBudgetDimension::IrItems,
                u64::try_from(phi.inputs.len().saturating_add(1)).unwrap_or(u64::MAX),
            )?;
        }
        let phis = self
            .phis
            .iter()
            .map(|phi| {
                let mut renamed = phi.clone();
                for input in renamed.inputs.iter_mut() {
                    if let PhiInput::Value(value) = input {
                        *value = self.target(*value);
                    }
                }
                renamed
            })
            .collect();
        Ok(SsaTable {
            values: self.values,
            phis,
            blocks,
            effects: CanonicalEffectFacts {
                instructions: self.effects,
            },
        })
    }
}

/// Binds a slot to one value, invalidating the halves a category-2 pair shares.
///
/// The frame pass ends both directions of a half-covered pair in `Top`; the shadow state of the name
/// assignment mirrors that, because a read of the half that was left has no value either.
fn write_slot(defs: &mut BTreeMap<Slot, (ValueId, u32)>, slot: Slot, value: ValueId, width: u32) {
    defs.remove(&slot);
    if let Some(upper) = slot.upper().filter(|_| width == 2) {
        defs.remove(&upper);
    }
    let covered = slot
        .lower()
        .filter(|lower| matches!(defs.get(lower), Some((_, 2))));
    if let Some(lower) = covered {
        defs.remove(&lower);
    }
    defs.insert(slot, (value, width));
}

/// The operand-stack depth one block's current state reaches, in slots.
fn stack_depth(defs: &BTreeMap<Slot, (ValueId, u32)>) -> u32 {
    defs.iter()
        .filter_map(|(slot, (_, width))| match slot {
            Slot::Stack(depth) => Some(depth.saturating_add(*width)),
            Slot::Local(_) => None,
        })
        .max()
        .unwrap_or(0)
}

/// The locals one instruction read, ascending and deduplicated.
fn sorted_local_reads(reads: &[(Slot, ValueId)]) -> Vec<u16> {
    let mut locals: Vec<u16> = reads
        .iter()
        .filter_map(|(slot, _)| match slot {
            Slot::Local(index) => Some(*index),
            Slot::Stack(_) => None,
        })
        .collect();
    locals.sort_unstable();
    locals.dedup();
    locals
}

/// The locals one instruction wrote, ascending and deduplicated.
fn sorted_local_writes(writes: &[(Slot, ValueId)]) -> Vec<u16> {
    let mut locals: Vec<u16> = writes
        .iter()
        .filter_map(|(slot, _)| match slot {
            Slot::Local(index) => Some(*index),
            Slot::Stack(_) => None,
        })
        .collect();
    locals.sort_unstable();
    locals.dedup();
    locals
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call_context::{CallContextOutcome, call_contexts};
    use crate::canonical::{CanonicalOutcome, canonical_cfg};
    use crate::cfg::{CfgCompleteness, raw_cfg};
    use crate::frame::{FrameOutcome, RefType, frames};
    use jarde_reader::budget::{BudgetDimension, Limits};
    use jarde_reader::classfile::{
        CpEntryFacts, class_facts, method_code_facts, test_class::single_method,
    };
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalVariant, SnapshotId,
    };
    use jarde_reader::view::LoaderId;

    /// Access flags of the fixture method: `ACC_PUBLIC | ACC_STATIC`, like the reader's builder.
    const ACC_STATIC: u16 = 0x0009;

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

    /// The same limits with the item dimension replaced, for the accounting cases below.
    fn budget_with_items(items: u64) -> Budget {
        let mut limits = limits();
        limits.ir_items = items;
        Budget::new(limits)
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

    /// One body of the fixtures below: the decoded facts, the class file's own pool, the
    /// canonical graph and the frames 4.1 published for it.
    struct Body {
        facts: MethodCodeFacts,
        pool: Vec<CpEntryFacts>,
        canonical: CanonicalCfg,
        frames: Box<FrameTable>,
    }

    fn body_of(bytes: &[u8]) -> (Body, CanonicalCfg) {
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
        let method = method_view(&pool, &loader);
        let frames = match frames(&facts, &canonical, &method, &mut budget)
            .expect("a legal run is answered")
        {
            FrameOutcome::Frames(table) => table,
            other => panic!("the fixture must have frames, got {other:?}"),
        };
        (
            Body {
                facts,
                pool,
                canonical: canonical.clone(),
                frames,
            },
            canonical,
        )
    }

    fn body_of_code(code: &[u8], max_locals: u16) -> (Body, CanonicalCfg) {
        body_of(&single_method(52, 8, max_locals, code))
    }

    fn method_view<'a>(pool: &'a [CpEntryFacts], loader: &'a LoaderId) -> FrameMethod<'a> {
        FrameMethod {
            access_flags: ACC_STATIC,
            name: b"method",
            descriptor: b"()V",
            owner: b"Test",
            super_class: Some(b"java/lang/Object"),
            pool,
            loader,
        }
    }

    /// One run of the pass over a fixture body, or the contradiction it reported.
    fn ssa_of(body: &Body) -> std::result::Result<SsaTable, String> {
        let loader = LoaderId("app".to_string());
        let method = method_view(&body.pool, &loader);
        match ssa(
            &body.facts,
            &body.canonical,
            &body.frames,
            &method,
            &mut budget(),
        )
        .expect("the budget is ample")
        {
            SsaOutcome::Ssa(table) => Ok(*table),
            SsaOutcome::Inconsistent { message } => Err(message),
        }
    }

    /// The block one fixture body's BCI starts, as the canonical graph names it.
    fn block(bci: u32) -> CanonicalBlockId {
        CanonicalBlockId {
            bci,
            path: Vec::new(),
        }
    }

    /// The phi one block and slot has, or the panic that says it has none.
    fn phi<'a>(table: &'a SsaTable, block: &CanonicalBlockId, slot: Slot) -> &'a SsaPhi {
        table
            .phis()
            .iter()
            .find(|phi| &phi.block == block && phi.slot == slot)
            .unwrap_or_else(|| panic!("{block:?} {slot:?} has a phi: {:#?}", table.phis()))
    }

    /// The entry definition of one slot of one block.
    fn entry_of(table: &SsaTable, block: &CanonicalBlockId, slot: Slot) -> ValueId {
        table
            .block(block)
            .unwrap_or_else(|| panic!("{block:?} is named"))
            .entry
            .iter()
            .find(|(found, _)| *found == slot)
            .unwrap_or_else(|| panic!("{block:?} names {slot:?}"))
            .1
    }

    #[test]
    fn a_straight_line_body_names_every_read_and_every_write() {
        // 0 iconst_1, 1 istore_0, 2 iload_0, 3 istore_1, 4 return.
        let (body, _) = body_of_code(&[0x04, 0x3b, 0x1a, 0x3c, 0xb1], 2);
        let table = ssa_of(&body).expect("a straight line body analyzes");
        assert_eq!(table.blocks().len(), 1);
        let named = table.blocks()[0].clone();
        assert_eq!(named.instructions.len(), 5);
        let store = named.instructions[1].clone();
        assert_eq!(store.reads.len(), 1, "`istore_0` reads the value it stores");
        assert_eq!(store.writes.len(), 1, "`istore_0` writes one slot");
        assert_eq!(store.writes[0].0, Slot::Local(0));
        let stored = store.writes[0].1;
        assert_eq!(
            table.value(stored).def,
            Definition::Instruction {
                block: block(0),
                bci: 1
            }
        );
        let load = named.instructions[2].clone();
        assert_eq!(load.reads, vec![(Slot::Local(0), stored)]);
        assert!(
            table.value(stored).uses.contains(&SsaUse {
                block: block(0),
                bci: Some(2)
            }),
            "the load's use is recorded on the definition: {:#?}",
            table.value(stored).uses
        );
        // The exit names both locals with the last value written into each.
        assert_eq!(named.exit[0].0, Slot::Local(0));
        assert_eq!(named.exit[1].0, Slot::Local(1));
        // The effects are per instruction and carry the class of every local, with the measured
        // stack delta: a push is +1, a store -1, and `return` ends the block on an empty stack.
        let effects = &table.effects().instructions;
        assert_eq!(effects.len(), 5);
        assert_eq!(
            effects
                .iter()
                .map(|effect| effect.stack_delta)
                .collect::<Vec<_>>(),
            vec![1, -1, 1, -1, 0]
        );
        assert_eq!(effects[1].locals_written, vec![0]);
        assert_eq!(effects[2].locals_read, vec![0]);
        assert!(effects.iter().all(|effect| !effect.may_throw));
    }

    #[test]
    fn a_diamond_merge_names_one_phi_per_merged_slot() {
        // 0 iconst_0, 1 ifeq -> 8, 4 iconst_1, 5 goto -> 9, 8 iconst_2, 9 istore_0, 10 return.
        let code = [
            0x03, // 0: iconst_0
            0x99, 0x00, 0x07, // 1: ifeq 8
            0x04, // 4: iconst_1
            0xa7, 0x00, 0x04, // 5: goto 9
            0x05, // 8: iconst_2
            0x3b, // 9: istore_0
            0xb1, // 10: return
        ];
        let (body, canonical) = body_of_code(&code, 1);
        let table = ssa_of(&body).expect("a diamond analyzes");
        // The join is the block at BCI 9, and the frames enter it with one stack slot that both
        // arms provide.
        let join = block(9);
        assert_eq!(
            canonical
                .edges
                .iter()
                .filter(|edge| edge.to == join)
                .count(),
            2,
            "the join is entered by both arms"
        );
        let state = body.frames.entry(&join).expect("the join is reached");
        assert_eq!(state.inputs.len(), 2, "two logical predecessors");
        let slot = Slot::Stack(0);
        let merged = phi(&table, &join, slot);
        assert_eq!(
            merged.inputs.len(),
            2,
            "one operand per logical predecessor"
        );
        assert_eq!(
            table.value(merged.value).ty,
            state.stack[0],
            "the phi carries the class 4.1 merged for this slot"
        );
        let operands: Vec<ValueId> = merged
            .inputs
            .iter()
            .map(|input| match input {
                PhiInput::Value(value) => *value,
                PhiInput::Itself => panic!("a diamond's arms hand the join their own values"),
            })
            .collect();
        let defs: Vec<Definition> = operands
            .iter()
            .map(|value| table.value(*value).def.clone())
            .collect();
        assert_eq!(
            defs,
            vec![
                Definition::Instruction {
                    block: block(4),
                    bci: 4
                },
                Definition::Instruction {
                    block: block(8),
                    bci: 8
                },
            ],
            "the operands are the two values the arms push"
        );
        assert_eq!(
            table.value(merged.value).def,
            Definition::Phi { block: join, slot }
        );
        // The store at BCI 9 uses the phi, so the phi's use is recorded with no instruction of
        // its own: an operand read at the merge point.
        assert!(table.value(merged.value).uses.contains(&SsaUse {
            block: block(9),
            bci: Some(9),
        }));
        assert_eq!(
            table.blocks().len(),
            body.frames.blocks().len(),
            "every block the frames hold is named"
        );
    }

    /// The published table is a copy of the shadow state, and the item charge counts it: one run
    /// bills the walk **and** what it publishes.
    ///
    /// The two prices are told apart without a second implementation of the walk: one run publishes
    /// and one stops just before the publication, and the difference between their bills is
    /// compared with the price of the copies computed from the table the first one published — one
    /// item per block record, per entry and exit record, per instruction record together with its
    /// read and write records, and per phi record together with its operands. A copy made without
    /// charging for it shortens that difference. The same claim is then stated as the observable
    /// stop: a request allowed exactly the walk's price must not publish.
    #[test]
    fn the_published_table_is_billed_by_the_copies_it_makes() {
        // 0 iconst_0, 1 ifeq -> 8, 4 iconst_1, 5 goto -> 9, 8 iconst_2, 9 istore_0, 10 return.
        let code = [
            0x03, // 0: iconst_0
            0x99, 0x00, 0x07, // 1: ifeq 8
            0x04, // 4: iconst_1
            0xa7, 0x00, 0x04, // 5: goto 9
            0x05, // 8: iconst_2
            0x3b, // 9: istore_0
            0xb1, // 10: return
        ];
        let (body, _) = body_of_code(&code, 1);
        let loader = LoaderId("app".to_string());
        let method = method_view(&body.pool, &loader);
        let mut ample = budget();
        let table = match ssa(
            &body.facts,
            &body.canonical,
            &body.frames,
            &method,
            &mut ample,
        )
        .expect("the budget is ample")
        {
            SsaOutcome::Ssa(table) => *table,
            SsaOutcome::Inconsistent { message } => panic!("a diamond names: {message}"),
        };
        let total = ample.usage().ir_items;
        let copies = std::iter::once(u64::try_from(table.blocks().len()).expect("a small fixture"))
            .chain(table.blocks().iter().map(|block| {
                let mut items =
                    u64::try_from(block.entry.len() + block.exit.len()).expect("a small fixture");
                for instruction in &block.instructions {
                    items += u64::try_from(1 + instruction.reads.len() + instruction.writes.len())
                        .expect("a small fixture");
                }
                items
            }))
            .chain(
                table
                    .phis()
                    .iter()
                    .map(|phi| u64::try_from(1 + phi.inputs.len()).expect("a small fixture")),
            )
            .sum::<u64>();
        assert!(
            copies > 0 && copies < total,
            "this fixture really publishes something the walk is billed apart from: {copies} of \
             {total}"
        );

        // The walk alone fits exactly under this limit; the first published record does not.
        let walk = total - copies;
        let mut stopped = budget_with_items(walk);
        let outcome = ssa(
            &body.facts,
            &body.canonical,
            &body.frames,
            &method,
            &mut stopped,
        );
        match outcome {
            Err(error) => assert!(
                matches!(
                    error,
                    Error::BudgetExceeded {
                        dimension: BudgetDimension::IrItems,
                        ..
                    }
                ),
                "the copy is what ran into the limit: {error:?}"
            ),
            Ok(_) => panic!(
                "a run that cannot afford its own publication must not publish: the walk costs \
                 {walk} and the run was allowed exactly that"
            ),
        }
        assert_eq!(
            stopped.usage().ir_items,
            walk,
            "the charges that fit are the walk's own"
        );
    }

    #[test]
    fn a_loop_phi_takes_its_own_value_on_the_path_that_never_writes_it() {
        // 0 iconst_0, 1 istore_0, 2 iload_0, 3 ifle -> 18, 6 iinc 0 1, 9 goto 2, 12 goto 2,
        // 15 return: the header at BCI 2 is entered by the fall-through, by the arm that
        // increments the local and by the arm that does not write it at all.
        let code = [
            0x03, // 0: iconst_0
            0x3b, // 1: istore_0
            0x1a, // 2: iload_0
            0x9e, 0x00, 0x09, // 3: ifle 12
            0x84, 0x00, 0x01, // 6: iinc 0, 1
            0xa7, 0xff, 0xf9, // 9: goto 2
            0xa7, 0xff, 0xf6, // 12: goto 2
            0xb1, // 15: return
        ];
        let (body, _) = body_of_code(&code, 1);
        let table = ssa_of(&body).expect("a loop analyzes");
        let header = block(2);
        let state = body.frames.entry(&header).expect("the header is reached");
        assert_eq!(state.inputs.len(), 3, "three logical predecessors");
        let merged = phi(&table, &header, Slot::Local(0));
        assert_eq!(merged.inputs.len(), 3);
        assert_eq!(
            merged
                .inputs
                .iter()
                .filter(|input| **input == PhiInput::Itself)
                .count(),
            1,
            "the arm that never writes the local hands the header the header's own value"
        );
        assert!(
            table.value(merged.value).replaced_by.is_none(),
            "two distinct operands are not trivial"
        );
    }

    #[test]
    fn a_cycle_with_two_entries_is_named_at_its_merge_point() {
        // 0 iconst_1, 1 istore_0, 2 iconst_0, 3 ifeq -> 9, 6 goto -> 9,
        // 9 iload_0, 10 iconst_1, 11 isub, 12 istore_0, 13 iload_0, 14 ifgt -> 9, 17 return.
        //
        // The cycle is the block at BCI 9 looping into itself through `ifgt`, and it has **two**
        // entries from outside it (the `ifeq` and the `goto`), which is what makes the control
        // flow irreducible in the sense this slice cares about: the merge point is entered from
        // inside the cycle and from two different places outside it, so a name resolved from any
        // single predecessor would be wrong.
        let code = [
            0x04, // 0: iconst_1
            0x3b, // 1: istore_0
            0x03, // 2: iconst_0
            0x99, 0x00, 0x06, // 3: ifeq 9
            0xa7, 0x00, 0x03, // 6: goto 9
            0x1a, // 9: iload_0
            0x04, // 10: iconst_1
            0x64, // 11: isub
            0x3b, // 12: istore_0
            0x1a, // 13: iload_0
            0x9d, 0xff, 0xfb, // 14: ifgt 9
            0xb1, // 17: return
        ];
        let (body, _) = body_of_code(&code, 1);
        let table = ssa_of(&body).expect("a cycle with two entries analyzes");
        assert_eq!(
            table.blocks().len(),
            body.frames.blocks().len(),
            "every block the frames hold is named"
        );
        let header = block(9);
        assert_eq!(
            body.frames.entry(&header).expect("reached").inputs.len(),
            3,
            "two entries from outside the cycle and one from the cycle itself"
        );
        let merged = phi(&table, &header, Slot::Local(0));
        assert_eq!(merged.inputs.len(), 3);
        assert_eq!(
            table.value(merged.value).ty,
            entry_class(&body, &header, Slot::Local(0))
        );
        // The cycle carries no operand stack at its entry, so no stack phi is invented.
        assert!(
            table
                .phis()
                .iter()
                .all(|phi| matches!(phi.slot, Slot::Local(_))),
            "only the local merges: {:#?}",
            table.phis()
        );
        for phi in table.phis() {
            let state = body
                .frames
                .entry(&phi.block)
                .expect("a named block is reached");
            assert!(
                phi.inputs.len() <= state.inputs.len() + 1,
                "{:?} {:?} counts at most its logical inputs (and the method's own state for the \
                 entry block): {} vs {}",
                phi.block,
                phi.slot,
                phi.inputs.len(),
                state.inputs.len()
            );
            assert!(phi.inputs.len() >= 2, "a phi has at least two operands");
            assert_eq!(
                table.value(phi.value).ty,
                entry_class(&body, &phi.block, phi.slot),
                "the phi carries the class the frames state for its slot"
            );
        }
    }

    /// The class the frames state for one slot of one block.
    fn entry_class(body: &Body, block: &CanonicalBlockId, slot: Slot) -> Value {
        let state = body.frames.entry(block).expect("the block is reached");
        match slot {
            Slot::Local(index) => state.locals[usize::from(index)].clone(),
            Slot::Stack(depth) => {
                state.stack[usize::try_from(depth).expect("a small depth")].clone()
            }
        }
    }

    #[test]
    fn a_top_slot_never_becomes_a_value_or_a_phi() {
        // 0 iconst_0, 1 ifeq -> 9, 4 iconst_1, 5 istore_1, 6 goto -> 11, 9 iconst_2, 10 pop,
        // 11 return: local 1 is written on one arm only, so the merge answers `Top` and neither
        // arm's value may reach the join as a definition.
        let code = [
            0x03, // 0: iconst_0
            0x99, 0x00, 0x08, // 1: ifeq 9
            0x04, // 4: iconst_1
            0x3c, // 5: istore_1
            0xa7, 0x00, 0x05, // 6: goto 11
            0x05, // 9: iconst_2
            0x57, // 10: pop
            0xb1, // 11: return
        ];
        let (body, _) = body_of_code(&code, 2);
        let table = ssa_of(&body).expect("a body whose dead local disagrees analyzes");
        let join = block(9);
        let state = body.frames.entry(&join).expect("the join is reached");
        assert_eq!(state.locals[1], Value::Top, "the merge answers Top");
        assert!(
            table.phis().is_empty(),
            "no slot of this body merges two usable values: {:#?}",
            table.phis()
        );
        let named = table.block(&join).expect("the join is named");
        assert!(
            named.entry.is_empty(),
            "a Top slot and an empty stack start no value: {:#?}",
            named.entry
        );
    }

    #[test]
    fn a_category_two_value_is_one_value_in_its_two_slots() {
        // 0 lconst_0, 1 lstore_0, 2 lload_0, 3 lstore_2, 4 return.
        let code = [0x09, 0x3f, 0x1e, 0x41, 0xb1];
        let (body, _) = body_of_code(&code, 4);
        let table = ssa_of(&body).expect("a category-2 body analyzes");
        let named = table.block(&block(0)).expect("the entry block is named");
        let store = named.instructions[1].clone();
        assert_eq!(
            store.writes.len(),
            1,
            "a category-2 store is one write, not two: {:#?}",
            store.writes
        );
        assert_eq!(store.writes[0].0, Slot::Local(0));
        let stored = store.writes[0].1;
        assert_eq!(table.value(stored).ty, Value::Long);
        let load = named.instructions[2].clone();
        assert_eq!(
            load.reads,
            vec![(Slot::Local(0), stored)],
            "the load reads the value's first slot and never its upper half"
        );
        assert_eq!(load.writes.len(), 1);
        assert_eq!(load.writes[0].0, Slot::Stack(0));
        assert!(
            named
                .exit
                .iter()
                .all(|(slot, _)| !matches!(slot, Slot::Local(1) | Slot::Local(3)))
        );
        assert!(named.exit.iter().any(|(slot, _)| *slot == Slot::Local(2)));
        let effects = &table.effects().instructions;
        assert_eq!(effects[0].stack_delta, 2, "a category-2 push is two slots");
        assert_eq!(effects[1].stack_delta, -2);
        assert_eq!(
            effects[1].locals_written,
            vec![0],
            "one local, never its half"
        );
        assert_eq!(effects[2].locals_read, vec![0]);
    }

    // -----------------------------------------------------------------------
    // The exception merge: the case the phi arity rule exists for
    // -----------------------------------------------------------------------

    /// A class file with one static `method()V` whose `Code` attribute carries an exception table.
    ///
    /// The reader's own builder writes no handler record, and the exception merge is the case this
    /// slice's phi rule exists for, so the fixture is assembled here: a constant pool naming the
    /// class, its superclass and `Code`, and one method whose body is the caller's bytes.
    fn class_with_handlers(
        code: &[u8],
        max_stack: u16,
        max_locals: u16,
        handlers: &[(u16, u16, u16, u16)],
    ) -> Vec<u8> {
        fn u16_be(bytes: &mut Vec<u8>, value: u16) {
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        fn utf8(bytes: &mut Vec<u8>, value: &[u8]) {
            bytes.push(1);
            u16_be(bytes, u16::try_from(value.len()).expect("a small name"));
            bytes.extend_from_slice(value);
        }
        fn class(bytes: &mut Vec<u8>, name: u16) {
            bytes.push(7);
            u16_be(bytes, name);
        }
        let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 52);
        u16_be(&mut bytes, 8); // one entry per index 1..=7
        utf8(&mut bytes, b"Test"); // 1
        class(&mut bytes, 1); // 2
        utf8(&mut bytes, b"java/lang/Object"); // 3
        class(&mut bytes, 3); // 4
        utf8(&mut bytes, b"Code"); // 5
        utf8(&mut bytes, b"method"); // 6
        utf8(&mut bytes, b"()V"); // 7
        u16_be(&mut bytes, 0x0021); // ACC_PUBLIC | ACC_SUPER
        u16_be(&mut bytes, 2); // this_class
        u16_be(&mut bytes, 4); // super_class
        u16_be(&mut bytes, 0); // interfaces
        u16_be(&mut bytes, 0); // fields
        u16_be(&mut bytes, 1); // methods
        u16_be(&mut bytes, 0x0009); // ACC_PUBLIC | ACC_STATIC
        u16_be(&mut bytes, 6);
        u16_be(&mut bytes, 7);
        u16_be(&mut bytes, 1); // one attribute
        u16_be(&mut bytes, 5);
        let mut content = Vec::new();
        u16_be(&mut content, max_stack);
        u16_be(&mut content, max_locals);
        content.extend_from_slice(
            &u32::try_from(code.len())
                .expect("fixture code fits u32")
                .to_be_bytes(),
        );
        content.extend_from_slice(code);
        u16_be(
            &mut content,
            u16::try_from(handlers.len()).expect("few handlers"),
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
                .expect("fixture content fits u32")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(&content);
        u16_be(&mut bytes, 0); // attributes of the class
        bytes
    }

    #[test]
    fn an_exception_merge_counts_throw_sites_and_never_raw_edges() {
        // One protected region holding **two** throwing instructions, both feeding the same
        // handler record — and therefore the same raw edge:
        //
        // 0  iconst_1, 1  istore_1, 2  iconst_1, 3  iconst_0, 4  idiv   (throw site A)
        // 5  iconst_2, 6  istore_1, 7  iconst_1, 8  iconst_0, 9  idiv   (throw site B)
        // 10 pop, 11 pop, 12 return, 13 iload_1, 14 pop, 15 pop, 16 return
        //
        // with one catch-all record covering 0..12 and entering the handler at 13. Local 1 holds
        // a *different* value at the two sites, and the handler reads it: the entry phi of the
        // handler has to count the two logical inputs this slice's rule describes, not the single
        // aggregated edge the raw graph holds.
        let code = [
            0x04, // 0: iconst_1
            0x3c, // 1: istore_1
            0x04, // 2: iconst_1
            0x03, // 3: iconst_0
            0x6c, // 4: idiv
            0x05, // 5: iconst_2
            0x3c, // 6: istore_1
            0x04, // 7: iconst_1
            0x03, // 8: iconst_0
            0x6c, // 9: idiv
            0x57, // 10: pop
            0x57, // 11: pop
            0xb1, // 12: return
            0x1b, // 13: iload_1
            0x57, // 14: pop
            0x57, // 15: pop
            0xb1, // 16: return
        ];
        let bytes = class_with_handlers(&code, 3, 2, &[(0, 13, 13, 0)]);
        let (body, canonical) = body_of(&bytes);
        let table = ssa_of(&body).expect("a body with two throw sites analyzes");
        let handler = block(13);
        let source = block(0);
        assert_eq!(
            canonical
                .edges
                .iter()
                .filter(
                    |edge| matches!(edge.kind, CanonicalEdgeKind::Exception { .. })
                        && edge.from == source
                        && edge.to == handler
                )
                .count(),
            1,
            "the raw graph aggregates both throw sites into one exception edge"
        );
        assert_eq!(
            canonical
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
        // The local the handler reads carries a value on both paths, and the two values are
        // different definitions: the merge point needs a phi with one operand per site.
        let merged = phi(&table, &handler, Slot::Local(1));
        assert_eq!(
            merged.inputs.len(),
            2,
            "the phi counts throw sites, not the aggregated edge: {:#?}",
            merged.inputs
        );
        let operands: Vec<Definition> = merged
            .inputs
            .iter()
            .map(|input| match input {
                PhiInput::Value(value) => table.value(*value).def.clone(),
                PhiInput::Itself => panic!("neither site's local is the handler's own value"),
            })
            .collect();
        assert_eq!(
            operands,
            vec![
                Definition::Instruction {
                    block: source.clone(),
                    bci: 1
                },
                Definition::Instruction {
                    block: source.clone(),
                    bci: 6
                },
            ],
            "each operand is the value that site leaves in the local"
        );
        // The exception reference itself is one value per throw site, with its own origin: the
        // handler is entered with the thrown object of the site that threw.
        let thrown = phi(&table, &handler, Slot::Stack(0));
        assert_eq!(thrown.inputs.len(), 2);
        let caught: Vec<Definition> = thrown
            .inputs
            .iter()
            .map(|input| match input {
                PhiInput::Value(value) => table.value(*value).def.clone(),
                PhiInput::Itself => panic!("the caught reference is never the handler's own value"),
            })
            .collect();
        assert_eq!(
            caught,
            vec![
                Definition::Caught {
                    block: source.clone(),
                    bci: 4
                },
                Definition::Caught {
                    block: source.clone(),
                    bci: 9
                },
            ]
        );
        for input in thrown.inputs.clone() {
            let PhiInput::Value(value) = input else {
                continue;
            };
            assert_eq!(table.value(value).ty, Value::Ref(RefType::Unknown));
            assert_eq!(
                table.value(value).origin.members.len(),
                1,
                "the caught value of one site carries that site's own origin"
            );
        }
        // The effects of the handler's instructions belong to the handler, and the sites' own
        // handlers belong to the throwing BCIs — never to the end of the block that holds them.
        let effects = &table.effects().instructions;
        let site_a = effects
            .iter()
            .find(|effect| effect.bci == 4)
            .expect("the site has an effect record");
        assert!(site_a.may_throw);
        assert_eq!(site_a.handlers, vec![0]);
        let site_b = effects
            .iter()
            .find(|effect| effect.bci == 9)
            .expect("the site has an effect record");
        assert!(site_b.may_throw);
        assert_eq!(site_b.handlers, vec![0]);
        assert!(
            effects
                .iter()
                .find(|effect| effect.bci == 11)
                .is_some_and(|effect| !effect.may_throw),
            "the block's last instruction carries no handler of its own"
        );
    }

    // -----------------------------------------------------------------------
    // The protocol itself, over flows a test names
    // -----------------------------------------------------------------------
    /// One fabricated block.
    fn flow(
        bci: u32,
        entry: Vec<(Slot, Value)>,
        inputs: Vec<FlowInput>,
        instructions: Vec<InstructionFlow>,
    ) -> BlockFlow {
        BlockFlow {
            id: block(bci),
            origin: OriginSet::default(),
            entry_values: entry.into_iter().collect(),
            inputs,
            instructions,
        }
    }

    fn from_seed() -> FlowInput {
        FlowInput {
            kind: InputKind::Seed,
            from: None,
            throw_site: None,
        }
    }

    fn from_transfer(bci: u32) -> FlowInput {
        FlowInput {
            kind: InputKind::Transfer,
            from: Some(block(bci)),
            throw_site: None,
        }
    }

    fn instruction(bci: u32, accesses: Vec<SlotTouch>) -> InstructionFlow {
        InstructionFlow {
            bci,
            opcode: 0x00,
            stack_after: 0,
            accesses,
            site: None,
        }
    }

    fn wrote(slot: u32, value: Value) -> SlotTouch {
        SlotTouch {
            region: SlotRegion::Local,
            slot,
            width: 1,
            write: true,
            value: Some(value),
        }
    }

    /// Runs the naming half over fabricated flows: the empty graph and the empty pool are all the
    /// exception paths would need, and none of these flows holds one.
    fn fabricated(
        flows: Vec<BlockFlow>,
        seed: BTreeMap<Slot, Value>,
    ) -> std::result::Result<SsaTable, String> {
        let canonical = CanonicalCfg {
            blocks: Vec::new(),
            edges: Vec::new(),
            throw_sites: Vec::new(),
            handler_rows: Vec::new(),
            unreachable: Vec::new(),
            clones: 0,
            completeness: CfgCompleteness::Complete,
        };
        let pool: Vec<CpEntryFacts> = Vec::new();
        let loader = LoaderId("app".to_string());
        let method = method_view(&pool, &loader);
        let mut budget = budget();
        let mut assigner = match Assigner::new(flows, method_id(), seed, &mut budget) {
            Ok(assigner) => assigner,
            Err(error) => return Err(fault(error)),
        };
        if let Err(error) = assigner.assign(&canonical, &method, &mut budget) {
            return Err(fault(error));
        }
        if let Err(error) = assigner.simplify(&mut budget) {
            return Err(fault(error));
        }
        match assigner.publish(&mut budget) {
            Ok(table) => Ok(table),
            Err(error) => Err(fault(error)),
        }
    }

    fn fault(error: Problem) -> String {
        match error {
            Problem::Inconsistent(message) => message,
            Problem::Budget(error) => panic!("the fixture budget is ample: {error}"),
        }
    }

    #[test]
    fn a_phi_with_one_usable_operand_is_replaced_and_its_uses_move_with_it() {
        // Two blocks: the entry hands the second one its value along **both** of its inputs, so
        // the phi names one value and nothing else.
        let seed: BTreeMap<Slot, Value> = [(Slot::Local(0), Value::Int)].into_iter().collect();
        let flows = vec![
            flow(
                0,
                vec![(Slot::Local(0), Value::Int)],
                vec![from_seed()],
                vec![instruction(1, vec![wrote(0, Value::Int)])],
            ),
            flow(
                8,
                vec![(Slot::Local(0), Value::Int)],
                vec![from_transfer(0), from_transfer(0)],
                vec![],
            ),
        ];
        let table = fabricated(flows, seed).expect("the fabricated flows resolve");
        let phi = phi(&table, &block(8), Slot::Local(0));
        let target = table
            .block(&block(0))
            .expect("the entry block is named")
            .exit[0]
            .1;
        assert_eq!(
            table.value(phi.value).replaced_by,
            Some(target),
            "the phi is the operand's name"
        );
        assert!(
            table.value(phi.value).uses.is_empty(),
            "every use moved to the operand: {:#?}",
            table.value(phi.value).uses
        );
        assert_eq!(
            table.value(target).uses.len(),
            2,
            "one def-use edge per operand of the removed phi"
        );
        assert!(
            table
                .value(target)
                .uses
                .iter()
                .all(|use_record| use_record.bci.is_none() && use_record.block == block(8))
        );
        assert_eq!(entry_of(&table, &block(8), Slot::Local(0)), target);
    }

    #[test]
    fn a_replaced_phi_keeps_one_operand_per_logical_predecessor() {
        // A merge point entered by two paths that hand it the same value and by its own back edge:
        // one usable operand, so the phi *is* that operand's name — but it merged three logical
        // predecessors, and both records of that stay readable in the published table.
        let seed: BTreeMap<Slot, Value> = [(Slot::Local(0), Value::Int)].into_iter().collect();
        let flows = vec![
            flow(
                0,
                vec![(Slot::Local(0), Value::Int)],
                vec![from_seed()],
                vec![instruction(1, vec![wrote(0, Value::Int)])],
            ),
            flow(
                8,
                vec![(Slot::Local(0), Value::Int)],
                vec![from_transfer(0), from_transfer(0), from_transfer(8)],
                vec![],
            ),
        ];
        let table = fabricated(flows, seed).expect("the fabricated flows resolve");
        let merged = phi(&table, &block(8), Slot::Local(0));
        let target = table
            .block(&block(0))
            .expect("the entry block is named")
            .exit[0]
            .1;
        assert_eq!(
            table.value(merged.value).replaced_by,
            Some(target),
            "one usable operand, so the phi stops being a definition"
        );
        assert_eq!(
            merged.inputs,
            vec![
                PhiInput::Value(target),
                PhiInput::Value(target),
                PhiInput::Itself
            ],
            "the merge keeps one operand per logical predecessor — three, not the one that stayed \
             usable — and its own value keeps its place among them"
        );
        let occurrences = merged
            .inputs
            .iter()
            .filter(|input| matches!(input, PhiInput::Value(value) if *value == target))
            .count();
        assert_eq!(occurrences, 2, "two of the three operands name the target");
        assert_eq!(
            table.value(target).uses.len(),
            occurrences,
            "the use records are the operand occurrences, and an `Itself` operand is no use of \
             the target"
        );
        assert!(
            table.value(merged.value).uses.is_empty(),
            "every use of the replaced phi moved to the target"
        );
        assert_eq!(entry_of(&table, &block(8), Slot::Local(0)), target);
        audit("a replaced phi", &table);
    }

    #[test]
    fn a_loop_head_that_is_handed_its_own_value_spells_it_as_itself() {
        // The 27-byte loop `tests/p2_ssa.rs` drives through the public entry: local 0 is written
        // once, the loop's own paths hand the head's phi that value through two other phis, and
        // those two phis are replaced by the head's own value. The operand the replacement
        // rewrites is therefore one the head already names — and the published list has to say so
        // in the one spelling this table gives a self-reference, `Itself`, with no use record
        // behind it. A table that kept the rewritten occurrence as `Value(own)` would state the
        // same thing in the form its records cannot account for, and the next replacement of that
        // value would have to leave the occurrence alone while still holding a record for it.
        let code: &[u8] = &[
            0x04, 0x3b, 0x04, 0x3c, 0x04, 0x3d, 0x04, 0x99, 0x00, 0x03, 0x04, 0x99, 0xff, 0xff,
            0x04, 0x99, 0xff, 0xfb, 0x1b, 0x04, 0x60, 0x3d, 0x04, 0x99, 0xff, 0xf7, 0xb1,
        ];
        let (body, canonical) = body_of_code(code, 3);
        assert_eq!(
            canonical
                .blocks
                .iter()
                .map(|b| b.id.bci)
                .collect::<Vec<_>>(),
            vec![0, 10, 14, 18, 26],
            "the loop's blocks are the ones the branches name"
        );
        let table = ssa_of(&body).expect("the loop names its values");
        let head = phi(&table, &block(10), Slot::Local(0));
        let target = table
            .value(head.value)
            .replaced_by
            .expect("every path hands the head one value, so it stops being a definition");
        assert_eq!(
            head.inputs,
            vec![PhiInput::Value(target), PhiInput::Itself, PhiInput::Itself],
            "one operand per logical predecessor, and the two the head's own value arrives by are \
             `Itself`"
        );
        assert!(
            table.value(head.value).uses.is_empty(),
            "every use of the replaced phi moved to the target"
        );
        audit("the loop head of the 27-byte body", &table);
    }

    #[test]
    fn a_replacement_records_the_block_that_holds_each_occurrence() {
        // The one body of this crate where dropping the records of the occurrences that became
        // self-references **by position** would publish a wrong def-use relation: the records of
        // one value sit in three different blocks, and the replacement hands one of them a
        // self-reference while the other two keep naming the target.
        //
        // The body is one of the corpus the property run generates (`legal_body` of
        // `tests/p2_properties.rs`, drawn from the search that looked for exactly this shape) — a
        // nest of loops over three int locals, a `()V` method at class-file version 52, 59 bytes:
        //
        // ```text
        //  0: iconst_1; istore_0; iconst_1; istore_1; iconst_1; istore_2
        //  6: iload_2; iconst_1; iadd; istore_1
        // 10: iconst_1; ifeq 50
        // 14: iload_2; iconst_1; iadd; istore_1; iload_2; iconst_3; iadd; istore_0
        // 22: iconst_1; ifeq 26
        // 26: iload_0; iconst_2; iadd; istore_0      the loop head the back edges at 39 and 47 name
        // 30: iconst_1; ifeq 42
        // 34: iload_1; iconst_2; iadd; istore_1
        // 38: iconst_1; ifeq 26
        // 42: iload_1; iconst_3; iadd; istore_1
        // 46: iconst_1; ifeq 26
        // 50: iload_1; iconst_1; iadd; istore_1; iload_2; iconst_2; iadd; istore_2; return
        // ```
        //
        // `istore_2` at BCI 5 defines the value the body's trivial phi is replaced *by*, and the
        // merge points of three blocks name that one value: BCI 26's head twice-over through its
        // back edges, and BCI 42 and BCI 50 once each. The three blocks complete their phis in an
        // order that is the wake order of the fixpoint, not the phis table's order, so a removal
        // that took "the first N entry records" would take the records of blocks whose occurrences
        // were *not* the rewritten ones — with the total still adding up, which is why the removal
        // is counted per block now (see the module documentation, "dropping those records is a
        // by-block decision") and why the check that the two counts agree is exact. What catches
        // it here is [`audit`], which compares the published records with the uses the published
        // table holds, both halves of the def-use relation.
        let code: &[u8] = &[
            0x04, 0x3b, 0x04, 0x3c, 0x04, 0x3d, 0x1c, 0x04, 0x60, 0x3c, 0x04, 0x99, 0x00, 0x27,
            0x1c, 0x04, 0x60, 0x3c, 0x1c, 0x07, 0x60, 0x3b, 0x04, 0x99, 0x00, 0x03, 0x1a, 0x06,
            0x60, 0x3b, 0x04, 0x99, 0x00, 0x0b, 0x1b, 0x05, 0x60, 0x3c, 0x04, 0x99, 0xff, 0xf3,
            0x1b, 0x07, 0x60, 0x3c, 0x04, 0x99, 0xff, 0xeb, 0x1b, 0x04, 0x60, 0x3c, 0x1c, 0x05,
            0x60, 0x3d, 0xb1,
        ];
        let (body, canonical) = body_of_code(code, 3);
        assert_eq!(
            canonical
                .blocks
                .iter()
                .map(|block| block.id.bci)
                .collect::<Vec<_>>(),
            vec![0, 14, 50, 26, 34, 42],
            "the branches and the back edges name the body's blocks, in the order the \
             normalization created them"
        );
        let table = ssa_of(&body).expect("the body names its values");
        let target = table
            .values()
            .iter()
            .max_by_key(|value| {
                value
                    .uses
                    .iter()
                    .filter(|use_record| use_record.bci.is_none())
                    .count()
            })
            .expect("the body defines values");
        let blocks: BTreeSet<&CanonicalBlockId> = target
            .uses
            .iter()
            .filter(|use_record| use_record.bci.is_none())
            .map(|use_record| &use_record.block)
            .collect();
        assert!(
            blocks.len() >= 2,
            "the body's most-used merge value has its entry records in only {blocks:?}: the shape \
             this case is about is one value named by merge points of more than one block"
        );
        audit("the three-block replacement body", &table);
    }

    #[test]
    fn a_phi_that_only_names_itself_keeps_its_definition() {
        // A cycle whose local is never written by anything: both inputs are the block's own exit,
        // so every operand is the phi itself and there is no value to replace it with.
        let flows = vec![flow(
            0,
            vec![(Slot::Local(0), Value::Int)],
            vec![from_transfer(0), from_transfer(0)],
            vec![],
        )];
        let table = fabricated(flows, BTreeMap::new()).expect("the cycle resolves");
        let phi = phi(&table, &block(0), Slot::Local(0));
        assert_eq!(phi.inputs, vec![PhiInput::Itself, PhiInput::Itself]);
        assert!(
            table.value(phi.value).replaced_by.is_none(),
            "a value no path defines is not replaced by one that no path defines either"
        );
    }

    #[test]
    fn a_slot_its_own_merge_gave_a_class_must_have_a_value_on_some_input() {
        // No input at all: the frames state a class the definition has no value for.
        let flows = vec![flow(0, vec![(Slot::Local(0), Value::Int)], vec![], vec![])];
        let message = fabricated(flows, BTreeMap::new()).expect_err("the slot has no input");
        assert!(
            message.contains("contributes a value to it"),
            "the message names the defect: {message}"
        );
        // An input whose source states no value for the slot either.
        let seed: BTreeMap<Slot, Value> = [(Slot::Local(0), Value::Int)].into_iter().collect();
        let flows = vec![
            flow(0, vec![], vec![from_seed()], vec![]),
            flow(
                8,
                vec![(Slot::Local(0), Value::Int)],
                vec![from_transfer(0)],
                vec![],
            ),
        ];
        let message = fabricated(flows, seed).expect_err("the source states no value");
        assert!(
            message.contains("states no value for it"),
            "a path with no value is not a value: {message}"
        );
    }

    #[test]
    fn the_names_do_not_depend_on_the_order_the_blocks_are_stored_in() {
        // The same graph twice, once with its blocks stored in the order the normalization
        // produced and once reversed: the resolution is driven by the definitions, so the two
        // runs agree up to the numbering of the values they create.
        let code = [
            0x04, // 0: iconst_1
            0x3b, // 1: istore_0
            0x03, // 2: iconst_0
            0x99, 0x00, 0x06, // 3: ifeq 9
            0xa7, 0x00, 0x03, // 6: goto 9
            0x1a, // 9: iload_0
            0x04, // 10: iconst_1
            0x64, // 11: isub
            0x3b, // 12: istore_0
            0x1a, // 13: iload_0
            0x9d, 0xff, 0xfb, // 14: ifgt 9
            0xb1, // 17: return
        ];
        let (body, _) = body_of_code(&code, 1);
        let loader = LoaderId("app".to_string());
        let method = method_view(&body.pool, &loader);
        let seed: BTreeMap<Slot, Value> = match entry_slots(&method, &body.facts) {
            Ok(Some(slots)) => slots
                .into_iter()
                .map(|slot| (Slot::of(slot.region, slot.slot), slot.value))
                .collect(),
            other => panic!("the entry state of the fixture is stated: {other:?}"),
        };
        let flows = match flow_facts(
            &body.facts,
            &body.canonical,
            &body.frames,
            &seed,
            &method,
            &mut budget(),
        ) {
            Ok(flows) => flows,
            Err(error) => panic!("the fixture has a flow: {error:?}"),
        };
        let natural = fabricated(flows.clone(), seed.clone()).expect("the natural order resolves");
        let mut reversed = flows;
        reversed.reverse();
        let shuffled = fabricated(reversed, seed).expect("the reversed order resolves");
        assert_eq!(projection(&natural), projection(&shuffled));
    }

    /// The table as text: every definition and every use by *what* it is, never by which number a
    /// run happened to give it.
    fn projection(table: &SsaTable) -> Vec<String> {
        fn named(value: ValueId, table: &SsaTable) -> String {
            let mut value = value;
            while let Some(next) = table.value(value).replaced_by {
                value = next;
            }
            let found = table.value(value);
            let def = match &found.def {
                Definition::Entry { block, slot } => format!("entry {:?} {slot:?}", block.bci),
                Definition::Instruction { block, bci } => format!("instr {:?} {bci}", block.bci),
                Definition::Phi { block, slot } => format!("phi {:?} {slot:?}", block.bci),
                Definition::Caught { block, bci } => format!("caught {:?} {bci}", block.bci),
            };
            format!("{def} {:?}", found.ty)
        }
        let mut lines = Vec::new();
        for block in table.blocks() {
            lines.push(format!("block {:?}", block.block.bci));
            for (slot, value) in &block.entry {
                lines.push(format!("  entry {slot:?} = {}", named(*value, table)));
            }
            for (slot, value) in &block.exit {
                lines.push(format!("  exit {slot:?} = {}", named(*value, table)));
            }
            for instruction in &block.instructions {
                for (slot, value) in &instruction.reads {
                    lines.push(format!(
                        "  read {} {slot:?} = {}",
                        instruction.bci,
                        named(*value, table)
                    ));
                }
                for (slot, value) in &instruction.writes {
                    lines.push(format!(
                        "  write {} {slot:?} = {}",
                        instruction.bci,
                        named(*value, table)
                    ));
                }
            }
        }
        for phi in table.phis() {
            let inputs: Vec<String> = phi
                .inputs
                .iter()
                .map(|input| match input {
                    PhiInput::Value(value) => named(*value, table),
                    PhiInput::Itself => "self".to_string(),
                })
                .collect();
            lines.push(format!(
                "phi {:?} {:?} -> {} = [{}]",
                phi.block.bci,
                phi.slot,
                named(phi.value, table),
                inputs.join(", ")
            ));
        }
        lines.sort();
        lines
    }

    // -----------------------------------------------------------------------
    // The invariant the table owes every consumer: every use traces to a definition
    // -----------------------------------------------------------------------

    /// Checks one table against itself: every read of an instruction and every operand of a phi
    /// is recorded as a use of exactly that value, and every recorded use is backed by one of
    /// them. This is the bidirectional def-use relation the slice's contract states, checked
    /// against the published table rather than against the code that built it.
    ///
    /// The comparison is a multiset one, so a value that is read twice needs two records, and the
    /// names come from the published table itself: a reference that survives into it must name a
    /// definition, so nothing here follows a replacement.
    fn audit(what: &str, table: &SsaTable) {
        let mut expected: BTreeMap<ValueId, Vec<SsaUse>> = BTreeMap::new();
        for block in table.blocks() {
            for instruction in &block.instructions {
                for (_, value) in &instruction.reads {
                    expected.entry(*value).or_default().push(SsaUse {
                        block: block.block.clone(),
                        bci: Some(instruction.bci),
                    });
                }
            }
        }
        for phi in table.phis() {
            for input in &phi.inputs {
                let PhiInput::Value(value) = input else {
                    continue;
                };
                expected.entry(*value).or_default().push(SsaUse {
                    block: phi.block.clone(),
                    bci: None,
                });
            }
        }
        for (id, value) in table.values().iter().enumerate() {
            let id = ValueId(u32::try_from(id).expect("a small table"));
            let mut recorded = value.uses.clone();
            recorded.sort();
            let mut wanted = expected.remove(&id).unwrap_or_default();
            wanted.sort();
            assert_eq!(
                recorded, wanted,
                "{what}: value {id:?} ({:?}) records exactly the uses the table holds",
                value.def
            );
        }
        assert!(
            expected.is_empty(),
            "every use names a value of the table: {expected:?}"
        );
    }

    #[test]
    fn every_use_of_a_real_body_traces_back_to_its_definition() {
        let (straight, _) = body_of_code(&[0x04, 0x3b, 0x1a, 0x3c, 0xb1], 2);
        let (diamond, _) = body_of_code(
            &[
                0x03, 0x99, 0x00, 0x07, 0x04, 0xa7, 0x00, 0x04, 0x05, 0x3b, 0xb1,
            ],
            1,
        );
        let (longs, _) = body_of_code(&[0x09, 0x3f, 0x1e, 0x41, 0xb1], 4);
        let exception = class_with_handlers(
            &[
                0x04, 0x3c, 0x04, 0x03, 0x6c, 0x05, 0x3c, 0x04, 0x03, 0x6c, 0x57, 0x57, 0xb1, 0x1b,
                0x57, 0x57, 0xb1,
            ],
            3,
            2,
            &[(0, 13, 13, 0)],
        );
        let (handled, _) = body_of(&exception);
        for (what, body) in [
            ("a straight line", &straight),
            ("a diamond", &diamond),
            ("category-2 values", &longs),
            ("two throw sites", &handled),
        ] {
            let table = ssa_of(body).unwrap_or_else(|message| panic!("{what} analyzes: {message}"));
            audit(what, &table);
            assert!(
                !table.values().is_empty(),
                "{what}: the body defines at least one value"
            );
            // Every value is created once and carries exactly one definition; a replaced phi is
            // the one value that stops being a definition, and its definition records that.
            for value in table.values() {
                if value.replaced_by.is_some() {
                    assert!(
                        value.uses.is_empty(),
                        "{what}: a replaced value has no uses left"
                    );
                }
            }
        }
    }

    #[test]
    fn def_use_records_are_the_reads_and_phi_operands_over_four_bodies() {
        // The same invariant over the shapes a consumer meets: a straight line needs no merge at
        // all, and a diamond, a loop and a category-2 merge must each publish reads and phi
        // operands whose occurrences are exactly the recorded uses.
        // 0 iconst_1, 1 istore_0, 2 iload_0, 3 istore_1, 4 return.
        let (body, _) = body_of_code(&[0x04, 0x3b, 0x1a, 0x3c, 0xb1], 4);
        audit(
            "a straight line",
            &ssa_of(&body).expect("straight analyzes"),
        );

        // 0 iconst_0, 1 istore_0, 2 iload_0, 3 ifne +6 (to 9), 6 iconst_1, 7 istore_0,
        // 8 nop, 9 iconst_2, 10 istore_1, 11 iload_0, 12 istore_2, 13 return.
        let (body, _) = body_of_code(
            &[
                0x03, 0x3b, 0x1a, 0x9a, 0x00, 0x06, 0x04, 0x3b, 0x00, 0x05, 0x3c, 0x1a, 0x3c, 0xb1,
            ],
            4,
        );
        let table = ssa_of(&body).expect("the diamond analyzes");
        let diamond = phi(&table, &block(9), Slot::Local(0));
        assert_eq!(
            diamond.inputs.len(),
            2,
            "the diamond's merge takes one operand per path: {:#?}",
            diamond
        );
        assert_eq!(
            table.block(&block(9)).expect("named").instructions[2].reads,
            vec![(Slot::Local(0), diamond.value)],
            "the load after the merge reads the phi"
        );
        audit("a diamond", &table);

        // 0 iconst_0, 1 istore_0, 2 iload_0, 3 iconst_3, 4 if_icmpge +9,
        // 7 iinc 0,1, 10 goto -8, 13 return.
        let (body, _) = body_of_code(
            &[
                0x03, 0x3b, 0x1a, 0x06, 0xa2, 0x00, 0x09, 0x84, 0x00, 0x01, 0xa7, 0xff, 0xf8, 0xb1,
            ],
            4,
        );
        audit("a loop", &ssa_of(&body).expect("the loop analyzes"));

        // 0 lconst_0, 1 lstore_0, 2 iconst_0, 3 ifne +6 (to 9), 6 lconst_1, 7 lstore_0,
        // 8 nop, 9 lconst_1, 10 lstore_2, 11 lload_0, 12 pop2, 13 return.
        let (body, _) = body_of_code(
            &[
                0x09, 0x3f, 0x03, 0x9a, 0x00, 0x06, 0x0a, 0x3f, 0x00, 0x0a, 0x41, 0x1e, 0x58, 0xb1,
            ],
            8,
        );
        let table = ssa_of(&body).expect("the long merge analyzes");
        let merged = phi(&table, &block(9), Slot::Local(0));
        assert_eq!(
            merged.inputs.len(),
            2,
            "two paths, one category-2 value: {:#?}",
            merged
        );
        assert_eq!(table.value(merged.value).ty, Value::Long);
        assert!(
            table.phis().iter().all(|phi| phi.slot != Slot::Local(1)),
            "the upper half of a category-2 pair is never a phi of its own: {:#?}",
            table.phis()
        );
        assert_eq!(
            table.block(&block(9)).expect("named").instructions[2].reads,
            vec![(Slot::Local(0), merged.value)],
            "the load reads the merged value at the pair's first slot"
        );
        audit("category-2 values", &table);
    }

    #[test]
    fn def_use_over_a_mixed_transfer_and_exception_input() {
        // One handler entered both by a plain transfer and by one throw site of the same source.
        // The site snapshot is taken **before** the throwing instruction, and the transfer hands
        // the source's exit: the two operands differ, so the phi is not trivial and its arity
        // says both inputs took part.
        let site = SiteFlow {
            handler_ordinals: vec![0],
            slots: vec![Slot::Local(0)],
            origin: OriginSet::default(),
        };
        let flows = vec![
            flow(
                0,
                vec![(Slot::Local(0), Value::Int)],
                vec![from_seed()],
                vec![
                    InstructionFlow {
                        bci: 1,
                        opcode: 0x00,
                        stack_after: 0,
                        accesses: Vec::new(),
                        site: Some(site),
                    },
                    instruction(2, vec![wrote(0, Value::Int)]),
                ],
            ),
            flow(
                8,
                vec![(Slot::Local(0), Value::Int)],
                vec![
                    from_transfer(0),
                    FlowInput {
                        kind: InputKind::Exception { handler_ordinal: 0 },
                        from: Some(block(0)),
                        throw_site: Some(1),
                    },
                ],
                vec![],
            ),
        ];
        let seed: BTreeMap<Slot, Value> = [(Slot::Local(0), Value::Int)].into_iter().collect();
        let table = fabricated(flows, seed.clone()).expect("the mixed merge resolves");
        let mixed = phi(&table, &block(8), Slot::Local(0));
        assert_eq!(
            mixed.inputs.len(),
            2,
            "the transfer and the throw site are two participants: {:#?}",
            mixed
        );
        let entry = table.block(&block(0)).expect("the entry is named").entry[0].1;
        let written = table.block(&block(0)).expect("named").instructions[1].writes[0].1;
        let mut operands: Vec<ValueId> = mixed
            .inputs
            .iter()
            .map(|input| match input {
                PhiInput::Value(value) => *value,
                PhiInput::Itself => panic!("neither input is the phi itself"),
            })
            .collect();
        operands.sort();
        let mut wanted = vec![entry, written];
        wanted.sort();
        assert_eq!(
            operands, wanted,
            "the exception operand is the pre-instruction state and the transfer is the exit"
        );
        audit("mixed transfer and exception inputs", &table);

        // The same shape with the source's exit equal to the site's snapshot: both operands are
        // the one value, so the phi is trivial and the check runs over a *replaced* value too.
        let site = SiteFlow {
            handler_ordinals: vec![0],
            slots: vec![Slot::Local(0)],
            origin: OriginSet::default(),
        };
        let flows = vec![
            flow(
                0,
                vec![(Slot::Local(0), Value::Int)],
                vec![from_seed()],
                vec![InstructionFlow {
                    bci: 1,
                    opcode: 0x00,
                    stack_after: 0,
                    accesses: Vec::new(),
                    site: Some(site),
                }],
            ),
            flow(
                8,
                vec![(Slot::Local(0), Value::Int)],
                vec![
                    from_transfer(0),
                    FlowInput {
                        kind: InputKind::Exception { handler_ordinal: 0 },
                        from: Some(block(0)),
                        throw_site: Some(1),
                    },
                ],
                vec![],
            ),
        ];
        let table = fabricated(flows, seed.clone()).expect("the equal-operand merge resolves");
        let trivial = phi(&table, &block(8), Slot::Local(0));
        let only = table.block(&block(0)).expect("named").entry[0].1;
        assert_eq!(
            table.value(trivial.value).replaced_by,
            Some(only),
            "one distinct operand: the phi is that operand's name"
        );
        assert!(
            table.value(trivial.value).uses.is_empty(),
            "a replaced value keeps no use"
        );
        assert_eq!(
            table.value(only).uses.len(),
            2,
            "one def-use edge per operand"
        );
        audit("mixed transfer and exception inputs, trivial phi", &table);
    }

    #[test]
    fn def_use_holds_under_every_rotation_of_a_loop_body() {
        // The same loop body as the existing order test, stored in every rotation of its block
        // order: a naive walk would name the back edge's use before its definition in most of
        // them.
        let code = [
            0x04, // 0: iconst_1
            0x3b, // 1: istore_0
            0x03, // 2: iconst_0
            0x99, 0x00, 0x06, // 3: ifeq 9
            0xa7, 0x00, 0x03, // 6: goto 9
            0x1a, // 9: iload_0
            0x04, // 10: iconst_1
            0x64, // 11: isub
            0x3b, // 12: istore_0
            0x1a, // 13: iload_0
            0x9d, 0xff, 0xfb, // 14: ifgt 9
            0xb1, // 17: return
        ];
        let (body, _) = body_of_code(&code, 1);
        let loader = LoaderId("app".to_string());
        let method = method_view(&body.pool, &loader);
        let seed: BTreeMap<Slot, Value> = match entry_slots(&method, &body.facts) {
            Ok(Some(slots)) => slots
                .into_iter()
                .map(|slot| (Slot::of(slot.region, slot.slot), slot.value))
                .collect(),
            other => panic!("the entry state of the fixture is stated: {other:?}"),
        };
        let flows = match flow_facts(
            &body.facts,
            &body.canonical,
            &body.frames,
            &seed,
            &method,
            &mut budget(),
        ) {
            Ok(flows) => flows,
            Err(error) => panic!("the fixture has a flow: {error:?}"),
        };
        let natural = fabricated(flows.clone(), seed.clone()).expect("the natural order resolves");
        assert!(flows.len() > 2, "the fixture has several blocks");
        for shift in 1..flows.len() {
            let mut rotated = flows.clone();
            rotated.rotate_left(shift);
            let table = fabricated(rotated, seed.clone()).expect("a rotation resolves");
            assert_eq!(
                projection(&natural),
                projection(&table),
                "the names of rotation {shift} are the natural ones"
            );
            audit("a rotated loop", &table);
        }
    }
}
