# JADX common-field-initializer comparison audit

Status: static source audit only. This records a candidate explanation for the separately observed JADX 1.5.6 `DifferentRhsByteArray` mismatch; it does not establish the root cause of that CLI run. No JADX CLI or build command was run for this audit.

## Relevant source path

The local source tree inspected is `/Users/lordcasser/workspace/testzone/jadx`. Its top-level `build.gradle.kts` lines 14–17 set `jadxVersion` from `JADX_VERSION`, defaulting to `dev`, then assign it to the Gradle project version. Without building or reading the build environment, this audit can identify the source checkout's default as `dev` but cannot claim which binary was produced from it. The observed CLI in the root's separate run is JADX 1.5.6; that runtime version and this local source tree must not be conflated.

In `jadx-core/src/main/java/jadx/core/dex/visitors/ExtractFieldInit.java`, `moveCommonFieldsInit` around lines 124–169 collects candidate `IPUT` instructions from constructors, calls `compareFieldInits`, and removes/moves the writes if every collected sequence compares equal. `compareFieldInits` around lines 358–371 first checks list lengths and then calls only `baseInsn.isSame(otherInsn)` for each corresponding field write. It does not compare the stored value's producer tree through `isDeepEquals`.

`jadx-core/src/main/java/jadx/core/dex/nodes/InsnNode.java` around lines 368–410 defines `isSame` as soft equality: it checks instruction type and argument count, and recursively calls `isSame` only for arguments that are `InsnWrapArg`. It does not compare ordinary arguments (including register arguments/literals) or the result. The adjacent `isDeepEquals` does compare result and full argument equality, but `compareFieldInits` does not call it.

For the field write, `jadx-core/src/main/java/jadx/core/dex/instructions/IndexInsnNode.java` around lines 37–46 adds equality of the indexed field/type key after the base soft check. This can distinguish different fields, while still not proving the right-hand-side value is equal when that value is carried by ordinary registers.

## Candidate explanation for the observed differing RHS

`InvokeNode.isSame` in `jadx-core/src/main/java/jadx/core/dex/instructions/InvokeNode.java` around lines 93–103 adds invoke kind and `MethodInfo` equality after the same base soft check. It does not compare actual argument values beyond the base check's wrapped-argument recursion. Thus two calls to the same `mark(int)` method with the same invoke shape but distinct register-fed integer values can compare as the same wrapped instruction shape. Similarly, `NewArrayNode.isSame` in `.../instructions/NewArrayNode.java` around lines 24–32 adds array type identity; its length and other ordinary register arguments remain subject to the base soft rule.

This gives a plausible static route for two `IPUT`s to pass the common-sequence comparison even when their array-producing computations differ: the compared `IPUT` may see the same opcode, field key, and argument shape, while the assigned arrays and the inputs that produced their elements are represented through registers whose value provenance is not compared by `InsnNode.isSame`. The root's observed `mark(31)`/`mark(32)` failure is consistent with this weakness. This audit did not inspect the exact parsed instruction graph from the CLI run, so it cannot prove that this is the path taken in JADX 1.5.6.

`InsnWrapArg.equals` in `.../instructions/args/InsnWrapArg.java` around lines 67–82 performs an additional loop comparing wrapped instruction arguments with `InsnArg.equals`. That stronger equality does not repair `compareFieldInits`: the latter calls instruction `isSame` directly, and `InsnNode.isSame` recursively checks only nested instruction `isSame`, not `InsnWrapArg.equals`.

## Array payload comparison is a separate concern

The local source has `FillArrayInsn.isSame` in `.../instructions/FillArrayInsn.java` around lines 30–39, which calls `Objects.equals(arrayData, other.arrayData)`. `FillArrayData` extends `InsnNode`, whose `equals` is final identity equality (`InsnNode.java` around lines 571–578); so this particular `Objects.equals` is not a deep payload comparison. `FillArrayData.isSame` around lines 98–107 compares element type and `data == other.data`, also identity rather than array contents. These observations identify another potentially weak or identity-sensitive comparison path, but the effectful `mark(...)` fixture need not use a fill-array payload, and no evidence here says this path caused its output.

## Scope and conclusion

The strongest verified source-level finding is narrow: common constructor writes are approved using the soft `InsnNode.isSame` comparison of the `IPUT` instructions, and that comparison does not establish equality of ordinary register-fed RHS values. This is a credible candidate for the false common-initializer match; it is not a demonstrated 1.5.6 call path or a substitute for the root's captured CLI evidence. Any production diagnosis should first correlate the exact parsed `IPUT` and nested RHS instructions from that binary/version. The document proposes no production change.
