# 验证记录

- 全仓 Rust 调用搜索：唯一的两参数测试调用已补齐；所有 `recover_for_class_source` 测试调用均为三参数。
- `cargo fmt --all -- --check`：通过。
- `openspec validate --strict align-class-source-test-callers`：通过。
- 使用 `CARGO_TARGET_DIR=/tmp/jarde-sidecar-test-target` 运行受影响测试：目标能编译。`class_initializer_candidates` 有 2 个失败（`initializer_sidecar_is_limited_to_non_annotation_interfaces` 的 usage 计数比较；`candidate_collection_budget_refusal_is_a_real_stop_without_a_partial_sidecar` 的既有预算停止位置断言）；其余 8 个通过。`p3_patterns` 有 1 个失败（`an_invocation_argument_keeps_its_recovered_cast` 的 cast 断言），其余 52 个通过。失败行为分别涉及候选预算/报告 usage 与调用参数恢复；本 change 只补齐调用参数且不改生产实现。

Cargo target 保留在 `/tmp/jarde-sidecar-test-target`，供 root 独立复验与清理。

Root 复核：8 个调用点均显式传 `false`，与 `recover_for_class_source` 的参数语义一致——本组测试不请求泛型返回/构造器候选。Root 独立重跑三个失败用例：非接口 `<clinit>` 的恢复正文相同，`ir_items` 为 102 对 86，旧断言错误地要求普通 recovery 与收集其它类级 sidecar 的适配器整份报告相等；预算用例的停止位置不满足旧 `at: Some(5)` 钉值，需单独判定实际停止位置；P3 cast 用例输出局部声明范围引用而非预期调用，属于独立恢复/fixture 问题。此 change 的目标仅为恢复测试编译，未更改生产恢复。Root 清理 `/tmp/jarde-sidecar-test-target`，Cargo 报告移除 2,264 个文件、880.8 MiB。
