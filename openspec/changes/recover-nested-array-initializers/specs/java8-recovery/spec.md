## ADDED Requirements

### Requirement: Proven nested array initialization preserves the original execution

当多维数组的每一层分配、元素写入及最终消费均可由字节码事实完整证明时，系统 SHALL 将嵌套数组链呈现为可编译的 Java 8 数组表达式。恢复后的分配次数、数组形状、元素值、求值顺序、副作用和异常行为 MUST 与原 class 一致。

#### Scenario: Nested primitive arrays with observable elements

- **WHEN** 原 class 为 `int[][]` 顺序分配外层与两个内层数组，内层元素按 1、2、3 调用可观察的 `element`，再按 0、1 写入外层
- **THEN** 生成的完整类 SHALL 以 Java 8 编译；在 `java -Xverify:all` 下，数组形状和值、三次调用的 1→2→3 顺序及计数 MUST 与原 class 相同

#### Scenario: Nested reference arrays

- **WHEN** 原 class 顺序构造 `String[][]`，各层引用组件与写入类型可直接证明兼容
- **THEN** 生成的 Java 8 完整类 SHALL 可编译，数组层级及各元素值 MUST 与原 class 相同

#### Scenario: Ownership and evidence for every nested level

- **WHEN** 系统将内层初始化值作为外层数组元素提交
- **THEN** 每次分配、复制、索引、元素生产者和写入的真实 BCI SHALL 可追踪且只执行一次；任一层证据不足或预算/取消停止时 MUST 不发布部分嵌套表达式

### Requirement: Array initializer folding never reorders observable stores

系统 MUST 保留 JVM 的物理元素求值及存储顺序；不能只因最终数组值相同就按索引排序后写成字面量。

#### Scenario: Effectful stores arrive in reverse index order

- **WHEN** 原 class 先执行 `result[1] = mark(1)`，再执行 `result[0] = mark(2)`，原验证运行结果的 trace 是 `12`
- **THEN** 输出 SHALL 保留 `mark(1)` 先于 `mark(2)`，或完整拒绝；若输出可编译 Java，其 trace MUST 是 `12`，MUST NOT 将两项改写成会产生 `21` 的数组字面量

#### Scenario: Nested proof has an unclosed boundary

- **WHEN** 任一层出现不连续/重复/乱序索引、额外引用使用、跨块入口、不同异常处理集合或无法证明等价的引用组件赋值
- **THEN** 系统 MUST 保留普通写入或带完整来源的拒绝，不得把其余层发布为声称完整恢复的嵌套表达式
