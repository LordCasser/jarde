# query-api Specification

## Purpose
提供稳定的查询入口和可序列化结果，使库、CLI 与 Agent 能够表达查询范围、关系种类、证据、分页、取消和未判定状态，而不把不完整扫描误报为否定结论。

## Requirements

### Requirement: Query compiler preserves soundness

查询 SHALL 先声明 P1 支持的目标关系（`mentions_symbol`、`literal_value` 或 raw CP）、artifact/runtime view、consumer 类别和预算，再选择过滤器。过滤器允许假阳性但 MUST 不因过滤造成假阴性；无法安全判定时 SHALL 将候选标记 Unknown 并继续 consumer 扫描或扩大范围，预算不足时显式 Partial。`Engine::query` 的 `references_definition` 与 `may_dispatch_to` 不在此入口解析：它们返回 `UnsupportedAnalysis`，声明解析与已知范围 dispatch 由显式运行环境下的独立入口 `Engine::resolve_symbol` 与 `Engine::declaration_references` 提供，把这两条关系接到 resolver 属独立变更。

#### Scenario: Definition relation requested from the query engine

- **WHEN** 调用方请求从调用点推导声明类或运行时派发目标
- **THEN** 响应为 `UnsupportedAnalysis`，保留原始 owner/name/descriptor 的 `mentions_symbol` 或 raw CP evidence，不将其误报为已解析关系；解析入口已存在也不改变本入口的关系语义，query 不因此启动 resolver

#### Scenario: Unknown filter continues scanning

- **WHEN** 过滤器无法安全判断某个候选是否匹配
- **THEN** 查询继续扫描该 consumer 类别或扩大扫描范围并保留候选；只有预算不足或取消导致扫描未完成时才返回 Partial，不得以 Unknown 直接跳过候选并报告 Complete

### Requirement: Coverage and pagination are explicit

每个查询结果 SHALL 独立返回 artifact/runtime/dynamic coverage、execution（Complete/Partial/Cancelled/Failed）、预算消耗、snapshot/view identity、evidence 和已扫描范围。Unknown 是候选/解析状态，MUST 不与 execution 混为同一枚举。分页游标 SHALL 绑定 snapshot、完整 query identity、view、排序/扫描边界和 schema 版本；完整 query identity MUST 包括 relation、完整 target（symbol kind/owner/name/descriptor 或 literal kind/原始值与浮点位模式）和 consumer schema。改变 target MUST 在重放扫描边界前拒绝，不得按另一目标的匹配序号跳过结果。每页 max_items 和预算是执行参数，可以改变，但不能改变查询含义。page.has_more 不等于搜索完成。取消、缺失搜索范围和预算耗尽 MUST 不得映射为 NoMatch。

#### Scenario: Cancelled full-range search

- **WHEN** 全范围查询在中途收到取消或发现请求纳入范围的 artifact 缺失
- **THEN** 响应包含已扫描范围、终止原因和 Cancelled/Partial，不报告 Complete（验收 A14）

#### Scenario: Symbolic query without platform headers

- **WHEN** 声明物理范围和 consumer schema 已完整扫描，但没有提供平台 Header
- **THEN** X1 仍可报告 complete-within-schema，runtime resolution 为 NotRequested；不因未请求的解析缺失而否定已经读取的结构事实

#### Scenario: Stable page continuation

- **WHEN** 调用方使用同一 snapshot/view 和返回的游标请求下一页
- **THEN** 结果不重复、不跳过已发布条目，并保留可追溯的 entry/member evidence

#### Scenario: Changed target cannot reuse a cursor

- **WHEN** 调用方取得目标 A 的第一页游标，保持 snapshot/view/relation/consumer schema 不变，但把目标改成 B 后请求续页
- **THEN** 库与 CLI 均返回 `query_cursor_mismatch`，不得跳过 B 的首项后返回 Complete；symbol 的 owner/name/descriptor 任一变化和 literal 的类型/原始值变化都适用

#### Scenario: Page size does not change query identity

- **WHEN** 调用方保持完整查询身份不变，仅调整下一页 max_items 或资源预算
- **THEN** 合法游标仍可续读；在各页均有足够预算且读到末尾时，拼接的有序结果与同身份无分页运行相同，预算中断仍返回真实状态

#### Scenario: Supported category with an unread standard location

- **WHEN** 已请求的 consumer 类别包含一个标准位置，但扫描器尚不能解释该位置或解析在该位置失败
- **THEN** 返回可定位的位置和未覆盖原因，coverage 不得为 CompleteWithinSchema；不得以该类别的其他位置已扫描为由报告无命中的完整搜索
