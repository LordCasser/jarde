## ADDED Requirements

### Requirement: 经证明的 switch 续接

Java 8 反编译恢复 MUST 仅在控制流边和物理 block 归属足以证明续接及其结果路径时输出 switch 续接。证明无法闭合时 MUST 保守拒绝。输出 MUST 保留 block 的唯一归属、终端行为、源码来源位置，以及请求实际产生的预算/Stop 结果。

#### Scenario: 已证明的 switch 后接外层直线路径

- **WHEN** 一个外层控制结构 arm 中的 switch 已完整证明，其内部汇合点之后有一段非空直线路径，并到达外层 arm 的边界
- **AND** 完整控制流边及 block 归属证明汇合点和后续路径没有外部入口、非普通控制流边、重叠或未归属 block
- **THEN** recovery 在外层 arm 中先输出 switch 再输出后续路径，该路径只出现一次，内部汇合点不属于任何 case body，且每条 statement 保留物理源码来源位置
- **AND** switch break、fallthrough 和外层 branch 保留已有的准确 target 与最近作用域约束

#### Scenario: 外层直线续接候选不属于有界直线路径

- **WHEN** 本片外层直线续接候选在 switch 汇合点之后发生分支、循环、进入另一 switch、越过外层边界、存在外部 predecessor、包含非普通控制流边，或 walk 新访问的 block 集合与物理 region 不相符
- **THEN** recovery 拒绝外层结构，且不重复、丢弃或部分消费 blocks

#### Scenario: 直接 case target 是有限无环路径的唯一共享续接

- **WHEN** 多条 switch arm 路径汇合到同一个直接 case target，且该 target 是唯一满足续接与归属条件的直接 case target；每条路径均有限且无环，最终到达该 block 或物理 Return/Throw terminal
- **AND** 至少一条 arm 路径到达该共享 block；其完整控制流入边均为互异的普通边，来源恰为 switch dispatch 和已证明的 arm 路径；并且至少一条 arm 路径以物理 Return/Throw terminal 结束
- **AND** 内部 branch 均为已证明的比较分支，arm 路径各自闭合且彼此不重叠，共享 block 的入边只来自 dispatch 和已证明的 arm 路径
- **THEN** recovery 可以将该 block 作为 switch 的汇合点，terminal return/throw 保留在原 arm 路径中，共享续接在所有 arms 外只输出一次

#### Scenario: 共享续接的归属存在歧义或不完整

- **WHEN** 不存在满足完整路径条件的唯一共享续接 block，或 arm 包含 cycle、nested switch、unknown terminal、非普通控制流边、未证明的 case-entry crossing、外部 incoming edge，或非汇合点 block 的归属重叠
- **THEN** switch 按既有拒绝/coverage 契约保守拒绝；recovery MUST NOT 输出归属不确定的共享续接

#### Scenario: 预算或取消导致续接处理停止

- **WHEN** 现有请求预算或 cancellation 在 node、edge、predecessor、candidate 或后续路径处理时停止工作
- **THEN** recovery 在该工作实际停止的位置返回既有 Stop outcome（汇合点发现使用 dispatch 位置，后续路径处理使用实际 `next`/continuation 位置），保留真实 usage、Stop reason 和 dimension，并且不发布部分 text 或 source map
