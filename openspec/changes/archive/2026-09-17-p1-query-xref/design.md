## Context

以 P0 完成为进入条件。本 change 消费 bounded in-memory snapshot、top-level locator、Header/bytecode 事实和公共 coverage/diagnostic 契约，组织 X0/X1 产品，并把 nested/MR/Boot 明确留在本阶段。当前工作树已包含这些实现，但尚未满足 P1 出口门槛。2026-09-17 的复核与修订见文末：target-bound cursor（3.1）已实施并复核，补齐 consumer 位置（2.2）与 class candidate 错误处理（2.1）按下文顺序实施；各任务的实施状态与验证证据以 `verification.md` 为准，本节只记录契约。

## Goals / Non-Goals

**Goals:**

- 以不可变 snapshot、physical view 和显式 RuntimeView 作为 query 输入。
- 用 noak/rawzip/flate2 适配层读取已有事实，覆盖 code、metadata、bootstrap、resource consumers。
- 在不构建 CFG/SSA/Java AST 的情况下提供 sound structural XRef、分页和可解释的 Partial。

**Non-Goals:**

- 不实现 X2 resolver、X3 复杂值传播、JVM IR 或 Java 输出。
- 不执行 bootstrap、服务、launcher 或用户 artifact；不自动联网解析依赖。
- 不把 P0 的顶层 locator 改写成全局索引，也不引入持久数据库。

## Decisions

1. **查询模型采用关系类型和 consumer schema。** P1 仅以 `mentions_symbol`、`literal_value` 和原始 CP 查询区分语义；`references_definition`、`may_dispatch_to` 保留 target relation 但返回 `UnsupportedAnalysis`，由 P2 resolver 负责。相比单一字符串搜索，这能避免 CP 候选被误报为实际调用。
2. **物理身份使用显式 root/entry location 和有向 origin chain。** standalone CLASS 以 snapshot root 表示；archive class 以真实 entry 表示。Nested origin 逐层记录父容器中产生 child container 的 entry ordinal/raw name 和 child container ID，不再以伪 entry 或只有节点而没有边的 `Vec<ContainerId>` 表示。普通/root MR 变体、版本变体与嵌套来源正交；相同 class bytes 仍不得合并不同物理来源。
3. **请求视图、视图身份与选择事实分离。** PhysicalView 声明 snapshot 和物理范围，RuntimeView 在其上绑定显式 RuntimeProfile 与 LoadDomain；只有 provider 实际派生的结果才能声称某定义被选择。LoadDomain 只记录 loader identity、委派策略、有序 roots、module mode、外部覆盖和 transformation uncertainty，P1 不据此实现 P2 definition/dispatch resolver。
4. **嵌套和布局使用显式 provider 边界。** 公共 `Engine` 的 artifact-tree 入口调用 `artifact` 所有的单线程、无缓存 provider；外层 rawzip entry 产生 child container 和 origin chain，Boot/WAR 规则只生成带 evidence 的物理 layout node，不声称运行时 classpath 已选择。当前只有一个实现，不提前发布 trait；出现第二个真实 provider 时再抽象。
5. **nested entry 从 root snapshot 可复核重放。** Provider 与后续 scanner 都通过 origin chain 逐层重新定位、校验并物化父 entry；中间 STORED/DEFLATED bytes 只计 read/entry 资源，只有调用方最终请求的 entry bytes 计 output。这样不持久化平行 container registry，也不信任 caller 可构造的 ID。
6. **嵌套深度是高水位预算，不是累加 charge。** root container depth 为 0，直接 child 为 1；`nested_depth` 进入 Limits、UsageSnapshot、BudgetDimension 和终止原因，但不进入 CountedBudgetDimension。超过限制时保留可靠前缀并标为 Partial，不能伪装为空容器或 Complete。
7. **MR 请求、物理 evidence 与选择事实分三层。** `RuntimeView` 继续只是请求 identity；公共入口固定为 `Engine::select_multi_release(&ArtifactSnapshot, &RuntimeView, &mut Budget) -> Result<MultiReleaseViewReport>`，从 immutable snapshot 重新取得 ordinary/artifact-tree 物理 evidence。Report 内嵌原样 `EnumerationReport` 或 `ArtifactTreeReport` 作为唯一物理事实源，MR container 只用完整 `PhysicalEntryId` 回指它，并保存 Manifest evidence、logical path、base/versioned/invalid classification、decision/compliance 及 logical outcome；aggregate 与 per-container 都带 coverage/execution。MR domain diagnostic 使用 closed typed code，physical/probe terminal diagnostic 显式包装既有开放 P0 Diagnostic，避免假装收窄共享错误契约。这样 physical provider 已计费的 entry/container/layout 不重复计费，也不会为未返回 layout 消耗隐藏 ResultItems。它不创建 `ResolvedDefinitionId`，不解释 LoadDomain 顺序，也不跨 container 消歧。`SnapshotAll` 只检查 root；`ArtifactTree` 才复用显式 tree provider 检查已建立 child。当前仅开放 library API，CLI 留到 3.2。
8. **标准 MR 激活必须由 Manifest 证明。** `MultiReleasePolicy::Enabled` 只请求规范算法，不强制归档成为 MR-JAR。按 JAR/JDK 行为，root Manifest path、属性名和 `true` 值以 ASCII 大小写不敏感方式识别，raw case 仍保留且 non-canonical path 诊断；只解析主段，named section 不激活，continuation 之外不 trim。Disabled 为 base-only；Custom/Unknown 分别以 `multi_release_custom_policy` / `multi_release_unknown_policy` 的 Unsupported reason Failed，并保留 candidates 为 Unknown。Manifest/path parser 线性、有界；重复 case-equivalent Manifest/主属性或无法可靠解析时产生完整的 Ambiguous/Malformed domain result，不采用 first/last-wins。
9. **选择与合规检查分离。** 合法 `META-INF/versions/N/` candidate 按同 container/logical raw path 的最高唯一 `N <= target` 覆盖 base；winning level 的 duplicate 返回 Ambiguous，遮蔽层的 duplicate 只诊断而不推翻高层唯一 winner。非法版本路径和 versioned `META-INF` resource 不参与选择。Classfile major 上界和 ClassFile `ACC_PUBLIC` 可直接观察的 root predecessor 违规形成 diagnostic，但不改变规范 entry overlay 已选中的事实；Selected 不等于 loadable/linkable/API-compatible。ClassFile flags 不含 `ACC_PROTECTED`，protected nested visibility/InnerClasses 延期；若物理 evidence 含 root/valid-versioned `module-info.class`，本阶段又未解析 exports，则 missing/mismatched public predecessor 按 modular non-exported-package 例外保持 Unknown，而不是误报 definite violation。完整 public API、module 和 verifier 合规留给后续 metadata/module consumer，不得从 `ConformantWithinChecks` 或 diagnostic absence 推导成功。
10. **MR coverage 单独使用 runtime-resolution 维度。** Artifact provider 的 structural coverage 原样保留；selection 对真实 container/entry ordinal 标记 scanned/skipped。任何可能隐藏更高版本或 duplicate 的 partial suffix 都使该 container 已见 logical path 的选择为 Unknown；已完整扫描得到的 Ambiguous/Inactive/NonConformant/Manifest-Unknown 是完整 domain result，不等于 Partial。合规 probe 在 path selection 已确定后失败时只把 compliance 标为 Unknown，不回退选择。Manifest/class 内部物化沿 root origin 重放并计 read/entry/class，不计 caller output；provider physical items 与新增 MR evidence 分别只计一次 `result_items`。预算/取消的 terminal reason 保持原始维度，ordinary diagnostic 计 `result_items`，必要 terminal diagnostic 延续控制元数据例外。
11. **MR provider 有独立所有者但不提前抽象 trait。** 新 `multi_release` 模块拥有 Manifest parser、raw-path classifier、selection/compliance 状态机和 report types，单向依赖 `artifact`/`view`/`model`/`classfile`；`artifact` 只补 crate-private 的 verified internal materialization accounting，`classfile` 只补返回 major/minor/access/this_class 的 crate-private minimal probe，二者都不能反向依赖 selection。Minimal probe facts嵌入最终 entry evidence，不构造/计费未返回的 Header report，也不遍历未请求 attribute/code。`Engine` 仅作薄委托，现阶段不增加缓存、registry、trait 或外部依赖。
12. **bootstrap 使用 deferred graph。** 先记录 dynamic use-site，后续按实际 consumer 补 bootstrap 边，配合 visited/depth/edge 预算。相比一次性展开所有 bootstrap，结果不会重复爆炸，也不会把未使用节点当调用。
13. **结果采用 immutable page + coverage。** 每页绑定 snapshot/view fingerprint 和 continuation token，所有终止状态走 P0 diagnostics。普通 entry/container/layout/重复项 diagnostic 计入 `result_items`；解释预算耗尽、取消或结构终止所必需的 terminal diagnostic 延续 P0 约定，属于控制元数据，即使 `result_items` 已耗尽也必须返回且不再计费。相比返回 `Vec` 加 bool，调用方能区分 NoMatch、Unknown、Partial 和 Complete。
14. **依赖只复用 P0 选择。** noak、rawzip、flate2、blake3、serde、thiserror、clap 通过内部 adapter 使用；是否增加任何库必须由后续证据和审计单独决定，本阶段不预设新依赖。

15. **CP 事实面是 crate-private reader 能力，不扩张 P0 公共契约。** X0/X1 需要按 index 解析常量池条目（符号、字面量、引用边、字节 span），但已生效的 `classfile-inspection` 主规格只承诺 Header 结构、attribute shell、指令 span 与适用的 CP index。P1 在 `classfile` 内新增 crate-private 事实视图（CP 条目 + 目标 attribute 内容，经由既有 noak adapter 与预算检查），公共 API 不变，因此不需要修订已归档主规格；暴露给调用方的是 query 结果里的符号与证据，而不是 CP 表本身。这与 1.3 的 `probe_minimal_header` 先例一致。
16. **`UnsupportedAnalysis` 是结果顶层的分析状态，不是 execution 状态。** `QueryReport.analysis = Performed | UnsupportedAnalysis { relation }`：P1 对 `references_definition`、`may_dispatch_to` 返回后者，同时仍返回目标关系与保留的 raw symbol/CP 证据，且不产生任何 resolution 字段。不向 P0 共享的 `TerminationReason` 新增 producer-owned unsupported code，也不用 `Failed` 表达"该关系本阶段不分析"；`execution` 描述本次扫描实际做了什么（可为 Complete），`Unknown/Unsupported` 与 execution 分属不同枚举。诊断以 query-owned code `query_relation_unsupported` 记录。此时 `coverage.artifact_structural` 只证明本次 raw CP 探针的覆盖，不证明请求的 consumer schema 已全扫，更不证明 declaration/dispatch 已解析。
17. **consumer 类别沿用 1.1 已交付的 `ConsumerKind`，不新增 variant。** hierarchy（super class/interfaces）recognition 以 `ConsumerKind::Type` + `XrefOperation::SuperClass|Interface` 表达，而不是新增 `Hierarchy` 类别；`Verification`/`Debug` 在 P1 不实现（P0 不解码 StackMapTable/LVT），被显式请求时进入 `QueryCoverage.unsupported_categories`，且不得对它们声明 complete-within-schema。这样既不推翻已验证的公共枚举，也不产生"声称已覆盖实际未扫描"的假阴性。
18. **X1 扫描范围只由 `PhysicalView` 决定，`RuntimeView` 不参与过滤。** MR/loader/domain 条件不用于裁剪 items（过滤即可能造成假阴性），MR 选择事实仍由 `select_multi_release` 单独提供并按 `PhysicalEntryId` 关联；query 结果的 `runtime_resolution` 在 P1 保持 `NotRequested`。
19. **符号匹配以原始字节精确相等为准。** owner 为内部名原始字节，不做归一化（数组与对象类型 owner 不同、大小写敏感）；当前 `SymbolRef` 的 descriptor 仅精确匹配，未实现 `Any`，本轮不增加通配查询。`literal_value` 的字符串/类字面量用 raw MUTF-8 字节，整数/长整型用 JVM 值，float/double 用位模式以避免 NaN 语义歧义。过滤器只允许假阳性：无法安全判定的候选标 `XrefCertainty::Unknown` 并继续扫描，不因过滤跳过。
20. **query 结果的计费与 MR 同纪律。** 每个返回 item = 1 `ResultItems`，每个 Domain diagnostic = 1；terminal diagnostic 属控制元数据不计费且必须返回；envelope/coverage/page/analysis 字段为 0；扫描按既有维度计费（`ClassBytes`/`AttributeBytes`/`CodeBytes`/`ReadBytes`/`EntryBytes`/`ArchiveEntries`），`OutputBytes` 只在 CLI 层计。预算耗尽保留可靠前缀，不构造未计费 item。
21. **cursor 是公开结构化值并绑定完整查询身份。** `QueryCursor { engine_schema, snapshot, physical, relation, target, consumers, boundary, digest }`（3.1 已实施），完整 `QueryTarget` 进入等值校验与既有 length-prefixed blake3 digest；编码包含所有 variant tag、原始字节与整数/浮点位模式，不哈希显示字符串。`QUERY_ENGINE_SCHEMA` 已随 3.1 从 1 升到 2，不保留旧游标兼容。`boundary` 记录已发布到的容器、unit ordinal 与该单元内按目标过滤后的已发布序号（不是字节 offset）；快照/视图/关系/target/schema 任一失配返回 `query_cursor_mismatch`。max_items 和预算可变，不进入语义身份。`page.has_more` 与 `execution` 独立；Partial/Cancelled 也可带游标，重放时只跳过同一查询已发布的前缀。复用现有 hash 与值类型，无需游标 registry、新 token 库或并行缓存。
22. **module 事实由 classfile 内部提供，资源 consumer 在 query 层实现。** `module-info.class` 的 uses/provides 需要 module attribute 内容，归 crate-private reader；Manifest 属性（Main-Class、agent、Class-Path、Automatic-Module-Name、Multi-Release）与 `META-INF/services/*` 属 query 层的 resource consumer；框架专用格式（spring.factories 等）留到 P4 plugin 边界。service provider 声明只表示结构注册关系，不表示已加载或已调用。
23. **bootstrap/condy 用共享 deferred 图。** 只有被实际 consumer 使用的动态点才展开；图节点以动态 CP index 标识并去重，边记录 bootstrap handle 与实参位置，item 携带 `via` 路径；visited/深度/边数预算与 cycle 诊断独立于容器 `nested_depth`；未使用的 `BootstrapMethods` 项不得作为引用输出。`via` 的 `argument_index` 是**静态实参槽位**（JVM 自行提供 `Lookup`、名字与站点描述符派生的 `Class`/`MethodType` 三个前置实参，它们不出现于表中也不出现于 `via`）；`CONSTANT_MethodType` 实参的描述符经 `LiteralValue` 通道发布（键为原始 descriptor 字节，`evidence.constant_pool_index` 指向被描述的节点条目，use-site 索引只在 `via[0]`），因为冻结 schema 没有 descriptor target。
24. **模块与写入所有权（并行实施边界）。** `classfile`（reader 事实，唯一 owner）→ `xref`（扫描编排与各 consumer 子模块，按文件分 owner）→ `query`（请求/结果/分页/关系分派）→ `engine`（薄委托）→ `jarde-cli`（3.2）。共享契约（`query.rs` 的请求/结果类型、`xref` 的 item 证据类型、cursor/page）由单一 owner 先落地，consumer 子模块不得各自新增公共类型或修改公共 schema；`artifact`/`multi_release` 不被 xref 修改。
25. **Annotation 的完整性按标准位置判断。** 除 class/field/method，扫描 Record component 内的 annotation/type-annotation 与 Code 内的 type-annotation。用已有 reader 的有界 attribute span/长度检查进入嵌套内容，Annotation-only 请求独立生效，不借 Signature 开关。保留 nested attribute/class offset 与适用的 method/BCI/target location；读取 Code 内 metadata 不要求 CFG、指令语义或 debug 表。未知自定义 attribute 与已知标准位置未覆盖分开，后者不能声称 CompleteWithinSchema。嵌套位置以"一条注解结构"为单位落位与发布：结构解析失败时该结构不发布任何事实（因此也不计 `ResultItems`），同一 attribute 内此前已完整结束的结构仍按各自位置发布；未请求的嵌套 attribute 内容不解析、不产 item，但声明与字节范围矛盾仍会停止扫描。
26. **descriptor 类型从实际使用点产生。** 复用 descriptor 解析能力，由 code/bootstrap 的实际 use-site 触发对 MethodType、MethodHandle 成员、dynamic site 及可达 bootstrap descriptor 的类型提取；以 `ConsumerKind::Type` 和原有 use-site operation（如 Ldc、InvokeDynamic、BootstrapArgument）发布，保留 CP index 与适用的 via。Type-only 请求须触发所需读取，不能隐式依赖同时请求 Invocation/Constant/Bootstrap；未使用 CP/BootstrapMethods 不生成 X1。原始 descriptor literal 可以保留，但不能代替 `SymbolRef::Class` 类型项。不扫描全池来伪造 consumer，也不增加 resolver 或执行 bootstrap。deferred 图的请求门槛是类别集合含 `Bootstrap` 或 `Type` 之一，且仍只在存在真实 use-site 时打开：`Type`-only 请求会读表并遍历该图以发布带 `via` 的 descriptor 类型，而节点自身的 symbol/literal 事实仍只属于 `Bootstrap` 类别；没有 use-site 时任何类别都不读 `BootstrapMethods`，未使用表项不产出事实。
27. **class candidate 规则由扫描编排层共享。** 在 `xref` 内统一非目录 raw name 的大小写敏感 `.class` 后缀判断，裸 `.class` 也进入候选；不增加 provider 抽象。候选的错误/不足 magic 是带 entry provenance 的解析失败，使用 query-owned `query_class_candidate_malformed` diagnostic，保留前缀及非 Complete execution/coverage。当前 fail-stop 编排可以继续使用，未处理 siblings 必须进入 skipped；本轮不另做失败隔离调度。普通资源排除与损坏候选失败分开，三类 class scanner 必须遵循同一规则。

## 公共 schema 骨架（2.1–2.4、3.1 的唯一契约来源）

以下定义为实施契约；命名、字段、variant 与 serde tag 不得由实现者各自改动。新增公共类型或 variant 必须先改本节与对应 spec。`QueryCursor.target` 由 3.1 实施，因此 `engine_schema = 2`。

```rust
// src/query.rs 追加（共享契约 owner：3.1）
pub struct QueryRequest {
    pub relation: QueryRelation,
    pub target: QueryTarget,
    pub physical: PhysicalView,
    pub consumers: ConsumerSchema,
    pub max_items: u64,          // 0 = 本页不设条目上限（仍受预算约束）
    pub cursor: Option<QueryCursor>,
}

pub enum QueryTarget {
    Symbol { value: SymbolRef },
    Literal { value: LiteralValue },
}
pub enum LiteralValue {                                 // 内部 tag + value 字段：serde 内部 tag 不能承载 newtype variant
    String { value: JvmBytes },
    Class { value: JvmBytes },
    Integer { value: i32 },
    Long { value: i64 },
    Float { value: u32 },                               // 位模式
    Double { value: u64 },                              // 位模式
}

pub struct QueryReport {
    pub physical: PhysicalView,
    pub relation: QueryRelation,
    pub consumers: ConsumerSchema,
    pub analysis: QueryAnalysis,
    pub items: Vec<XrefItem>,
    pub page: QueryPage,
    pub coverage: QueryCoverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

pub enum QueryAnalysis { Performed, UnsupportedAnalysis { relation: QueryRelation } }
pub struct QueryPage { pub has_more: bool, pub returned_items: u64, pub cursor: Option<QueryCursor> }

pub struct QueryCursor {
    pub engine_schema: u16,
    pub snapshot: SnapshotId,
    pub physical: PhysicalView,
    pub relation: QueryRelation,
    pub target: QueryTarget,                         // 待实施：完整目标绑定，engine_schema = 2
    pub consumers: ConsumerSchema,
    pub boundary: QueryBoundary,
    pub digest: Digest,
}
pub struct QueryBoundary { pub container: ContainerOrigin, pub ordinal: u64, pub item_index: u64 }

pub struct QueryCoverage {
    pub dimensions: Coverage,                        // P0 三维；P1 runtime_resolution = NotRequested
    pub consumer_schema: ConsumerSchema,
    pub unsupported_categories: Vec<ConsumerKind>,
    pub scanned_items: u64,
    pub unknown_candidates: u64,
}

pub struct XrefItem {
    pub relation: QueryRelation,
    pub source: Provenance,
    pub target: XrefTarget,
    /// `None` 仅用于 `XrefDerivation::ConstantPoolCandidate`（X0 原始池条目没有 consumer 类别）。
    pub consumer: Option<ConsumerKind>,
    pub operation: XrefOperation,
    pub derivation: XrefDerivation,
    pub certainty: XrefCertainty,
    pub resolution: QueryResolution,
    pub evidence: XrefEvidence,
}
pub enum XrefTarget { Symbol { value: SymbolRef }, Literal { value: LiteralValue } }
pub enum XrefDerivation { StructuralConsumer, ConstantPoolCandidate, BootstrapEdge }
pub enum XrefCertainty { Exact, Unknown }
pub enum QueryResolution { NotRequested }             // P1 固定；P2 增 variant 需先改 spec
pub struct XrefEvidence {
    pub constant_pool_index: Option<u16>,
    pub bci: Option<u32>,
    pub opcode: Option<u8>,
    pub attribute: Option<ArchiveNameBytes>,          // attribute 名原始字节
    pub span: Option<ByteSpan>,                       // class 文件坐标系
    pub via: Vec<BootstrapVia>,                       // 仅 bootstrap/condy 边
}
pub struct BootstrapVia { pub constant_pool_index: u16, pub bootstrap_index: Option<u16>, pub argument_index: Option<u16> }

pub enum XrefOperation {   // 闭合；P1 实现集合
    ConstantPoolEntry,
    InvokeVirtual, InvokeSpecial, InvokeStatic, InvokeInterface, InvokeDynamic,
    GetField, GetStatic, PutField, PutStatic,
    New, NewArray, MultiNewArray, CheckCast, InstanceOf,
    Ldc, ConstantValue,
    ExceptionHandler, ExceptionsAttribute,
    SuperClass, Interface, FieldDescriptor, MethodDescriptor, GenericSignature, RecordComponent,
    Annotation, TypeAnnotation, AnnotationDefault,
    InnerClass, EnclosingMethod, NestHost, NestMembers, PermittedSubclasses,
    ModuleUses, ModuleProvides,
    BootstrapMethod, BootstrapArgument,
    ManifestMainClass, ManifestAgent, ManifestClassPath, ManifestAutomaticModuleName, ManifestMultiRelease,
    ServiceProvider,
}
```

入口：`Engine::query(&ArtifactSnapshot, &QueryRequest, &mut Budget) -> Result<QueryReport>`（薄委托到 `query::execute`）。稳定错误码：`query_snapshot_mismatch`、`query_artifact_tree_root_mismatch`、`query_cursor_mismatch`、`query_target_relation_mismatch`（InvalidInput）；`query_consumer_schema_version`（Unsupported，非 1 的 schema 版本）。诊断码：`query_relation_unsupported`（`UnsupportedAnalysis` 时记录）。

模块与写入所有权（并行边界，owner 唯一）：

| 文件 | 内容 | owner |
| --- | --- | --- |
| `src/classfile.rs` | crate-private CP 事实与目标 attribute 内容解码 | R |
| `src/query.rs` | 上述公共类型 + 请求校验/关系分派/分页与 coverage 组装 | D |
| `src/xref/mod.rs` | 扫描编排、item 计费与页装配、unit 遍历 | D |
| `src/xref/resource.rs` | Manifest 属性与 `META-INF/services` consumer | D |
| `src/xref/code.rs` | 2.1 CP 候选与 code consumer | A |
| `src/xref/metadata.rs` | 2.2 metadata consumer | B |
| `src/xref/bootstrap.rs` | 2.3 deferred bootstrap/condy 图 | C |
| `src/engine.rs` | `Engine::query` 薄委托 | D |

`mod.rs` 一次声明全部子模块与统一签名 `pub(super) fn scan(ctx, unit, out) -> Result<()>`；各 stream 只填自己的文件，不得改 `mod.rs` 的公共类型、签名或 `artifact.rs`/`multi_release.rs`。

## Risks / Trade-offs

- [Risk] 公共 schema 变化被多个并行 consumer 子模块各自扩张 → 共享契约单一 owner，新增公共类型或 variant 必须先改本 design 与 specs。
- [Risk] 复杂 Boot 变体或第三方布局无法安全识别 → 保留 candidate、规则版本和 Unsupported，而不是猜测。
- [Risk] consumer 类别扩张导致扫描成本上升 → 每类计数、预算和分页；默认不读无关 Body。
- [Risk] 多层 nested/condy 图出现爆炸 → visited set、深度/边数上限和共享 graph node。
- [Risk] loader/order 语义不完整 → 返回候选集合与 open-world/ambiguous 状态，不伪造唯一解析。

## Migration Plan

实现前以 P0 的 snapshot、entry、Header、diagnostic API 为输入基线。P1 规划完成后用 A01–A08、A14、A17、A18 样本验证，再由下一阶段消费；若契约需要变更，必须通过 OpenSpec 显式修订，不作旧接口向后兼容承诺。

## 2026-09-17 复核与推进顺序

本节是当前实施计划的依据；`verification.md` 保留此前各轮的历史结论，其中“留待后续、不阻塞”的口径不能覆盖本节已复现的正确性问题。复核基线为 `ee0a3a16b618d923f2e7aef012bdf925e1d62295` 加接手时的未提交工作树，并非该 commit 本身。复核范围是 query/cursor、xref 编排及 consumer、MR 与 tree 的契约/终止状态、CLI、对应测试和 OpenSpec；不是对整个引擎的无遗漏证明。Atlas 的 scoped search 确认 `src/query.rs::execute` 和 `src/xref/metadata.rs::annotation_shells` 的局部结构；结论以源码读取和公共 CLI 反例为主。

### 已复现的问题

| ID / 优先级 | 位置与原因 | 公共入口实测 | 负责与退出条件 |
| --- | --- | --- | --- |
| R1 / P1 | `src/query.rs::validate_cursor`、`cursor_digest` 与 `src/xref/mod.rs::build_cursor` 均未包含 target；boundary.item_index 是过滤后的序号 | 单 Manifest 中 A 有 2 项、B 有 2 项。先取 A 的 1 项游标，再查 B，返回 1 项；全新 B 查询返回 2 项。错误续页仍为 Complete / CompleteWithinSchema、空诊断 | 重新打开 3.1；跨目标游标在扫描前拒绝，同目标续页和可变页大小与全量结果等价；库与 CLI 各有反例 |
| R2 / P1 | `src/xref/metadata.rs::annotation_shells` 只枚举 class/field/method；`scan_record` 只解释 Signature | 真实 javac 生成的 Record component-only 注解及 Java 8 Code 内 CAST 类型注解，各自仅请求 Annotation 时均为 0 项 / Complete / CompleteWithinSchema、空诊断、无 unsupported category | 重新打开 2.2；覆盖 visible/invisible、record/Code 两种嵌套位置，Annotation-only 和组合请求均通过，证据必须指向实际位置 |
| R3 / P1 | `src/xref/code.rs::load_use` 将 MethodType descriptor 的类型交给其他 consumer，但 metadata 没有对应实际使用点的入口 | Java 8 结构样本以 `ldc #9; pop; return` 使用 MethodType `(Lp/OnlyParam;)Lp/OnlyResult;`；Type/Signature/Constant/Bootstrap 联合查询 `p/OnlyResult` 仍为 0 项 / Complete / CompleteWithinSchema、空诊断 | 2.2 统筹 reader 与 code/bootstrap 适配；Type-only 命中，未使用相同 CP 条目不命中，MethodHandle/dynamic/可达 bootstrap descriptor 做同类对照 |
| R4 / P1 | `code::class_content`、metadata/bootstrap 的 magic 检查对损坏候选返回 Ok 空结果；metadata 后缀规则另与两者不同 | CRC/size 正常的 ZIP 中 `Broken.class = CA FE BA`，请求 Invocation+Annotation，得到 0 项 / Complete / CompleteWithinSchema、空诊断 | 重新打开 2.1；损坏候选给出可定位诊断及非 Complete，三类 scanner 的候选规则一致，普通非 class resource 不误报损坏 |

这些问题违背已声明 consumer 不漏报及 I10“失败不能伪装为否定”，不能归入 P5 性能优化。R2 中 Code 类型注解属于 Java 8，也不能作为 P4 现代恢复延期。Record/Code 的标准 attribute 位置见 [JVMS 4.7.16](https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-4.html#jvms-4.7.16)、[4.7.20](https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-4.html#jvms-4.7.20)、[4.7.30](https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-4.html#jvms-4.7.30)；MethodType 保存方法 descriptor 的结构见 [JVMS 4.4.9](https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-4.html#jvms-4.4.9)。类型查询覆盖边界同时来自本项目架构 §9.3/9.6，不把规范结构描述当作本项目已实现的能力。

2.2 实施期间另确认同 consumer 的第三类假失败并纳入本轮修复：方法 `Signature` 的 `Result` 与 `ThrowsSignature` 曾按 descriptor 产生式解析，合法 javac 23.0.1 输出（`()Ljava/util/List<Ljava/lang/String;>;`、`(TT;)TT;`、`()V^TE;`）会使请求以 `query_signature_malformed` 失败、返回 0 item 且非 Complete。修复按 JVMS 4.7.9.1 分离签名与 descriptor 两条语法路径并解析 throws 序列（`^` 后的 class type 产生 `GenericSignature` item，type variable 不产生）；反例与证据见 verification 的修复轮 R2。

### 反例复现材料

本轮反例通过当前 `target/debug/jarde-cli` 的 JSON stdin 公共入口执行；所有十一项 limit 均为 `100000000`，scope 为 `snapshot_all`、relation 为 `mentions_symbol`，因此这里的空结果不是小预算导致。未执行生成的 Java class，仅编译并用 `javap -v` 核对 attribute/CP/指令位置。下一轮须将这些反例固化到现有 integration tests；临时目录不作为长期验收依赖。

- **R1**：创建 STORED JAR，其唯一 `META-INF/MANIFEST.MF` 主段依次为 `Manifest-Version: 1.0`、`Main-Class: p.A`、`Premain-Class: p.A`、`Agent-Class: p.B`、`Launcher-Agent-Class: p.B`，CRLF 并以空行结束。以 Resource 类别、`SymbolRef::Class(owner=p/A)`、max_items=1 取得游标，保留游标改成 `p/B`、max_items=0；应拒绝，实际漏掉 B 首项。
- **R2 record**：`RecordOnly.java` 内容如下，用 `javac 23.0.1 --release 17 -g:none -d OUT RecordOnly.java`；仅查询 `RecordOnly.class` 中 `RecordMarker`，不要把注解自身 class 纳入搜索。输出 class SHA-256：`11342150dff8bd6dbe0d7270c0d7c89ee736424da86cdfcdb020ed09f636e293`。

```java
import java.lang.annotation.*;
@Retention(RetentionPolicy.RUNTIME) @Target(ElementType.RECORD_COMPONENT)
@interface RecordMarker {}
record RecordOnly(@RecordMarker int value) {}
```

- **R2 Code**：`CodeOnly.java` 内容如下，以同一 javac 的 `--release 8 -g:none` 编译，仅查 `CodeOnly.class` 的 `CodeMarker`。`javap -v` 确认 `RuntimeVisibleTypeAnnotations` 位于 `f` 的 Code 中。输出 SHA-256：`8aabf4f4c3314a42d24d7077583597ecb68cc85b0d7d23ba7dfa2a05e982ce29`。不同编译器产物需另记摘要，不冒充同一输入。

```java
import java.lang.annotation.*;
@Retention(RetentionPolicy.RUNTIME) @Target(ElementType.TYPE_USE)
@interface CodeMarker {}
class CodeOnly { Object f(Object o) { return (@CodeMarker String)o; } }
```

- **R3**：手工结构 fixture，major/minor=52/0；CP #1..#9 依次为 Utf8 `MethodTypeOnly`、Class #1、Utf8 `java/lang/Object`、Class #3、Utf8 `probe`、Utf8 `()V`、Utf8 `Code`、Utf8 `(Lp/OnlyParam;)Lp/OnlyResult;`、MethodType #8。class flags=0x21，唯一 static 方法 probe flags=0x9，max_stack=1/max_locals=0，Code 为 `12 09 57 B1`，无 handlers 或其他 attributes。SHA-256：`d7c33b7b8ec9247da93dea44a930a963bd4ec73f6e1f4d86561bc53e7d93982e`。`javap -v` 可读取其 CP 与指令；本轮没有证明该样本完成 JVM linkage/verification。
- **R4**：STORED JAR 仅含 `Broken.class`，内容三个字节 `CA FE BA`，由正常 ZIP writer 生成匹配 CRC/size；查询任意类符号并请求 Invocation+Annotation。另做错误 magic、0–3 字节截断、`Good.class`/`Good.CLASS`/裸 `.class` 对照。

### 本轮验证的实际边界

确定性核验使用 `CARGO_BUILD_JOBS=1`、`CARGO_INCREMENTAL=0`、`RUST_TEST_THREADS=1`：fmt 通过；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`cargo test --workspace` 为 **250 passed / 0 failed / 1 ignored**；`openspec validate --all --strict --no-interactive` 为 **8 passed / 0 failed**；`git diff --check` 通过。现有 tests 全绿与 R1–R4 反例失败并存，表明当前测试未覆盖这些契约。

本轮没有重跑双固定种子、`--all-features --locked` 全量门禁、ignored JDK 25 oracle、MSRV、supply-chain 或远端 CI；不能继承前一 agent 的运行结果作为本轮独立执行记录。当前代码不作任何修复，原 staged/unstaged 实现和仓库卫生改动保持原状。

### 3.3 的验收包

先关闭 R1–R4，再整理以下已有测试和缺口。统一索引直接记录在现有 `verification.md`，fixture 的来源说明与稳定期望值放到 `tests/fixtures`，不建立新的 corpus 管理框架。内存构造样本要记录 generator/test 名及构造版本、生成 bytes 摘要；真实编译样本记录源文件、编译器、命令、摘要与 dialect。Golden 可以用普通 JSON 和现有 serde_json 比较，不为快照引入额外依赖。

| 验收 | 复用的现有证据 | 还需补齐的组合/断言 |
| --- | --- | --- |
| A01/A02/A17 | `tests/p1_xref_code.rs`：未使用 Methodref、调用位置、死代码引用 | X0/X1 双侧 golden、完整 owner/name/descriptor/BCI/opcode/CP index；生产模块与依赖不引入 CFG/SSA/AST，不能只检查无源码输出 |
| A03 | `tests/p1_xref_metadata.rs` | R2/R3、类别单独/组合对照、不经过 CONSTANT_Class 的类型；正例断言 Complete+coverage+diagnostics，反例断言明确未覆盖而非仅空 items |
| A04/A05 | `tests/p1_xref_bootstrap.rs` | 真实 javac lambda 与现有结构样本区分；MethodType/dynamic descriptor 类型、未使用表项、共享/循环图、各 use-site via；有限时间成功不代表 bootstrap 可链接 |
| A06/A07 | `tests/p1_multi_release.rs`、`tests/p1_query_model.rs` | MR Java 8/11/17 selection 与同一 physical query 并列；WAR/嵌套中同名同字节的多个真实 XRef origin，不只检查值类型；不跨 container 静默合并 |
| A08/A14 | `tests/p1_artifact_tree.rs`、`tests/p1_query_api.rs`、`crates/jarde-cli/tests/query_cli.rs` | nested STORED/DEFLATED + 截断 class + sibling + 页边界/预算/取消组合；R4；保留 terminal dimension、可靠前缀与 skipped，不以页 has_more 推导全范围完成 |
| A18 | `src/artifact.rs`、`tests/engine.rs` 的快照 seam 和 query snapshot mismatch | 同快照在源路径内容变化后仍稳定；重新 open 的 snapshot 拒绝旧游标；新增 R1 跨 target/同 target 分页等价，独立记录到验收索引 |

性质测试继续使用已锁定的 proptest 1.11.0：固定两个种子，检查分页拼接等价、consumer 类别组合的一致性、未使用 CP 插入不制造 X1、相同 bytes 不同 origin 不合并。Normalize 只允许剔除 elapsed 等非语义运行值；origin、coverage、diagnostic、via、完整 symbol 和 item 顺序不能为了使 golden 通过而删除。

仓库当前没有 fuzz target，reader 的 proptest 不能替代这项。3.3 选用 [cargo-fuzz/libFuzzer](https://rust-fuzz.github.io/book/cargo-fuzz.html) 的独立 test-only workspace，建立 bounded query 与 MR/artifact-tree 两个入口；输入同时来自最小有效 corpus 和任意损坏 bytes。复用现有 reader/budget，不自写 mutation engine。按 [cargo-fuzz 上游说明](https://github.com/rust-fuzz/cargo-fuzz) 固定其工具/运行库版本和 nightly，CI 单独准备 C++/sanitizer 工具链；不得进入纯 Rust 生产依赖或改变主 workspace 的 MSRV。实现时核验工具依赖的许可证/advisories 并保存锁文件和来源。

每个 fuzz target 的收口 smoke 至少单 worker 60 秒，限制 max_len=65536、rss_limit_mb=512，并给引擎所有预算明确的小上限；限制值及实际命令入档，不把这段运行写成全面安全证明。门槛是不 panic、不越过公开预算、损坏或中断不假 Complete、可稳定重放；发现失败先最小化并加入确定性回归。属性失败同样保留 seed。扫描编排现在先累积 unit_items 再计 ResultItems，3.3 必须用重复引用/高扇出样本检查这一缓冲和取消边界；若证实结果预算不能约束构造，作为 I12 正确性修复，不等待 P5 缓存优化。

### 3.4 的退出门槛与提交边界

1. 按 `tasks.md` 重新验收 2.1、2.2、3.1 及 3.3；复核者只读检查真实反例和修复，不能仅认可新增测试名称或通过总数。
2. 同步根 README、`docs/support-matrix.md`、`openspec/README.md`、roadmap、acceptance、当前 proposal/verification。它们现有“仅完成 1.1/1.2”“X1 全部 NotImplemented”等陈述已落后；矩阵要区分实现存在、验证通过、已知未覆盖和不支持，parse/X1/resolution/decompile-quality/output-level 分列。
3. 使用 `.github/workflows/ci.yml` 的完整参数及两个 seed `5350648285461741569`、`5350648285461741570`；另跑 ignored JDK 25 oracle、公共示例、feature/normal dependency tree、Rust 1.88.0 MSRV、supply-chain 和新增 fuzz smoke。所有 Cargo 保持单作业。记录命令、工具版本、退出状态及测试数。
4. 提交按可编译的依赖闭合边界，而不是历史任务编号机械切块：若 MR 1.3 能独立构建则先提交；共享 reader + xref/query/CLI 若不能再分就一起提交；R1–R4 回归修复和验收文档分别保留可复核边界。`.atlas/*`、`.agents/skills/.openspec-target` 的取消跟踪与 `.gitignore` 属独立仓库卫生提交，不删除本地数据。本轮不执行提交或推送。
5. 最终候选推送后，将对应 SHA 与 CI run 的 stable/MSRV/supply-chain/fuzz 状态写入 verification。1.1/1.2 的旧成功 run 不证明当前工作树。最终 checks 通过后才勾选 3.4、archive 并同步主 specs，再进入 P2；planning complete 不替代这一步。

### 拆分处理的后续项

| 项目 | 后续归属与触发条件 |
| --- | --- |
| 同一 unit 多次物化、重复 read/entry/class/attribute 计费 | P5 测量基线后处理；当前只如实记录提前 Partial，不为减少读次数引入缓存。Standalone root 的内部读取还计 OutputBytes，与决策 20 的目标不同，支持矩阵须披露当前行为，账目纠正可独立小变更 |
| classfile 事实层冗余 allow(dead_code) | 单独卫生改动；不和 consumer 正确性混合，不因未做此清理阻止 P1 |
| MR/query 与 tree 的 aggregate priority 不同 | 单独分析 caller 语义；tree 把 NestedDepth 当局部失败，不能仅因数字不同就共用 helper。若组合反例证明真实 budget/cancel 被遮蔽，再作为当前阻塞项处理 |
| tree 从未 pop 的已建立 child 只存在聚合 coverage | 保留物理 Partial 事实；provider 增加 pending-container evidence 需独立修订相应契约，不能在 MR 内伪造 per-container ordinal；3.3 验证上层没有误报完整选择 |
| `has_more = true` 且 `cursor = null`（续页或取消在发布任何新项前被中断，含损坏候选使续页永远无法越过该 entry） | R1/R4 复核发现的既有契约缝：调用方无法区分"没有更多"与"更多但需重试同一请求"。当前行为已被 A14/取消用例钉死且不违反决策 21；要改成回传输入游标需先修订 query-api spec 的 page 契约，属独立小变更 |
| `Location::Entry.span` 的口径分歧 | xref 的 terminal 诊断用 `(0, uncompressed_size)`，tree/MR 诊断用 entry 的 `layout.compressed_data`。entry id 与 origin chain 已足以定位，不影响"真实 origin"要求；统一坐标含义属独立小变更（记入 model/Location 文档） |
| record component 的描述符类型走 `Signature`/`RecordComponent`，字段与方法描述符走 `Type`/`FieldDescriptor`/`MethodDescriptor` | R2 复核发现的既存类别归属不一致：同一个"声明类型"语义分属两个 consumer 类别，容易让后续实现者误判。改变既有 item 的类别字段须先修订 structural-xref spec 并同步受影响断言，属独立小变更 |
| `Type`-only 请求会解两遍方法体（code 找 descriptor use-site、bootstrap 找 dynamic use-site），紧 `code_bytes` 预算下比 code-only 更早返回 Partial | 决策 26 让 deferred 图对 `Type` 开放的直接代价，与"同一 unit 多次物化"同族：结果仍是如实更早 Partial，不是漏报。压回成本需要 code/bootstrap 共享一次 body 解码，属 P5「按实测决定的缓存/索引」 |
| 结果预算不约束构造：编排先累积 `unit_items` 再计 `ResultItems` | 3.3 用高扇出样本（单 entry 2048 条命中）证实：`result_items=16` 时整段 body 已解码（`code_bytes=6145`）后才停。构造仍被输入维度预算约束（每条 item 对应已计费字节：invoke 3 B、`ldc` 2 B、注解/描述符类型 ≥3 B、CP 候选 ≥1 B；`XrefItem` 392 B），不存在与预算无关的放大，故**不作为 I12 修复**；若后续出现纯内存放大（例如聚合缓存）需重新评估 |
| 性质生成器形状未覆盖 annotation/Bootstrap/Record | `tests/p1_xref_properties.rs` 的生成类只有 `Code` 体与未使用引用，类别交互（如 Type×Bootstrap、Annotation×Signature）由确定性用例覆盖而非性质测试；扩展形状属独立小改动，不阻塞 3.3 |
| CI `fuzz-smoke` 的 `-rss_limit_mb=512` 余量偏紧 | 本地实测 `artifact_tree` peak RSS 418–463 MB（ASan），余量约 10%；若该 job 在 Linux runner 上抖动，先记录实测 RSS 再决定调整限值或缩短运行时间，不静默放宽 |
| 性质 P1 在"合法停止页"分支只断言较弱的等价性 | `tests/p1_xref_properties.rs` 对 `cursor = null` 且 `execution != Complete` 的分支只断言 `has_more` 与"已发布前缀等于整跑前缀"（当前生成器/限值下不可达，全 splice 等价仍每例执行）；若将来某形状恒停在停止页，整轮等价性由 `p1_query_bounds.rs`/`p1_query_api.rs` 的确定性用例承担 |
| cursor digest 是无密钥的公开哈希 | 客户端可自行重算并篡改 boundary，只影响自己这次请求、不改变引擎 soundness；决策 21 明确不引入 registry 或 HMAC，这里只记录边界，P1 不处理 |

本轮不新增 OpenSpec change；这些修复都直接服务于 P1 已有出口契约。P2 resolver/IR、P4 现代源码恢复和 P5 优化仍沿既定阶段推进，不把审计扩展成新的实现阶段。
