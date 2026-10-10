# Conditional switch physical boundary test drafts, v2

This version is an independent private directory. The v1 directory is preserved byte-for-byte. No repository files were changed. No Git, Cargo, Rust compiler, rustfmt, JDK, Jarde CLI, or tests were run.

## Composition

`tests/conditional_switch_proof_tests.rs` is a unit-test fragment to append inside the existing private `tests` module in `crates/jarde-java/src/region.rs`, after composing `prove_switch_fallthroughs`. It calls that same helper with the untouched full edge iterator from each real reader-produced canonical graph. It does not perturb edge rows, fabricate a graph, or repeat the caller's label-order predicate. A requires `None` because case 36 exits to two entries (57 and 67); B requires exactly `{36: 67}`. The public recovery test below owns checking the production caller's rejection of B.

`tests/p3_conditional_switch_boundary_rejection.rs` is a new integration-test file to add beside the existing positive `tests/p3_conditional_switch_fallthrough.rs`. Keep the existing positive file intact; this file has a distinct name and tests B's non-adjacent rejection. Its `reader_debug_locals` is reused from the positive draft's reader-backed facts helper. It uses `RecoveryEvidenceRequest::default()` for the actual default selection and `all()` for full evidence. It compares report text, expects the default source map to be empty, and checks each present all-evidence origin against the exact physical method and instruction BCI. It does not compare the two source maps.

The four `.class` files under `tests/fixtures/p3-conditional-switch-boundaries/` are complete byte copies from `/private/tmp/jarde-conditional-physical-root-v1/cases/`. The test drafts currently consume javac 8 A/B; javac 23 A/B copies are retained for later root choice. `SHA256SUMS` records every source and binary fixture in this bundle.

## Pinned reader observations

The reader output is `/private/tmp/jarde-conditional-physical-ir-root-v1/0.stdout.raw`. For the two javac 8 classes it reports `canonical.unreachable() == []`, six canonical blocks at 0/36/40/57/67/74, eight Normal rows, and empty clone paths. This is a claim about that field for these two analyzed methods. The physical instructions at BCIs 50, 51, 53, and 56 are inside physical code ranges but are omitted from the canonical block-start set; the tests assert those exact facts.

A's full edge multiset is `(0,36), (0,57), (0,67), (36,40), (36,57), (40,67), (57,74), (67,74)`. B changes `(36,57)` to `(36,67)`. The public test gets branch opcodes and relative offsets from `MethodCodeFacts.instructions` zipped with `MethodCodeFacts.operands()`, the same reader payload used by analysis.

## Root-owned work

- Compile and run only after composing the matching implementation. These drafts remain uncompiled.
- Confirm the B recovery report actually contains the expected all-evidence source-map records and the structural region assertions identify rejection. Add a precise fallback code/reason only after observing it; this draft does not guess a refusal classification.
- Capture real proof budget/cancellation traces before adding exact stop dimensions, costs, or BCIs. The pre-cancel test checks only public atomic cancellation.
- Run full-class compile/runtime replay separately; these boundary tests do not establish it.
