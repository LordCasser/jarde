## MODIFIED Requirements

### Requirement: Historical and Java 8 compiler patterns

系统 SHALL 覆盖已验证的 synthetic accessor、inner/local/anonymous capture、bridge、enum/enum switch、try-with-resources、default/static interface、synchronized/finally 和构造器/字段初始化模式；编译器来源未知时 MUST 使用 generic JVM semantics。对于已证明的标准包装类装箱调用，Java 8 恢复 SHALL 在可证明相同自动装箱目标时呈现底层 primitive 表达式。

#### Scenario: Synthetic accessor recovery

- **WHEN** accessor 的 flags、body 和调用适配满足已验证模式
- **THEN** Java 输出可显示直接字段访问，同时 X1 保留调用 accessor 和 accessor-to-field 两条原始边（验收 A12）

#### Scenario: Missing debug metadata

- **WHEN** Java 8 方法没有 LVT 或 LineNumberTable
- **THEN** 恢复使用确定性 arg/local 名称或 Conservative 输出，不因缺少 debug metadata 伪造源码作用域（验收 A10）

#### Scenario: Primitive literal boxed directly into a return

- **WHEN** 完整 Java 8 方法证据证明一个标准 `Boolean`、`Byte`、`Short`、`Character`、`Integer` 或 `Long` 静态 `valueOf(primitive)` 调用直接产生方法返回值，参数是可呈现的 primitive 常量，且返回类型恰为对应包装类或 `Object`
- **THEN** 源码 SHALL 直接呈现该 primitive 常量，并保留为取得相同包装类型所需的窄化 cast；完整类源码 MUST 通过 Java 8 重编，返回的包装类、值和可观察 identity 行为 MUST 与原 class 一致。owner、name、descriptor、唯一消费者、参数类型或返回转换任一项未证时 MUST 保留显式调用或按现有规则拒绝，不得去掉必要 cast 或推断其它目标类型
