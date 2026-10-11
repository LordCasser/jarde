# `computed for` controls — private preparation only

## Reuse points found

- The precise positive oracle is `preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/original-sources/LoopCases.java`, method `lastIndexOf([IIII)I`, and its frozen `cases/javac23-original/classes/cf07/LoopCases.class` (SHA-256 `004e99baeb062861f95a4dac13e3a78399a66efd31f2589b0028cfe50432ae33`). The accompanying `Runner.java` tests hit, miss, and narrowed start. It is the right existing target for the computed `end - 1` initializer and unique return leaf; no need to invent a replacement positive as the primary regression.
- Existing Rust boundary contract is `tests/p3_for_latch.rs` with `tests/fixtures/p3-for-latch/v8/ForBoundaries.class` (SHA-256 `9d580094884645a9acd495f604a54f7e8b42e00da837ae99ee184003e560fafe`) and source/runner beside it. Its `observedAfter`, `twoUpdates`, `varyingBound`, and `noUpdate` are the exact required loop refusals.
- Add/store adversarial precedent is `tests/p3_for_add_store.rs` and `tests/fixtures/p3-for-add-store/v8/ForAddStoreBoundaries.class` (SHA-256 `b38100ed17d41b8a60eb6a25e0270484b1b0c8fb617fd9170faba10c84f8a106`); it already checks changed step, extra consumer, shared tail effect, and post-loop observation.
- Producer/independent-call precedent is `tests/p3_deferred_value_order.rs` with `tests/fixtures/p3-deferred-value-order/v8/DeferredValueOrder.class` (SHA-256 `44eaaf23c354566f25ba429e49f7d34a73a114c7de35909360b705dcc63c3d04`) and helper/Runner sources. Reuse the narrow raw `Code`-attribute patching style in `openspec/evidence/java-syntax-2026-09-22/deferred-evaluation/fixture/patch.py`; its patched full class and runner demonstrate side-effect/throw-order observation.

## Prepared private inputs

`ComputedForControls.java` and `ComputedForControlsRunner.java` cover a computed `f() - 1` loop initializer, a shared `f()` result, `f`/`g` trace and throw modes, `g` mutating the bound, a post-loop read, an outside write, nested declaration scope, and a try/catch covering initialization/body. The runner reports return/value, trace, and mutated bound for each reset case.

`patch_pending_stack.py` is a small standard-library-only class-file patcher for just `find([III)I` and `shared(I)I`. After root compiles the sources, it swaps the one-byte store and three-byte `invokestatic g` immediately after `iconst_1; isub`: `... isub; store; g` becomes `... isub; g; store`. Code length, branch offsets, exception table, and stack-map offsets stay fixed; the integer remains pending across `g`. This is a verifier-bound transformation of javac output, not bytecode javac naturally emits. Root must run `-Xverify:all`, inspect disassembly and compare the raw behavior before treating it as a legal fixture.

## Expected observation and limits

- Predicted `find` observations: hit `2:12:2`, miss `-1:12:2`, `f` throw `-2:1:6`, `g` throw `-2:12:2` (fields are result:trace:end). `g` changes `end` from 6 to 2, while the computed start stays 5; the `f` throw skips `g`.
- Predicted remaining rows: `shared` returns `599:12:2`; `observedAfter` returns `-1:1:6`; `externalWrite` returns `0:1:6`; nested scope enter returns `-1:1:6`, skip returns `-7:0:6`. `shared` makes the one `f()` producer feed saved `seed` and the `-1` initializer.
- `observedAfter` and `externalWrite` are explicit refusal controls; `scoped` and `find` exercise nested local visibility and a handler boundary.
- The pending-stack transformation has not been compiled or JVM-verified here. The expected traces/results are source-derived predictions only; actual compilation, `javap`, verifier, Jarde/JADX reports, and runtime outputs remain unknown for root to measure. It does not independently prove all share/effect/scope cases.
