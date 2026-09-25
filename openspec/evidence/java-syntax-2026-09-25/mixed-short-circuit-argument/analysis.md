# 混合短路值作为调用实参的独立 parity 边界

[冻结 Java 8 源码/class/Runner](../../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-argument/README.md)中的 `call(Z)V` 把 `(a && b()) || c()` 传给 `sink(Z)V`。`javac --release 8 -g:none` 产物 SHA-256 为 `5dbdfc2e66a44bab18c3ab39b824a0173a09cc38fa1bdf33d5ff9fdf04a2eeaa`。Runner 遍历 `a/bValue/cValue` 的八组输入，记录最终字段值以及 `b()`、`c()`、`sink()` 各自调用次数。原 class 的 [八行输出](original-run.txt)与 JADX 1.5.6 [重编完整类输出](jadx-run.txt)逐字一致，均经 `java -Xverify:all`；JADX 加的伪 `package defpackage;` 仅在临时重编副本中去掉。

[反汇编](javap.txt)中 BCI 1/7/13 三次测试汇入 BCI 16/20 的 `1/0` producer，BCI 21 是唯一 `invokestatic sink:(Z)V`，BCI 24 返回 `void`。它与已验收的[混合字段写入](../mixed-short-circuit-field/analysis.md)及待实施的[直接返回](../mixed-short-circuit-return/analysis.md)具有同一闭合测试图，区别只在唯一 Phi 消费者是布尔调用实参。当前 [Jarde 基线完整类](jarde-baseline-MixedBooleanArgument.java)给 `call` 一个 `jre_region_ownership_overlap` 引用，`quality=fallback`、`content=explanation_only`。该完整类虽然 [Java 8 重编](jarde-baseline-javac.log)成功，但 [八行运行](jarde-baseline-run.txt)全部不同于原件，尤其 `sinkCalls=0` 而原件每条路径均为 1；可编译并不代表恢复了调用行为。基线 CLI 在混合字段图验收后、直接返回实现前冻结。

后续应单列 OpenSpec：复用同一图和 SSA/唯一 Phi 证明，只在调用指令真实 Methodref 描述符为 `(Z)V`、Phi 恰好绑定该参数且无其它参数/接收者求值顺序歧义时先开放最小子集，交给现有 invocation argument/Call AST 发射。任何额外前驱、异常边、第二消费者、参数重排或无法证明的 Methodref 必须继续整体拒绝；不因这个正例扩大当前直接返回任务。

直接返回任务验收后，用比 `region.rs` 更新更晚编译的当前 CLI 再导出[调用实参报告](jarde-after-return-report.json)：完整 Java 文本与上述 baseline 逐字相同，`call(Z)V` 仍为 `fallback/mixed/explanation_only`，仅引用 BCI `{0,1,4,7,10,13,16,17,20,21,24}`，没有执行 `sink`。因此下一项实施的基线已经从当前共享树复核；报告中与 baseline 不同的其它元数据不作为文本等价或行为恢复的依据。

调用实参任务交付后，根代理从更新后的 CLI 重新导出[完整类](jarde-after-argument-MixedBooleanArgument.java)和[报告](jarde-after-argument-report.json)：`call(Z)V` 为 `structured/java/contains_statements`，正文恰有一次 `sink(...)` 且没有 `@bytecode`。该类经 [Java 8 重编](jarde-after-argument-javac.log)和 `java -Xverify:all`，其[八行输出](jarde-after-argument-run.txt)与冻结原件及 JADX 逐字相同。多参数 `(ZI)V` 与实例 `invokevirtual (Z)V` 真实近例仍整图拒绝；独立根验收记录见 [verification](../../../changes/recover-short-circuit-invocation-arguments/verification.md)。
