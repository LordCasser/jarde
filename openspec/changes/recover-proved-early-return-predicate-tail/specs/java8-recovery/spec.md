## ADDED Requirements

### Requirement: 提前返回谓词的正常路径闭合

对于 Java 8 方法中先由判空或类型 guard 提前返回、再于剩余路径计算布尔值并返回的形状，系统 SHALL 在输出可执行源码前证明每条正常可达路径及其返回值都属于同一完整区域闭包。恢复后的完整类源码 MUST 能以 Java 8 重编，且与原 class 保持返回值、效果次数和异常顺序一致。不能证明闭包时，系统 MUST 原子拒绝受影响的恢复候选、保留物理来源及未证报告；MUST NOT 生成一条无条件返回语句掩盖仍可达的尾路径。

#### Scenario: 判空与类型 guard 后的布尔尾返回

- **WHEN** 一个方法先对 `null` 或非目标类型返回 `false`，真臂安全强转后调用目标方法并将比较结果作为唯一尾返回，所有正常前驱、SSA 值和返回消费者均可证闭合
- **THEN** 系统 SHALL 恢复早退加尾返回或等价短路源码；原 class、JADX、Jarde 的完整 Java 8 类源码与共同 Runner 验证运行 MUST 对 `null`、错误类型、空串和 `"x"` 输出相同结果

#### Scenario: 内层分支后继不同于外层早退 join

- **WHEN** one-armed `if` 的内层区域返回一个可达 `arm_next`，且它不同于外层 join
- **THEN** 系统 MUST 在发表该 `if` 前证明并消费该后继，或拒绝整个受影响候选；MUST NOT 丢弃 `arm_next` 后仍输出把真路径截断的 `return false`

#### Scenario: 路径或效果不完整

- **WHEN** 尾返回的前驱、布尔生产者、cast/调用效果、异常边或区域来源有一项不能证明，或恢复因预算/停止而中断
- **THEN** 系统 MUST 保留可定位的物理 BCI 和 `unproven` 诊断，且 MUST NOT 声称该方法的可执行源码语义已恢复

#### Scenario: 既有比较条件不回退

- **WHEN** 浮点 NaN、无穷、负零比较以及数组上下界判断的现有完整路径无需早退尾合流
- **THEN** 系统 SHALL 保持这些路径的 Java 8 可重编性和与原 class 相同的运行结果
