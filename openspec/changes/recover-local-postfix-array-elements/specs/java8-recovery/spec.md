## ADDED Requirements

### Requirement: A proved local postfix value can appear in an array initializer

当数组元素的旧局部值读取与立即发生的同槽加一更新完整构成 Java 后置自增值时，系统 SHALL 在该元素位置呈现 `local++`，并保持后续元素读取更新后的局部值。输出 SHALL 可用 Java 8 编译，原 class 与生成类的数组值及局部整数溢出行为 MUST 相同。

#### Scenario: Old value and later updated value

- **WHEN** 原 class 的元素字节码来自 `new int[]{1, a++, a * 2}`，且后一个元素读取自增后的 `a`
- **THEN** 输出 SHALL 在第二元素写出一次 `a++`（实际标识符可由命名策略决定），第三元素 SHALL 使用更新后的局部值；`-2`、`0`、`1`、`7` 和 `Integer.MAX_VALUE` 的数组结果 MUST 与原 class 一致

#### Scenario: Exact physical provenance and one evaluation

- **WHEN** 后置自增元素被提交为数组创建表达式
- **THEN** 旧值读取、局部更新、数组分配、索引、所有元素生产者与写入的真实 BCI SHALL 可追踪；更新只执行一次且发生在下一元素求值之前

### Requirement: Similar local updates do not become postfix increments without proof

系统 MUST 仅对同槽、增量一且求值/消费位置闭合的形状写 `local++`；证明不足时 SHALL 保留原普通语句或完整来源拒绝，不发布有错误副作用的数组初始化表达式。

#### Scenario: Increment amount is two

- **WHEN** verifier-valid class 的旧值读取后是 `iinc slot,2` 而非 `iinc slot,1`，其输入 0 的真实数组为 `[1, 0, 4]`
- **THEN** 输出 MUST NOT 把该元素写作会产生 `[1, 0, 2]` 的 `local++`；任何可编译输出的结果 SHALL 为 `[1, 0, 4]`

#### Scenario: Different slot or extra consumer

- **WHEN** 更新的槽与旧值读取不同，或旧值/更新后的局部另有不能归属的读取、入口、异常区边界
- **THEN** 系统 MUST 拒绝该后置自增投影，且不发布只恢复部分数组链的正文

#### Scenario: Bounded or cancelled analysis

- **WHEN** 证明期间预算耗尽或任务取消
- **THEN** 执行状态 SHALL 按现有停止合同返回，MUST NOT 留下只抑制局部更新或只抑制数组写入的部分文本
