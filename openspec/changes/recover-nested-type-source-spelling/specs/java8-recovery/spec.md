## ADDED Requirements

### Requirement: 嵌套类型引用按源码语法拼写

系统 SHALL 在源码语法位的类型引用（声明/参数/局部/调用限定/new/instanceof/cast）中，将常量池嵌套名 `Outer$Inner` 呈现为自嵌套简单名（`Outer` = 当前类时 `Inner`）或点分限定（`Outer.Inner`）；尾段为数字的合成名（`X$1`）与匿名/局部类命名约定 SHALL 逐字不变；方法注与描述符注释位 SHALL 不变。

#### Scenario: 嵌套枚举引用可重编

- **WHEN** 外围类参数类型 `V1$Op` 与常量限定 `V1$Op.MUL` 形态（单类与 jar 输入）三方 Java 8 重编
- **THEN** 呈现 `Op`/`Op.MUL`，`javac --release 8` 通过，运行与原 class 一致

#### Scenario: 既有拼写不变

- **WHEN** 输入为包级类简单名、平台限定名、`X$1` 合成名或 enum 折叠家族文本
- **THEN** 输出与本变更前逐字一致
