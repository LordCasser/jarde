## ADDED Requirements

### Requirement: A proved short-circuit Boolean value can be returned directly

当 Java 8 方法的闭合、无环短路测试图将两个精确的 `1/0` 生产者汇入唯一栈 Phi，并由返回类型为 `Z` 的方法中的唯一 `ireturn` 消费时，恢复结果 SHALL 用 Java 8 的惰性条件表达式输出一个返回语句，且每条路径的返回值、RHS 调用次数及顺序与原 class 相同。若测试边、入口、生产者、Phi、返回描述符、效果或预算中的任一证明不足，恢复 MUST 整体引用该图并保留全部已解码指令来源。

#### Scenario: Mixed AND then OR returns a Boolean

- **WHEN** `return (a && rhsB()) || rhsC()` 编译出 BCI 1/7/13 三测试、BCI 16/20 的 `1/0` 生产者和 BCI 21 的唯一 `ireturn`
- **THEN** 完整恢复类 SHALL 用 Java 8 重编，八组 `a`、`bValue`、`cValue` 的返回值及 `bCalls`/`cCalls` SHALL 与原 class 相同；`a=false` 不调用 `rhsB()`，`a=true,bValue=true` 不调用 `rhsC()`

#### Scenario: Consumer type or graph closure is not proved

- **WHEN** `ireturn` 所在方法不声明 `Z`、Phi 有第二消费者、测试/producer 有候选外前驱或异常边、测试携带不能折入表达式的独立效果、或来源/预算停止
- **THEN** 恢复 MUST 不输出猜测的布尔 `return`，且 SHALL 以现有整体回退保留测试、两个 producer、消费与后缀的真实来源；已证明的字段写入短路正例不受影响
