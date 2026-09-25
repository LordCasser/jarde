## ADDED Requirements

### Requirement: Proven compound additions to field and array lvalues

当 Java 8 方法中实例 `int` 字段或 `int[]` 元素的复合加法写入可由输入指令、成员/数组类型和数据流完整证明时，恢复层 SHALL 生成语义等价的 `+=` 语句。该语句 MUST 对接收者、数组和索引各求值一次，MUST 在右侧表达式之前取得左值旧值，并在右侧之后写回运算结果。证据不足时 MUST 保留被拒绝的读取、复制、右侧效果、运算和写入及其物理来源，MUST NOT 用丢失写入或重复求值的简单赋值冒充恢复。

#### Scenario: Instance field receiver and right side have effects
- **WHEN** 一次实例 `int` 字段复合加法的接收者与右侧各调用一个可观察方法，且两个字段访问指向同一已证明成员
- **THEN** 生成的完整类编译运行后 SHALL 与原 class 具有相同字段值、调用次数和调用顺序；接收者与右侧均各执行一次

#### Scenario: Array index and right side have effects
- **WHEN** 一次 `int[]` 元素复合加法的索引与右侧各调用一个可观察方法，且读取与写入消费同一数组/索引身份
- **THEN** 生成的完整类编译运行后 SHALL 与原 class 具有相同元素值和调用次数，数组及索引只求值一次

#### Scenario: Right side mutates the same lvalue
- **WHEN** 字段或数组元素的右侧方法在返回操作数前改写该左值
- **THEN** 生成代码 SHALL 使用右侧调用之前取得的旧值进行加法，并在右侧结束后写回结果

#### Scenario: Compound update fails before the right side
- **WHEN** 字段接收者为 null，或数组接收者为 null、索引越界
- **THEN** 生成代码 SHALL 在与原 class 相同的阶段抛出相同类别的异常，MUST NOT 执行尚未到达的右侧方法

#### Scenario: Unproved update chain
- **WHEN** 字段身份、数组/索引身份、复制值、类型或消费链任一条件不能证明，或者复制值另有消费者
- **THEN** 恢复层 MUST 保守引用整条受影响链及可观察生产者，MUST NOT 输出声称单次求值的 `+=` 或造成重复求值的 `=` 正文

#### Scenario: Origins and bounded output
- **WHEN** 调用方请求默认或完整来源，或在复合更新中耗尽正文/来源预算或取消任务
- **THEN** 两种来源模式 SHALL 保持同一正文，并为实际接收者、索引、读取、运算、写入和右侧消费者保留原始 BCI/成员来源；停止 MUST 遵守既有停止契约，不交付部分声称完整的更新
