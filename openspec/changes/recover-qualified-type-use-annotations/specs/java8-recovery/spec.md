## ADDED Requirements

### Requirement: Class source preserves provable qualified type-use annotations

类源码 SHALL 将字段类型、方法返回类型与形参类型的运行时可见及不可见类型注解，和相同位置的声明注解分开呈现。只有目标、类型路径、形参位置、注解类型和值及 Java 源放置位置均有完整证据时，才 SHALL 写出该类型使用；写出的源码 MUST 保持原有类型注解的可见性、目标、形参位置与值，MUST NOT 因类型注解新增声明注解。无法忠实拼写的类型使用 SHALL 保留原属性归属并给出明确拒绝，不得静默省略或输出半条注解；受损读取、预算拒绝或取消 MUST 保留真实执行停止。

#### Scenario: Field type annotation has a distinct source position

- **WHEN** 字段的简单限定引用类型有路径为空、目标为 `FIELD` 的可拼写运行时类型注解，且与声明注解无冲突
- **THEN** 字段的完整类源码 SHALL 在类型名内部的位置拼写该注解；重编译后字段 `AnnotatedType` 的注解类型和值与原 class 相同，而字段声明注解集合不增加

#### Scenario: Return and parameter types retain separate targets

- **WHEN** 方法返回类型和一个形参类型各有路径为空、分别为 `METHOD_RETURN` 和 `METHOD_FORMAL_PARAMETER` 的可拼写注解
- **THEN** 源码 SHALL 在各自的类型内部拼写，重编译后的返回类型及该 descriptor 参数位置的 `AnnotatedType` 注解与原 class 相同；`long`/`double` 的 JVM slot 宽度 MUST NOT 改变形参目标位置

#### Scenario: Runtime-invisible type annotation

- **WHEN** 上述可拼写位置的原属性为 `RuntimeInvisibleTypeAnnotations`
- **THEN** 重编译 class 的同一类型目标保留注解类型和值及运行时不可见性，结果 MUST NOT 声称它可由运行时反射读取

#### Scenario: Source position cannot prove type-only meaning

- **WHEN** 注解位于基本类型、无可用限定内部位置的单段引用名、数组层级或非空类型路径，或者同位置声明与类型注解的拼写可能制造重复或改变归属
- **THEN** 该类型使用 SHALL 明确拒绝并保留原目标、路径和值；MUST NOT 把类型注解写成普通声明前缀，也 MUST NOT 猜测外部注解类型的 `@Target`

#### Scenario: Invalid or unsupported type-annotation content

- **WHEN** 类型注解属性的结构或长度损坏、目标与承载位置矛盾、形参目标越过 descriptor 参数个数，或者注解的一项无法作为 Java 8 源码忠实拼写
- **THEN** 结构/读取失败 SHALL 给出相应停止，语义不可拼写 SHALL 逐条拒绝；已读取的原始属性壳和其它独立、可证明的注解仍可对照，MUST NOT 用空列表伪装完整成功
