## ADDED Requirements

### Requirement: Output admission preserves the actual stop

恢复输出 SHALL 保留检查与计费期间的真实停止类别及位置。取消 MUST 报告为 Cancelled；deadline 中断 MUST 保持既有 Interrupted 形态。只有实际输出额度不足时 SHALL 报告 OutputBytes 预算停止，并保留真实已写字节及适用限值。停止 MUST 丢弃未提交的正文与来源，不能交付部分产物。

#### Scenario: Cancellation during output admission

- **WHEN** 取消在最初轮询通过之后、输出额度检查或计费期间被观察到
- **THEN** 输出 SHALL 返回 Cancelled 与当前写入位置，MUST NOT 报告虚构的 OutputBytes 额度不足；正文与来源不提交

#### Scenario: Interrupted output admission

- **WHEN** deadline 在输出检查或计费期间终止该次工作
- **THEN** 输出 SHALL 保留既有中断类别、原因与位置，MUST NOT 将其重新归类为 OutputBytes 额度不足

#### Scenario: Actual output quota refusal

- **WHEN** 已写入一段正文后，下一段因实际输出额度不足而被拒
- **THEN** 输出 SHALL 报告 OutputBytes 预算停止及真实已写进度、限值与拒绝位置；未提交的正文和来源 MUST 全部丢弃

#### Scenario: Exact quota and replay remain consistent

- **WHEN** 完整输出正好适合额度，且同一已决定产物进行来源重放
- **THEN** 输出 SHALL 正常完成，正文、物理来源、额度费用与重放一致性 SHALL 保持既有契约；停止分类修复 MUST NOT 改变正常产物
