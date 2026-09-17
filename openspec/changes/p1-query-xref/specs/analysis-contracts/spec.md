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
