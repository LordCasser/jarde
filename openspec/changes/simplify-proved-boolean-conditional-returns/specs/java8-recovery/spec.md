## ADDED Requirements

### Requirement: 精确 0/1 条件汇合的布尔返回应呈现为测试表达式

当 Java 8 方法的条件值已证明只有一个布尔返回消费者，测试表达式具有 Boolean 类型，两臂无额外效果且精确产生 0 与 1 时，恢复器 SHALL 按真实分支极性将该返回呈现为测试或其逻辑否定，而不添加整数取余。转换 SHALL 保持测试操作数只求值一次、两臂来源可追及完整类的返回行为。若常量、类型、消费者、控制流或来源任一条件无法证明，恢复器 MUST 保留现有低位适配或诚实引用，不得把任意整数当作源码 Boolean。

#### Scenario: 否定的 instanceof 测试
- **WHEN** 冻结的 `InstanceOfMerge.inverted(Object)` 在 Boolean `instanceof` 测试后，以 true 臂常量 0、false 臂常量 1 汇合并由唯一 `ireturn Z` 消费
- **THEN** 完整 Java 8 类 SHALL 包含等价的 `return !(value(arg0) instanceof java.lang.String);` 形态，不含此返回的 `% 2`，四种输入的布尔结果与 `value` 调用次数 SHALL 与原 class 相同；直接 `instanceof` 返回仍保持原行为

#### Scenario: 正极性的 1/0 测试
- **WHEN** 冻结的 `BooleanMergeControls.positive(Object)` 在 Boolean 测试的 true 臂产生 1、false 臂产生 0，且唯一消费者为 `ireturn Z`
- **THEN** 返回 SHALL 可写为测试表达式本身，源级文本不应增加取余或重复测试操作数的求值

#### Scenario: 非 0/1 整数或未证明的值图
- **WHEN** 任一臂产生 2、3、其它整数或非恒定值，存在其它消费者、外部入口或不完整的 SSA/来源证明
- **THEN** 本规则 MUST NOT 投影为测试或其否定；原有 `ireturn Z` 对整数最低位的行为及拒绝边界 SHALL 保持不变
