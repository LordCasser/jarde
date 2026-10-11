# 私有候选 v2：recover-proved-compound-statement-conditions

本目录是供 root 串行审阅/集成的冻结候选。它基于本 change 的两个文件 SHA-256 pin，项目工作区没有被改写；没有运行 Cargo、rustfmt、JDK、CLI 或编译器。

`region.rs.candidate` 增加窄的 `CompoundIf` 证明载体和一个有界 compound-statement-condition proof。该 proof 复用普通 branch/frame/canonical facts，不改既有 `If`、`TwoExitReturn`、`ShortCircuitValue` proof；只接受至多 24 个同路径比较测试、两个且仅两个出口、精确 join 与一个 payload、Normal canonical rows、完整测试节点 successor 身份、每个非根测试的精确 incoming closure、无循环/边界/其他所有权冲突。条件由 successor 关系折叠，不用 BCI 排序推导拓扑；有界 exit alignment 必须令每个物理 test 恰好出现一次，且最终条件没有出口常量。

通过后沿已有 `region_at` 解析 payload。完整 canonical edge pass 把测试/arm 入边、payload 入边和 join 入边分类闭合；join 允许候选扫描开始前已由外层 Region 访问的来源，这覆盖 Switch2 的外层 BCI 78 到 join 136，同时拒绝任何既不是本 proof 也不是外层已拥有的来源。提交前失败恢复 Walker 所有权状态；Budget/取消 Stop 传播而不发布候选。

条件表达式复用 `test_expr(..., false)`、`short_circuit_not` 和 `logical_expression`。根测试的 `test_effects` 仍只发出一次；block 98 预先计算并保存的 float `scale` 保持普通 lead 赋值，条件只读它，不吞进首个比较、不再次求值。浮点取反沿用现有 not 逻辑，不用互补比较改写 NaN 行为。builder 其余 arm/path/guard/cleanup/assignment 消费者对 CompoundIf 两臂做显式递归，专项 `If`-only proof 仍拒绝未支持的 CompoundIf。

增量由四部分构成：Region 数据/结构合同；唯一有界 canonical DAG 证明与提交闭包；将树构造成既有 Expr/Stmt 的 builder 代码；真实 Switch2 回归及预算/取消/边界断言。没有新增通用 DAG 框架或公共分析入口。逐文件 patch 标签使用真实路径 `crates/jarde-java/src/region.rs` 和 `crates/jarde-java/src/build.rs`。

唯一新增测试名是 `real_switch2_direct_target_join_proves_candidate_witnesses`（`region.rs.candidate`）。它 pin 1123-byte class 的 BLAKE3，核 canonical edges 和 join 164 的 incoming Normal sources；直接调用实际 Walker proof，核 `{98,111,118,124}`、payload 131、join 136；用成功 proof 的实际步数构造 `total-1` 预算，验证完整扫描末尾 Stop；也核零步预算、预取消、frame-boundary 和错误 join 拒绝，并断言直接 proof 不改 `visited`。完整 `recover` 必须 structured、fallback-free、claimed 全覆盖，输出唯一 CompoundIf，条件为 `((98&&!111)||(118&&124))`，then arm 独占 131、join 136 外置一次；还核实物理 BCI 136/163 的 Return，以及 Switch join 164 外置一次。

这些均是候选源码断言，尚未运行。Rust 编译与格式、全套已有测试、Builder 对真实 class 的最终语句/source map、float 副作用和 NaN 运行时对照仍待 root 工具链/runtime fixture 验收。编译器才能确认所有 exhaustive consumer 和借用边界。
