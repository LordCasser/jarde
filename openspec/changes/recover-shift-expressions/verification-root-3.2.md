# Root 独立验收：移位邻接边界（2026-09-24）

Root 已用冻结的最终 CLI SHA-256 `9467c73083d721a51abae84455175980bb7a2f2b38ed29c12c7f073d3672f6a9` 独立重放 `ShiftSlice`、`ShiftCore`、`ShiftWidth` 以及有副作用的 `ShiftEffectOrder`。前三者原/JADX/Jarde 的 Java 8 完整类均重编，运行分别一致于 56、723、3 行；后者原/Jarde 三行一致，JADX 虽可重编却因字段读取前移而三行不同。布尔左值/距离及布尔返回消费者是 JVM 可校验的拒绝边界；旧局部读取保持移位前的值。这些原 class、输出哈希、BCI 和命令分别记录在 [整类重放](verification-2.2-and-3.1.md)、[求值顺序](../../evidence/java-syntax-2026-09-24/shift-effect-order/analysis.md)、[负边界](../../evidence/java-syntax-2026-09-24/shift-negative-boundaries/analysis.md)。

Root 在当前工作树复跑 `p3_shift_expressions` 5/5 与 `p3_shift_negative_boundaries` 2/2；`p3_eval_context` 10/10、`p3_boolean_contexts` 13/13、`p3_bitwise` 3/3（2 ignored）、`p3_invocation_arguments` 3/3（1 ignored）、`p3_deferred_value_order` 2/2（1 ignored）。来源/预算测试核对 BCI 1、5 的 helper 与 BCI 8 的移位、同一物理方法身份、essential/all 文本和资源停止；移位没有新增独立求值机制。

相邻全套并非绿色，且不能算作移位回归已修：`p3_numeric_comparison::call_operands_keep_order_and_the_negative_boolean_merge_stays_quoted` 因旧断言期待 BCI 2、当前拒绝落在返回 BCI 11 而失败，此项在 3.1 原验收已列为既存问题；`p3_required_conversions::the_conversion_costs_no_ir_item_and_no_normalization_clone` 当前 `ir_items=234`，旧固定计数为 224。后者调用独立 `Engine::recover_method`，不经过泛型方法头侧证据；本项只记录差异，原因另行审计。两项均不改变移位四组冻结类的重编执行结论，也不在移位 change 中改动其它规则的预算或来源契约。

`openspec validate recover-shift-expressions --strict` 通过。3.3 的整仓 census/fingerprint、格式与严格 Clippy 集成门禁另行记录，不能由本页代替。
