# Final CLI runner v5

Runner v5 derives from the specified v3 script (input SHA-256: `9191b26304c7489c959785082ed1fb0aa586d8ceca498490b5c315f9ed79aeef`). The docstring accurately says it freezes candidate CLI v2; the execution record schema is `compound-condition-focused-and-cli-root-v5`.

Pin selection starts from the guard's `PRODUCT_PATHS` and existing public test/baseline inputs, adds `canonical.rs` and the fixture macro definition, scans every selected `.rs` source containing direct literal includes for `include_bytes!` and `include_str!` inputs, and resolves each literal `crate::test_fixtures::fixture!("…")` in canonical.rs against the jarde-jvm fixtures root. The macro definition is pinned and skipped by the direct-literal scan because it uses `concat!` and its documentation contains a non-resolvable illustrative include path; canonical.rs itself is scanned, so new direct class includes are picked up. Current observed product path count is 124; no cardinality assertion exists. The set includes all four negative fixture classes.

The existing 5 GiB free-space and 1 GiB target-size guard remains polled once per second through the v9 guard, and final `cargo clean` remains in the cleanup path. No additional cargo command was introduced. No toolchain/build/CLI was run. `product-pins-current.json` is a pre-edit snapshot; execution recomputes hashes when run.
