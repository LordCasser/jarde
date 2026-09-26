## ADDED Requirements

### Requirement: 词法 super 桥投影的双端来源

每处由物理 `access$` 方法桥投影的 `Outer.super` 表达式 SHALL 同时保留成员调用点的完整物理方法/BCI、桥定义的完整物理方法及其 `invokespecial` BCI、目标 `MethodRef` 与解析后的目标物理方法身份，并标注派生关系。只有源码声明可按完整闭包省略该 helper；物理桥、参数和方法体 MUST 留在报告中。不同物理定义的相同 BCI MUST NOT 合并。

#### Scenario: 一个桥被两个成员调用点使用

- **WHEN** 两个已证明的成员调用点都投影为词法 super 调用
- **THEN** 两处源码表达式 SHALL 分别指回各自 caller BCI，同时都能追到同一个准确的桥定义与桥内 `invokespecial` BCI

#### Scenario: 桥投影拒绝

- **WHEN** 任何调用点或桥体无法证明而家族保持物理呈现
- **THEN** 原桥与调用点 SHALL 仍以各自完整方法身份和 BCI 留在 source map/拒绝报告，不能只有一个合成的 `Outer.super` 来源
