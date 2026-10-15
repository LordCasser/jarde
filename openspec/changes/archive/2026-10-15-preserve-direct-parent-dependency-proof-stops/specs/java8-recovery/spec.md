# Spec Delta

## ADDED Requirements

### Requirement: Direct-parent dependency proof stops remain visible

class-source 的直接父类 accessor 调用与字段写入证明 SHALL 保留实际预算、取消、期限停止及原诊断，涵盖 source 直接父类关系和 owner 成员表的必需读取。停止 MUST 在正文恢复前结束后继工作，保留本次分析、真实用量及已合法交付的成员前缀，MUST NOT 降格为空证明或 Complete。完整读取后的普通拒绝和值级停止仍遵循既有隔离契约。

#### Scenario: A field dependency header stops the pending body

- **WHEN** 真实公共 class-source 在无更早停止时，直接父类字段证明取得依赖读取的原 ClassHeaders 预算停止报告
- **THEN** 恢复与类汇总 MUST 保留该原维度及全部原诊断，正文尚未提交，恢复为 Stopped/not_produced，类 execution 非 Complete，JSON CLI 退出4；不得凭当前用量编造原错误未提供的额度数字

#### Scenario: An accessor stop ends later field proof work

- **WHEN** 直接父类 accessor 证明先观察到本请求的实际停止
- **THEN** 系统 MUST 保留该原停止，不再执行字段证明、正文恢复、可选证据或后继收费投影；修复前继续发生的第二次拒绝不构成应保留的工作

#### Scenario: Required source and owner reads share the stop contract

- **WHEN** 任一直接父类消费者核对实际 source 的直接父类关系或读取 owner 的方法/字段表时发生共享请求停止
- **THEN** 原停止 SHALL 在公共报告可见，不能只保护 owner 读点而将 source 读取中断解释为普通缺证；已由本次 IR 提供的关系不要求额外读取

#### Scenario: A directly returned failure preserves its cause

- **WHEN** 实际证明检查或收费直接返回结束请求的原错误
- **THEN** 原错误具备的维度和额度数字 MUST 保留；Cancelled 不得改写为预算，非 counted 期限不得冒充 IrItems 或 OutputBytes；合法请求仍须返回带分析与恢复停止的报告

#### Scenario: Cancellation or deadline is observed inside proof work

- **WHEN** token 初始未取消，实际证明收费内发生取消并由 checkpoint 观察，或真实期限在实际证明边界到期
- **THEN** 系统 SHALL 保留实际 Cancelled 或 ElapsedMillis 停止，结束后继工作，不再次 poll/charge 重建原因，不回滚停止前已获准用量；预取消、sleep 或伪造错误不能替代该验收

#### Scenario: Publication keeps the reliable prefix and admission boundary

- **WHEN** 较早成员已合法交付，而后续直接父类证明在正文恢复前停止
- **THEN** 较早 text/source map MUST 保留，当前正文不得伪称已产出；无法合法支付当前 stopped member 发布费用时，不强迫发布该 record，类汇总仍须保存原诊断与真实 coverage 缺口

#### Scenario: Ordinary absence and value-level stops retain isolation

- **WHEN** 完整读取后 source/owner 缺失或歧义、字段/accessor 不满足原身份访问条件，或遇既有值级 Unsupported/decode 情况
- **THEN** 系统 MUST 保持既有普通拒绝、物理来源与隔离，不制造预算或取消停止，不将所有非 Complete 无差别终结为共享请求停止

#### Scenario: Wide recovery preserves field ownership and helper behavior

- **WHEN** 合法同包直接父类字段/accessor 在宽预算下完成证明
- **THEN** 原字段身份、访问约束、receiver 与 BCI 来源、显式父 owner cast 及物理 helper 调用 SHALL 保持；完整生成类族与原 Runner 的编译执行行为须一致，不以普遍拒绝或停止通过反例
