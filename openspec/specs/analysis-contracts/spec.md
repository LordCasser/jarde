# analysis-contracts Specification

## Purpose
让库调用方和命令行用户准确解释分析结果的来源、覆盖边界和终止原因，避免把未扫描、取消、解析失败或不支持的内容误解为不存在或已经验证成功。本契约覆盖当前全部分析平面：artifact 枚举与物理视图、classfile Header 与 bytecode inspection、结构 XRef、显式运行环境下的符号解析与声明引用，以及方法分析报告（IR 阶段状态、质量、覆盖、执行与语义证据平面）；方法分析交付的是内部 IR 与报告平面，不含 Java 源码恢复。

## Requirements

### Requirement: Independent analysis planes
系统 SHALL 允许直接打开 artifact 请求事实或单个方法，不要求全程序加载、全局索引、持久数据库或预先反编译。Query 与 Decompiler MUST 共享输入解码和身份契约，但不得隐式互相启动；依赖默认只扩展 Header，Body 升级必须附理由和预算。

#### Scenario: Artifact inspection before indexing
- **WHEN** 调用方首次打开一个 JAR，仅请求某 entry 的 Header
- **THEN** 系统不构建全局 XRef、CFG、SSA 或 Java AST，并记录实际读取/物化范围

### Requirement: Physical symbolic and resolved identities
系统 SHALL 区分物理定义、SymbolRef 和运行视图下的 ResolvedDefinition；物理定义身份包括 class bytes 身份、物理变体，以及显式的 standalone snapshot root 或真实 archive entry。Archive entry 身份 SHALL 保留 snapshot、当前容器、entry ordinal/raw name，以及逐层记录“父容器中的哪个 entry 产生哪个 child container”的有向 origin chain。成员身份保留完整 descriptor。相同字节可共享解析 facts，但 MUST 不合并不同物理 origin 或 loader binding；系统 MUST NOT 为 standalone CLASS 伪造 ZIP entry，也不得用无法复核边的容器 ID 列表代替 nested origin chain。

#### Scenario: Identical bytes at distinct origins
- **WHEN** 两个 entry 含完全相同的 class 字节但位于不同位置
- **THEN** 内容摘要可以相同，物理来源仍可独立定位，不能因解析缓存复用而合并定义

#### Scenario: Symbol without definition
- **WHEN** classfile 引用了没有提供定义的成员
- **THEN** 原始 owner/name/完整 descriptor 保留；定义解析和运行时派发不能从该符号的存在自动推断

#### Scenario: Standalone class root identity
- **WHEN** 调用方打开 standalone CLASS 并为其建立物理定义身份
- **THEN** 身份绑定真实 snapshot root 和 class bytes，不创建虚假的 container、entry ordinal 或 archive name

#### Scenario: Duplicate nested archive entries
- **WHEN** 父容器中两个不同 ordinal 的同名 entry 各自产生一个 child container
- **THEN** 两条 origin chain 分别保留其父 entry ordinal/raw name 和 child container ID，内层同名定义不得因容器名称相同而合并

### Requirement: Provenance and execution are explicit
系统 SHALL 为枚举、artifact-tree、Header、bytecode、解析/闭包和方法分析返回快照/entry 身份及适用的 container origin、class offset 或方法 BCI，并分别返回 coverage、execution、diagnostics 和预算消耗。非累加的高水位预算维度 SHALL 进入 limits、usage 和 BudgetExceeded 结果：`nested_depth` 表示已接受的容器嵌套深度（root container depth 为 0，直接 child 为 1），`dependency_depth` 表示已接受的依赖闭包深度；两者相互独立，任一维度不得代替或改写另一维度。任何计费维度 SHALL 同时出现在 `Limits`、`UsageSnapshot`、终止维度枚举以及库与 CLI 两个请求 schema 中，请求 schema 保持显式——缺失维度是协议错误，不静默取默认值。

#### Scenario: Cancelled enumeration
- **WHEN** ZIP 根容器已经建立，且枚举开始前或过程中收到取消
- **THEN** 返回 `Ok(EnumerationReport)`，execution 为 Cancelled，并保留已完成的可靠前缀及其 coverage，不能将其标为 Complete

#### Scenario: Partial or failed enumeration after root open
- **WHEN** ZIP 根容器已经建立，枚举过程中耗尽预算或后续 entry 结构损坏
- **THEN** 返回 `Ok(EnumerationReport)` 并保留已验证前缀；预算耗尽为 Partial 和对应 BudgetExceeded 维度，结构损坏为 Failed 和错误 diagnostic，artifact coverage 为 Partial，未完成范围明确标为 skipped

#### Scenario: Root container cannot be established
- **WHEN** 输入不是 ZIP，或 ZIP 根容器无法建立
- **THEN** 枚举可以返回 `Err`，不得伪造可枚举的根容器或部分前缀

#### Scenario: Nested depth is a high-water limit
- **WHEN** artifact-tree 已接受 depth 1 的 child，随后请求 depth 2 而 `nested_depth` limit 为 1
- **THEN** usage 的 nested depth 高水位保持 1，execution 为 Partial 且 reason 指向 `nested_depth`，depth 2 child 标为未扫描而不是累加消费或空成功

#### Scenario: Dependency depth is an independent high-water limit
- **WHEN** 同一请求已接受容器深度 2（`nested_depth` limit 为 3），随后依赖闭包扩展到深度 5 而 `dependency_depth` limit 为 4
- **THEN** execution 为 Partial 且 reason 指向 `dependency_depth`，保留已接受的容器深度与依赖深度 usage；`usage.nested_depth` 与 `usage.dependency_depth` 分别记录各自高水位，任一维度超限不改写另一维度的 usage 或判定

### Requirement: Independent evidence and coverage dimensions
结构引用 SHALL 分开表示 relation、derivation、resolution、applicability 和 validity。结果 MUST 分开描述 artifact structural coverage、runtime resolution coverage、dynamic analysis coverage、execution 和分页；“对已声明 schema 完整”不得扩张为理解任意未知 attribute 或完整运行图。

#### Scenario: Complete structure with missing runtime definitions
- **WHEN** 声明范围内结构扫描完整，但运行时定义缺失或未请求解析
- **THEN** 结构覆盖可为 complete-within-schema，解析状态为 Missing/NotRequested，不能用一个 supported/complete 布尔值替代二者

### Requirement: Pure Rust library first
系统 SHALL 提供同步、可取消的库 API 和调用相同语义的 JSON CLI；运行不需要 JVM、外部反编译器、网络或持久索引。

#### Scenario: Offline inspection
- **WHEN** 未安装 JDK 的调用方打开本地 CLASS/JAR 并检查某方法
- **THEN** 库及 CLI 均可完成声明能力内的读取，且不会启动外部进程

### Requirement: Honest release matrix
项目 SHALL 分别列出 parse、X1、resolution、decompile-quality、output-level 的已实现状态；未完成阶段 MUST 保留未完成标记。

#### Scenario: P0 release
- **WHEN** 用户查看本阶段能力
- **THEN** 文档声明 Header/bytecode 能力及其限制，并将 X1、resolver、Java recovery 标记为未实现
