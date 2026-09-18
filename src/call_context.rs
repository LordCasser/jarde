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
//! The pass bills exactly the dimensions its descriptor declares
//! (`[Blocks, Steps]`, and `Blocks` is `IrItems` + `IrEdges`), and only for work it really
//! does.
//!
//! The derived storage of this pass — derivation and assembly alike — is billed one item per
//! storage item, **before** it is added:
//!
//! * one `IrItems` per `jsr`/`jsr_w` site (its `CallSitePlan` and, once assembled, its
//!   [`SubroutineContext`] and [`ExceptionCoverage`]), per call-site entry of the raw graph,
//!   per element of the per-context sets the walk grows (the affected locals, the owners of a
//!   `ret`, the covering exception records and the nesting relation) and per node of the cycle
//!   search's state;
//! * one `IrEdges` per derived edge of the successor lists;
//! * one `IrItems` per block **per context** for the walk's own `visited` matrix. That is the
//!   product this pass has to bound: `Vec<bool>` is one byte per entry, so a body of 3 000
//!   contexts over 3 000 blocks would hold ~9 MB of visited flags, and a budget that charges
//!   the matrix only once per context — or not at all — cannot refuse it;
//! * one `AnalysisSteps` per instruction looked at while walking a context, per worklist pop,
//!   per worklist enqueue and per frame the cycle search pushes (a repeated visit of a block is
//!   charged again);
//! * the empty context set of a body with no `jsr`/`jsr_w`/`ret` is charged nothing.
//!
//! Every phase — the plans and the entry map, the successor lists, the instruction ranges, the
//! walk's own construction, the cycle search and the assembly — also polls for cancellation
//! (billing polls by itself, and the phases without a billing point poll explicitly, through
//! [`checkpoint`]), so a cancelled or exhausted request stops there instead of finishing the
//! assembly first. A stop keeps the raw facts as they are, publishes **no** [`CallContexts`] and
//! does not truncate the payload of a run that is allowed to finish. The structural scan that
//! decides whether the method holds a legacy opcode at all reads the reader's facts without
//! charging, like the scans of 3.3's graph.
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
use std::collections::{BTreeMap, BTreeSet, btree_map};

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
/// Charges the declared dimension set of the pass, `[Blocks, Steps]`: one `IrItems` per derived
/// storage item it builds (the plans, the call-site entries, every element of the per-context
/// sets, one row of the walk's `visited` matrix per block per context), one `IrEdges` per
/// derived successor edge, and one `AnalysisSteps` per instruction examined in a walked block,
/// per worklist pop, per worklist enqueue and per frame of the cycle search — all before the
/// storage item or the step they describe. Every phase of the run polls for cancellation, so an
/// exhausted or cancelled request stops without publishing a context set.
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
    let entries = call_edges(cfg, budget)?;
    let mut plans = Vec::with_capacity(legacy.jsr_sites.len());
    // One context per call site is derived storage of this pass: the plan itself and, in the
    // entry map, the call-site node `call_edges` already billed.
    checkpoint(Phase::Plans, budget)?;
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
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        plans.push(CallSitePlan {
            call_site_bci: *call_site_bci,
            return_bci,
            entry_bci,
        });
    }

    let successors = successors(cfg, &blocks, budget)?;
    let ranges = instruction_ranges(facts, &blocks, budget)?;
    let mut walk = Walk::new(
        facts,
        cfg,
        &raw.effects,
        &blocks,
        &ranges,
        &successors,
        &plans,
        budget,
    )?;
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

    if let Some(cycle) = nesting_cycle(&contains, budget)? {
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

    let walked = Walked {
        affected,
        returns,
        coverage,
    };
    Ok(CallContextOutcome::Established(assemble(
        cfg,
        &blocks,
        &plans,
        &legacy.returns,
        walked,
        budget,
    )?))
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
///
/// The map is derived storage of this pass, so each entry it really adds is billed as `IrItems`
/// before it is inserted.
fn call_edges(cfg: &RawCfg, budget: &mut Budget) -> Result<BTreeMap<u32, u32>> {
    let mut entries = BTreeMap::new();
    for edge in &cfg.edges {
        if let EdgeKind::SubroutineReturn { call_site } = edge.kind {
            if !entries.contains_key(&call_site) {
                budget.charge(CountedBudgetDimension::IrItems, 1)?;
            }
            entries.insert(call_site, edge.to_bci);
        }
    }
    Ok(entries)
}

fn block_bcis(cfg: &RawCfg) -> Vec<u32> {
    cfg.blocks.iter().map(|block| block.bci).collect()
}

/// Position of the block that holds one BCI, or of the block that starts there.
///
/// A block covers the half-open range `bci..end_bci`, and a `jsr`/`ret` cannot sit before the
/// first block: the position is the last block whose start is at or below the BCI. The reading
/// itself is [`crate::cfg::block_of`]'s — the crate has one BCI→block rule (0.4), so a `jsr`
/// that is not the first instruction of its block is placed the same way by this pass and by the
/// raw graph — and this wrapper only names that case with the code of *this* pass.
fn block_of(blocks: &[u32], bci: u32) -> Result<usize> {
    crate::cfg::block_of(blocks, bci, |start| *start)
        .ok_or_else(|| inconsistent(format!("BCI {bci} lies before the first raw block")))
}

/// Whether 3.3's truth table says the entry can reach the block holding one BCI.
fn reachable(cfg: &RawCfg, blocks: &[u32], bci: u32) -> bool {
    match block_of(blocks, bci) {
        Ok(position) => cfg.unreachable.binary_search(&blocks[position]).is_err(),
        Err(_) => false,
    }
}

/// One phase of a run of this pass, named where [`checkpoint`] is called, so the phases are read
/// from the code instead of being re-listed by a test.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    /// The `jsr` site plans and the entry map's derived nodes.
    Plans,
    /// The `IrEdges` successor lists of the raw graph.
    Successors,
    /// The instruction ranges of the blocks.
    InstructionRanges,
    /// The construction of the walk's own state.
    Walk,
    /// One context's `visited` row: the product state this pass has to bound.
    Visited,
    /// The cycle search's own nodes and frames.
    CycleSearch,
    /// The assembly of the published fact.
    Assembly,
}

#[cfg(test)]
/// Every phase, so a test can put its seam on each one in turn.
///
/// The list is beside the enum on purpose: a phase left out of it would keep its checkpoint
/// unproven, and the sweep that uses it would pass without ever reaching the new phase.
const PHASES: [Phase; 7] = [
    Phase::Plans,
    Phase::Successors,
    Phase::InstructionRanges,
    Phase::Walk,
    Phase::Visited,
    Phase::CycleSearch,
    Phase::Assembly,
];

/// The published fact's own charge: the last thing a successful run pays for.
///
/// The fact a caller receives is derived storage too, and publishing it is the only point where
/// a stopped run could still hand out a payload it did not pay for, so the charge is made once,
/// in `IrItems`, after the fact exists.
fn published_item(budget: &mut Budget) -> Result<()> {
    budget.charge(CountedBudgetDimension::IrItems, 1)?;
    Ok(())
}

#[cfg(test)]
thread_local! {
    /// The phase a test cancels at, and whether the checkpoint has already fired.
    static CHECKPOINT: std::cell::Cell<Option<Phase>> = const { std::cell::Cell::new(None) };
}

/// Cancels the run at the next checkpoint of one phase, and holds the seam until the test ends.
///
/// A cancellation cannot be injected from outside into one synchronous call, so this is how a
/// test stops the run between two phases of the pass; the seam is inert for every run that
/// does not use it.
#[cfg(test)]
fn checkpoint_seam(phase: Phase) -> CheckpointSeam {
    CHECKPOINT.with(|slot| slot.set(Some(phase)));
    CheckpointSeam
}

/// Clears one test's seam when the test ends, so the next test starts inert.
#[cfg(test)]
struct CheckpointSeam;

#[cfg(test)]
impl Drop for CheckpointSeam {
    fn drop(&mut self) {
        CHECKPOINT.with(|slot| slot.set(None));
    }
}

/// Polls the run's own stop condition at one phase of the pass.
///
/// This is the place of the pass that has no billing point of its own: every phase either charges
/// (`Budget::charge` polls by itself) or reaches this, so a cancelled or exhausted run stops at a
/// phase boundary instead of finishing the assembly. The stop is the ordinary
/// `Error::Cancelled`/`Error::BudgetExceeded` the rest of the crate reports, and a stopped run
/// publishes no [`CallContexts`].
fn checkpoint(phase: Phase, budget: &Budget) -> Result<()> {
    #[cfg(test)]
    if CHECKPOINT.with(|slot| slot.get()) == Some(phase) {
        CHECKPOINT.with(|slot| slot.set(None));
        budget.cancellation_token().cancel();
    }
    let _ = phase;
    budget.poll()
}

/// The successor edges of every block, by block position, in the published edge order.
///
/// The lists are derived storage of this pass — one derived edge per raw edge — so each is
/// billed as `IrEdges` before it is pushed, and the phase polls before it builds the lists.
fn successors(
    cfg: &RawCfg,
    blocks: &[u32],
    budget: &mut Budget,
) -> Result<Vec<Vec<(EdgeKind, usize)>>> {
    checkpoint(Phase::Successors, budget)?;
    let mut successors = vec![Vec::new(); blocks.len()];
    for edge in &cfg.edges {
        let from = blocks
            .binary_search(&edge.from_bci)
            .map_err(|_| inconsistent(format!("BCI {} is not a raw block", edge.from_bci)))?;
        let to = blocks
            .binary_search(&edge.to_bci)
            .map_err(|_| inconsistent(format!("BCI {} is not a raw block", edge.to_bci)))?;
        budget.charge(CountedBudgetDimension::IrEdges, 1)?;
        successors[from].push((edge.kind, to));
    }
    Ok(successors)
}

/// The instruction index range of every block, in the block order.
///
/// The ranges are derived storage, and the phase polls for cancellation before it starts.
fn instruction_ranges(
    facts: &MethodCodeFacts,
    blocks: &[u32],
    budget: &mut Budget,
) -> Result<Vec<(usize, usize)>> {
    checkpoint(Phase::InstructionRanges, budget)?;
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
    /// Builds the walk's own state.
    ///
    /// The per-context sets this construction allocates start empty and are billed element by
    /// element as the walk grows them ([`Walk::visit`] and [`Walk::step`]), so the
    /// construction itself only checks the run's stop condition.
    #[allow(clippy::too_many_arguments)]
    fn new(
        facts: &'a MethodCodeFacts,
        cfg: &'a RawCfg,
        effects: &'a EffectFacts,
        blocks: &'a [u32],
        ranges: &'a [(usize, usize)],
        successors: &'a [Vec<(EdgeKind, usize)>],
        plans: &'a [CallSitePlan],
        budget: &Budget,
    ) -> Result<Self> {
        checkpoint(Phase::Walk, budget)?;
        let context_of = plans
            .iter()
            .enumerate()
            .map(|(index, plan)| (plan.call_site_bci, index))
            .collect();
        let affected: Vec<BTreeSet<u16>> = vec![BTreeSet::new(); plans.len()];
        let coverage: Vec<BTreeSet<u32>> = vec![BTreeSet::new(); plans.len()];
        let contains: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); plans.len()];
        Ok(Self {
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
        })
    }

    /// Walks one context: its entry, everything reachable from it through normal transfers and
    /// nested calls, and the continuations of those nested calls.
    ///
    /// The `visited` matrix is the product this pass must bound, and it is billed as such: one
    /// `IrItems` per block **per context** whose row the walk allocates, plus one for the map
    /// entry that holds the row, before the row is allocated. A body that is walked under 3 000
    /// contexts therefore charges 3 000 rows instead of one matrix, which is what makes the
    /// ~9 MB of `Vec<bool>` refusable. Worklist pops and enqueues are `AnalysisSteps`.
    fn run(&mut self, root: usize, budget: &mut Budget) -> Result<()> {
        let mut visited: BTreeMap<usize, Vec<bool>> = BTreeMap::new();
        let mut worklist: Vec<(usize, usize)> = Vec::new();
        self.enqueue(
            &mut worklist,
            (root, block_of(self.blocks, self.plans[root].entry_bci)?),
            budget,
        )?;
        let mut written: BTreeSet<u16> = BTreeSet::new();
        while let Some((active, block)) = worklist.pop() {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            let seen = match visited.entry(active) {
                btree_map::Entry::Occupied(entry) => entry.into_mut(),
                btree_map::Entry::Vacant(entry) => {
                    // The row is the product state this pass bounds: `blocks.len()` items for
                    // the row and one for the map entry that holds it, charged before the
                    // allocation itself.
                    checkpoint(Phase::Visited, budget)?;
                    budget.charge(
                        CountedBudgetDimension::IrItems,
                        1 + u64::try_from(self.blocks.len())
                            .expect("a body cannot hold more blocks than the BCI space"),
                    )?;
                    entry.insert(vec![false; self.blocks.len()])
                }
            };
            if seen[block] {
                continue;
            }
            seen[block] = true;
            let (start, end) = self.ranges[block];
            for index in start..end {
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                self.visit(active, index, &mut written, budget)?;
            }
            for (kind, to) in &self.successors[block] {
                self.step(active, *kind, *to, &mut worklist, budget)?;
            }
        }
        self.affected[root] = written;
        Ok(())
    }

    /// Enqueues one worklist entry, billed as one `AnalysisSteps` before it is pushed.
    fn enqueue(
        &self,
        worklist: &mut Vec<(usize, usize)>,
        entry: (usize, usize),
        budget: &mut Budget,
    ) -> Result<()> {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        worklist.push(entry);
        Ok(())
    }

    /// Records one instruction of the walk: a `ret` it holds, its writes and the exception
    /// records covering it.
    ///
    /// Everything the walk stores is derived storage, so each element it **really grows** a set
    /// by is billed as `IrItems` before the insertion: one owner per `(ret, context)` pair, one
    /// local the context writes, one exception record covering the context's body.
    fn visit(
        &mut self,
        active: usize,
        index: usize,
        written: &mut BTreeSet<u16>,
        budget: &mut Budget,
    ) -> Result<()> {
        let bci = self.facts.instructions[index].bci;
        // A `wide ret` is the same return the short form is: the reader classifies a `wide`
        // form by the opcode it wraps (0.2).
        if self.facts.operands[index].effective_opcode == OPCODE_RET {
            let owners = self.returns.entry(bci).or_default();
            if !owners.contains(&active) {
                budget.charge(CountedBudgetDimension::IrItems, 1)?;
                owners.insert(active);
            }
        }
        for local in &self.effects.instructions[index].locals_written {
            if !written.contains(local) {
                budget.charge(CountedBudgetDimension::IrItems, 1)?;
                written.insert(*local);
            }
        }
        for ordinal in covering_handlers(&self.cfg.handlers, bci) {
            if !self.coverage[active].contains(&ordinal) {
                budget.charge(CountedBudgetDimension::IrItems, 1)?;
                self.coverage[active].insert(ordinal);
            }
        }
        Ok(())
    }

    /// Follows one edge of the walk.
    fn step(
        &mut self,
        active: usize,
        kind: EdgeKind,
        to: usize,
        worklist: &mut Vec<(usize, usize)>,
        budget: &mut Budget,
    ) -> Result<()> {
        match kind {
            EdgeKind::Normal => self.enqueue(worklist, (active, to), budget)?,
            EdgeKind::SubroutineReturn { call_site } => {
                // A nested call: the nested subroutine runs under its own context, and this
                // context resumes at the nested call's continuation once it returns.
                let nested = *self.context_of.get(&call_site).ok_or_else(|| {
                    inconsistent(format!(
                        "the raw graph calls the `jsr` at BCI {call_site}, which is not a call \
                         site of this body"
                    ))
                })?;
                if !self.contains[active].contains(&nested) {
                    budget.charge(CountedBudgetDimension::IrItems, 1)?;
                    self.contains[active].insert(nested);
                }
                self.enqueue(
                    worklist,
                    (nested, block_of(self.blocks, self.plans[nested].entry_bci)?),
                    budget,
                )?;
                self.enqueue(
                    worklist,
                    (
                        active,
                        block_of(self.blocks, self.plans[nested].return_bci)?,
                    ),
                    budget,
                )?;
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

/// What the walk produced for every plan, in plan order.
///
/// The collections are keyed by plan index, so they travel together: grouping them keeps
/// [`assemble`] to the arguments it actually decides with, and it makes the invariant they share
/// — the two per-plan lists are as long as the plan list, and the owner map is keyed by the `ret`
/// BCI — a property of one value instead of three parameters that could be passed out of step.
struct Walked {
    /// The locals each context's subroutine wrote, by plan index.
    affected: Vec<BTreeSet<u16>>,
    /// The contexts that reach every `ret`, by `ret` BCI.
    returns: BTreeMap<u32, BTreeSet<usize>>,
    /// The exception records each context covers, by plan index.
    coverage: Vec<BTreeSet<u32>>,
}

/// The published context set, with every `Vec<_>` in its own order.
///
/// `returns` is the owner relation the walk collected and `rets` every `ret` of the decoded
/// prefix: the published list keeps one entry per `ret`, so a `ret` no context owns is a fact
/// with an empty target list instead of a missing one.
///
/// The assembly of the published fact is charged as `IrItems` too: one item per assembled entry
/// (a context, a return, a coverage record, an unreachable call site), which is the storage the
/// assembled fact holds.
///
/// The assembly is also the last phase that can still be stopped: it polls for cancellation
/// before it starts, after every entry it assembles and before it hands the fact out, so a
/// cancelled or exhausted request ends without a context set instead of publishing the payload of
/// a run that was refused.
fn assemble(
    cfg: &RawCfg,
    blocks: &[u32],
    plans: &[CallSitePlan],
    rets: &[u32],
    walked: Walked,
    budget: &mut Budget,
) -> Result<CallContexts> {
    checkpoint(Phase::Assembly, budget)?;
    let mut contexts: Vec<SubroutineContext> = Vec::with_capacity(plans.len());
    for (plan, locals) in plans.iter().zip(walked.affected) {
        checkpoint(Phase::Assembly, budget)?;
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        contexts.push(SubroutineContext {
            call_site_bci: plan.call_site_bci,
            return_bci: plan.return_bci,
            entry_bci: plan.entry_bci,
            affected_locals: locals.into_iter().collect(),
        });
    }
    let mut published_returns: Vec<SubroutineReturn> = Vec::with_capacity(rets.len());
    for ret_bci in rets {
        checkpoint(Phase::Assembly, budget)?;
        let owners = walked.returns.get(ret_bci);
        budget.charge(
            CountedBudgetDimension::IrItems,
            1 + u64::try_from(owners.map_or(0, BTreeSet::len))
                .expect("a body cannot hold more ret owners than contexts"),
        )?;
        published_returns.push(SubroutineReturn {
            ret_bci: *ret_bci,
            targets: owners
                .into_iter()
                .flatten()
                .map(|index| CallTarget {
                    call_site_bci: plans[*index].call_site_bci,
                    return_bci: plans[*index].return_bci,
                })
                .collect(),
        });
    }
    let mut exception_coverage: Vec<ExceptionCoverage> = Vec::with_capacity(plans.len());
    for (plan, handlers) in plans.iter().zip(walked.coverage) {
        checkpoint(Phase::Assembly, budget)?;
        let covering = covering_handlers(&cfg.handlers, plan.call_site_bci);
        budget.charge(
            CountedBudgetDimension::IrItems,
            1 + u64::try_from(covering.len() + handlers.len())
                .expect("a body cannot hold more exception records than the BCI space"),
        )?;
        exception_coverage.push(ExceptionCoverage {
            call_site_bci: plan.call_site_bci,
            call_site_handlers: covering,
            subroutine_handlers: handlers.into_iter().collect(),
        });
    }
    let mut unreachable_call_sites = Vec::new();
    for plan in plans {
        checkpoint(Phase::Assembly, budget)?;
        if !reachable(cfg, blocks, plan.call_site_bci) {
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            unreachable_call_sites.push(plan.call_site_bci);
        }
    }
    published_item(budget)?;
    Ok(CallContexts {
        contexts,
        returns: published_returns,
        exception_coverage,
        unreachable_call_sites,
    })
}

/// One cycle of the nesting relation, as context indexes, or `None` when it is acyclic.
///
/// A call site that contains itself is a cycle: a subroutine that calls itself has an unbounded
/// stack of return addresses and cannot be described by a finite context set.
///
/// The search's own state is derived storage and it is billed for what it really grows: one
/// `IrItems` per context whose colour/child state is allocated, one per edge the child lists
/// copy, and one `AnalysisSteps` per frame pushed onto the path/`cursor` stacks.
fn nesting_cycle(contains: &[BTreeSet<usize>], budget: &mut Budget) -> Result<Option<Vec<usize>>> {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    checkpoint(Phase::CycleSearch, budget)?;
    // The children lists copy every context's edges, and `colour` holds one node, all before
    // the storage exists.
    let edges = u64::try_from(contains.iter().map(BTreeSet::len).sum::<usize>())
        .expect("a body cannot hold more nesting edges than the BCI space");
    let nodes =
        u64::try_from(contains.len()).expect("a body cannot hold more contexts than the BCI");
    budget.charge(CountedBudgetDimension::IrItems, nodes + edges + nodes)?;
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
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
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
                        return Ok(Some(path[at..].to_vec()));
                    }
                    BLACK => {}
                    _ => {
                        colour[next] = GREY;
                        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
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
    Ok(None)
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

    /// A `goto` whose relative offset points `offset` bytes ahead of its own BCI.
    fn goto(bci: u32, offset: i32) -> (InstructionFact, InstructionOperands) {
        /// `goto`.
        const OPCODE_GOTO: u8 = 0xa7;
        instruction(
            bci,
            OPCODE_GOTO,
            3,
            InstructionOperands {
                branch_offset: Some(offset),
                ..operands(OPCODE_GOTO)
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
    fn the_declared_dimensions_of_the_pass_are_ir_items_ir_edges_and_steps() {
        // The declared dimension set of the pass is `[Blocks, Steps]`: the walk charges steps for
        // the instructions it looks at and for every worklist pop/enqueue, and `Blocks` for the
        // storage it derives — the successor lists as edges, the contexts, the per-context sets
        // and the rows of the visited matrix as items. The empty context set of a body with no
        // legacy opcode still charges nothing, which the test below covers.
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
            ir_items: u64::MAX,
            ir_edges: u64::MAX,
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
        assert!(
            usage.ir_items > 0,
            "the derived storage of the pass is billed as items: {usage:?}"
        );
        assert!(
            usage.ir_edges > 0,
            "the successor lists are derived edges, billed as edges: {usage:?}"
        );
    }

    /// The `Limits` of the billing tests: ample headroom everywhere, and one dimension bounded.
    fn limits_bounded(dimension: CountedBudgetDimension, limit: u64) -> Limits {
        let mut limits = Limits {
            class_bytes: 0,
            code_bytes: 0,
            ir_items: 1_000_000,
            ir_edges: 1_000_000,
            analysis_steps: 1_000_000,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        };
        match dimension {
            CountedBudgetDimension::IrItems => limits.ir_items = limit,
            CountedBudgetDimension::IrEdges => limits.ir_edges = limit,
            CountedBudgetDimension::AnalysisSteps => limits.analysis_steps = limit,
            other => panic!("the billing tests bound the pass's own dimensions, not {other:?}"),
        }
        limits
    }

    /// One fixture whose walk touches every billing site the pass has.
    fn billing_fixture() -> MethodCodeFacts {
        body(
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
        )
    }

    /// The usage one successful run of `facts` charges for the dimensions given enough headroom:
    /// the baseline the "just under" and "exactly" bounds are read from.
    fn baseline_usage(facts: &MethodCodeFacts) -> UsageSnapshot {
        let raw = graph(facts, &mut budget());
        let mut budget = budget();
        assert!(matches!(
            call_contexts(facts, &raw, 45, &mut budget),
            Ok(CallContextOutcome::Established(_))
        ));
        budget.usage()
    }

    fn counted(usage: &UsageSnapshot) -> Vec<(CountedBudgetDimension, u64)> {
        CountedBudgetDimension::ALL
            .into_iter()
            .map(|dimension| (dimension, usage.counted_usage(dimension)))
            .collect()
    }

    #[test]
    fn a_zero_item_budget_stops_the_pass_without_publishing_a_context_set() {
        // The R4 counter-example: the raw graph is built with headroom, then the pass is asked to
        // run with `ir_items = 0` and all the steps it wants. Before the derived storage was
        // billed, this run succeeded — the walk allocated its visited matrix, its per-context sets
        // and its published fact without ever asking the item dimension for permission. It must
        // now stop in `IrItems`, keep the raw facts and publish no `CallContexts`.
        let facts = billing_fixture();
        let raw = graph(&facts, &mut budget());
        assert!(
            !raw.cfg.blocks.is_empty(),
            "the fixture has a graph, so the stop is not an empty answer"
        );
        let mut starved = Budget::new(Limits {
            class_bytes: 0,
            code_bytes: 0,
            ir_items: 0,
            ir_edges: 1_000_000,
            analysis_steps: 1_000_000,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        let error = call_contexts(&facts, &raw, 45, &mut starved)
            .expect_err("a zero item budget cannot pay for one context");
        assert!(
            matches!(
                error,
                Error::BudgetExceeded {
                    dimension: BudgetDimension::IrItems,
                    limit: 0,
                    ..
                }
            ),
            "the stop names `IrItems`: {error:?}"
        );
        assert_eq!(
            starved
                .usage()
                .counted_usage(CountedBudgetDimension::IrItems),
            0,
            "a refused item is not consumed"
        );

        // The same stop, funded for everything except the visited rows: `2 contexts * (1 + blocks)`
        // items are withheld from a budget that is otherwise exactly what the run needs, so the
        // run must stop in `IrItems` at the row allocation. This is the case that isolates the
        // matrix: a pass that stopped billing the rows would complete on this budget, because the
        // withheld items are exactly the ones it no longer asks for.
        let complete = baseline_usage(&facts).counted_usage(CountedBudgetDimension::IrItems);
        let blocks = u64::try_from(raw_block_count(&facts)).expect("a small fixture");
        let rows = 2 * (1 + blocks);
        assert!(
            complete > rows,
            "the matrix must be a real part of the bill: {complete} items, {rows} rows"
        );
        let raw = graph(&facts, &mut budget());
        let mut withheld = Budget::new(limits_bounded(
            CountedBudgetDimension::IrItems,
            complete - rows,
        ));
        let error = call_contexts(&facts, &raw, 45, &mut withheld)
            .expect_err("a budget without the visited rows cannot pay for them");
        assert!(
            matches!(
                error,
                Error::BudgetExceeded {
                    dimension: BudgetDimension::IrItems,
                    ..
                }
            ),
            "the stop is the item dimension the matrix is billed in: {error:?}"
        );
    }

    #[test]
    fn the_item_bill_of_the_fixture_matches_its_published_composition() {
        // A boundary test cannot see every charge: a one-item deficit is absorbed by the charges
        // that follow it, so `complete - 1` stops for reasons that do not identify *which* charge
        // was refused — deleting a constant from any single charge leaves that test green. The
        // pinned total is the other half: it was measured on this fixture and is compared here, so
        // a charge that changes its amount moves this number.
        //
        // The fixture is two call-site contexts over five blocks with one `ret`, no handlers and
        // no nesting, and a complete run of it bills 34 items. The number is deliberately not
        // decomposed into per-charge arithmetic: neutralising one charge also changes the charges
        // that follow it, so per-site deltas are not additive and a table of them would be a
        // plausible-looking fiction. The shape assertions below say what the total is read from.
        let facts = billing_fixture();
        let complete = baseline_usage(&facts).counted_usage(CountedBudgetDimension::IrItems);
        assert_eq!(
            complete, 34,
            "the item bill of this fixture is the measured composition above"
        );
        assert_eq!(
            raw_block_count(&facts),
            5,
            "the block count the row charge is read from"
        );
        assert_eq!(
            baseline_contexts(&facts),
            2,
            "the context count the row charge is multiplied by"
        );
    }

    #[test]
    fn one_item_short_of_the_items_stops_the_pass_and_exactly_enough_completes_it() {
        // The bound is exact: with one item less than a complete run charges, some charge of the
        // run is refused; with exactly that many, every charge is accepted and the context set is
        // published entire — the payload is never truncated to fit a budget.
        let facts = billing_fixture();
        let usage = baseline_usage(&facts);
        let charges = usage.counted_usage(CountedBudgetDimension::IrItems);
        assert!(
            charges > 1,
            "the fixture must really charge several items: {charges}"
        );

        // The steps and edges of a run are wired to the items of the same fixture: one item less
        // is refused, wherever in the run the missing item is noticed.
        for (dimension, exact) in counted(&usage).into_iter().filter(|(dimension, _)| {
            matches!(
                dimension,
                CountedBudgetDimension::IrItems
                    | CountedBudgetDimension::IrEdges
                    | CountedBudgetDimension::AnalysisSteps
            )
        }) {
            assert!(exact > 0, "{dimension:?} of a complete run is not zero");
            let raw = graph(&facts, &mut budget());
            let mut short = Budget::new(limits_bounded(dimension, exact - 1));
            let error = call_contexts(&facts, &raw, 45, &mut short)
                .expect_err("one unit less than a complete run is refused");
            assert!(
                matches!(
                    error,
                    Error::BudgetExceeded { dimension: refused, .. } if refused == dimension.into()
                ),
                "{dimension:?}: {error:?}"
            );

            let raw = graph(&facts, &mut budget());
            let mut enough = Budget::new(limits_bounded(dimension, exact));
            assert!(
                matches!(
                    call_contexts(&facts, &raw, 45, &mut enough),
                    Ok(CallContextOutcome::Established(_))
                ),
                "{dimension:?} at exactly the complete run's usage must establish"
            );
            assert_eq!(
                enough.usage().counted_usage(dimension),
                exact,
                "{dimension:?}: the same fixture spends the same budget twice"
            );
        }
    }

    #[test]
    fn the_item_bill_grows_with_the_visited_product_and_not_with_the_outer_vector() {
        // The counter of the ordering rule: `visited` is one row of `blocks.len()` flags **per
        // context**, so the bill is the product of the two and not one charge per outer vector.
        // Two bodies with the same number of contexts but a growing block count must therefore
        // charge strictly more items: a pass that charged the matrix once per context — or once
        // per walk — would charge both of them the same, and could never refuse the ~9 MB matrix
        // of 3 000 contexts over 3 000 blocks.
        let contexts = |extra_blocks: u32| {
            // A chain of `goto`s, one per extra block: each one is a branch target and follows a
            // branch, so each really opens a block of its own between the call sites and the
            // subroutine. Both `jsr` sites enter that same subroutine, so both runs below have the
            // same context count and differ only in the blocks a visited row has to cover.
            let entry = 6 + 3 * extra_blocks;
            let mut code = vec![
                jsr(0, i32::try_from(entry).expect("a small fixture")),
                jsr(3, i32::try_from(entry - 3).expect("a small fixture")),
            ];
            for index in 0..extra_blocks {
                code.push(goto(6 + 3 * index, 3));
            }
            code.push(store(entry, 0x4c, 1));
            code.push(iinc(entry + 1, 2, 1));
            code.push(ret(entry + 4, 1));
            let code_length = entry + 6;
            let facts = body(code, Vec::new(), code_length);
            let blocks = raw_block_count(&facts);
            let usage = baseline_usage(&facts);
            (
                blocks,
                usage.counted_usage(CountedBudgetDimension::IrItems),
                baseline_contexts(&facts),
            )
        };
        let (small_blocks, small_items, small_contexts) = contexts(0);
        let (large_blocks, large_items, large_contexts) = contexts(20);
        assert!(
            large_blocks > small_blocks,
            "the fixture must really grow its block count: {small_blocks} vs {large_blocks}"
        );
        assert_eq!(
            (small_contexts, large_contexts),
            (2, 2),
            "the outer dimension is held fixed: the comparison is two contexts against two"
        );
        assert!(
            large_items > small_items,
            "the item bill must grow with the visited product: {small_items} items over \
             {small_blocks} blocks vs {large_items} items over {large_blocks} blocks"
        );
        // The claim itself, from outside the run that produced it: two contexts over
        // `large_blocks` blocks allocate two rows of that width, so the item bill can never be
        // below the product `contexts * blocks`. A run that charged one item per outer vector
        // instead of one per row would land far below this and could not refuse the ~9 MB matrix
        // of 3 000 contexts.
        let rows = u64::try_from(2 * large_blocks).expect("a small fixture");
        assert!(
            large_items >= rows,
            "the visited product of 2 contexts over {large_blocks} blocks is {rows} items, but \
             the run charged {large_items}"
        );
    }

    /// The number of contexts one fixture establishes, so a comparison can hold that dimension
    /// fixed while another one grows.
    fn baseline_contexts(facts: &MethodCodeFacts) -> usize {
        let raw = graph(facts, &mut budget());
        established(facts, &raw, 45).contexts.len()
    }

    #[test]
    fn the_item_bill_charges_the_elements_of_a_context_set_and_not_the_outer_vector() {
        // The other half of the ordering rule: a per-context set is billed **per element**, not
        // once for the vector that holds it. Two bodies with the same one call site, the same
        // instruction layout and the same block count — they differ only in how many locals the
        // subroutine writes, because a one-byte `nop` and a one-byte `astore`/`istore` occupy the
        // same slot — must therefore charge a different number of items, exactly one per written
        // local. A pass that charged the affected-locals set once per outer vector would charge
        // both runs the same and could never refuse a context whose set is huge.
        let variant = |writes: u32| {
            assert!(writes <= 4, "the fixture has four one-byte slots");
            let mut code = vec![jsr(0, 6), plain(3, 0xb1), plain(4, 0x00), plain(5, 0x00)];
            /// `astore_1`..`astore_3` and `istore`, one byte each, naming their local.
            const STORES: [(u8, u16); 4] = [(0x4c, 1), (0x4d, 2), (0x4e, 3), (0x36, 4)];
            for index in 0..4 {
                if index < writes {
                    let (opcode, local) = STORES[index as usize];
                    code.push(store(6 + index, opcode, local));
                } else {
                    code.push(plain(6 + index, 0x00));
                }
            }
            code.push(ret(10, 1));
            let facts = body(code, Vec::new(), 12);
            let blocks = raw_block_count(&facts);
            let usage = baseline_usage(&facts);
            (
                blocks,
                usage.counted_usage(CountedBudgetDimension::IrItems),
                baseline_contexts(&facts),
            )
        };
        let (one_block, one_item, one_contexts) = variant(1);
        let (four_blocks, four_items, four_contexts) = variant(4);
        assert_eq!(
            (one_block, one_contexts, four_contexts),
            (four_blocks, 1, 1),
            "the comparison holds the blocks and the contexts fixed"
        );
        assert_eq!(
            four_items - one_item,
            3,
            "three more written locals are three more items: {one_item} vs {four_items}"
        );
    }

    /// The block count of the raw graph of one fixture, as the pass sees it.
    fn raw_block_count(facts: &MethodCodeFacts) -> usize {
        graph(facts, &mut budget()).cfg.blocks.len()
    }

    #[test]
    fn a_cancellation_during_the_assembly_stops_the_pass_without_publishing() {
        // The assembly is the last phase that can still be stopped, and it is the one with no
        // billing point of its own: without its polls a cancelled request would still hand out a
        // complete `CallContexts`. The seam cancels at the next checkpoint of the assembly, and
        // the run must stop there with `Error::Cancelled` and no payload.
        let facts = billing_fixture();
        let raw = graph(&facts, &mut budget());
        let _seam = checkpoint_seam(Phase::Assembly);
        let mut budget = budget();
        let error = call_contexts(&facts, &raw, 45, &mut budget)
            .expect_err("a cancellation inside the assembly publishes nothing");
        assert!(
            matches!(error, Error::Cancelled { .. }),
            "the stop is the crate's own cancellation: {error:?}"
        );
    }

    #[test]
    fn every_phase_of_the_pass_stops_a_cancelled_run() {
        // One case per phase, because a checkpoint only stops a run if it is *reached*: the
        // contract names four phases that must each poll (the plans, the successor lists, the
        // instruction ranges and the assembly), and three more carry the walk, the visited rows
        // and the cycle search. A seam on one phase therefore proves that phase alone — deleting
        // any single checkpoint has to fail this test. `instruction_ranges` is the one with no
        // charge of its own, so without this sweep its only stop point would be invisible.
        for phase in PHASES {
            let facts = billing_fixture();
            let raw = graph(&facts, &mut budget());
            let _seam = checkpoint_seam(phase);
            let mut budget = budget();
            let error = call_contexts(&facts, &raw, 45, &mut budget)
                .expect_err("a cancelled run stops at the phase it was cancelled in");
            assert!(
                matches!(error, Error::Cancelled { .. }),
                "the stop at {phase:?} is the crate's own cancellation: {error:?}"
            );
        }
    }

    #[test]
    fn the_assembly_publishes_the_whole_payload_when_it_is_allowed_to_finish() {
        // The other half of the cancellation case: the payload of a run that is allowed to finish
        // is complete — every context, every `ret`, every coverage record — because a budget is
        // never paid for by publishing a truncated fact.
        let facts = billing_fixture();
        let raw = graph(&facts, &mut budget());
        let contexts = established(&facts, &raw, 45);
        assert_eq!(contexts.contexts.len(), 2, "one context per call site");
        assert_eq!(contexts.returns.len(), 1, "one entry per `ret`");
        assert_eq!(
            contexts.exception_coverage.len(),
            2,
            "one coverage record per context"
        );
        assert!(
            contexts
                .contexts
                .iter()
                .all(|context| context.affected_locals.contains(&1)),
            "the subroutine's writes are kept: {:?}",
            contexts.contexts
        );
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
