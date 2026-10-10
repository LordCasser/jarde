# Proposal: test the full switch-path certificate using real block identities

Private architecture proposal only. No repository files were changed; no Cargo, JDK, CLI, or Git command was run. This is not compiled, applied, or accepted.

## Finding

The complete graph certificate in the reviewed `private-implementation-luna-v2/implementation.patch` is a pure read of five inputs plus budget mutation:

- all canonical edge rows, from which it builds complete incoming/outgoing adjacency;
- the already built `NormalFlowView` projection;
- decoded switch groups/targets and join identity;
- SSA terminal-instruction lookup plus decoded `Operations`;
- the existing mutable `Budget`.

It does not mutate `Walker`, `visited`, a Region, or an owner map outside its local variables. That makes this one method a good candidate for a private free-function extraction. Tests can take a real analysis's canonical IDs and complete edge iterator, add or replace a row using only IDs already published by that IR, and pass that iterator to the **same complete certificate**. No new graph value type, constructor, public API, service, or alternate proof is needed. Keep the real view unchanged for hidden-edge tests: `Exception` and `Call` must be rejected from full rows even though the projection excludes them; `Return` must be rejected as a non-Normal row even though the projection includes it.

## Proposed private signature

```rust
fn prove_switch_fallthroughs<'edge, E>(
    edges: E,
    view: &NormalFlowView,
    ssa: &SsaTable,
    operations: &Operations,
    groups: &[(Vec<i64>, bool, u32)],
    targets: &BTreeMap<u32, usize>,
    join: Option<usize>,
    join_bci: Option<u32>,
    dispatch_node: usize,
    switch_bci: u32,
    budget: &mut Budget,
) -> Result<Option<BTreeMap<u32, u32>>, StopReason>
where
    E: IntoIterator<
        Item = (
            &'edge CanonicalBlockId,
            CanonicalEdgeKind,
            &'edge CanonicalBlockId,
        ),
    >,
```

The canonical production call supplies exactly the present rows:

```rust
let edges = self
    .canonical
    .edges()
    .iter()
    .map(|edge| (edge.from(), edge.kind(), edge.to()));
let fall_throughs = prove_switch_fallthroughs(
    edges,
    self.view,
    self.ssa,
    self.operations,
    &groups,
    &targets,
    join_node,
    join_bci,
    node,
    branch.bci(),
    self.budget,
)?;
```

The function body is the current full `switch_fallthroughs` body from the v2 patch, moved with mechanical receiver substitutions only:

- `self.canonical.edges()` is replaced by the `edges` iterator;
- `self.view` becomes `view`;
- `self.ssa` supplies the existing terminal query inline: `ssa.block(&id).and_then(|block| block.instructions().last().map(|instruction| instruction.bci()))` (equivalent to `Walker::terminal_bci`);
- `self.operations` becomes `operations`;
- all other predicates, charge amounts, poll locations, sort/dedup checks, result shape, and early returns remain byte-for-byte equivalent where practical.

Leave `switch_break_transfer` and `switch_break_branch` on `Walker`: they are later presentation checks, distinct from the graph-wide case-path certificate and should continue to re-read production canonical edges. This proposal does not claim to test those two methods with transformed rows.

## Why this is a small extraction

The helper has no dependency on `code`, `pool`, `chains`, `sites`, loop targets, Walker depth, or any mutable region state. The function's inputs are just the evidence it reads today. `ssa.block(id)` avoids adding a terminal callback or a terminal-data struct. `Operations` is already the decoded terminal evidence owner. The edge iterator keeps the production representation intact and permits a test to perturb a finite set of rows without constructing `CanonicalCfg` or `CanonicalBlockId` values.

Potential borrow concern: the call borrows distinct `Walker` fields (`canonical`, `view`, `ssa`, `operations`, and `budget`). Rust's field-sensitive borrowing should permit it. If the current compiler rejects the combined call, bind the immutable field references and edge iterator to locals immediately before calling; do not clone the CFG or move adjacency construction back into a separate test implementation.

The helper still clones IDs into the same `BTreeMap` adjacency it already builds, so it does not add a second graph walk or change its asymptotic cost. The test iterator adds no work in production beyond the current `.edges()` scan.

## Test recipe using the actual complete function

Inside `region.rs`'s existing `#[cfg(test)] mod tests`, start from one real `MethodIr` with a decoded switch and call the helper with:

```rust
let base_rows: Vec<_> = canonical
    .edges()
    .iter()
    .map(|edge| (edge.from(), edge.kind(), edge.to()))
    .collect();
let ids: Vec<_> = canonical.blocks().iter().map(|block| block.id()).collect();
```

All endpoints for variants must be selected from `ids`; clone paths must come from actual `CanonicalBlockId::path()` values. For every variant, retain the same `NormalFlowView` built from the unmodified real canonical graph, then call `prove_switch_fallthroughs` with the variant iterator. This is intentional for injected hidden/external evidence: the certificate has to reconcile the full rows against the real projection. Assert `None` from the full certificate, not merely a local edge predicate.

The base case calls the helper with `base_rows.iter().copied()` and must equal the expected `Some(map)` or `Some(empty)` observed from the real method. Each negative should first assert its variant has the exact row in question and that the real projection has the stated behavior; then assert the full helper refuses it.

### Variants expressible with existing real IDs and rows

- **Hidden Exception/Call/Return:** append a row with the relevant kind between two IDs already in the fixture. For Exception, use an ordinal present in that real graph's handler rows; for Call/Return, use a call site from a real `jsr` method's clone path. Verify the full proof rejects. `Exception`/`Call` are absent from the original view; `Return` is present in the projection, but is still not an admissible Normal certificate edge.
- **Duplicate Normal:** duplicate an existing dispatch edge or closure edge exactly. Assert the full helper refuses, and verify the projection has only the original logical successor. This exercises duplicate detection in the dispatch and closure checks without fabricating identities.
- **External/interior incoming:** append a Normal row from an existing ID outside the probed closure into an interior closure ID; similarly append an external row into a case-entry ID. The existing view remains unchanged and the full incoming validation must refuse.
- **Differing clone path:** use a real switch-containing `jsr` clone method and pass a `targets` entry whose `view` index names an existing clone ID with a path different from the dispatch ID; assert the path gate returns `None`. The case BCI and replacement ID must come from the same observed IR. If no existing legacy fixture has a switch under a clone path, a dedicated frozen class is required; do not manufacture `CanonicalBlockId`. As a second reconciliation test, an extra row between existing IDs whose paths differ should be refused against the unchanged real projection, but that proves row/projection disagreement rather than the start-path guard itself.

These variants intentionally test evidence perturbations against one real view. They exercise the full production certificate and are suitable as unit tests of fail-closed reconciliation. They do not assert that a valid classfile can naturally produce each injected extra row.

### Cases requiring a genuine altered normal-flow graph

An injected Normal edge with the original `NormalFlowView` is rejected at canonical/projection equivalence before it can exercise later DAG logic. To test the *cycle detector itself*, a two-exit closure, a real non-adjacent fallthrough, or a Return/Throw leaf under a matching altered projection, use a separate valid frozen class fixture and rebuild `CanonicalCfg` and `NormalFlowView` through the ordinary `analyze_method_ir` path. Do not treat “rejected because it disagrees with the old view” as evidence for the later DAG gate.

The currently frozen CF12 conditional-switch class is a positive case for one adjacent case route plus join paths only if the observer confirms that exact shape on the applied patch. A small real switch with every arm reaching the join supplies `Some(empty)`. A real returning arm with no canonical outgoing row tests Return/Throw leaf acceptance. Unknown terminal/outgoing terminal cases are fixture- or canonical-producer questions; do not synthesize a block or operation for them.

## Testability boundaries and non-goals

This extraction makes the complete certificate directly testable with real identity values and controlled full-edge rows; it does not make every graph topology a classfile-realizable case. It also cannot deterministically cancel midway through adjacency, DAG, or incoming scans: the existing token only exposes an atomic cancellation flag, with no per-phase hook. Existing AnalysisSteps charging can force Stops in those phases once root measures the exact bounds. A deterministic phase-cancel test would need a separate test seam and is not included here.

The extractor does not cover nearest-switch lexical ownership. Keep those tests at `recover`/Region/report level using actual nested-switch and switch-in-loop class fixtures. Nor does it assert that malformed augmented edge sets are valid canonical products. Their purpose is to prove this exact certificate refuses when its complete input evidence is inconsistent with the normal-flow projection or closure invariants.

## Application range

In the v2 patch, replace only the `impl Walker` method block beginning at the comment `Prove the case-entry outcomes ...` and ending after its `Ok(Some(fallthroughs))` return with the free function above. At the existing call site in `switch_region`, replace the method call with the iterator-backed helper call. Add tests in the existing region unit-test module. No production change is proposed to `CanonicalCfg`, `NormalFlowView`, the report schema, the Region enum, or the proof's control flow.
