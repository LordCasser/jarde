## ADDED Requirements

### Requirement: Class-source snapshot proof stops before body recovery

class-source 的同快照层级 widening 证明因预算、取消或期限停止时，系统 SHALL 保留原执行原因、用量及诊断，并在方法正文恢复前交付既有 Stopped/not_produced。系统 MUST 保留本次已完成 raw analysis 与可靠前缀，MUST NOT 将停止转换为空证明后继续正文或后继投影，也不得把合法请求变成无报告的裸错误。

#### Scenario: Snapshot header budget stops the pending body

- **WHEN** 真实公共 class-source 请求已取得方法分析，但所需同快照层级证明在 Header 预算处停止
- **THEN** 该方法尚未交付的恢复产物 MUST 为 Stopped、content=not_produced、无未提交文本或段表；execution 为 Partial 且保留原 ClassHeaders 维度与诊断，raw analysis 与已发布兄弟前缀保持，类 execution 非 Complete，CLI 以既有未完整状态退出4

#### Scenario: A stopped dependency report is not absence

- **WHEN** 同快照层级证明所需依赖读取已返回非 Complete 执行报告，其中承载结束共享请求的原预算/取消/期限停止，且没有产生可用证明事实
- **THEN** 系统 MUST 将该原始报告的执行停止及诊断交给恢复与类汇总，MUST NOT 仅凭无证明事实判为普通拒绝；原预算字段仅在原报告/错误确实提供时保留，不编造 limit、consumed、requested 或位置

#### Scenario: No recovery work follows the first proof stop

- **WHEN** 前置证明在本次真实收费或检查点首次观察到执行停止
- **THEN** 原停止 SHALL 保留，之后不再启动正文、optional evidence 或后继类投影收费；为发布停止保留原有报告与可靠前缀，不重跑分析、重置预算或重试收费以改换原因

### Requirement: Class-source postbody dependency stops retain committed content

class-source 的 double-brace 依赖证明在正文已交付后因预算、取消或期限停止时，系统 SHALL 保留已提交方法正文、逐方法结果及可靠兄弟/raw/source 前缀，将原停止汇入类 execution 与 diagnostics，结束后继投影。系统 MUST NOT 因已有正文而发布类 Complete，也不得将已提交产物改成 not_produced 或清空。

#### Scenario: Header budget stops only the postbody projection

- **WHEN** 公共 class-source 请求已交付含语句的 main，随后 double-brace 依赖 Header 读取因预算拒绝，且前置证明已完成
- **THEN** main 的已提交正文、content、执行结果与 source map SHALL 保留，类 execution MUST 为 Partial、reason 保留原 ClassHeaders、diagnostics 保留原停止；类汇总不能因 main Complete 而变 Complete，CLI 退出4

#### Scenario: Later projections do not resume a stopped request

- **WHEN** double-brace 依赖证明停止且报告仍有可读正文
- **THEN** 系统 MUST 保留这些已提交结果并终止后继 anonymous-interface 等本次恢复投影，不再花费其预算或以其后续结果覆盖首次停止；结构 coverage 仍按实际已读范围报告，不伪造未完成或已完成范围

### Requirement: Dependency proof absence and execution stop remain distinct

上述两条证明消费路径 SHALL 区分完整读取后的普通保守拒绝与实际执行停止。缺失、歧义、不适用或固定算法搜索界造成的无证明事实 MUST 保持既有 refusal；原错误或依赖报告承载的预算/取消/期限共享请求停止 MUST 保留原执行语义，不从文本推断或伪造原因。content、quality、syntax、compile 和 verification MUST NOT 代替 execution。

#### Scenario: Wide proof and ordinary refusal keep their source behavior

- **WHEN** 宽预算下同快照层级/double-brace 证明完整完成，或完整读取后因缺失、歧义、不适用、无关联、固定算法界而普通拒绝
- **THEN** 系统 SHALL 保持既有合法正文或拒绝边界、物理来源及效果，不将所有无证明事实变成 Stop；普通拒绝可为 Complete，不能普遍停止以通过预算反例；value-level Unsupported/decode 不完整 SHALL 按现有隔离与汇总契约处理，不一律升级为共享请求结束

#### Scenario: Cancellation is observed during real proof work

- **WHEN** token 初始未取消，真实公共请求在上述任一路径的实际证明收费中收到取消并于检查点观察到
- **THEN** 执行 SHALL 为 Cancelled 并保留真实停止位置/诊断，按前置或后置交付边界保留对应结果，不改为 BudgetExceeded、不再继续后继恢复工作

#### Scenario: Deadline and earlier causes are not rewritten

- **WHEN** 实际证明检查点观察到期限耗尽，或在此之前已发生另一真实预算停止
- **THEN** 系统 SHALL 保留实际先发生且已观察到的停止；期限保持原 ElapsedMillis 语义，其他预算保持原维度，不将非累加维度改成输出/IR预算，也不以稍后的 poll 覆盖先前原因
