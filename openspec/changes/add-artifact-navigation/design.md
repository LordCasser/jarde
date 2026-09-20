## Context

见 [proposal](proposal.md)。P0/P1 已交付 `ArtifactSnapshot`、`EnumerationReport`、`PhysicalEntryId`/`ContainerOrigin`/`PhysicalClassLocation`/`PhysicalDefinitionId`/`PhysicalMethodId`；reader 层已交付按 entry 的 Header 读取（`inspect_header`，`ClassTarget::Entry` 接受完整枚举所得的 `PhysicalEntry`）与 `ClassFacts`（`this_class`、class flags、super/interface、`fields`、`methods: Vec<MemberHeader>`），`physical_variant_for_path` 已能从 raw name 派生 `PhysicalVariant`。缺口不在事实层级，而在调用方要自己把 entry 变成物理身份、把一次 Header 读取变成方法列表；本 change 把这些装配收回库内。

## Goals / Non-Goals

**Goals:** 让“列类 → 选类 → 列方法 → 得到可直接使用的物理身份”成为库内闭合闭环；每条结果的证据等级（entry 候选 / header 确认）与物理来源可复核。

**Non-Goals:** 不新增 crate、依赖或第二套身份；不做容器访问定向化（`bound-container-lookup`）；不引入 prefix root（`bind-prefixed-load-roots`）；不建全局索引；不解析声明、不做 dispatch、不启动 IR 或恢复；不提供 CLI 面（`add-task-oriented-cli`）。

## Decisions

### 1. 列举是现有物理视图上的读取，不是新视图

类/成员列举直接消费 `ArtifactSnapshot` 的既有枚举结果与既有 Header 读取语义；不引入持久索引、全局 class map 或第二套物理身份。类条目复用 `PhysicalClassLocation`/`PhysicalEntryId`，方法项复用 `PhysicalMethodId`，因此列举结果可以成为后续方法请求的输入，而不是需要调用方再翻译的中间格式。选择这一形状而不是另建导航专用身份，是因为 A07/A16 的可复核性依赖物理来源，第二套身份会先制造一次不可审计的映射。

### 2. 两种证据等级分开成两个形状

entry 候选列举只按大小写敏感 `.class` 后缀的 raw name 规则筛选，零 Header 读取；header 确认列举对每个候选真实执行既有 Header 检查，命中 `class_headers`/`class_bytes`/`read_bytes` 预算。两者返回不同报告形状，而不是同形状加一个布尔标志：路径像类与 Header 证明这是类是不同证据，混进一条记录会重演“用文件名冒充事实”的错误。这条边界也让“路径名与 `this_class` 不一致”必须如实并列，而不是由某一方改写另一方。

### 3. 成员列举从同一次读取派生完整身份

`ClassFacts` 已含成员表，成员列举在其上派生 `PhysicalMethodId { owner: PhysicalDefinitionId { location, class_bytes, variant: physical_variant_for_path(raw_name) }, name, descriptor }`。选择派生而不是让调用方拼装，是因为 variant、entry ordinal 与 class bytes 都来自同一次读取，第二个来源就是第二个可能的答案。abstract/native 由成员 flags 判定，不需要 Code；列举不读取任何 Body。

### 4. 歧义返回候选，选择依据是物理身份

同名多定义（跨 container、嵌套库、重复 ordinal）与同名不同 descriptor 的重载都返回候选列表。选择依据是条目自带物理身份，而不是遍历顺序或路径排序；调用方回传所选身份即得到唯一结果。friendly 名称只在查找阶段使用，绝不进入结果身份，因为显示名不是 A07 可复核的来源。

### 5. 复用与层次

复用 `ArtifactSnapshot` 枚举、`inspect_header`、`MemberHeader`、`physical_variant_for_path`、既有 coverage/execution/diagnostics 与 serde；没有缺失的通用能力，不需要新依赖，也不需要新的 parser。列举属于 reader/facade 的读取面，不进入 resolver、IR 或恢复层，A17 的构造计数边界因此保持成立。

## Risks / Trade-offs

- 确认列举的 Header 成本被低估 → 报告同时给出候选数与已确认数并计入既有预算；测试覆盖预算中断时的可靠前缀（A14）。
- 新报告与 `EnumerationReport`/`EngineHeaderReport` 字段语义重复 → 只在其上组合与派生，不复制身份或 coverage 语义；对同一 fixture 比较 source/location 一致性。
- 路径内部名与 Header 内部名不一致被静默洗掉 → 专项 scenario 要求两者都保留，且不得据此声称绑定成立。
- 列举被读成加载或解析结论 → 文档与 scenario 明确只读 Header，不解析、不加载、不构建 IR。

## Migration Plan

新增入口与报告，不改变 `Engine::enumerate`/`inspect_header`/`inspect_method_bytecode` 的既有语义；`add-task-oriented-operations` 与 `add-task-oriented-cli` 的实现随后迁移到列举身份。无持久数据迁移；回滚只需移除新增入口与其测试。
