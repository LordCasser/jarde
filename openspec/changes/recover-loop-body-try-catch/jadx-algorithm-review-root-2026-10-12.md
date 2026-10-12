# 循环体 try/catch：JADX 算法与现有认证的差异

这里只记录 root 对本地代码的审阅。没有执行新基线、修改产品或认定当前缺陷；不能据此关闭语法单元。原始图、异常表、物理来源和预算约束仍是实现边界。

## 已读的 JADX 实现

本地参考仓库为 `/Users/lordcasser/workspace/testzone/jadx`。`BlockExceptionHandler.wrapBlocksWithTryCatch` 从保护块寻找 top/bottom，在入口添加或复用 synthetic top splitter；有合适出口时建立 bottom splitter，连接 handler。`searchBottomBlock` 先求保护块内部共同 post-dominator，再考虑外部 path cross；没有共同出口时允许 bottom 为 null。源码明确区分 exit nodes 的数据流需求。它还修正辅助出口插入后造成的假回边，避免把异常归约产生的跨边误判成循环。

`ProcessTryCatchRegions` 随后按 top splitter 寻找 enclosing region。遇到 `LoopRegion` 时进入 body，遇到 branch region 时提升到 parent；收集能从 dominator 到达、但不能从 handler 到达的子容器作为 try 区。此阶段是在已改造过的图上包区域，不能直接将它的图修改搬进 Jarde 的物理事实层。

参考测试 `loops/TestTryCatchInLoop.java` 是无限重试循环：try 中 `exc()` 后 break，catch 后再次迭代；check 断言调用计数为 3，生成源码断言 catch 和 break 各一次。它需要保留为代表场景，不能用有限 for 的成功替代它的验收。

## Jarde 的现有条件

`region::proved_loop_catch_joins` 证明 exception-only handler 的唯一普通后继，并核对完整声明行和 canonical 行。保护区所有重叠块及实际 throw-site 块都必须位于自然环且被 header 支配；每个 throw-site 的普通路径还必须到同一 join。

`plain_paths_reach_join` 对到达 join 前的无普通后继节点和循环直接返回 false。`normal_flow::natural_loops` 从 latch 沿普通前驱逆行计算成员，止于 header。因此只沿异常边回到 handler/join 的终止 throw 分支，可能同时触发环成员与普通路径两道拒绝。这是由代码导出的候选原因，尚未用当前 DF/DV 的同次 IR 证明。

## 实现前必须验证的决定

先复跑当前 CLI 的 debug/none、Java8/23 DF/DV 整类对照，再对 DV.plainFor 的完整异常表、header、handler、join、保护块和普通/异常边逐项验证 Q-i/Q-ii。foreach 的 protected-local SSA 拒绝单独验证 Q-iii，不能合并归因。

若确认为合法的 body-only catch 终止分支缺少所有权证明，优先在现有 catch 认证内补齐有界条件。必须证明保护行、header 支配关系、handler 入口和区域所有权；终止分支不能借许可跨 header、借其他异常行或吞掉真实出口。是否要支持正常 return/break 等出口，由具体场景及已有 emitter 能力决定，不能把“任何 terminal 都允许”当修复。

参考 JADX 的价值是区分异常区域所有权与普通循环回边，并处理辅助图带来的假循环。Jarde 应保留原 CFG 和来源，仅把已证明的归约事实交给现有区域恢复；当前没有证据要求新增通用 pass、第二图服务或修改 SSA。任何实现仍需停止传播、逐维度计费和整类运行对照。
