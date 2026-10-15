# Root verification

## 验收范围与结论

根 focused 验收已接受，本记录覆盖 `preserve-emitter-output-stop-reasons` 的本地候选 focused scope。它不代表必要 workspace 全量门已通过，也不代表实现提交自身 CI 已通过；两者仍待完成。归档由根维护于 [`results/root-focused-v1/`](results/root-focused-v1/)，未记录未经核实的归档摘要。

变更将 emitter 输出许可错误保留为真实停止原因：只有真实 OutputBytes 额度拒绝才发布该 Budget，并保留已写字节、限制与当前位置；取消保持 Cancelled，elapsed deadline 保持既有 Interrupted。失败会丢弃未提交文本；此前已成功写入的费用不退还。共享 `stop::charge` 的 deadline 分类一并修复。此项不改变全局额度许可/first-stop 语义，不扩展 Java 恢复准入，也不替代其它 change 的验证。

## 旧 production 根复现

根复现收据 `emitter-repro-root-review-v1.json` 状态为 `accepted_root_old_production_counterexample_only`，基线为 `d6fba33aa24b0dd0b55d9ca6f05a1d4f77fadeea`。它在旧生产源码闭包上仅加入 tests-only harness，执行三项精确回归：

`cargo test -p jarde-java --lib --locked --message-format json -- --exact emit::tests::output_admission_preserves_cancellation_after_initial_poll emit::tests::cancellation_after_a_prior_write_discards_text_and_keeps_only_prior_charge stop::tests::deadline_inside_budget_charge_keeps_the_interrupted_reason --nocapture`

三项均如预期失败（0 passed、3 failed）：两个检查内取消场景被错报，且真实 `elapsed_millis=0` 的 deadline 对 OutputBytes 与 IR 两个请求维度均实际执行，错误观察为 Budget 而非 Interrupted。收据确认生产源码前缀匹配 d6f 基线、4,887 文件源码闭包稳定，finally `cargo clean` 成功且 target 不存在。这是旧实现反例，不是候选的失败或通过记录。

私有候选历史曾有 Java lib 420 passed/3 failed，原因是三处既有 quota fixture 默认 `elapsed_millis=0`，被即时 deadline 抢先拒绝；私有验证仅将这些 quota fixture 的 elapsed 限制设为 `u64::MAX`，保留原 quota 数值和断言。私有记录另报告修后 Java lib 423 pass。上述属于 reviewer 私有候选历史，不是根复跑，本记录不把它计入根验收。

## 候选范围与源码闭包

候选补丁 SHA-256 为 `78fac16f36e2f75aaeea0633c31f09947659fd5ad0e6f9a04e2134c6eac4e3d0`。独立只读审查确认补丁与候选工作树四文件内容匹配、未发现阻断问题。根 focused 收据的 4,887 文件源码闭包在运行前后稳定，四个文件最终 SHA-256 如下：

| 文件 | SHA-256 |
| --- | --- |
| `crates/jarde-java/src/emit.rs` | `07ad789e67a0861a5e5400842b7c0c910bf04d2fb83da65cb387d18ea0780c31` |
| `crates/jarde-java/src/stop.rs` | `d3833ac7e623ab2347f535bd0d0b8860b16853a69241d4d57113c307c8041792` |
| `crates/jarde-java/src/bridge.rs` | `65cd1f4e2e6cda9e763036f3d5bafd92b2d8e263bedd684e7edf0189fac60bcc` |
| `crates/jarde-java/src/report.rs` | `6871a5ae95b2e725bf70834864f096ace19e1b321c253bb48d258b75ecfe93ca` |

`bridge.rs` 一处和 `report.rs` 两处的 `elapsed_millis: u64::MAX` 只隔离 quota fixtures 的 elapsed 维度，让各自原有 IR/analysis quota 断言测到目标限制；没有提高原额度、改期望或放宽断言。生产 deadline 由零 elapsed 的确定性测试覆盖，不需要 sleep。

## 根 focused 执行

执行收据 `emitter-focused-v1/execution.json` 共有 9 条命令、18 条原始输出流；根协议审查 `emitter-focused-v1/protocol-review.json` 状态 `accepted`，4,962 checks、4,887 个源码文件、440 passed、0 failed、0 ignored。结果构成为 Java 库 424、report/content 4、declaration handoff 10、class-source 停止 2。

命令及结果：

1. `cargo fmt --all -- --check` — 通过。
2. 项目 CI 对应 `cargo clippy --workspace --all-targets --all-features --locked ... -D warnings` — 通过，使用项目现行 Clippy allow 列表。
3. bundled OpenSpec CLI `openspec validate --all --strict --no-interactive`（1.11.0）— 通过。
4. local OpenSpec CLI `/home/lordcasser/.local/bin/openspec validate --all --strict --no-interactive`（1.14.1）— 通过。
5. `cargo test -p jarde-java --lib --locked --message-format json` — 424/0/0。
6. `cargo test -p jarde --test p3_content --test p3_declaration_handoff --all-features --locked --message-format json` — 两目标分别 4/0/0、10/0/0。
7. `cargo test -p jarde --test class_source --all-features --locked --message-format json -- --exact anonymous_superclass_budget_and_cancellation_never_publish_partial_source anonymous_interface_projection_cancellation_never_publishes_partial_source` — 2/0/0。
8. `cargo build -p jarde-cli --locked` — 通过。
9. `cargo clean` — 通过；清理后 target 不存在。

收据记录 Rust stable 预期 1.99.0、OpenSpec 1.11.0 与 local 1.14.1，并使用 Codex bundled Node runtime；没有据此声称与 CI 在线工具链完全等价。构建守护阈值为至少 5 GiB 可用磁盘、target 不超过 1 GiB，每秒检查；finally `cargo clean` 的结束状态为成功且 target absent。

这些测试支持停止分类、输出许可、部分写入丢弃、source/report 不发布以及相关 Java 库路径。测试结果不代表完整 workspace 门。根收据在私有缓存 `/home/lordcasser/.cache/jarde-validation/core-integrity-20261015/` 下；此处不附未经核实的私有归档摘要。

## 任务状态建议

- 1.1 已完成：旧 production 根复现实际重现三项错误，取消时点与两个 deadline 计费维度均有根证据。
- 1.2 已完成：根 Java 库及 report/source-stop 测试覆盖修复行为，额度 fixtures 仅隔离 elapsed 维度，保留原 quota 与断言。
- 2.1 保持未完成：fmt、CI Clippy、两个 OpenSpec strict、相关库/report/source-stop 测试、CLI build 与清理均通过；但任务要求的必要 workspace 检查尚缺。
- 2.2 保持未完成：独立切片提交及该提交自身 CI 尚无通过证据。

归档已逐项复开核验：[root-focused-v1/README.md](results/root-focused-v1/README.md)，`capture.tar.gz` SHA-256 `6888c9363c212ca6ce21d6add451575c8494062bd7b9e53087f7976d143a1384`，42 payload。
