# TestNestedLoops same-IR diagnostic guarded runner v1

This runner is prepared for root review only. It has not been executed. It is bound to private candidate v2 and exactly two temporary workspace source paths: `crates/jarde-java/src/build.rs` and `src/facade.rs`. It refuses to start if either live base SHA or any candidate/tool/input pin differs.

Before mutation it verifies the accepted final-build-v3 receipt and runner, candidate CLI identity (path, byte count, SHA-256, mode 0555), the pinned guard and target-size helper, the candidate v2 pins and patch, all source dependencies listed by the candidate pins, and the external frozen class. It independently enumerates the ordinary workspace include closure before the external include is applied, requires an exact match with the receipt's 4,718 before/after source-pin maps, and requires the target directory to be absent. It does not run the ordinary closure scanner while the temporary facade source contains an absolute external `include_bytes!`; the class remains separately SHA-pinned.

On application it snapshots both original source files, writes the two pinned candidate files, reads them back, and hashes the full 4,718-file baseline closure. Exactly the two candidate source hashes may differ. It then runs only:

`cargo test -p jarde --lib --all-features --locked facade::same_class_generic_call_inventory_tests::frozen_upstream_nested_loops_foreach_candidate_diagnostic -- --ignored --exact --nocapture`

The environment explicitly enables `JARDE_FOREACH_CANDIDATE_DIAGNOSTIC=1`, keeps `RUST_MIN_STACK` unset, uses the final-build seed `5350648285461741569`, and sets `RUST_TEST_THREADS=1`. The command must exit zero and print exactly one matching `1 passed; 0 failed; 0 ignored` test summary; zero selected tests are rejected. Both raw streams and their hashes are recorded, including guarded pre-spawn failures.

The pinned guard starts the subprocess in a process group, checks free disk and target size at one-second intervals, and terminates the process group on a limit breach. The runner verifies 5 GiB free space and a 1 GiB target ceiling before commands. Its `finally` path restores each changed source only if its live content is still the pinned candidate (or already the original), re-enumerates the restored in-tree closure, verifies both source files and all 4,718 hashes, runs `cargo clean`, and checks that `target/` is absent. Any command failure, restore mismatch, source-pin drift, cleanup failure, or tool identity drift leaves status failed while preserving the raw capture.

The runner is static-preparation only. No Git, Cargo, Rust compiler, test, Java, CLI, or diagnostic command was executed. The v2 Rust candidates remain uncompiled; this runner does not predict what the trace will report.
