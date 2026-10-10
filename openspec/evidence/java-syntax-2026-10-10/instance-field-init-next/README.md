# Instance field initializer follow-up fixture

Root 已完成实际双 JDK 对照并独立验收，入口为 `results/baseline-root-verification-v7.json`。完整三类加 Runner 共 8 腿、31 命令：原始源码 2/2、Jarde 2/2 raw 一致；fresh JADX 1.5.6 default/none 两种配置各重编两次，4/4 编译成功但运行失败，准确失败是 DifferentRhsByteArray 的第二个构造器把 mark(32) 误改为 mark(31)。不是 8/8 语义通过。

The current `array-field-initializers-literal` baseline already covers literal instance-array initializer rendering and runtime semantics. This follow-up isolates the remaining constructor-presentation question: can identical instance-field writes from all direct-super constructors become one field initializer while preserving effect order, and can a `this(...)` chain initialize that field exactly once? The fixtures are candidate probes, not an implementation claim or a claim that existing JADX tests cover these exact combinations.

| Fixture | Existing JADX anchor | Exact extension in this fixture |
| --- | --- | --- |
| `CommonDirectSuperByteArray.java` | `jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrayInitField.java`, `test()` / nested `TestCls`: an instance `byte[]` field initializer and its array-literal rendering. `.../arrays/TestArrayInit.java`, `test()` / nested `TestCls.test2()`: an array literal assigned to an instance field in a method. | Two direct-super constructors contain the same effectful `byte[]` RHS, followed by different constructor-body trace writes. This multi-constructor common-write combination is an extension, not an existing assertion. |
| `ThisDelegatingByteArray.java` | `jadx-core/src/test/java/jadx/tests/integration/conditions/TestTernaryOneBranchInConstructor.java`, `test()` / nested `TestCls` and `test2()` / nested `TestCls2`: constructor `this(...)` delegation. | A delegated constructor target owns one effectful array assignment; trace asserts target initialization occurs once and before the delegating constructor body. The array-field plus delegation combination is an extension. |
| `DifferentRhsByteArray.java` | `jadx-core/src/test/java/jadx/tests/integration/others/TestFieldInitNegative.java`, `test()` / nested `TestCls`: a field assignment with an intervening side effect is kept in the constructor. `.../others/TestFieldInit2.java`, `test()` / nested `TestCls`: a class with multiple constructors. | Two direct-super constructors assign different effectful array RHSs. This is a negative control for refusing a shared initializer; that exact combination is an extension. |

JADX implementation context for the positive multi-constructor idea is `jadx-core/src/main/java/jadx/core/dex/visitors/ExtractFieldInit.java`: `moveCommonFieldsInit` (around lines 124–169) gathers candidate writes from constructors and requires a common sequence; `collectFieldsInit` (around lines 172–198) limits candidates to eligible blocks; `checkInsn` (around lines 240–264) checks which RHS forms can move. This is algorithm context, not proof that these exact effectful fixtures are accepted by JADX. The `this(...)` fixture deliberately tests the ownership/counting boundary separately from that common-direct-super case.

The runner checks array values, per-instance array identity, evaluation order, and constructor-body order. Expected source-oracle trace shapes are asserted in `InstanceFieldInitRunner.java`; 实际原输出与两份 Jarde 输出逐字相同，完整 raw 在 baseline-root-v3/streams。 In the direct-super positive fixture, each constructor must evaluate `10,20` once before its own body. In the delegation fixture, the target evaluates `21,22` once, then its body runs, then the delegating body. The negative control must preserve `31` versus `32` and each corresponding constructor trace.

## Review and execution plan

1. Review these minimal sources against the intended bytecode shapes. In particular, verify the generated constructors retain direct `Object.<init>` calls for the first and third fixtures, and the no-arg delegating constructor invokes `this(int)` for the second.
2. After review, compile each complete class set under the already established JDK profiles and preserve the original-class Runner output as the semantic oracle.
3. Decompile the complete class sets with the frozen CLI configuration. Compare source presentation and recompile the full decompiled class set under both profiles before claiming a rendering result.
4. Treat `CommonDirectSuperByteArray` as the positive common-write case, `ThisDelegatingByteArray` as the exactly-once ownership case, and `DifferentRhsByteArray` as a refusal control. Do not infer broad constructor support from these probes alone.

上述计划最初为 preparation，现已执行。JADX 从同一 javac23 三类 jar 各 fresh 提取 default/none 一次，再以 JDK8/23 编译完整生成三类；不声称双 JDK 各自提取。Jarde 对两个原始 class set 各提取三类。全部源码原样编译，空 CP/SP、仅新 classes -Xverify:all；Runner 只加必要 package。

## root 验收与下一片

baseline-root-v1 保留首次脚本最后汇总 KeyError: label 的失败现场；baseline-root-v3 完成全部采集，执行 exit1 正是已记录 JADX 反例，不是 Jarde 产品失败。root verifier v2-v6 的 javap flags、argv位置、BLAKE3/SHA-256、method identity schema 和 package classpath错误逐项修正，v7实际执行接受；详见 results/verifier-root-review.md，不改任何原 raw。

CommonDirectSuperByteArray 的共同数组初值被 JADX 提升而 Jarde保留；ThisDelegatingByteArray 两者都保留 target ctor赋值且恰一次。下一 MVP只处理全部 direct-super 构造器共同连续数组前缀，遇任意 this() 委托保守不提升，构造器图不是本片必要机制。DifferentRhsByteArray 必须继续保留31/32，不能借同形调用认定语义相等。静态源码审计见 results/jadx-common-init-comparison-audit-luna.md：本地 dev checkout 的 isSame软比较是可信候选原因，但未核1.5.6真实内部图，不冒称确定二进制根因。
