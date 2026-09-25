## ADDED Requirements

### Requirement: Invocation arguments preserve proved array widening

当实参的已呈现数组类型与被调用方法参数 descriptor 共同证明一个无需运行时检查的数组上溯时，系统 SHALL 恢复该调用并保留被调用 Methodref 的静态参数类型。

#### Scenario: Array components widen to Object components

- **WHEN** `int[][]` 传给 `Object[]`、`String[]` 传给 `Object[]`，或 `String[][]` 传给 `Object[][]`
- **THEN** 完整恢复源码 SHALL 可重编译，实际调用目标与原 class 相同，参数仍为同一数组对象

#### Scenario: Array marker-interface supertypes

- **WHEN** `int[][]` 传给 `Cloneable[]`、`String[][]` 传给 `Serializable[]`，或 `int[]` 传给 `Cloneable`
- **THEN** 完整恢复源码 SHALL 可重编译并与原 class 执行一致，证明只来自 Java 内置数组超型，参数仍为同一数组对象

#### Scenario: A more specific overload is present

- **WHEN** 同类同时提供 `overload(Object[])` 和 `overload(String[])`，字节码通过无需 `checkcast` 的 `String[]` 值调用前者
- **THEN** 恢复文本 SHALL 明确保留 `Object[]` 参数的静态类型，运行结果 SHALL 仍由前者产生，不得只靠文本实参的更具体类型重新决议

#### Scenario: Primitive array element is not a reference element

- **WHEN** 数组上溯判据检查 `int[]` 与 `Object[]`/`Cloneable[]`，或调用关系需要未知外部继承证据
- **THEN** 判据 SHALL 不把原始元素当作引用元素；合法 class 中若缺少可证关系，系统 SHALL 保留包含实参生产者与调用指令真实 BCI 的拒绝，不从目标 descriptor 发明一个可能失败的引用 cast；现有 `Object`、同型、null 路径 SHALL 保持原合同

### Requirement: Array argument typing preserves effects and bounds

对已证明数组上溯的调用恢复 SHALL 沿现有一次求值、来源与受限执行机制。

#### Scenario: Argument expression is evaluated once

- **WHEN** 协变实参由一个可能抛错或有副作用的表达式产生
- **THEN** 恢复的目标类型表达式 SHALL 不复制、提前或跳过它，也不改变其异常和运行时数组类型

#### Scenario: Evidence and stop behavior remain stable

- **WHEN** 默认/完整证据或重放呈现同一调用，或数组证明过程遇到预算/取消限制
- **THEN** 成功时正文 SHALL 相同，原实参与调用 BCI SHALL 可追溯；无法完成时 SHALL 沿既有停止通道结束，不提交未经证明的调用
