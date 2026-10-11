# For-condition next-round private runner review

This directory contains private preparation only. No shared file, Git state, Cargo command, JDK/JADX tool, or product CLI was changed or invoked here.

## Frozen runners

`jarde-for-condition-workspace-root-v4.py` is the root-v3 runner with one normalized source change: `OUT` now points to `/private/tmp/jarde-for-condition-workspace-root-v4`. Its schema and execution logic stay as before: fresh-target preflight, fixed seed `5350648285461741569`, 95 metadata-derived batches covering 360 targets, 4,718-file source closure, per-command guards, and guarded cleanup. SHA-256 is `70f4b0f974ea934f0bc761b4b56722aef8ceee010fa7e6027ed9c30b88b8fabd` (18,953 bytes; mode 0555 when pinned).

`jarde-for-condition-final-lint-build-root-v3.py` adapts the unrun root-v2 build runner in three exact places: output directory v2→v3; final CLI path v2→v3; expected changed-source set adds `tests/recover_unicode_identifiers.rs`, yielding exactly four paths. The JSON schema remains `for-condition-final-lint-build-root-v1`. It keeps the accepted candidate-v3 byte-identity assertion. SHA-256 is `85e25bb6d87a5993305c0fdeb3c58cd9300886701cf8158267df267ce0b92f1e` (5,761 bytes; mode 0555 when pinned).

Unified normalized diffs are in `workspace.patch` and `final-build.patch`. The root-v3 workspace runner and final-build-v2 runner are preserved and pinned as references; neither was overwritten.

## Failure evidence and current status

The preserved root-v3 workspace receipt is `workspace-root-v3/execution.json`, SHA-256 `cef6a38aa7640f6bee51a1522470715311dcf782e65b1f7dab8790a34de0a087`. It records schema `for-condition-full-workspace-batched-root-v1`, status `failed`, 95 expected batches, 360 targets, 4,718 source files, and 77 command rows. Row 75 exited 101 with no guard stop; the raw stdout/stderr are separately pinned. Its failing test was the prior whole-output Unicode assertion `the_ascii_presentations_are_byte_for_byte_the_pre_change_ones`. This is the already-reported stale golden failure during the two proved header changes. Row 76's guarded `cargo clean` passed and the target directory was absent afterward. Before/after source pin maps were identical with 4,718 entries. The v3 run is failed evidence and is not promoted to an accepted full-workspace result.

The final-build-v3 receipt does not exist in the prepared state. It must only be treated as evidence after the runner has actually completed and produced its record. The candidate-v3 executable is pinned at 61,213,840 bytes, mode 0555, SHA-256 `51c9a1326031fd10444f7e18a055c6daecef233423c90478257becf24d1e0bd4`.

## Verifier review

`jarde-for-condition-ci-verifier-luna-v4.py` binds the actual workspace-v4 and build-v3 SHA-256 values, their output/receipt paths, and their unchanged root-v1 schemas. It retains the accepted candidate-v3 identity rule and requires these exact final source paths: `tests/p3_nested_loop_side_exits.rs`, `tests/fixtures/java8/cf11-for-condition/README.md`, `tests/p3_crossing_array_read_values.rs`, and `tests/recover_unicode_identifiers.rs`. It also requires the new Unicode regression test `the_ascii_presentations_only_change_in_the_two_proved_f1_for_headers`, alongside the existing named tests and CI outcome comparisons.

I parsed all three Python scripts with `ast.parse` and audited their symbol tables for unresolved global references. The verifier and final-build runner have none; workspace-runner globals reported by the checker are all declared via its explicit `global` statement in `main` (its deliberate module-level execution pattern). This is static source review only; it is not a run of the verifier or the runners.

Pins, exact artifact hashes, old failure-stream hashes, receipt paths, and schema values are recorded in `pins.json`. The v4 full-workspace run and final build remain to be performed by the root-controlled process; no new full-workspace or final-build pass is claimed here.
