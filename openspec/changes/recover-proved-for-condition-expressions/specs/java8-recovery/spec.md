## ADDED Requirements

### Requirement: A proved counted loop retains its supported condition expressions

系统 SHALL 将满足既有初始化、计数更新、作用域与单次求值证明的循环呈现为for，包括条件含已支持的调用、字段读取、数组读取、取负与数值比较的值表达式。条件 SHALL 留在每轮循环头部，保持原字节码的求值次数、操作数顺序、异常位置和完整物理来源；不能缓存到初始化、移至正文或重复展开可观察生产者。

#### Scenario: A nested loop tests a list on every outer iteration

- **WHEN** 固定TestNestedLoops2外层具有唯一索引初始化与更新，条件调用List.size，内层循环变量不具有对应初始化证明
- **THEN** 完整源码恢复外层for与内层while，空/边界长度/null控制在重编执行后同原class；泛型签名投影的独立拒绝不改变本循环呈现与行为验收

#### Scenario: Condition producers have observable order and exceptions

- **WHEN** 同一已证明循环的条件含多个调用或可能抛异常的读取，原程序可观察每次头部检查的次数与次序
- **THEN** 重编源码保持正常、零次迭代以及初次/后续检查抛出时的结果、调用记录和异常类型，不先执行正文或省略最终条件检查

#### Scenario: The body changes a field or the condition reads an array

- **WHEN** 已证明条件读取正文可能改变的字段，或在原位置读取数组长度/元素
- **THEN** for逐次读取当前值，null与越界异常的位置和可观察行为同原class，读取及条件的所有物理来源保持完整

#### Scenario: Local roles or evaluation ownership are not proved

- **WHEN** 正文写入作为条件不变量的局部、初始化/更新/phi/回边不明确、独立statement或共享effect不能保留原求值与作用域
- **THEN** 系统 MUST 拒绝新的for投影并保留可靠既有表示与来源，不能为出现for丢语句、改变continue目标或提高恢复质量声明

#### Scenario: Evidence modes and stopping agree

- **WHEN** 同一输入以default/all恢复，或在证明/消费/来源发布期间触发预算或取消
- **THEN** 已完成两模式的正文/身份/来源一致；停止传播准确既有状态且不发布部分正文、source map或未完成循环产物
