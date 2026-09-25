## ADDED Requirements

### Requirement: 嵌套注解默认值忠实呈现

当成员的 `AnnotationDefault` 属性含 `@` 或含 `@` 的数组且每个 descriptor、成员名及递归子值均可写成 Java 8 注解值时，class-source SHALL 在该成员声明上输出完整 `default` 表达式。属性中值的顺序和原始事实 MUST 保留；任何子值无法忠实呈现时 MUST 不输出部分数组或凭空补默认值。

#### Scenario: Scalar nested annotation
- **WHEN** 注解成员的默认值是带显式成员对的嵌套注解
- **THEN** 完整生成类 SHALL 编译，反射取得的嵌套注解类型与每个元素值等于原 class；不会把整个 `default` 静默省略

#### Scenario: Array of nested annotations
- **WHEN** 注解成员的默认值是按顺序包含多个嵌套注解的数组
- **THEN** 生成的数组默认值 SHALL 按原顺序递归拼写，完整类反射的数组长度、元素类型和值等于原 class

#### Scenario: Unspellable descendant
- **WHEN** 嵌套注解的 descriptor、成员名或任一子值无法在当前 Java 子集忠实拼写
- **THEN** 本层 MUST 不输出失真的部分 `default`，原始 class 身份/字节及 reader 已解析的属性事实不得因呈现失败而改变，预算/取消沿既有有界读取契约；此项不要求 class-source JSON 新增属性值字段

#### Scenario: Adjacent supported defaults
- **WHEN** 同一注解类型还声明普通整数、字符串、class 或原已支持数组默认值
- **THEN** 这些已支持值的声明和正文 SHALL 不变，默认/完整来源请求的正文一致
