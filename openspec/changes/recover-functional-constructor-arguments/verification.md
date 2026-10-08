# Verification

Implementation and behavior evidence for tasks 1.1, 1.2, and 2.2. Task 2.1's nullable bound-reference edge remains under root review before it is marked complete.

## Implementation

`new@1` now accepts an `InvokeDynamic` only when the instruction lies in the physical constructor argument dependency set. The existing lambda renderer remains the authority for bootstrap, SAM, implementation handle, captures, and adaptation. Dynamic arguments join the existing construction handler-coverage gate, which checks every instruction from allocation through constructor plus the sole consumer. The handler flag is set only for an accepted dynamic operation physically inside the span; a factory materialized into a local before allocation is not part of the new-expression interval.

## Verification run

Using the shared cache at `/Users/lordcasser/workspace/projects/jarde/target`, with `CARGO_BUILD_JOBS=1` and `CARGO_INCREMENTAL=0`:

- `cargo fmt --all` completed.
- `cargo test -p jarde-java --lib dynamic_constructor_arguments_keep_one_handler_coverage -- --nocapture`: passed; independently changes handler coverage at allocation, factory, constructor, and consumer.
- `cargo test -p jarde-java --lib functional_constructor_arguments_are_in_the_verified_dependency_run -- --nocapture`: passed; nine constructor-argument forms in both frozen compiler legs.
- `cargo test -p jarde-java --test p3_java_recovery -- --nocapture`: 51 passed, one pre-existing evidence-dump test ignored.
- `cargo test -p jarde-java --test nested_ctor_argument_sites -- --nocapture`: 5 passed, including complete method bodies for both legs and same-length unknown-bootstrap mutation fallback.
- `cargo test -p jarde-cli --test json_cli functional_constructor_arguments_replay_the_complete_class_on_both_jdks -- --ignored --exact --nocapture`: passed. It recovers FunctionalConstructors and IntBox through class-source, strips comments, recompiles both outputs with the frozen Driver, runs each leg under `-Xverify:all`, and matches each frozen source stdout. It selects local Corretto 8/OpenJDK 23 when present and falls back to JAVA_HOME/PATH so CI cannot silently skip missing local JDKs.
- `functional_constructor_arguments_replay_the_complete_class_on_both_jdks` is the exact ignored test name for CI invocation.

The full class-source CLI currently emits source even when its command status reports an unrelated `anonymous_interface_projection` unsupported result. The replay test gates on complete source, compilation, verifier execution, and behavior, not that unrelated status. IntBox is also recovered and recompiled independently for each leg, in a fresh output directory.

## Negative controls and origins

- Same-length `metafactory` → `bad_factory` mutation preserves whole-method bytecode fallback without guessed lambda/method-reference syntax.
- An independently valid dynamic creation followed by another dynamic value that actually serves as the constructor argument reaches the physical effect scanner; the first operation is rejected at its own BCI as an `InvokeDynamic`, and the constructor remains quoted.
- A verifier-valid straight-line `new Thread; dup; indy; pop; indy; invokespecial; areturn` control is retained as conservative fallback. Corretto 8 `Class.forName("Test")` under `-Xverify:all` accepted the generated class. Jarde does not build an allocation candidate for this exact sequence, so this control is not claimed as proof of the dependency gate; the preceding control exercises that gate directly.
- An earlier materialized lambda loaded from a local remains outside the new-expression producer record (`arguments == [10]`, not the earlier factory BCI 0) and the construction still presents.
- Allocation, factory, constructor, and sole-consumer producer origins are retained in the dual-leg method source map. The `runnable()` map asserts primary factory BCI 4 and derived producer BCIs 0, 3, and 9.
- Discarded construction, unknown/untyped bound receiver, and a typed nullable bound reference with an actual `Objects.requireNonNull` creation-time effect remain refused. The latter is a distinct interleaved-effect case; it does not stand in for the untyped SAM mismatch test.

## Deferred review boundary

Root's independent JVM oracle confirmed the no-explicit-check, exact-typed nullable bound-reference case is a correctness failure: OpenJDK 23 `-Xverify:all` reports the frozen original as `creation=ok\ninvocation=NPE`, while current full-class replay reports `creation=NPE`. This implementation did not modify `lambda.rs`. Do not interpret the explicit `Objects.requireNonNull` negative control as proof for the no-check case; task 2.1 remains unchecked. Root has split the prerequisite into `preserve-bound-reference-creation-timing` so it can be completed before this change is accepted.

## Disk and temporary output

The shared target directory measured 2.8 GiB after the runs (it measured 2.1 GiB before this work); free space was 69 GiB, above the 20 GiB stop line. Test-local temp directories clean themselves. Manually created verifier fixture files under `/tmp/functional-ctor-nondep` and associated logs are removed before handoff. No target directory was created inside this worktree.

## Root final acceptance

上述 Deferred review boundary 是主片首次实现时的历史状态。后续 preserve-bound-reference-creation-timing 已完成，root 用最终候选 7a296303 双 JVM 验证 NoCheck/NoStand 安全拒绝及精确来源，完整构造类与非空引用正例回放通过；task 2.1 已验收。以 verification-root.md 与 results/root/gates-summary.txt 的最终记录为准。
