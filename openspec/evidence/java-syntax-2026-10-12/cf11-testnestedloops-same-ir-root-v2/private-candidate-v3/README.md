# TestNestedLoops same-IR diagnostic candidate v3

This private, unapplied v3 keeps both source candidates and the patch byte-for-byte identical to v2. It corrects only the exact test-path binding and relocates the private candidate artifact paths. The guarded runner v1 preflight rejected v2 because the pins recorded the child module path without the `facade::` prefix used by the crate test harness.

The exact test path is `facade::same_class_generic_call_inventory_tests::frozen_upstream_nested_loops_foreach_candidate_diagnostic`. It remains ignored by default. The test uses `ClassNameQuery::dotted("cf11.UpstreamNestedLoops")`, the JDK 23 frozen class, and the public `Engine::class_source()` path with the pinned profile and actual class facts.

V2 fixed two statically found issues from v1: it reads the first body statement through the borrowed loop kind after the original body has moved, and uses the CLI-equivalent dotted class query. This candidate has not been compiled or run; those remain static conclusions. Diagnostic event logic and source bytes are unchanged from v2.

No shared files changed. No Rust compiler, Cargo test, Git, or diagnostic run was performed.
