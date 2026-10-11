# Current-product own-CI handoff checklist (read-only)

Snapshot date: 2026-10-11. This is a source/workflow audit, not a run result. I did not invoke Git, Cargo, JDK, JADX, or any CLI. The exact product commit for the upcoming CI run must be the commit that contains the complete current product increment; the earlier accepted CI records below are reference contracts only and must not be reused as acceptance for that commit.

## Workflow contract

Pinned workflow SHA-256: .github/workflows/ci.yml = b545890a55cc4fda31d70b332889c111d789afbf0949581e96bf2b94ea6cd18a.

The workflow has four required jobs, and the recent root verifiers enforce that every job and expanded API step succeeds: stable / test and specification; MSRV 1.88.0; supply chain; fuzz smoke. They expect 52 GitHub API steps (composite action expansion; the YAML has fewer top-level step entries). The accepted verifier contracts are 4 jobs / 52 steps.

In the stable job, preserve order and run the existing full gate: checkout; stable Rust with rustfmt/clippy; Temurin 25.0.4+7.0.LTS; Node 22; cargo fmt --all -- --check; Clippy with the workflow's existing 29 explicit allowlist entries and -D warnings; then both complete workspace passes:

- seed 5350648285461741569: cargo test --workspace --all-targets --all-features --locked
- seed 5350648285461741570: same command

The environment is set per step as PROPTEST_RNG_SEED. Do not replace either with a focused test or combine the two. Continue with the exact named workflow steps for ignored JDK 25 oracle, ignored P3 Java 8 execution comparison, functional-constructor full-class two-JDK comparison, BigDecimal full-class comparisons, the two nested-int-array comparisons, both public API examples, feature tree, normal dependency tree, layered dependency closure, OpenSpec 1.11.0 install/strict validation, and tracked-tree clean check. Then require MSRV check (cargo check --workspace --all-targets --locked), supply chain checks (both cargo-deny workspace commands), fuzz corpus test and all three bounded fuzz targets, and fuzz clean-tree check. The authoritative step commands and arguments are in the pinned workflow above.

## Seed summaries and historical verifier boundaries

The latest accepted conditional own-product verifier reference is verify-conditional-ci-root-v3.py (SHA-256 efd89e40df484e0b1fcdaebf1278996221d1d2067682067b541c1d4c711f47e6) with acceptance SHA-256 c50bb7b672896ce19f14e152f288ef0522830fd22b6749ba18d68fc0ebf2fd7a. It is for product commit 6476b56c357443ef17318891b12142f509977234 / run 38096105240 only. Its two seeds each reported 359 targets and workspace totals 3422 passed, 0 failed, 97 ignored.

The latest accepted local-source-types verifier reference is jarde-verify-typed-ci-root-v7.py (SHA-256 4d1d3341a81e0adfe49b0f3f37a61a83a4d1fc0707237121d5cf575d7ddb3feb) with acceptance SHA-256 6bcdf9b1e1f21e59fd1660ccfe73347ff485c574d3e0723a0a46d5dac0abdb81. It is for product commit 43dffc9b806dc2b69102e50f52ee78a50850c6d0 only. Its two seeds each reported 3408 passed, 0 failed, 97 ignored, 356 workspace result records; the typed integration target had 9/0/0. Those totals differ because the product snapshots differ. Neither record proves the current increment.

For the new commit, capture both new workspace summaries and the complete target/outcome inventory from the exact product-CI run. In particular, check the jarde-jvm lib target at 243 passed / 0 failed / 0 ignored under each seed (the older conditional run was 234); ensure the Java nested regression named below appears in both seed runs. Recompute workspace totals from the new raw summaries instead of pinning either historical total.

## Changed-source pins to bind to the exact product commit

These SHA-256 values are for the current working-tree bytes read during this audit. The new verifier should compare these paths against the exact product commit's Git blobs, then verify live bytes agree; do not substitute the two older product commit IDs.

| Path | SHA-256 |
|---|---|
| .github/workflows/ci.yml | b545890a55cc4fda31d70b332889c111d789afbf0949581e96bf2b94ea6cd18a |
| crates/jarde-jvm/src/canonical.rs | 79d7725b697433ae9f00360818cf5c3deece227c4d15101412a730fd8bed9a86 |
| crates/jarde-jvm/src/call_context.rs | 119acaf83c14eefd0a7b707c08aeae6fd56fec06287f0d7912efc5b587478f57 |
| crates/jarde-java/src/build.rs | 2586f9eaa09c06056bc8c565ed6bd00c18c35ad6ddb28970ef3d6e0ee2fa5bef |
| crates/jarde-java/src/region.rs | acdb5d22195ce0b5d283fe1b7a3003f35830c85811b18e2e713b7f4728e93168 |
| crates/jarde-java/src/init.rs | f1c0c96fed82e86729fefbef55f40d85473526fb86b0389b9253bb6f41f4c4a1 |
| crates/jarde-java/tests/cf12_proved_local_source_types.rs | 1eb2c9e039a6aa2dcb706f29969e7006f4ec6029340763b1d883accfb58e98db |
| tests/p2_golden.rs | 0abe0466b8e7cac5ba0ab75cdcc182e608377bd77c2f0d15d1abe0d79fae0001 |
| tests/fixtures/p2-golden/historical.json | ea12ca89d92dc4a70b6fc2aa3f55a208887d2983b399940f14412c3837fa826e |
| tests/fixtures/p2-golden/legacy-clone.json | d152e7246a15b161ce838914293930adb2205f24ec708da5ba8bf3f0151f3f31 |
| tests/p3_reference_slot_lifetimes.rs | 9956da89a4472f3d113416df88456cc2401c26d724f4767388bb78f773c8d014 |
| tests/fixtures/p3-reference-slot-lifetimes/negative/unknown-null/NullThenBuilder.class | 93acdfe27b312e2c98666a00dbfcb5f61299b03f62ac0dbd916b385232f260b3 |
| tests/fixtures/p3-reference-slot-lifetimes/negative/unknown-null/NullThenBuilder.baseline.java | f04aad4af575953e14ce91084bc5e3e5fc23b3656f42dd21280701a81ccb8e1e |

## Named regressions and golden deltas

Bind these names exactly in the new workspace outcome inventory:

- JVM nested return: call_context::tests::a_nested_call_returns_to_its_own_continuation.
- Legacy clone coverage additions in canonical.rs: shared_conditional_subroutine_is_covered_per_call_path; clone_coverage_rejects_a_disconnected_internal_node; clone_coverage_rejects_a_disconnected_self_cycle; clone_coverage_rejects_wrong_normal_call_return_and_exception_paths; clone_coverage_accepts_a_nested_return_continuation; an_uncovered_nested_return_cannot_cover_its_outer_continuation; explicit_dead_callsite_seed_stays_unreachable_after_coverage; clone_coverage_budgets_storage_work_and_assembly_cancellation; a_clone_exception_edge_reaches_its_handler_and_ret_restores_the_caller_path.
- Java nested construction: init::tests::nested_argument_sites_present_the_complete_inner_construction. Compound/conditional nested-value cases also include build::tests::a_three_leaf_nested_conditional_is_published_as_one_value, nested_conditionals_in_the_false_arm_and_four_leaf_trees_are_supported, and an_external_edge_into_a_nested_test_keeps_the_region_quoted.
- The NullThenBuilder regression test is now null_first_exact_builder_writes_refine_local_type_and_compound_condition; the previous name null_first_exact_builder_writes_refine_only_the_local_type is stale and must not satisfy the new commit's test pin. Keep the original fixture class and baseline pinned; the test derives its new exact expected output by the local-type replacement and nested-if-to-compound-if transformation.
- The permanent P2 golden currently pins the new usage for historical-v45…v48 at ir_items=552 / analysis_steps=186 (prior 540/154), and shared-subroutine-two-call-sites at 205/125 (prior 195/98). All other report fields and the four v49…v52 zero-clone historical entries remain unchanged. Golden names and bytes are pinned in the table above; the whole-report comparison remains required.

The historical product records also contain prior typed-test names and source pins. Use them only to identify deltas (especially the NullThenBuilder rename), and make the new verifier's expected names and source pins come from this complete product increment. No standalone verifier framework is needed beyond exact commit/blob pins, the 4-job/52-step API summary, the two stable raw-log test summaries, named new test outcomes, and the existing clean/build evidence.
