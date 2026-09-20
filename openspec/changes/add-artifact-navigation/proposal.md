## Why

CLI 只有一个 JSON 请求入口，调用方必须填满 18 个预算字段；一次恢复请求还要手工拼出物理方法身份、完整解析环境和 analysis stage 列表。库示例 `examples/resolve_and_analyze.rs` 展示同一成本：调用方自己构造 `PhysicalDefinitionId`、`PhysicalMethodId` 和 loader 环境。这些事实属于引擎而不是调用方；用户想说的是“打开这个 artifact，看看这个方法”，但当前没有任何入口能把 artifact 里的类与方法列出来并交回可直接使用的物理身份。

## What Changes

- 增加物理范围上的类导航列举：每个条目保留真实来源（standalone snapshot root，或 container origin 与 entry ordinal/raw name），重复物理定义全部返回，不合并、不 first-wins。
- 明确区分两种列举等级：archive entry 候选列举只按 raw name 规则给出候选，零 Header 读取；header 确认的类声明列举必须真实读取 class Header，未读取的候选不得进入确认集合，任何路径都不得被当作“已找到类”的证明。
- 增加类成员列举：一次有界 Header 读取给出类声明、字段与方法项，方法项带 raw name、descriptor、access flags 和可直接使用的 `PhysicalMethodId`；类级、字段级与 resource 级命中保留各自的种类与位置，不被包装成“某个方法”。
- 名称匹配不唯一时返回全部候选与选择依据，不静默取第一个；friendly 输入（点分隔类名、内部名、descriptor、过滤条件）可用，但结果必须绑定真实物理身份。
- 不新增 crate 或依赖，不进入 resolver、IR 或恢复。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `artifact-views`：物理范围上的类导航列举、entry 候选与 header 确认声明的区分、重复定义保留与名称歧义候选。
- `classfile-inspection`：一次有界 Header 读取内的成员列举及其物理方法身份绑定。

## Impact

前提为 P0 的快照/枚举与 classfile Header 检查、P1 的物理身份（`PhysicalEntryId`/`ContainerOrigin`/`PhysicalDefinitionId`/`PhysicalMethodId`）已交付。影响 `jarde-reader` 的 inspect/classfile 事实与新增列举报告、根 facade 入口及对应测试；复用现有 raw bytes、预算与 serde，不新增依赖，不改变 `Engine::enumerate`/`inspect_header`/`inspect_method_bytecode` 的既有语义。

与在途 change 的关系：本 change 只读现有物理身份与 Header，不做容器访问定向化，也不引入 prefix root，因此与 `bound-container-lookup`、`bind-prefixed-load-roots` 不重叠；但它是 `add-task-oriented-operations` 的前置，后者消费这里返回的身份与歧义规则，`add-task-oriented-cli` 再在此基础上提供命令行任务链。

不包含：全局索引或持久 class map、声明解析或 dispatch、Java 源码恢复、WAR 前缀 root、自动加载策略，也不声称列举结果证明类可加载、可链接或已解析。
