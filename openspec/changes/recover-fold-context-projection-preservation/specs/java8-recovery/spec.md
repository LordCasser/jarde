## ADDED Requirements

### Requirement: 带伴生投影的根类可折叠且保留首轮投影呈现

系统 SHALL 在根类文本含 lambda 伴生投影（或数组/枚举/初始化器投影）时仍可执行成员折叠，且折叠产物保留与首轮一致的投影呈现（不退回物理文本）；折叠产物二次运行 SHALL 不出现劣于首轮的呈现。无投影根的家族折叠 SHALL 与本变更前逐字一致。

#### Scenario: lambda 根折叠

- **WHEN** Y1 形（lambda 内联根 + 单成员子）家族三方 Java 8 重编
- **THEN** `interface StrFn` 嵌套声明与 lambda 内联同时呈现，`java -Xverify:all` 逐路径与原 class 一致（`hi!`/`45`/`[b, aa]`/`8`）

#### Scenario: 既有家族与重跑不变

- **WHEN** 输入为无投影根家族，或折叠产物再次运行
- **THEN** 前者与本变更前逐字一致；后者不劣于首轮
