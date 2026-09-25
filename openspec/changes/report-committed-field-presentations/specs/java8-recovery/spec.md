## ADDED Requirements

### Requirement: Field presentation evidence reflects committed Java operations

恢复报告 SHALL 区分一条字段指令通过 `field@1` 身份/形状证明与它确实作为最终 Java 字段操作发射。只有最终提交的 Java AST 含与该方法真实字段指令同一 BCI、同一访问方向的字段操作时，`FieldRecord.presented` 才能为 true；摘要中的 presented 数 MUST 使用同一判定。引用注释或 source map 的 BCI 命中 SHALL 仅说明来源可追，不得充当呈现证明。`field@1` 自身拒绝与后续 Region/Builder 压掉的已证明字段 MUST 给可区分的理由。

#### Scenario: An enclosing refusal suppresses a proved field write

- **WHEN** verifier-valid 的额外入口或真实异常边控制中，`ChainExtraBoundary.assign` 的 BCI 30 `putstatic result:Z` 通过字段身份计划，但最终仅提交包含 BCI 30 的 `@bytecode` 引用
- **THEN** 字段记录 SHALL 为 `presented=false`，摘要 SHALL 不计它为已呈现；BCI 30 的 source map 仍须可追；不能为掩盖报告错误而改变原子 Region 拒绝

#### Scenario: Real field operations remain presented

- **WHEN** 最终 Java AST 提交普通字段读取/写入、嵌套表达式中的读取、或由已证明 field++/++field 形状写出的字段更新
- **THEN** 对应本方法真实字段指令的记录 SHALL 为 `presented=true`，读写各按原操作计数；合成 accessor 的成员文字不得被误算作本方法的真实字段指令

#### Scenario: Evidence selection cannot change the verdict

- **WHEN** 同一方法分别请求 essential、all、BCI 范围 RuleDetails，或在回执/证据物化期间触及预算和取消
- **THEN** 完整运行的字段摘要 SHALL 相同，只有选中的记录进入详细表；不完整运行 MUST 保持 partial/stop 且不得发布自相矛盾的完整字段呈现结论
