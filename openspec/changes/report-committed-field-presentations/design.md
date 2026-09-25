## Context

`field::plan` 先用 SSA、Operation 和声明事实把可证明字段存入 `Plan::claimed`。Builder 仍要用 `claim(bci)` 决定 Field/FieldAssign 的身份，故 claim 不能改称“已发射”。当前 `Plan::materialize` 对所有 claimed 无条件造 `FieldRecord(presented=true)`，`counts()` 直接以 claimed 长度算已呈现。原 `ChainExtraBoundary.class` 已由后续短路恢复写出 BCI 30 字段赋值；同一语法点的额外入口、异常边 verifier-valid 控制仍整体引用 BCI 30，却继续误报已呈现，作为本 change 的 false 对照。

## Goals / Non-Goals

**Goals:** 报告的字段呈现事实只来自最终提交的 Java AST；同一规则的记录/摘要/诊断一致；未选择 RuleDetails 时不构造其记录，driver range 只裁剪记录不改变全方法摘要；预算停止不发布部分矛盾结果。

**Non-Goals:** 放宽 Region 拒绝、改变 Field 身份准入、把注释中 BCI 算 Java 语法、为 field++ 另建公开 AST 变体、统一所有规则的报告机制。其它规则若有同类债务另案审计。

## Decisions

1. **最终 AST 判定与先验 claim 分离。** 在 `build::build` 完成所有 Region、局部声明与折叠后，对最终 `Program.stmts` 做有预算的穷尽遍历。普通字段读取只认真实 `ExprKind::Field` 且其 direct BCI 与本方法 field plan 的 read claim 相符；写入只认 `StmtKind::FieldAssign` 且 direct BCI 与 write claim 相符。嵌套 `If`、loop、switch、try、synchronized、头部表达式与所有子表达式均遍历；`Fallback` 注释的 BCI 永不计入。使用私有 BCI 集合作为呈现回执，不新增公开形状。
2. **覆盖现有两个更新表示而不解析文本。** 新 `ExprKind::PostIncrement` 的目标字段 read 和外层 update write 各按原始 BCI/claim 核对。旧 field++/++field 由 `increment_expression` 写成 `ExprKind::Local` 文本，不能从字符串扫描；只允许该现有 `FieldIncrement` 证明计划与最终 `Return` 语句的 direct update/return BCI 相互匹配后，将其已证明的读写指令加入回执。若最终语句被替换或引用，不收取计划中的字段。
3. **报告物化读取同一个回执。** `Plan::materialize` 对 claimed BCI 只有在回执包含时才写 `presented=true`；否则给稳定的“字段身份已证明但最终 Java 操作未发射”拒绝理由。`counts` 的 read 数仍为 claimed+plan refusals，presented 数只数回执与 claimed 的交集。已有 plan refusals 保持自己的要求/代码，不因最终回执改写。摘要全方法计算，RuleDetails 仍由 EvidencePhase 选择和逐项计费、range 过滤。
4. **提交边界与预算。** 回执在 AST 完成后、报告成功提交前形成并计入 `AnalysisSteps`/`IrItems` 且轮询取消；证据物化失败保持现有 partial/stop 契约。不能从 emit 的文本/SourceMap 重建事实，因为 quote、派生 origin 和合成 accessor 均可能共享成员名或 BCI。合成 accessor 字段节点不属于本方法真实 field claim，不参与 `field@1` 计数。

## Risks / Trade-offs

- [穷尽遍历遗漏新 AST 分支] → 对 `StmtKind`/`ExprKind` 做无通配符匹配，新增形状时由编译器要求补齐；覆盖方法头、嵌套与 field++ 两种表示。
- [BCI 来源被误当字段语法] → 只由语义 AST 节点或既有 FieldIncrement 证明加入回执，`Fallback` 和普通 Local 不凭 origin 入集。
- [统计与物化分离再度漂移] → 两者读取同一回执和同一 claimed 集合；essential/all/range 与预算测试共同验证。
