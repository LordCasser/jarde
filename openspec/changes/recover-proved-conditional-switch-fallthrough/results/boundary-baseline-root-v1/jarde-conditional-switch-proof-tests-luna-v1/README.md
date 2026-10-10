# Conditional switch proof boundary tests (private draft)

This directory is a test-design artifact only. No repository files were changed and no Cargo, JDK, CLI, or Git command was run. The code below has not been compiled.

## What the reviewed implementation actually proves

The proposed production entry is `Walker::switch_fallthroughs` in the private v2 patch. It first builds `incoming` and `outgoing` from **all** `CanonicalCfg::edges()`, charging and polling once per canonical edge. It checks that the dispatch's canonical Normal targets exactly equal the normal-flow projection, including no duplicate target. It then checks each case closure as a DAG, requiring Normal-only outgoing edges to match the projection; no more than one adjacent case exit; and exact Return/Throw terminal operations for leaves with no successors. It later checks incoming rows at every case entry, closure ownership, and the full dispatch and prior-case predecessor set.

The direct consumers are also important: `switch_break_transfer` rechecks the exact single canonical Normal edge from a transfer terminal to the proved join; `switch_break_branch` rechecks both canonical branch edges and their normal-flow identities. Tests that only call a predicate over a hand-made vector of Normal edges do not exercise these production positions.

## Existing test seam and its limit

`crates/jarde-java/src/region.rs` has a `#[cfg(test)] mod tests` with scalar edge predicates (`closed_transfer_edges`, `exact_straight_chain_edges`, `exact_normal_predecessors`) and a precedent for constructing a `Walker` from one real `analyze_method_ir` result (`cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir` in the current source). The v2 conditional-switch integration test also establishes the frozen CF12 class and public report checks.

There is no existing constructor or mutation API for `CanonicalCfg` in `jarde-java`: `CanonicalCfg` and `CanonicalEdge` are defined in `jarde-jvm/src/canonical.rs`; their fields and `CanonicalBlockId`'s fields are `pub(crate)`. `CanonicalCfg::edges()` is read-only. The synthetic bytecode graph constructors live in `jarde-jvm`'s own `canonical.rs` unit-test module, where the `jarde-java::region::Walker` proof is not callable. So a fabricated or edge-mutated `CanonicalCfg` cannot be made in a `jarde-java` test using current APIs. This is the exact obstacle to direct synthetic cases for duplicate edges, altered clone paths, and impossible canonical terminals.

The minimal honest choices are: (a) commit real frozen class fixtures and invoke the actual analysis/recovery path for each representable graph; or (b) if root requires graph-level synthetic coverage, add a deliberately scoped internal test seam in a later change. Do not call a synthetic `(kind, from, to)` list a full proof test. This draft does not propose a production graph abstraction or a public constructor.

## Concrete Rust test skeleton for real graph fixtures

Add this helper inside `region.rs`'s existing test module, adapting only the class bytes and exact method selector after a real fixture is built and inspected. It follows the existing real-IR test's actual `Walker` fields and calls the production proof method. The `groups` and `targets` arguments must be filled from the analyzed method's decoded switch and `NormalFlowView`, not guessed from source line numbers.

```rust
fn assert_real_switch_probe(
    ir: &MethodIr,
    branch_bci: u32,
    groups: &[(Vec<i64>, bool, u32)],
    targets: &BTreeMap<u32, usize>,
    join: Option<usize>,
    join_bci: Option<u32>,
    dispatch_node: usize,
    expect: Option<BTreeMap<u32, u32>>,
) {
    let canonical = ir.canonical().expect("fixture canonical CFG");
    let ssa = ir.ssa().expect("fixture SSA");
    let code = ir.code().expect("fixture decoded body");
    let mut budget = test_budget_with_large_analysis_steps();
    let operations = Operations::of(code, ir.constant_pool());
    let chains = crate::concat::plan_four_conditional_strings(
        ssa,
        canonical,
        &operations,
        &crate::build::FieldCopies::default(),
        &mut budget,
    ).expect("existing concat plan");
    let view = NormalFlowView::build(canonical, &mut budget).expect("real canonical projection");
    let mut walker = Walker {
        canonical,
        view: &view,
        ssa,
        operations: &operations,
        pool: ir.constant_pool(),
        chains: &chains,
        sites: &crate::init::Sites::empty(),
        code,
        method_synchronized: Some(false),
        has_reachable_explicit_monitor: false,
        return_is_boolean: false,
        handlers: &code.exception_handlers,
        profile: &crate::pass::JAVA_8,
        budget: &mut budget,
        catch_joins: BTreeSet::new(),
        fragmented: None,
        excluded_edge_nodes: BTreeSet::new(),
        visited: BTreeSet::new(),
        unclosed_tail_at: None,
        own_monitor: None,
        depth: 0,
    };
    assert_eq!(
        walker.switch_fallthroughs(
            groups, targets, join, join_bci, dispatch_node, branch_bci,
        ).expect("proof completes"),
        expect,
    );
}
```

This is a call-site template, not a compilable test yet: root must supply the accepted v2 signature and fixture observer values. In the reviewed patch the signature order is `(groups, targets, join, join_bci, dispatch_node, switch_bci)`; the current checked-in source predates this patch and therefore does not expose that method/signature.

For the real exception-edge case, the fixture must produce one case closure block with a canonical `Exception` edge while `NormalFlowView::successors(node)` still reports its normal successor. Before asserting refusal, the test should assert those two facts from `ir.canonical()` and `view`, then call the helper and require `None`. That guards against accidentally testing a Normal-only substitute graph.

## Boundary matrix for root fixture construction

Every row below must be backed by a class whose exact bytes are frozen under the change results, and by a test that obtains its `MethodIr`, canonical graph, projection, decoded operations, and recovered result from the same analysis. Record the exact physical method identity and block identities from the observer run. Expected result is `None` from the proof, or the production switch refusal, with no structured Switch/If source for that attempted switch.

| Case | Exact evidence to assert before calling proof | Fixture direction / current constraint |
|---|---|---|
| Exceptional incident edge | A case-closure block has canonical `Exception`; normal projection has its Normal successor; complete edge list remains unchanged | Java `try/catch` around a throwing call in a switch arm. Use actual observed block, not assumed invoke block boundaries. |
| Call incident edge | A case-closure block has canonical `Call`; its normal projection excludes that edge | Requires real legacy `jsr`/`ret` class bytes. Reuse a frozen legacy fixture only if an actual switch arm reaches the cloned node; otherwise root needs a dedicated frozen `.class`. |
| Return incident edge | A case-closure block has canonical `Return`; the projection includes it; the switch proof rejects the non-Normal row | Same legacy-subroutine limitation. `Return` is intentionally different from `Call` in `NormalFlowView`. |
| Duplicate Normal | Two identical canonical `(from, Normal, to)` rows exist while the projection has one logical successor; proof refuses | Current canonical assembly sorts but does not visibly deduplicate `CanonicalEdge`s; nevertheless the raw CFG's edge producer must be inspected and the actual duplicate proved present before writing this assertion. Do not manufacture rows. |
| Differing clone path | Entry and interior identity paths differ; full edge identities show the mismatch | Needs a real `jsr` clone reached from a switch fixture. `CanonicalBlockId::path()` is readable, but IDs cannot be built/changed from `jarde-java`. |
| External incoming into interior | A non-closure block has a canonical Normal edge into a strict interior closure node | Bytecode fixture with an extra branch into a block normally reached from the case entry; inspect physical targets and canonical edges. |
| Extra external/case-entry predecessor | Incoming rows to a case entry contain a source other than dispatch and a proved earlier case closure (or duplicate source row) | Use a branch from outside switch to a case target, if verifier-valid. Assert all incoming rows before proof. |
| Cycle/backedge | A reachable node in the case closure has a path back to a gray node; all canonical rows are retained | Source fixture with a loop in a case. A loop may be independently recovered; test proof directly and separately assert switch behavior. |
| Two distinct case exits | One case closure reaches two different other case-entry BCIs | Conditional branch fixture where arms go to two labels; must verify neither target is the common join and exact target order. |
| Non-adjacent target | Unique case exit exists but sorted label position is not immediately next | Small switch with a skipped intermediate case target in source/layout; inspect actual decoded labels/targets and fail the adjacency ordering check. |
| Unknown/nonterminal leaf | A leaf has no canonical successor but its final physical operation is neither exact Return nor Throw | Requires a valid classfile shape whose CFG ends in an unrecognized terminal/decode boundary. If analysis refuses before canonical CFG, it does not reach this proof and is not evidence for this row. |
| Terminal with outgoing edge | Exact physical terminal Return/Throw is present and canonical outgoing rows are nonempty | This conflicts with ordinary JVM CFG terminal semantics. Only test if a real canonical input can produce it; otherwise document as unreachable under the canonical producer invariant, not a fabricated graph case. |
| Positive: join-only | Every path reaches common join; proof returns `Some(empty)` | Existing real acyclic switch with no adjacent-case path. Assert the proof result, not only printed text. |
| Positive: adjacent + join | Exactly one case closure reaches its next case entry; other paths reach join; map contains one source-target pair | Existing frozen CF12 method can supply the candidate if the actual observer confirms the revised branch graph shape. Keep separate from the full report's source checks. |
| Positive: return/throw | A closure path ends at physical `Operation::Return` or `Operation::Throw`, has zero canonical outgoing rows, and has closed incoming ownership | A real method containing a terminating switch arm. Check exact terminal operation and outgoing-edge count first. |

The listed unsupported/malformed graph cases are not allowed to be “covered” by directly feeding ad hoc edges to `exact_normal_predecessors` or by dropping the hidden canonical edge before building `NormalFlowView`.

## Budget and cancellation test plan (no asserted numeric threshold yet)

The patch shows these proof phases and their actual accounting points:

1. Adjacency build: one AnalysisSteps charge and poll for each canonical edge.
2. DAG walk: one charge/poll for every popped work item, plus one for each successor considered.
3. Closed/owner validation: a bulk charge of `2 * post_scan_nodes + entries.len()`, polls for owner insertions and entry scans, then a row-count charge and per-row polls for every entry's incoming edges.
4. `switch_break_transfer` and `switch_break_branch`: separate full-edge scans each poll and charge per canonical edge.

When root runs the tests, first measure the real ample successful run's delta on the exact budget object immediately before/after `switch_fallthroughs`; do not assume `full_usage - 1` reaches this proof. For each phase, use the real fixture and low limits, increase the limit until `StopReason::Budget { dimension: AnalysisSteps, at: Some(switch_bci), .. }` lands in the intended phase, and assert the observed `usage().analysis_steps` and `at` from that run. The current `CancellationToken` API is an atomic flag with `cancel()` and `is_cancelled()`; it has no deterministic callback or phase barrier. Existing tests can deterministically prove a pre-cancelled proof stops at its first poll, but they cannot reliably cancel *during* adjacency construction, DAG traversal, or incoming validation without a test hook or scheduler race. Use AnalysisSteps limits to force `StopReason::Budget` inside each phase and prove no partial map/Region/report; treat phase-specific mid-run cancellation as an explicit test-seam gap rather than a deterministic test. A concurrent thread that races `cancel()` against these short loops would be flaky. `switch_fallthroughs` itself returns only `Result<Option<Map>, StopReason>` and has no output parameter where a partial map can escape.

Do not write a numeric budget constant until root has observed the actual charge trace on the accepted patch and frozen fixture. The unit test should preserve raw failure output and report the observed threshold rather than guess.

## Switch/loop nesting

The proof frame carries switch identity and target through nested region walks. The actual integration assertions belong at recovered Region level: nearest-switch `break;` is valid; a transfer to a parent switch crossing a child switch or an inner loop must not become an unlabelled `SwitchBreak`. The private patch's `switch_break_transfer` checks only the frame's `switch_branch_bci`, `switch_join`, exact target and canonical outgoing edge; nested-scope correctness therefore needs real nested-switch / switch-in-loop fixtures exercising `region_at` and final report. A direct call to `switch_fallthroughs` cannot prove those presentation-scope rules.

Root must freeze those classes after observing their bytecode/IR shape. If a fixture cannot be expressed by Java source without changing the actual target shape, use a frozen valid classfile and state how it was produced; do not claim this draft covered the nesting cases.
