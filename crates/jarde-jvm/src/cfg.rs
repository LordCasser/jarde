//! The raw control-flow graph of one decoded method body (3.3).
//!
//! The graph is *raw* in a precise sense, and every part of that sense is a fact this module
//! records rather than a limitation it hides:
//!
//! * the blocks are the basic blocks of the **decoded instruction stream**. When the reader
//!   stopped early (`MethodCodeFacts::stopped_at`), the graph covers the reliable prefix and
//!   says so ([`CfgCompleteness::Truncated`]) instead of pretending the body ended there —
//!   the sound-but-incomplete view 1.2 registered as a debt;
//! * the edges are the transfers the instructions themselves encode ([`EdgeKind::Normal`]),
//!   the exception table's own records ([`EdgeKind::Exception`]) and the call into a
//!   subroutine ([`EdgeKind::SubroutineReturn`]). `ret` has **no successor here**: its
//!   return point belongs to the call contexts 3.4 builds and the canonical graph 3.5
//!   rebuilds, and the raw graph names the unresolved call sites
//!   ([`RawCfg::unresolved_returns`]) instead of guessing one;
//! * throw sites are recorded **per throwing instruction**, not per block: the block-level
//!   exception edge is only the set of handlers the block's own instructions can enter,
//!   while 4.x needs the locals and the effect order *at the throw site*;
//! * no handler is filtered by its catch type. Deciding "this handler cannot catch that
//!   throw" needs the type hierarchy, which is the resolver's, and `cfg` does not depend on
//!   it: the handler list of a throw site is the set of records whose protected range covers
//!   the instruction, in exception-table declaration order.
//!
//! # Determinism
//!
//! Every published `Vec<_>` is sorted on its own coordinates — blocks by BCI, edges by
//! `(from, kind, to)`, throw sites and effects by BCI, handlers by ordinal, unreachable
//! blocks by BCI — and no published order is petgraph's. The graph is a plain
//! [`DiGraph`] because the raw CFG is a multigraph: two exception records may name the same
//! handler block ([`EdgeKind::Exception`] keeps their ordinals apart) and a `goto` to itself
//! or a handler covering its own block are ordinary self-loops. The reachability walk over
//! that graph only produces a *set* of visited blocks, which the published list is derived
//! from by sorting.
//!
//! # Scale and billing
//!
//! The pass bills exactly the dimensions its descriptor declares (`Blocks` = `IrItems` +
//! `IrEdges`, `Steps` = `AnalysisSteps`), always **before** the item exists:
//!
//! * `IrItems` — one per block, one per handler record, one per throw site and one per
//!   instruction effect;
//! * `IrEdges` — one per edge, before it is added to the graph;
//! * `AnalysisSteps` — one per instruction placed into a block, and one per reachability
//!   worklist pop (a repeated visit is real work and is charged again).
//!
//! The graph also has a scale ceiling of its own, because the phases that consume it (SCC,
//! dominance) run inside petgraph with no interruption hook (3.1): at most
//! [`MAX_BLOCKS_DEFAULT`] blocks, and [`MAX_BLOCKS_HARD_LIMIT`] is the structural maximum
//! (`code_length` is an array of at most `u16::MAX` bytes, so a body cannot hold more
//! instruction starts than that). The ceiling is enforced through the `Blocks` dimension's
//! `IrItems`, so it stops as a budget refusal of the dimension that paid for the blocks, with
//! the effective ceiling as its `limit`.
//!
//! # Opcodes
//!
//! Every opcode decision of this module — what ends a block, which transfer an instruction
//! is, whether it may throw, what its stack delta is, and which local it reads or writes —
//! reads [`InstructionOperands::effective_opcode`], never the raw
//! [`InstructionFact::opcode`](jarde_reader::classfile::InstructionFact::opcode). A `wide` form is
//! the opcode it wraps (0.2): `wide iload` is a load and `wide ret` ends its block exactly
//! like the short forms, while the raw `0xc4` prefix decides nothing on its own.
//!
//! # Known boundaries
//!
//! * The handler list of a throw site is structural. A caller that needs "which handler
//!   really catches this" has to resolve the thrown type against the catch types.
//!
//! # Payload visibility
//!
//! The graph and the effect facts are crate-private IR payloads (invariant 11): the report
//! publishes the pass's status planes — stage states, coverage, execution, diagnostics — and
//! 5.1 is where a published IR count would have to add its own type first. Their readers are
//! the following slices (3.4 builds the call contexts from the graph and its unresolved
//! returns, 3.5 rebuilds the edges, 4.x reads the effects) and this module's own unit tests,
//! which is why the payload fields carry `allow(dead_code)` rather than a second, decorative
//! consumer in the engine.

use jarde_reader::budget::{Budget, BudgetDimension, CountedBudgetDimension};
use jarde_reader::classfile::{
    BytecodeStop, ControlFlowTarget, ControlFlowTargetKind, ExceptionHandlerFact, MethodCodeFacts,
};
use jarde_reader::error::{Error, Result};
use petgraph::graph::{DiGraph, NodeIndex};

/// Blocks one raw CFG may hold before the pass stops (3.1's scale bound).
pub(crate) const MAX_BLOCKS_DEFAULT: usize = 16_384;

/// Structural maximum of the same bound: `code_length` cannot exceed `u16::MAX` bytes and a
/// block needs at least one instruction byte.
pub(crate) const MAX_BLOCKS_HARD_LIMIT: u32 = 65_535;

// The default ceiling only means something below the structural maximum: `partition` stops at
// `MAX_BLOCKS_DEFAULT` before it could ever reach `MAX_BLOCKS_HARD_LIMIT`, so a default raised
// past the hard limit would leave the guardrail to a `debug_assert!` that release builds do not
// run. The relation is therefore rejected at compile time, in every profile.
const _: () = assert!(MAX_BLOCKS_DEFAULT as u32 <= MAX_BLOCKS_HARD_LIMIT);

/// Code of a raw CFG whose target is not a block of the same graph.
///
/// Unreachable in practice — `MethodCodeFacts::control_flow_targets` validates every target
/// against the instruction starts the same facts carry — but the graph must not silently drop
/// an edge it cannot place, so it is a structured error rather than an `unwrap`.
const IR_RAW_CFG_UNKNOWN_TARGET: &str = "ir_raw_cfg_unknown_target";

const OPCODE_GOTO: u8 = 0xa7;
/// `jsr`: a call into a legacy subroutine (also read by 3.4's call contexts).
pub(crate) const OPCODE_JSR: u8 = 0xa8;
/// `ret`: the return from a legacy subroutine (also read by 3.4's call contexts).
pub(crate) const OPCODE_RET: u8 = 0xa9;
const OPCODE_TABLESWITCH: u8 = 0xaa;
const OPCODE_LOOKUPSWITCH: u8 = 0xab;
const OPCODE_ATHROW: u8 = 0xbf;
const OPCODE_MULTIANEWARRAY: u8 = 0xc5;
const OPCODE_IFNULL: u8 = 0xc6;
const OPCODE_IFNONNULL: u8 = 0xc7;
const OPCODE_GOTO_W: u8 = 0xc8;
/// `jsr_w`: the wide form of the same call (also read by 3.4's call contexts).
pub(crate) const OPCODE_JSR_W: u8 = 0xc9;

/// Kind of one raw edge.
///
/// The declaration order is also the order the edge sort uses for kinds (`Normal` before
/// `Exception` before `SubroutineReturn`), so the published order is a total order over
/// `(from, kind, to)` — two edges of one block can only tie when they are the same edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum EdgeKind {
    /// A fall-through, a conditional branch, a `goto` or a switch target.
    Normal,
    /// The exception edge of one exception-table record, in **declaration order**.
    ///
    /// The ordinal is part of the edge: two records may name the same handler entry for
    /// different protected ranges, and both edges are kept — the raw CFG is a multigraph, and
    /// collapsing the ordinals would lose the handler order 4.x has to respect.
    Exception { handler_ordinal: u32 },
    /// The transfer into a `jsr`/`jsr_w` subroutine, labelled by its call site.
    ///
    /// Raw semantics only: the *return* half of the transfer is what 3.4's call contexts
    /// decide, and `ret` has no outgoing edge here (see [`RawCfg::unresolved_returns`]).
    SubroutineReturn { call_site: u32 },
}

/// One raw basic block: the half-open BCI range `bci..end_bci` of the decoded instruction
/// prefix.
///
/// The instructions themselves stay in the reader's facts, which are ordered by BCI, so a
/// block names its range instead of copying them. `blocks[0]` is the entry block (BCI 0)
/// whenever the body holds any instruction at all.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RawBlock {
    /// BCI of the block's first instruction: a block starts where a transfer can land.
    pub(crate) bci: u32,
    /// BCI just past the block's last instruction: the next block's start, or the end of the
    /// decoded prefix for the last block.
    pub(crate) end_bci: u32,
}

/// One edge of the raw graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RawEdge {
    /// BCI of the source block's first instruction.
    pub(crate) from_bci: u32,
    /// BCI of the target block's first instruction.
    pub(crate) to_bci: u32,
    pub(crate) kind: EdgeKind,
}

/// One instruction that may throw, with the handlers its own BCI can enter.
///
/// The handlers are exception-table ordinals in declaration order. An empty list is a fact
/// too: this instruction lies inside no protected range, so an exception it raises leaves the
/// method instead of entering a handler of it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ThrowSite {
    /// BCI of the throwing instruction.
    pub(crate) bci: u32,
    /// Its opcode, so a consumer can tell an `athrow` from an `invokevirtual` without
    /// re-deriving it from the instruction list.
    pub(crate) opcode: u8,
    /// BCI of the block that holds it.
    pub(crate) block_bci: u32,
    /// Feasible handlers, in exception-table declaration order.
    pub(crate) handlers: Vec<u32>,
}

/// One exception-table record of the raw graph, in declaration order.
///
/// This is the reader's own record ([`ExceptionHandlerFact`]) rather than a second struct: the
/// raw CFG publishes the protected ranges and handler entries the reader validated (1.2), and
/// a copy could drift from the bytes it describes.
pub(crate) type HandlerFact = ExceptionHandlerFact;

/// How complete the graph is with respect to the method body it describes.
///
/// Published through the read-only handoff ([`crate::method_ir`]) beside the canonical graph it
/// belongs to: a consumer that decides between a structured result and a fallback has to know
/// whether the graph covers the whole body or its reliable decoded prefix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CfgCompleteness {
    /// The reader decoded the whole `Code` attribute, so the blocks cover the whole body.
    Complete,
    /// The reader stopped before the end of the body: the graph covers the reliable decoded
    /// prefix and the coverage plane names the rest. The graph states which stop produced it
    /// rather than leaving a caller to guess from the instruction list.
    Truncated { stopped_at: BytecodeStop },
}

impl CfgCompleteness {
    /// Whether the graph covers the whole method body.
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }
}

/// The raw CFG of one method body.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawCfg {
    /// Blocks by BCI ascending; `blocks[0]` is the entry block when the body has bytes.
    pub(crate) blocks: Vec<RawBlock>,
    /// Edges by `(from, kind, to)`.
    pub(crate) edges: Vec<RawEdge>,
    /// Throw sites by BCI: one per throwing instruction, not one per block.
    pub(crate) throw_sites: Vec<ThrowSite>,
    /// The exception table as declared, in declaration (ordinal) order.
    pub(crate) handlers: Vec<HandlerFact>,
    /// Blocks the entry cannot reach, as a truth table: the block is listed, not dropped.
    ///
    /// Reachability follows the raw transfer edges, and the continuation of a reachable `jsr`
    /// counts as reachable too: a `ret` returns there, and the raw graph has no edge for that
    /// half of the transfer. The list therefore under-approximates while
    /// [`Self::unresolved_returns`] is not empty instead of claiming a block is dead.
    pub(crate) unreachable: Vec<u32>,
    /// Whether the blocks cover the whole body; see [`CfgCompleteness`].
    pub(crate) completeness: CfgCompleteness,
    /// Call sites of `jsr`/`jsr_w` whose return address the raw graph does not resolve.
    ///
    /// 3.4 turns these into call contexts (call site, return BCI, entry BCI, affected locals)
    /// and 3.5 rebuilds the edges from them. Until then this list is what a consumer reads
    /// next to [`Self::unreachable`] and next to the missing `ret` successors.
    pub(crate) unresolved_returns: Vec<u32>,
}

/// Per-instruction effect facts of one method body, in BCI order.
///
/// This is the `Effects` fact of the raw pass: what an instruction *does* to the frame it runs
/// in. It is derived from the reader's typed operands (1.2) and from the opcode alone, never
/// from rendered text.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectFacts {
    pub(crate) instructions: Vec<InstructionEffect>,
}

/// What one instruction reads, writes, pops/pushes and may raise.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InstructionEffect {
    pub(crate) bci: u32,
    /// The instruction's effective opcode
    /// ([`jarde_reader::classfile::InstructionOperands::effective_opcode`]): the opcode a `wide` form
    /// wraps, and the raw opcode otherwise. The raw `0xc4` prefix stays in the reader's own fact.
    pub(crate) opcode: u8,
    /// Locals this instruction reads, ascending, deduplicated: the operand 1.2 records for a
    /// load, `iinc` or `ret`, including the forms whose index is encoded in the opcode.
    ///
    /// A `wide`-prefixed access is classified by the opcode it wraps, so it is listed exactly
    /// like the short form of that opcode.
    pub(crate) locals_read: Vec<u16>,
    /// Locals this instruction writes, ascending, deduplicated.
    pub(crate) locals_written: Vec<u16>,
    /// Change of the operand-stack depth, or `None` when the opcode alone does not decide it.
    ///
    /// The `None` cases are exactly the ones that need more than the opcode: an `invoke*`, a
    /// field access or an `ldc` needs the descriptor or the constant-pool tag the descriptor
    /// table of 4.1 owns, and `athrow` clears the stack (a data-flow property, not a constant).
    /// `multianewarray`'s delta is `1 - dimensions` and is decided here from the reader's
    /// `dimensions` fact (D41); a `wide` form *is* decided here too, through the opcode it wraps.
    pub(crate) stack_delta: Option<i32>,
    /// Whether this instruction may raise an exception.
    pub(crate) may_throw: bool,
    /// Feasible handlers at this instruction, in declaration order; empty when the instruction
    /// may throw but no protected range covers it.
    pub(crate) handlers: Vec<u32>,
}

/// The two facts one raw-CFG pass publishes: the graph and its effects.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawCfgOutcome {
    pub(crate) cfg: RawCfg,
    pub(crate) effects: EffectFacts,
}

/// Builds the raw CFG and the effect facts of one decoded method body.
///
/// The graph is built over `facts.control_flow_targets()`, so every branch, switch case and
/// handler entry is a **validated** instruction start before it becomes a block, and a target
/// the reader refuses is a structured error here too. A body whose decode stopped early is
/// still built — over its reliable prefix — and marked [`CfgCompleteness::Truncated`]; a
/// caller that sees the target validation fail on such a body must not report corruption
/// (1.2's rule), and `analyze_method` reports the reader's own stop instead.
///
/// Charges, in order: one `AnalysisSteps` and one `IrItems` per block (with the block ceiling
/// checked before the block is created), one `IrEdges` per edge before it is added to the
/// graph, one `IrItems` per handler record, throw site and instruction effect, and one
/// `AnalysisSteps` per reachability pop.
pub(crate) fn raw_cfg(facts: &MethodCodeFacts, budget: &mut Budget) -> Result<RawCfgOutcome> {
    debug_assert_eq!(
        facts.instructions.len(),
        facts.operands().len(),
        "operand facts are produced in lockstep with instructions"
    );
    let targets = facts.control_flow_targets()?;
    let leaders = leader_flags(facts, &targets)?;
    let (blocks, ranges) = partition(facts, &leaders, budget)?;
    debug_assert!(
        blocks.first().is_none_or(|block| block.bci == 0),
        "the entry block starts at BCI 0"
    );
    let per_instruction = InstructionTargets::build(facts, &targets)?;

    let mut edges = Vec::new();
    transfer_edges(
        facts,
        &blocks,
        &ranges,
        &per_instruction,
        &mut edges,
        budget,
    )?;
    let (handlers, mut throw_sites, exception_edges) =
        throw_sites_and_handlers(facts, &blocks, &ranges, budget)?;
    edges.extend(exception_edges);
    // The published order is the contract's `(from, kind, to, ordinal)`. The kind carries the
    // ordinal, and two edges of one block can only tie when they are the same edge: a normal
    // edge is deduplicated by target and an exception edge by record.
    edges.sort_unstable_by_key(|edge| (edge.from_bci, edge.kind, edge.to_bci));
    throw_sites.sort_unstable_by_key(|site| (site.bci, site.block_bci));

    // The graph holds exactly the published edges, in the published order; every one of them
    // was charged when its item was created.
    let mut graph: DiGraph<u32, EdgeKind> = DiGraph::with_capacity(blocks.len(), edges.len());
    let nodes: Vec<NodeIndex> = blocks
        .iter()
        .map(|block| graph.add_node(block.bci))
        .collect();
    for edge in &edges {
        let from = block_position(&blocks, edge.from_bci)?;
        let to = block_position(&blocks, edge.to_bci)?;
        graph.add_edge(nodes[from], nodes[to], edge.kind);
    }

    let continuations = jsr_continuations(facts, &blocks);
    let unreachable = unreachable_blocks(&graph, &nodes, &continuations, budget)?;
    let effects = effect_facts(facts, &throw_sites, budget)?;

    Ok(RawCfgOutcome {
        cfg: RawCfg {
            blocks,
            edges,
            throw_sites,
            handlers,
            unreachable,
            completeness: completeness_of(facts),
            unresolved_returns: jsr_call_sites(facts)?,
        },
        effects,
    })
}

/// Whether the graph covers the whole body or only the reader's reliable prefix.
pub(crate) fn completeness_of(facts: &MethodCodeFacts) -> CfgCompleteness {
    match &facts.stopped_at {
        Some(stopped_at) => CfgCompleteness::Truncated {
            stopped_at: stopped_at.clone(),
        },
        None => CfgCompleteness::Complete,
    }
}

/// The block starts the instructions and the validated targets force.
fn leader_flags(facts: &MethodCodeFacts, targets: &[ControlFlowTarget]) -> Result<Vec<bool>> {
    let mut leaders = vec![false; facts.instructions.len()];
    if let Some(first) = leaders.first_mut() {
        // BCI 0 is the entry of the body: the first block is the entry block.
        *first = true;
    }
    for (index, (instruction, operands)) in facts
        .instructions
        .iter()
        .zip(facts.operands().iter())
        .enumerate()
    {
        // A transfer cannot fall through from inside a block, so the instruction after a
        // branch, a `goto`, a `jsr`, a `ret`, a switch, a return or an `athrow` starts one.
        // A `jsr` is included on purpose: control returns to the instruction after it through
        // the subroutine's `ret`, so it is a join point even though the raw graph has no edge
        // into it yet. The instruction list is contiguous (the reader validated the widths),
        // so the instruction after a block ender is exactly the one at `bci + width`.
        if ends_block(operands.effective_opcode)
            && let Some(next) = leaders.get_mut(index + 1)
        {
            debug_assert_eq!(
                facts.instructions[index + 1].bci,
                instruction.bci + instruction.width,
                "the decoded instruction list is contiguous"
            );
            *next = true;
        }
    }
    for target in targets {
        leaders[instruction_index(facts, target.target_bci)?] = true;
    }
    Ok(leaders)
}

/// The blocks of one body and the instruction range of each, as `(start, end)` indexes into
/// the reader's instruction list.
type Partitioned = (Vec<RawBlock>, Vec<(usize, usize)>);

/// Partitions the decoded prefix into blocks, billing one step per instruction and one
/// `IrItems` per block.
fn partition(
    facts: &MethodCodeFacts,
    leaders: &[bool],
    budget: &mut Budget,
) -> Result<Partitioned> {
    let mut blocks: Vec<RawBlock> = Vec::new();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut open: Option<usize> = None;
    for (index, instruction) in facts.instructions.iter().enumerate() {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !leaders[index] {
            continue;
        }
        if let Some(start) = open {
            let block = blocks
                .last_mut()
                .expect("a block is open exactly while it is the last one");
            block.end_bci = instruction.bci;
            ranges.push((start, index));
        }
        // A body cannot hold more blocks than the structural maximum: every block needs at
        // least one instruction byte and `code_length` cannot exceed `u16::MAX`.
        debug_assert!(blocks.len() < MAX_BLOCKS_HARD_LIMIT as usize);
        if blocks.len() >= MAX_BLOCKS_DEFAULT {
            // 3.1's scale bound, charged in the dimension that paid for the blocks: the
            // effective limit of this dimension is the ceiling, which may be lower than the
            // caller's own `ir_items` limit.
            return Err(Error::BudgetExceeded {
                dimension: BudgetDimension::IrItems,
                limit: MAX_BLOCKS_DEFAULT as u64,
                consumed: blocks.len() as u64,
                requested: 1,
            });
        }
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        blocks.push(RawBlock {
            bci: instruction.bci,
            end_bci: instruction.bci,
        });
        open = Some(index);
    }
    if let Some(start) = open {
        let block = blocks
            .last_mut()
            .expect("a block is open exactly while it is the last one");
        block.end_bci = prefix_end(facts)?;
        ranges.push((start, facts.instructions.len()));
    }
    Ok((blocks, ranges))
}

/// The BCI just past the last decoded instruction.
fn prefix_end(facts: &MethodCodeFacts) -> Result<u32> {
    facts.instructions.last().map_or(Ok(0), |instruction| {
        instruction
            .bci
            .checked_add(instruction.width)
            .ok_or_else(|| {
                Error::invalid_input(
                    "classfile_instruction_width_overflow",
                    "instruction end overflow",
                )
            })
    })
}

/// Per-instruction transfer targets, indexed the way the instruction list is.
struct InstructionTargets {
    /// Target of a `Branch` row: a conditional branch, a `goto` or a `jsr`.
    branch: Vec<Option<u32>>,
    /// Default target of a switch.
    switch_default: Vec<Option<u32>>,
    /// Case targets of a switch, in payload order.
    switch_cases: Vec<Vec<u32>>,
}

impl InstructionTargets {
    fn build(facts: &MethodCodeFacts, targets: &[ControlFlowTarget]) -> Result<Self> {
        let mut branch = vec![None; facts.instructions.len()];
        let mut switch_default = vec![None; facts.instructions.len()];
        let mut switch_cases: Vec<Vec<u32>> = vec![Vec::new(); facts.instructions.len()];
        for target in targets {
            let index = instruction_index(facts, target.instruction_bci)?;
            match target.kind {
                ControlFlowTargetKind::Branch { .. } => branch[index] = Some(target.target_bci),
                ControlFlowTargetKind::SwitchDefault => {
                    switch_default[index] = Some(target.target_bci);
                }
                ControlFlowTargetKind::SwitchCase { .. } => {
                    switch_cases[index].push(target.target_bci);
                }
                // A handler record is not the transfer of an instruction: it becomes an
                // exception edge of every throw site inside its protected range.
                ControlFlowTargetKind::Handler { .. } => {}
            }
        }
        Ok(Self {
            branch,
            switch_default,
            switch_cases,
        })
    }
}

/// The `Normal` and `SubroutineReturn` edges every block's own transfer encodes.
fn transfer_edges(
    facts: &MethodCodeFacts,
    blocks: &[RawBlock],
    ranges: &[(usize, usize)],
    targets: &InstructionTargets,
    edges: &mut Vec<RawEdge>,
    budget: &mut Budget,
) -> Result<()> {
    for (block_index, (_, end)) in ranges.iter().copied().enumerate() {
        let block = blocks[block_index];
        let last = end - 1;
        // The instruction the block really ends with: a `wide` form is classified by the
        // opcode it wraps, so `wide ret` leaves the block exactly like `ret` does.
        let opcode = facts.operands()[last].effective_opcode;
        // The instruction a block falls through to, when the block is not the last one: a
        // block only ends where a leader starts, so that instruction starts the next block.
        let fall_through = facts
            .instructions
            .get(end)
            .map(|instruction| instruction.bci);
        if is_jsr(opcode) {
            // The call into the subroutine. Its return address belongs to 3.4/3.5, so the raw
            // graph has no edge back to the continuation.
            let target = targets.branch[last].ok_or_else(|| missing_target("jsr", block.bci))?;
            push_edge(
                edges,
                block.bci,
                target,
                EdgeKind::SubroutineReturn {
                    call_site: facts.instructions[last].bci,
                },
                budget,
            )?;
            continue;
        }
        if opcode == OPCODE_TABLESWITCH || opcode == OPCODE_LOOKUPSWITCH {
            let Some(default) = targets.switch_default[last] else {
                return Err(missing_target("switch", block.bci));
            };
            // One edge per **distinct** target: a switch whose cases share a label is one
            // transfer, and the label list itself stays the reader's fact, not this graph's.
            let mut distinct: Vec<u32> = vec![default];
            for target in &targets.switch_cases[last] {
                if !distinct.contains(target) {
                    distinct.push(*target);
                }
            }
            for target in distinct {
                push_edge(edges, block.bci, target, EdgeKind::Normal, budget)?;
            }
            continue;
        }
        if is_conditional_branch(opcode) {
            let target = targets.branch[last].ok_or_else(|| missing_target("branch", block.bci))?;
            push_edge(edges, block.bci, target, EdgeKind::Normal, budget)?;
            // A branch to its own fall-through (`ifeq +3` targets the instruction right after
            // it) has one successor, not two: the published order is a total order over
            // `(from, kind, to)`, so the same rule the switch arm applies to its distinct
            // labels applies to the target and the fall-through as well.
            if let Some(next) = fall_through
                && next != target
            {
                push_edge(edges, block.bci, next, EdgeKind::Normal, budget)?;
            }
            continue;
        }
        if opcode == OPCODE_GOTO || opcode == OPCODE_GOTO_W {
            let target = targets.branch[last].ok_or_else(|| missing_target("goto", block.bci))?;
            push_edge(edges, block.bci, target, EdgeKind::Normal, budget)?;
            continue;
        }
        if is_transfer_stop(opcode) {
            // A return, an `athrow` or a `ret`: control leaves the block without a raw edge.
            continue;
        }
        // Every other instruction falls through to the next block, which exists exactly when
        // this block is not the last one of the decoded prefix.
        if let Some(next) = fall_through {
            push_edge(edges, block.bci, next, EdgeKind::Normal, budget)?;
        }
    }
    Ok(())
}

fn push_edge(
    edges: &mut Vec<RawEdge>,
    from_bci: u32,
    to_bci: u32,
    kind: EdgeKind,
    budget: &mut Budget,
) -> Result<()> {
    budget.charge(CountedBudgetDimension::IrEdges, 1)?;
    edges.push(RawEdge {
        from_bci,
        to_bci,
        kind,
    });
    Ok(())
}

/// The throw sites of every throwing instruction, the handler facts, and the block-level
/// exception edges they imply, all in declaration order.
fn throw_sites_and_handlers(
    facts: &MethodCodeFacts,
    blocks: &[RawBlock],
    ranges: &[(usize, usize)],
    budget: &mut Budget,
) -> Result<(Vec<HandlerFact>, Vec<ThrowSite>, Vec<RawEdge>)> {
    let mut handlers = Vec::with_capacity(facts.exception_handlers.len());
    for handler in &facts.exception_handlers {
        // The record itself is a derived storage item of this pass: it is copied out of the
        // reader's facts so the graph can be read without the reader's own structures.
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        handlers.push(handler.clone());
    }
    handlers.sort_unstable_by_key(|handler| handler.ordinal);

    let mut throw_sites = Vec::new();
    let mut edges: Vec<RawEdge> = Vec::new();
    for (block_index, (start, end)) in ranges.iter().copied().enumerate() {
        let block = blocks[block_index];
        let mut seen: Vec<u32> = Vec::new();
        for (instruction, operands) in facts.instructions[start..end]
            .iter()
            .zip(facts.operands()[start..end].iter())
        {
            if !may_throw(operands.effective_opcode) {
                continue;
            }
            let feasible: Vec<u32> = handlers
                .iter()
                .filter(|handler| {
                    handler.start_bci <= instruction.bci && instruction.bci < handler.end_bci
                })
                .map(|handler| handler.ordinal)
                .collect();
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            throw_sites.push(ThrowSite {
                bci: instruction.bci,
                opcode: operands.effective_opcode,
                block_bci: block.bci,
                handlers: feasible.clone(),
            });
            for ordinal in feasible {
                if seen.contains(&ordinal) {
                    // Several throw sites of one block entering the same record are one edge:
                    // the site-level fact is `throw_sites`, and it kept both.
                    continue;
                }
                seen.push(ordinal);
                let to_bci = handlers
                    .iter()
                    .find(|handler| handler.ordinal == ordinal)
                    .map(|handler| handler.handler_bci)
                    .ok_or_else(|| missing_handler(ordinal))?;
                push_edge(
                    &mut edges,
                    block.bci,
                    to_bci,
                    EdgeKind::Exception {
                        handler_ordinal: ordinal,
                    },
                    budget,
                )?;
            }
        }
    }
    Ok((handlers, throw_sites, edges))
}

/// One effect entry per decoded instruction, in BCI order.
fn effect_facts(
    facts: &MethodCodeFacts,
    throw_sites: &[ThrowSite],
    budget: &mut Budget,
) -> Result<EffectFacts> {
    let mut instructions = Vec::with_capacity(facts.instructions.len());
    // Throw sites are one per instruction and sorted by BCI, so one cursor classifies the
    // whole list.
    let mut cursor = 0usize;
    for (instruction, operands) in facts.instructions.iter().zip(facts.operands().iter()) {
        // The instruction's own opcode: for a `wide` form that is the opcode it wraps, which
        // is what decides the local's direction (a `wide iload 300` reads local 300) and the
        // stack delta, exactly like the short form of the same opcode.
        let opcode = operands.effective_opcode;
        let (locals_read, locals_written) = match operands.local {
            Some(local) if operands.increment.is_some() => {
                // `iinc` reads and writes the same local, and its increment is not a stack
                // effect; `wide iinc` carries its increment the same way.
                (vec![local.index], vec![local.index])
            }
            Some(local) if is_load(opcode) => (vec![local.index], Vec::new()),
            Some(local) if is_store(opcode) => (Vec::new(), vec![local.index]),
            // `ret` names the local holding the return address: it reads it.
            Some(local) if opcode == OPCODE_RET => (vec![local.index], Vec::new()),
            _ => (Vec::new(), Vec::new()),
        };
        let (may_throw, handlers) = match throw_sites
            .get(cursor)
            .filter(|site| site.bci == instruction.bci)
        {
            Some(site) => {
                cursor += 1;
                (true, site.handlers.clone())
            }
            None => (false, Vec::new()),
        };
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        instructions.push(InstructionEffect {
            bci: instruction.bci,
            opcode,
            locals_read,
            locals_written,
            stack_delta: fixed_stack_delta(opcode, operands.dimensions),
            may_throw,
            handlers,
        });
    }
    Ok(EffectFacts { instructions })
}

/// The call sites of every `jsr`/`jsr_w` of the body, as BCIs ascending.
fn jsr_call_sites(facts: &MethodCodeFacts) -> Result<Vec<u32>> {
    let mut sites: Vec<u32> = facts
        .instructions
        .iter()
        .zip(facts.operands().iter())
        .filter(|(_, operands)| is_jsr(operands.effective_opcode))
        .map(|(instruction, _)| instruction.bci)
        .collect();
    sites.sort_unstable();
    Ok(sites)
}

/// The continuation of every `jsr`, as a block index: where the subroutine's `ret` returns.
///
/// Both ends are read with [`block_of`], the crate's "block *containing* this BCI" rule: a `jsr`
/// may sit in the middle of a block — the instructions before it transfer into it, so the block
/// ends at the `jsr` and not before it — and its continuation is then a successor of *that*
/// block. Requiring the call site to start a block (an exact match) would find no block, drop
/// this relation silently, and let the truth table call a live continuation dead.
fn jsr_continuations(facts: &MethodCodeFacts, blocks: &[RawBlock]) -> Vec<Option<NodeIndex>> {
    let mut continuations = vec![None; blocks.len()];
    for (index, (instruction, operands)) in facts
        .instructions
        .iter()
        .zip(facts.operands().iter())
        .enumerate()
    {
        if !is_jsr(operands.effective_opcode) {
            continue;
        }
        let Some(next) = facts.instructions.get(index + 1) else {
            continue;
        };
        let (Some(from), Some(to)) = (
            block_of(blocks, instruction.bci, |block| block.bci),
            block_of(blocks, next.bci, |block| block.bci),
        ) else {
            continue;
        };
        continuations[from] = Some(NodeIndex::new(to));
    }
    continuations
}

/// The blocks the entry cannot reach, by BCI ascending.
fn unreachable_blocks(
    graph: &DiGraph<u32, EdgeKind>,
    nodes: &[NodeIndex],
    continuations: &[Option<NodeIndex>],
    budget: &mut Budget,
) -> Result<Vec<u32>> {
    if nodes.is_empty() {
        return Ok(Vec::new());
    }
    let mut visited = vec![false; nodes.len()];
    // A worklist over the graph's own successors: one pop per visit, and a repeated visit is
    // charged again (the dimension counts real work). Which node a pop yields does not reach
    // the result — the published list is the *set* of unvisited blocks, sorted by BCI.
    let mut frontier = vec![nodes[0]];
    while let Some(node) = frontier.pop() {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let index = node.index();
        if visited[index] {
            continue;
        }
        visited[index] = true;
        for next in graph.neighbors(node) {
            frontier.push(next);
        }
        if let Some(continuation) = continuations[index] {
            // Raw semantics: a `ret` returns to the continuation of its call site, so a
            // reachable `jsr` keeps its continuation alive even though the raw graph has no
            // edge for the return half of the transfer.
            frontier.push(continuation);
        }
    }
    Ok(nodes
        .iter()
        .enumerate()
        .filter(|(index, _)| !visited[*index])
        .map(|(_, node)| graph[*node])
        .collect())
}

// The opcode predicates below all take the **effective** opcode
// ([`jarde_reader::classfile::InstructionOperands::effective_opcode`]): the `wide` prefix is not an
// instruction, so `wide iload`/`wide istore`/`wide ret` reach these predicates as `iload`,
// `istore` and `ret`.

/// Whether this opcode ends its block: an instruction whose transfer is not a plain
/// fall-through to the instruction after it.
///
/// The frame layer's dense table carries the same fact for its own rows — it needs it to walk a
/// canonical node's fused chain one original block at a time — and the frame module's tests check
/// the two against each other, so a classification cannot drift here without failing there.
pub(crate) fn ends_block(opcode: u8) -> bool {
    is_conditional_branch(opcode)
        || opcode == OPCODE_GOTO
        || opcode == OPCODE_GOTO_W
        || is_jsr(opcode)
        || is_transfer_stop(opcode)
        || opcode == OPCODE_TABLESWITCH
        || opcode == OPCODE_LOOKUPSWITCH
}

/// Whether this opcode is a branch whose target *and* fall-through are successors.
fn is_conditional_branch(opcode: u8) -> bool {
    matches!(opcode, 0x99..=0xa6 | OPCODE_IFNULL | OPCODE_IFNONNULL)
}

/// Whether control leaves the block with no raw edge at all: a return, an `athrow`, or a
/// `ret` whose return point belongs to the call contexts.
fn is_transfer_stop(opcode: u8) -> bool {
    // `ireturn`..`areturn`, `return`, `athrow`, `ret`.
    matches!(opcode, 0xac..=0xb1 | OPCODE_ATHROW | OPCODE_RET)
}

fn is_jsr(opcode: u8) -> bool {
    opcode == OPCODE_JSR || opcode == OPCODE_JSR_W
}

fn is_load(opcode: u8) -> bool {
    // `iload`..`aload` and the implicit `_0`..`_3` forms of all five types: one contiguous
    // block, 0x15 through 0x2d.
    matches!(opcode, 0x15..=0x2d)
}

fn is_store(opcode: u8) -> bool {
    // `istore`..`astore` and the implicit `_0`..`_3` forms of all five types: one contiguous
    // block, 0x36 through 0x4e.
    matches!(opcode, 0x36..=0x4e)
}

/// Whether an exception this instruction raises can enter a handler of the same method.
///
/// "Can" is the structural question, not a typing one: every instruction whose specified
/// exceptions a `catch` clause may name is listed. Comparison, conversion, local access, stack
/// manipulation, `goto` and the returns cannot raise, and neither can floating-point division
/// or remainder (they follow IEEE 754 instead of throwing), which is why they are absent.
fn may_throw(opcode: u8) -> bool {
    match opcode {
        // Array loads and stores: NPE, ArrayIndexOutOfBoundsException, ArrayStoreException.
        0x2e..=0x35 | 0x4f..=0x56 => true,
        // Integer division and remainder: ArithmeticException.
        0x6c | 0x6d | 0x70 | 0x71 => true,
        // Constant-pool constants: linkage, initialization and allocation errors.
        0x12..=0x14 => true,
        // Field access: linkage, initialization and NPE.
        0xb2..=0xb5 => true,
        // Invocations: linkage, NPE, and everything a callee may raise.
        0xb6..=0xba => true,
        // `new`..`monitorexit`: instantiation, array creation, `arraylength`, `athrow`,
        // `checkcast`, `instanceof` and synchronization.
        0xbb..=0xc3 => true,
        // `multianewarray`: NegativeArraySizeException, OutOfMemoryError.
        OPCODE_MULTIANEWARRAY => true,
        _ => false,
    }
}

/// The change of the operand-stack depth one instruction has, or `None` when the opcode alone
/// is not enough; see [`InstructionEffect::stack_delta`].
///
/// `dimensions` is the reader's own `multianewarray` operand fact, which is what makes that
/// instruction's delta decidable here instead of by a consumer (D41).
///
/// The opcode numbers follow the JVMS 6.5 table the reader itself is built on: the implicit
/// local forms run `_0`..`_3` for all five types (so `aload_0` is `0x2a`), `iinc` is `0x84`
/// and the integer arithmetic block starts at `iadd = 0x60`.
pub(crate) fn fixed_stack_delta(opcode: u8, dimensions: Option<u8>) -> Option<i32> {
    Some(match opcode {
        0x00 => 0,
        // Constants: `aconst_null`, `iconst_*`, `fconst_*` and the immediate pushes take one
        // slot, `lconst_*` and `dconst_*` two. `ldc`/`ldc_w` take what the constant is, so
        // they are not in this table; `ldc2_w` always takes two.
        0x01..=0x08 | 0x0b..=0x0d | 0x10 | 0x11 => 1,
        0x09 | 0x0a | 0x0e | 0x0f | 0x14 => 2,
        0x15 | 0x17 | 0x19 => 1,
        0x16 | 0x18 => 2,
        0x1a..=0x1d | 0x22..=0x25 | 0x2a..=0x2d => 1,
        0x1e..=0x21 | 0x26..=0x29 => 2,
        // Array loads pop the array and the index and push the element.
        0x2e | 0x30 | 0x32..=0x35 => -1,
        0x2f | 0x31 => 0,
        0x36 | 0x38 | 0x3a => -1,
        0x37 | 0x39 => -2,
        0x3b..=0x3e | 0x43..=0x46 | 0x4b..=0x4e => -1,
        0x3f..=0x42 | 0x47..=0x4a => -2,
        // Array stores pop the array, the index and the value.
        0x4f | 0x51 | 0x53..=0x56 => -3,
        0x50 | 0x52 => -4,
        0x57 => -1,
        0x58 => -2,
        0x59..=0x5b => 1,
        0x5c..=0x5e => 2,
        0x5f => 0,
        // Binary arithmetic and bitwise operations on one-slot and two-slot values.
        0x60 | 0x62 | 0x64 | 0x66 | 0x68 | 0x6a | 0x6c | 0x6e | 0x70 | 0x72 | 0x7e | 0x80
        | 0x82 => -1,
        0x61 | 0x63 | 0x65 | 0x67 | 0x69 | 0x6b | 0x6d | 0x6f | 0x71 | 0x73 | 0x7f | 0x81
        | 0x83 => -2,
        0x74..=0x77 => 0,
        // Shifts pop the value and the distance, and push the shifted value: one slot less,
        // whatever the value's width.
        0x78..=0x7d => -1,
        // `iinc` touches a local only.
        0x84 => 0,
        // Conversions: one slot gained when a one-slot value becomes a two-slot one, one lost
        // the other way, nothing when the widths agree.
        0x85 | 0x87 | 0x8c | 0x8d => 1,
        0x88 | 0x89 | 0x8e | 0x90 => -1,
        0x86 | 0x8a | 0x8b | 0x8f | 0x91..=0x93 => 0,
        // Comparisons produce one int from the values they consume.
        0x94 | 0x97 | 0x98 => -3,
        0x95 | 0x96 => -1,
        0x99..=0x9e | OPCODE_IFNULL | OPCODE_IFNONNULL => -1,
        0x9f..=0xa6 => -2,
        OPCODE_GOTO | OPCODE_GOTO_W => 0,
        OPCODE_JSR | OPCODE_JSR_W => 1,
        OPCODE_RET => 0,
        OPCODE_TABLESWITCH | OPCODE_LOOKUPSWITCH => -1,
        // `multianewarray` pops one length per dimension and pushes the array it creates: D41
        // is `1 - dimensions`, decided from the reader's operand fact instead of left to a
        // consumer. A count the encoding cannot carry makes the delta undecidable here, which is
        // a fact about the operand and not a value to invent.
        OPCODE_MULTIANEWARRAY => 1 - i32::from(dimensions?),
        0xac | 0xae | 0xb0 => -1,
        0xad | 0xaf => -2,
        0xb1 => 0,
        0xbb => 1,
        0xbc..=0xbe => 0,
        0xc0 | 0xc1 => 0,
        0xc2 | 0xc3 => -1,
        // What is left either needs more than the opcode (`ldc`, field access, invocations and
        // `athrow`, which the frame layer decides from its own table) or is not an instruction.
        // A `wide` form never reaches this table as `0xc4`: it arrives as the opcode it wraps.
        _ => return None,
    })
}

/// Index of the instruction at one BCI.
fn instruction_index(facts: &MethodCodeFacts, bci: u32) -> Result<usize> {
    facts
        .instructions
        .binary_search_by_key(&bci, |instruction| instruction.bci)
        .map_err(|_| {
            Error::invalid_input(
                IR_RAW_CFG_UNKNOWN_TARGET,
                format!("BCI {bci} is not an instruction start of this method body"),
            )
        })
}

/// Index of the block **containing** one BCI: the last block whose start is at or below it.
///
/// This is the crate's one BCI→block reading (0.4): 3.4's call contexts pass their own block
/// list to this same function, so `cfg` and `call_context` cannot drift apart on what "the block
/// of a BCI" means. It is a `partition_point` search and not an exact match because a BCI that
/// lies inside a block belongs to that block — `jsr` is what makes the difference observable: a
/// `jsr` that is not the first instruction of its block still belongs to it, and the
/// continuation of that call is a successor of that block.
///
/// `blocks` is ordered by start BCI ascending, as every published block list is. `None` means
/// the BCI lies before the first block, which a graph consistent with the reader's facts cannot
/// produce; callers name that case with their own diagnostic. `start_bci` reads the start of one
/// entry so the same reading serves a [`RawBlock`] list and 3.4's list of start BCIs.
pub(crate) fn block_of<T>(blocks: &[T], bci: u32, start_bci: impl Fn(&T) -> u32) -> Option<usize> {
    blocks
        .partition_point(|block| start_bci(block) <= bci)
        .checked_sub(1)
}

/// Position of the block that starts at one BCI: the block identity of one graph coordinate.
///
/// Exact on purpose, and only for inputs that are block starts **by construction**: its callers
/// are the endpoints of the published edges, whose BCIs are branch, switch or handler targets or
/// the fall-through after a block ender, and [`leader_flags`] makes every one of those a leader.
/// A BCI that merely *lies inside* a block is not this function's question — that one is
/// [`block_of`]'s, and `jsr` is the case where the two answers differ.
fn block_position(blocks: &[RawBlock], bci: u32) -> Result<usize> {
    blocks
        .binary_search_by_key(&bci, |block| block.bci)
        .map_err(|_| {
            Error::invalid_input(
                IR_RAW_CFG_UNKNOWN_TARGET,
                format!("BCI {bci} is not the start of a raw block"),
            )
        })
}

fn missing_target(kind: &str, bci: u32) -> Error {
    Error::invalid_input(
        IR_RAW_CFG_UNKNOWN_TARGET,
        format!("the {kind} at BCI {bci} has no validated target"),
    )
}

fn missing_handler(ordinal: u32) -> Error {
    Error::invalid_input(
        IR_RAW_CFG_UNKNOWN_TARGET,
        format!("exception-table record {ordinal} has no handler entry"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarde_reader::budget::{Limits, UsageSnapshot};
    use jarde_reader::classfile::{
        InstructionFact, InstructionOperands, LocalDebugTable, LocalOperand, SwitchOperands,
    };
    use jarde_reader::model::{ByteSpan, ExecutionReport, TerminationReason};

    /// Class-file offset the fixture bodies start at: the facts and the graph carry BCIs, but
    /// the spans must still be coherent.
    const CODE_OFFSET: u64 = 200;

    fn limits() -> Limits {
        Limits {
            ir_items: 1_000_000,
            ir_edges: 1_000_000,
            analysis_steps: 1_000_000,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        }
    }

    fn budget() -> Budget {
        Budget::new(limits())
    }

    /// One instruction fact with its typed operands, at `bci`, `width` bytes long.
    ///
    /// The fixture states the effective opcode itself: it is the raw opcode for every
    /// instruction except a `wide` form, whose raw opcode is the prefix `0xc4` and whose
    /// effective opcode is the one it wraps (0.2).
    fn instruction(
        bci: u32,
        opcode: u8,
        width: u32,
        operands: InstructionOperands,
    ) -> (InstructionFact, InstructionOperands) {
        debug_assert!(
            operands.effective_opcode == opcode || opcode == 0xc4,
            "a fixture must state the opcode the classifications read"
        );
        let start = CODE_OFFSET + u64::from(bci);
        (
            InstructionFact {
                bci,
                opcode,
                width,
                span: ByteSpan::new(start, u64::from(width)),
                operands_span: ByteSpan::new(start + 1, u64::from(width - 1)),
                constant_pool_index: operands.constant_pool_index,
            },
            operands,
        )
    }

    /// Operands of an instruction whose effective opcode is its raw opcode.
    fn operands(opcode: u8) -> InstructionOperands {
        InstructionOperands {
            effective_opcode: opcode,
            ..InstructionOperands::default()
        }
    }

    /// An operandless instruction (`nop`, a load of a fixed form, a return, …).
    fn plain(bci: u32, opcode: u8) -> (InstructionFact, InstructionOperands) {
        instruction(bci, opcode, 1, operands(opcode))
    }

    fn branch(bci: u32, opcode: u8, offset: i32) -> (InstructionFact, InstructionOperands) {
        instruction(
            bci,
            opcode,
            3,
            InstructionOperands {
                branch_offset: Some(offset),
                ..operands(opcode)
            },
        )
    }

    /// One named local operand of an instruction whose effective opcode is `effective_opcode`,
    /// in the short form unless `wide` says otherwise.
    fn local(index: u16, wide: bool, effective_opcode: u8) -> InstructionOperands {
        InstructionOperands {
            local: Some(LocalOperand { index, wide }),
            effective_opcode,
            ..InstructionOperands::default()
        }
    }

    /// A `wide`-prefixed local access: the raw opcode is the prefix and the effective one is
    /// the opcode it wraps.
    fn wide_local(index: u16, wrapped_opcode: u8) -> InstructionOperands {
        local(index, true, wrapped_opcode)
    }

    /// A `tableswitch` whose payload is read from the operands, with the padding its BCI
    /// forces (the same rule the reader applies).
    fn table_switch(
        bci: u32,
        default_offset: i32,
        low: i32,
        offsets: Vec<i32>,
    ) -> (InstructionFact, InstructionOperands) {
        let padding = (4 - ((bci + 1) & 3)) & 3;
        let width = 1 + padding + 12 + 4 * u32::try_from(offsets.len()).expect("fixture width");
        let high = low + i32::try_from(offsets.len()).expect("fixture width") - 1;
        instruction(
            bci,
            OPCODE_TABLESWITCH,
            width,
            InstructionOperands {
                switch: Some(SwitchOperands::Table {
                    default_offset,
                    low,
                    high,
                    offsets,
                }),
                ..operands(OPCODE_TABLESWITCH)
            },
        )
    }

    fn catch(
        ordinal: u32,
        start: u32,
        end: u32,
        handler: u32,
        catch_type: Option<u16>,
    ) -> ExceptionHandlerFact {
        ExceptionHandlerFact {
            ordinal,
            start_bci: start,
            end_bci: end,
            handler_bci: handler,
            catch_type_index: catch_type,
        }
    }

    /// A complete body over the given instructions.
    fn body(
        code: Vec<(InstructionFact, InstructionOperands)>,
        handlers: Vec<ExceptionHandlerFact>,
        code_length: u32,
    ) -> MethodCodeFacts {
        let exception_handler_count = u32::try_from(handlers.len()).expect("fixture handlers");
        MethodCodeFacts::from_parts(
            8,
            8,
            ByteSpan::new(CODE_OFFSET, u64::from(code_length)),
            code,
            handlers,
            exception_handler_count,
            ExecutionReport::Complete {
                usage: UsageSnapshot::default(),
            },
            None,
            // An assembled body states no debug table: the names come from a class file's own
            // `LocalVariableTable`, which no assembled fixture has (P3 3.1).
            LocalDebugTable::Absent,
        )
    }

    /// The same body as one whose decode stopped after the decoded prefix, with `code_length`
    /// the length the `Code` attribute declares.
    fn truncated(mut facts: MethodCodeFacts, stop_bci: u32, code_length: u32) -> MethodCodeFacts {
        facts.code_span = ByteSpan::new(CODE_OFFSET, u64::from(code_length));
        facts.stopped_at = Some(BytecodeStop::Instructions {
            bci: stop_bci,
            class_offset: CODE_OFFSET + u64::from(stop_bci),
            code: "classfile_bytecode_budget_exceeded".to_string(),
        });
        facts.execution = ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::CodeBytes,
            },
            usage: UsageSnapshot::default(),
        };
        facts
    }

    fn block_bcis(cfg: &RawCfg) -> Vec<u32> {
        cfg.blocks.iter().map(|block| block.bci).collect()
    }

    fn edge_tuples(cfg: &RawCfg) -> Vec<(u32, EdgeKind, u32)> {
        cfg.edges
            .iter()
            .map(|edge| (edge.from_bci, edge.kind, edge.to_bci))
            .collect()
    }

    #[test]
    fn a_conditional_branch_keeps_both_successors_and_partitions_the_body() {
        let facts = body(
            vec![
                plain(0, 0x03),     // iconst_0
                branch(1, 0x99, 5), // ifeq +5 -> 6
                plain(4, 0x03),     // iconst_1
                plain(5, 0xac),     // ireturn
                plain(6, 0x05),     // iconst_2
                plain(7, 0xac),     // ireturn
            ],
            Vec::new(),
            8,
        );
        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        assert_eq!(block_bcis(&outcome.cfg), vec![0, 4, 6]);
        assert_eq!(
            edge_tuples(&outcome.cfg),
            vec![(0, EdgeKind::Normal, 4), (0, EdgeKind::Normal, 6),],
            "a conditional branch has its target and its fall-through"
        );
        assert!(outcome.cfg.unreachable.is_empty());
        assert!(outcome.cfg.completeness.is_complete());
        assert!(outcome.cfg.unresolved_returns.is_empty());
        // The partition covers the decoded prefix exactly once: every instruction sits in one
        // block range, and the ranges tile the body.
        assert_eq!(
            outcome
                .cfg
                .blocks
                .iter()
                .map(|block| block.end_bci - block.bci)
                .sum::<u32>(),
            8,
            "the blocks tile the decoded body"
        );
        assert_eq!(
            outcome
                .cfg
                .blocks
                .first()
                .map(|block| (block.bci, block.end_bci)),
            Some((0, 4)),
            "the entry block is the first block and starts at BCI 0"
        );
    }

    #[test]
    fn a_branch_to_its_own_fall_through_is_one_successor() {
        // `ifeq +3` at BCI 1 targets BCI 4, which is the instruction it falls through to
        // anyway: that is one successor, not two. The branch at BCI 5 keeps its two — a
        // fall-through and a target that is not it — so the deduplication drops a repeated
        // successor and not the second edge of a branch.
        let facts = body(
            vec![
                plain(0, 0x03),     // iconst_0
                branch(1, 0x99, 3), // ifeq +3 -> 4: its own fall-through
                plain(4, 0x03),     // iconst_1
                branch(5, 0x99, 4), // ifeq +4 -> 9, falling through to 8
                plain(8, 0xb1),     // return
                plain(9, 0xb1),     // return
            ],
            Vec::new(),
            10,
        );
        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        assert_eq!(block_bcis(&outcome.cfg), vec![0, 4, 8, 9]);
        assert_eq!(
            edge_tuples(&outcome.cfg),
            vec![
                (0, EdgeKind::Normal, 4),
                (4, EdgeKind::Normal, 8),
                (4, EdgeKind::Normal, 9),
            ],
            "the branch whose target is its fall-through has one edge, the other one two"
        );
        assert!(outcome.cfg.unreachable.is_empty());
        assert!(outcome.cfg.completeness.is_complete());
    }

    #[test]
    fn a_switch_emits_one_edge_per_distinct_target() {
        let facts = body(
            vec![
                plain(0, 0x03),                           // iconst_0
                table_switch(1, 29, 0, vec![27, 28, 27]), // default 30, cases 28, 29, 28
                plain(28, 0x00),                          // nop
                plain(29, 0xb1),                          // return
                plain(30, 0x00),                          // nop
                plain(31, 0xb1),                          // return
            ],
            Vec::new(),
            32,
        );
        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        assert_eq!(
            block_bcis(&outcome.cfg),
            vec![0, 28, 29, 30],
            "every case label is a block start; BCI 31 is reached by falling through 30"
        );
        assert_eq!(
            edge_tuples(&outcome.cfg),
            vec![
                (0, EdgeKind::Normal, 28),
                (0, EdgeKind::Normal, 29),
                (0, EdgeKind::Normal, 30),
                (28, EdgeKind::Normal, 29),
            ],
            "the default and each distinct case label are successors, the duplicate once"
        );
    }

    #[test]
    fn an_unreachable_tail_is_listed_and_kept_as_blocks() {
        let facts = body(
            vec![
                branch(0, 0xa7, 3), // goto +3 -> 3
                plain(3, 0xb1),     // return
                plain(4, 0x03),     // iconst_1 (unreachable)
                plain(5, 0xac),     // ireturn
            ],
            Vec::new(),
            6,
        );
        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        assert_eq!(
            block_bcis(&outcome.cfg),
            vec![0, 3, 4],
            "an unreachable block is kept in the block list"
        );
        assert_eq!(outcome.cfg.unreachable, vec![4]);
    }

    #[test]
    fn overlapping_handlers_keep_declaration_order_and_parallel_edges() {
        // `astore_2` and `aload_3` of one subroutine, plus an `athrow` at 14; the record at
        // ordinal 1 has an empty handler list on purpose.
        let facts = body(
            vec![
                plain(0, 0x01),     // aconst_null
                plain(1, 0xbe),     // arraylength: the throw site
                plain(2, 0x57),     // pop
                branch(3, 0xa7, 5), // goto +5 -> 8
                plain(6, 0xb1),     // return (unreachable)
                plain(7, 0x00),     // nop
                plain(8, 0xb1),     // return (handlers 0 and 1)
                plain(9, 0x00),     // nop (unreachable)
                plain(10, 0xb1),    // return (handler 2)
            ],
            vec![
                catch(0, 0, 6, 8, Some(1)),
                catch(1, 1, 6, 8, Some(2)),
                catch(2, 1, 6, 10, None),
            ],
            11,
        );
        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        assert_eq!(block_bcis(&outcome.cfg), vec![0, 6, 7, 8, 9, 10]);
        assert_eq!(
            outcome
                .cfg
                .handlers
                .iter()
                .map(|handler| handler.ordinal)
                .collect::<Vec<_>>(),
            vec![0, 1, 2],
            "handler facts stay in declaration order"
        );
        assert_eq!(
            outcome.cfg.throw_sites,
            vec![ThrowSite {
                bci: 1,
                opcode: 0xbe,
                block_bci: 0,
                handlers: vec![0, 1, 2],
            }],
            "one throw site per throwing instruction, with every covering record in order"
        );
        assert_eq!(
            edge_tuples(&outcome.cfg),
            vec![
                (0, EdgeKind::Normal, 8),
                (0, EdgeKind::Exception { handler_ordinal: 0 }, 8),
                (0, EdgeKind::Exception { handler_ordinal: 1 }, 8),
                (0, EdgeKind::Exception { handler_ordinal: 2 }, 10),
                (7, EdgeKind::Normal, 8),
                (9, EdgeKind::Normal, 10),
            ],
            "records 0 and 1 share a handler entry: both edges are kept"
        );
        assert_eq!(outcome.cfg.unreachable, vec![6, 7, 9]);
    }

    #[test]
    fn repeated_throw_sites_of_one_block_share_one_edge_per_record() {
        // Two throwing instructions in the same block, covered by the same two records: the
        // site-level facts stay one per instruction, while each record contributes one edge
        // for the block. The deduplication is keyed by the record — the entry is the same BCI
        // for both records, and both edges are kept.
        let facts = body(
            vec![
                plain(0, 0x01), // aconst_null
                plain(1, 0xbe), // arraylength: the first throw site of the block
                plain(2, 0x00), // nop
                plain(3, 0xbe), // arraylength: the second throw site of the same block
                plain(4, 0x00), // nop
                plain(5, 0xb1), // return
                plain(6, 0xb1), // return: the entry both records name
            ],
            vec![catch(0, 0, 6, 6, Some(1)), catch(1, 0, 6, 6, Some(2))],
            7,
        );
        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        assert_eq!(block_bcis(&outcome.cfg), vec![0, 6]);
        assert_eq!(
            outcome
                .cfg
                .throw_sites
                .iter()
                .map(|site| (site.bci, site.block_bci, site.handlers.clone()))
                .collect::<Vec<_>>(),
            vec![(1, 0, vec![0, 1]), (3, 0, vec![0, 1])],
            "two throw sites of one block stay two facts, each listing both records"
        );
        assert_eq!(
            edge_tuples(&outcome.cfg),
            vec![
                (0, EdgeKind::Exception { handler_ordinal: 0 }, 6),
                (0, EdgeKind::Exception { handler_ordinal: 1 }, 6),
            ],
            "one edge per record and block, not one per throw site or per handler entry"
        );
        assert!(outcome.cfg.unreachable.is_empty());
    }

    #[test]
    fn self_loops_are_expressed_and_do_not_break_reachability() {
        let facts = body(
            vec![
                branch(0, 0xa7, 0), // goto +0 -> 0
                plain(3, 0x01),     // aconst_null
                plain(4, 0xbe),     // arraylength: the throw site of a handler at BCI 0
            ],
            vec![catch(0, 3, 5, 0, None)],
            5,
        );
        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        assert_eq!(
            edge_tuples(&outcome.cfg),
            vec![
                (0, EdgeKind::Normal, 0),
                (3, EdgeKind::Exception { handler_ordinal: 0 }, 0),
            ],
            "a `goto` to itself and a handler entering the block that raises are self-loops"
        );
        assert_eq!(
            outcome.cfg.unreachable,
            vec![3],
            "the walk terminates and the block no edge reaches stays listed"
        );
    }

    #[test]
    fn a_truncated_body_marks_the_graph_truncated() {
        let facts = truncated(
            body(
                vec![
                    plain(0, 0x03),     // iconst_0
                    branch(1, 0x99, 3), // ifeq +3 -> 4
                    plain(4, 0x03),     // iconst_1
                    plain(5, 0xac),     // ireturn
                ],
                Vec::new(),
                6,
            ),
            6,
            9,
        );
        let outcome = raw_cfg(&facts, &mut budget()).expect("the decoded prefix is valid");
        assert_eq!(
            outcome.cfg.completeness,
            CfgCompleteness::Truncated {
                stopped_at: facts
                    .stopped_at
                    .clone()
                    .expect("the fixture carries its stop"),
            }
        );
        assert!(!outcome.cfg.completeness.is_complete());
        assert_eq!(
            block_bcis(&outcome.cfg),
            vec![0, 4],
            "the graph covers the decoded prefix"
        );
        assert_eq!(outcome.cfg.blocks[1].end_bci, 6, "the prefix ends at BCI 6");
        assert_eq!(
            block_bcis(&outcome.cfg).last().copied(),
            Some(4),
            "no block is invented for the unread suffix"
        );
    }

    #[test]
    fn jsr_returns_stay_unresolved_and_their_continuation_stays_reachable() {
        let facts = body(
            vec![
                branch(0, OPCODE_JSR, 5), // jsr +5 -> 5
                plain(3, 0xb1),           // return: the continuation of the call
                plain(4, 0x00),           // nop
                plain(5, 0x00),           // nop: the subroutine entry
                instruction(6, OPCODE_RET, 2, local(1, false, OPCODE_RET)),
            ],
            Vec::new(),
            8,
        );
        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        assert_eq!(
            edge_tuples(&outcome.cfg),
            vec![
                (0, EdgeKind::SubroutineReturn { call_site: 0 }, 5),
                (4, EdgeKind::Normal, 5),
            ],
            "the raw graph has the call edge, and `ret` contributes none"
        );
        assert_eq!(outcome.cfg.unresolved_returns, vec![0]);
        assert!(
            !outcome.cfg.unreachable.contains(&3),
            "the continuation of a reachable `jsr` is where its `ret` returns"
        );
        assert_eq!(
            outcome.cfg.unreachable,
            vec![4],
            "the `return` after the call does not fall through either"
        );
    }

    #[test]
    fn a_jsr_inside_a_block_keeps_its_continuation_reachable() {
        // The shape the shared `finally` of the ECJ corpus has (0.4, D-1): the `jsr` at BCI 5 is
        // **not** the first instruction of its block — nothing before BCI 4 ends a block, so the
        // `istore_1` at BCI 4 and the `jsr` belong to the same block `[4, 8)`. Its continuation
        // is a successor of that block, so reading the call site as "the block that starts at
        // BCI 5" finds no block, drops the relation, and the truth table then lists the live
        // continuation at BCI 8 as dead. The dead tail at BCI 12 is unreachable for real and
        // must stay listed: the fix narrows the table, it does not empty it.
        let facts = body(
            vec![
                plain(0, 0x04),                                              // iconst_1
                branch(1, 0x99, 3), // ifeq +3 -> 4 (the block before the call)
                plain(4, 0x3c),     // istore_1: the instruction before the call
                branch(5, OPCODE_JSR, 4), // jsr +4 -> 9, inside the call's own block
                plain(8, 0xac),     // ireturn: the continuation of the call
                plain(9, 0x4c),     // astore_1: the subroutine entry
                instruction(10, OPCODE_RET, 2, local(1, false, OPCODE_RET)), // ret 1
                plain(12, 0xb1),    // return: no transfer enters it
            ],
            Vec::new(),
            13,
        );
        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        assert_eq!(
            block_bcis(&outcome.cfg),
            vec![0, 4, 8, 9, 12],
            "the `jsr` at BCI 5 shares block `[4, 8)` with the `istore_1` before it, and the \
             `ret` shares block `[9, 12)` with the `astore_1` that stores the return address"
        );
        assert_eq!(
            edge_tuples(&outcome.cfg),
            vec![
                (0, EdgeKind::Normal, 4),
                (4, EdgeKind::SubroutineReturn { call_site: 5 }, 9),
            ],
            "the call is an edge from the block that holds it, not from a block starting at 5"
        );
        assert_eq!(outcome.cfg.unresolved_returns, vec![5]);
        assert_eq!(
            outcome.cfg.unreachable,
            vec![12],
            "the continuation at BCI 8 is where the `ret` returns, and only the dead tail is dead"
        );
    }

    #[test]
    fn the_historical_finally_paths_keep_the_subroutine_return_reachable() {
        // The same defect on the committed corpus: `finallyPath(I)I` of 45–48 calls its shared
        // subroutine from BCI 5, which sits inside block `[0, 8)` (the `istore_1` at BCI 4 before
        // it is no leader), so the block at BCI 8 — where `ret 1` returns — is live, and the
        // truth table has to say so. The two blocks that stay listed are the handler path
        // `[11, 15)` (entered through the exception table, which no raw edge of this body can
        // reach) and `[15, 17)`, the continuation of the `jsr` inside that dead path.
        const V45: &[u8] =
            crate::test_fixtures::fixture!("historical/ecj-4.6.1/v45/HistoricalControlFlow.class");
        const V46: &[u8] =
            crate::test_fixtures::fixture!("historical/ecj-4.6.1/v46/HistoricalControlFlow.class");
        const V47: &[u8] =
            crate::test_fixtures::fixture!("historical/ecj-4.6.1/v47/HistoricalControlFlow.class");
        const V48: &[u8] =
            crate::test_fixtures::fixture!("historical/ecj-4.6.1/v48/HistoricalControlFlow.class");
        for (major, bytes) in [(45_u16, V45), (46, V46), (47, V47), (48, V48)] {
            let facts = historical(bytes, b"finallyPath");
            let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
            assert_eq!(
                block_bcis(&outcome.cfg),
                vec![0, 8, 11, 15, 17],
                "classfile major {major}"
            );
            assert_eq!(
                outcome.cfg.unreachable,
                vec![11, 15],
                "classfile major {major}: `ret` returns to BCI 8, so `[8, 11)` is not dead"
            );
        }
    }

    #[test]
    fn effects_classify_locals_throw_sites_and_stack_deltas() {
        let facts = body(
            vec![
                instruction(0, 0x1b, 1, local(1, false, 0x1b)), // iload_1
                instruction(
                    1,
                    0x84,
                    3,
                    InstructionOperands {
                        local: Some(LocalOperand {
                            index: 2,
                            wide: false,
                        }),
                        increment: Some(1),
                        ..operands(0x84)
                    },
                ), // iinc 2 by 1
                instruction(4, 0x3d, 1, local(2, false, 0x3d)), // istore_2
                plain(5, 0x60),                                 // iadd
                plain(6, 0x6c),                                 // idiv
                plain(7, OPCODE_ATHROW),                        // athrow
                instruction(8, 0x12, 2, operands(0x12)),        // ldc
                instruction(10, 0xc4, 4, wide_local(3, 0x15)),  // wide iload 3
                plain(14, 0xac),                                // ireturn
            ],
            Vec::new(),
            15,
        );
        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        let effects = &outcome.effects.instructions;
        let effect = |bci: u32| {
            effects
                .iter()
                .find(|effect| effect.bci == bci)
                .expect("every instruction has an effect")
        };
        assert_eq!(effects.len(), facts.instructions.len());
        assert_eq!(effect(0).locals_read, vec![1]);
        assert!(effect(0).locals_written.is_empty());
        assert_eq!(effect(0).stack_delta, Some(1));
        assert!(!effect(0).may_throw);
        assert_eq!(effect(1).locals_read, vec![2]);
        assert_eq!(effect(1).locals_written, vec![2], "`iinc` reads and writes");
        assert_eq!(effect(1).stack_delta, Some(0));
        assert!(effect(4).locals_read.is_empty());
        assert_eq!(effect(4).locals_written, vec![2]);
        assert_eq!(effect(4).stack_delta, Some(-1));
        assert_eq!(effect(6).stack_delta, Some(-1));
        assert!(effect(6).may_throw, "`idiv` raises ArithmeticException");
        assert_eq!(effect(7).stack_delta, None, "`athrow` clears the stack");
        assert!(effect(7).may_throw);
        assert_eq!(
            effect(8).stack_delta,
            None,
            "`ldc` needs the constant's own width"
        );
        assert!(effect(8).may_throw);
        assert_eq!(
            effect(10).opcode,
            0x15,
            "a `wide iload` is the `iload` it wraps"
        );
        assert_eq!(
            effect(10).stack_delta,
            Some(1),
            "the wrapped opcode decides the delta, exactly like `iload 3`"
        );
        assert_eq!(
            effect(10).locals_read,
            vec![3],
            "the wrapped opcode decides the direction: the wide form reads its local"
        );
        assert!(effect(10).locals_written.is_empty());
        assert!(!effect(10).may_throw);
        assert_eq!(effect(14).stack_delta, Some(-1));
        assert_eq!(
            outcome
                .cfg
                .throw_sites
                .iter()
                .map(|site| (site.bci, site.handlers.clone()))
                .collect::<Vec<_>>(),
            vec![(6, Vec::new()), (7, Vec::new()), (8, Vec::new())],
            "an unprotected throw site is a fact, and it lists no handler"
        );
        assert!(effect(6).handlers.is_empty());
        assert!(effect(6).may_throw);
    }

    /// The decoded facts of one body built by the shared test class builder, through the real
    /// reader: these bytes are decoded by the same path the engine uses, so the classification
    /// below is exercised on reader facts rather than on hand-written ones.
    fn decoded(code: &[u8], major: u16) -> MethodCodeFacts {
        decoded_with_locals(code, major, 8, 8)
    }

    /// The same with the fixture's own frame: a body that names a local above 255 — the case the
    /// wide forms of 0.2 are for — has to declare that many local slots.
    fn decoded_with_locals(
        code: &[u8],
        major: u16,
        max_stack: u16,
        max_locals: u16,
    ) -> MethodCodeFacts {
        let bytes =
            jarde_reader::classfile::test_class::single_method(major, max_stack, max_locals, code);
        let mut budget = Budget::new(Limits {
            // The real reader charges the byte dimensions of this module's own fixture.
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            ..limits()
        });
        let header = jarde_reader::classfile::class_facts(&bytes, &mut budget)
            .expect("the fixture is a class file");
        let member = header
            .methods
            .first()
            .expect("the fixture declares one method");
        jarde_reader::classfile::method_code_facts(&bytes, member, &mut budget)
            .expect("the fixture's body decodes")
    }

    /// The decoded facts of one method of a committed historical fixture, through the real
    /// reader: the ECJ corpus bytes 3.4's own unit tests read, decoded here for the raw graph.
    fn historical(bytes: &[u8], name: &[u8]) -> MethodCodeFacts {
        let mut budget = Budget::new(Limits {
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            ..limits()
        });
        let facts = jarde_reader::classfile::class_facts(bytes, &mut budget)
            .expect("the fixture is a class file");
        let member = facts
            .methods
            .iter()
            .find(|member| member.name.raw().0.as_slice() == name)
            .expect("the fixture declares the method");
        jarde_reader::classfile::method_code_facts(bytes, member, &mut budget)
            .expect("the fixture's body decodes")
    }

    #[test]
    fn wide_forms_are_classified_exactly_like_the_short_forms_of_the_same_opcode() {
        // Real bytes, one body, the four local-access instructions in both encodings:
        //
        //   0  iload 0          7  ret 0          23  wide ret 0
        //   2  istore 0         9  wide iload 0   27  return
        //   4  iinc 0, 1       13  wide istore 0
        //                      17  wide iinc 0, 1
        //
        // The classification of each wide form must be the classification of the short form
        // it wraps — a read, a write, read+write with no stack effect, or the end of a block
        // with no raw edge — and the raw `0xc4` prefix must stay in the reader's fact.
        let code = [
            0x15, 0x00, // iload 0
            0x36, 0x00, // istore 0
            0x84, 0x00, 0x01, // iinc 0, 1
            0xa9, 0x00, // ret 0
            0xc4, 0x15, 0x00, 0x00, // wide iload 0
            0xc4, 0x36, 0x00, 0x00, // wide istore 0
            0xc4, 0x84, 0x00, 0x00, 0x00, 0x01, // wide iinc 0, 1
            0xc4, 0xa9, 0x00, 0x00, // wide ret 0
            0xb1, // return
        ];
        let facts = decoded(&code, 52);
        assert!(matches!(facts.execution, ExecutionReport::Complete { .. }));
        assert_eq!(facts.stopped_at, None);
        assert_eq!(
            facts
                .instructions
                .iter()
                .map(|fact| fact.opcode)
                .collect::<Vec<_>>(),
            vec![0x15, 0x36, 0x84, 0xa9, 0xc4, 0xc4, 0xc4, 0xc4, 0xb1],
            "the raw opcodes, byte for byte: the four wide forms keep the prefix"
        );

        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        assert_eq!(
            outcome
                .cfg
                .blocks
                .iter()
                .map(|block| (block.bci, block.end_bci))
                .collect::<Vec<_>>(),
            vec![(0, 9), (9, 27), (27, 28)],
            "`ret` and `wide ret` alike end their block: the instruction after each starts one"
        );
        assert_eq!(
            outcome.cfg.unresolved_returns,
            Vec::<u32>::new(),
            "`jsr` is what makes a return unresolved, and this body has none"
        );
        assert_eq!(
            edge_tuples(&outcome.cfg),
            Vec::<(u32, EdgeKind, u32)>::new(),
            "a `ret` leaves no raw edge, and its wide form is a `ret`"
        );

        let effects = &outcome.effects.instructions;
        let effect = |bci: u32| {
            effects
                .iter()
                .find(|effect| effect.bci == bci)
                .unwrap_or_else(|| panic!("no effect at BCI {bci}"))
        };
        for (short_bci, wide_bci) in [(0u32, 9u32), (2, 13), (4, 17), (7, 23)] {
            let short = effect(short_bci);
            let wide = effect(wide_bci);
            assert_eq!(
                InstructionEffect {
                    bci: wide.bci,
                    ..short.clone()
                },
                *wide,
                "BCI {wide_bci} must classify exactly like the short form at BCI {short_bci}"
            );
        }
        // The pairs are not two identical empty classifications: the wrapped opcode decides
        // each of them, and the raw prefix decides none.
        assert_eq!(
            (effect(0).locals_read.clone(), effect(0).stack_delta),
            (vec![0], Some(1))
        );
        assert_eq!(
            (effect(2).locals_written.clone(), effect(2).stack_delta),
            (vec![0], Some(-1))
        );
        assert_eq!(
            (
                effect(4).locals_read.clone(),
                effect(4).locals_written.clone(),
                effect(4).stack_delta
            ),
            (vec![0], vec![0], Some(0))
        );
        assert_eq!(
            (effect(7).locals_read.clone(), effect(7).stack_delta),
            (vec![0], Some(0))
        );
        assert!(
            effects.iter().all(|effect| !effect.may_throw),
            "a local access cannot enter a handler, wide or not"
        );
    }

    #[test]
    fn category_two_wide_local_accesses_match_their_short_forms() {
        // D40: the other half of the same classification — the two-slot local accesses, in both
        // encodings, with the wide form naming a local above 255 so the wrapped opcode's width
        // cannot be confused with the prefix's:
        //
        //   0  lload 30          4  wide lload 300
        //   2  lstore 31         8  wide lstore 301
        //                       12  return
        //
        // A `wide lload` reads one local and pushes two slots exactly like `lload`, and a
        // `wide lstore` writes one local and pops two exactly like `lstore`.
        let code = [
            0x16, 0x1e, // lload 30
            0x37, 0x1f, // lstore 31
            0xc4, 0x16, 0x01, 0x2c, // wide lload 300
            0xc4, 0x37, 0x01, 0x2d, // wide lstore 301
            0xb1, // return
        ];
        let facts = decoded_with_locals(&code, 52, 4, 302);
        assert!(matches!(facts.execution, ExecutionReport::Complete { .. }));
        assert_eq!(facts.stopped_at, None);
        assert_eq!(
            facts.max_locals, 302,
            "the fixture declares the slots it names"
        );

        let outcome = raw_cfg(&facts, &mut budget()).expect("the fixture is a valid body");
        let effects = &outcome.effects.instructions;
        let effect = |bci: u32| {
            effects
                .iter()
                .find(|effect| effect.bci == bci)
                .unwrap_or_else(|| panic!("no effect at BCI {bci}"))
        };
        /// One effect with the *identity* of its locals erased: `wide lload 300` and `lload 30`
        /// must differ in which local they name and in nothing else, so the comparison below runs
        /// on the classification with each local list replaced by its positions.
        fn classification_without_the_local(effect: &InstructionEffect) -> InstructionEffect {
            let positions = |locals: &[u16]| -> Vec<u16> {
                (0..locals.len())
                    .map(|index| u16::try_from(index).expect("a local list is tiny"))
                    .collect()
            };
            InstructionEffect {
                bci: 0,
                locals_read: positions(&effect.locals_read),
                locals_written: positions(&effect.locals_written),
                ..effect.clone()
            }
        }
        for (short_bci, wide_bci) in [(0_u32, 4_u32), (2, 8)] {
            assert_eq!(
                classification_without_the_local(effect(short_bci)),
                classification_without_the_local(effect(wide_bci)),
                "BCI {wide_bci} must classify exactly like the short form at BCI {short_bci}"
            );
        }
        // The pairs are not two identical empty classifications: the wrapped opcode decides the
        // local, the direction and the two-slot delta, and the raw prefix decides none of them.
        assert_eq!(effect(0).locals_read, vec![30]);
        assert_eq!(effect(4).locals_read, vec![300]);
        assert!(effect(4).locals_written.is_empty());
        assert_eq!(effect(2).locals_written, vec![31]);
        assert_eq!(effect(8).locals_written, vec![301]);
        assert!(effect(8).locals_read.is_empty());
        assert_eq!(
            (effect(0).stack_delta, effect(4).stack_delta),
            (Some(2), Some(2)),
            "`lload` pushes two slots, wide or not"
        );
        assert_eq!(
            (effect(2).stack_delta, effect(8).stack_delta),
            (Some(-2), Some(-2)),
            "`lstore` pops two slots, wide or not"
        );
        assert!(
            effects.iter().all(|effect| !effect.may_throw),
            "a local access cannot enter a handler, wide or not"
        );
    }

    #[test]
    fn throw_semantics_cover_the_throwing_categories() {
        for opcode in [
            0x2e, // iaload
            0x35, // saload
            0x4f, // iastore
            0x56, // sastore
            0x6c, // idiv
            0x70, // irem
            0x12, // ldc
            0x14, // ldc2_w
            0xb2, // getstatic
            0xb5, // putfield
            0xb6, // invokevirtual
            0xba, // invokedynamic
            0xbb, // new
            0xbc, // newarray
            0xbe, // arraylength
            0xbf, // athrow
            0xc0, // checkcast
            0xc3, // monitorexit
            OPCODE_MULTIANEWARRAY,
        ] {
            assert!(may_throw(opcode), "{opcode:#04x} may raise");
        }
        for opcode in [
            0x00, // nop
            0x01, // aconst_null
            0x15, // iload
            0x36, // istore
            0x57, // pop
            0x60, // iadd
            0x6e, // fdiv: IEEE 754, no exception
            0x72, // frem
            0x74, // ineg
            0x78, // ishl
            0x84, // iinc
            0x85, // i2l
            0x94, // lcmp
            0x99, // ifeq
            0xa6, // if_acmpne
            0xa7, // goto
            0xa8, // jsr
            0xa9, // ret
            0xac, // ireturn
            0xb1, // return
            0xc4, // wide
            0xc6, // ifnull
            0xc8, // goto_w
            0xc9, // jsr_w
        ] {
            assert!(!may_throw(opcode), "{opcode:#04x} cannot enter a handler");
        }
    }

    /// The opcode numbers of the table are the reader's own (JVMS 6.5): the implicit local
    /// forms are `_0`..`_3` for all five types and `iinc` is `0x84`. A table written from a
    /// different recollection would land the stack ops eight opcodes early, which is what
    /// this test pins down.
    #[test]
    fn the_stack_table_follows_the_readers_opcode_numbering() {
        assert_eq!(
            fixed_stack_delta(0x2a, None),
            Some(1),
            "aload_0 pushes one slot"
        );
        assert_eq!(
            fixed_stack_delta(0x3d, None),
            Some(-1),
            "istore_2 pops one slot"
        );
        assert_eq!(
            fixed_stack_delta(0x4b, None),
            Some(-1),
            "astore_0 pops one slot"
        );
        assert_eq!(
            fixed_stack_delta(0x4f, None),
            Some(-3),
            "iastore pops three slots"
        );
        assert_eq!(
            fixed_stack_delta(0x50, None),
            Some(-4),
            "lastore pops four slots"
        );
        assert_eq!(fixed_stack_delta(0x57, None), Some(-1), "pop");
        assert_eq!(fixed_stack_delta(0x59, None), Some(1), "dup");
        assert_eq!(fixed_stack_delta(0x5f, None), Some(0), "swap");
        assert_eq!(fixed_stack_delta(0x60, None), Some(-1), "iadd");
        assert_eq!(fixed_stack_delta(0x61, None), Some(-2), "ladd");
        assert_eq!(fixed_stack_delta(0x84, None), Some(0), "iinc");
        assert_eq!(fixed_stack_delta(0x85, None), Some(1), "i2l");
        assert_eq!(fixed_stack_delta(0x88, None), Some(-1), "l2i");
        assert_eq!(fixed_stack_delta(0x94, None), Some(-3), "lcmp");
        assert_eq!(fixed_stack_delta(0x99, None), Some(-1), "ifeq");
        assert_eq!(fixed_stack_delta(0x9f, None), Some(-2), "if_icmpeq");
        assert_eq!(fixed_stack_delta(0xa5, None), Some(-2), "if_acmpeq");
        assert_eq!(fixed_stack_delta(0xa7, None), Some(0), "goto");
        assert_eq!(
            fixed_stack_delta(0xa8, None),
            Some(1),
            "jsr pushes the return address"
        );
        assert_eq!(fixed_stack_delta(0xac, None), Some(-1), "ireturn");
        assert_eq!(fixed_stack_delta(0xb1, None), Some(0), "return");
        // D41: `multianewarray` pops one length per dimension and pushes the array, so its delta
        // is `1 - dimensions`, decided from the reader's own operand fact instead of left to a
        // consumer — which is also what lets this table and the frame layer's dense table be
        // checked against each other. Without the fact there is no delta, and none is invented.
        assert_eq!(
            fixed_stack_delta(0xc5, Some(1)),
            Some(0),
            "one dimension: one length popped, the array pushed"
        );
        assert_eq!(
            fixed_stack_delta(0xc5, Some(2)),
            Some(-1),
            "two dimensions: two lengths popped, the array pushed"
        );
        assert_eq!(
            fixed_stack_delta(0xc5, Some(3)),
            Some(-2),
            "three dimensions"
        );
        assert_eq!(
            fixed_stack_delta(0xc5, None),
            None,
            "a count the operands do not carry decides nothing"
        );
    }

    #[test]
    fn the_pass_bills_exactly_its_declared_dimensions() {
        let facts = body(
            vec![
                plain(0, 0x03),     // iconst_0
                branch(1, 0x99, 5), // ifeq +5 -> 6
                plain(4, 0x03),
                plain(5, 0xac),
                plain(6, 0x05),
                plain(7, 0xac),
            ],
            Vec::new(),
            8,
        );
        let mut budget = budget();
        let outcome = raw_cfg(&facts, &mut budget).expect("the fixture is a valid body");
        let usage = budget.usage();
        // Three blocks, no handler record, no throw site and one effect per instruction.
        assert_eq!(usage.ir_items, 3 + 6);
        assert_eq!(
            usage.ir_edges,
            u64::try_from(outcome.cfg.edges.len()).expect("fixture edge count")
        );
        // Six instructions placed into blocks and three reachability pops (the entry, then its
        // two successors).
        assert_eq!(usage.analysis_steps, 6 + 3);
        for dimension in CountedBudgetDimension::ALL {
            if matches!(
                dimension,
                CountedBudgetDimension::IrItems
                    | CountedBudgetDimension::IrEdges
                    | CountedBudgetDimension::AnalysisSteps
            ) {
                continue;
            }
            assert_eq!(
                usage.counted_usage(dimension),
                0,
                "{dimension:?} is not a dimension this pass declares"
            );
        }
    }

    #[test]
    fn the_block_ceiling_stops_the_pass_with_the_effective_limit() {
        // Every `goto` starts its own block, so the body holds one block per instruction.
        let count = MAX_BLOCKS_DEFAULT + 1;
        let mut code = Vec::with_capacity(count);
        for index in 0..count {
            let bci = 3 * u32::try_from(index).expect("fixture BCI");
            let offset = if index + 1 == count { 0 } else { 3 };
            code.push(branch(bci, OPCODE_GOTO, offset));
        }
        let length = 3 * u32::try_from(count).expect("fixture length");
        let facts = body(code, Vec::new(), length);
        let error = raw_cfg(&facts, &mut budget()).expect_err("the ceiling stops the pass");
        assert_eq!(
            error,
            Error::BudgetExceeded {
                dimension: BudgetDimension::IrItems,
                limit: MAX_BLOCKS_DEFAULT as u64,
                consumed: MAX_BLOCKS_DEFAULT as u64,
                requested: 1,
            }
        );
        assert_eq!(
            MAX_BLOCKS_HARD_LIMIT,
            u32::from(u16::MAX),
            "the hard ceiling is the structural maximum of `code_length`"
        );
    }

    #[test]
    fn two_runs_publish_the_same_facts_and_order() {
        let facts = || {
            body(
                vec![
                    plain(0, 0x01),
                    plain(1, 0xbe),
                    plain(2, 0x57),
                    branch(3, 0xa7, 5),
                    plain(6, 0xb1),
                    plain(7, 0x00),
                    plain(8, 0xb1),
                    plain(9, 0x00),
                    plain(10, 0xb1),
                ],
                vec![
                    catch(0, 0, 6, 8, Some(1)),
                    catch(1, 1, 6, 8, Some(2)),
                    catch(2, 1, 6, 10, None),
                ],
                11,
            )
        };
        let mut first_budget = budget();
        let first = raw_cfg(&facts(), &mut first_budget).expect("the fixture is a valid body");
        let mut second_budget = budget();
        let second = raw_cfg(&facts(), &mut second_budget).expect("the fixture is a valid body");
        assert_eq!(first, second, "the same input publishes the same graph");
        for dimension in CountedBudgetDimension::ALL {
            assert_eq!(
                first_budget.usage().counted_usage(dimension),
                second_budget.usage().counted_usage(dimension),
                "{dimension:?} is charged deterministically"
            );
        }
    }
}
