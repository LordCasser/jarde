# Focused tests still required before acceptance

The permanent real-class test in `after/p3_conditional_switch_fallthrough.rs` must run on the exact frozen input and prove all of these from the produced report: the complete method is structured, case 2 appears once, the case-1-to-case-2 route remains fallthrough, the two transfers at BCIs 89 and 114 map to their own `break;`, BCI 171 stays outside the case body, all 11 physical blocks have one owner, every physical BCI has source, and no Loop rule is reported. Keep the whole outer class, `check()`, and `Inner` intact.

Add small focused graph/region tests (or real bytecode fixtures if the existing test seam cannot state the edge identities) for each refusal boundary. Each negative must name the exact rejected evidence and leave no published Switch/If source:

- Add a canonical Exception, Call, or subroutine Return incident edge hidden by the normal-flow view; each must refuse rather than certify a normal path.
- Add a duplicate canonical Normal edge, differing clone `path`, external incoming edge into an interior node, and an extra external/case-entry predecessor; each must refuse without claiming the affected block.
- Add a cycle/backedge, two distinct case exits, a non-adjacent target, a terminal with no exact physical return/throw operation, and a terminal operation with an outgoing canonical edge; each must refuse.
- Preserve the positive contrasts: all paths to the common join returns `Some(empty)` (no fallthrough), one unique adjacent case target plus join is the mixed positive, and exact physical return/throw leaves are accepted only with zero canonical outgoing edges and closed incoming ownership.
- Exercise `SwitchBreak` targeting the nearest switch, a parent switch across a nested switch, and a parent switch across an inner loop. Only the nearest unlabelled break is presentable; the two cross-scope cases must retain an explicit refusal.
- Use zero/tight AnalysisSteps budgets and cancellation during adjacency construction, DAG traversal, and incoming validation. Verify the normal Stop result and that no partial map, Region, owner, or report is published.
- Keep straight-line fallthrough, shared labels, ordinary switches, nested switches, loop/switch nesting, and the existing full switch-expression tests as regressions.

No such tests were run or claimed by this draft. The test seam must not synthesize a normal-only graph by dropping hidden canonical edges; the negative must preserve both full canonical edges and the normal-flow projection so it exercises the exact proof boundary.
