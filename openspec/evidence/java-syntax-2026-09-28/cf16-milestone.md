# CF-16：按固定 JADX 测试推进的 finally 里程碑

基准为 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `trycatch` 目录 24 个主归属测试文件，文件清单见[控制流账本](../jadx-feature-inventory-2026-09-27/control-flow.md)。**文件数不等于已恢复的 lowering 数**：一个文件可含多个 profile、`@NotYetImplemented` 或禁用编译的断言。下面只给当前证据等级和下一步，不把窄切片当成整个文件通过。

| 组 | 文件 | 当前证据 | 下一关口 |
| --- | --- | --- | --- |
| 固定切片已由 root 验收 | `TestTryCatchFinally`、`4`、`11`、`12`、`13`、`14`、`16`、`17` | 同布局 Java 8 目标或完整类的原/JADX/Jarde 重编、验证运行及对应反例分别验收；[Test4](cf16-test4-nested-cleanup/README.md)的四行嵌套清理目标有 31 个 BCI 来源与 5 个有效近邻，固定目标仅验证正常路径；[Test11](cf16-test11-loop-finally/implementation-evidence.md)的双行 handler 循环覆盖 10 条清理异常路径和 33 个 BCI；`13` 的扩围样本揭示 JADX 漏第二次清理。 | 只在新 profile/组合出现时扩验，不重做已通过切片。 |
| 已取证的实现差距 | `TestEmptyFinally`、`TestTryCatchFinally3`、`5`、`9` | [空 finally](cf16-testemptyfinally/README.md)固定具名 IOException catch 加透明 catch-all 的两行形态；[Test3](cf16-test3-catch-finally/README.md)固定四行、正文 foreach 与 catch 日志；[Test5](cf16-test5-multi-return/README.md)固定三行、两个返回值和正常可达循环；[Test9](cf16-test9-catch-finally/README.md)固定两行、可空资源和保存返回值。四个目标 Jarde 均安全拒绝。Test9 原 class 与 JADX DEX 输出会关闭资源，JADX Java-input 输出漏关，不能照抄。 | Test3 已有方法级 OpenSpec，复用 `SharedFinally` 的 joined completion；其余按透明 handler、保存的退出值、可空清理分别设计有界证书。不要把 Test5 正文循环误认成 Test11 的异常 handler 循环，也不要把 Test9 的 Java-input 结果当语义正例。 |
| 已审计为弱正例或非正例 | `TestTryCatchFinally8`、`10`、`15`、`18`、`19` | `8`、`19` 目标被标 `@NotYetImplemented`；`10`、`15` 关闭编译且已按 DEX/Smali 与 Java 8 等价样本分开取证；`18` 的 Java 8 测试标未完成，另有 DX/D8 文本断言且关闭编译。 | 不把这些文件的文本断言算作 JADX 可重编正向能力；另造 verifier 可运行的 JVM 对照后才决定 Jarde 任务。 |
| JVM 输入 profile 已区分 | `TestTryCatchFinally6` | [debug/no-debug 独立审计](cf16-test6-profile-audit/README.md)确认默认 JADX 测试走 DX；Java classfile CLI 的两个输出未保持同一局部别名，Jarde 两者安全拒绝。 | DEX 断言不外推 JVM；后续以原 classfile 语义另定恢复边界，不复制失真的 Java-input 输出。 |
| 待按文件取证 | `TestFinally`、`TestFinally2`、`TestFinally3`、`TestFinallyExtract`、`TestTryCatchFinally2`、`7` | `TestFinally3` 有单独未完成测试；其余尚无完整三方物理与行为证据。 | `2` 的循环先核正常/异常可达性和清理完成值，再决定能否复用已证机制；其余按 JVM lowering 聚类，不按文件名机械建机制。 |

此表为 **8 + 4 + 5 + 1 + 6 = 24** 个文件的取证队列，不是 24 个全量验收承诺。第一里程碑是关闭已证差距与有效正例：固定物理 class、原/JADX/Jarde 完整源码、`javac --release 8`、`java -Xverify:all` 的正常及可注入异常路径、来源覆盖、verifier 有效负例。JADX 的 `MarkFinallyVisitor.processTryBlock` 与 `findCommonInsns` 可作为重复副本寻找顺序的参考；Jarde 仍在既有 Guard/Region/Build 中按异常表、SSA 和完整所有权证明，不能仅按相似指令删副本。

优先次序来自复用关系：Test11 异常组件循环与 Test4 嵌套 catch 清理已验收；Test3 的方法级证书在实施中；空 finally、Test5 和 Test9 已取证，待按各自异常表与退出值设计任务。Test9 的正确性基准是固定原 class；JADX DEX 输出可作正向参照，Java-input 输出是已证反例。低证据文件只调整分母或作为负例，不挤占先完成有效 JVM 正例的实施容量。
