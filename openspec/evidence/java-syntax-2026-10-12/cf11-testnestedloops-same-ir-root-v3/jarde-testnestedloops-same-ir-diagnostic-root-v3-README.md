# TestNestedLoops same-IR guarded diagnostic runner v3

This runner applies only the private v4 diagnostic candidate to the same two Rust source files and runs the same exact ignored test path as v2. V2 completed with 1 passed / 0 failed / 0 ignored. Its raw stderr showed the inner candidate at BCI 42 reached the post-entry proof but returned `post_entry_candidate_proof_incomplete`; the outer candidate at BCI 13 was refused earlier because its body begins with `Assign`. V4 adds stage-only logs within the inner candidate closure to narrow that first refusal.

The runner retains the v2 controls: full `facade::` test-path binding, JDK23 frozen class pin, accepted final-build-v3 receipt and CLI identity, 4,718-file source closure before and after restoration, only two temporary source mutations, raw streams, process-group disk guards, finally restoration, and guarded `cargo clean`. It requires an actual test summary of 1 passed / 0 failed / 0 ignored.

Static preparation only. Runner v3 and candidate v4 have not been executed or compiled.
