## ADDED Requirements

### Requirement: A closed mixed-polarity Boolean decision graph preserves its field value

Java 8 方法把 `&&` 与 `||` 混合形成的无环条件图汇入两个 1/0 生产者，再由唯一栈 Phi 写入一个静态 `Z` 字段时，若每个比较测试、正常边、异常边界、SSA 值与字段消费均可证明，恢复结果 SHALL 输出可用 Java 8 重编的结构化表达式，并在每条路径保持字段值及各 RHS 调用次数。若任何入口、效果、值或消费不能证明，MUST 原子拒绝整张局部图并完整保留来源，不得将共享 join 误称循环或发布部分字段写入。

#### Scenario: AND followed by OR

- **WHEN** Java 8 编译 `result = (a && b()) || c()`，BCI 1/7 的不同出口汇入 BCI 10 的 `c()` 测试，BCI 16/20 是 1/0 producer，BCI 21 唯一写字段
- **THEN** 原 class 与恢复完整类在所有八组 `a`、`b()`、`c()` 值组合下 SHALL 有相同 `result`、`bCalls` 和 `cCalls`；`a=false` MUST 不调用 `b()`，`a=true,b=true` MUST 不调用 `c()`

#### Scenario: OR followed by AND

- **WHEN** Java 8 编译 `result = (a || b()) && c()`，BCI 1/7 的正常出口汇入 BCI 10 的 `c()` 测试，其余 producer/consumer 同样唯一
- **THEN** 八组值/调用路径 SHALL 与原 class 一致；`a=true` MUST 不调用 `b()`，`a=false,b=false` MUST 不调用 `c()`

#### Scenario: An additional entry or use breaks graph closure

- **WHEN** 一个 transfer-only 路径从候选外进入共享 producer、任意测试/producer 有额外真实前驱或异常边、Phi 有第二消费者、字段身份不明、或递归展开超出预算
- **THEN** 恢复 MUST 不输出该字段的伪结构化赋值；拒绝结果 SHALL 保留参与测试、两个 producer、消费与后缀的完整字节码来源，不得丢失该外部入口
