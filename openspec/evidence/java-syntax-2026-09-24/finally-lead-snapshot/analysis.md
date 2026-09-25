# 同一融合块中 try 前置赋值与返回值快照

`FinallyLeadSnapshot.java`（SHA-256 `9a9ca7ae6d49aa476c2cdd67797c9efede39ad6475801ac8f01584bb65975a97`）先写静态字段 `value = 41`，再执行 `try { return value; } finally { value = 99; }`。source-only runner（SHA-256 `901009e13d3dfa02e251d417af634c0973e33af19f202ee4d56e444179b2c09c`）同时观察保存的返回值和 cleanup 后的字段。`javac 23.0.1 --release 8 -g:none` 生成 287 B Java 8 class，SHA-256 `187546178ad20c70ac00b8ab9df464a6cb9bf617dae6794dc16884463a003a3c`；`java -Xverify:all` 输出 `41:99`。

字节码在 BCI 0–5 写 `41`，异常表只保护 `[5,9)` 的 `getstatic; istore_0`；BCI 9–14 与 17–22 分别写 `99`，BCI 14–15 返回旧局部，BCI 16 开始的 handler 重抛原异常。由于规范化 CFG 把前置赋值与受保护片段并入同一块，先前 `finally@1` 的 `proof.protected.0 == start` 门槛拒绝它。基线 Jarde CLI SHA-256 `8d93cf642dbca5de2a625677c3e13a44a989b2d5824ed43058d5724927744faf` 输出完整类时仍有两处 `@bytecode` 引用，未编译；这是当时正确的保守行为。JADX 1.5.6 完整类可重编并得到 `41:99`，但在 try 内多写了无用的 `int i = 99`。

最小接缝是已有 `Plan.lead`：单独证明 `[0,5)` 不受该异常表保护、可按原顺序呈现，作为 try 前的语句；`Plan.body` 仍严格是 `[5,9)`，saved return、正常/handler cleanup 副本和半开保护范围的现有证明不变。`explained` 与 owned 块须同时覆盖 lead，不能因融合块归属而重复发射或把 lead 错放进 try。先只处理这个无分支、无 `pop` 的样本；`CleanupBoundaries.snapshotReturn` 还含 discarded 调用结果，属后续相邻边界，不用放宽全局 `statement_free` 来暗中准入。

[2.3 lead 子切片验收](../../../changes/recover-proved-finally-cleanup/verification-2.3-lead.md)已复用该接缝。最终 CLI 输出一条 try 前字段赋值、保存的返回值和唯一 finally 字段写；root 独立重编完整类并在 `-Xverify:all` 下复现 `41:99`。只将异常表 `start_pc` 从 5 改为 2 的 JVM 有效变体让 BCI 0 的栈值跨边界，最终规则继续引用，说明准入条件不是“任意直线前缀”。
