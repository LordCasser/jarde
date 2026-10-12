## ADDED Requirements

### Requirement: Ordered instance field reads compose with concatenation arguments

系统 SHALL 恢复同块已证明拼接中直接供下一片段转换消费的实例字段读值，只要该字段成员可精确呈现、receiver 是链内原位的实际局部读取、读值仅被该片段消费一次且完整拼接至其唯一消费者的求值顺序与异常保护范围一致。系统 MUST 保持读取、receiver 求值、片段转换、可能抛出的异常与 volatile 访问次数及顺序；MUST NOT 把字段视为常量或纯值进行移动、复制或删除。该拼接作为构造实参时，系统 SHALL 组合真正的字符串结果，保持物理构造器 overload、异常类型与消息，而非把中间 Builder 当作实参。

#### Scenario: A dynamic Error message remains observable

- **WHEN** 完整 retry-v2 类以 `new AssertionError("uncaught-error:" + c)` 在第二次调用逃逸，且已有拼接与构造形状符合上述条件
- **THEN** 未改写的恢复整类 SHALL 能编译，调用、catch、break、事件顺序、Error 类型及消息 MUST 与原 class 相同；debug/none × Java8/23 四组 SHALL 全部通过，不能用简化消息的 retry-v3 代替

#### Scenario: An instance read keeps its original position and conversion

- **WHEN** 合法链直接拼接实际局部 receiver 的普通或 volatile 实例字段，含原 receiver 为 null、Object 片段转换会改变后续字段值的输入
- **THEN** 恢复 SHALL 保持一次字段读取与 receiver 求值、原转换时机及异常顺序；null 输入的异常 MUST 留在原保护范围，不能为读取增加 pure/nonvolatile 假设

#### Scenario: Evidence cannot preserve one field operand

- **WHEN** 字段成员呈现未证明、读值有额外消费者、不是原位直接片段、链跨块、字段/拼接/消费者跨异常范围、或夹有无归属写入及 void 调用
- **THEN** 系统 MUST 保留原拒绝与物理来源，不能发布部分拼接所有权或吞掉被拒表达式的可观察生产者；本许可 MUST NOT 扩展到未证明的 static 类初始化/常量折叠或任意 receiver 计算

#### Scenario: New operand verification stops

- **WHEN** 新增字段值流、消费者、区间或异常行核验在预算耗尽或取消时停止
- **THEN** 系统 MUST 返回真实停止原因与 BCI，MUST NOT 把停止转为不匹配、普通拒绝或成功，MUST NOT 发布部分 owner、正文或来源结果
