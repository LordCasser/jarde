## ADDED Requirements

### Requirement: Diagnostic codes are version-scoped identities

诊断码 SHALL 标识「该引擎版本在本次运行中记录了哪个事实」，MUST NOT 被读作跨版本可比的量。同一个码在不同版本的含义变化（例如声明事实交接前 `jre_declaration_class_not_in_run` 在单个 artifact 上出现 10,720 次、交接后语料全局 0 次，而 `jre_declaration` 出现在每个 artifact 上）MUST 被登记为词汇/事实来源的变化；消费方、比较报告与文档 MUST NOT 把某个码的出现次数下降直接读作质量或覆盖改善，比较 MUST 同时声明两个引擎版本与各自的 code 词汇。

#### Scenario: A declaration code changes meaning

- **WHEN** 两个版本或两份报告之间某个诊断码的出现次数发生变化，或一个码消失而另一个码出现
- **THEN** 报告与文档 MUST 说明这是词汇/事实来源变化而不是结论；比较 MUST 带上两个引擎版本与 code 词汇，MUST NOT 只给出「从 N 次降到 0 次」的差值

#### Scenario: A code count is not a quality metric

- **WHEN** 报告按码或按 artifact 呈现诊断计数
- **THEN** 这些计数 MUST 被表述为某声明版本下记录的事实数量，MUST NOT 被表述为正确性、覆盖或语义改善；某个码缺席 MUST NOT 在没有版本与词汇上下文时被当作对应风险消失的证据

#### Scenario: Diagnostic codes are stable within one version

- **WHEN** 同一引擎版本对同一受控输入的重复运行
- **THEN** 诊断码集合与计数保持确定；本 requirement 要求的是把码读作带版本的词汇身份，不改变任何码在单个版本内的含义或确定性
