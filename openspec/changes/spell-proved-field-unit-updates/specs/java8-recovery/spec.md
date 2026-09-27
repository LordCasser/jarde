## ADDED Requirements

### Requirement: Proven statement-only unit field updates use Java update operators

当同次字段与 SSA 证据证明本类 `int` 字段只被读取一次、与常量 1 作准确加法或减法并写回同一字段，且更新结果没有返回或其它消费者时，Java 8 恢复 SHALL 能分别写成后缀 `++`/`--`。输出 MUST 保留字段身份、接收者求值次数、读写顺序与来源；不得把未证的重复字段访问或有结果消费者的表达式视为单位更新。

#### Scenario: Current-instance increment

- **WHEN** 无 handler 的完整单块只执行当前实例准确 `int` 字段的单次加一和写回，随后无值返回
- **THEN** 恢复 SHALL 输出该字段的一次 `++` 语句，完整类 SHALL 以 Java 8 重编并在验证运行中得到与原 class 相同的字段状态

#### Scenario: Same-class static decrement

- **WHEN** 同等完整证据只执行本类非 volatile 静态 `int` 字段的单次减一和写回，随后无值返回
- **THEN** 恢复 SHALL 输出该静态字段的一次 `--` 语句，且不得多执行字段读写

#### Scenario: Member, value flow, or effect is unproved

- **WHEN** 读写字段身份/宽度不同、接收者不是已证明的当前实例/本类静态 owner、字段为 volatile、旧值或更新值还有消费者、链中插入效果或异常范围不完整
- **THEN** 恢复 MUST NOT 输出此首片的后缀语句，并 MUST 保留现有投影或可定位拒绝

#### Scenario: Budget or cancellation stops proof

- **WHEN** 同次证明或原子发射因预算/取消停止
- **THEN** 恢复 MUST NOT 发布半条更新或丢失字段效果，且 SHALL 按现有执行状态报告停止
