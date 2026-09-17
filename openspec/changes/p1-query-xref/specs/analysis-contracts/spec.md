## Purpose

让 P1 查询在 standalone CLASS、顶层 archive entry 与嵌套容器中使用同一套可复核的物理身份，避免伪造 entry 或丢失容器边。

## MODIFIED Requirements

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
系统 SHALL 为枚举、artifact-tree、Header 和 bytecode 返回快照/entry 身份及适用的 container origin、class offset 或方法 BCI，并分别返回 coverage、execution、diagnostics 和预算消耗。`nested_depth` SHALL 作为非累加的高水位预算维度进入 limits、usage 和 BudgetExceeded 结果；root container depth 为 0，直接 child 为 1。

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
