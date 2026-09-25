# 批量恢复语料基线漂移，独立于复合赋值语义

root 在复合 `+=` 的独立验收中运行 `cargo test --locked --test p5_bulk_corpus`，三项现有断言 RED：`the_fixed_shape_bills_the_counts_the_corpus_pins` 首个 `flat-mixed` 实测 `ir_items=2219, output_bytes=3657`，旧 pin 为 `1989/3676`；`every_case_holds_the_shape_it_is_named_for` 的 `many-method-class` explanation-only 从旧 6 变 5；`the_old_per_method_arms_keep_their_ledger_and_the_same_text` 在逐成员语义对照之后、账单 pin 处失败，direct 实测 `ir_items=33123, analysis_steps=11255, output_bytes=40425`，旧值 `29016/11251/41510`。这些数来自当前多项已实施恢复变更的组合，不能归因于单一复合赋值规则。

对 `Guarded.boom` 的完整类输出已用冻结 CLI 核实为 `throw new java.lang.IllegalStateException("boom");`，它不再属于旧注释列出的 explanation-only 成员；数量变化有可定位的真实语句，不能只改数字。其余账单应在生产实现稳定后由仓库 `record_the_billing_table` 测六个 case，并由 direct/shared arm 测试各自打印总账，核对完整成员文本/分类、两臂相等和未计费读取语义，再单独更新 pins 与因果注释。

此处只记录验证维护工作，不把账单重钉混入 `recover-compound-lvalue-updates` 的实现或声称全部 bulk 门禁已通过。与之不同，`p5_corpus_fingerprint` 的新增六个合法 gap class 及 patch manifest 已按仓库命令重录，root 重跑常规测试为 5 通过、1 ignored；账单的三个 RED 仍待独立语义审阅和重钉。
