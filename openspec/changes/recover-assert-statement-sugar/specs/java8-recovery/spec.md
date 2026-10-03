## ADDED Requirements

### Requirement: assert 降低按 assert 语句呈现并消隐合成守卫物

系统 SHALL 在识别 javac 断言合成模式（`$assertionsDisabled` 字段 + `<clinit>` desiredAssertionStatus 初始化 + 使用点守卫形）且回写为 `assert cond [: msg];` 后，消隐该合成字段与初始化行。无 assert 类、守卫含额外语句或识别失败的形态 SHALL 与本变更前逐字一致；`-ea` 与无 `-ea` 两态运行行为 SHALL 与原 class 一致。

#### Scenario: assert 语句回写

- **WHEN** A1 形（两形 assert + 嵌套类独立字段）家族三方 Java 8 重编并以两断言态运行
- **THEN** `assert cond : msg;` 语句呈现、合成物消隐，两态输出与原 class 一致

#### Scenario: 既有与负例不变

- **WHEN** 输入为无 assert 类或守卫含额外语句
- **THEN** 输出与本变更前逐字一致
