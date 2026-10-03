## ADDED Requirements

### Requirement: 嵌套泛型类的类型参数头可投影

系统 SHALL 在嵌套类的 InnerClasses 行满足源名/kind/名字判据（与折叠片同一判据源）时，按其 Signature 呈现类型参数头（如 `Box<U>`），并向该类成员投影注入自身类型变量域。顶层泛型头、既有泛型切片与折叠防线 SHALL 零回退；行不可拼或名字不一致的形状 SHALL 保持既有阻断链。

#### Scenario: 嵌套泛型头呈现

- **WHEN** `static class Box<U> { U value; }` 家族三方 Java 8 重编
- **THEN** 分离与折叠两口径呈现 `Box<U>` 头与 `U value` 字段，运行与原 class 一致

#### Scenario: 既有与负例不变

- **WHEN** 输入为顶层泛型类、行不可拼嵌套类或既有泛型投影场景
- **THEN** 输出与本变更前逐字一致
