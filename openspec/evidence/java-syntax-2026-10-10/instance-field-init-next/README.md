# Instance field initializer follow-up fixture

Preparation only. These source files and the runner have not been compiled, decompiled, or executed. They do not change the existing static-field change or any baseline.

The current `array-field-initializers-literal` baseline already covers literal instance-array initializer rendering and runtime semantics. This follow-up isolates the remaining constructor-presentation question: can identical instance-field writes from all direct-super constructors become one field initializer while preserving effect order, and can a `this(...)` chain initialize that field exactly once? The fixtures are candidate probes, not an implementation claim or a claim that existing JADX tests cover these exact combinations.

| Fixture | Existing JADX anchor | Exact extension in this fixture |
| --- | --- | --- |
| `CommonDirectSuperByteArray.java` | `jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrayInitField.java`, `test()` / nested `TestCls`: an instance `byte[]` field initializer and its array-literal rendering. `.../arrays/TestArrayInit.java`, `test()` / nested `TestCls.test2()`: an array literal assigned to an instance field in a method. | Two direct-super constructors contain the same effectful `byte[]` RHS, followed by different constructor-body trace writes. This multi-constructor common-write combination is an extension, not an existing assertion. |
| `ThisDelegatingByteArray.java` | `jadx-core/src/test/java/jadx/tests/integration/conditions/TestTernaryOneBranchInConstructor.java`, `test()` / nested `TestCls` and `test2()` / nested `TestCls2`: constructor `this(...)` delegation. | A delegated constructor target owns one effectful array assignment; trace asserts target initialization occurs once and before the delegating constructor body. The array-field plus delegation combination is an extension. |
| `DifferentRhsByteArray.java` | `jadx-core/src/test/java/jadx/tests/integration/others/TestFieldInitNegative.java`, `test()` / nested `TestCls`: a field assignment with an intervening side effect is kept in the constructor. `.../others/TestFieldInit2.java`, `test()` / nested `TestCls`: a class with multiple constructors. | Two direct-super constructors assign different effectful array RHSs. This is a negative control for refusing a shared initializer; that exact combination is an extension. |

JADX implementation context for the positive multi-constructor idea is `jadx-core/src/main/java/jadx/core/dex/visitors/ExtractFieldInit.java`: `moveCommonFieldsInit` (around lines 124–169) gathers candidate writes from constructors and requires a common sequence; `collectFieldsInit` (around lines 172–198) limits candidates to eligible blocks; `checkInsn` (around lines 240–264) checks which RHS forms can move. This is algorithm context, not proof that these exact effectful fixtures are accepted by JADX. The `this(...)` fixture deliberately tests the ownership/counting boundary separately from that common-direct-super case.

The runner checks array values, per-instance array identity, evaluation order, and constructor-body order. Expected source-oracle trace shapes are asserted in `InstanceFieldInitRunner.java`; no output is recorded here because the runner has not been executed. In the direct-super positive fixture, each constructor must evaluate `10,20` once before its own body. In the delegation fixture, the target evaluates `21,22` once, then its body runs, then the delegating body. The negative control must preserve `31` versus `32` and each corresponding constructor trace.

## Review and execution plan

1. Review these minimal sources against the intended bytecode shapes. In particular, verify the generated constructors retain direct `Object.<init>` calls for the first and third fixtures, and the no-arg delegating constructor invokes `this(int)` for the second.
2. After review, compile each complete class set under the already established JDK profiles and preserve the original-class Runner output as the semantic oracle.
3. Decompile the complete class sets with the frozen CLI configuration. Compare source presentation and recompile the full decompiled class set under both profiles before claiming a rendering result.
4. Treat `CommonDirectSuperByteArray` as the positive common-write case, `ThisDelegatingByteArray` as the exactly-once ownership case, and `DifferentRhsByteArray` as a refusal control. Do not infer broad constructor support from these probes alone.

No compiler, JDK, JADX, Jarde CLI, or other build/test command was run while preparing this directory.
