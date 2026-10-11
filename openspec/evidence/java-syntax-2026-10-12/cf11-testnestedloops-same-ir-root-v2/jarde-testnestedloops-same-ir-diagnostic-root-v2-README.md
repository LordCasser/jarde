# TestNestedLoops same-IR guarded diagnostic runner v2

This runner is the v2 successor to the private v1 runner. V1 reached the static preflight and rejected the candidate because candidate v2 pins omitted the crate-level `facade::` prefix from the exact test path. The new candidate v3 preserves both source candidate byte streams and the patch unchanged; it corrects that test-path binding and relocates the private artifact paths.

The runner binds `facade::same_class_generic_call_inventory_tests::frozen_upstream_nested_loops_foreach_candidate_diagnostic` in both its command and candidate pins. It requires an exact Rust test summary of 1 passed, 0 failed, 0 ignored. It retains the v1 frozen external class, baseline receipt, final-build-v3 receipt/runner/CLI identity, guard tools, the 4,718-file before/after source closure, two-file temporary source mutation, raw output retention, process-group resource limits, finally restoration, and guarded cleanup.

Static preparation only: this runner has not been executed, and the Rust candidate has not been compiled or run.
