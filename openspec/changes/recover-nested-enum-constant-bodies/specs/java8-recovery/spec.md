## ADDED Requirements

### Requirement: 嵌套枚举的常量专属匿名体折叠

系统 SHALL 在枚举二进制名含 `$`（嵌套枚举，如 `Holder$Operation`）时，其常量专属匿名体折叠与顶层名枚举同语义：子类经结构事实（InnerClasses 关系/构造链）而非字符串切分匹配，常量体含覆盖方法恢复文本。顶层名行为、人为 `$` 命名的用户类与既有义务证明 SHALL 逐字不变；义务不满足的嵌套形状 SHALL 保持逐字段呈现。

#### Scenario: 嵌套常量体折叠

- **WHEN** `class Holder { enum Operation implements I { PLUS { … } } }` 以含全部兄弟类的 JAR 三方 Java 8 重编运行
- **THEN** 常量体完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 顶层与负例不变

- **WHEN** 输入为顶层名枚举（`Op`/`OpIface`）、人为 `$` 命名顶层类、或义务不满足的嵌套形状
- **THEN** 输出与本变更前逐字一致
