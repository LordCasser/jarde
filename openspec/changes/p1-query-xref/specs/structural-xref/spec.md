## Purpose

让调用方能够在不启动反编译管线的情况下查询 classfile、归档 metadata 和资源中的结构引用，并沿证据路径复核每一个结果。

## ADDED Requirements

### Requirement: Structural consumer queries

系统 SHALL 区分 `constant_pool_contains`、`mentions_symbol` 和 `literal_value`，并覆盖已声明的 invocation、field、type、constant、exception、signature、annotation、inner/nest、module、bootstrap 和标准资源 consumer。P1 不实现 `references_definition` 或 `may_dispatch_to`；对这两类请求 MUST 返回 `UnsupportedAnalysis`，同时保留 target relation 与原始 symbol evidence。X1 查询 MUST 不构建 CFG、SSA 或 Java AST。

#### Scenario: Unused constant-pool method reference

- **WHEN** classfile 的常量池包含未被任何 Code consumer 使用的 `Methodref`
- **THEN** X0 可报告该候选，但 X1 不得生成调用引用（验收 A01、A17）

#### Scenario: Metadata-only type reference

- **WHEN** 类型只出现在 Signature、annotation descriptor 或 Code catch type 中
- **THEN** 开启对应 consumer 类别时查询必须命中并保留类别与位置证据（验收 A03）

#### Scenario: Definition relation requested before resolver

- **WHEN** 调用方在 P1 请求 `references_definition` 或 `may_dispatch_to`
- **THEN** 返回 `UnsupportedAnalysis`，保留目标关系和 `mentions_symbol`/raw CP 证据，不进行候选扩展或解析；该能力由 P2 规划

#### Scenario: Deferred bootstrap graph

- **WHEN** 实际 consumer 使用 invokedynamic、dynamic constant 或 MethodHandle
- **THEN** 结果记录 use-site、bootstrap handle 和参数边；未被 consumer 使用的 BootstrapMethods 不得被当作执行引用（验收 A04、A05）

### Requirement: Resource and nested consumer coverage

系统 SHALL 扫描 Manifest 的 Main-Class、agent、Class-Path、Automatic-Module-Name、Multi-Release、`META-INF/services` 和 module-info 的 uses/provides，并 SHALL 将框架专用格式留在带规则版本和 coverage 的插件边界内。

#### Scenario: Service provider declaration

- **WHEN** 归档含 `META-INF/services/<service>` 及 provider 名称
- **THEN** 查询返回结构注册关系及来源 entry，但不声称服务已经加载或调用

#### Scenario: Nested JAR consumer

- **WHEN** WAR 或 Boot 布局的 nested JAR 被显式纳入查询范围
- **THEN** 系统在预算内递归读取其 consumer；超出预算返回 Partial/Unsupported 和已处理范围，不返回空结果（验收 A08、A14）
