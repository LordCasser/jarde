## ADDED Requirements

### Requirement: The instance receiver is spelled by its identity

呈现层 SHALL 把非 `static` 方法（构造器包含在内）的槽位 0 按 JVMS 定义的接收者呈现，而不是按槽位序数或调试名呈现：该槽位的读取、实例字段访问与实例方法调用 MUST 写成 `this`（`this.f`、`this.m()`、作为实参时写 `this`）。接收者身份 SHALL 由该成员自己的 `ACC_STATIC` 事实单独判定，MUST NOT 依赖 `LocalVariableTable` 的存在、缺失或其内容；调试表把槽位 0 命名成什么（包含 `this` 这一关键字）MUST NOT 导致别名为 `this_` 一类拼写。

其它槽位的命名规则 MUST 保持不变：参数槽位使用调试名，缺少证据时写 `arg<slot>`；局部槽位使用调试名，缺少证据时写 `local<slot>`；不可拼写的名字仍按既有确定性别名规则处理。MUST NOT 为旧拼写保留兼容开关，MUST NOT 在同一产物里对同一接收者混用两种拼写。

#### Scenario: An instance method with debug information names its receiver

- **WHEN** 一个非 `static` 方法带有声明槽位 0 的 `LocalVariableTable`（例如名字为 `this`），且其字节码读取该槽位上的字段
- **THEN** 文本 MUST 写成 `return this.field;` 形态，MUST NOT 写成 `this_.field` 或 `arg0.field`；该成员其余槽位的命名与改动前一致（C01；A10）

#### Scenario: An instance method without debug information still names its receiver

- **WHEN** 同一个实例方法没有 `LocalVariableTable`
- **THEN** 接收者 MUST 仍写成 `this`，而参数按既有规则写成 `arg<slot>`（槽位序数不含接收者以外的含义）；MUST NOT 出现把接收者写成 `arg0` 的文本（C01；A10）

#### Scenario: A static method is untouched

- **WHEN** 一个 `static` 方法的槽位 0 是普通参数
- **THEN** 命名 MUST 保持既有规则（调试名或 `arg0`），MUST NOT 写成 `this`（C01）

#### Scenario: Receiver expressions keep their grouping and evaluation

- **WHEN** 呈现把子表达式写在调用或字段读取的接收者位置，或以接收者自身作为实参
- **THEN** 既有 receiver 分组与求值位置要求 MUST 继续成立：文本 MUST 解析回同一棵树，MUST NOT 因本要求改变括号、操作数顺序或求值点（C03；A13）

#### Scenario: A receiver that was never read is not invented

- **WHEN** 某方法的字节码从未读取槽位 0
- **THEN** 呈现 MUST NOT 为写 `this` 而新增任何读取或语句；本要求只改变该槽位被写出时的拼写（C03）
