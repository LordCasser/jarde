## ADDED Requirements

### Requirement: A method's Region tree has one owner for every canonical block it presents

Java 8 恢复若在同一次方法 walk 中把一个 canonical block 身份放进多个 Region 位置，MUST 在发射 Java 前原子拒绝这棵部分结构，MUST 报告明确的所有权重叠理由，并 MUST 完整引用已解码 live 指令。它 MUST NOT 将普通汇合点的重访声称为已证明的循环，或在值生产者未归属时发表消费点的结构化语句。不同 `jsr` 路径的同 BCI canonical 身份仍应分别核对，不得仅按数字地址误拒绝。

#### Scenario: A ternary arm enters a shared OR producer

- **WHEN** Java 8 class 的 `result = (gate ? extra : left) || other || rhs()` 包含 BCI 8 `goto 25`，而后续正常短路测试也进入 BCI 25 的真值生产者
- **THEN** 方法恢复 SHALL 只给该 canonical 生产者一个拒绝所有者，报告 `fallback` 而非四个伪循环；引用及 source map SHALL 覆盖 BCI 8、25、26、29、30 及其它已解码 live 指令，不得给 BCI 30 发表缺少栈来源的结构化字段赋值

#### Scenario: Existing closed structures keep their owners

- **WHEN** 输入是已证明的两/三测试纯短路写入、合法 loop、switch fallthrough、try/catch 或具有不同路径身份的 `jsr` 克隆
- **THEN** 新校验 SHALL 保留原有结构化/拒绝决策；只有同一 canonical 身份在 Region 树中实际重复时才可整方法引用

#### Scenario: Ownership validation cannot finish

- **WHEN** 所有权扫描或完整来源采集的预算用尽、超时或调用方取消
- **THEN** 本次恢复 SHALL 停止而不交付半棵 Region 树、半段 Java 或伪完整 source map
