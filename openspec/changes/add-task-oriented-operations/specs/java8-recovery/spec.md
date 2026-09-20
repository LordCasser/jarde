## ADDED Requirements

### Requirement: Recovery presentation leads with delivered content

任务导向的恢复结果 SHALL 先表达交付内容（`contains_statements`、`explanation_only`、`not_produced`），再表达 quality，最后表达任何停止原因；上述三者 MUST 从报告的既有字段读取，MUST NOT 通过重新分析 `text`、剥离注释、统计 token 相似度或第二次构建结构来判定。呈现顺序 MUST NOT 改变任何既有契约：content 仍不证明完整恢复、可编译或语义等价，representation、quality、syntax/compile/semantic/verification、execution 与 text/source map 保持原语义，停止时继续遵循既有停止契约。

#### Scenario: Statement-bearing result leads with content

- **WHEN** 产物含实际发射的语句
- **THEN** 呈现先给出 contains_statements 再给 quality；不得把该分类读作完整恢复、验证通过或可编译（验收 A13）

#### Scenario: Explanation-only and stopped results

- **WHEN** 产物只有说明，或运行在交付产物前停止
- **THEN** 呈现分别先给 explanation_only 或 not_produced，再给 quality 与停止原因；停止时 text/source map 与 execution 保持原有契约（验收 A13、A14）

#### Scenario: Presentation does not re-analyse text

- **WHEN** 交付文本的注释措辞或字面量内容变化而结构不变，例如说明中出现 `return`
- **THEN** 呈现的 content、quality 与停止原因不变，库与 CLI 对同一请求给出相同字段（验收 A13）
