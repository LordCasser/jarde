## ADDED Requirements

### Requirement: Single evaluation of a proved popped static-call qualifier

当一段 Java 8 字节码把某个表达式结果丢弃后立即执行静态调用，且恢复层能证明后者的 Java 调用文本可承载该表达式时，生成正文 SHALL 只执行该表达式一次，MUST 保持它在静态调用和后续写入之前的顺序。若不能证明可承载，恢复层 MUST 保留原生产者的独立效果或在拒绝范围及来源中点名它，MUST NOT 同时独立发射又嵌入另一表达式。

#### Scenario: Static field write with a popped expression
- **WHEN** 普通 Java 8 方法先调用有副作用的引用值生产者、丢弃结果，再调用静态 RHS 并写入静态字段
- **THEN** 完整生成类的字段值、生产者及 RHS 调用次数、顺序和异常与原 class SHALL 一致；生产者 MUST 只执行一次

#### Scenario: Source-qualified static call
- **WHEN** 源码通过有副作用的表达式限定静态方法调用，字节码为生产者、丢弃和静态调用
- **THEN** 完整生成类 SHALL 与原 class 同次数执行生产者与目标静态调用，并保留其求值顺序

#### Scenario: Qualifier throws before static call
- **WHEN** 被丢弃表达式在求值时抛异常
- **THEN** 生成代码 SHALL 在静态调用及后续写入之前抛同类异常；目标调用及写入 MUST NOT 已发生

#### Scenario: A qualifier cannot be rendered or attached
- **WHEN** 生产者、其引用类型、静态调用目标或消费关系不足以证明一个可编译且单次求值的限定调用
- **THEN** 恢复层 MUST 保留生产者、丢弃、调用及最终消费者的物理来源与可观察效果，MUST NOT 输出重复调用或不合法的限定调用

#### Scenario: Constant-pool target differs from the qualifier's selectable member
- **WHEN** 合法 JVM 输入中的 `invokestatic` 常量池目标与限定表达式的 Java 类型所选到的方法不同，即使两者同名且签名相同
- **THEN** 恢复层 MUST NOT 以 `qualifier().method()` 改绑到另一个目标；若生产者可独立作为语句，SHALL 先发射该语句并按常量池目标调用，无法证明等价时 SHALL 拒绝并保留目标及限定生产者来源

#### Scenario: Interface static method after a discarded invocation result
- **WHEN** 已验证的 JVM 字节码先丢弃方法调用返回的接口引用，再以 `InterfaceMethodref` 调用接口静态方法
- **THEN** 生成代码 SHALL 先且仅先执行一次生产者，再以接口类型名调用静态方法；MUST NOT 输出 Java 禁止的实例表达式限定接口静态调用

#### Scenario: Ordinary discard remains separate
- **WHEN** 一个返回值调用被丢弃，后面不存在已证明以它为限定符的静态调用
- **THEN** 调用 SHALL 作为一次独立求值或完整拒绝呈现，不得因相邻 `pop` 被漏掉或附到无关调用

#### Scenario: Evidence and bounded refusal
- **WHEN** 调用方请求默认/完整来源，或在限定符生产者与消费链中耗尽正文/来源预算或取消
- **THEN** 两种来源模式 SHALL 有相同正文；成功时保留生产者、丢弃、静态调用及其消费者的真实 BCI，拒绝时来源仍包含延期生产者；停止遵守既有不发布部分结果的契约
