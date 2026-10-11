# CI stack overflow audit: overdeep lambda helper

Read-only audit of the current source. No repository files were changed and no tests or toolchain commands were run. CI’s observed failure and local 359-pass result are inputs supplied by root; this audit does not claim an independent reproduction.

The test at `tests/p3_immediate_functional_receivers.rs:1491-1523` synthesizes a lambda body `x + 1 + ...` with 300 additions, then calls full class-source recovery. That Java expression is a long, left-associated value tree in the synthetic `lambda$deep$0` method. The test’s intended behavior is a renamed physical helper (`$jarde`) when inline proof refuses the oversized donor.

The compound region walker is unlikely to be the direct cause for this fixture. Its structural recursion is centralized in `region::Walker::region_at` at `crates/jarde-java/src/region.rs:2734-2755`; it checks `depth >= MAX_REGION_DEPTH` before calling the inner walk or descending, and the limit is 32 at lines 2502-2525. This fixture’s arithmetic chain is straight-line bytecode: it does not create nested if/switch/loop regions, so the region recursion should remain shallow. The new nested-switch certification test exercises structural recursion, but is a different shape. This conclusion is source-shape-based; the failing CI stack trace would settle it.

The lambda-specific gate is sound but too late to protect earlier generic work. `plan_class_source_lambda_inline` counts AST nodes iteratively with `program_node_count` and refuses helpers over 256 nodes at `crates/jarde-java/src/report.rs:7059-7071`; therefore it avoids the recursive donor coverage/substitution walk. However, class-source recovery processes and emits each method before the facade invokes that plan. In `recover_inner`, the retained helper sidecar clones the complete `Program` at `report.rs:11639-11669`, then regular body emission is called at `report.rs:11886-11894`. The emitter’s `ExprKind::Binary` arm recurses through `binary_operand` at `crates/jarde-java/src/emit.rs:1415-1419`, and `Emitter::expr` is itself recursive (for example, line 1205). The derived `Clone` for `Expr` at `crates/jarde-java/src/ast.rs:482-485` is also a recursive path over boxed expression children. Thus the 256-node inline gate cannot protect a stack-heavy clone or ordinary report emission that has already happened.

The smallest evidence-backed direction is to identify the first overflowing frame from the CI backtrace, then put a bounded, non-recursive preflight at the earliest recursive consumer of the helper’s AST. If the frame is AST clone or emission, the durable fix is a narrow iterative path for the deep left-associated primitive arithmetic shape, or a deliberate bounded fallback before that recursive consumer while preserving the class-level rename decision. Keep the cutoff before recursive work; checking depth inside an already deep recursive walk is too late. Do not increase `RUST_MIN_STACK`: that changes the environment threshold without constraining the input path. Do not broaden the general CFG/DAG or region-depth design based on this one fixture.

If the backtrace instead points into `region_at`, retain the current pre-descent check and measure actual call depth and per-frame stack on Linux. The source guard is already before descent; changing its bound without evidence could reject ordinary nested control flow and would not address this straight-line lambda trigger.

Important exact call ordering:
- `report.rs:11834-11877` gathers same-run lambda candidates while the method Program exists.
- `report.rs:11639-11669` retains a cloned AST sidecar.
- `report.rs:11886-11894` emits the ordinary method report.
- `facade.rs:9191-9217` later runs the bounded inline plan; `None` selects the rename branch at lines 9236-9277.

This audit does not establish which recursive consumer actually aborts on Linux. A CI backtrace or root’s reproduction is needed to choose between clone/emitter and any unexpected region path. It does establish that the region depth guard is pre-descent, while the lambda node-count guard is downstream of ordinary AST retention/emission.
