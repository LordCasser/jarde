## ADDED Requirements

### Requirement: Class source preserves proved declaration annotation uses

类源码 SHALL 仅依据该类自身的 `RuntimeVisibleAnnotations` 和 `RuntimeInvisibleAnnotations` 属性呈现其类声明上的注解使用，MUST 保留每个属性内部的条目与具名值顺序、注解类型、元素名和值；无属性时 MUST NOT 根据类名、`Deprecated` 标记属性、注解类型声明或其它位置推断注解。可拼写值 MUST 在类声明前构成完整、合法的 Java 8 注解使用；某条注解的类型、元素名或任一值无法忠实拼写时 MUST NOT 输出该注解的部分源码，并 MUST 在类源码与结构化结果中明确记录这一拒绝。原始类声明事实及属性来源 MUST 保持可核对；类级处理 MUST NOT 改写字段、方法、参数或类型使用位置的注解。

#### Scenario: Runtime-visible marker annotation

- **WHEN** Java 8 类声明具有 `@Deprecated` 对应的运行时可见注解属性，且完整类源码可重新编译
- **THEN** 类源码写出 `@java.lang.Deprecated` 或等价拼写，重新编译后 `isAnnotationPresent(Deprecated.class)` 与原 class 同为 `true`

#### Scenario: Runtime-visible annotation with an enum element

- **WHEN** 注解类型声明的类自身带有 `@Retention(RUNTIME)`，属性写出 `value` 枚举元素
- **THEN** 类源码写出完整注解使用，重新编译后的 `getAnnotation(Retention.class).value()` 与原 class 同为 `RUNTIME`

#### Scenario: Runtime-invisible declaration annotation

- **WHEN** 类自身的 `RuntimeInvisibleAnnotations` 声明一条可拼写的类级注解
- **THEN** 类源码保留这条注解，且不把它声称为运行时可见；重新编译后的对应 class 属性保留注解类型与值

#### Scenario: Unspellable value remains one refusal

- **WHEN** 某条类级注解的一项值有合法原始属性事实，但无忠实的 Java 8 常量表达式，或同一类型重复出现而无法证明 Java 声明合法
- **THEN** 该注解整体省略并出现明确拒绝记录，其他独立且可拼写的类级注解仍按自身属性呈现；MUST NOT 输出部分元素或猜测归一化值

#### Scenario: Damaged or refused attribute read

- **WHEN** 类级注解属性内容损坏、深度超过受支持界限、预算拒绝或请求取消
- **THEN** 结果报告真实解析/执行停止及来源，MUST NOT 把未完成的属性读当成无注解的完整成功，也 MUST NOT 从其它属性补造注解使用
