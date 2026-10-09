# Finally tight-budget pin migration v1

将 `crates/jarde-java/tests/p3_patterns.rs` 中 `straight_finally_reuses_its_guard_verdict_with_a_tight_budget` 的固定值从 52 改为 61，并把断言说明改为 ordinary Site census 的逐条 SSA instruction 计费。

依据如下：

- 分区门禁 `tests-021/seed-5350648285461741569/stdout` 的实际失败行记录 `full_budget.usage().analysis_steps == 61`，旧断言预期 52。只迁移这个 AnalysisSteps 固定值。
- 冻结的 `FinallyStraightThrow.class` `javap.txt` 中 `run()I` 有九条物理指令，BCI 为 `0, 3, 4, 7, 8, 9, 10, 13, 14`。`init::sites_with_pending_array_composition` 在普通 Site census 中对每条 SSA 指令 charge 一次 `AnalysisSteps`（`init.rs` 577–582）；这段方法没有 `new`，所以不会进入 `verify_metered` 的构造器候选验证。九条指令正好解释 `+9`。
- `report::recover` 先运行 Site census，再调用 `region::recover`。后者在当前 guard 决策点调用一次 `guard::examine`，并用同一个 `guard_verdict` 继续结构化 finally 分支（`region.rs` 2752–2788）。因此新增的九步发生在 region/guard 处理之前，证据支持它们来自 census，而不是第二次 guard proof。
- 保留 `assert!(full.produced())` 与 `assert_eq!(tight.text, full.text)`。前者仍确认完整预算产出，后者仍让紧预算验证相同正文；本次未运行 focused test，因此不记录为已重验通过。

本测试只固定 `AnalysisSteps`。虽然所审查 census 新增的 charge 使用该维度，且 `run()` 无 allocation candidate、不调用构造器 verifier，本次没有独立读取或实测其它 Budget usage 维度，故不对其它维度是否变化作结论。没有检索到需迁移的其它 52 / AnalysisSteps 固定值；搜索范围为 `crates/jarde-java/tests` 中 `steps, 52`、`analysis_steps.*52` 与 `AnalysisSteps.*52`。
