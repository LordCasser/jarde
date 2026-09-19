//! ① The normal-flow view: the canonical blocks and their **plain transfers**, as a graph the
//! structure layer runs dominance and cycle questions on (P3 1.2's first row of the four-layer
//! table).
//!
//! # What enters this view
//!
//! Only edges the canonical graph itself calls plain transfers: [`CanonicalEdgeKind::Normal`] (a
//! fall-through, a conditional branch, a `goto`, a switch target) and [`CanonicalEdgeKind::Return`]
//! (the successor of one call context's `ret`, which is a plain transfer once that context decided
//! where it returns). Everything else stays out:
//!
//! * [`CanonicalEdgeKind::Exception`] — an exception edge is a statement about a handler, not about
//!   the control flow a Java statement falls through. A view that kept it would turn every
//!   protected call into a branch and every `try` into a diamond;
//! * [`CanonicalEdgeKind::Call`] — the entry into a `jsr` context's clone. A subroutine is not a
//!   structured control-flow construct Java can spell, and the clone structure is exactly what the
//!   recovery subset treats as unprovable.
//!
//! # What this view is not allowed to do
//!
//! It **does not delete** an edge of the canonical graph, **does not write back** to it, and is
//! **not** the source of any exception semantics: the exception facts stay what they are — the
//! canonical graph's own throw sites and handler rows — and the recovery layer reads them where it
//! needs them, as facts. The view is a derived, read-only *projection*: a second graph over the same
//! blocks with a subset of the edges, plus the algorithm results. The canonical graph remains the
//! only description of the method, and a block the projection does not reach is reported as
//! unreachable *in the view* (which is a fact about the projection) rather than removed.
//!
//! # Billing
//!
//! The blocks and kept edges are charged to `IrItems` and the algorithm walks to `AnalysisSteps`
//! **before** they run: petgraph's dominator and SCC passes have no interruption hook, so a run that
//! cannot afford them refuses to enter them (see [`crate::stop`]).

use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use petgraph::algo::{dominators, tarjan_scc};
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;

use crate::stop::{StopReason, charge};

/// Which kinds of canonical edge the projection kept, so that a caller can state what it left out.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExcludedEdges {
    /// Exception edges, which describe handlers and not fall-through.
    pub exception: usize,
    /// `jsr` context entries, which describe a subroutine and not a structured transfer.
    pub call: usize,
}

/// The projection of one canonical graph onto its plain transfers, with its algorithm results.
///
/// Built once per run and read by the structure layer ([`crate::region`]); nothing else owns a
/// graph over the canonical blocks. Node identity is the index into [`Self::ids`], which is the
/// order the canonical graph publishes its blocks in, so a node id is stable for one payload and
/// never reused across runs.
#[derive(Clone, Debug)]
pub struct NormalFlowView {
    ids: Vec<CanonicalBlockId>,
    index: BTreeMap<CanonicalBlockId, usize>,
    graph: DiGraph<usize, ()>,
    post_dominators: Vec<Option<usize>>,
    return_edges: usize,
    excluded: ExcludedEdges,
    cycles: usize,
}

impl NormalFlowView {
    /// The projection of one canonical graph, with its dominators and cycles already computed.
    ///
    /// Every counted charge happens before the work it pays for (see the module documentation).
    pub fn build(canonical: &CanonicalCfg, budget: &mut Budget) -> Result<Self, StopReason> {
        let blocks = canonical.blocks();
        let nodes = u64::try_from(blocks.len()).unwrap_or(u64::MAX);
        charge(budget, CountedBudgetDimension::IrItems, nodes, None)?;

        let mut ids = Vec::with_capacity(blocks.len());
        let mut index = BTreeMap::new();
        for block in blocks {
            index.insert(block.id().clone(), ids.len());
            ids.push(block.id().clone());
        }

        let mut graph = DiGraph::<usize, ()>::with_capacity(blocks.len(), canonical.edges().len());
        for node in 0..ids.len() {
            graph.add_node(node);
        }
        let mut excluded = ExcludedEdges::default();
        let mut return_edges = 0usize;
        let mut kept = 0u64;
        for edge in canonical.edges() {
            // Both endpoints are published blocks: the canonical graph states its own edges over
            // its own nodes, and a payload whose edge names an unpublished block would be a
            // malformed payload rather than something to paper over here.
            let Some(from) = index.get(edge.from()).copied() else {
                continue;
            };
            let Some(to) = index.get(edge.to()).copied() else {
                continue;
            };
            match edge.kind() {
                CanonicalEdgeKind::Normal => kept += 1,
                CanonicalEdgeKind::Return { .. } => {
                    kept += 1;
                    return_edges += 1;
                }
                CanonicalEdgeKind::Exception { .. } => {
                    excluded.exception += 1;
                    continue;
                }
                CanonicalEdgeKind::Call { .. } => {
                    excluded.call += 1;
                    continue;
                }
            }
            graph.add_edge(
                petgraph::graph::NodeIndex::new(from),
                petgraph::graph::NodeIndex::new(to),
                (),
            );
        }
        charge(budget, CountedBudgetDimension::IrEdges, kept, None)?;
        // One step per block for each of the two walks below, billed before either runs.
        charge(
            budget,
            CountedBudgetDimension::AnalysisSteps,
            nodes.saturating_mul(2),
            None,
        )?;

        let post_dominators = immediate_post_dominators(&graph);
        let cycles = cycle_count(&graph);
        Ok(Self {
            ids,
            index,
            graph,
            post_dominators,
            return_edges,
            excluded,
            cycles,
        })
    }

    /// Every block of the projection, in the canonical graph's own order.
    pub fn ids(&self) -> &[CanonicalBlockId] {
        &self.ids
    }

    /// The number of blocks the projection holds — all of them; the view never drops a node.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether the method has no canonical block at all.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// The block one node stands for.
    pub fn id_of(&self, node: usize) -> Option<&CanonicalBlockId> {
        self.ids.get(node)
    }

    /// The node one block is, when the projection holds it.
    pub fn index_of(&self, id: &CanonicalBlockId) -> Option<usize> {
        self.index.get(id).copied()
    }

    /// The nodes one node transfers to, in the canonical graph's edge order.
    pub fn successors(&self, node: usize) -> Vec<usize> {
        let Some(node) = self.node_index(node) else {
            return Vec::new();
        };
        self.graph
            .neighbors(node)
            .map(|neighbour| neighbour.index())
            .collect()
    }

    /// The blocks one block transfers to, in the canonical graph's edge order.
    pub fn successor_ids(&self, id: &CanonicalBlockId) -> Vec<CanonicalBlockId> {
        match self.index_of(id) {
            Some(node) => self
                .successors(node)
                .into_iter()
                .filter_map(|node| self.ids.get(node).cloned())
                .collect(),
            None => Vec::new(),
        }
    }

    /// The nodes that transfer to one node, in the canonical graph's edge order.
    pub fn predecessors(&self, node: usize) -> Vec<usize> {
        let Some(node) = self.node_index(node) else {
            return Vec::new();
        };
        self.graph
            .neighbors_directed(node, petgraph::Direction::Incoming)
            .map(|neighbour| neighbour.index())
            .collect()
    }

    /// The nearest node every path out of `node` passes through, in the projection.
    ///
    /// Computed over the projection's plain transfers with a virtual exit joining every block that
    /// transfers nowhere (a `return`, or a block whose only successors were exception or `jsr`
    /// edges) — the definition a join point needs. A block with no path to the exit (a live block
    /// reachable only through an exception edge) has none, and a caller that needs a join treats
    /// that as "not provable here" rather than as "the method ends".
    pub fn immediate_post_dominator(&self, node: usize) -> Option<usize> {
        self.post_dominators.get(node).copied().flatten()
    }

    /// Whether every path leaving `node` meets `join` before it leaves the method.
    ///
    /// The walk is bounded by the projection's own size — a block is expanded once — so it cannot
    /// run longer than the graph it reads; the caller's counted bound was already paid when the
    /// projection was built.
    pub fn reaches(&self, node: usize, join: usize) -> bool {
        let mut seen = BTreeSet::new();
        let mut worklist = vec![node];
        while let Some(current) = worklist.pop() {
            if current == join {
                continue;
            }
            if !seen.insert(current) {
                continue;
            }
            let successors = self.successors(current);
            if successors.is_empty() {
                return false;
            }
            worklist.extend(successors);
        }
        true
    }

    /// The blocks reachable from one node by plain transfers, the node included.
    pub fn reachable(&self, node: usize) -> BTreeSet<usize> {
        let mut seen = BTreeSet::new();
        let mut worklist = vec![node];
        while let Some(current) = worklist.pop() {
            if current >= self.graph.node_count() || !seen.insert(current) {
                continue;
            }
            worklist.extend(self.successors(current));
        }
        seen
    }

    /// How many of the projection's blocks lie on a cycle.
    ///
    /// The provable subset is acyclic: a block that can reach itself is a loop, and loops are 1.3b.
    /// The count is stated rather than acted on here — the structure layer reads it to decide
    /// whether it may attempt a region at all.
    pub fn cyclic_blocks(&self) -> usize {
        self.cycles
    }

    /// How many plain `ret` successors the projection kept.
    pub fn return_edges(&self) -> usize {
        self.return_edges
    }

    /// What the projection left out, and how much of it.
    pub fn excluded(&self) -> ExcludedEdges {
        self.excluded
    }

    /// The projection's own edge count, which is the size of what it kept.
    pub fn kept_edges(&self) -> usize {
        self.graph.edge_count()
    }

    /// Whether one edge kind is part of this view, stated for the reader of a report.
    pub fn keeps(kind: CanonicalEdgeKind) -> bool {
        matches!(
            kind,
            CanonicalEdgeKind::Normal | CanonicalEdgeKind::Return { .. }
        )
    }

    fn node_index(&self, node: usize) -> Option<petgraph::graph::NodeIndex> {
        (node < self.graph.node_count()).then(|| petgraph::graph::NodeIndex::new(node))
    }
}

/// The immediate post-dominator of every block, over plain transfers, with a virtual exit.
fn immediate_post_dominators(graph: &DiGraph<usize, ()>) -> Vec<Option<usize>> {
    let nodes = graph.node_count();
    let exit = nodes;
    let mut reversed = DiGraph::<usize, ()>::with_capacity(nodes + 1, graph.edge_count() + nodes);
    for _ in 0..=nodes {
        reversed.add_node(0);
    }
    for edge in graph.edge_references() {
        // Reversed: a dominator of the reversed graph is a post-dominator of this one.
        reversed.add_edge(
            petgraph::graph::NodeIndex::new(edge.target().index()),
            petgraph::graph::NodeIndex::new(edge.source().index()),
            (),
        );
    }
    for node in 0..nodes {
        let index = petgraph::graph::NodeIndex::new(node);
        if graph.neighbors(index).next().is_none() {
            // A virtual exit is what every block that transfers nowhere leads to: the original graph
            // gains the edge `sink -> exit`, and reversing that edge is what makes the exit the root
            // of the walk below it. Adding it the other way round (sink <- exit) would leave the root
            // with no outgoing edge at all, and every post-dominator would come back empty.
            reversed.add_edge(petgraph::graph::NodeIndex::new(exit), index, ());
        }
    }
    let dominators = dominators::simple_fast(&reversed, petgraph::graph::NodeIndex::new(exit));
    (0..nodes)
        .map(|node| {
            dominators
                .immediate_dominator(petgraph::graph::NodeIndex::new(node))
                .map(|dominator| dominator.index())
                .filter(|dominator| *dominator != exit)
        })
        .collect()
}

/// How many blocks lie on a cycle of the projection.
fn cycle_count(graph: &DiGraph<usize, ()>) -> usize {
    tarjan_scc(graph)
        .into_iter()
        .filter(|component| {
            component.len() > 1 || {
                let node = component[0];
                graph.neighbors(node).any(|neighbour| neighbour == node)
            }
        })
        .map(|component| component.len())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_view_states_exactly_which_edge_kinds_it_keeps() {
        assert!(NormalFlowView::keeps(CanonicalEdgeKind::Normal));
        assert!(NormalFlowView::keeps(CanonicalEdgeKind::Return {
            call_site: 1
        }));
        assert!(
            !NormalFlowView::keeps(CanonicalEdgeKind::Call { call_site: 1 }),
            "a jsr context entry is not a structured transfer"
        );
        assert!(
            !NormalFlowView::keeps(CanonicalEdgeKind::Exception { handler_ordinal: 0 }),
            "an exception edge describes a handler, not fall-through"
        );
    }
}
