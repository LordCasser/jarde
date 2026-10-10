# Attach points and focused fragment

This is deliberately a call-site fragment, not a claimed compiling patch. The private v2 production patch is not applied to the working tree, so the current source has no `Walker::switch_fallthroughs` method. After root applies the candidate, place the helper in `crates/jarde-java/src/region.rs`'s existing `#[cfg(test)] mod tests`; it can access the private method without widening visibility.

```rust
fn assert_real_switch_probe(
    ir: &MethodIr,
    groups: &[(Vec<i64>, bool, u32)],
    targets: &BTreeMap<u32, usize>,
    join: Option<usize>,
    join_bci: Option<u32>,
    dispatch_node: usize,
    switch_bci: u32,
    expected: Option<BTreeMap<u32, u32>>,
) {
    let canonical = ir.canonical().expect("canonical CFG");
    let ssa = ir.ssa().expect("SSA");
    let code = ir.code().expect("decoded code");
    let mut budget = probe_budget();
    let operations = Operations::of(code, ir.constant_pool());
    let chains = crate::concat::plan_four_conditional_strings(
        ssa, canonical, &operations, &crate::build::FieldCopies::default(), &mut budget,
    ).expect("concat plan");
    let view = NormalFlowView::build(canonical, &mut budget).expect("normal-flow projection");
    let sites = crate::init::Sites::empty();
    let mut walker = Walker {
        canonical, view: &view, ssa, operations: &operations, pool: ir.constant_pool(),
        chains: &chains, sites: &sites, code, method_synchronized: Some(false),
        has_reachable_explicit_monitor: false, return_is_boolean: false,
        handlers: &code.exception_handlers, profile: &crate::pass::JAVA_8,
        budget: &mut budget, catch_joins: BTreeSet::new(), fragmented: None,
        excluded_edge_nodes: BTreeSet::new(), visited: BTreeSet::new(),
        unclosed_tail_at: None, own_monitor: None, depth: 0,
    };
    assert_eq!(
        walker.switch_fallthroughs(
            groups, targets, join, join_bci, dispatch_node, switch_bci,
        ).expect("proof completes"),
        expected,
    );
}
```

The fixture observer must supply `groups`, `targets`, `join`, and `dispatch_node` from this exact `ir`/`view` pair. Do not copy node indices from another run. For hidden-edge negatives, assert the edge in `canonical.edges()` and its corresponding projection behavior with `view.successors()` *before* invoking this helper. The current normal-flow projection excludes `Exception` and `Call`, and includes `Return`; `Return` therefore supplies a distinct negative boundary.

A `SwitchBreak` scope test is not the direct proof helper: exercise `recover` from the same real method and inspect its `Region`/report tree. Confirm the leaf's `switch_bci` and `join` identify the nearest switch; nested switch and inner-loop examples must keep the parent transfer refused rather than emit an unlabeled break.
