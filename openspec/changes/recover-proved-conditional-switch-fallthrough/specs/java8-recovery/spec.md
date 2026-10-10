## ADDED Requirements

### Requirement: Proved conditional switch fallthrough

系统 MUST 对有限无环条件 case 恢复准确 fallthrough，只有所有路径到已证公共出口、唯一相邻 case 或准确物理 return/throw 终点才准入。完整控制流的边、上下文身份和入边所有权 MUST 支持该证明；异常、子程序调用/返回、外来入口、循环、多个或非相邻目标及未知出口 MUST 保持已有拒绝。证明 MUST NOT 通过提前消费 case 正文改变访问状态。

#### Scenario: 部分路径进入下一 case
- **WHEN** 当前 case 一些路径进入唯一相邻标签、其他路径到已证公共出口
- **THEN** 生成条件结构和准确 break/fallthrough，后续 case 的代码只归属后续标签一次
- **AND** 公共出口留在 switch 之后，不能将重复访问当成成功恢复的循环

#### Scenario: 条件路径都结束在公共出口
- **WHEN** case 内存在分支且所有继续路径到公共出口
- **THEN** 恢复无 fallthrough 的 case，不因分支数大于一拒绝

#### Scenario: 无法闭合证明
- **WHEN** case 图包含循环、不同 clone context、外部入边、异常或子程序边、多个 case 目标、非相邻目标或未知出口
- **THEN** 不发布未证 case 结构，不把投影隐藏的边当成正常闭合路径

#### Scenario: 准确物理终止路径
- **WHEN** 部分路径结束于准确解码的 return 或 throw 且完整图中该块无 outgoing edge
- **THEN** 保留该独立终止路径，不把它当作 case transfer 或公共出口
- **AND** 子程序 canonical return 边不充当这种物理终止证据

### Requirement: Conditional fallthrough source integrity

系统 MUST 保留共享标签分组、既有直线 fallthrough、准确源码来源与原子 Stop 行为；default/all 配置恢复正文和 map MUST 一致。完整原始 class 恢复后 MUST 可原样重编并保持原运行行为，不能用移除 check 方法、Inner 或替身 SDK 换取通过。

#### Scenario: 原上游完整条件 fallthrough class
- **WHEN** 恢复 CF12 原始 TestSwitchWithFallThroughCase 完整类
- **THEN** 原/JADX/Jarde 全类使用相同合法 SDK 重编运行结果相同，所有实际物理 BCI 有准确来源
- **AND** 117 只归属 case2，171 留在公共出口，无该 switch 的假循环

#### Scenario: 既有控制保持
- **WHEN** 输入直线 fallthrough 或多个标签共用一个 target
- **THEN** 保持准确标签顺序、分组、来源和运行行为

#### Scenario: 证明被预算或取消停止
- **WHEN** 检查实际节点或边时预算耗尽或收到取消
- **THEN** 返回准确 Stop，不发布部分正文或 map

#### Scenario: Break 目标属于准确当前 switch
- **WHEN** 条件路径跳出 switch 且能证明它是当前最近的可 break 作用域
- **THEN** 在准确条件分支呈现 break，来源属于该真实 transfer 或 branch
- **AND** 若退出需要跨更内层 loop/switch 的 label 而当前呈现未证明它，保持明确拒绝，不把无label break绑定到错误作用域
