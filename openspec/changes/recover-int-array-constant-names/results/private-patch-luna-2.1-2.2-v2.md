# Recover direct int-array constant names: private patch v2 review

This is a new private revision; `private-patch-luna-2.1-2.2.patch` remains unchanged (SHA-256 `cff8532daf70de060fcad9c7ee4ed09cd398c47057b0b956be3b9ba54c740e21`). V1 remains an unapplied, untested proposal. V2 is also unapplied and untested; it was prepared in `/tmp/jarde-array-v2-stage` and emitted as a diff against the current checkout. No product/test source in the checkout, OpenSpec task/spec, or immutable evidence was edited. No Git, Cargo/Rust compiler/rustfmt, JDK/JADX, or Jarde CLI was run.

The review used all four change context files (`proposal.md`, `design.md`, `specs/java8-recovery/spec.md`, and `tasks.md`) plus the complete v1 patch and report. Scope remains only tasks 2.1/2.2; no task checkbox or implementation status changed.

## V2 corrections

- Reuses the existing `case_label` switch-use discriminator. `body_range` is present only for direct array-leaf uses; switch case/return uses retain their existing prefix/suffix lookup and `None` range. No use-kind enum was added.
- Preserves the original switch-opcode capture. Additional primitive `newarray` capture requires an ordinary method with exact return descriptor ending in `)[I`; constructors and class initializers are excluded, as are other array return descriptors. The transformer repeats this method/descriptor gate before direct return analysis.
- Removes the whole-report `member_texts` rejection. Only a method with array-name uses is refused when that physical method index already has staged text, or when rebuilding its original same-run AST body does not match the current method text. Refusal falls back to the old switch-only projection for an int-return method. The existing root source writer remains the final authority for rebuilding the complete source; this patch does not promise composition with unrelated projections.
- Adds polling/`AnalysisSteps` before statement, element, candidate, BCI, source-map segment, range-pair and method-target scans. `IrItems` is charged before candidate/use/range/text staging. An array BCI must occur in the retained complete method instruction list; a present origin method must match the AST's physical method, and replay must expose the name segment under that exact method and BCI. The assigned range vector is explicitly `Vec<std::ops::Range<usize>>`.
- Keeps all output local until method text copies, output charging and a final cancellation poll succeed; only then are method text, root text and derived anchors assigned. The emitter unit test now checks both replay-budget stop and cancellation. It does not claim dynamic cancellation injection during the facade's final publish window.
- Adds focused tests for distinct element BCI/token anchors, unsupported nested/call-element array shapes, parameter/local shadowing, duplicate same-value `ConstantValue` fields, and refusal when the target physical method already has staged text. Existing switch tests remain.

## Files and remaining verification

The patch changes only four existing files: `crates/jarde-java/src/report.rs`, `crates/jarde-java/src/emit.rs`, `src/facade.rs`, and `src/class_source.rs`. It adds no public JSON/schema, dependency, candidate-table, or AST-kind changes. Body replay uses existing `Emitter::replay`/`SegmentPublication::Whole`; the method writer maps body-only ranges through its existing envelope placement. Numeric equality supports a unique same-class constant name presentation; it does not prove the original source token.

A local static check verified all unified-diff hunk old/new line counts (23 hunks). This does not establish Rust compilation, formatting, application to the root's intended HEAD, or behavior. Root should inspect the patch and run apply-check/focused tests. The key behavioral check is that the real emitter map yields one unique element segment with the expected BCI for each direct array leaf, while the existing switch tests stay unchanged.

Patch SHA-256: `b5d848f0964582388688d12ad4d21afbb277a512a6194dabe3c11646ec805133`
