## ADDED Requirements

### Requirement: Shift expressions preserve integral width and evaluation

系统 SHALL 恢复已有可接受区域内且操作数证据足够的 int/long 左移、带符号右移和无符号右移表达式，保持左值决定的结果宽度、距离语义、表达式分组以及左右各一次求值。系统 MUST NOT 将移位等同普通二元数值提升，或对不能表达的操作数输出貌似完整的 Java 正文。

#### Scenario: Signed values and extreme distances retain their results

- **WHEN** int/long 移位的左值含负数、零、极值，距离含负数、31、32、63、64 和超宽值
- **THEN** 完整恢复类 SHALL 可原样重编译，三种移位的实际执行结果与原 class 相同

#### Scenario: Narrow operands and nesting retain source types

- **WHEN** byte/short/char 左值参与已支持的移位，结果经过嵌套移位、算术、局部保存、实参或条件消费
- **THEN** 恢复正文 SHALL 使用左值独立提升得到的类型并保留原分组，不因右侧类型改变结果宽度或选错调用目标

#### Scenario: Operand effects occur once and in order

- **WHEN** 左右值由调用或其它已接受的效果表达式产生，包括任一侧抛错或生产值跨过独立语句的已支持形状
- **THEN** 恢复正文 SHALL 保持左右求值顺序、次数与异常优先级，左侧抛错后不执行右侧，不能在消费者处重新计算已产生的值

#### Scenario: Unspellable operands remain an explicit boundary

- **WHEN** JVM 合法输入将 descriptor 已证明的 boolean 用作移位操作数，或消费链含未支持转换、失效旧局部或无法证明的区域关系
- **THEN** 系统 SHALL 明确引用不能恢复的部分并保留必要生产者和消费者来源，不能输出 boolean 移位、读取覆盖后的新值或丢失调用效果

### Requirement: Shift recovery retains bounded artifact production

移位恢复 SHALL 沿用有界工作、递归限制、取消和来源提交契约，证据选择不能改变已恢复正文。

#### Scenario: Evidence changes metadata without changing text

- **WHEN** 同一移位方法分别请求默认与完整来源且预算充足
- **THEN** 正文 SHALL 逐字一致，完整来源含实际运算、操作数、消费者及成员坐标，默认请求不发布来源表

#### Scenario: Resource stops never publish an unproved normal expression

- **WHEN** 移位嵌套超过深度、工作或 IR 预算，发生取消，或正文提交后来源预算耗尽
- **THEN** 系统 SHALL 有界返回并保留已有产物契约，不发生进程崩溃、重复发射、伪造来源或将未完成证明当作正常表达式
