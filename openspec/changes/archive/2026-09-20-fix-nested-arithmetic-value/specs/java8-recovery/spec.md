## MODIFIED Requirements

### Requirement: Evidence-gated Java 8 recovery

系统 SHALL 仅在对应 IR、descriptor、异常/effect 和编译器模式前置条件满足时启用 Structured recovery；证据不足时 MUST 返回 representation=Mixed/Bytecode、quality=Conservative/Fallback，并说明缺失条件。值的正确性 SHALL 按生成代码实际求值的位置与顺序验证，嵌套表达式不能以其原始生产位置代替当前求值位置；当一个算术的结果作为另一个算术的操作数时，呈现 MUST 使用该操作数自己的值并保留这一嵌套，MUST NOT 省略外层运算、也不得把某个操作数改写成另一种运算（例如把 `i * (2 - d * i)` 写成 `i * 2 - d * i`）。**该嵌套也 MUST 体现在输出文本的分组上**：表达式树到文本的转换 MUST 按运算优先级与结合性补足括号（或写成不依赖默认结合的形式），使文本解析回同一棵树；仅凭「树是对的」不满足本要求，因为调用方收到的是文本。层不能证明某操作数的值或求值点时 MUST 拒绝该区域而不是呈现，并保留被拒区域的 bytecode 与 origin；quality、content、execution 等结构性平面 MUST NOT 代替这一证据。fallback MUST 保留被省略表达式所依赖的可观察生产者及其物理 origin，包括字段读取、类初始化与可能抛异常的操作。

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

#### Scenario: An arithmetic operand is the result of another arithmetic

- **WHEN** 字节码把一个算术的结果作为另一个算术的操作数，例如 `iload_1; iconst_2; iload_0; iload_1; imul; isub; imul; istore_1`（源码形状 `i = i * (2 - d * i)`）
- **THEN** 呈现 MUST 使用该操作数自己的值并保留嵌套，输出 `local1 * (2 - arg0 * local1)`；MUST NOT 省略外层 `imul` 或把 `isub` 的常量左操作数写成乘法（`local1 * 2 - arg0 * local1`），因为那会计算另一个值

#### Scenario: A presented body computes what its bytecode computes

- **WHEN** 受控 fixture 以一组输入（至少包含 `d = -1` 与一个正向输入）分别执行原 class 与呈现文本
- **THEN** 每个输入的返回值/可观察行为 MUST 与原 class 相同；`d = -1` 时原 class 返回 `-1`，呈现文本不得返回 `-81`。呈现被拒绝时改由拒绝边界验收，不给「未生成 Java」留通过路径

#### Scenario: An unproven arithmetic operand is refused

- **WHEN** 层无法证明某个算术操作数的值或它实际被求值的位置
- **THEN** 该区域 MUST 拒绝并保留 bytecode 与 origin（representation=Mixed、quality=Fallback、拒绝诊断点名相关 BCI），MUST NOT 省略该操作数或换一个定义后继续呈现；拒绝产物按既有 refusal 契约可定位

#### Scenario: The structural planes do not stand in for this

- **WHEN** 一份产物报告 quality=Structured、content=contains_statements、execution=complete 且没有诊断
- **THEN** 这些平面 MUST NOT 被当作值正确性的证据；本 requirement 的证据只能是受控执行对照，或对相关操作数值与求值点的证明。已知反例表明同时满足上述三个平面的产物仍可计算与字节码不同的值

#### Scenario: Grouping survives printing

- **WHEN** 表达式树把一个优先级更低或结合方向不同的子表达式放在某个运算的操作数位置，例如 `a * (2 - b * a)`、`a - (b - c)`、`a / (b * c)`、`a - (b + 1) * 2`
- **THEN** 呈现文本 MUST 保持该树的分组：打印时按运算优先级与结合性补足括号（或等价地写成不依赖默认结合的方式），使文本按 Java 语法解析回同一棵树；MUST NOT 依赖默认优先级与结合性直接拼接子表达式。四个形状中的每一个都必须解析回原树，且以受控执行对照验收（`a * (2 - b * a)` 已实测被打印成 `arg0 * 2 - arg1 * arg0`，其余三个同样丢失分组）
