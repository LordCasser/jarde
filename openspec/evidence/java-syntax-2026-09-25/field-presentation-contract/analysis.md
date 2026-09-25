# 字段 claim 与最终呈现的证据契约

`field::plan` 在 Region/AST 构建前，从 SSA 和 Operation 验证字段身份及形状并放入 `claimed`；`Plan::materialize` 把所有 `claimed` 无条件写成 `FieldRecord(presented=true)`，`counts()` 也把其长度当成已呈现数。历史上 [`ChainExtraBoundary.class`](../short-circuit-chain-extra-entry/ChainExtraBoundary.class) 的 BCI 30 曾因 Region 重叠仅发引用；后续短路转接恢复已把它正确写成结构化赋值，不能为维持旧规格而回退 Region。现有[额外入口控制](../../../../tests/fixtures/p3-conditional-values/short-circuit-transfer-gateway-controls/ChainExtraBoundary.class)和[异常边控制](../../../../tests/fixtures/p3-conditional-values/short-circuit-transfer-gateway-controls/ChainExtraBoundaryException.class)仍使该字段只有引用、报告却称已呈现。`claimed` 是 Builder 需要的**可使用证明**，不是最终正文发生的事实。

Root 用当前工作树独立构建 CLI（SHA-256 `b72291f8cf4c75aa9a8db9d3c2a3a0f71bbb278b41180448af21de308048eadd`），以同一 `ChainExtraRunner` 对原 class 和两个 verifier-valid 拒绝控制执行 `java -Xverify:all`，三类均完成 32 行。冻结的 `class-source --evidence all` JSON 依次为 [`field-base.json`](field-base.json)、[`field-multi.json`](field-multi.json)、[`field-exception.json`](field-exception.json)。

| 物理 class | SHA-256 | BCI 30 Java 赋值 | BCI 30 来源 | `FieldRecord.presented` |
| --- | --- | --- | --- | --- |
| 原 class | `2630a7ea3e395b74121053dfa9370288dc510ea4d1fbfcf3bd9fbb9d04e1ba55` | 1 条，`structured/java` | 有 | true，正确 |
| 额外入口控制 | `76d4531ecde32f8d1697ba255652b17ac564b4c41ac3981c764369ef8c5bd617` | 0 条，`fallback/mixed` | 有 | true，错误 |
| 异常边控制 | `4bd4e3192eb3b430e351a4209fe0645c27c2aea28467832006965a65b92fb704` | 0 条，`fallback/mixed` | 有 | true，错误 |

后续独立任务应保留 claim，另在已成功提交的 Java AST/发射路径上统计真实字段操作。不能靠 BCI 是否出现在 source map、AST origins 或文本判定：fallback 注释也用同一 BCI 产生来源 span。普通读取的真实节点是 `ExprKind::Field`，普通写入是 `StmtKind::FieldAssign`；调用 accessor 合成的字段节点必须与本方法的 `fields.claim(bci)` 区分。`PostIncrement` 的目标仍是 Field 节点，但 `increment_expression` 的另一条 field++/++field 路径目前把完整 `this.n++` 文本装进 `ExprKind::Local`，仅扫描 AST 节点无法辨认它的字段指令；需要最小的明确字段操作回执或先改成语义化节点，不能解析字符串补判。`<clinit>` 候选抽取已有“只看实际 Program FieldAssign 且核对 field claim”的局部先例。

字段 summary 现在是全方法统计，不随 evidence selection 或 driver range 变化；RuleDetails 的记录仍在被选择时由 EvidencePhase 按 BCI/range 筛选、逐项计费。修复必须保持这两个边界并在预算/取消时不发布部分矛盾状态。最小验收矩阵包括：普通读/写成功与两个当前外层 fallback 控制、field++/++field 两条表示、constructor prologue 的 `putfield`、`<clinit>` 的静态 final 写与 RHS 读、原 class BCI 30 的 true 对照、不请求/全量/范围/部分 RuleDetails。实现归[独立 change](../../../changes/report-committed-field-presentations/tasks.md)。
