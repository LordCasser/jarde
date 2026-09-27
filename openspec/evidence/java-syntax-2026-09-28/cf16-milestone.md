# CF-16：按固定 JADX 测试推进的 finally 里程碑

基准为 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `trycatch` 目录 24 个主归属测试文件，文件清单见[控制流账本](../jadx-feature-inventory-2026-09-27/control-flow.md)。**文件数不等于已恢复的 lowering 数**：一个文件可含多个 profile、`@NotYetImplemented` 或禁用编译的断言。下面只给当前证据等级和下一步，不把窄切片当成整个文件通过。

| 组 | 文件 | 当前证据 | 下一关口 |
| --- | --- | --- | --- |
| 固定切片已由 root 验收 | `TestTryCatchFinally`、`12`、`13`、`14`、`16`、`17` | 同布局 Java 8 目标或完整类的原/JADX/Jarde 重编、验证运行及对应反例已经分别验收；`13` 的扩围样本还揭示 JADX 漏第二次清理。见[CF-16 账本](../jadx-feature-inventory-2026-09-27/control-flow.md)。 | 只在新 profile/组合出现时扩验，不重做已通过切片。 |
| 正在按 OpenSpec 实现 | `TestTryCatchFinally4`、`11` | [Test4 四行嵌套清理](cf16-test4-nested-cleanup/README.md)与[Test11 handler 循环](cf16-test11-loop-finally/README.md)已有固定 class 和对照；当前 Jarde 均安全拒绝。 | 两项分别完成联合证明、完整源码三方运行与 verifier 有效近邻；root 复核后合并。 |
| 正在取证 | `TestTryCatchFinally3` | 活动断言同时要求 `catch(Exception)`、foreach 与 `finally`；固定物理布局和可观察路径尚待独立三方证据。 | 先确认与既有 shared-finally 证书的差距，必要时才开新 OpenSpec。 |
| 已审计为弱正例或非正例 | `TestTryCatchFinally8`、`10`、`15`、`18`、`19` | `8`、`19` 目标被标 `@NotYetImplemented`；`10`、`15` 关闭编译且已按 DEX/Smali 与 Java 8 等价样本分开取证；`18` 的 Java 8 测试标未完成，另有 DX/D8 文本断言且关闭编译。 | 不把这些文件的文本断言算作 JADX 可重编正向能力；另造 verifier 可运行的 JVM 对照后才决定 Jarde 任务。 |
| 待按文件取证 | `TestEmptyFinally`、`TestFinally`、`TestFinally2`、`TestFinally3`、`TestFinallyExtract`、`TestTryCatchFinally2`、`5`、`6`、`7`、`9` | 已知 `TestEmptyFinally` 明确要求消除空 finally；`TestFinally3` 有单独未完成测试；`TestTryCatchFinally6` 的无 debug profile 明言不能证明局部合并。其余尚无完整三方物理与行为证据。 | 先做 `6` 的 debug/no-debug 边界和 `9` 的可空清理；`2`、`5` 的嵌套循环在 Test11 循环证明落地后重放；其余按 JVM lowering 聚类，不按文件名机械建机制。 |

此表为 **6 + 2 + 1 + 5 + 10 = 24** 个文件的取证队列，不是 24 个全量验收承诺。第一里程碑是关闭已证差距与有效正例：固定物理 class、原/JADX/Jarde 完整源码、`javac --release 8`、`java -Xverify:all` 的正常及可注入异常路径、来源覆盖、verifier 有效负例。JADX 的 `MarkFinallyVisitor.processTryBlock` 与 `findCommonInsns` 可作为重复副本寻找顺序的参考；Jarde 仍在既有 Guard/Region/Build 中按异常表、SSA 和完整所有权证明，不能仅按相似指令删副本。

优先次序来自复用关系：先验收 Test11 的异常组件循环与 Test4 的嵌套 catch 清理；随后对 Test3、6、9 做相互独立的固定取证。`2` 和 `5` 含循环/泛型组合，应在 Test11 后观察是否已经自然受益，再决定是否新增机制。低证据文件只调整分母或作为负例，不挤占先完成有效 JVM 正例的实施容量。
