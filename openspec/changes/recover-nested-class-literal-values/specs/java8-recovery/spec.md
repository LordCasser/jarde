## ADDED Requirements

### Requirement: 嵌套类的类字面量值可证明并呈现

系统 SHALL 在 `ldc` 的 CONSTANT_Class 名为源码可拼的嵌套类型名（各段为合法标识符、尾段非纯数字、非本地类形）时准入该类字面量值，并按既有类字面量呈现通道与 nested-spelling 拼写规则输出。顶层类字面量的全部既有形态 SHALL 逐字不变；本地类、匿名类与不可拼名的字面量 SHALL 保持既有拒绝。

#### Scenario: 嵌套类字面量恢复

- **WHEN** `Nested.class.getSimpleName()` 与反射形 `Marked.class.getAnnotation(Tag.class)` 家族三方 Java 8 重编
- **THEN** 方法完整呈现（无引注），`javac --release 8` 通过，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 顶层与负例不变

- **WHEN** 输入为顶层类字面量五形（实参/receiver/拼接/链式/存局部）或本地类、匿名类字面量
- **THEN** 前者与本变更前逐字一致；后者保持既有拒绝
