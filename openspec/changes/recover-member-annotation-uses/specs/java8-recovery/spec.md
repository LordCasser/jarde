## ADDED Requirements

### Requirement: Class source preserves proved field, method, and parameter declaration annotations

类源码 SHALL 仅依据每个字段、方法和方法参数**自身**的运行时可见/不可见声明注解属性，分别在字段声明前、方法声明前及对应参数类型前写出完整 Java 8 注解使用；MUST 保留注解类型、具名元素和值及属性内顺序，MUST NOT 从 `Deprecated` 标记属性、类级属性、成员名字或其它成员复制注解。参数注解 SHALL 按方法 descriptor 中的参数**位置**对应，long/double 占用的额外 JVM slot MUST NOT 改变其位置。类源码与结构化结果 SHALL 并列保留原属性归属和无法忠实拼写的明确拒绝；一条注解的任一元素失败时 MUST NOT 输出该条的部分源码。此恢复 MUST NOT 改变成员的方法体、字段初始化、`throws`、默认值或无注解成员的签名。

#### Scenario: Field and method runtime-visible annotations

- **WHEN** Java 8 类的一个字段及一个方法各有真实 `@Deprecated` 运行时可见属性
- **THEN** 重新编译的完整类中 `Field.isAnnotationPresent` 和 `Method.isAnnotationPresent` 均与原 class 同为 `true`，普通方法执行值保持相同

#### Scenario: Parameter annotation is attached to its descriptor position

- **WHEN** 方法 descriptor 有多个参数，其中 long/double 占两个 slot，而参数注解属性在某个参数**位置**声明 `@Deprecated`
- **THEN** 源码只在该位置的参数类型前写出注解，重编译后反射的该位置注解类型与原 class 相同；不得偏移到相邻参数

#### Scenario: Runtime-invisible member annotation

- **WHEN** 字段、方法或参数有可拼写的运行时不可见声明注解
- **THEN** 源码保留该位置的注解，重新编译 class 的相应属性保留类型和值；MUST NOT 把它声称为运行时可见

#### Scenario: Ambiguous parameter count refuses parameter annotations

- **WHEN** 参数注解属性计数与方法 descriptor 的参数个数不一致，且没有足够事实证明位置映射
- **THEN** 所有受影响的参数注解保持明确拒绝，不把 annotation index 当成 slot 或擅自左/右对齐；同一方法自身的独立、可拼写注解仍呈现

#### Scenario: One unspellable or repeated annotation does not forge a partial declaration

- **WHEN** 一条成员或参数注解含不可忠实拼写的值、非法 Java 名字，或同一位置同类型重复且无可重复性证明
- **THEN** 该条或重复类型的所有实例不输出部分源码，源码和结构化结果说明拒绝；其它位置或不同类型的已证注解仍可写出

#### Scenario: Damaged attribute or refused read

- **WHEN** 声明/参数注解属性长度或内容损坏、预算拒绝或请求取消
- **THEN** 结果保留真实属性来源及解析/执行停止，MUST NOT 将未完成读取伪装为无注解的完整成功
