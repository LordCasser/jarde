# 全工作区测试债务清单（独立于语法切片）

主线 `89100e4c` 执行 `CARGO_TARGET_DIR=/tmp/jarde-root-integration-target cargo test --workspace --all-features --no-fail-fast --quiet`，输出见 [原始日志](no-fail-fast.log)。它继续执行全部测试目标后以 101 退出：**15 个失败测试**。这不改变 CF-03/04/05 的各自三方验收结论，也不能宣称工作区测试已通过。根 Cargo target 已在复制 CLI 后 `cargo clean`，释放 21.2 GiB。

| 独立处理组 | 失败测试 | 当前证据与下一步 |
| --- | --- | --- |
| JVM IR 固定读数 | `p3_method_ir::the_payload_carries_the_tables_of_one_real_run` | 预期 handler row 0、SSA 10，实际 row 1、SSA 16；`2ad29cee` 修改了异常边/handler 行生成，须核旧基线与当前物理图后更新或修生产。Luna 独立审计中。 |
| 恢复/证据旧断言待核 | `p3_numeric_comparison::call_operands_keep_order_and_the_negative_boolean_merge_stays_quoted`；`p3_popped_static_qualifier::an_unmatched_non_invoke_qualifier_is_refused_with_its_full_source_chain`；`p3_shift_negative_boundaries::a_boolean_return_consumer_keeps_the_shift_refusal_at_its_real_bci`；`p3_throw::lower_parameter_stack_value_is_not_an_extra_throw_read`；`p3_try_local` 的 2 项；`p3_type_qualifier::static_owners_survive_generated_parameter_and_suffix_names` | 有的当前输出比旧拒绝断言更完整，有的证据请求或来源位置变化；逐测试核基线、源码可编译性、运行语义和物理 BCI，不能批量改为正例。单独于 CF-06/07。 |
| 预算与停止断言待核 | `p3_required_conversions::the_conversion_costs_no_ir_item_and_no_normalization_clone`；`p3_shift_expressions::shift_recovery_stops_cleanly_at_resource_limits_and_before_work`；`p5_bulk_corpus` 的 2 项 | 先核限额计费点和冻结 fixture，更新过时阈值或定位实际回归；不在语法点实现中调大预算。 |
| 本次运行环境 | `p5_optimize_workloads::the_three_sink_modes_publish_the_same_result` | 测试以编译时 `env!("CARGO_TARGET_TMPDIR")` 为 scratch root，并断言它不以 `/tmp` 开头；本次把 `CARGO_TARGET_DIR` 设在 `/tmp`，因此该前置断言必然失败。换用非 `/tmp` target 才能判断后续测试逻辑是否通过；这项不计作已确认的产品回归。 |
| 语料账本 | `p5_corpus_fingerprint::corpus_files_match_the_recorded_fingerprint`；`jarde-reader::classfile::tests::repository_class_fixtures_validate_without_false_target_rejections` | 本轮添加了多个 `.class` fixture，指纹清单报告 61 个新/变化文件，reader 固定人口数从 `(356,1817,160,1004,8)` 到 `(364,1852,171,1084,8)`；需用仓库既有清单维护流程逐项核对并更新，不能盲改计数。 |

这里的分类是处理顺序，不是根因结论。每项仍需在独立分支上确认前后基线、保留必要负例、针对性测试，再纳入主线；任何生产缺陷单列 OpenSpec，不混入 CF-03～08 变更。
