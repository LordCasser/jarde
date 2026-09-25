## ADDED Requirements

### Requirement: Integer returns preserve descriptor narrowing

对已有证据可完整呈现的整数值，系统 SHALL 保留 ireturn 按本方法 byte、char 或 short 返回类型执行的窄化。不能仅因源码局部或表达式类型为 int 就拒绝这一已知转换，也不能输出与返回签名不相容的正常 Java 正文。

#### Scenario: A narrow local is presented as int

- **WHEN** byte、char 或 short 参数经过局部存取，再由同种窄整数方法返回，且其它结构已经可恢复
- **THEN** 完整输出 SHALL 能原样重编译并保持原 class 的返回值，无需假称已恢复原局部声明类型

#### Scenario: Out of range integers are narrowed by the return instruction

- **WHEN** 合法 class 的 ireturn 向 B、C 或 S 返回任意 int 值，包括负数、边界外值及极值
- **THEN** 返回值 SHALL 与原 JVM 的截断和符号/零扩展结果一致，不能把超范围值直接作为未转换的源码返回值

#### Scenario: A field update is followed by a narrow return

- **WHEN** 已识别的前置或后置字段自增值经 ireturn 窄化返回
- **THEN** 输出 SHALL 保持一次字段更新、更新后字段值与实际返回的窄值，不在字段写入前提前窄化原 int 字段

#### Scenario: Existing structured returns keep the same conversion

- **WHEN** 可呈现的窄整数返回位于现有同步结构中，或由已证明的 switch 返回下推生成
- **THEN** 输出 SHALL 保留原返回窄化、同步效果及分支选择，不能因返回呈现路径不同而省略转换或改变表达式求值位置

#### Scenario: An unsupported operand remains a stated boundary

- **WHEN** 返回操作数本身不能可靠呈现，或其呈现为 boolean 等不满足本项整数转换前提的类型
- **THEN** 系统 SHALL 保留拒绝原因、实际生产者和返回来源，不能凭窄返回签名猜测值或扩大其它位置的转换规则

### Requirement: Narrow return adaptation preserves bounded evidence

窄返回恢复 SHALL 使用有界来源与既有停止契约，证据选择不能改变正文或另行决定转换。

#### Scenario: Complete sources identify the return and its operand

- **WHEN** 同一窄返回请求默认与完整证据
- **THEN** 正文 SHALL 相同，完整来源同时保留实际 ireturn 与值生产位置，特殊返回的重定位不能伪造新的字节码位置

#### Scenario: Insufficient work or output budget stops without fabricated recovery

- **WHEN** 返回转换、节点构造或来源物化期间预算不足或收到取消
- **THEN** 系统 SHALL 沿既有通道停止，不留下未支付的正常转换、丢失操作数引用或在 replay 时改变文本
