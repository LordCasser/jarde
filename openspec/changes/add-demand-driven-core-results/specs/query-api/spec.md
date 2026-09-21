## MODIFIED Requirements

### Requirement: Coverage and pagination are explicit

每个查询结果 SHALL 独立返回 artifact/runtime/dynamic coverage、execution（Complete/Partial/Cancelled/Failed）、预算消耗、snapshot/view identity、evidence 和已扫描范围。Unknown 是候选/解析状态，MUST 不与 execution 混为同一枚举。分页游标 SHALL 绑定 snapshot、完整 query identity、view、排序/扫描边界和 schema 版本；完整 query identity MUST 包括 relation、完整 target（symbol kind/owner/name/descriptor 或 literal kind/原始值与浮点位模式）和 consumer schema。改变 target MUST 在重放扫描边界前拒绝，不得按另一目标的匹配序号跳过结果。每页 max_items 和预算是执行参数，可以改变，但不能改变查询含义。page.has_more 不等于搜索完成。取消、缺失搜索范围和预算耗尽 MUST 不得映射为 NoMatch。

查询 SHALL 将续扫位置推进到实际 consumer/provider 边界，而不是只保存已返回结果的序号。页达到 `max_items` 时 MUST 停止后续 provider/consumer 工作，未访问 consumer、未解释位置和未验证后缀保持未知；游标 MUST 保留足以从该边界继续的排序、consumer 内偏移、必要的可复核结构边界和 schema 身份。必要的边界验证在续页重放时可重新读取，但其读取与解码费用 MUST 记入当前页的 usage，不能把旧页的工作当作免费命中。


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

#### Scenario: Continuation resumes inside a consumer

- **WHEN** 一页在一个 consumer 的候选或位置序列中达到 `max_items`
- **THEN** 游标保存该 consumer 的可复核后继位置和已完成验证边界；下一页从该位置继续，不重复已发布条目、不跳过同一 consumer 的未发布条目，也不把页满报告为搜索完成

#### Scenario: Page full is distinct from complete scan

- **WHEN** 当前页已达到 `max_items` 但请求范围仍有未访问 consumer 或未解释后缀
- **THEN** 该页在没有其它停止的情况下 execution=Complete，查询范围 coverage 仍未完整并保留未扫描范围与可续读游标；只有实际扫描到声明范围末尾且无未知后缀时才可报告 CompleteWithinSchema。页满不是预算耗尽或取消

#### Scenario: Unknown suffix remains visible after continuation

- **WHEN** 当前页结束时 consumer 后缀尚未访问，而后续位置实际存在损坏或未支持形态
- **THEN** 当前页保留已确认候选及“未访问”范围，不提前发布尚未观察到的损坏诊断；续页实际到达后报告错误与位置，未覆盖范围不能压成 NoMatch 或 complete-within-schema

#### Scenario: Old cursor schema is rejected before scanning

- **WHEN** 调用方提交旧 schema、不同排序/consumer schema 或缺少新续扫边界字段的游标
- **THEN** 库与 CLI 在重放任何查询扫描前返回可识别的 `query_cursor_mismatch`，不按旧匹配序号继续、不产生新的候选或 Complete 结果

#### Scenario: Replayed boundary validation is charged honestly

- **WHEN** 续页为了验证 cursor 绑定、物理定义或 consumer 边界而重新读取必要 Header/Body 事实
- **THEN** 当前页的 `usage`、`execution`、`coverage` 和终止原因包含这次实际工作；验证失败、预算耗尽或取消保留真实前缀，不能借用前页费用或把重放隐藏成缓存命中

#### Scenario: Concatenated pages equal the unpaged query

- **WHEN** 同一完整 query identity 在足够预算下分别执行不分页查询和连续分页查询
- **THEN** 按稳定排序拼接各页得到相同的有序 evidence；汇总各页实际检查范围和观察到的缺口后得到相同覆盖结论，单页不得继承未验证的旧前缀，也不要求早页预知后缀诊断；必要验证允许重新计费

#### Scenario: Cursor data does not prove a prefix was scanned

- **WHEN** 调用方提供经过编辑的 consumer 位置或自行声称前缀已验证的游标
- **THEN** 系统在当前预算内校验身份和位置，拒绝无效边界；即使结构位置有效，也不能将游标携带的断言作为当前页已经扫描前缀的证据
