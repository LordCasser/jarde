## Context

`ChainExtraBoundary.assign(ZZZZ)V` 的 BCI 1/5/12/16/22 测试最终汇到 25/29 的 1/0 producer，BCI 30 唯一写 `result:Z`。`extra=true` 先执行 BCI 8 `goto 25`；该块没有比较或值生产。当前 `Region::short_circuit_value` 遇单正常后继只承认首指令为 `Push` 的 producer，且精确前驱验证要求 producer 直接由测试出口到达，因此不能认领完整图。重叠所有权检查把非闭合候选原子引用，避免了错误结构，但不恢复语义。

## Goals / Non-Goals

**Goals:** 只恢复同一有界无环图中可证明的纯前向转接，保留每个物理块的 owner/source map，输出一个字段赋值，原/Jarde 完整类 32 路径值和 RHS 次数逐字一致；额外入口、异常边、独立效果与预算失败完整拒绝。

**Non-Goals:** 任意控制流化简、跨异常区转接、loop/backedge、多个值 producer、通用三元 AST 识别、对 JADX 错误文本逐字追平。调用实参/返回消费者已按自身 OpenSpec 处理，本 change 不放宽它们的准入。

## Decisions

1. **在现有图内给转接一个显式私有角色。** 只有 canonical 节点真实指令均为已解码、无效果、不可抛的直接前向控制转移，且恰有一个正常入口和一个正常出口、没有异常或 call 边时，才列为 gateway。物理前驱/后继和 decoded target 都要一致；任何多入口或回边拒绝。gateway 计入图节点、owner、预算和 source map，但不是 1/0 producer。
2. **逻辑折叠与物理归属分开校验。** 图搜索可沿 gateway 到最终 producer，把测试边的逻辑目标视为 producer；随后仍用原始 canonical 边核对每个内部测试、gateway、producer 的精确真实前驱。不得用 BCI 区间、dominance frontier 相等或地址先后代替边身份。整图证明通过前不标记 `visited`。
3. **保留唯一 Phi/字段消费合同。** 两个 producer 仍必须真实生产 1/0 并只进入同一消费块；SSA 的唯一同槽 Phi 只能被一个 `putstatic Z` 使用。测试的 taken/fallthrough 极性、独立效果和惰性顺序按既有字段图 proof 检查。递归 `Conditional` 可在互斥臂复制共享子图，但须按节点、深度、展开与输出预算预检，任何中止不发布部分赋值。
4. **JADX 算法只作线索。** 本地 JADX `IfRegionMaker` 的 ternary 合并与 AND/OR 合并可提示需跳过空转接块，但冻结类的可编译输出有 16/32 行错误。验收以原 JVM 的 32 路径值/调用轨迹为准，并给多入口、转接块带效果/异常边及第二消费构造拒绝对照。

## Risks / Trade-offs

- [折叠掉转接后误收外部前驱] → 逻辑图和物理图分别核对，gateway 只有已证明的一条入口，producer 前驱精确枚举。
- [漏掉 `goto` 来源] → gateway 仍在 Region owner 集合，字段语句 source map 要包含 BCI 8 及完整 16 个已解码起点。
- [递归条件复制导致 RHS 多执行] → 互斥路径证明加全 32 组 JVM 值与 `calls` 对照；成本超限整体拒绝。
