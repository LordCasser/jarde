# SequentialLoops2 same-IR diagnostic candidate

This is a private, test-only candidate for root review. It targets the frozen javac 23 original `cf11.UpstreamSequentialLoops2.class` and reuses `init.rs` test helpers plus the existing Reader, canonical CFG, SSA, NormalFlowView, `new@1` site census, Region recovery, and `prove_conditional_value`. It does not add production behavior or claim a cause.

The bounded observations cover canonical edges and normal-flow nodes, SSA instructions at physical BCIs 83–99, referenced ValueIds with definitions and all uses (so `bci=None` Phi uses remain visible), the exact `outside_readers` result for producer BCIs 91/94/96, the site census/refusal, recovery shape, and the existing branch-84 conditional proof attempt. This is intended to confirm or falsify the Phi-reader hypothesis in the same IR; it is not a complete frame trace.

`init.rs` is the full candidate file; `init.rs.patch` is the minimal insertion diff against the pinned base. Before any future run, verify `base.sha256` in `pins.json`, apply the patch only to a temporary checkout, and check whether the method signature and current private APIs still match. The candidate has not been compiled or executed; toolchain, Cargo, Git, CLI, and shared source were not used.

The recorded future test command is for root to run only after the current product delivery and lock are clear.
