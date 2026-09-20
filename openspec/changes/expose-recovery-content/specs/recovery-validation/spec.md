## ADDED Requirements

### Requirement: Coverage metrics distinguish artifact content and validation

恢复评测 SHALL 分别报告请求适用范围、有产物、含语句、只有说明和停止数量，并注明分母与分类版本；MUST NOT 把 Produced 比例称为语句恢复率或行为正确率。文本、调用序列、字符串相似度 SHALL 各自报告有效 pair 数和排除原因。行为对照只为实际执行的受控 fixture 提供证据，不能由相似度或多数投票推导语义等价。

#### Scenario: Produced includes explanation-only outputs

- **WHEN** 一组请求产生含语句、仅解释及停止三种结果
- **THEN** 报告可对账的三类数量与 Produced 的合计；没有 Code 的方法单列适用状态，不用纯解释结果抬高语句覆盖

#### Scenario: Reference decompiler folds initializers

- **WHEN** 对照工具折叠 `<clinit>` 或省略隐式 `<init>`，且无法建立方法级 pair
- **THEN** 报告表示差异/未匹配范围，不自动计为任一引擎失败或正确，并从相应 pair 指标分母中明确排除

#### Scenario: Historical heuristic differs from structural classification

- **WHEN** 旧报告用去注释 token 判内容，新报告使用引擎 content
- **THEN** 保留旧分类定义并标明新版本，记录差异样例，不静默重写旧覆盖数字或把两者视为同口径
