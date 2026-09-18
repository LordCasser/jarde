//! Raw `returnAddress` facts and the call contexts of one decoded method body (3.4).
//!
//! A `jsr`/`jsr_w` pushes a return address — the BCI of the instruction after it — and jumps
//! into a subroutine whose `ret` returns to a call site that entered it. The raw graph of 3.3
//! names that half of the transfer instead of resolving it
//! ([`crate::cfg::RawCfg::unresolved_returns`], [`EdgeKind::SubroutineReturn`]); this module
//! turns that shape into facts a later pass can rebuild edges from:
//!
//! * one [`SubroutineContext`] per `jsr`/`jsr_w` site of the decoded prefix, naming the call
//!   site, the return address it pushes, the entry it jumps to and the locals the subroutine
//!   writes ([`SubroutineContext::affected_locals`]);
//! * where every `ret` returns: one [`SubroutineReturn`] per `ret` of the decoded prefix, with
//!   the call contexts whose bodies own it. A subroutine shared by two call sites therefore has
//!   **two** contexts and its `ret` has **two** return points, and the return point of a `ret`
//!   always comes from the context that owns it instead of from a guess;
//! * the exception records that cross a call: [`ExceptionCoverage`] keeps the records covering
//!   the `jsr` itself and the records covering the subroutine's own body, as exception-table
//!   ordinals. A handler entry is **not** an ordinary successor — the walk never follows an
//!   exception edge — so a handler's own instructions never grow a subroutine's affected
//!   locals, and the records stay where 3.5 can rebuild the edges from.
//!
//! # The walk: sharing, nesting and the handler boundary
//!
//! The context set is *one context per call site*, so a shared subroutine is not copied: each
//! call site owns a context, and the walk from a context's entry collects
//!
//! * the locals its body writes, ascending and deduplicated — including the slot the
//!   subroutine stores its own return address in, which is a write like any other, and the
//!   writes of nested calls, which are part of what the context inlines;
//! * the `ret`s it owns (a `ret` reached while this context is active), which is what makes a
//!   shared `ret` return to several points;
//! * the exception records covering the instructions it walks.
//!
//! A nested `jsr` inside a subroutine is followed **twice, in two directions**: into the nested
//! subroutine under *its own* context (so a nested `ret` belongs to the nested call site and
//! never to the outer one) and into the nested call site's continuation under the outer context
//! (because that is where control resumes once the nested call returns). Neither direction is a
//! guess: both are the two halves of the same `SubroutineReturn` edge.
//!
//! # Reachability
//!
//! Every `jsr`/`jsr_w` site of the decoded prefix gets a context, including the ones 3.3's
//! truth table lists as unreachable ([`CallContexts::unreachable_call_sites`] names those). A
//! context is a *static* fact about a call site — its return address is encoded in the
//! instruction and the locals its body writes do not depend on how the code is entered — while
//! reachability is a separate question the raw graph already answers, and the historical ECJ
//! 4.6 `finally` shape needs both of its call sites (its handler path is entered by the
//! exception table, not by a raw edge) visible to 3.5's one-to-many clone contract.
//!
//! The refusal rules below, in contrast, are scoped to the part the raw graph can reach: a dead
//! call site does not stop the analysis of a live one. A block counts as reachable exactly when
//! 3.3's truth table does not list it.
//!
//! # Dialect, refusal and what must not be faked
//!
//! * `jsr`/`jsr_w`/`ret` in a class file of major version [`FIRST_MODERN_MAJOR`] or above are a
//!   dialect violation (JVMS 4.9.1 keeps them for class files below 51.0): the raw facts stay
//!   exactly as 3.3 built them, no context is published, and the run fails under
//!   [`IR_LEGACY_OPCODE_FORBIDDEN`] so the method cannot enter a canonical CFG. The dialect is
//!   decided by the class-file version alone, and a `wide` form is classified by the opcode it
//!   wraps (0.2), so `wide ret` is exactly as forbidden as `ret` — and a modern method whose
//!   only `wide` forms are loads, stores or `iinc` has nothing to refuse.
//! * a call site whose return address is not an instruction start of the decoded prefix (a
//!   `jsr` at the end of a body whose decode stopped early), a reachable call site whose body
//!   owns no `ret` (its return address is never used, so the return half of its transfer has no
//!   owner), a reachable `ret` no context owns, and call sites that nest through each other in a
//!   cycle (the nesting has no bound, so no finite context set describes it) are all reported as
//!   [`IR_CALL_CONTEXT_UNRESOLVED`]: the raw facts are kept, the context fact is **not**
//!   published, and no call graph is invented.
//! * a body with neither `jsr`/`jsr_w` nor `ret` has the empty context set as its complete
//!   answer, and the pass charges nothing for it.
//!
//! # Billing and determinism
//!
//! The pass bills exactly the dimension its descriptor declares (`Steps` = `AnalysisSteps`),
//! and only for work it really does: one step per instruction it looks at while walking a
//! context and one per worklist pop (a repeated visit of a block is charged again). The
//! structural scan that decides whether the method holds a legacy opcode at all reads the
//! reader's facts without charging, like the scans of 3.3's graph.
//!
//! Every published `Vec<_>` is sorted on its own coordinates — contexts and coverage by call
//! site BCI, returns by `ret` BCI with targets by call site, locals and handler ordinals
//! ascending — and the walk's own order is the published edge order of the raw graph, so the
//! same request spends the same budget twice.

use crate::budget::{Budget, CountedBudgetDimension};
use crate::cfg::{
    EdgeKind, EffectFacts, OPCODE_JSR, OPCODE_JSR_W, OPCODE_RET, RawCfg, RawCfgOutcome,
};
use crate::classfile::{ExceptionHandlerFact, MethodCodeFacts};
use crate::error::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};

/// First class-file major version in which `jsr`/`jsr_w`/`ret` are forbidden: the modern
/// dialect starts at 51.
pub(crate) const FIRST_MODERN_MAJOR: u16 = 51;

/// Code of a method that holds `jsr`/`jsr_w`/`ret` in a class file that forbids them.
pub(crate) const IR_LEGACY_OPCODE_FORBIDDEN: &str = "ir_legacy_opcode_forbidden";

/// Code of a call context set that cannot be established, with the raw facts kept as they are.
pub(crate) const IR_CALL_CONTEXT_UNRESOLVED: &str = "ir_call_context_unresolved";

/// Code of a raw graph that does not line up with the reader's facts.
///
/// Unreachable while the graph is the one [`crate::cfg::raw_cfg`] built from the same facts —
/// the graph places one `SubroutineReturn` edge per call site and a block per target — but a
/// reader of one body has to hear about a disagreement instead of reading a context set that
/// silently dropped a call site.
const IR_CALL_CONTEXT_INCONSISTENT: &str = "ir_call_context_inconsistent";

/// One `jsr`/`jsr_w` call context: the raw return address and what its subroutine writes.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SubroutineContext {
    /// BCI of the `jsr`/`jsr_w` instruction.
    pub(crate) call_site_bci: u32,
    /// BCI of the instruction after it: the return address the `jsr` pushes, validated to be an
    /// instruction start of the decoded prefix.
    pub(crate) return_bci: u32,
    /// BCI of the subroutine entry: the target the `jsr` jumps to.
    pub(crate) entry_bci: u32,
    /// Locals the subroutine's body writes, ascending and deduplicated. This includes the slot
    /// the subroutine stores the return address in and the locals a nested call writes.
    pub(crate) affected_locals: Vec<u16>,
}

/// One call context a `ret` can return to.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct CallTarget {
    /// BCI of the `jsr` whose context owns the `ret`.
    pub(crate) call_site_bci: u32,
    /// That context's return address: where this `ret` returns for this call site.
    pub(crate) return_bci: u32,
}

/// Where one `ret` of the decoded prefix can return.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SubroutineReturn {
    /// BCI of the `ret` instruction.
    pub(crate) ret_bci: u32,
    /// The contexts whose bodies own it, by call site BCI, ascending; empty when no context
    /// does (a `ret` in a block the raw graph already lists as unreachable).
    pub(crate) targets: Vec<CallTarget>,
}

/// The exception records that cross one call context.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExceptionCoverage {
    /// BCI of the `jsr`/`jsr_w` this record belongs to.
    pub(crate) call_site_bci: u32,
    /// Exception-table ordinals whose protected range covers the `jsr` itself, ascending: the
    /// record of a protected range that spans the call.
    pub(crate) call_site_handlers: Vec<u32>,
    /// Exception-table ordinals whose protected range covers an instruction of the subroutine's
    /// own body, ascending. A handler entry is not a successor of those instructions — the walk
    /// does not follow exception edges — and the ordinal is kept so 3.5 can rebuild the edges
    /// for the context's blocks.
    pub(crate) subroutine_handlers: Vec<u32>,
}

/// The `CallContexts` fact: every call context of one method body, where its `ret`s return and
/// which exception records cross it.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CallContexts {
    /// One context per `jsr`/`jsr_w` site of the decoded prefix, by call site BCI.
    pub(crate) contexts: Vec<SubroutineContext>,
    /// One entry per `ret` of the decoded prefix, by `ret` BCI.
    pub(crate) returns: Vec<SubroutineReturn>,
    /// One entry per context, by call site BCI.
    pub(crate) exception_coverage: Vec<ExceptionCoverage>,
    /// Call sites whose block 3.3's truth table lists as unreachable, by BCI. They have a
    /// context like every other call site; this list names the ones the raw graph cannot enter.
    pub(crate) unreachable_call_sites: Vec<u32>,
}

/// What one run of the pass produced.
#[derive(Debug)]
pub(crate) enum CallContextOutcome {
    /// The context set of this body, complete for the decoded prefix.
    Established(CallContexts),
    /// The dialect forbids the opcodes the body holds: no context, and the method must not be
    /// canonicalized. Reported as a failure, not as a limitation.
    Forbidden { message: String },
    /// The context set cannot be established: the raw facts are kept, no call graph is faked.
    /// Reported as a partial run under [`IR_CALL_CONTEXT_UNRESOLVED`].
    Unresolved { message: String },
}

/// Builds the call contexts of one decoded method body over the raw graph of the same facts.
///
/// `raw` is the outcome of [`crate::cfg::raw_cfg`] for exactly these `facts` (the engine keeps it
/// while the later passes run), and `major_version` is the class file's own version, which is
/// the only thing that decides the dialect.
///
/// Charges one `AnalysisSteps` per instruction examined in a walked block and one per worklist
/// pop, both before the work they describe, and nothing else: the declared dimension set of the
/// pass is `[Steps]`.
pub(crate) fn call_contexts(
    facts: &MethodCodeFacts,
    raw: &RawCfgOutcome,
    major_version: u16,
    budget: &mut Budget,
) -> Result<CallContextOutcome> {
    debug_assert_eq!(
        facts.instructions.len(),
        raw.effects.instructions.len(),
        "effect facts are produced in lockstep with instructions"
    );
    let legacy = legacy_opcodes(facts);

    // The dialect first: a forbidden opcode is a certainty about the version, not a limitation
    // of this build, and every legacy opcode is classified by the opcode it is — a `wide ret`
    // reaches this arm as `ret`, exactly like the short form.
    if major_version >= FIRST_MODERN_MAJOR
        && let Some((bci, opcode)) = legacy.opcodes.first()
    {
        return Ok(CallContextOutcome::Forbidden {
            message: format!(
                "the class file's major version {major_version} forbids `{}`, and the decoded \
                 prefix holds it at BCI {bci}: the raw facts are kept, but this method builds no \
                 call contexts and must not be canonicalized",
                opcode_name(*opcode)
            ),
        });
    }
    if legacy.jsr_sites.is_empty() && legacy.returns.is_empty() {
        // No `jsr`/`jsr_w` and no `ret`: the empty context set is this body's complete answer,
        // and there is no walk to bill for it.
        return Ok(CallContextOutcome::Established(empty_contexts()));
    }

    let cfg = &raw.cfg;
    let blocks = block_bcis(cfg);
    if blocks.is_empty() {
        // A body whose decoded prefix holds a `jsr`/`ret` always has at least the block that
        // holds it; a graph without one cannot describe these facts.
        return Err(inconsistent(
            "the raw graph holds no block for a method body whose prefix holds a `jsr`/`ret`",
        ));
    }
    let entries = call_edges(cfg);
    let mut plans = Vec::with_capacity(legacy.jsr_sites.len());
    for call_site_bci in &legacy.jsr_sites {
        let index = instruction_index(facts, *call_site_bci)?;
        let return_bci = call_site_bci
            .checked_add(facts.instructions[index].width)
            .ok_or_else(|| inconsistent("a `jsr` return address overflows the BCI space"))?;
        if facts
            .instructions
            .binary_search_by_key(&return_bci, |instruction| instruction.bci)
            .is_err()
        {
            return Ok(CallContextOutcome::Unresolved {
                message: format!(
                    "the instruction after the `jsr` at BCI {call_site_bci} is not part of the \
                     decoded prefix (the return address would be BCI {return_bci}): the return \
                     point of that call site is not in this body, so no context is established \
                     for it"
                ),
            });
        }
        let entry_bci = entries.get(call_site_bci).copied().ok_or_else(|| {
            inconsistent(format!(
                "the raw graph holds no call edge for the `jsr` at BCI {call_site_bci}"
            ))
        })?;
        plans.push(CallSitePlan {
            call_site_bci: *call_site_bci,
            return_bci,
            entry_bci,
        });
    }

    let successors = successors(cfg, &blocks)?;
    let ranges = instruction_ranges(facts, &blocks)?;
    let mut walk = Walk::new(
        facts,
        cfg,
        &raw.effects,
        &blocks,
        &ranges,
        &successors,
        &plans,
    );
    for root in 0..plans.len() {
        walk.run(root, budget)?;
    }
    let Walk {
        affected,
        returns,
        coverage,
        contains,
        ..
    } = walk;

    if let Some(cycle) = nesting_cycle(&contains) {
        let sites = cycle
            .iter()
            .map(|index| plans[*index].call_site_bci.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Ok(CallContextOutcome::Unresolved {
            message: format!(
                "the `jsr` call sites at BCI {sites} call each other in a cycle: the nesting of \
                 this method has no bound, so no finite context set describes its return \
                 addresses"
            ),
        });
    }
    if let Some(plan) = plans
        .iter()
        .enumerate()
        .find(|(index, plan)| {
            reachable(cfg, &blocks, plan.call_site_bci)
                && !returns.values().any(|owners| owners.contains(index))
        })
        .map(|(_, plan)| plan)
    {
        return Ok(CallContextOutcome::Unresolved {
            message: format!(
                "the `jsr` at BCI {} jumps to the subroutine at BCI {}, whose reachable body owns \
                 no `ret`: the return address it pushes is never used, so the return half of that \
                 transfer has no owner and the call graph is not established",
                plan.call_site_bci, plan.entry_bci
            ),
        });
    }
    if let Some(ret_bci) = legacy
        .returns
        .iter()
        .find(|ret_bci| reachable(cfg, &blocks, **ret_bci) && !returns.contains_key(ret_bci))
    {
        return Ok(CallContextOutcome::Unresolved {
            message: format!(
                "the `ret` at BCI {ret_bci} is not reached by any `jsr` call context: the return \
                 address it consumes has no call site, so the call graph is not established"
            ),
        });
    }

    Ok(CallContextOutcome::Established(assemble(
        cfg,
        &blocks,
        &plans,
        &legacy.returns,
        affected,
        returns,
        coverage,
    )))
}

/// A context set that says exactly nothing: a body without `jsr`/`jsr_w`/`ret`.
fn empty_contexts() -> CallContexts {
    CallContexts {
        contexts: Vec::new(),
        returns: Vec::new(),
        exception_coverage: Vec::new(),
        unreachable_call_sites: Vec::new(),
    }
}

/// The legacy opcodes of one decoded prefix, in BCI order.
struct LegacyOpcodes {
    /// Every `jsr`/`jsr_w`/`ret` of the prefix as `(bci, effective opcode)`, ascending.
    ///
    /// A `wide ret` is one of them: the reader classifies a `wide` form by the opcode it wraps
    /// (0.2), so the `ret` a `wide` prefix encodes is the same forbidden opcode here.
    opcodes: Vec<(u32, u8)>,
    /// The `jsr`/`jsr_w` sites alone, ascending.
    jsr_sites: Vec<u32>,
    /// The `ret` instructions alone, ascending.
    returns: Vec<u32>,
}

/// The opcodes this pass is about, read from the instruction stream alone: the reader's
/// effective opcode, never the raw one, because `0xc4` is a prefix and not a legacy opcode.
fn legacy_opcodes(facts: &MethodCodeFacts) -> LegacyOpcodes {
    let mut opcodes = Vec::new();
    let mut jsr_sites = Vec::new();
    let mut returns = Vec::new();
    for (instruction, operands) in facts.instructions.iter().zip(facts.operands.iter()) {
        match operands.effective_opcode {
            OPCODE_JSR | OPCODE_JSR_W | OPCODE_RET => {
                opcodes.push((instruction.bci, operands.effective_opcode));
                if operands.effective_opcode == OPCODE_RET {
                    returns.push(instruction.bci);
                } else {
                    jsr_sites.push(instruction.bci);
                }
            }
            _ => {}
        }
    }
    LegacyOpcodes {
        opcodes,
        jsr_sites,
        returns,
    }
}

/// One call site with its validated return address and entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CallSitePlan {
    call_site_bci: u32,
    return_bci: u32,
    entry_bci: u32,
}

fn inconsistent(message: impl Into<String>) -> Error {
    Error::invalid_input(IR_CALL_CONTEXT_INCONSISTENT, message)
}

/// The entry of every call site, from the raw graph's `SubroutineReturn` edges.
fn call_edges(cfg: &RawCfg) -> BTreeMap<u32, u32> {
    let mut entries = BTreeMap::new();
    for edge in &cfg.edges {
        if let EdgeKind::SubroutineReturn { call_site } = edge.kind {
            entries.insert(call_site, edge.to_bci);
        }
    }
    entries
}

fn block_bcis(cfg: &RawCfg) -> Vec<u32> {
    cfg.blocks.iter().map(|block| block.bci).collect()
}

/// Position of the block that holds one BCI, or of the block that starts there.
///
/// A block covers the half-open range `bci..end_bci`, and a `jsr`/`ret` cannot sit before the
/// first block: the position is the last block whose start is at or below the BCI.
fn block_of(blocks: &[u32], bci: u32) -> Result<usize> {
    blocks
        .partition_point(|start| *start <= bci)
        .checked_sub(1)
        .ok_or_else(|| inconsistent(format!("BCI {bci} lies before the first raw block")))
}

/// Whether 3.3's truth table says the entry can reach the block holding one BCI.
fn reachable(cfg: &RawCfg, blocks: &[u32], bci: u32) -> bool {
    match block_of(blocks, bci) {
        Ok(position) => cfg.unreachable.binary_search(&blocks[position]).is_err(),
        Err(_) => false,
    }
}

/// The successor edges of every block, by block position, in the published edge order.
fn successors(cfg: &RawCfg, blocks: &[u32]) -> Result<Vec<Vec<(EdgeKind, usize)>>> {
    let mut successors = vec![Vec::new(); blocks.len()];
    for edge in &cfg.edges {
        let from = blocks
            .binary_search(&edge.from_bci)
            .map_err(|_| inconsistent(format!("BCI {} is not a raw block", edge.from_bci)))?;
        let to = blocks
            .binary_search(&edge.to_bci)
            .map_err(|_| inconsistent(format!("BCI {} is not a raw block", edge.to_bci)))?;
        successors[from].push((edge.kind, to));
    }
    Ok(successors)
}

/// The instruction index range of every block, in the block order.
fn instruction_ranges(facts: &MethodCodeFacts, blocks: &[u32]) -> Result<Vec<(usize, usize)>> {
    let mut starts = Vec::with_capacity(blocks.len());
    for bci in blocks {
        starts.push(instruction_index(facts, *bci)?);
    }
    let mut ranges: Vec<(usize, usize)> =
        starts.windows(2).map(|pair| (pair[0], pair[1])).collect();
    if let Some(last) = starts.last() {
        ranges.push((*last, facts.instructions.len()));
    }
    Ok(ranges)
}

/// Index of the instruction at one BCI, or the structured inconsistency of a graph that names a
/// BCI the reader did not decode.
fn instruction_index(facts: &MethodCodeFacts, bci: u32) -> Result<usize> {
    facts
        .instructions
        .binary_search_by_key(&bci, |instruction| instruction.bci)
        .map_err(|_| {
            inconsistent(format!(
                "BCI {bci} is not an instruction start of this method body"
            ))
        })
}

/// One context walk: what it writes, which `ret`s it owns and which records cover it.
struct Walk<'a> {
    facts: &'a MethodCodeFacts,
    cfg: &'a RawCfg,
    effects: &'a EffectFacts,
    blocks: &'a [u32],
    ranges: &'a [(usize, usize)],
    successors: &'a [Vec<(EdgeKind, usize)>],
    plans: &'a [CallSitePlan],
    /// Context index of every call site BCI, so a nested call edge can switch contexts.
    context_of: BTreeMap<u32, usize>,
    /// Locals written per context, by context index.
    affected: Vec<BTreeSet<u16>>,
    /// Owners of every walked `ret`, by `ret` BCI, as context indexes.
    returns: BTreeMap<u32, BTreeSet<usize>>,
    /// Exception records covering the instructions walked under one context, by context index.
    coverage: Vec<BTreeSet<u32>>,
    /// Call sites a context's body directly holds, by context index (the nesting relation).
    contains: Vec<BTreeSet<usize>>,
}

impl<'a> Walk<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        facts: &'a MethodCodeFacts,
        cfg: &'a RawCfg,
        effects: &'a EffectFacts,
        blocks: &'a [u32],
        ranges: &'a [(usize, usize)],
        successors: &'a [Vec<(EdgeKind, usize)>],
        plans: &'a [CallSitePlan],
    ) -> Self {
        let context_of = plans
            .iter()
            .enumerate()
            .map(|(index, plan)| (plan.call_site_bci, index))
            .collect();
        let affected: Vec<BTreeSet<u16>> = vec![BTreeSet::new(); plans.len()];
        let coverage: Vec<BTreeSet<u32>> = vec![BTreeSet::new(); plans.len()];
        let contains: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); plans.len()];
        Self {
            facts,
            cfg,
            effects,
            blocks,
            ranges,
            successors,
            plans,
            context_of,
            affected,
            returns: BTreeMap::new(),
            coverage,
            contains,
        }
    }

    /// Walks one context: its entry, everything reachable from it through normal transfers and
    /// nested calls, and the continuations of those nested calls.
    fn run(&mut self, root: usize, budget: &mut Budget) -> Result<()> {
        let mut visited: BTreeMap<usize, Vec<bool>> = BTreeMap::new();
        let mut worklist = vec![(root, block_of(self.blocks, self.plans[root].entry_bci)?)];
        let mut written: BTreeSet<u16> = BTreeSet::new();
        while let Some((active, block)) = worklist.pop() {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            let seen = visited
                .entry(active)
                .or_insert_with(|| vec![false; self.blocks.len()]);
            if seen[block] {
                continue;
            }
            seen[block] = true;
            let (start, end) = self.ranges[block];
            for index in start..end {
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                self.visit(active, index, &mut written);
            }
            for (kind, to) in &self.successors[block] {
                self.step(active, *kind, *to, &mut worklist)?;
            }
        }
        self.affected[root] = written;
        Ok(())
    }

    /// Records one instruction of the walk: a `ret` it holds, its writes and the exception
    /// records covering it.
    fn visit(&mut self, active: usize, index: usize, written: &mut BTreeSet<u16>) {
        let instruction = &self.facts.instructions[index];
        // A `wide ret` is the same return the short form is: the reader classifies a `wide`
        // form by the opcode it wraps (0.2).
        if self.facts.operands[index].effective_opcode == OPCODE_RET {
            self.returns
                .entry(instruction.bci)
                .or_default()
                .insert(active);
        }
        written.extend(
            self.effects.instructions[index]
                .locals_written
                .iter()
                .copied(),
        );
        for ordinal in covering_handlers(&self.cfg.handlers, instruction.bci) {
            self.coverage[active].insert(ordinal);
        }
    }

    /// Follows one edge of the walk.
    fn step(
        &mut self,
        active: usize,
        kind: EdgeKind,
        to: usize,
        worklist: &mut Vec<(usize, usize)>,
    ) -> Result<()> {
        match kind {
            EdgeKind::Normal => worklist.push((active, to)),
            EdgeKind::SubroutineReturn { call_site } => {
                // A nested call: the nested subroutine runs under its own context, and this
                // context resumes at the nested call's continuation once it returns.
                let nested = *self.context_of.get(&call_site).ok_or_else(|| {
                    inconsistent(format!(
                        "the raw graph calls the `jsr` at BCI {call_site}, which is not a call \
                         site of this body"
                    ))
                })?;
                self.contains[active].insert(nested);
                worklist.push((nested, block_of(self.blocks, self.plans[nested].entry_bci)?));
                worklist.push((
                    active,
                    block_of(self.blocks, self.plans[nested].return_bci)?,
                ));
            }
            // A handler entry is not an ordinary successor: the record is kept in the coverage
            // fact, and the walk does not fold the handler's own instructions into the body.
            EdgeKind::Exception { .. } => {}
        }
        Ok(())
    }
}

/// The exception-table ordinals whose protected range covers one BCI, ascending.
fn covering_handlers(handlers: &[ExceptionHandlerFact], bci: u32) -> Vec<u32> {
    handlers
        .iter()
        .filter(|handler| handler.start_bci <= bci && bci < handler.end_bci)
        .map(|handler| handler.ordinal)
        .collect()
}

/// The published context set, with every `Vec<_>` in its own order.
///
/// `returns` is the owner relation the walk collected and `rets` every `ret` of the decoded
/// prefix: the published list keeps one entry per `ret`, so a `ret` no context owns is a fact
/// with an empty target list instead of a missing one.
fn assemble(
    cfg: &RawCfg,
    blocks: &[u32],
    plans: &[CallSitePlan],
    rets: &[u32],
    affected: Vec<BTreeSet<u16>>,
    returns: BTreeMap<u32, BTreeSet<usize>>,
    coverage: Vec<BTreeSet<u32>>,
) -> CallContexts {
    let contexts: Vec<SubroutineContext> = plans
        .iter()
        .zip(affected)
        .map(|(plan, locals)| SubroutineContext {
            call_site_bci: plan.call_site_bci,
            return_bci: plan.return_bci,
            entry_bci: plan.entry_bci,
            affected_locals: locals.into_iter().collect(),
        })
        .collect();
    let returns: Vec<SubroutineReturn> = rets
        .iter()
        .map(|ret_bci| SubroutineReturn {
            ret_bci: *ret_bci,
            targets: returns
                .get(ret_bci)
                .into_iter()
                .flatten()
                .map(|index| CallTarget {
                    call_site_bci: plans[*index].call_site_bci,
                    return_bci: plans[*index].return_bci,
                })
                .collect(),
        })
        .collect();
    let exception_coverage: Vec<ExceptionCoverage> = plans
        .iter()
        .zip(coverage)
        .map(|(plan, handlers)| ExceptionCoverage {
            call_site_bci: plan.call_site_bci,
            call_site_handlers: covering_handlers(&cfg.handlers, plan.call_site_bci),
            subroutine_handlers: handlers.into_iter().collect(),
        })
        .collect();
    let unreachable_call_sites = plans
        .iter()
        .map(|plan| plan.call_site_bci)
        .filter(|bci| !reachable(cfg, blocks, *bci))
        .collect();
    CallContexts {
        contexts,
        returns,
        exception_coverage,
        unreachable_call_sites,
    }
}

/// One cycle of the nesting relation, as context indexes, or `None` when it is acyclic.
///
/// A call site that contains itself is a cycle: a subroutine that calls itself has an unbounded
/// stack of return addresses and cannot be described by a finite context set.
fn nesting_cycle(contains: &[BTreeSet<usize>]) -> Option<Vec<usize>> {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    let children: Vec<Vec<usize>> = contains
        .iter()
        .map(|set| set.iter().copied().collect())
        .collect();
    let mut colour = vec![WHITE; children.len()];
    // The search is iterative: `path` is the current path (all grey) and `cursor` the child
    // index each of its frames is at, so a deep nesting cannot overflow the stack.
    let mut path: Vec<usize> = Vec::new();
    let mut cursor: Vec<usize> = Vec::new();
    for start in 0..children.len() {
        if colour[start] != WHITE {
            continue;
        }
        colour[start] = GREY;
        path.push(start);
        cursor.push(0);
        while let Some(frame) = path.len().checked_sub(1) {
            let node = path[frame];
            if cursor[frame] < children[node].len() {
                let next = children[node][cursor[frame]];
                cursor[frame] += 1;
                match colour[next] {
                    GREY => {
                        let at = path
                            .iter()
                            .position(|visited| *visited == next)
                            .expect("a grey node is on the current path");
                        return Some(path[at..].to_vec());
                    }
                    BLACK => {}
                    _ => {
                        colour[next] = GREY;
                        path.push(next);
                        cursor.push(0);
                    }
                }
            } else {
                colour[node] = BLACK;
                path.pop();
                cursor.pop();
            }
        }
    }
    None
}

/// Stable name of one legacy opcode, for the diagnostic of a forbidden dialect.
fn opcode_name(opcode: u8) -> &'static str {
    match opcode {
        OPCODE_JSR => "jsr",
        OPCODE_JSR_W => "jsr_w",
        OPCODE_RET => "ret",
        _ => "a legacy opcode",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::{BudgetDimension, Limits, UsageSnapshot};
    use crate::cfg::{CfgCompleteness, RawBlock, RawCfgOutcome, RawEdge, raw_cfg};
    use crate::classfile::{
        BytecodeStop, InstructionFact, InstructionOperands, LocalOperand, class_facts,
        method_code_facts,
    };
    use crate::model::{ByteSpan, ExecutionReport, TerminationReason};

    /// Class-file offset the fixture bodies start at: the facts and the graph carry BCIs, but
    /// the spans must still be coherent.
    const CODE_OFFSET: u64 = 200;

    /// The committed historical fixtures: one source compiled to every dialect of the jsr era.
    const V45: &[u8] =
        include_bytes!("../tests/fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class");
    const V46: &[u8] =
        include_bytes!("../tests/fixtures/historical/ecj-4.6.1/v46/HistoricalControlFlow.class");
    const V47: &[u8] =
        include_bytes!("../tests/fixtures/historical/ecj-4.6.1/v47/HistoricalControlFlow.class");
    const V48: &[u8] =
        include_bytes!("../tests/fixtures/historical/ecj-4.6.1/v48/HistoricalControlFlow.class");
    const V52: &[u8] =
        include_bytes!("../tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

    fn limits() -> Limits {
        Limits {
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
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

    /// An operandless instruction (`nop`, `return`, …).
    fn plain(bci: u32, opcode: u8) -> (InstructionFact, InstructionOperands) {
        instruction(bci, opcode, 1, operands(opcode))
    }

    /// A `jsr` whose relative offset points `offset` bytes ahead of its own BCI.
    fn jsr(bci: u32, offset: i32) -> (InstructionFact, InstructionOperands) {
        instruction(
            bci,
            OPCODE_JSR,
            3,
            InstructionOperands {
                branch_offset: Some(offset),
                ..operands(OPCODE_JSR)
            },
        )
    }

    /// A `ret` of one local, in the short form.
    fn ret(bci: u32, index: u16) -> (InstructionFact, InstructionOperands) {
        instruction(bci, OPCODE_RET, 2, local(index, false, OPCODE_RET))
    }

    /// A store of one local, whose named operand is what makes it a write (the `_0`..`_3`
    /// forms name their local too, exactly like the reader records them).
    fn store(bci: u32, opcode: u8, index: u16) -> (InstructionFact, InstructionOperands) {
        instruction(bci, opcode, 1, local(index, false, opcode))
    }

    /// A load of one local with a byte operand, which is what makes it a read.
    fn load(bci: u32, opcode: u8, index: u16) -> (InstructionFact, InstructionOperands) {
        instruction(bci, opcode, 2, local(index, false, opcode))
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

    /// A `wide`-prefixed local access: one event of the reader, with the prefix as the raw
    /// opcode and the wrapped opcode as the one the classifications read.
    fn wide_local(
        bci: u32,
        wrapped_opcode: u8,
        index: u16,
    ) -> (InstructionFact, InstructionOperands) {
        instruction(bci, 0xc4, 4, local(index, true, wrapped_opcode))
    }

    /// A `wide iload` of one local.
    fn wide_load(bci: u32, index: u16) -> (InstructionFact, InstructionOperands) {
        /// `iload`.
        const ILOAD: u8 = 0x15;
        wide_local(bci, ILOAD, index)
    }

    /// A `wide istore` of one local.
    fn wide_store(bci: u32, index: u16) -> (InstructionFact, InstructionOperands) {
        /// `istore`.
        const ISTORE: u8 = 0x36;
        wide_local(bci, ISTORE, index)
    }

    /// A `wide ret` of one local.
    fn wide_ret(bci: u32, index: u16) -> (InstructionFact, InstructionOperands) {
        wide_local(bci, OPCODE_RET, index)
    }

    /// An `iinc` of one local.
    fn iinc(bci: u32, index: u16, value: i32) -> (InstructionFact, InstructionOperands) {
        /// `iinc`.
        const IINC: u8 = 0x84;
        instruction(
            bci,
            IINC,
            3,
            InstructionOperands {
                local: Some(LocalOperand { index, wide: false }),
                increment: Some(value),
                ..operands(IINC)
            },
        )
    }

    /// A `wide iinc` of one local.
    fn wide_iinc(bci: u32, index: u16, value: i32) -> (InstructionFact, InstructionOperands) {
        /// `iinc`.
        const IINC: u8 = 0x84;
        instruction(
            bci,
            0xc4,
            6,
            InstructionOperands {
                local: Some(LocalOperand { index, wide: true }),
                increment: Some(value),
                ..operands(IINC)
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
        let (instructions, operands) = code.into_iter().unzip();
        MethodCodeFacts {
            max_stack: 8,
            max_locals: 8,
            code_span: ByteSpan::new(CODE_OFFSET, u64::from(code_length)),
            instructions,
            operands,
            exception_handler_count: u32::try_from(handlers.len()).expect("fixture handlers"),
            exception_handlers: handlers,
            execution: ExecutionReport::Complete {
                usage: UsageSnapshot::default(),
            },
            stopped_at: None,
        }
    }

    /// The same body as one whose decode stopped after the decoded prefix.
    fn truncated(body: MethodCodeFacts, stop_bci: u32, code_length: u32) -> MethodCodeFacts {
        MethodCodeFacts {
            code_span: ByteSpan::new(CODE_OFFSET, u64::from(code_length)),
            stopped_at: Some(BytecodeStop::Instructions {
                bci: stop_bci,
                class_offset: CODE_OFFSET + u64::from(stop_bci),
                code: "classfile_bytecode_budget_exceeded".to_string(),
            }),
            execution: ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::CodeBytes,
                },
                usage: UsageSnapshot::default(),
            },
            ..body
        }
    }

    fn graph(facts: &MethodCodeFacts, budget: &mut Budget) -> RawCfgOutcome {
        raw_cfg(facts, budget).expect("the fixture is a valid body")
    }

    /// The established context set of one fixture, or a failure naming the outcome.
    fn established(
        facts: &MethodCodeFacts,
        raw: &RawCfgOutcome,
        major_version: u16,
    ) -> CallContexts {
        match call_contexts(facts, raw, major_version, &mut budget()) {
            Ok(CallContextOutcome::Established(contexts)) => contexts,
            Ok(CallContextOutcome::Forbidden { message }) => {
                panic!("expected contexts, got a forbidden dialect: {message}")
            }
            Ok(CallContextOutcome::Unresolved { message }) => {
                panic!("expected contexts, got an unresolved call graph: {message}")
            }
            Err(error) => panic!("expected contexts, got {error}"),
        }
    }

    /// The refusal of one fixture, as `(code, message)`.
    fn refusal(
        facts: &MethodCodeFacts,
        raw: &RawCfgOutcome,
        major_version: u16,
    ) -> (String, String) {
        match call_contexts(facts, raw, major_version, &mut budget()) {
            Ok(CallContextOutcome::Established(contexts)) => {
                panic!(
                    "expected a refusal, got {} contexts",
                    contexts.contexts.len()
                )
            }
            Ok(CallContextOutcome::Forbidden { message }) => {
                (IR_LEGACY_OPCODE_FORBIDDEN.to_string(), message)
            }
            Ok(CallContextOutcome::Unresolved { message }) => {
                (IR_CALL_CONTEXT_UNRESOLVED.to_string(), message)
            }
            Err(error) => panic!("expected a refusal, got {error}"),
        }
    }

    /// The decoded facts of one method of the committed historical fixtures.
    fn historical(bytes: &[u8], name: &[u8]) -> (MethodCodeFacts, u16) {
        let mut budget = budget();
        let facts = class_facts(bytes, &mut budget).expect("the fixture is a class file");
        let member = facts
            .methods
            .iter()
            .find(|member| member.name.raw().0.as_slice() == name)
            .expect("the fixture declares the method");
        let decoded =
            method_code_facts(bytes, member, &mut budget).expect("the fixture's body decodes");
        (decoded, facts.major_version)
    }

    #[test]
    fn the_historical_finally_path_keeps_both_call_sites_of_its_shared_subroutine() {
        // The ECJ 4.6 `finallyPath(I)I` of every jsr-era dialect: the return value is saved,
        // `jsr` enters one subroutine from BCI 5 (the return path) and from BCI 12 (the
        // handler path), and the single `ret` of that subroutine returns to both call sites.
        // The handler path is entered by the exception table, not by a raw edge, so the second
        // call site is *unreachable* for the raw graph — and it still gets its own context,
        // because a context is a static fact of the call site and 3.5 needs both to clone the
        // subroutine per context.
        for (version, bytes) in [(45, V45), (46, V46), (47, V47), (48, V48)] {
            let (facts, major) = historical(bytes, b"finallyPath");
            assert_eq!(major, version);
            let raw = graph(&facts, &mut budget());
            assert_eq!(raw.cfg.unresolved_returns, vec![5, 12]);
            let contexts = established(&facts, &raw, major);

            assert_eq!(
                contexts.contexts,
                vec![
                    SubroutineContext {
                        call_site_bci: 5,
                        return_bci: 8,
                        entry_bci: 17,
                        affected_locals: vec![1, 2],
                    },
                    SubroutineContext {
                        call_site_bci: 12,
                        return_bci: 15,
                        entry_bci: 17,
                        affected_locals: vec![1, 2],
                    },
                ],
                "classfile major {version}"
            );
            assert_eq!(
                contexts.returns,
                vec![SubroutineReturn {
                    ret_bci: 21,
                    targets: vec![
                        CallTarget {
                            call_site_bci: 5,
                            return_bci: 8,
                        },
                        CallTarget {
                            call_site_bci: 12,
                            return_bci: 15,
                        },
                    ],
                }],
                "the one `ret` returns to both call sites, each with its own return address \
                 (classfile major {version})"
            );
            assert_eq!(
                contexts.exception_coverage,
                vec![
                    ExceptionCoverage {
                        call_site_bci: 5,
                        call_site_handlers: vec![0],
                        subroutine_handlers: Vec::new(),
                    },
                    ExceptionCoverage {
                        call_site_bci: 12,
                        call_site_handlers: Vec::new(),
                        subroutine_handlers: Vec::new(),
                    },
                ],
                "the protected range [0, 8) spans the `jsr` at BCI 5 and only it \
                 (classfile major {version})"
            );
            assert_eq!(
                contexts.unreachable_call_sites,
                vec![12],
                "3.3's truth table cannot enter the handler path \
                 (classfile major {version})"
            );
        }
    }

    #[test]
    fn the_modern_dialect_of_the_same_source_has_no_call_context() {
        // 52 compiles the same source with the `finally` inlined: no `jsr`, no `ret`, and the
        // empty context set is the complete answer — charged as nothing. The fixture's opcodes
        // are read the way the pass reads them: a `wide` form is the opcode it wraps.
        let (facts, major) = historical(V52, b"finallyPath");
        assert!(
            facts
                .operands
                .iter()
                .all(|operands| operands.effective_opcode != OPCODE_JSR
                    && operands.effective_opcode != OPCODE_JSR_W
                    && operands.effective_opcode != OPCODE_RET),
            "the modern fixture holds no legacy opcode, wide or not"
        );
        let raw = graph(&facts, &mut budget());
        let mut budget = budget();
        let outcome = call_contexts(&facts, &raw, major, &mut budget).expect("no opcode to refuse");
        assert!(matches!(
            outcome,
            CallContextOutcome::Established(CallContexts { contexts, returns, .. })
                if contexts.is_empty() && returns.is_empty()
        ));
        assert_eq!(budget.usage().analysis_steps, 0);
    }

    #[test]
    fn a_shared_entry_keeps_one_context_per_call_site() {
        // Two calls into one subroutine: `astore_1` stores the return address a caller pushed,
        // `iinc 2, 1` writes a second local, `istore 5` writes a local nothing reads in the
        // subroutine, and `iload 7` reads a local it never writes. Both contexts name the same
        // *written* locals — `1`, `2` and `5`, and emphatically not the read-only `7` — and each
        // keeps its own return address.
        let facts = body(
            vec![
                jsr(0, 9),          // -> BCI 9, return address 3
                jsr(3, 6),          // -> BCI 9, return address 6
                plain(6, 0xb1),     // return
                plain(7, 0x00),     // nop
                plain(8, 0x00),     // nop
                store(9, 0x4c, 1),  // astore_1
                iinc(10, 2, 1),     // iinc 2, 1
                load(13, 0x15, 7),  // iload 7
                store(15, 0x36, 5), // istore 5
                ret(17, 1),
            ],
            Vec::new(),
            19,
        );
        let raw = graph(&facts, &mut budget());
        let contexts = established(&facts, &raw, 45);
        assert_eq!(
            contexts.contexts,
            vec![
                SubroutineContext {
                    call_site_bci: 0,
                    return_bci: 3,
                    entry_bci: 9,
                    affected_locals: vec![1, 2, 5],
                },
                SubroutineContext {
                    call_site_bci: 3,
                    return_bci: 6,
                    entry_bci: 9,
                    affected_locals: vec![1, 2, 5],
                },
            ]
        );
        assert_eq!(
            contexts.returns,
            vec![SubroutineReturn {
                ret_bci: 17,
                targets: vec![
                    CallTarget {
                        call_site_bci: 0,
                        return_bci: 3,
                    },
                    CallTarget {
                        call_site_bci: 3,
                        return_bci: 6,
                    },
                ],
            }]
        );
        assert!(contexts.unreachable_call_sites.is_empty());
    }

    #[test]
    fn a_nested_call_returns_to_its_own_continuation() {
        // The outer subroutine (entry 9) calls a second one (entry 16). The nested `ret` at
        // BCI 20 belongs to the nested call site and returns to *its* continuation (13), while
        // the outer `ret` at 13 returns to BCI 3: a return point taken from the first
        // instruction, from the outermost context or from the entry would be a guess.
        let facts = body(
            vec![
                jsr(0, 9),          // outer call -> 9, return address 3
                plain(3, 0xb1),     // return
                plain(4, 0x00),     // nop
                plain(5, 0x00),     // nop
                plain(6, 0x00),     // nop
                plain(7, 0x00),     // nop
                plain(8, 0x00),     // nop
                store(9, 0x4c, 1),  // astore_1 (outer return address)
                jsr(10, 6),         // nested call -> 16, return address 13
                ret(13, 1),         // outer `ret`
                plain(15, 0x00),    // nop
                store(16, 0x4d, 2), // astore_2 (nested return address)
                iinc(17, 3, 1),
                ret(20, 2), // nested `ret`
            ],
            Vec::new(),
            22,
        );
        let raw = graph(&facts, &mut budget());
        let contexts = established(&facts, &raw, 45);
        assert_eq!(
            contexts.contexts,
            vec![
                SubroutineContext {
                    call_site_bci: 0,
                    return_bci: 3,
                    entry_bci: 9,
                    affected_locals: vec![1, 2, 3],
                },
                SubroutineContext {
                    call_site_bci: 10,
                    return_bci: 13,
                    entry_bci: 16,
                    affected_locals: vec![2, 3],
                },
            ],
            "the outer context covers the nested body it inlines, the nested one only its own"
        );
        assert_eq!(
            contexts.returns,
            vec![
                SubroutineReturn {
                    ret_bci: 13,
                    targets: vec![CallTarget {
                        call_site_bci: 0,
                        return_bci: 3,
                    }],
                },
                SubroutineReturn {
                    ret_bci: 20,
                    targets: vec![CallTarget {
                        call_site_bci: 10,
                        return_bci: 13,
                    }],
                },
            ],
            "a nested `ret` returns to the nested call site, not to the outer one"
        );
    }

    #[test]
    fn a_handler_entry_is_no_successor_and_its_range_is_recorded() {
        // The protected range [0, 11) spans the `jsr` at BCI 0 and covers the `invokevirtual`
        // at BCI 8, so the handler at BCI 13 is a real entry — but not an ordinary successor of
        // the subroutine: the walk keeps the record and does not fold `astore_2` into the
        // subroutine's affected locals.
        let facts = body(
            vec![
                jsr(0, 7),         // -> BCI 7, return address 3
                plain(3, 0xb1),    // return
                plain(4, 0x00),    // nop
                plain(5, 0x00),    // nop
                plain(6, 0x00),    // nop
                store(7, 0x4c, 1), // astore_1
                instruction(
                    8,
                    0xb6,
                    3,
                    InstructionOperands {
                        constant_pool_index: Some(20),
                        ..operands(0xb6)
                    },
                ), // invokevirtual: may throw, covered by record 0
                ret(11, 1),
                store(13, 0x4d, 2), // astore_2 (handler entry)
                plain(14, 0xb1),    // return
            ],
            vec![catch(0, 0, 11, 13, None)],
            15,
        );
        let raw = graph(&facts, &mut budget());
        assert!(
            raw.cfg
                .edges
                .iter()
                .any(|edge| matches!(edge.kind, EdgeKind::Exception { handler_ordinal: 0 })),
            "the fixture's throw site reaches the handler"
        );
        let contexts = established(&facts, &raw, 45);
        assert_eq!(
            contexts.contexts,
            vec![SubroutineContext {
                call_site_bci: 0,
                return_bci: 3,
                entry_bci: 7,
                affected_locals: vec![1],
            }],
            "the handler's own local is not part of the subroutine"
        );
        assert_eq!(
            contexts.exception_coverage,
            vec![ExceptionCoverage {
                call_site_bci: 0,
                call_site_handlers: vec![0],
                subroutine_handlers: vec![0],
            }]
        );
    }

    #[test]
    fn a_wide_local_access_is_classified_by_the_opcode_it_wraps() {
        // `wide` is a prefix (JVMS 6.5), not an instruction: the wrapped opcode decides. A
        // `wide istore` writes its local, a `wide iinc` reads and writes its own and a
        // `wide ret` is the return of its context — none of them was classifiable while the
        // reader kept only the prefix, and all of them behave here like the short forms.
        let facts = body(
            vec![
                jsr(0, 3),           // -> BCI 3, return address 3
                store(3, 0x4c, 1),   // astore_1: the return address
                wide_store(4, 300),  // wide istore 300
                wide_load(8, 7),     // wide iload 7: a read, so not an affected local
                wide_iinc(12, 2, 1), // wide iinc 2, 1: reads and writes local 2
                wide_ret(18, 1),     // wide ret 1: the return of this context
            ],
            Vec::new(),
            22,
        );
        let raw = graph(&facts, &mut budget());
        let contexts = established(&facts, &raw, 45);
        assert_eq!(
            contexts.contexts,
            vec![SubroutineContext {
                call_site_bci: 0,
                return_bci: 3,
                entry_bci: 3,
                affected_locals: vec![1, 2, 300],
            }],
            "the wide store, the wide `iinc` and the return address are the written locals; \
             the wide load's local 7 is only read"
        );
        assert_eq!(
            contexts.returns,
            vec![SubroutineReturn {
                ret_bci: 18,
                targets: vec![CallTarget {
                    call_site_bci: 0,
                    return_bci: 3,
                }],
            }],
            "the `wide ret` returns to the call site exactly like `ret` does"
        );
    }

    #[test]
    fn a_modern_body_classifies_its_wide_forms_by_the_opcode_they_wrap() {
        // 0.2 closed a real gap here: a modern method whose only `wide` forms are loads,
        // stores and `iinc` holds no legacy opcode at all, so the empty context set is its
        // complete answer — it must not be refused as an undecidable `wide` boundary.
        let facts = body(
            vec![
                store(0, 0x4b, 0),  // astore_0
                wide_load(1, 300),  // wide iload 300
                wide_iinc(5, 2, 1), // wide iinc 2, 1
                plain(11, 0xb1),    // return
            ],
            Vec::new(),
            12,
        );
        let raw = graph(&facts, &mut budget());
        let contexts = established(&facts, &raw, 52);
        assert!(
            contexts.contexts.is_empty() && contexts.returns.is_empty(),
            "a modern method with wide loads, stores and `iinc` has no legacy opcode to report"
        );
        assert_eq!(budget().usage().analysis_steps, 0);

        // The same classification makes a `wide ret` the legacy opcode this dialect forbids:
        // the dialect violation is reported before any walk, and the identical facts under
        // the 45 dialect are the `ret`-without-owner limitation instead — the version, not
        // the `wide` prefix, is what separates the two answers.
        let facts = body(
            vec![store(0, 0x4b, 0), wide_ret(1, 0), plain(5, 0xb1)],
            Vec::new(),
            6,
        );
        let raw = graph(&facts, &mut budget());
        let (legacy_code, legacy_message) = refusal(&facts, &raw, 45);
        assert_eq!(
            legacy_code, IR_CALL_CONTEXT_UNRESOLVED,
            "under the dialect that keeps `ret`, the wide return is a `ret` with no owner"
        );
        assert!(
            legacy_message.contains("`ret` at BCI 1"),
            "{legacy_message}"
        );

        let (code, message) = refusal(&facts, &raw, 52);
        assert_eq!(code, IR_LEGACY_OPCODE_FORBIDDEN, "a `wide ret` is a `ret`");
        assert!(
            message.contains("forbids `ret`") && message.contains("BCI 1"),
            "{message}"
        );
    }

    #[test]
    fn a_modern_method_of_real_bytes_with_only_wide_local_accesses_has_no_context() {
        // The same two cases as above, on real bytes and through the real reader (0.2's gap):
        // a modern method whose `wide` forms are a load, a store and an `iinc` holds no legacy
        // opcode at all, so the empty context set is its complete answer and the pass charges
        // nothing for it.
        let code = [
            0xc4, 0x15, 0x00, 0x00, // wide iload 0
            0xc4, 0x36, 0x00, 0x00, // wide istore 0
            0xc4, 0x84, 0x00, 0x00, 0x00, 0x01, // wide iinc 0, 1
            0xb1, // return
        ];
        let bytes = crate::classfile::test_class::single_method(52, 8, 8, &code);
        let (facts, major) = historical(&bytes, b"method");
        assert_eq!(major, 52);
        assert_eq!(
            facts
                .instructions
                .iter()
                .map(|instruction| instruction.opcode)
                .collect::<Vec<_>>(),
            vec![0xc4, 0xc4, 0xc4, 0xb1],
            "the reader keeps the raw prefix of every wide form"
        );
        let raw = graph(&facts, &mut budget());
        let contexts = established(&facts, &raw, major);
        assert!(
            contexts.contexts.is_empty() && contexts.returns.is_empty(),
            "a modern body of wide loads, stores and `iinc` has no legacy opcode to refuse"
        );
        assert_eq!(
            budget().usage().analysis_steps,
            0,
            "the structural scan decides this without charging a step"
        );
    }

    #[test]
    fn a_wide_ret_of_real_bytes_is_the_dialect_violation_of_its_version() {
        // `wide ret` is a `ret`: a class file of major 51 or above forbids it, exactly like the
        // short form, and the same bytes at 45 are the `ret`-without-owner limitation instead.
        // The body also holds a `wide iload`, so the violation cannot come from a body that is
        // otherwise unrecognized: classification, not the prefix, decides.
        let code = [
            0xc4, 0x15, 0x00, 0x00, // wide iload 0
            0xc4, 0xa9, 0x00, 0x00, // wide ret 0
            0xb1, // return
        ];
        for major in [51u16, 52] {
            let bytes = crate::classfile::test_class::single_method(major, 8, 8, &code);
            let (facts, header_major) = historical(&bytes, b"method");
            assert_eq!(header_major, major);
            let raw = graph(&facts, &mut budget());
            let (failure, message) = refusal(&facts, &raw, major);
            assert_eq!(failure, IR_LEGACY_OPCODE_FORBIDDEN, "major {major}");
            assert!(
                message.contains("forbids `ret`") && message.contains("BCI 4"),
                "major {major}: {message}"
            );
        }

        // The dialect that keeps `ret` reads the same instruction as a `ret` no call site owns.
        let bytes = crate::classfile::test_class::single_method(45, 8, 8, &code);
        let (facts, header_major) = historical(&bytes, b"method");
        assert_eq!(header_major, 45);
        let raw = graph(&facts, &mut budget());
        let (failure, message) = refusal(&facts, &raw, 45);
        assert_eq!(failure, IR_CALL_CONTEXT_UNRESOLVED);
        assert!(message.contains("`ret` at BCI 4"), "{message}");
    }

    #[test]
    fn a_modern_class_file_with_a_legacy_opcode_is_forbidden() {
        // The dialect is the class-file version alone: the same bytes of the 45 fixture under
        // major version 51 are a violation, and one that also carries a `wide` form still
        // reports the violation, because the dialect is decided before any walk.
        let mut patched = V45.to_vec();
        patched[6..8].copy_from_slice(&51u16.to_be_bytes());
        let (facts, major) = historical(&patched, b"finallyPath");
        assert_eq!(major, 51);
        let raw = graph(&facts, &mut budget());
        let (code, message) = refusal(&facts, &raw, major);
        assert_eq!(code, IR_LEGACY_OPCODE_FORBIDDEN);
        assert!(
            message.contains("51") && message.contains("jsr") && message.contains("BCI 5"),
            "{message}"
        );

        let facts = body(
            vec![
                jsr(0, 4),
                plain(3, 0x00),
                store(4, 0x4c, 1),
                wide_load(5, 300),
                ret(9, 1),
            ],
            Vec::new(),
            11,
        );
        let raw = graph(&facts, &mut budget());
        let (code, _) = refusal(&facts, &raw, 51);
        assert_eq!(
            code, IR_LEGACY_OPCODE_FORBIDDEN,
            "the forbidden opcode is reported before anything reads a wide form"
        );

        // A `ret` alone is a violation too: it is the return half of the forbidden call pair.
        let facts = body(vec![store(0, 0x4b, 0), ret(1, 0)], Vec::new(), 3);
        let raw = graph(&facts, &mut budget());
        let (code, message) = refusal(&facts, &raw, 52);
        assert_eq!(code, IR_LEGACY_OPCODE_FORBIDDEN);
        assert!(
            message.contains("ret") && message.contains("BCI 1"),
            "{message}"
        );
    }

    #[test]
    fn a_call_site_whose_body_owns_no_ret_is_unresolved() {
        // The subroutine returns through a plain `return`: the return address the `jsr` pushed
        // is never consumed, so the return half of that transfer has no owner. The raw facts
        // are the answer that survives, and no context is published.
        let facts = body(
            vec![
                jsr(0, 3),         // -> BCI 3, return address 3
                store(3, 0x4c, 1), // astore_1
                plain(4, 0xb1),    // return
            ],
            Vec::new(),
            5,
        );
        let raw = graph(&facts, &mut budget());
        let (code, message) = refusal(&facts, &raw, 45);
        assert_eq!(code, IR_CALL_CONTEXT_UNRESOLVED);
        assert!(
            message.contains("`jsr` at BCI 0") && message.contains("BCI 3"),
            "{message}"
        );
    }

    #[test]
    fn a_truncated_body_whose_ret_was_not_decoded_is_unresolved() {
        // The same shape with the `ret` in the unread suffix: the decoded prefix is the
        // reader's reliable one, so the context cannot be established over it, and the reader's
        // own stop stays the reason the facts are partial.
        let facts = truncated(
            body(vec![jsr(0, 3), store(3, 0x4c, 1)], Vec::new(), 6),
            4,
            6,
        );
        let raw = graph(&facts, &mut budget());
        assert!(!raw.cfg.completeness.is_complete());
        let (code, _) = refusal(&facts, &raw, 45);
        assert_eq!(code, IR_CALL_CONTEXT_UNRESOLVED);
    }

    #[test]
    fn a_ret_no_call_context_owns_is_unresolved() {
        // A `ret` the method's own flow reaches without any `jsr`: the local it consumes holds
        // no return address of this method, so the call graph is not established.
        let facts = body(vec![store(0, 0x4b, 0), ret(1, 0)], Vec::new(), 3);
        let raw = graph(&facts, &mut budget());
        let (code, message) = refusal(&facts, &raw, 45);
        assert_eq!(code, IR_CALL_CONTEXT_UNRESOLVED);
        assert!(message.contains("`ret` at BCI 1"), "{message}");
    }

    #[test]
    fn call_sites_that_nest_through_each_other_are_unresolved() {
        // A subroutine that calls itself: every level pushes another return address, so the
        // nesting has no bound and no finite context set describes it.
        let facts = body(
            vec![
                jsr(0, 3),         // -> BCI 3
                jsr(3, 0),         // -> BCI 3 (itself)
                store(6, 0x4c, 1), // astore_1
                ret(7, 1),
            ],
            Vec::new(),
            9,
        );
        let raw = graph(&facts, &mut budget());
        let (code, message) = refusal(&facts, &raw, 45);
        assert_eq!(code, IR_CALL_CONTEXT_UNRESOLVED);
        assert!(
            message.contains("cycle") && message.contains("BCI 3"),
            "{message}"
        );
    }

    #[test]
    fn a_body_whose_prefix_holds_a_legacy_opcode_needs_the_graph_to_place_it() {
        // The guard of the payload's own preconditions: a graph without the call edge of a
        // `jsr` (or without any block at all) is a disagreement to report, not a context set
        // that silently dropped a call site.
        let facts = body(vec![jsr(0, 3), store(3, 0x4b, 0), ret(4, 0)], Vec::new(), 6);
        let real = graph(&facts, &mut budget());
        let without_edge = RawCfgOutcome {
            cfg: RawCfg {
                blocks: vec![
                    RawBlock { bci: 0, end_bci: 3 },
                    RawBlock { bci: 3, end_bci: 6 },
                ],
                edges: Vec::new(),
                throw_sites: Vec::new(),
                handlers: Vec::new(),
                unreachable: Vec::new(),
                completeness: CfgCompleteness::Complete,
                unresolved_returns: vec![0],
            },
            effects: EffectFacts {
                instructions: real.effects.instructions.clone(),
            },
        };
        let error = call_contexts(&facts, &without_edge, 45, &mut budget())
            .expect_err("a call site without its edge is an inconsistency");
        assert!(
            error
                .to_string()
                .contains("no call edge for the `jsr` at BCI 0"),
            "{error}"
        );

        let without_block = RawCfgOutcome {
            cfg: RawCfg {
                blocks: Vec::new(),
                edges: Vec::new(),
                throw_sites: Vec::new(),
                handlers: Vec::new(),
                unreachable: Vec::new(),
                completeness: CfgCompleteness::Complete,
                unresolved_returns: vec![0],
            },
            effects: EffectFacts {
                instructions: real.effects.instructions.clone(),
            },
        };
        assert!(
            call_contexts(&facts, &without_block, 45, &mut budget())
                .expect_err("a body with a `jsr` needs a block")
                .to_string()
                .contains("holds no block"),
        );

        // A `RawEdge` the graph cannot place in a block is the same kind of disagreement.
        let stray_edge = RawCfgOutcome {
            cfg: RawCfg {
                blocks: vec![
                    RawBlock { bci: 0, end_bci: 3 },
                    RawBlock { bci: 3, end_bci: 6 },
                ],
                edges: vec![RawEdge {
                    from_bci: 0,
                    to_bci: 99,
                    kind: EdgeKind::SubroutineReturn { call_site: 0 },
                }],
                throw_sites: Vec::new(),
                handlers: Vec::new(),
                unreachable: Vec::new(),
                completeness: CfgCompleteness::Complete,
                unresolved_returns: vec![0],
            },
            effects: EffectFacts {
                instructions: real.effects.instructions.clone(),
            },
        };
        assert!(
            call_contexts(&facts, &stray_edge, 45, &mut budget())
                .expect_err("an edge to an unknown block is an inconsistency")
                .to_string()
                .contains("is not a raw block"),
        );
    }

    #[test]
    fn the_walk_bills_analysis_steps_and_nothing_else() {
        // The declared dimension set of the pass is `[Steps]`: the walk charges steps, and the
        // dimensions `raw_cfg` owns (`IrItems`/`IrEdges`) stay untouched here even though their
        // limits are zero.
        let facts = body(
            vec![
                jsr(0, 9),
                jsr(3, 6),
                plain(6, 0xb1),
                plain(7, 0x00),
                plain(8, 0x00),
                store(9, 0x4c, 1),
                iinc(10, 2, 1),
                ret(13, 1),
            ],
            Vec::new(),
            15,
        );
        let raw = graph(&facts, &mut budget());
        let mut budget = Budget::new(Limits {
            class_bytes: 0,
            code_bytes: 0,
            ir_items: 0,
            ir_edges: 0,
            analysis_steps: 1_000,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        assert!(matches!(
            call_contexts(&facts, &raw, 45, &mut budget),
            Ok(CallContextOutcome::Established(_))
        ));
        let usage = budget.usage();
        assert!(usage.analysis_steps > 0);
        assert_eq!(usage.ir_items, 0, "the walk creates no IR item of raw_cfg");
        assert_eq!(usage.ir_edges, 0, "the walk adds no edge");
    }

    #[test]
    fn an_exhausted_step_budget_stops_the_walk() {
        let facts = body(
            vec![
                jsr(0, 9),
                jsr(3, 6),
                plain(6, 0xb1),
                plain(7, 0x00),
                plain(8, 0x00),
                store(9, 0x4c, 1),
                iinc(10, 2, 1),
                ret(13, 1),
            ],
            Vec::new(),
            15,
        );
        let raw = graph(&facts, &mut budget());
        let mut budget = Budget::new(Limits {
            analysis_steps: 3,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        let error = call_contexts(&facts, &raw, 45, &mut budget)
            .expect_err("three steps cannot cover two contexts");
        assert!(
            matches!(error, Error::BudgetExceeded { .. }),
            "the stop names the dimension: {error}"
        );
        assert!(budget.usage().analysis_steps <= 3);
        assert_eq!(budget.usage().ir_items, 0);
    }
}
