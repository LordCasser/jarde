# Root 3.2：泛型投影预算、取消与原子发布（2026-09-24）

Root 复核并运行 `generic_method_budget` 4/4。两个冻结正例的 essential/all 完整类文本相同；`method_bodies=1` 在 `choose` 前停止时保留两个物理方法记录、`choose` 的真实 descriptor 与可写物理声明，报告 `BudgetExceeded(MethodBodies)` 且无泛型半头。预取消在类选择阶段报告 `Cancelled`，此时尚无可保留的成员记录。成功基线的用量各减一，分别约束 `attribute_bytes`、`ir_items`、`analysis_steps`、`output_bytes`：四项均在类选择之后产生 `Partial` 和相应维度的 `BudgetExceeded`，保留 `choose` 物理身份；文本只允许完整物理声明，或完整泛型声明及其正文，不接受半个方法头。

Root 使用当前 CLI SHA-256 `b50fd38aa480db684bfc92466e4da66c73959d33a0e81b1b3a7b45cc9f0b56e4` 对同一冻结类执行 `class-source --policy single-class --budget method_bodies=1`。text 与 JSON 两种格式均为 partial 的退出码 4、无泛型头、保留 `java.lang.Number choose(java.lang.Number arg0, …)` 与 `budget_exceeded_method_bodies` 停止原因。`cargo fmt --all -- --check`、`git diff --check` 与严格 OpenSpec 校验通过。

这组测试确认现有预算/停止接缝可承载已准入的静态返回子切片；它不扩大 2.3/3.1 的正文类型证明范围，也不以预取消代替方法中途取消的成员保留证明。
