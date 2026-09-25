## ADDED Requirements

### Requirement: Direct numeric comparison conditions preserve unordered values

对于 long/float/double 的 JVM 比较结果只由同块紧邻零条件分支消费、且操作数和外围结构可呈现的形状，恢复文本 SHALL 表达与该比较及分支组合一致的 Java 条件。浮点无序结果、正负零及分支方向 MUST 保留；每个操作数 SHALL 按原顺序求值一次。缺少该组合证据时，结果 MUST 保留有来源的缺口，MUST NOT 静默丢掉比较或其调用生产者。

#### Scenario: Long conditions do not use overflowing subtraction

- **WHEN** 自写方法分别通过比较结果的六种零测试判断两个 long，输入包含最小值和最大值
- **THEN** 恢复文本 SHALL 重编译并与原 class 的分支结果一致，MUST NOT 用相减是否小于零替代关系

#### Scenario: NaN true polarity remains a negated relation

- **WHEN** 浮点比较和分支组合在任一操作数为 NaN 时选择真路径，例如 `!(a < b)`
- **THEN** 文本 SHALL 保留无序情况下的真结果，MUST NOT 简化成 NaN 时为假的相反关系 `a >= b`

#### Scenario: Ordered floating comparisons and both branch directions agree

- **WHEN** float/double 条件覆盖六种关系、taken/fall-through 方向、正负零、无穷、有限值与两侧 NaN
- **THEN** 重编译文本的条件结果 SHALL 与原 class 一致，正负零的相等关系 MUST NOT 被排序辅助函数改变

#### Scenario: Calls retain their order count and exceptions

- **WHEN** 可恢复 if 的左右比较操作数分别由有计数或抛异常的调用生成
- **THEN** 恢复正文 SHALL 保持左右顺序、每侧一次求值以及原有异常结果，不得为判定 NaN 重复调用

#### Scenario: Unsupported consumers and operands keep evidence

- **WHEN** 比较结果有多个消费者、存入局部、跨块传递或被其它计算消费，或者操作数在最终消费点无法恢复
- **THEN** 结果 SHALL 明示相应缺口，保留比较、分支及未写成语句的调用生产者来源，MUST NOT 发布未经证明的完整条件

#### Scenario: Accepted conditions retain their sources and budgets

- **WHEN** 恢复成功并请求来源证据，或通过低预算/取消路径终止请求
- **THEN** 成功文本 SHALL 保留两个操作数、比较指令和分支的来源；终止 SHALL 沿用任务状态传播，MUST NOT 另读外部方法体或静默扩大预算
