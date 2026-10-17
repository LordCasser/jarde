# CF-16：按固定 JADX 测试推进的 finally 里程碑

基准为 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `trycatch` 目录 24 个主归属测试文件，文件清单见[控制流账本](../jadx-feature-inventory-2026-09-27/control-flow.md)。**文件数不等于已恢复的 lowering 数**：一个文件可含多个 profile、`@NotYetImplemented` 或禁用编译的断言。下面只给当前证据等级和下一步，不把窄切片当成整个文件通过。

## 2026-10-18 增补：仅异常完成 finally 叶片

### 后续原始 TestFinally3 profile 已验收

[recover-direct-array-constructor-arguments](../../changes/recover-direct-array-constructor-arguments/) 已7/7，产品81b0684d5自身CI39062097681的4jobs/52步骤和双seed各381targets、3707/0/98已独立接受。原始JADX源码新编译的debug/noDebug两输入已区别于旧转写Tf3：100物理方法配置全BCI/UTF8来源、20次原样整类重编、40次隔离运行320case通过，88非目标配置不变；JADX六次整类重编均因两处inputStream未初始化失败，无rebuilt runtime。major55测试harness限制owner重编只用javac23，不冒称javac8接受原owner；原close为空，不能由driver推断其调用次数或顺序。

最终605CLI相对完整矩阵e01CLI只删除重复证明遍历，40真实请求重放核完整源码和非usage报告恒同；只ir_items/elapsed_millis变化，按相同源码复用实际JDK证据，不冒称重新运行。完整证据见该change的results/tf3-final-matrix-root-v1、output-replay-root-v1、workspace-acceptance-root-v3及own-product-ci-root-v1。TestFinally3的原始debug/noDebug缺口已闭合；Smali仍独立记账，下表2026-09-30的相关待办属于历史。此项与显式finally return/throw债务分开，不增加CF16或71/612整单元完成数。

[recover-exception-only-finally-completion](../../changes/recover-exception-only-finally-completion/) 在既有Guard/Finally完成枚举及Try AST中补齐直线try抛异常、finally正常落穿的完成类型；handler只重抛同一捕获值，cleanup异常自然覆盖pending primary。产品3fa506df0已推送，最终7287CLI的独立整类144个重建运行case及入口参数24case通过；旧15class正文/来源保持，两fresh native seed各378targets、3700/0/98，来源与逐target计数相同。精确产品自身CI39051492999已独立接受，4jobs/52steps、双seed各378targets/3700通过/0失败/98忽略，9关键各一次通过；本leaf11/11，两版strict各366/0；后续文档不冒称产品CI的新运行。

原FinallyCompletion七方法均保留，本次改善tryThrowsFinallyRuns；finallyReturns/finallyThrows的原样整类编译红项继续作为独立欠账。当前71单元/612文件及本表24主归属文件数保持。下表为2026-09-30首轮范围记录；后续TestFinally3的debug/noDebug/Smali须区分输入来源，转写Tf3不替代原nested TestCls的独立profile证据。

| 组 | 文件 | 当前证据 | 下一关口 |
| --- | --- | --- | --- |
| 固定切片已由 root 验收 | `TestEmptyFinally`、`TestTryCatchFinally`、`3`、`4`、`5`、`11`、`12`、`13`、`14`、`16`、`17` | 同布局 Java 8 目标或完整类的原/JADX/Jarde 重编、验证运行及对应反例分别验收；[空 finally](cf16-testemptyfinally/implementation-results.md)的两行透明 handler 已恢复为完整类普通 catch，三条路径和三个有效近邻验收；[Test3](cf16-test3-catch-finally/results/acceptance.txt)四行正文循环已方法级恢复，九条路径、34 个 BCI 来源及七个有效近邻验收，整类 `<clinit>` 仍独立回退；[Test4](cf16-test4-nested-cleanup/README.md)的四行嵌套清理目标有 31 个 BCI 来源与 5 个有效近邻，固定目标仅验证正常路径；[Test5](../../changes/recover-multi-return-loop-finally/verification/root-acceptance.md)的三行双返回、正文 do-while 完整类通过九路径、45 个 BCI 来源和六个有效近邻验收；[Test11](cf16-test11-loop-finally/implementation-evidence.md)的双行 handler 循环覆盖 10 条清理异常路径和 33 个 BCI；`13` 的扩围样本揭示 JADX 漏第二次清理。 | 只在新 profile/组合出现时扩验，不重做已通过切片；Test3 整类静态初始化另案处理。 |
| 已取证的实现差距（首片已闭合） | `TestTryCatchFinally2`、`7`、`9` | [Test2](cf16-test2-loop-finally/README.md)固定两行、三个正常可达正文循环和无返回值的 `close()` 清理，已按 [Void 完成证书](../../changes/recover-void-loop-finally/verification/README.md)恢复并 root 复核（一个 `try/finally`、一次 `close()`、三循环、89 BCI 来源、四方六路径一致）；[Test7](cf16-test7-audit/README.md)与[Test9](cf16-test9-catch-finally/README.md)已由主线 `80523785`、`fcc864ce` 闭合。Test9 的正确性基准仍是固定原 class；JADX Java-input 输出是已证反例。 | 家族残留按变体另列：TestFinally 四文件已[取证](../java-syntax-2026-09-30/testfinally-patrol/README.md)。 |
| 已审计为弱正例或非正例 | `TestTryCatchFinally8`、`10`、`15`、`18`、`19` | `8`、`19` 目标被标 `@NotYetImplemented`；`10`、`15` 关闭编译且已按 DEX/Smali 与 Java 8 等价样本分开取证；`18` 的 Java 8 测试标未完成，另有 DX/D8 文本断言且关闭编译。 | 不把这些文件的文本断言算作 JADX 可重编正向能力；另造 verifier 可运行的 JVM 对照后才决定 Jarde 任务。 |
| JVM 输入 profile 已区分 | `TestTryCatchFinally6` | [debug/no-debug 独立审计](cf16-test6-profile-audit/README.md)确认默认 JADX 测试走 DX；Java classfile CLI 的两个输出未保持同一局部别名，Jarde 两者安全拒绝。 | DEX 断言不外推 JVM；后续以原 classfile 语义另定恢复边界，不复制失真的 Java-input 输出。 |
| 家族收官（2026-09-30） | `TestFinally`、`TestFinally2`、`TestFinally3`、`TestFinallyExtract` | [首次三方取证](../java-syntax-2026-09-30/testfinally-patrol/README.md)后四片全部落地并 root 复核：Tf4 [flag-conditional](../../changes/recover-flag-conditional-finally/)、Tf1 [local-null-conditional](../../changes/recover-local-null-conditional-finally/)、Tf2 [null-lead-straight](../../changes/recover-null-lead-straight-finally/)、Tf3 [segmented-null-lead](../../changes/recover-segmented-null-lead-finally/)（合并主线 bf6ffad4 全仓 2700/0）。JADX 侧 Tf3 输出 definite-assignment 编译失败为新登记反例；Tf3 helper 的嵌套数组分配回退为独立既有限制。 | 本组 24 文件全部完成首轮取证与正例闭合；CF-16 无登记已证差距，剩余为各家族变体扩验（InnerClasses family 等）与 `TestFinally3` noDebug/smali profile 分母调整。 |

此表为 **11 + 3 + 5 + 1 + 4 = 24** 个文件的取证队列，不是 24 个全量验收承诺。第一里程碑是关闭已证差距与有效正例：固定物理 class、原/JADX/Jarde 完整源码（Test3 为明确标注的方法级 harness）、`javac --release 8`、`java -Xverify:all` 的正常及可注入异常路径、来源覆盖、verifier 有效负例。JADX 的 `MarkFinallyVisitor.processTryBlock` 与 `findCommonInsns` 可作为重复副本寻找顺序的参考；Jarde 仍在既有 Guard/Region/Build 中按异常表、SSA 和完整所有权证明，不能仅按相似指令删副本。

优先次序来自复用关系：Test11 异常组件循环、Test4 嵌套 catch 清理、Test3 方法级共享清理、Test5 双返回正文循环和空 finally 的固定切片已验收；Test2 与 Test9 已取证，待按各自异常表与退出值设计任务。Test9 的正确性基准是固定原 class；JADX DEX 输出可作正向参照，Java-input 输出是已证反例。低证据文件只调整分母或作为负例，不挤占先完成有效 JVM 正例的实施容量。
