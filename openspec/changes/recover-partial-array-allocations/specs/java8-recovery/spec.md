## ADDED Requirements

### Requirement: Partial-dimensional array creation preserves total and allocated ranks

当合法数组创建指令及常量池描述符证明结果数组的基本元素、总维数与已分配前缀维数时，系统 SHALL 将二者分别保留并呈现为可编译的 Java 数组创建表达式。

#### Scenario: `anewarray` allocates one dimension of an array component

- **WHEN** `anewarray` 的组件类是 `[I` 或 `[Ljava/lang/String;` 等合法数组 descriptor
- **THEN** 系统 SHALL 输出对应的 `new int[n][]` 或 `new java.lang.String[n][]` 形式，并将结果证明为比组件多一维的数组

#### Scenario: `multianewarray` allocates a prefix

- **WHEN** 常量池数组 descriptor 的总维数大于指令的 `dimensions` 立即数
- **THEN** 系统 SHALL 为每个已分配维度写出一个带表达式的括号，按差值追加空括号，并保留完整结果数组类型

#### Scenario: Fully allocated arrays remain stable

- **WHEN** `newarray`、普通类组件 `anewarray` 或总维数与分配维数相等的 `multianewarray` 已可恢复
- **THEN** 系统 SHALL 维持原表达式、类型及执行结果，不加入多余空括号

### Requirement: Array dimensions preserve evaluation and evidence

部分维度分配 SHALL 沿既有一次求值、异常、来源和受限执行契约恢复。

#### Scenario: Dimension expressions evaluate left to right

- **WHEN** 两个已分配维度调用可计数或可能抛错的函数，尺寸可能为负或零
- **THEN** 恢复结果 SHALL 与原 class 的调用次数、顺序、异常类别和身份及数组形状一致；未分配维度 SHALL 不被求值或实例化

#### Scenario: Consumers read the complete result type

- **WHEN** 部分维度创建值流向局部、字段、`.length` 或后续数组访问
- **THEN** 已证明的总维数 SHALL 用于类型与源码呈现，不因只分配前缀而降低结果类型 rank

#### Scenario: Invalid or unprovable rank is refused

- **WHEN** descriptor/立即数不足以证明 `1 ≤ allocated ≤ total ≤ 255`，或基本元素无法合法呈现，或尺寸生产者不能按既有机制安全呈现
- **THEN** 系统 SHALL 保留来源完整的拒绝；预算或取消耗尽时 SHALL 沿既有停止通道结束，不输出正常但语义改变的正文
