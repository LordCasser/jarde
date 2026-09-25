## ADDED Requirements

### Requirement: Proven two-arm stack values recover as one conditional expression

当现有 `if` 区域与 SSA 共同证明一个栈汇合值仅来自 true/false 两臂、没有额外入口，并在一个已支持位置被消费时，系统 SHALL 将测试与两臂值恢复为 Java 条件表达式。

#### Scenario: Return, local, arithmetic and argument consumers

- **WHEN** 两臂分别产生同型且可呈现的值，汇合值随后被 return、局部写入、算术或调用参数消费
- **THEN** 系统 SHALL 在该消费位置输出可编译的条件值，并让两臂中只有实际选中者求值一次

#### Scenario: Reference, null and overload target

- **WHEN** `null` 与已证明的引用类型构成条件值，或条件值成为有重载的调用参数
- **THEN** 条件值及必要的现有目标 cast SHALL 保持字节码 Methodref 的静态选择，不允许重编译后选到另一重载

#### Scenario: A proved boolean branch value initializes a field

- **WHEN** 两臂各自产生可按目标 `Z` 字段上下文证明的布尔值（如 `0`/`1`），汇合值由同一方法的真实 `putstatic`/`putfield` 唯一消费
- **THEN** 系统 SHALL 在该字段写入位置表达条件值，并保持声明类型、final 赋值规则、测试与选中臂的求值顺序；MUST NOT 将任意整数 Phi 当成布尔值

#### Scenario: Non-0/1 Phi inputs stored to a boolean field

- **WHEN** 验证器接受的 class 在同一双臂汇合后向 `Z` 字段写入整数 `2` 或 `3`，原类的 `-ea`/`-da` 输出与使用 `0`/`1` 的 class 相反
- **THEN** 系统 MUST NOT 仅因目标 descriptor 是 `Z` 就将两臂拼写为布尔字面量；若无法证明精确等价的 Java 表达式，SHALL 保留原写入与汇合的拒绝及来源

#### Scenario: Assertion-status field keeps its actual class literal

- **WHEN** Java 8 类的合成 `Z` 字段在 `<clinit>` 中由 `desiredAssertionStatus()` 的分支汇合值写入，且输入 class 可能把调用接收者写成当前类或另一个类的 class literal
- **THEN** 条件值与字段赋值 SHALL 保留输入 class 实际引用的类、真实 `putstatic` 目标及 `-ea`/`-da` 下的求值门控；系统 MUST NOT 仅凭 `$assertionsDisabled` 名字或断言形状改写为源级 `assert` 并改变选择性启停时的行为

### Requirement: Conditional values preserve effects, refusal and evidence

条件值恢复 SHALL 保持测试、臂与消费者的执行/来源顺序，并受既有预算及取消约束。

#### Scenario: Selected arm alone controls effects and exceptions

- **WHEN** 两臂的 helper 分别记录调用或抛出不同异常
- **THEN** 重编译后的完整类 SHALL 与原 class 的返回值、trace、调用次数、异常类别及消息逐项一致；未选臂 SHALL 不执行

#### Scenario: Unproved joins remain refused

- **WHEN** Phi 有外部或多于两个前驱、属于循环/switch/异常边界，臂内有不可折叠独立语句，或类型与最终消费位置不能证明
- **THEN** 本项条件值规则 SHALL 拒绝将该汇合认作双臂 `?:`；若没有另一条已证明的恢复路径，系统 SHALL 保留真实测试、生产者、跳转与消费者 BCI 的完整拒绝，不输出语义不明的条件值或半成品 Java 正文

#### Scenario: Bounded all-or-nothing publication

- **WHEN** 证明、构建或发射途中触发预算/取消，或某臂生产者无法安全表达
- **THEN** 系统 SHALL 沿现有停止/拒绝通道结束；成功时默认及完整证据的正文一致，源映射可查每个真实 BCI，生产者只出现一次
