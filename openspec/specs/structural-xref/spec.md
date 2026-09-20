# structural-xref Specification

## Purpose
让调用方能够在不启动反编译管线的情况下查询 classfile、归档 metadata 和资源中的结构引用，并沿证据路径复核每一个结果。

## Requirements

### Requirement: Structural consumer queries

系统 SHALL 区分 `constant_pool_contains`、`mentions_symbol` 和 `literal_value`，并覆盖已声明的 invocation、field、type、constant、exception、signature、annotation、inner/nest、module、bootstrap 和标准资源 consumer。P1 不实现 `references_definition` 或 `may_dispatch_to`；对这两类请求 MUST 返回 `UnsupportedAnalysis`，同时保留 target relation 与原始 symbol evidence。X1 查询 MUST 不构建 CFG、SSA 或 Java AST。

#### Scenario: Unused constant-pool method reference

- **WHEN** classfile 的常量池包含未被任何 Code consumer 使用的 `Methodref`
- **THEN** X0 可报告该候选，但 X1 不得生成调用引用（验收 A01、A17）

#### Scenario: Metadata-only type reference

- **WHEN** 类型只出现在 Signature、annotation descriptor 或 Code catch type 中
- **THEN** 开启对应 consumer 类别时查询必须命中并保留类别与位置证据（验收 A03）

#### Scenario: Annotation present only in a record component

- **WHEN** 一个注解类型或其 element value 只出现在 Record component 的可见/不可见注解或类型注解中，调用方仅请求 Annotation 类别
- **THEN** 查询命中该 consumer，来源指向对应 component 的嵌套 attribute；不得依赖额外开启 Signature，也不得要求生成的 field/accessor 上有同一注解

#### Scenario: Type annotation present only inside Code

- **WHEN** 类型注解只出现在 Code 内的可见/不可见类型注解 attribute，调用方请求 Annotation 类别
- **THEN** 查询命中并保留方法、attribute span 和适用的 target location；读取嵌套 metadata 不构建 CFG/SSA/AST，不把该位置归入未请求的 Debug 类别

#### Scenario: Descriptor type of an actually used constant

- **WHEN** 类型只出现在实际 consumer 使用的 MethodType、MethodHandle 成员、dynamic site 或其可达 bootstrap descriptor 中，调用方请求 Type 类别
- **THEN** 查询返回 descriptor 中的类型和实际 use-site/CP index/适用 via 证据；原始 descriptor 字符串的 literal 命中不能代替类型引用

#### Scenario: Unused descriptor-bearing pool entry

- **WHEN** 同一 descriptor-bearing CP 或 BootstrapMethods 条目没有实际 consumer 使用
- **THEN** 不产生上述 X1 类型引用；X0 可保留原始候选，不据此生成调用或运行时目标

#### Scenario: Definition relation requested before resolver

- **WHEN** 调用方在 P1 请求 `references_definition` 或 `may_dispatch_to`
- **THEN** 返回 `UnsupportedAnalysis`，保留目标关系和 `mentions_symbol`/raw CP 证据，不进行候选扩展或解析；该能力由 P2 规划

#### Scenario: Deferred bootstrap graph

- **WHEN** 实际 consumer 使用 invokedynamic、dynamic constant 或 MethodHandle
- **THEN** 结果记录 use-site、bootstrap handle 和参数边；未被 consumer 使用的 BootstrapMethods 不得被当作执行引用（验收 A04、A05）

### Requirement: Class candidate classification preserves failure evidence

所有 class consumer SHALL 使用一致的归档候选规则：非目录 entry 的 raw name 以大小写敏感 `.class` 结尾即为候选，包含裸 `.class` 名称；名字只是物理扫描规则，不证明 JVM 可加载性。本阶段的"目录 entry"按归档约定识别，即 raw name 以 `/` 结尾；只带 MS-DOS 目录位而名字不以 `/` 结尾的 entry 作为普通 entry 处理，其被误纳入候选只会多报非 Complete，不会漏掉真实 class。普通不满足该规则的资源可以排除。已纳入范围的候选若 magic 缺失、截断或错误，MUST 返回带物理 origin 的损坏诊断，保留已有可靠结果，并把 query 的 execution 和 artifact structural coverage 标为非 Complete。不得因字节无法构成 class 而静默返回完整无命中。Standalone CLASS 的打开与 root 错误继续遵循 P0 契约。

#### Scenario: Truncated class candidate in an otherwise readable archive

- **WHEN** ZIP 可完整枚举且 CRC/size 校验通过，但 `Broken.class` 仅包含 `CA FE BA` 三个字节，查询请求 class consumer
- **THEN** 返回该 entry 的损坏 diagnostic 与非 Complete execution/coverage，而不是 `items=[]`、空诊断和 CompleteWithinSchema

#### Scenario: Candidate rules agree across scanners

- **WHEN** 同一归档包含合法 class bytes 的 `Good.class`、`Good.CLASS`、裸 `.class` 和普通资源名
- **THEN** code、metadata、bootstrap 对候选范围的判断一致；`Good.class` 和裸 `.class` 被检查，其余项不因某个 scanner 的大小写规则而被单独纳入

### Requirement: Resource and nested consumer coverage

系统 SHALL 扫描 Manifest 的 Main-Class、agent、Class-Path、Automatic-Module-Name、Multi-Release、`META-INF/services` 和 module-info 的 uses/provides，并 SHALL 将框架专用格式留在带规则版本和 coverage 的插件边界内。

#### Scenario: Service provider declaration

- **WHEN** 归档含 `META-INF/services/<service>` 及 provider 名称
- **THEN** 查询返回结构注册关系及来源 entry，但不声称服务已经加载或调用

#### Scenario: Nested JAR consumer

- **WHEN** WAR 或 Boot 布局的 nested JAR 被显式纳入查询范围
- **THEN** 系统在预算内递归读取其 consumer；超出预算返回 Partial/Unsupported 和已处理范围，不返回空结果（验收 A08、A14）

### Requirement: Reference results keep derivation classes and group by owner

引用结果 SHALL 保留既有语义分离：常量池出现（未被任何 consumer 使用的 CP 常量候选）、结构/指令引用（consumer 实际使用的 use-site），以及在给出声明环境时解析到声明的引用，三者 MUST 作为不同发现分别表示，MUST NOT 压平为单一“引用”种类或互相代替。呈现 SHALL 按 owning method 组织结构引用，并保留 relation、derivation、certainty、resolution、evidence、物理位置与 BCI；类级、字段级与 resource 位置命中 SHALL 保留自身位置，MUST NOT 被归入任何方法。分组、排序与过滤 MUST NOT 改变 coverage、execution、诊断或解析状态。

#### Scenario: Constant-pool occurrence stays a candidate

- **WHEN** classfile 常量池含未被任何 Code consumer 使用的 `Methodref`
- **THEN** 结果只给出常量池候选，不被呈现为调用或结构引用，分组与排序不改变其 derivation（验收 A01、A17）

#### Scenario: Structural reference is not a resolved declaration

- **WHEN** 实际调用的 CP owner 为 Sub 而声明在 Base，且调用方未提供运行环境
- **THEN** 结果保留原始符号与结构 use-site，不出现 resolves_to；在显式环境下请求声明引用时，同一 use-site 才可解析到 Base 声明（验收 A11）

#### Scenario: Grouping keeps class-level and resource positions

- **WHEN** 结果同时包含方法体内命中、类级 metadata 命中与 resource 命中
- **THEN** 方法体内命中按 owning method 组织并保留 BCI 与物理方法身份，类级与 resource 命中保留各自位置，不被分配到任何方法（验收 A03）

#### Scenario: Partial scan is not completed by grouping

- **WHEN** 扫描因预算或取消未完成
- **THEN** 已发布分组保留真实 coverage/execution 与未扫描范围，不补造完整分组或补全未决候选（验收 A14）
