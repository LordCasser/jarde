## ADDED Requirements

### Requirement: Proven boolean XOR with a literal is presented without redundant XOR

当完整方法证据证明 JVM `ixor` 左值是 boolean 且右值为准确布尔常量 `1` 或 `0` 时，Java 8 恢复 SHALL 分别投影为左值的逻辑否定或左值本身。投影 MUST 恰好求值左表达式一次，保持原有调用、副作用与来源事实；不得把仅在结果上下文要求 boolean 的整数位运算误作布尔运算。

#### Scenario: Boolean parameter or call XOR true

- **WHEN** 左值有独立 `Z` 描述符/已决局部证据，右值是准确常量 `1`，且两个操作数的 SSA 读取、`ixor` 与结果用途完整
- **THEN** 恢复 SHALL 输出 `!` 作用于原左表达式，完整 Java 8 类 SHALL 可重编，并在左侧为有副作用调用时只调用一次

#### Scenario: Boolean call XOR false

- **WHEN** 同等完整证据下右值是准确常量 `0`
- **THEN** 恢复 SHALL 输出原左表达式一次，不得删除它的调用或改变返回值

#### Scenario: Numeric or unproved XOR

- **WHEN** 左值为整数/长整型或布尔类型不明、右侧不是准确 0/1 字面量、或者操作数/来源证明不完整
- **THEN** 恢复 MUST NOT 应用布尔否定/恒等简化，并 MUST 保留原有可编译投影或原有拒绝

#### Scenario: Budget or cancellation stops proof

- **WHEN** 完整证明或表达式写入因预算或取消停止
- **THEN** 恢复 MUST NOT 发布半个简化表达式，并 SHALL 按现有状态报告停止
