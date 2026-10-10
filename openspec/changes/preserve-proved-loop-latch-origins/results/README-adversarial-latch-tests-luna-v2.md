# Latch-origin 对抗测试增量（Luna v2）

`adversarial-latch-tests-luna-v2.patch` 是仅涉及两个既有测试文件的私有候选 diff。它只加强已有断言，不新增 fixture、测试函数、计数或通用源码解析器。v1 保留原样。

根目录 `tests/p3_loop_arm_join.rs` 的既有 `LoopIfJoin.run` 正例中，BCI 23 是物理 `goto 15`。增量断言该 BCI 有 source-map span，且至少一个 derived span 从 `recovery.text` 的实际 `while (` 起始。原有 loop-arm join 结构和外层 tail 断言保留。

`p3_loop_body_double_jumps.rs` 的既有 transfer-anchor 循环现在也检查每个 BCI 都有 source-map span，并拒绝其任一来源 span 以 while/for 循环头起始。检测只看 span 的第一行，同时识别 `label: while` 和 `label: for`，不会把第一行是 `if`、正文内含嵌套 loop 的 span 误判为 loop 头。原有 region owner 断言保留。

断言设计依据 `transfer-map-old-cli-root-v1/anchor-spans.json` 记录的旧 CLI 观察：列出的 break/continue anchors 落在 transfer 或外层 if span；`LoopIfJoin.run` 的 BCI 23 当时没有 source-map span。这些观察是在 candidate 应用前由旧冻结 CLI 得到，仅用于设计断言，不能证明 candidate 或本 delta 通过。

本次只做静态结构与文件 hash 检查；没有修改真实测试源码，没有运行 Git、Cargo、JDK 或 CLI。root 仍需自行执行 `git apply --check` 和后续验证。
