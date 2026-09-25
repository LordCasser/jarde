# Root 独立验收

结论：接受 `report-committed-field-presentations` 的 6/6 项实现。`field@1` 的先验 claim 仍为 Builder 提供成员/形状证明；`report` 在完整 AST 发射成功后遍历最终语义节点，所得回执同时驱动 `FieldRecord.presented`、未发射理由和全方法摘要。遍历只认本方法 direct BCI 与对应的读写方向，引用区和合成 accessor 不构成字段呈现；`field++` 的旧式局部返回形状由原计划及最终返回来源互证。停止时不发布部分回执。

独立重建 CLI 的 SHA-256 为 `875020286a06f465f48c842735695592cfa514b1f0d50f1b217d027fdc6c846e`。用它重新生成[原 class 报告](../../evidence/java-syntax-2026-09-25/field-presentation-contract/root-final-base.json)、[额外入口报告](../../evidence/java-syntax-2026-09-25/field-presentation-contract/root-final-multi.json)及[异常边报告](../../evidence/java-syntax-2026-09-25/field-presentation-contract/root-final-exception.json)：原 `assign` BCI 30 为 true，有真实 `ChainExtraBoundary.result =` 和 `1 presented, 0 refused`；两个控制均为 false、`jre_field_not_emitted`、无该赋值、`0 presented, 1 refused`，但 BCI 30 仍在 source map。两个控制保持 mixed/not_java 的保守 Region 拒绝，没有改动其判定。

Root 重跑 `jarde-java` 的 5 个 `field::tests`，以及 `jarde` 的 `p3_field_presentations`、`p3_eval_context`、`p3_field_increment`、`p3_compound_lvalue_updates`、`p3_final_static`、`p3_postfix_lvalue_values`、`p3_region_owner_overlap`、`p3_region_fallback_origins`：所有非 ignored 测试通过。定向单测分别证实交错拒绝按 BCI 排序，以及回执遍历中预算耗尽、预取消都返回 Stop 而不返回部分集合。`cargo fmt --all -- --check`、`openspec validate report-committed-field-presentations --strict`、`git diff --check` 通过。`cargo clippy --locked -q -p jarde-java --lib` 成功退出，报告 17 条已有告警，均不在新增 `field.rs` 回执。另一个 accessor clone 的 BCI 重复断言仍失败；其 fixture 无字段，作为独立来源映射债务，不混入本 change。

剩余边界：回执只检验 `field@1` 本方法已认领指令是否存在于最终字段语义节点，不承担跨方法 accessor 字段或源码可编译性的全局证明。其他规则是否也把先验计划误当已发射事实，须分别审计。Root 私有 Cargo target 在验收后清理。
