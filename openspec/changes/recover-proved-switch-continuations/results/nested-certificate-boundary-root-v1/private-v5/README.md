# Private test-only nested-switch candidate v5

This candidate adds exactly one internal unit test to the live `region.rs` baseline. It does not change production code or add other tests. No repository file was written and no Git/Cargo/JDK/CLI/toolchain command was run.

- Baseline: live `crates/jarde-java/src/region.rs`, SHA-256 `d295d8dce7ac2c2e9d1d70fdc79a7e7698281a427dc34c611c47f07e5e28d32e`.
- Candidate: `region.rs`, SHA-256 `a7a6cee3c836b129d13db8b928e29355bbf3b1f6a9c1bb10e90f230203c33796`.
- Unified patch: `region.rs.patch` (one test, no production hunks).
- Test: `real_nested_switch_target_refuses_candidate_certificate_without_partial_result`.

It reuses `with_real_conditional_switch` and `RealSwitchInputs` with the checked-in javac 8 original `ConditionalSwitchBoundaries.class` (1494 bytes, SHA-256 `49795c66605066655f48ac37b862ccd132e43b74d290653afd51762ab1a409b7`). It pins the actual Reader canonical blocks/edges, selects outer switch BCI 9 and candidate join 80, with closure roots 36 and 90. Assertions establish that the SSA block at 36 ends at the physical nested `lookupswitch` BCI 37, its operation is `Operation::Switch`, the real edges 36→56 / 36→66 form its two successors, and 73→80 is the real arm predecessor to the candidate. `input.prove_candidate()` must return `None`; the pure proof result has no partial certificate, and the test checks the immutable Reader edge rows are unchanged.

The guard is dynamically reached from the pinned inputs: the candidate proof's first non-candidate closure starts at 36. Its canonical successors are 56 and 66, and its terminal SSA instruction is BCI 37. The `successors.len() > 1 && !operation.comparison()` refusal at current `region.rs:15322-15327` therefore rejects the candidate at the nested-switch boundary. This test directly exercises the candidate certificate layer via `prove_candidate`; it does not claim to execute `Walker::switch_forward_join`'s final candidate-list cardinality match or test Walker-owned mutable state. Existing class/public refusal evidence covers the external no-statement behavior.

Evidence pins were read from `openspec/changes/recover-proved-conditional-switch-fallthrough/results/physical-boundaries-root-v1/jarde-conditional-physical-root-v1/streams/03.stdout.raw:90-132` (javap offsets) and `/private/tmp/jarde-conditional-boundary-ir-root-v2/0.stdout.raw:775-1165` (Reader/SSA/canonical rows for the real method). The first file shows outer targets 36/80/90 and inner switch at 37 to 56/66. The second shows canonical edges 0→36/80/90, 36→56/66, 56/66→73, 73→80, 80/90→97, matching the test's full graph pin.

The prior v4 cardinality argument is retained with correction: other arms may return/throw, so do not claim all other candidates must reach the tested join. Given a successful candidate J, a second eligible candidate K's non-dispatch predecessor cannot be owned by any L≠K closure in J's proof: that closure reaches K as an ordinary case entry, producing forbidden non-empty fallthrough. If owned by K's own closure, it closes a cycle back to K and the DFS rejects it. The incoming-owner validation rejects unowned sources. Thus two candidates cannot both certify, without relying on terminal paths being absent; no multiple-list mock is required.

This is a private candidate only. It has not been compiled, formatted by rustfmt, or executed. Root should apply/review the patch against the stated baseline and run the actual validation.
