# Typed-local proof-budget test draft (private, unexecuted)

This scratch patch adds only `#[cfg(test)]` instrumentation and one internal Rust test. It is intended to be layered onto the typed-local implementation v3 after that product patch is applied. It does not contain or alter the typed-local production algorithm, and it does not include the separate discarded-call/pop source-origin repair currently being handled by another change.

The test analyzes the frozen `TestSwitch$TestCls.test(String)String` and `TestSwitchNoDefault$TestCls.test(I)V` class bytes through `analyze_method_ir` with `AnalysisStage::ALL`. It obtains the declaration, method metadata, and local-variable debug records from that same analysis, builds `RecoveryFacts`, and calls the normal `report::recover` path. It does not fabricate a method IR, a slot-use, a frame, or a source fact.

The test-only observer records the AnalysisSteps count immediately before the typed-local proof charge. For the real `char c` candidate (slot 5, index 0, stored write BCI 29), the char all-write scan charges one step per physical write; the known frozen method has one such write. A second recovery uses the recorded pre-charge count as its limit. It therefore reaches that same BCI 29 gate and fails the gate's next one-step charge. The test asserts an AnalysisSteps stop at BCI 29 and an empty body/source map. A separate replay cancels the token at the same observed gate, after full IR analysis, and asserts Cancelled at BCI 29 with no partial output.

For the real null-leading `String s` candidate (slot 2, index 0, first write BCI 1), `null_leading_reference_type` charges the complete actual `uses.len()` before scanning the local's writes and values. The frozen method's physical write count is five (BCIs 1, 34, 40, 46, and 52); `uses.len()` also includes reads, so the test captures the actual helper charge instead of assuming it equals five. Its bounded replay sets the limit to `observed_start + observed_amount - 1`, so the aggregate proof charge itself is refused at BCI 1. The test asserts the AnalysisSteps stop and empty body/source map. The cancellation replay cancels at the same null-reference gate and asserts Cancelled at BCI 1 without partial output.

Each bounded/cancelled replay reuses the immutable output of a completed real analysis but starts a fresh recovery budget. The observer is test-only and records no facts used by production. Full unlimited recovery asserts the rendered `char c` and `String s` declarations, but this draft does not assert complete physical source-map coverage: the root's first typed-local run found that discarded-call `pop` origins at BCIs 82, 92, and 105 still need the separately scoped origin fix.

Files:

- `internal-budget-test.patch` is the test-only unified diff, based on the private typed-local v3 candidate.
- `internal-test-module.rs` and `internal-test-snippet.rs` are readable source fragments used to assemble the patch.
- `base/` is a private copy of the repository source with typed-local v3 applied; `candidate/` adds this test-only patch.

No Cargo, JDK, CLI, formatter, or Git command was run for this draft. It has not been compiled or executed and must be reviewed/applied by the root agent against the current production patch before it can count as verification. The source-origin results above are reported from the root's prior actual run, not from this draft.
