# CF-08 带效果双出口：主线独立验收

root 审阅合入的 `5848b1b0`：新 `LoopForm::Endless` 只在三个自然循环块、两个独占出口块、单入口/单回边、唯一共同后继、无 handler、逐块 Code/SSA 指令对应，以及 BCI 35 的局部 φ 两个定义和唯一消费全部闭合后发布。循环体首访 header，其比较由体内 `If` 拥有；效果臂和直达臂各有一条 `break`，纯回边不重复执行效果。候选不闭合仍走原有引用路径。

root 用主线重新构建 CLI（SHA-256 `4c7ec29e97d9a4dd3b2879c5badf0a8c9b3374bb30763b7e4ae539ca1f6ca0b7`），运行固定 [重放脚本](../replay-effectful.py)，独立输出保存在 `/tmp/jarde-root-cf08-lioDL5gS`。原始、固定 JADX、Jarde 的完整 Java 8 源码均重编成功，并以 `java -Xverify:all` 输出 `8:1 / 3:0 / 8:1`；Jarde 完整源码无 `@bytecode`。`p3_effectful_exits` 3/3、`p3_java_recovery` 37/37、`p3_loop_exit_gateways` 3/3、`p3_loop_terminal_return` 4/4 均通过，workspace check、格式与 OpenSpec 严格校验通过。

这完成了带效果双出口隔离样例；固定 JADX `TestNotIndexedLoop` 的外层分支与循环后 slot 2 汇合仍安全拒绝，CF-08 整单元保持已证差距。
