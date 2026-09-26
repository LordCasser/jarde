# 2.3a Root 验收：资源体内保存值的返回尾部

提交 `05e96409`。新增冻结 Java 8 样例 `tests/fixtures/p3-multi-resource-twr/TwrReturnTail.java`（SHA-256 `27036818d89904e921949753551d45678f9d711eb731c3a9058eeaef12a617ad`）及 class（SHA-256 `7c0f0f2d6b3bd27ed7559a9ab888fbf14afac3f736cf3d15cc6c3bd1c518f9e6`）。

`guard::twr_return_tail` 仅认终端正常 CFG 块中的两条 `load; return`，要求 load 的 SSA 局部值来自资源体内唯一 store，且 store 的输入由体内指令产生；额外效果或不同读取值不纳入 TWR。Plan 把真实 return BCI 交给已有 Builder 返回表达式路径，保留 load/store/return 来源。Root 审读后撤回了按物理 slot 全局跳过声明的尝试，未放宽词法作用域拒绝。

Root 在隔离工作树从源码重建并复跑：`jarde-java` 的计划正例 1/1；`p3_twr_return_tail` 4/4、`p3_multi_resource_twr_geometry` 4/4、`p3_guard` 13/13。`cargo fmt --all -- --check`、`git diff --check` 通过。私有构建目标已清理。

本项仅证明返回尾部的 Region/AST 归属，不声称完整方法恢复。`runSaved` 的已证清理 handler 在 return 后，尚未进入 Guard `owned`，被 Region 引用为 fallback；同一 slot 的 synthetic Throwable 访问又与体内 int 混入声明规划，仍拒绝。冻结双资源 `run` 因 BCI 0/10 的 `new; dup` 头部生产者未归属仍报 `jre_guard_span`。分别留给 2.3b 与 2.4，三方重编执行验收仍是 3.1，不能把上述局部测试算作该任务通过。
