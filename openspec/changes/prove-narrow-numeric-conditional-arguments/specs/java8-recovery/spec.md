## ADDED Requirements

### Requirement: 已证窄数值条件值保留被选中的调用重载

当一个 Java 8 条件值作为 `byte` 或 `short` 方法形参使用，且条件两臂分别可由物理类型来源或范围内整数常量证明为该窄类型时，系统 SHALL 恢复可以 Java 8 重编的条件表达式，MUST 保留原 class 选择的调用目标、条件求值顺序与字段/常量值。证明不能闭合时 MUST 保留 bytecode、origin 和拒绝原因，MUST NOT 仅凭 JVM 的 int 栈表示将任意值强转成窄类型。

#### Scenario: byte 和 short 条件实参的两种来源

- **WHEN** `ConversionCases` 中 `byteField`/`shortField` 的一臂为准确类型字段、另一臂为范围内 int 常量，且 `castShort`/`shortConstant` 的两臂为范围内常量，消费者准确为 `write(B)` 或 `write(S)`
- **THEN** Jarde SHALL 恢复一次条件求值和原调用重载；原 class、固定 JADX、Jarde 的完整 Java 8 类源码 SHALL 重编并以 `java -Xverify:all` 运行 20 行一致

#### Scenario: 无需窄化的数值条件返回保持原行为

- **WHEN** 原方法是已恢复的 int、long、byte、float 或 double 简单条件返回
- **THEN** 系统 MUST 保留完整 Java 8 源码的类型、值和分支行为，不得把 1/0 的数值返回误写成 boolean

#### Scenario: 不能证明的窄化不得伪装为源码

- **WHEN** 条件臂中的 int 值超出目标 byte/short 范围、字段 descriptor 不匹配、目标调用签名或唯一消费者不明，或存在额外使用、异常边、预算/取消停止
- **THEN** 系统 MUST 原子拒绝受影响的方法并保留可定位来源；MUST NOT 插入可能改变值或重载选择的 cast，也不得发布缺少返回却声称完整的源码
