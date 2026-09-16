## Purpose

为嵌套归档、MR-JAR 与 Boot 布局建立可复核的物理和运行时选择模型，避免把文件路径启发式或单一 classpath 视为 JVM 的完整加载规则。

## ADDED Requirements

### Requirement: Physical and runtime views are separate

系统 SHALL 先保留选定范围内全部物理定义和 entry，再根据显式 RuntimeProfile、LoadDomain、版本和布局规则派生 RuntimeView。物理视图不得因运行时选择而丢失证据。

#### Scenario: Multi-release variants

- **WHEN** 同一 MR-JAR 含 root、versions/11 和 versions/17 变体
- **THEN** PhysicalView 列出全部变体，Java 8、11、17 的 RuntimeView 分别选择正确候选（验收 A06）

#### Scenario: Duplicate definitions in WAR

- **WHEN** WAR 或嵌套依赖中存在多个同名类
- **THEN** 系统保留每个物理 origin，并在 RuntimeView 中保留显式 loader/order 条件；P1 不做唯一解析或静默消歧（验收 A07）

### Requirement: Bounded nested and Boot layout traversal

系统 SHALL 将 nested JAR、`BOOT-INF/classes`、`BOOT-INF/lib`、WAR 的 `WEB-INF/classes` 和 `WEB-INF/lib` 表示为带来源的布局节点；递归展开 MUST 受深度、entry、字节和取消预算约束，并且不得自动联网或执行 launcher。

#### Scenario: Nested DEFLATED archive

- **WHEN** 嵌套 JAR 使用 DEFLATED 且展开后不超过预算
- **THEN** 系统物化并扫描其内容，同时保留外层 entry 到内层 definition 的 origin 链（验收 A08）

#### Scenario: Nested traversal exceeds budget

- **WHEN** 递归展开超过任一嵌套深度、字节或 entry 预算
- **THEN** 结果声明具体预算维度、已扫描范围和未扫描范围，且不标为 Complete（验收 A14）
