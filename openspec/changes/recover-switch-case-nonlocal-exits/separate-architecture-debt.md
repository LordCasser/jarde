# 单独记录：jsr clone 内的自然循环视图

边界控制 `tests/fixtures/p3-switch-jsr-clones/v49/SwitchLoopJsrClone.class` 是完整、合法的 v49 输入。Java 8 与 23 均通过 `-Xverify:all`，独立 URLClassLoader 核 CodeSource 后得到 `run(0)=0`、`run(3)=12`。同次 Reader 分析产生两个物理 switch BCI 相同、clone path 不同的上下文。

实际 `boundary-focused-root-v1` 中，试图寻找第一个 clone 的自然循环失败，其他 464 个库测试通过。不能构造虚假的 loop frame 把它当成完整 switch-loop proof 的正例。当前边界测试改为在现有 `switch_completion_edge` 上核同 clone 的真实 direct 边通过、另一 clone 的同 BCI 目标拒绝。没有改 class 字节，也没有声称错配图经过 JVM 验证。

后续若需要恢复这类历史编译器输入，应先分析 NormalFlowView 对 Call/Return 上下文与 dominator 入口的定义，再确定是否需要改动。该问题与本次 Java 8 case-return/break/mixed 的实际恢复分开，不在当前实现中增加机制。测试不把“没有自然循环”固定成未来必须保留的行为。
