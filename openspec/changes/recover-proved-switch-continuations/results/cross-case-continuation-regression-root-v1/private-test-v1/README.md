# Private candidate: switch cross-case continuation test

This is a private test-only proposal for `tests/p3_switch_forward_join.rs`.

The former combined negative asserted that both a cross-case target (`+6`) and a looping backedge (`-3`) must stay quoted. Root's fresh real run at `openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/observed-workspace-v3/57.stdout.raw` shows the `+6` class now correctly recovers a statement switch and its shared tail. The test is split accordingly: `+6` asserts exact recovered text, statement content, no fallbacks, a single post-switch `local1 = 30;`, and unique source-map ownership for BCI 37 and BCI 40; `-3` retains the negative quoted-bytecode expectation.

The positive uses `RecoveryEvidenceRequest::all()` so source-map completeness and method identity can be checked. It locates `mixedExits`, checks descriptor `(I)I`, and requires BCI 37 and 40 to map to the expected tail statements with the real member as primary origin. The original positive join test and the budget test are left intact.

`region.rs.base` is byte-identical to the pinned working test source. `region.rs.candidate` and `test.patch` are the proposed artifact. This candidate has not been compiled or executed here. Root should apply and verify it against the fresh whole-class run before accepting it.
