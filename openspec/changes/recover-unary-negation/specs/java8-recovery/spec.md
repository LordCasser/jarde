## ADDED Requirements

### Requirement: Numeric negation preserves its operand and Java grouping

对于操作数及外围消费位置可恢复的 `ineg/lneg/fneg/dneg`，恢复结果 SHALL 用 Java 一元 `-` 呈现该值。文本 MUST 保持字节码的操作数求值次数与位置，并保留来源映射。数值窄类型的取负遵循 Java 一元数值提升。取负 MUST NOT 改写成零减操作数或递减。操作数不能恢复时，结果 MUST 明示缺口，且 MUST NOT 静默丢失该操作数的调用生产者。

#### Scenario: Four numeric kinds can be negated

- **WHEN** 方法分别返回参数的 int、long、float、double 取负
- **THEN** 四个方法 SHALL 输出可编译的 `return -参数;`，不因取负指令留下字节码缺口

#### Scenario: Nested negation does not become decrement

- **WHEN** 两条取负指令相继作用于同一个数值，或取负作用于求和结果
- **THEN** 文本 SHALL 保持两次取负和求和分组，如 `-(-x)`、`-(x + y)`，且 MUST NOT 产生 `--x` 或 `-x + y`

#### Scenario: Runtime boundaries and evaluation count remain equivalent

- **WHEN** 自写样例输入包含整数最小值、浮点正负零、无穷、NaN，以及一次有可观测计数或抛异常的调用
- **THEN** 恢复文本重编译执行 SHALL 与原 class 的整数结果、非 NaN 浮点位、NaN 分类、调用次数与异常结果一致；该证据 SHALL 只陈述这些受控样例，不泛化为全面语义等价

#### Scenario: An unrecovered operand keeps the gap and its producer

- **WHEN** 取负的操作数来自当前不能呈现的指令，且其输入由调用产生
- **THEN** 结果 SHALL 保留有来源的字节码缺口或生产者语句，MUST NOT 因把取负认作已消费的值而丢掉该调用
