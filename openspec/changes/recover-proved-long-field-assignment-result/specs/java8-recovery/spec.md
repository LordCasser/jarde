## ADDED Requirements

### Requirement: Proven instance long field assignment result is recoverable

当完整方法证据证明一次对本实例 `long` 字段的赋值和紧随其后的返回使用同一个方法参数值时，Java 8 恢复 SHALL 输出可编译源码，使字段只写一次且返回该次写入的值。输出 MUST 保留物理字段目标、值宽度、求值顺序和来源位置，不得把未经证明的栈复制当作第二次求值。

#### Scenario: Direct parameter assigned and returned

- **WHEN** Java 8 方法只把一个 `long` 参数写入本类实例的准确 `long` 字段，并返回同一值，且其代码、字段目标、值流与异常范围均完整可证
- **THEN** 恢复结果 SHALL 写出一次字段赋值和一次同值返回，完整类源码 SHALL 能以 Java 8 重编，并在正数、跨 32 位边界和负数输入下与原 class 的返回值及字段状态一致

#### Scenario: Copy or field identity is unproved

- **WHEN** 字段 owner/描述符、参数来源、复制值的两个消费者、返回目标、附加操作或异常范围有任一项与已证明形态不符
- **THEN** 恢复 MUST 不发布该赋值结果的源码投影，并 MUST 保留可定位的拒绝证据，不得输出声称完整却重复写入或返回另一值的方法

#### Scenario: Budget or cancellation stops proof

- **WHEN** 同次证明或发射在输出预算、分析预算或取消处停止
- **THEN** 恢复 MUST 不发布半个赋值/返回组合，且 SHALL 按现有执行状态报告停止
