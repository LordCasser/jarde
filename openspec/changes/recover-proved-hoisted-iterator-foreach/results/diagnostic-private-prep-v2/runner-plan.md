# Root runner handoff notes

Use the successful guarded runner pattern from `/private/tmp/jarde-testnestedloops-same-ir-diagnostic-root-v3.py` (SHA-256 `056a8c41fa5bb22806ad9bf50eaf6b9268dea7889cdc3601a0d64e43c81f3dda`) as a template for a new version/output root. Do not overwrite the existing v1/v2/v3 evidence directories. The accepted test remains exactly:

`facade::same_class_generic_call_inventory_tests::frozen_upstream_nested_loops_foreach_candidate_diagnostic`

The one Rust test invocation remains:

`cargo test -p jarde --lib --all-features --locked facade::same_class_generic_call_inventory_tests::frozen_upstream_nested_loops_foreach_candidate_diagnostic -- --ignored --exact --nocapture`

Set `JARDE_FOREACH_CANDIDATE_DIAGNOSTIC=1`. Bind the frozen JDK 23 original class at `/private/tmp/jarde-cf11-testnestedloops-baseline-root-v1/legs/javac23-original/classes/cf11/UpstreamNestedLoops.class` with SHA-256 `8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231`, the exact same test path, and the new private pins/patch hashes. Base `build.rs` remains SHA-256 `f5c0757ec670ec72aa377ea1f84529a576fe595c895a2d3ce1c7c66ee441c5fe`; candidate must be the only temporary build-source delta beyond the established facade test helper.

Keep the established bounded guarded-run policy: exact two-file source pin comparison before/after, process-group timeout, disk/target caps, raw stdout/stderr retained and hashed, test summary must be 1 passed / 0 failed / 0 ignored, restore both files in `finally`, verify exact restored bytes, and clean/confirm target absent. This note is a recommendation only; no runner was created or invoked here.
