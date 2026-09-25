## ADDED Requirements

### Requirement: Explicit primitive conversions preserve values and types

系统 SHALL 恢复已有可接受区域内、操作数证据足够的15种明确JVM基本数值转换，保留每步转换的数值语义、结果类型和后续调用目标。系统 MUST NOT 仅因转换链首尾类型相同而删除中间舍入或窄化。

#### Scenario: Numeric boundaries retain conversion results

- **WHEN** int/long/float/double 与 byte/char/short 间的明确转换接受负数、极值、NaN、无穷、负零及次正规值
- **THEN** 恢复完整类 SHALL 可以原样重编译，整数与浮点位观察结果和原class一致，包括浮点转整数的截断/饱和及窄化

#### Scenario: Round trips retain intermediate rounding

- **WHEN** int经float回到int，long经float/double回到long，或double经float回到double
- **THEN** 恢复结果 SHALL 保持原中间精度，2²⁴与2⁵³附近不可精确表示的整数不能被当作未转换的原值

#### Scenario: Converted arguments keep their original overload

- **WHEN** 转换结果用作重载调用的实参，目标存在byte/short/char/int/long/float/double等多个候选
- **THEN** 恢复文本 SHALL 调用原descriptor指定的目标并保留转换后的值，不能以允许的widening关系删掉必要静态类型

#### Scenario: Operand effects remain in place

- **WHEN** 转换操作数由调用、读取或其它受支持表达式产生，包括跨独立语句保存与任一侧抛错
- **THEN** 系统 SHALL 保留原求值次数、顺序及异常优先级，不重算旧局部或重复执行操作数

#### Scenario: Int-shaped boolean is not silently reinterpreted

- **WHEN** 合法JVM输入的转换读取已证明为boolean的源码值，或操作数类别不能证明与转换相容
- **THEN** 系统 SHALL 明确引用无法恢复的部分并保留必要生产者/消费者来源，不能生成非法boolean数值cast或猜测转换

### Requirement: Primitive conversion recovery remains bounded and traceable

数值转换 SHALL 遵循现有有界恢复及产物契约，不以新表达式绕过深度、预算、取消或证据选择。

#### Scenario: Evidence requests share one body

- **WHEN** 同一转换方法在充足预算下分别请求默认和完整证据
- **THEN** 正文 SHALL 逐字相同，完整来源保留真实转换、操作数、消费者及成员坐标，默认请求不生成来源表

#### Scenario: Deep or interrupted conversion stops preserve artifacts

- **WHEN** 转换嵌套超过深度或工作/IR预算，收到取消，或正文提交后来源预算耗尽
- **THEN** 系统 SHALL 有界停止并保持既有已提交产物，不发生栈溢出、半证明正常表达式、伪造来源或重复发射
