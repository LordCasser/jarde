//! Bounded `jsr`/`ret` normalization: the canonical CFG of one decoded method body (3.5).
//!
//! 3.4 proves *what* the legacy `jsr`/`ret` calls of a body mean — one call context per call
//! site, the return point of every `ret` from the context that owns it, and the exception
//! records that cross a call. This module turns that payload into a graph in which the
//! subroutine transfers are **gone**: every block belongs to one call path, a `ret` has the
//! return point of its own context as its successor, and a subroutine shared by two call sites
//! is cloned once per path instead of being merged.
//!
//! Five properties define the artifact, and each one is a deliberate refusal of a cheaper
//! alternative:
//!
//! * **The payload is the only source of return points.** No `ret` target is re-derived from
//!   the bytes here; the module consumes the [`CallContexts`] 3.4b established, and a `ret`
//!   the payload does not prove for its own context stops the normalization instead of
//!   producing an edge the bytes never established.
//! * **Clones are per call path, and they keep every original BCI.** A canonical block that
//!   was reached through `jsr` call sites is a *clone node*, billed one
//!   `NormalizationClones` before it exists, and its [`OriginSet`] holds one
//!   [`OriginMember::MethodPoint`] per original block it stands for — never a single
//!   representative for a region.
//! * **Chains inside one path become super blocks.** Consecutive blocks with a single normal
//!   successor and a single predecessor are fused, which is what makes an origin one-to-many
//!   for a *region* rather than one member per node, and is why the graph stays small enough
//!   to be billed per node.
//! * **Every context of the payload gets its copy, live or not.** The traversal starts at the
//!   method's entry *and* at the call site of every context the live walk never entered — the
//!   ECJ `finally` has its second `jsr` on a handler path no throw site reaches, so the raw
//!   reachability of 3.3 lists it as dead while the payload still names a context for it. The
//!   region that opens is listed in [`CanonicalCfg::unreachable`] instead of being dropped or
//!   merged into the live clone.
//! * **Stopping is honest.** A clone over the `NormalizationClones` bound, an exhausted
//!   `AnalysisSteps` budget or a context the payload cannot decide returns **no graph** — the
//!   raw facts and the call contexts stay the answer — under
//!   [`IR_LEGACY_NORMALIZATION_UNBOUNDED`]. Nothing here replaces a call transfer with a
//!   linear fall-through to pretend the semantics survived.
//!
//! The graph stays a crate-private payload (invariant 11): the report publishes the stage
//! plane, the diagnostic codes and the charged usage, and 5.1 decides what becomes public.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{ExceptionHandlerFact, MethodCodeFacts};
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{OriginMember, OriginSet, PhysicalMethodId};

use crate::call_context::CallContexts;
use crate::cfg::{CfgCompleteness, EdgeKind, RawCfgOutcome, block_of};

/// Code of a normalization that stopped: the clone bound, the step budget, or a call context
/// the payload does not decide.
///
/// The run keeps its raw bytecode facts and its call contexts, publishes no canonical graph and
/// reports `quality = Fallback`; the code names the reason rather than pretending the
/// normalization completed over a graph it could not build.
pub(crate) const IR_LEGACY_NORMALIZATION_UNBOUNDED: &str = "ir_legacy_normalization_unbounded";

/// What one run of the canonicalization produced.
#[derive(Debug)]
pub(crate) enum CanonicalOutcome {
    /// The canonical graph of the decoded body. Its `completeness` is the raw graph's own, so a
    /// body whose decode stopped early yields the graph of its reliable prefix.
    Canonical(Box<CanonicalCfg>),
    /// No graph: the normalization stopped on a reason about the bytes. The reason is the
    /// diagnostic's message.
    Fallback { message: String },
}

/// Identity of one canonical block: an original block start plus the call path it runs under.
///
/// `path` is the stack of `jsr` call site BCIs that were entered to reach this block, outermost
/// first; it is empty exactly for the blocks of the method's own code. The path is what makes a
/// clone a different node from the original block and from the clone of another call site,
/// which is the whole point of the slice: two call sites never share one cloned block.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct CanonicalBlockId {
    /// BCI of the first original block of this node.
    pub(crate) bci: u32,
    pub(crate) path: Vec<u32>,
}

impl CanonicalBlockId {
    /// Whether this node exists only because a `jsr` was crossed: a clone node.
    pub(crate) fn is_clone(&self) -> bool {
        !self.path.is_empty()
    }

    /// The call site this node's path was entered by, for a clone.
    pub(crate) fn call_site(&self) -> Option<u32> {
        self.path.last().copied()
    }
}

/// Kind of one canonical edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum CanonicalEdgeKind {
    /// A fall-through, a conditional branch, a `goto`, a switch target — or the successor of a
    /// `ret`, which is a plain transfer once the context decided where it returns.
    Normal,
    /// The exception edge of one exception-table record, in declaration order.
    Exception { handler_ordinal: u32 },
    /// The transfer into a call context's clone, labelled by the `jsr` that makes it.
    Call { call_site: u32 },
    /// The return half of a `jsr`: the successor of one context's `ret`, labelled by that
    /// context. A shared subroutine has one such edge per clone, pointing at the continuation
    /// of *that* call site.
    Return { call_site: u32 },
}

/// One edge of the canonical graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CanonicalEdge {
    pub(crate) from: CanonicalBlockId,
    pub(crate) to: CanonicalBlockId,
    pub(crate) kind: CanonicalEdgeKind,
}

/// One canonical block: a set of original blocks that one call path reaches as a unit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CanonicalBlock {
    pub(crate) id: CanonicalBlockId,
    /// The original block starts this node stands for, ascending and deduplicated. One entry for
    /// a plain block, more for a fused chain (and for a clone of a shared subroutine, each of
    /// the subroutine's blocks is its own node before the fusion).
    pub(crate) blocks: Vec<u32>,
    /// BCI just past the last original block of this node.
    pub(crate) end_bci: u32,
    /// One `MethodPoint` member per entry of [`Self::blocks`]: a clone keeps **every** original
    /// BCI it stands for, in that order.
    pub(crate) origin: OriginSet,
    /// Exception-table ordinals whose protected range covers this node, in declaration order.
    /// The protected ranges of the table are mapped to the clone they were mapped for, because
    /// the node carries the path.
    pub(crate) protected: Vec<u32>,
}

impl CanonicalBlock {
    /// The original BCIs this node maps back to, in origin order.
    pub(crate) fn origin_bcis(&self) -> Vec<u32> {
        self.origin
            .members
            .iter()
            .map(|member| match member {
                OriginMember::MethodPoint { bci, .. } => *bci,
                other => panic!("a canonical origin holds method points only, found {other:?}"),
            })
            .collect()
    }
}

/// One throwing instruction, kept at instruction granularity through the normalization.
///
/// The raw graph aggregates its exception edges per (block, handler ordinal); this record is
/// what a value-flow consumer reads instead: the exact BCI, its own handler list in declaration
/// order, and the context the instruction runs under — which is the context of
/// [`Self::block`], because a clone's throw site is the same instruction executed under one
/// call path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CanonicalThrowSite {
    pub(crate) bci: u32,
    pub(crate) opcode: u8,
    pub(crate) block: CanonicalBlockId,
    pub(crate) handlers: Vec<u32>,
    pub(crate) origin: OriginSet,
}

/// One exception-table record mapped to the canonical blocks of one call path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CanonicalHandlerRow {
    /// Ordinal of the original record, in declaration order.
    pub(crate) ordinal: u32,
    pub(crate) catch_type_index: Option<u16>,
    /// Original BCI of the handler entry, the anchor the raw facts carry.
    pub(crate) handler_bci: u32,
    /// The canonical block the entry maps to under this row's path, when the graph holds one.
    /// `None` means no clone of this path reaches the handler entry: the exception path leaves
    /// the graph instead of being replaced by an invented successor.
    pub(crate) handler: Option<CanonicalBlockId>,
    /// The protected range of the record, mapped to the clones it is traversed in.
    pub(crate) protected: Vec<CanonicalBlockId>,
}

/// The canonical CFG of one decoded method body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CanonicalCfg {
    /// Blocks: the nodes the entry reaches, in the order the normalization created them.
    pub(crate) blocks: Vec<CanonicalBlock>,
    /// Edges by `(from, kind, to)`.
    pub(crate) edges: Vec<CanonicalEdge>,
    /// Throw sites by `(BCI, path)`: one per throwing instruction per clone of its block.
    pub(crate) throw_sites: Vec<CanonicalThrowSite>,
    /// The exception table mapped to canonical blocks.
    pub(crate) handler_rows: Vec<CanonicalHandlerRow>,
    /// The canonical blocks the entry of the method cannot reach through the canonical
    /// transfers, ascending by identity, naming identities rather than BCIs because one original
    /// block can stand for several clones and only some of them are dead. A truth table, not a
    /// deletion.
    ///
    /// This lists the nodes the normalization **created** and the entry cannot reach. It is not
    /// a partition of the graph: an original block that nothing walked was never created as a
    /// node at all, so it is in neither [`Self::blocks`] nor here - absent, which is a third
    /// state a consumer must not read as reachable. Its identity type is also not the raw
    /// graph's `unreachable`, which is a list of BCIs: the same BCI is `(bci, [])` when the top
    /// level runs it and `(bci, [call_site])` when a clone does, so the two lists answer
    /// different questions about the same BCI and must never be compared or unioned.
    pub(crate) unreachable: Vec<CanonicalBlockId>,
    /// Clone nodes the normalization created and billed, before the fusion. A fused node is one
    /// artifact node but the work of cloning it was already done.
    pub(crate) clones: usize,
    /// The raw graph's own completeness, carried over: a body whose decode stopped early has the
    /// canonical graph of its reliable prefix.
    pub(crate) completeness: CfgCompleteness,
}

impl CanonicalCfg {
    /// Post-condition of 3.5, checked before the graph is published.
    ///
    /// Every block maps back to original instruction starts, through **every** member of its
    /// origin; a clone is covered by the call site that reaches it, so no clone is orphaned; and
    /// the handler rows and edges only name blocks the graph holds.
    pub(crate) fn postcondition(
        &self,
        facts: &MethodCodeFacts,
        method: &PhysicalMethodId,
    ) -> std::result::Result<(), String> {
        let starts: BTreeSet<u32> = facts.instructions.iter().map(|fact| fact.bci).collect();
        let ids: BTreeSet<&CanonicalBlockId> = self.blocks.iter().map(|block| &block.id).collect();
        for block in &self.blocks {
            if block.blocks.is_empty() {
                return Err(format!("block {:?} covers no original block", block.id));
            }
            if block.blocks[0] != block.id.bci
                || !block.blocks.windows(2).all(|pair| pair[0] < pair[1])
            {
                return Err(format!(
                    "block {:?} does not name its original blocks ascending from its own start: {:?}",
                    block.id, block.blocks
                ));
            }
            let origin = block.origin_bcis();
            if origin != block.blocks {
                return Err(format!(
                    "block {:?} maps back to {origin:?} instead of every original BCI {:?} it stands for",
                    block.id, block.blocks
                ));
            }
            for bci in &origin {
                if !starts.contains(bci) {
                    return Err(format!(
                        "block {:?} maps back to BCI {bci}, which is not an instruction start",
                        block.id
                    ));
                }
            }
            for member in &block.origin.members {
                match member {
                    OriginMember::MethodPoint { method: owner, .. } if owner == method => {}
                    other => {
                        return Err(format!(
                            "block {:?} carries an origin member outside the analyzed method: {other:?}",
                            block.id
                        ));
                    }
                }
            }
            if let Some(call_site) = block.id.call_site() {
                // A clone is covered by the call path it belongs to: either the `jsr` at its last
                // call site entered that frame, or a `ret` of the frame one level deeper returned
                // into it (the continuation of that nested call is part of the outer clone).
                let parent = &block.id.path[..block.id.path.len() - 1];
                let covered = self.edges.iter().any(|edge| {
                    if edge.to != block.id {
                        return false;
                    }
                    match edge.kind {
                        CanonicalEdgeKind::Call { call_site: entered } => {
                            entered == call_site && edge.from.path.as_slice() == parent
                        }
                        CanonicalEdgeKind::Return {
                            call_site: returned,
                        } => edge.from.path.split_last().is_some_and(|(last, parent)| {
                            *last == returned && parent == block.id.path.as_slice()
                        }),
                        _ => false,
                    }
                });
                if !covered {
                    return Err(format!(
                        "clone {:?} is orphaned: no `jsr` at BCI {call_site} enters its frame and \
                         no `ret` returns into it",
                        block.id
                    ));
                }
            }
        }
        for edge in &self.edges {
            if !ids.contains(&edge.from) || !ids.contains(&edge.to) {
                return Err(format!(
                    "edge {edge:?} names a block the graph does not hold"
                ));
            }
        }
        for site in &self.throw_sites {
            if !ids.contains(&site.block) {
                return Err(format!(
                    "throw site {site:?} names a block the graph does not hold"
                ));
            }
        }
        for row in &self.handler_rows {
            if row.protected.is_empty() {
                return Err(format!(
                    "handler row {} protects no canonical block",
                    row.ordinal
                ));
            }
            for block in row.protected.iter().chain(row.handler.iter()) {
                if !ids.contains(block) {
                    return Err(format!(
                        "handler row {} names a block the graph does not hold: {block:?}",
                        row.ordinal
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Builds the canonical CFG of one decoded body from 3.4b's call contexts.
///
/// `raw` is the raw graph outcome of exactly these `facts` and `contexts` the payload the
/// driver kept from 3.4b; `method` is the physical method the origins are anchored to. The
/// function never re-derives a return point: it only consumes the payload, so a body whose
/// contexts are unproven or truncated cannot reach it (the driver stops the run before).
///
/// Charges, in order: one `IrItems` per derived successor list and one `IrEdges` per entry of
/// them, one `AnalysisSteps` per worklist pop, per raw transfer examined, per resolved `ret` and
/// per step of the fusion and of the reachability walk, one `NormalizationClones` and one
/// `IrItems` per created node (the block and the origin member its own BCI contributes) **before**
/// the node exists, one `IrEdges` per canonical edge, one `IrItems` per handler row, per
/// throw-site record and for the published artifact itself. The fusion moves origin members
/// between nodes instead of creating them, so it charges steps and no second item.
pub(crate) fn canonical_cfg(
    facts: &MethodCodeFacts,
    raw: &RawCfgOutcome,
    contexts: &CallContexts,
    method: &PhysicalMethodId,
    budget: &mut Budget,
) -> Result<CanonicalOutcome> {
    match build(facts, raw, contexts, method, budget) {
        Ok(graph) => Ok(CanonicalOutcome::Canonical(Box::new(graph))),
        Err(Norm::Unproven(message)) => Ok(CanonicalOutcome::Fallback { message }),
        Err(Norm::Halted(error)) => Err(error),
    }
}

/// Why one normalization run stopped.
enum Norm {
    /// The payload does not decide the normalization: no graph is published.
    Unproven(String),
    /// The request's own budget or cancellation stopped the work.
    Halted(Error),
}

impl From<Error> for Norm {
    fn from(error: Error) -> Self {
        Norm::Halted(error)
    }
}

/// The unproven stop of one normalization, as the type the traversal returns.
fn unproven<T>(message: String) -> std::result::Result<T, Norm> {
    Err(Norm::Unproven(message))
}

/// One phase of a run of this pass, named where [`checkpoint`] is called.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    /// The payload indices and the ret-at-block-end map.
    Payloads,
    /// The successor lists derived from the raw graph.
    Successors,
    /// One node of the cloning traversal.
    Clone,
    /// The exception table mapped to the canonical blocks.
    Handlers,
    /// The fusion of single-successor chains into super blocks.
    Fusion,
    /// The assembly of the published graph and its post-condition.
    Assembly,
}

#[cfg(test)]
/// Every phase, so a test can put its seam on each one in turn.
///
/// The list is beside the enum so that adding a phase is an obvious edit here too, but it is not
/// enforced: a variant left out of it would keep its checkpoint unproven and the sweep would
/// still pass, because the sweep can only visit the phases it is given. Adding a phase therefore
/// means adding it here as well.
const PHASES: [Phase; 6] = [
    Phase::Payloads,
    Phase::Successors,
    Phase::Clone,
    Phase::Handlers,
    Phase::Fusion,
    Phase::Assembly,
];

/// Polls the run's own stop condition at one phase of the pass.
///
/// Every phase either charges (`Budget::charge` polls by itself) or reaches this, so a cancelled
/// or exhausted run stops at a phase boundary instead of finishing the assembly. The stop is the
/// ordinary `Error::Cancelled`/`Error::BudgetExceeded` the rest of the crate reports, and a
/// stopped run publishes no [`CanonicalCfg`].
fn checkpoint(phase: Phase, budget: &Budget) -> Result<()> {
    #[cfg(test)]
    if CHECKPOINT.with(|slot| slot.get()) == Some(phase) {
        CHECKPOINT.with(|slot| slot.set(None));
        budget.cancellation_token().cancel();
    }
    let _ = phase;
    budget.poll()
}

#[cfg(test)]
thread_local! {
    /// The phase a test cancels at, and whether the checkpoint has already fired.
    static CHECKPOINT: std::cell::Cell<Option<Phase>> = const { std::cell::Cell::new(None) };
}

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

/// The published artifact's own charge: the last thing a successful run pays for.
fn published_item(budget: &mut Budget) -> Result<()> {
    budget.charge(CountedBudgetDimension::IrItems, 1)?;
    Ok(())
}

/// One raw transfer of a block, as the raw graph publishes it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RawTransfer {
    Normal,
    Exception { handler_ordinal: u32 },
    Call { call_site: u32 },
}

/// The canonical successor list of one block, derived from the raw graph.
struct Transfers {
    /// By block start BCI, in the raw graph's published edge order.
    by_block: BTreeMap<u32, Vec<(RawTransfer, u32)>>,
}

/// One canonical edge before the assembly: `(from, to, kind)`.
type EdgeDraft = (CanonicalBlockId, CanonicalBlockId, CanonicalEdgeKind);

/// One node of the cloning traversal, before the fusion.
struct Draft {
    id: CanonicalBlockId,
    blocks: Vec<u32>,
    end_bci: u32,
    protected: Vec<u32>,
}

/// The state of one cloning traversal: the payload it consumes and the graph it builds.
struct CloneState<'a> {
    cfg: &'a crate::cfg::RawCfg,
    /// The call contexts by call site BCI: the only source of `jsr` entries and of return points.
    contexts_of: BTreeMap<u32, &'a crate::call_context::SubroutineContext>,
    returns_of: BTreeMap<u32, &'a crate::call_context::SubroutineReturn>,
    /// The `ret` that ends each block, by block start BCI.
    block_ends: BTreeMap<u32, u32>,
    transfers: Transfers,
    drafts: Vec<Draft>,
    index_of: BTreeMap<CanonicalBlockId, usize>,
    edges: Vec<EdgeDraft>,
    clones: usize,
}

impl CloneState<'_> {
    /// Creates the node for one identity, or reports that it already exists.
    ///
    /// The charge is made **before** the node exists: one `NormalizationClones` for a clone node
    /// and one `IrItems` for the block together with the origin member its own BCI contributes. An
    /// identity that already has a node is neither created nor charged twice.
    fn create(
        &mut self,
        id: CanonicalBlockId,
        budget: &mut Budget,
    ) -> std::result::Result<bool, Norm> {
        if self.index_of.contains_key(&id) {
            return Ok(false);
        }
        let Some(index) = block_of(&self.cfg.blocks, id.bci, |block| block.bci) else {
            return unproven(format!(
                "BCI {} is not a block of the raw graph this normalization consumes",
                id.bci
            ));
        };
        let block = self.cfg.blocks[index];
        if id.is_clone() {
            budget.charge(CountedBudgetDimension::NormalizationClones, 1)?;
            self.clones += 1;
        }
        budget.charge(CountedBudgetDimension::IrItems, 2)?;
        self.index_of.insert(id.clone(), self.drafts.len());
        self.drafts.push(Draft {
            id,
            blocks: vec![block.bci],
            end_bci: block.end_bci,
            protected: Vec::new(),
        });
        Ok(true)
    }

    /// Whether one call site already has the clone its `jsr` enters.
    fn enters(&self, call_site: u32) -> bool {
        self.edges
            .iter()
            .any(|(_, _, kind)| *kind == CanonicalEdgeKind::Call { call_site })
    }

    /// Drains one worklist: every node it pops, the raw transfers it holds and its `ret` half.
    fn drain(
        &mut self,
        queue: &mut VecDeque<CanonicalBlockId>,
        budget: &mut Budget,
    ) -> std::result::Result<(), Norm> {
        while let Some(id) = queue.pop_front() {
            // One node of the traversal is one phase of the pass: the checkpoint makes an
            // exhausted or cancelled run stop between two nodes instead of after the whole graph
            // was built.
            checkpoint(Phase::Clone, budget)?;
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            // The block's transfers are copied out of the map so the loop below can create
            // nodes while it walks them.
            let transfers_of_block = self
                .transfers
                .by_block
                .get(&id.bci)
                .cloned()
                .unwrap_or_default();
            for (transfer, to_bci) in &transfers_of_block {
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                let (child, kind) = match *transfer {
                    RawTransfer::Normal => (
                        CanonicalBlockId {
                            bci: *to_bci,
                            path: id.path.clone(),
                        },
                        CanonicalEdgeKind::Normal,
                    ),
                    RawTransfer::Exception { handler_ordinal } => (
                        CanonicalBlockId {
                            bci: *to_bci,
                            path: id.path.clone(),
                        },
                        // The handler runs in the frame of the block it protects, so it belongs
                        // to the same call path, and the ordinal stays the record's own: two
                        // records that name the same entry stay two edges.
                        CanonicalEdgeKind::Exception { handler_ordinal },
                    ),
                    RawTransfer::Call { call_site } => {
                        // The raw transfer is the `jsr` half of a call: the target is the entry of
                        // that call site's clone, one call path deeper.
                        let Some(context) = self.contexts_of.get(&call_site) else {
                            return unproven(format!(
                                "the raw graph transfers to the subroutine at BCI {to_bci} from \
                                 the `jsr` at BCI {call_site}, which no call context of this body \
                                 names"
                            ));
                        };
                        if context.entry_bci != *to_bci {
                            return unproven(format!(
                                "the call context of the `jsr` at BCI {call_site} names entry BCI \
                                 {}, but the raw graph transfers to BCI {to_bci}",
                                context.entry_bci
                            ));
                        }
                        let mut path = id.path.clone();
                        path.push(call_site);
                        (
                            CanonicalBlockId { bci: *to_bci, path },
                            CanonicalEdgeKind::Call { call_site },
                        )
                    }
                };
                if self.create(child.clone(), budget)? {
                    queue.push_back(child.clone());
                }
                budget.charge(CountedBudgetDimension::IrEdges, 1)?;
                self.edges.push((id.clone(), child, kind));
            }
            let Some(ret_bci) = self.block_ends.get(&id.bci).copied() else {
                continue;
            };
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            // The return half: the payload decides where this `ret` returns, and only for the
            // context this clone belongs to.
            let Some(call_site) = id.call_site() else {
                return unproven(format!(
                    "the `ret` at BCI {ret_bci} is reached by BCI {} without any call context: \
                     the bytes do not name the `jsr` it returns from",
                    id.bci
                ));
            };
            let Some(row) = self.returns_of.get(&ret_bci) else {
                return unproven(format!(
                    "the call contexts do not report the `ret` at BCI {ret_bci}: its return point \
                     is not proven"
                ));
            };
            let Some(target) = row
                .targets
                .iter()
                .find(|target| target.call_site_bci == call_site)
            else {
                return unproven(format!(
                    "the `ret` at BCI {ret_bci} has no return point for the call site at BCI \
                     {call_site}: the payload proves no successor for it"
                ));
            };
            let mut parent = id.path.clone();
            parent.pop();
            let return_bci = block_start_of(self.cfg, target.return_bci)?;
            let child = CanonicalBlockId {
                bci: return_bci,
                path: parent,
            };
            if self.create(child.clone(), budget)? {
                queue.push_back(child.clone());
            }
            budget.charge(CountedBudgetDimension::IrEdges, 1)?;
            self.edges
                .push((id, child, CanonicalEdgeKind::Return { call_site }));
        }
        Ok(())
    }
}

/// Builds the graph, or stops.
fn build(
    facts: &MethodCodeFacts,
    raw: &RawCfgOutcome,
    contexts: &CallContexts,
    method: &PhysicalMethodId,
    budget: &mut Budget,
) -> std::result::Result<CanonicalCfg, Norm> {
    debug_assert_eq!(
        facts.instructions.len(),
        raw.effects.instructions.len(),
        "effect facts are produced in lockstep with instructions"
    );
    let cfg = &raw.cfg;
    if cfg.blocks.is_empty() {
        // No instruction at all: no call context can exist and the canonical graph is empty.
        published_item(budget)?;
        return Ok(CanonicalCfg {
            blocks: Vec::new(),
            edges: Vec::new(),
            throw_sites: Vec::new(),
            handler_rows: Vec::new(),
            unreachable: Vec::new(),
            clones: 0,
            completeness: cfg.completeness.clone(),
        });
    }

    checkpoint(Phase::Payloads, budget)?;
    let block_ends = ret_at_block_end(facts, cfg)?;
    let transfers = raw_transfers(cfg, budget)?;
    let mut state = CloneState {
        cfg,
        contexts_of: contexts
            .contexts
            .iter()
            .map(|context| (context.call_site_bci, context))
            .collect(),
        returns_of: contexts
            .returns
            .iter()
            .map(|row| (row.ret_bci, row))
            .collect(),
        block_ends,
        transfers,
        drafts: Vec::new(),
        index_of: BTreeMap::new(),
        edges: Vec::new(),
        clones: 0,
    };

    let root = CanonicalBlockId {
        bci: cfg.blocks[0].bci,
        path: Vec::new(),
    };
    let mut queue: VecDeque<CanonicalBlockId> = VecDeque::new();
    state.create(root.clone(), budget)?;
    queue.push_back(root.clone());
    state.drain(&mut queue, budget)?;

    // Every context of the payload gets its own clone, including one whose call site the raw
    // graph cannot enter — the ECJ `finally` corpus has exactly that shape, where the second
    // `jsr` sits on a handler path no throw site reaches and the raw reachability of 3.3
    // therefore lists it as unreachable. The payload still names a context for that call site
    // (3.4b establishes one per `jsr` of the decoded prefix), and a normalization that kept only
    // the live ones would merge two contexts the bytes keep apart. The dead call site is seeded
    // under the method's own call path, so its clone is still covered by the `jsr` that makes it,
    // and the region it opens is listed in `unreachable` rather than claimed live.
    for context in &contexts.contexts {
        let call_site = context.call_site_bci;
        if state.enters(call_site) {
            continue;
        }
        let block = block_start_of(cfg, call_site)?;
        // Seeding hangs the call site on the method's own path, which is the right frame only
        // for a block the raw graph cannot reach at all: nothing enters it, so no other call
        // opened a region around it. A call site the raw graph *can* reach but the walk never
        // entered sits inside a subroutine, and hanging that on the method's path would file a
        // nested block under the top level - it could collide with a real top-level node and
        // hand it an edge from a frame it is not in. Refuse instead of guessing the frame.
        //
        // This is a guard, not a covered path: no fixture in this crate reaches it (instrumenting
        // it and running the whole suite prints nothing), because the walk reaches every block
        // the raw graph can and crosses the `jsr` it finds there. It is kept because the
        // alternative to refusing is a wrong frame, and a wrong frame is not a bound failure the
        // caller can see through - it is a graph that merges two contexts the bytes keep apart.
        if !cfg.unreachable.contains(&block) {
            return unproven(format!(
                "the `jsr` at BCI {call_site} is not entered by any call path, but its block at \
                 BCI {block} is reachable in the raw graph: it belongs to a subroutine this \
                 normalization cannot place, so no canonical graph is built"
            ));
        }
        let site = CanonicalBlockId {
            bci: block,
            path: Vec::new(),
        };
        if state.create(site.clone(), budget)? {
            queue.push_back(site);
        }
    }
    state.drain(&mut queue, budget)?;

    checkpoint(Phase::Handlers, budget)?;
    let handler_rows = handler_rows(cfg, &state.drafts, method, budget)?;
    let throw_sites = throw_sites(cfg, &state.drafts, method, budget)?;

    checkpoint(Phase::Fusion, budget)?;
    let CloneState {
        drafts,
        edges,
        clones,
        ..
    } = state;
    let (drafts, edges) = fuse(drafts, edges, budget)?;

    checkpoint(Phase::Assembly, budget)?;
    let graph = assemble(
        cfg,
        drafts,
        edges,
        throw_sites,
        handler_rows,
        clones,
        root,
        facts,
        method,
        budget,
    )?;
    if let Err(message) = graph.postcondition(facts, method) {
        // The graph is only published when its own post-condition holds: a graph that does not
        // map every block back to its original BCIs, or that holds an orphaned clone, is a
        // stopped normalization and not an artifact.
        return Err(Norm::Unproven(message));
    }
    published_item(budget)?;
    Ok(graph)
}

/// The successor lists of the raw graph, as this pass consumes them.
fn raw_transfers(
    cfg: &crate::cfg::RawCfg,
    budget: &mut Budget,
) -> std::result::Result<Transfers, Norm> {
    checkpoint(Phase::Successors, budget)?;
    let mut by_block: BTreeMap<u32, Vec<(RawTransfer, u32)>> = BTreeMap::new();
    for edge in &cfg.edges {
        let transfer = match edge.kind {
            EdgeKind::Normal => RawTransfer::Normal,
            EdgeKind::Exception { handler_ordinal } => RawTransfer::Exception { handler_ordinal },
            EdgeKind::SubroutineReturn { call_site } => RawTransfer::Call { call_site },
        };
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        budget.charge(CountedBudgetDimension::IrEdges, 1)?;
        by_block
            .entry(edge.from_bci)
            .or_default()
            .push((transfer, edge.to_bci));
    }
    Ok(Transfers { by_block })
}

/// The `ret` that ends each block it belongs to, by block start BCI.
fn ret_at_block_end(
    facts: &MethodCodeFacts,
    cfg: &crate::cfg::RawCfg,
) -> std::result::Result<BTreeMap<u32, u32>, Norm> {
    let mut ends: BTreeMap<u32, u32> = BTreeMap::new();
    for (position, block) in cfg.blocks.iter().enumerate() {
        let start = instruction_index(facts, block.bci)?;
        // The block's instructions end where the next block starts, or at the end of the decoded
        // prefix for the last block.
        let end = match cfg.blocks.get(position + 1) {
            Some(next) => instruction_index(facts, next.bci)?,
            None => facts.instructions.len(),
        };
        if end <= start {
            continue;
        }
        let last = end - 1;
        if facts.operands()[last].effective_opcode == crate::cfg::OPCODE_RET {
            ends.insert(block.bci, facts.instructions[last].bci);
        }
    }
    Ok(ends)
}

/// Index of the instruction at one BCI.
fn instruction_index(facts: &MethodCodeFacts, bci: u32) -> std::result::Result<usize, Norm> {
    facts
        .instructions
        .binary_search_by_key(&bci, |instruction| instruction.bci)
        .map_err(|_| {
            Norm::Unproven(format!(
                "BCI {bci} is not an instruction start of this method body"
            ))
        })
}

/// The start BCI of the raw block that holds one BCI.
fn block_start_of(cfg: &crate::cfg::RawCfg, bci: u32) -> std::result::Result<u32, Norm> {
    let index = block_of(&cfg.blocks, bci, |block| block.bci).ok_or_else(|| {
        Norm::Unproven(format!(
            "BCI {bci} is not a block of the raw graph this normalization consumes"
        ))
    })?;
    Ok(cfg.blocks[index].bci)
}

/// The origin of one set of original block starts, in ascending order.
fn origin_of(blocks: &[u32], method: &PhysicalMethodId) -> OriginSet {
    let mut origin = OriginSet::default();
    for bci in blocks {
        origin.insert(OriginMember::MethodPoint {
            method: method.clone(),
            bci: *bci,
        });
    }
    origin
}

/// The labels one canonical graph holds, the method's own code first.
fn labels_of(drafts: &[Draft]) -> Vec<Vec<u32>> {
    let mut labels: BTreeSet<Vec<u32>> = BTreeSet::new();
    for draft in drafts {
        labels.insert(draft.id.path.clone());
    }
    labels.into_iter().collect()
}

/// The exception table mapped to the canonical blocks, one row per (record, call path).
fn handler_rows(
    cfg: &crate::cfg::RawCfg,
    drafts: &[Draft],
    method: &PhysicalMethodId,
    budget: &mut Budget,
) -> std::result::Result<Vec<CanonicalHandlerRow>, Norm> {
    let _ = method;
    let mut rows = Vec::new();
    for HandlerFact {
        ordinal,
        start_bci,
        end_bci,
        handler_bci,
        catch_type_index,
    } in cfg.handlers.iter()
    {
        for label in labels_of(drafts) {
            let protected: Vec<CanonicalBlockId> = drafts
                .iter()
                .filter(|draft| {
                    draft.id.path == label
                        && draft
                            .blocks
                            .iter()
                            .any(|bci| *bci >= *start_bci && *bci < *end_bci)
                })
                .map(|draft| draft.id.clone())
                .collect();
            if protected.is_empty() {
                continue;
            }
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            let entry = CanonicalBlockId {
                bci: *handler_bci,
                path: label,
            };
            let handler = drafts
                .iter()
                .any(|draft| draft.id == entry)
                .then_some(entry);
            rows.push(CanonicalHandlerRow {
                ordinal: *ordinal,
                catch_type_index: *catch_type_index,
                handler_bci: *handler_bci,
                handler,
                protected,
            });
        }
    }
    rows.sort_by(|left, right| {
        left.ordinal
            .cmp(&right.ordinal)
            .then_with(|| path_of(&left.protected).cmp(&path_of(&right.protected)))
    });
    Ok(rows)
}

/// The call path of one row's protected blocks: they all belong to the same clone.
fn path_of(blocks: &[CanonicalBlockId]) -> Vec<u32> {
    blocks.first().map_or_else(Vec::new, |id| id.path.clone())
}

/// The throw sites of the raw graph, one record per clone of the block that holds them.
fn throw_sites(
    cfg: &crate::cfg::RawCfg,
    drafts: &[Draft],
    method: &PhysicalMethodId,
    budget: &mut Budget,
) -> std::result::Result<Vec<CanonicalThrowSite>, Norm> {
    let mut sites = Vec::new();
    for site in &cfg.throw_sites {
        let mut paths: Vec<Vec<u32>> = drafts
            .iter()
            .filter(|draft| draft.id.bci == site.block_bci)
            .map(|draft| draft.id.path.clone())
            .collect();
        paths.sort();
        paths.dedup();
        for path in paths {
            let id = CanonicalBlockId {
                bci: site.block_bci,
                path,
            };
            if !drafts.iter().any(|draft| draft.id == id) {
                continue;
            }
            budget.charge(CountedBudgetDimension::IrItems, 1)?;
            let mut origin = OriginSet::default();
            origin.insert(OriginMember::MethodPoint {
                method: method.clone(),
                bci: site.bci,
            });
            sites.push(CanonicalThrowSite {
                bci: site.bci,
                opcode: site.opcode,
                block: id,
                handlers: site.handlers.clone(),
                origin,
            });
        }
    }
    sites.sort_by(|left, right| {
        left.bci
            .cmp(&right.bci)
            .then_with(|| left.block.path.cmp(&right.block.path))
    });
    Ok(sites)
}

/// Fuses single-successor chains inside one call path into super blocks.
///
/// A node is fused with its successor when the transfer between them is the only edge leaving
/// the first and the only edge entering the second: the two run as a unit, and their original
/// blocks and origins become one node's. The fusion is what makes an origin one-to-many for a
/// region, and it never crosses a call boundary, because a `jsr` transfer is a `Call` edge
/// between two different paths.
fn fuse(
    drafts: Vec<Draft>,
    edges: Vec<EdgeDraft>,
    budget: &mut Budget,
) -> std::result::Result<(Vec<Draft>, Vec<EdgeDraft>), Norm> {
    let mut outgoing: BTreeMap<CanonicalBlockId, Vec<usize>> = BTreeMap::new();
    let mut incoming: BTreeMap<CanonicalBlockId, Vec<usize>> = BTreeMap::new();
    for (index, (from, to, _)) in edges.iter().enumerate() {
        outgoing.entry(from.clone()).or_default().push(index);
        incoming.entry(to.clone()).or_default().push(index);
    }
    let mut owner: BTreeMap<CanonicalBlockId, CanonicalBlockId> = drafts
        .iter()
        .map(|draft| (draft.id.clone(), draft.id.clone()))
        .collect();
    let mut fused_edges: BTreeSet<usize> = BTreeSet::new();
    let ids: Vec<CanonicalBlockId> = drafts.iter().map(|draft| draft.id.clone()).collect();
    let mut drafts = drafts;
    for id in &ids {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let head = owner.get(id).cloned().unwrap_or_else(|| id.clone());
        // The successors are taken by value because fusing rewrites the edge table below: the
        // loop condition cannot borrow it while the body inserts and removes the same map.
        while let Some(candidates) = outgoing.get(&head).cloned() {
            // Exactly one edge may leave the head, and it must be the plain transfer of a fused
            // chain: a `return` half or an exception edge is a context boundary, not a fuse.
            if candidates.len() != 1 {
                break;
            }
            let edge_index = candidates[0];
            let (from, to, kind) = &edges[edge_index];
            if from != &head || !matches!(kind, CanonicalEdgeKind::Normal) {
                break;
            }
            if incoming.get(to).map_or(0, Vec::len) != 1 {
                break;
            }
            // The successor is reached by this edge alone and lies ahead: a superblock runs
            // forward, so absorbing a block that starts *before* this one both inverts the
            // node's own start and can swallow a loop header or the method's entry. A backward
            // edge out of a block with a single successor - the loop body returning to its
            // header - reaches this arm, and the resulting node would not name its own start.
            if owner.get(to).cloned() != Some(to.clone()) || to == &head || to.bci <= head.bci {
                break;
            }
            // The successor is reached by this edge alone: the two run as one node.
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            fused_edges.insert(edge_index);
            owner.insert(to.clone(), head.clone());
            let tail = drafts
                .iter()
                .find(|draft| draft.id == *to)
                .expect("a node the edge names");
            let mut merged = tail.blocks.clone();
            let end_bci = tail.end_bci;
            let protected = tail.protected.clone();
            let head_draft = drafts
                .iter_mut()
                .find(|draft| draft.id == head)
                .expect("the head of a chain");
            head_draft.blocks.append(&mut merged);
            head_draft.blocks.sort_unstable();
            head_draft.blocks.dedup();
            head_draft.end_bci = end_bci;
            for ordinal in protected {
                if !head_draft.protected.contains(&ordinal) {
                    head_draft.protected.push(ordinal);
                }
            }
            head_draft.protected.sort_unstable();
            let tail_out = outgoing.get(to).cloned().unwrap_or_default();
            outgoing.insert(head.clone(), tail_out);
            outgoing.remove(to);
            incoming.remove(to);
        }
    }
    let mut fused = Vec::new();
    for draft in &drafts {
        if owner.get(&draft.id).cloned() == Some(draft.id.clone()) {
            fused.push(Draft {
                id: draft.id.clone(),
                blocks: draft.blocks.clone(),
                end_bci: draft.end_bci,
                protected: draft.protected.clone(),
            });
        }
    }
    let edges = edges
        .into_iter()
        .enumerate()
        .filter(|(index, _)| !fused_edges.contains(index))
        .map(|(_, (from, to, kind))| {
            let from = owner.get(&from).cloned().unwrap_or(from);
            let to = owner.get(&to).cloned().unwrap_or(to);
            (from, to, kind)
        })
        .collect();
    Ok((fused, edges))
}

/// Assembles the published graph from the fused drafts.
#[allow(clippy::too_many_arguments)]
fn assemble(
    cfg: &crate::cfg::RawCfg,
    drafts: Vec<Draft>,
    edges: Vec<EdgeDraft>,
    throw_sites: Vec<CanonicalThrowSite>,
    handler_rows: Vec<CanonicalHandlerRow>,
    clones: usize,
    root: CanonicalBlockId,
    facts: &MethodCodeFacts,
    method: &PhysicalMethodId,
    budget: &mut Budget,
) -> std::result::Result<CanonicalCfg, Norm> {
    let _ = (facts, method);
    let mut blocks = Vec::with_capacity(drafts.len());
    for draft in &drafts {
        let mut protected = draft.protected.clone();
        protected.sort_unstable();
        protected.dedup();
        blocks.push(CanonicalBlock {
            id: draft.id.clone(),
            blocks: draft.blocks.clone(),
            end_bci: draft.end_bci,
            origin: origin_of(&draft.blocks, method),
            protected,
        });
    }
    let ids: BTreeSet<CanonicalBlockId> = blocks.iter().map(|block| block.id.clone()).collect();
    let mut edges: Vec<CanonicalEdge> = edges
        .into_iter()
        .filter(|(from, to, _)| ids.contains(from) && ids.contains(to))
        .map(|(from, to, kind)| CanonicalEdge { from, to, kind })
        .collect();
    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.to.cmp(&right.to))
    });
    // What the entry of the method really reaches, over the canonical transfers: everything else
    // is a truth table entry rather than a silently dropped node.
    let mut outgoing: BTreeMap<&CanonicalBlockId, Vec<&CanonicalBlockId>> = BTreeMap::new();
    for edge in &edges {
        outgoing.entry(&edge.from).or_default().push(&edge.to);
    }
    let mut seen: BTreeSet<CanonicalBlockId> = BTreeSet::new();
    let mut worklist: Vec<CanonicalBlockId> = vec![root];
    while let Some(id) = worklist.pop() {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !seen.insert(id.clone()) {
            continue;
        }
        for child in outgoing.get(&id).map_or(&[][..], Vec::as_slice) {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            worklist.push((*child).clone());
        }
    }
    let unreachable: Vec<CanonicalBlockId> =
        ids.into_iter().filter(|id| !seen.contains(id)).collect();
    Ok(CanonicalCfg {
        blocks,
        edges,
        throw_sites,
        handler_rows,
        unreachable,
        clones,
        completeness: cfg.completeness.clone(),
    })
}

/// One exception-table record, as the raw graph publishes it.
type HandlerFact = ExceptionHandlerFact;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call_context::{CallContextOutcome, call_contexts};
    use crate::cfg::raw_cfg;
    use jarde_reader::budget::{BudgetDimension, Limits, UsageSnapshot};
    use jarde_reader::classfile::{
        BytecodeStop, InstructionFact, InstructionOperands, LocalOperand, class_facts,
        method_code_facts,
    };
    use jarde_reader::model::{
        ByteSpan, ClassBytesId, Digest, ExecutionReport, JvmBytes, PhysicalClassLocation,
        PhysicalDefinitionId, PhysicalVariant, SnapshotId,
    };

    /// Class-file offset the synthetic fixture bodies start at.
    const CODE_OFFSET: u64 = 200;

    /// The committed historical fixtures: one source compiled to every dialect of the jsr era.
    const V45: &[u8] =
        crate::test_fixtures::fixture!("historical/ecj-4.6.1/v45/HistoricalControlFlow.class");
    const V46: &[u8] =
        crate::test_fixtures::fixture!("historical/ecj-4.6.1/v46/HistoricalControlFlow.class");
    const V47: &[u8] =
        crate::test_fixtures::fixture!("historical/ecj-4.6.1/v47/HistoricalControlFlow.class");
    const V48: &[u8] =
        crate::test_fixtures::fixture!("historical/ecj-4.6.1/v48/HistoricalControlFlow.class");

    fn limits() -> Limits {
        Limits {
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        }
    }

    fn budget() -> Budget {
        Budget::new(limits())
    }

    /// The same limits with the one dimension a stop test bounds.
    fn limits_bounded(dimension: CountedBudgetDimension, limit: u64) -> Limits {
        let mut limits = limits();
        match dimension {
            CountedBudgetDimension::NormalizationClones => limits.normalization_clones = limit,
            CountedBudgetDimension::AnalysisSteps => limits.analysis_steps = limit,
            CountedBudgetDimension::IrItems => limits.ir_items = limit,
            CountedBudgetDimension::IrEdges => limits.ir_edges = limit,
            other => panic!("this pass bills {other:?} and nothing else in these tests"),
        }
        limits
    }

    /// The identity the origins of the synthetic bodies are anchored to.
    fn method() -> PhysicalMethodId {
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
            name: JvmBytes(b"m".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        }
    }

    /// One instruction fact with its typed operands, at `bci`, `width` bytes long.
    fn instruction(
        bci: u32,
        opcode: u8,
        width: u32,
        operands: InstructionOperands,
    ) -> (InstructionFact, InstructionOperands) {
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

    fn operands(opcode: u8) -> InstructionOperands {
        InstructionOperands {
            effective_opcode: opcode,
            ..InstructionOperands::default()
        }
    }

    fn local(index: u16, wide: bool, effective_opcode: u8) -> InstructionOperands {
        InstructionOperands {
            local: Some(LocalOperand { index, wide }),
            effective_opcode,
            ..InstructionOperands::default()
        }
    }

    fn plain(bci: u32, opcode: u8) -> (InstructionFact, InstructionOperands) {
        instruction(bci, opcode, 1, operands(opcode))
    }

    fn jsr(bci: u32, offset: i32) -> (InstructionFact, InstructionOperands) {
        instruction(
            bci,
            crate::cfg::OPCODE_JSR,
            3,
            InstructionOperands {
                branch_offset: Some(offset),
                ..operands(crate::cfg::OPCODE_JSR)
            },
        )
    }

    fn ret(bci: u32, index: u16) -> (InstructionFact, InstructionOperands) {
        instruction(
            bci,
            crate::cfg::OPCODE_RET,
            2,
            local(index, false, crate::cfg::OPCODE_RET),
        )
    }

    /// `astore_0`..`astore_3`, the stores a `jsr` subroutine saves its return address with.
    fn astore(bci: u32, index: u16) -> (InstructionFact, InstructionOperands) {
        match index {
            0..=3 => {
                let opcode = 0x4b + u8::try_from(index).expect("a short store index");
                instruction(bci, opcode, 1, local(index, false, opcode))
            }
            _ => instruction(bci, 0x3a, 2, local(index, false, 0x3a)),
        }
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
        )
    }

    /// The raw graph of one synthetic body.
    fn raw_of(facts: &MethodCodeFacts) -> RawCfgOutcome {
        raw_cfg(facts, &mut budget()).expect("the fixture body has a raw graph")
    }

    /// The call contexts 3.4b establishes for one synthetic body.
    fn contexts_of(facts: &MethodCodeFacts, raw: &RawCfgOutcome) -> CallContexts {
        match call_contexts(facts, raw, 50, &mut budget()).expect("the walk runs") {
            CallContextOutcome::Established(contexts) => contexts,
            other => panic!("the fixture must establish contexts, got {other:?}"),
        }
    }

    /// The canonical graph of one synthetic body, through the payload 3.4b established.
    fn normalize(facts: &MethodCodeFacts) -> CanonicalCfg {
        let raw = raw_of(facts);
        let contexts = contexts_of(facts, &raw);
        match canonical_cfg(facts, &raw, &contexts, &method(), &mut budget())
            .expect("the budget is ample")
        {
            CanonicalOutcome::Canonical(graph) => *graph,
            CanonicalOutcome::Fallback { message } => {
                panic!("the fixture must normalize, but stopped: {message}")
            }
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

    /// The canonical graph of one historical fixture method, with the identity its origins carry.
    fn historical_graph(
        bytes: &[u8],
        name: &[u8],
    ) -> (MethodCodeFacts, PhysicalMethodId, CanonicalCfg) {
        let (facts, major) = historical(bytes, name);
        let raw = raw_of(&facts);
        let contexts =
            match call_contexts(&facts, &raw, major, &mut budget()).expect("the walk runs") {
                CallContextOutcome::Established(contexts) => contexts,
                other => panic!("the historical fixture must establish contexts, got {other:?}"),
            };
        let mut budget = budget();
        let method = method();
        let graph = match canonical_cfg(&facts, &raw, &contexts, &method, &mut budget)
            .expect("the budget is ample")
        {
            CanonicalOutcome::Canonical(graph) => *graph,
            CanonicalOutcome::Fallback { message } => {
                panic!("the historical fixture must normalize, but stopped: {message}")
            }
        };
        (facts, method, graph)
    }

    /// The synthetic bodies below, as `(instructions, exception table, code_length)` triples.
    ///
    /// * **an exception-path call**: a `jsr` in the handler of a protected `athrow`, whose
    ///   subroutine is a three-block chain (`astore`, `nop`, `goto`, `ret`) so the clone of the
    ///   region stands for more than one original block;
    /// * **nested calls**: the outer subroutine holds a `jsr` of its own;
    /// * **a shared subroutine with a nested call**: two call sites enter the outer routine, so
    ///   the nested routine's clone exists once per call path.
    fn exception_path_call() -> (MethodCodeFacts, u32) {
        let code = vec![
            plain(0, 0xbf), // athrow, protected by record 0
            plain(1, 0xb1), // return
            astore(2, 0),   // astore_0: the handler entry
            jsr(3, 4),      // jsr 7: the call on the exception path
            plain(6, 0xb1), // return: the continuation of the call at BCI 3
            astore(7, 1),   // astore_1: the subroutine entry
            plain(8, 0x00), // nop
            instruction(
                9,
                0xa7,
                3,
                InstructionOperands {
                    branch_offset: Some(3),
                    ..operands(0xa7)
                },
            ), // goto 12
            ret(12, 1),     // ret 1
        ];
        (body(code, vec![catch(0, 0, 1, 2, None)], 14), 14)
    }

    fn nested_calls() -> (MethodCodeFacts, u32) {
        let code = vec![
            jsr(0, 4),      // call site 0 -> subroutine at BCI 4, return at BCI 3
            plain(3, 0xb1), // return
            astore(4, 0),   // the outer subroutine
            jsr(5, 5),      // nested call site 5 -> subroutine at BCI 10, return at BCI 8
            ret(8, 0),      // ret 0
            astore(10, 1),  // the nested subroutine
            ret(11, 1),     // ret 1
        ];
        (body(code, Vec::new(), 13), 13)
    }

    fn shared_routine_with_a_nested_call() -> (MethodCodeFacts, u32) {
        let code = vec![
            jsr(0, 7),      // call site 0 -> subroutine at BCI 7, return at BCI 3
            jsr(3, 4),      // call site 3 -> the same subroutine, return at BCI 6
            plain(6, 0xb1), // return
            astore(7, 0),   // the shared subroutine
            jsr(8, 5),      // nested call site 8 -> subroutine at BCI 13, return at BCI 11
            ret(11, 0),     // ret 0
            astore(13, 1),  // the nested subroutine
            ret(14, 1),     // ret 1
        ];
        (body(code, Vec::new(), 16), 16)
    }

    /// The blocks of one graph whose identity starts at one BCI, ascending by path.
    fn nodes_at(graph: &CanonicalCfg, bci: u32) -> Vec<&CanonicalBlock> {
        graph
            .blocks
            .iter()
            .filter(|block| block.id.bci == bci)
            .collect()
    }

    /// The edges of one graph that leave one identity.
    fn edges_from<'a>(graph: &'a CanonicalCfg, from: &CanonicalBlockId) -> Vec<&'a CanonicalEdge> {
        graph
            .edges
            .iter()
            .filter(|edge| &edge.from == from)
            .collect()
    }

    /// The `Return` edges of one clone: the successors its own `ret` decides.
    fn returns_of(graph: &CanonicalCfg, from: &CanonicalBlockId) -> Vec<(u32, CanonicalBlockId)> {
        edges_from(graph, from)
            .into_iter()
            .filter_map(|edge| match edge.kind {
                CanonicalEdgeKind::Return { call_site } => Some((call_site, edge.to.clone())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_historical_finally_clones_its_shared_subroutine_per_call_site() {
        // The ECJ 4.6 `finallyPath(I)I` of every jsr-era dialect: one subroutine at BCI 17,
        // entered by the `jsr` at BCI 5 (the normal path) and by the `jsr` at BCI 12 (the
        // handler path the raw graph cannot enter), and one `ret` at BCI 21 whose payload
        // names both call sites, each with its own return address. The canonical graph must
        // hold one clone per call site — not one shared clone, and not only the live one.
        for (version, bytes) in [(45, V45), (46, V46), (47, V47), (48, V48)] {
            let (facts, method, graph) = historical_graph(bytes, b"finallyPath");
            graph
                .postcondition(&facts, &method)
                .unwrap_or_else(|message| panic!("classfile major {version}: {message}"));
            // The graph is a property of the bytes and the payload, so a second run of the same
            // normalization is the same graph field by field (no hash order, no clock).
            let (_, _, again) = historical_graph(bytes, b"finallyPath");
            assert_eq!(graph, again, "classfile major {version}");

            assert_eq!(
                graph.clones, 2,
                "one clone node per call site, major {version}"
            );
            let clones = nodes_at(&graph, 17);
            assert_eq!(
                clones
                    .iter()
                    .map(|block| block.id.path.clone())
                    .collect::<Vec<_>>(),
                vec![vec![5], vec![12]],
                "each call site gets its own copy of the subroutine, major {version}"
            );
            // The one `ret` of the subroutine returns to the continuation of *its* call site.
            let live = CanonicalBlockId {
                bci: 17,
                path: vec![5],
            };
            let dead = CanonicalBlockId {
                bci: 17,
                path: vec![12],
            };
            assert_eq!(
                returns_of(&graph, &live),
                vec![(
                    5,
                    CanonicalBlockId {
                        bci: 8,
                        path: Vec::new()
                    }
                )],
                "the clone of the call at BCI 5 returns to BCI 8, major {version}"
            );
            assert_eq!(
                returns_of(&graph, &dead),
                vec![(
                    12,
                    CanonicalBlockId {
                        bci: 15,
                        path: Vec::new()
                    }
                )],
                "the clone of the call at BCI 12 returns to BCI 15, major {version}"
            );
            // The `jsr` itself is the edge that makes the clone a covered node, and the two
            // transfers are the only way into the two clones.
            assert_eq!(
                graph
                    .edges
                    .iter()
                    .filter(|edge| edge.kind == CanonicalEdgeKind::Call { call_site: 5 })
                    .map(|edge| edge.to.clone())
                    .collect::<Vec<_>>(),
                vec![live.clone()],
                "major {version}"
            );
            assert_eq!(
                graph
                    .edges
                    .iter()
                    .filter(|edge| edge.kind == CanonicalEdgeKind::Call { call_site: 12 })
                    .map(|edge| edge.to.clone())
                    .collect::<Vec<_>>(),
                vec![dead.clone()],
                "major {version}"
            );
            // Every canonical block maps back to an original BCI, and the origin of a clone
            // holds the BCI of the block it stands for — every member of it.
            for block in &graph.blocks {
                assert_eq!(block.origin_bcis(), block.blocks, "major {version}");
                assert!(!block.origin.is_empty(), "major {version}");
            }
            // The dead handler path is listed, not dropped: the raw graph cannot enter it, the
            // canonical graph holds it, and the truth table says so.
            assert_eq!(
                graph.unreachable,
                vec![
                    CanonicalBlockId {
                        bci: 11,
                        path: Vec::new()
                    },
                    CanonicalBlockId {
                        bci: 15,
                        path: Vec::new()
                    },
                    dead.clone(),
                ],
                "the handler path of the `jsr` at BCI 12 is a truth table entry, major {version}"
            );
            // The protected range of record 0 is mapped to the clone it protects, and its
            // handler entry keeps the ordinal and the original BCI.
            assert_eq!(graph.handler_rows.len(), 1, "major {version}");
            let row = &graph.handler_rows[0];
            assert_eq!(row.ordinal, 0);
            assert_eq!(row.handler_bci, 11);
            assert_eq!(
                row.handler,
                Some(CanonicalBlockId {
                    bci: 11,
                    path: Vec::new()
                }),
                "major {version}"
            );
            assert_eq!(
                row.protected,
                vec![CanonicalBlockId {
                    bci: 0,
                    path: Vec::new()
                }],
                "the record protects the block that holds the call, major {version}"
            );
            // The throwing instruction of the finally's rethrow path stays instruction level.
            assert_eq!(
                graph
                    .throw_sites
                    .iter()
                    .map(|site| (site.bci, site.opcode, site.block.clone()))
                    .collect::<Vec<_>>(),
                vec![(
                    16,
                    0xbf,
                    CanonicalBlockId {
                        bci: 15,
                        path: Vec::new()
                    }
                )],
                "major {version}"
            );
        }
    }

    #[test]
    fn a_clone_of_a_multi_block_region_keeps_every_original_bci() {
        // The subroutine of this body is three blocks (`astore`, `nop`, `goto`, `ret`), and the
        // fusion makes one node of that chain per context. Its origin is therefore one-to-many,
        // and it holds **every** original BCI of the region: a single representative would lose
        // the mapping the post-condition is stated over.
        let (facts, _length) = exception_path_call();
        let method = method();
        let graph = normalize(&facts);
        graph
            .postcondition(&facts, &method)
            .expect("the post-condition holds");

        let clone = CanonicalBlockId {
            bci: 7,
            path: vec![3],
        };
        let block = graph
            .blocks
            .iter()
            .find(|block| block.id == clone)
            .expect("the subroutine has its clone for the call at BCI 3");
        assert_eq!(block.blocks, vec![7, 12]);
        assert_eq!(block.origin_bcis(), vec![7, 12]);
        assert_eq!(block.end_bci, 14);
        // Two clone nodes were created — one per original block of the region — and the fusion
        // made one artifact node of them; the clone dimension bills the nodes it created, not the
        // nodes that survive the fusion.
        assert_eq!(graph.clones, 2);
        // The `ret` of the clone returns into the handler path that called it.
        assert_eq!(
            returns_of(&graph, &clone),
            vec![(
                3,
                CanonicalBlockId {
                    bci: 6,
                    path: Vec::new()
                }
            )]
        );
        // A truncated origin is exactly what the post-condition refuses, so a graph that
        // collapsed a region's origin into one BCI could never be published.
        let mut truncated = graph.clone();
        let block = truncated
            .blocks
            .iter_mut()
            .find(|block| block.id == clone)
            .expect("the clone");
        block.origin.members.truncate(1);
        let message = truncated
            .postcondition(&facts, &method)
            .expect_err("a collapsed origin must be refused");
        assert!(
            message.contains("instead of every original BCI"),
            "the refusal names the property it checked: {message}"
        );
    }

    #[test]
    fn a_jsr_in_a_handler_is_cloned_with_the_context_of_the_exception_path() {
        // The handler of record 0 is entered by an exception edge, not by a fall-through, and
        // the `jsr` it holds is a call like any other: the clone of the subroutine belongs to
        // the handler's context, and the exception edge that reaches the handler keeps the
        // record's ordinal.
        let (facts, _length) = exception_path_call();
        let method = method();
        let graph = normalize(&facts);
        graph
            .postcondition(&facts, &method)
            .expect("the post-condition holds");

        let handler = CanonicalBlockId {
            bci: 2,
            path: Vec::new(),
        };
        assert_eq!(
            graph
                .edges
                .iter()
                .filter(|edge| edge.to == handler)
                .map(|edge| (edge.from.clone(), edge.kind))
                .collect::<Vec<_>>(),
            vec![(
                CanonicalBlockId {
                    bci: 0,
                    path: Vec::new()
                },
                CanonicalEdgeKind::Exception { handler_ordinal: 0 }
            )],
            "the handler is reached by the record's own exception edge"
        );
        assert_eq!(
            graph
                .throw_sites
                .iter()
                .map(|site| (
                    site.bci,
                    site.opcode,
                    site.block.clone(),
                    site.handlers.clone()
                ))
                .collect::<Vec<_>>(),
            vec![(
                0,
                0xbf,
                CanonicalBlockId {
                    bci: 0,
                    path: Vec::new()
                },
                vec![0]
            )]
        );
        let clone = CanonicalBlockId {
            bci: 7,
            path: vec![3],
        };
        assert!(graph.blocks.iter().any(|block| block.id == clone));
        // The protected range maps to the clone of the block it covers and to nothing else.
        let row = &graph.handler_rows[0];
        assert_eq!(row.handler, Some(handler));
        assert_eq!(
            row.protected,
            vec![CanonicalBlockId {
                bci: 0,
                path: Vec::new()
            }]
        );
        assert!(
            graph.unreachable.is_empty(),
            "this body has a live handler path"
        );
    }

    #[test]
    fn nested_calls_keep_their_call_path() {
        // `jsr` at BCI 0 enters the routine at BCI 4; that routine calls the one at BCI 10 and
        // returns from there first. The path of the nested clone is the stack of call sites, and
        // each `ret` returns into the continuation of the call that owns it.
        let (facts, _length) = nested_calls();
        let method = method();
        let graph = normalize(&facts);
        graph
            .postcondition(&facts, &method)
            .expect("the post-condition holds");

        let nested = CanonicalBlockId {
            bci: 10,
            path: vec![0, 5],
        };
        assert!(
            graph.blocks.iter().any(|block| block.id == nested),
            "the nested routine's clone carries both call sites: {:?}",
            graph
                .blocks
                .iter()
                .map(|b| b.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            returns_of(&graph, &nested),
            vec![(
                5,
                CanonicalBlockId {
                    bci: 8,
                    path: vec![0]
                }
            )],
            "the nested `ret` returns to the outer routine's continuation"
        );
        assert_eq!(
            returns_of(
                &graph,
                &CanonicalBlockId {
                    bci: 8,
                    path: vec![0]
                }
            ),
            vec![(
                0,
                CanonicalBlockId {
                    bci: 3,
                    path: Vec::new()
                }
            )],
            "the outer `ret` returns to the method's own continuation"
        );
        assert_eq!(
            graph.clones, 3,
            "the outer entry, the nested entry and the outer ret"
        );
    }

    #[test]
    fn a_subroutine_shared_by_two_call_sites_clones_its_nested_call_per_path() {
        // Two `jsr` sites (BCI 0 and BCI 3) enter the routine at BCI 7, and that routine calls
        // the one at BCI 13. The nested routine's clone therefore exists once per call **path**
        // (`[0, 8]` and `[3, 8]`), and each returns into the continuation of the call that owns
        // it: merging them would make one clone answer for two different continuations.
        let (facts, _length) = shared_routine_with_a_nested_call();
        let method = method();
        let graph = normalize(&facts);
        graph
            .postcondition(&facts, &method)
            .expect("the post-condition holds");

        assert_eq!(
            nodes_at(&graph, 13)
                .iter()
                .map(|block| block.id.path.clone())
                .collect::<Vec<_>>(),
            vec![vec![0, 8], vec![3, 8]],
            "one nested clone per outer call path"
        );
        assert_eq!(
            returns_of(
                &graph,
                &CanonicalBlockId {
                    bci: 13,
                    path: vec![0, 8]
                }
            ),
            vec![(
                8,
                CanonicalBlockId {
                    bci: 11,
                    path: vec![0]
                }
            )]
        );
        assert_eq!(
            returns_of(
                &graph,
                &CanonicalBlockId {
                    bci: 13,
                    path: vec![3, 8]
                }
            ),
            vec![(
                8,
                CanonicalBlockId {
                    bci: 11,
                    path: vec![3]
                }
            )]
        );
        assert_eq!(
            nodes_at(&graph, 7).len(),
            2,
            "the shared routine has two clones"
        );
        assert_eq!(graph.clones, 6);
    }

    #[test]
    fn an_exact_clone_budget_completes_and_one_less_stops() {
        // The clone bound is a bound of the artifact: the number the pass needs is discoverable
        // from its own usage, and the dimension is charged per clone node before it exists.
        let (facts, _length) = exception_path_call();
        let raw = raw_of(&facts);
        let contexts = contexts_of(&facts, &raw);

        let mut ample = budget();
        let graph = match canonical_cfg(&facts, &raw, &contexts, &method(), &mut ample)
            .expect("the ample budget is enough")
        {
            CanonicalOutcome::Canonical(graph) => *graph,
            other => panic!("the fixture normalizes, got {other:?}"),
        };
        let needed = ample.usage().normalization_clones;
        assert_eq!(
            needed,
            u64::try_from(graph.clones).expect("a clone count fits u64")
        );
        assert!(needed > 0, "this body really clones a subroutine");

        // Exactly the clones it needs: it completes.
        let mut exact = Budget::new(limits_bounded(
            CountedBudgetDimension::NormalizationClones,
            needed,
        ));
        assert!(matches!(
            canonical_cfg(&facts, &raw, &contexts, &method(), &mut exact),
            Ok(CanonicalOutcome::Canonical(_))
        ));
        assert_eq!(exact.usage().normalization_clones, needed);

        // One clone less: it stops without a graph, and the dimension it stopped on is named.
        let mut short = Budget::new(limits_bounded(
            CountedBudgetDimension::NormalizationClones,
            needed - 1,
        ));
        match canonical_cfg(&facts, &raw, &contexts, &method(), &mut short) {
            Err(Error::BudgetExceeded {
                dimension,
                limit,
                consumed,
                requested,
            }) => {
                assert_eq!(dimension, BudgetDimension::NormalizationClones);
                assert_eq!(limit, needed - 1);
                assert_eq!(consumed, needed - 1);
                assert_eq!(requested, 1);
            }
            other => panic!("a clone over the bound must stop, got {other:?}"),
        }
        assert_eq!(short.usage().normalization_clones, needed - 1);
    }

    #[test]
    fn an_exhausted_step_budget_stops_the_walk_without_a_graph() {
        // The steps of the two passes before it are the budget the normalization finds left: the
        // walk itself then stops on its first charged step, and no graph is published.
        let (facts, _length) = exception_path_call();
        let mut measured = budget();
        let raw = raw_cfg(&facts, &mut measured).expect("the fixture body has a raw graph");
        let _contexts = match call_contexts(&facts, &raw, 50, &mut measured).expect("the walk runs")
        {
            CallContextOutcome::Established(contexts) => contexts,
            other => panic!("the fixture must establish contexts, got {other:?}"),
        };
        let spent = measured.usage().analysis_steps;
        assert!(spent > 0, "the raw graph and the walk charge steps");

        let mut budget = Budget::new(limits_bounded(CountedBudgetDimension::AnalysisSteps, spent));
        let raw = raw_cfg(&facts, &mut budget).expect("the same charges fit the same budget");
        let contexts = match call_contexts(&facts, &raw, 50, &mut budget).expect("the walk runs") {
            CallContextOutcome::Established(contexts) => contexts,
            other => panic!("the fixture must establish contexts, got {other:?}"),
        };
        assert_eq!(budget.usage().analysis_steps, spent);
        match canonical_cfg(&facts, &raw, &contexts, &method(), &mut budget) {
            Err(Error::BudgetExceeded { dimension, .. }) => {
                assert_eq!(dimension, BudgetDimension::AnalysisSteps);
            }
            other => panic!("an exhausted step budget must stop the walk, got {other:?}"),
        }
    }

    #[test]
    fn a_cancelled_run_publishes_no_graph_at_any_phase() {
        let (facts, _length) = exception_path_call();
        let raw = raw_of(&facts);
        let contexts = contexts_of(&facts, &raw);
        for phase in PHASES {
            let _seam = checkpoint_seam(phase);
            match canonical_cfg(&facts, &raw, &contexts, &method(), &mut budget()) {
                Err(Error::Cancelled { .. }) => {}
                other => panic!("the run must stop at {phase:?} when cancelled, got {other:?}"),
            }
        }
    }

    #[test]
    fn a_loop_header_is_not_absorbed_by_the_body_that_jumps_back_to_it() {
        // A method whose body opens with a loop, which is what a `while` compiles to:
        //
        //   0: iload_0       the loop header, reachable only from the entry and the back edge
        //   1: ifle +6 -> 7  two successors, so the header cannot absorb anything
        //   4: goto -4 -> 0  the back edge: the body's only successor is the header
        //   7: return
        //
        // Fusion runs forward and merges a block with its unique successor. Read the other way
        // round, the body block at BCI 4 has a single successor too - the header - so a fusion
        // that did not care about direction merged the header *into* the body. The node was then
        // identified as BCI 4 while standing for BCI 0 first, which the postcondition refuses,
        // so an ordinary loop fell back and was told it had exceeded a bound it never reached.
        let facts = body(
            vec![
                plain(0, 0x1a), // iload_0
                instruction(
                    1,
                    0x9e,
                    3,
                    InstructionOperands {
                        branch_offset: Some(6),
                        ..operands(0x9e)
                    },
                ), // ifle 7
                instruction(
                    4,
                    0xa7,
                    3,
                    InstructionOperands {
                        branch_offset: Some(-4),
                        ..operands(0xa7)
                    },
                ), // goto 0
                plain(7, 0xb1), // return
            ],
            Vec::new(),
            8,
        );
        let graph = normalize(&facts);
        assert_eq!(graph.clones, 0, "a body without calls has no clones");
        assert_eq!(
            graph.blocks.len(),
            3,
            "the three blocks stay three: {:#?}",
            graph.blocks
        );
        assert!(
            graph.blocks.iter().any(|block| block.id.bci == 0),
            "the entry block is a node of its own: {:#?}",
            graph.blocks
        );
        assert!(
            graph
                .blocks
                .iter()
                .all(|block| block.origin.members.iter().any(|member| matches!(
                    member,
                    OriginMember::MethodPoint { bci, .. } if *bci == block.id.bci
                ))),
            "every node still maps back to the block it starts at: {:#?}",
            graph.blocks
        );
    }

    #[test]
    fn a_ret_the_payload_proves_no_return_point_for_stops_the_normalization() {
        // The guard the slice is stated over: a `ret` whose target list is empty is a `ret` the
        // bytes do not let this pass decide (3.4b's own rule: an empty list is not an
        // authorization to build an edge). The normalization stops instead of inventing a
        // successor, and the run keeps its raw facts.
        let (facts, _length) = exception_path_call();
        let raw = raw_of(&facts);
        let mut contexts = contexts_of(&facts, &raw);
        let row = contexts
            .returns
            .iter_mut()
            .find(|row| !row.targets.is_empty())
            .expect("the fixture's `ret` has a target");
        row.targets.clear();

        match canonical_cfg(&facts, &raw, &contexts, &method(), &mut budget())
            .expect("a stop is a fallback and not an error")
        {
            CanonicalOutcome::Fallback { message } => assert!(
                message.contains("proves no successor"),
                "the reason names the missing return point: {message}"
            ),
            CanonicalOutcome::Canonical(graph) => {
                panic!(
                    "no graph may be built from an unproven `ret`: {:?}",
                    graph.edges
                )
            }
        }
    }

    #[test]
    fn a_truncated_body_normalizes_its_reliable_prefix_and_says_so() {
        // The canonical graph of a body whose decode stopped early is the graph of the reliable
        // prefix, and it carries the raw graph's own completeness instead of claiming the whole
        // method: the driver marks the stage `Partial` from this field.
        let (facts, length) = exception_path_call();
        let raw = raw_of(&facts);
        let contexts = contexts_of(&facts, &raw);
        let mut truncated = facts.clone();
        truncated.stopped_at = Some(BytecodeStop::Instructions {
            bci: 8,
            class_offset: CODE_OFFSET + 8,
            code: "fixture_stop".to_string(),
        });
        truncated.execution = ExecutionReport::Partial {
            reason: jarde_reader::model::TerminationReason::Error {
                code: "fixture_stop".to_string(),
            },
            usage: UsageSnapshot::default(),
        };
        let raw = raw_cfg(&truncated, &mut budget()).expect("the prefix has a raw graph");
        let mut budget = budget();
        match canonical_cfg(&truncated, &raw, &contexts, &method(), &mut budget)
            .expect("the prefix normalizes")
        {
            CanonicalOutcome::Canonical(graph) => {
                assert!(
                    !graph.completeness.is_complete(),
                    "the graph of a truncated body does not claim the whole method"
                );
            }
            CanonicalOutcome::Fallback { message } => {
                panic!("the prefix must normalize: {message}")
            }
        }
        let _ = length;
    }

    #[test]
    fn the_pass_charges_the_dimensions_it_declares() {
        // The property 3.2's declared dimension set exists for: every dimension the pass bills is
        // one the table declares, and none of them stays at zero on a body that really clones.
        let (facts, _length) = exception_path_call();
        let mut budget = budget();
        let raw = raw_of(&facts);
        let contexts = contexts_of(&facts, &raw);
        let before = budget.usage();
        let graph = match canonical_cfg(&facts, &raw, &contexts, &method(), &mut budget)
            .expect("the ample budget is enough")
        {
            CanonicalOutcome::Canonical(graph) => *graph,
            other => panic!("the fixture normalizes, got {other:?}"),
        };
        let after = budget.usage();
        assert_eq!(
            after.normalization_clones,
            u64::try_from(graph.clones).expect("a clone count fits u64")
        );
        assert!(after.ir_items > before.ir_items + u64::try_from(graph.blocks.len()).unwrap());
        assert!(after.ir_edges > before.ir_edges);
        assert!(after.analysis_steps > before.analysis_steps);
    }
}
