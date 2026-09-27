# 固定 `TestTryCatchFinally12.runTest` 的 switch/try 所有权债务

固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestTryCatchFinally12.TestCls.runTest(II)String` 是一层 `try { switch (testNumber) { case 1/2/3: testN(excType); } } catch (IllegalArgumentException e) { ... }`，然后在保护范围外执行 `return sb.toString()`。它本身没有 `finally`；清理发生在被调用的 `test1/2/3` 中。

Java 8 class 的具名异常行是 `[11,61) → 64 IllegalArgumentException`。BCI 12 为 switch dispatch，case 入口为 BCI 40、48、56，共同正常汇合点 BCI 61 位于保护范围外，handler 从 BCI 64 开始，后续 return 从 BCI 79 开始。主线 `61a167b2` 的 Jarde 报 `jre_region_ownership_overlap`：canonical BCI 40 在完成的 Region tree 中有多个 owner，故整方法安全引用，完整固定类不能重编。这个拒绝与本轮 `test1/2/3` 的 `jre_guard_finally_copy` 是独立缺口。

主线 `9440dd30` 的隔离诊断进一步拿到完成树：`ROOT[0] Region::Switch` 的 case 1 arm 是 BCI 40 的 `Region::Fallback(ExceptionEdge, handler ordinal 0)`；`ROOT[1]` 又是同一个 BCI 40 的 sibling fallback。case 2/3 的 BCI 48/56 也各被 arm 和 sibling 重复认领。形成点是 `region.rs::switch_region` 将 `arm_run` 收作 case arm，又把 `entered` 中的同一 fallback 过滤后追加到 switch 后的 root run（约 8955–8993 行）；这里的 sibling 并非未由 arm 持有的尾部。`overlapping_owner` 拒绝正确地阻止双 owner。这个局部重复应单独修正，但只消掉重复不会恢复外层 catch：诊断树根本没有 `Region::Try`，具体哪一个 `guard::catches` 前提未闭合仍待追踪，不能猜测。

完整正例的候选边界是单条具名 catch 完整覆盖 switch dispatch 和各 case 正文、唯一正常 join 在范围外；固定类行从 BCI 11 起，而 switch opcode 在 BCI 12、canonical block 从 BCI 0 起，必须明确处理同块 lead。保护范围只覆盖部分 arm、切入 case body、handler 交叉、arm 间跳转、非唯一 join、handler 回流到 try 正文应拒绝。已有 verifier 有效的 `PartialSwitchCatch` 临时负例只保护 case 1 调用，Jarde 没有把它扩成整 switch 外层 catch；它仍有其它局部 fallback，不能声称完整恢复。不能因现有 `TestSwitchWithTryCatch`（switch 外层、每个 case 内层 try）通过或失败，就推定此反向嵌套形态的结果。
