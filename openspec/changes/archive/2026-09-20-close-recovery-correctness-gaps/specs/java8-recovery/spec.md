## MODIFIED Requirements

### Requirement: Evidence-gated Java 8 recovery

系统 SHALL 仅在对应 IR、descriptor、异常/effect 和编译器模式前置条件满足时启用 Structured recovery；证据不足时 MUST 返回 representation=Mixed/Bytecode、quality=Conservative/Fallback，并说明缺失条件。值的正确性 SHALL 按生成代码实际求值的位置与顺序验证，嵌套表达式不能以其原始生产位置代替当前求值位置；fallback MUST 保留被省略表达式所依赖的可观察生产者及其物理 origin，包括字段读取、类初始化与可能抛异常的操作。

#### Scenario: Java 8 lambda and method reference

- **WHEN** invokedynamic 符合已验证的 LambdaMetafactory 形态且实现 handle、SAM descriptor 和 capture 可追溯
- **THEN** 输出可读 lambda/method reference，并保留 bootstrap/use-site/origin evidence；任意 bootstrap 不得套用该模式（验收 A04）

#### Scenario: String concatenation order

- **WHEN** IR 证明 StringBuilder/StringBuffer 拼接模式及每个转换的求值顺序
- **THEN** 输出拼接表达式或等价结构，不重复调用、移动可能抛异常的操作或丢失转换语义

#### Scenario: Generic output without a compiler-specific match

- **WHEN** 方法已满足支持范围内的普通控制流、类型与 effect 前提，但没有命中编译器语法糖规则
- **THEN** 仍能生成带稳定名称与 source map 的通用 Java 表达；不得仅因没有语法糖匹配就放弃已证明的基础结构或声称恢复了原始源码写法

#### Scenario: Loop header has observable effects

- **WHEN** 可恢复循环的条件或 header 包含调用、读取或可能抛异常的操作
- **THEN** 输出保持这些操作在每条正常/异常路径上的执行次数与次序；不能把每轮执行的操作移到循环外，证明不足时保留可靠表示与降级原因

#### Scenario: A loaded value survives a later local write

- **WHEN** 一个值经 load 留在 operand stack 上，原 local 随后被 iinc/store 覆盖，旧值之后才被 return、调用或条件消费
- **THEN** 恢复 SHALL 使用 load 时的 SSA 值，不重新读取已经改变的 slot；例如 `return x++` 必须返回递增前的值，不能因结构可识别而标记语义不同的 Java 为 Structured

#### Scenario: A consumer falls back after reading a call result

- **WHEN** 调用结果的消费者属于未证明的 cast/field/其他形态，消费者最终不能生成表达式
- **THEN** 生产者调用及其求值顺序 SHALL 保留为可靠语句或完整 fallback 范围与 origin；不能先省略生产者，再仅引用消费者 BCI，不能因 quality=Fallback 就丢失 effect

#### Scenario: A nested expression is evaluated after a local write

- **WHEN** 一个局部变量先参与子表达式求值，其后被写入，而该子表达式的结果更晚才被消费，例如 `(x + 1) + ++x`
- **THEN** 输出 SHALL 保留先前子表达式的值；输入 7 时若生成可执行 Java，结果必须为 16。无法证明时 MUST 明确降级并保留相关生产者/消费处证据，不能生成返回 17 的 Java/Structured，也不能因子表达式原始 BCI 位于写入前而放行

#### Scenario: A refused cast depends on a static field read

- **WHEN** `getstatic External.value; checkcast String; areturn` 的 cast 不能可靠呈现，字段读取可能触发类初始化或异常
- **THEN** 输出 SHALL 保留字段读取的语句或完整 fallback 引用，并为读取 BCI、消费 BCI 和物理方法提供 source map；仅保留 cast/return 或独立的字段识别记录不满足产物的 effect 与来源完整性。无需执行类初始化或读取无关 Body 来保留此证据

