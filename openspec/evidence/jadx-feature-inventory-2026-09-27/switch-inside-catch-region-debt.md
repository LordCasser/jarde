# 固定 `TestTryCatchFinally12.runTest` 的 switch/try 所有权债务

固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestTryCatchFinally12.TestCls.runTest(II)String` 是一层 `try { switch (testNumber) { case 1/2/3: testN(excType); } } catch (IllegalArgumentException e) { ... }`，然后在保护范围外执行 `return sb.toString()`。它本身没有 `finally`；清理发生在被调用的 `test1/2/3` 中。

Java 8 class 的具名异常行是 `[11,61) → 64 IllegalArgumentException`。BCI 12 为 switch dispatch，case 入口为 BCI 40、48、56，共同正常汇合点 BCI 61 位于保护范围外，handler 从 BCI 64 开始，后续 return 从 BCI 79 开始。主线 `61a167b2` 的 Jarde 报 `jre_region_ownership_overlap`：canonical BCI 40 在完成的 Region tree 中有多个 owner，故整方法安全引用，完整固定类不能重编。这个拒绝与本轮 `test1/2/3` 的 `jre_guard_finally_copy` 是独立缺口。

当前报告只给出重复 BCI，未给出两条具体 Region 路径。下一步应在不改变生产判定的情况下获取该树的 owner 路径，再用固定方法和 verifier 有效近邻反例提出独立 OpenSpec。候选边界是单条具名 catch 完整覆盖 switch dispatch 和各 case 正文、唯一正常 join 在范围外；保护范围只覆盖部分 arm、切入 case body、handler 交叉、arm 间跳转、非唯一 join、handler 回流到 try 正文应拒绝。不能因现有 `TestSwitchWithTryCatch`（switch 外层、每个 case 内层 try）通过或失败，就推定此反向嵌套形态的结果。
