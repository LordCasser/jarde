# 2.4 选择器、来源与停止验收

在 2026-09-24 以冻结的 `StringSwitchProbe.class` 对完整 `choose(Ljava/lang/String;)I` 运行公开 class-source 入口：

- essential 与 all-evidence 两种请求生成完全相同的方法正文；完整证据正文只有一个 `switch (`。
- all-evidence 来源映射覆盖了本 class 中实际被折叠的 29 个指令 BCI：selector load/store、discriminator 初始化、hash 调用与 hash switch、每个 `equals`/条件分支/discriminator 写入、join 和最终整数 switch。BCI 0 的 selector producer 映射到 selector 表达式，BCI 76 映射到输出的 String switch。
- `ir_items = 1000` 在 SSA 构建阶段以 `BudgetExceeded(IrItems)` 停止，恢复报告没有文本且 outcome 为 `Stopped`。`output_bytes = full_run_usage - 1` 以 `BudgetExceeded(OutputBytes)` 停止 class-source 请求；`choose` 的恢复正文为空或包含一个完整 String switch，不出现半个 switch。CLI 采用同样低限时在最终文档序列化收费处拒绝触碰输出文件，这一外层行为不等同于已经发布完整类。
- 在 class-source 调用前取消 token 返回 `OperationOutcome::Incomplete`，没有交付部分 class-source 报告。
- `rustfmt --edition 2024 tests/p3_string_switch_projection.rs`、`cargo fmt --all -- --check`、`CARGO_TARGET_DIR=/private/tmp/jarde-string-switch-24 cargo test -p jarde --test p3_string_switch_projection` 与 `openspec validate recover-string-switch --strict` 均通过。

边界审读：证明先发布合并后的 `Region::StringSwitch`，随后 builder 才呈现 selector。若呈现失败，当前 builder 对新区域执行 fallback；它不会退回之前两个普通 switch 区域。此轮未找到可快速复现且 JVM 验证通过、同时原两层输出可正常呈现的 selector 反例，因此没有改动产品路径。将 selector 呈现拒绝后的行为保留为 3.2 的未证明质量边界，不把它记作已证实的正确性缺陷，也不为它增加猜测性的回退机制。
