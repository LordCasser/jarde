# recover-loop-body-try-catch：候选 focused 与 whole-class 对照取证

本文是 `candidate-focused-and-comparison-root-v1` 证据包的记录。结论严格限于已归档的 focused 命令和特定对照 fixture，不将其外推为完整产品验收。

## 可接受的结论

Focused 独立验收状态为 `accepted_focused_commands_only`。归档的 v5 执行记录显示 `cargo test -p jarde-java --lib --locked` 的真实汇总为 **385 passed, 0 failed, 0 ignored**；同一执行中格式化、测试和 CLI 构建命令均退出 0，CLI 为 62,018,176 bytes，SHA-256 `d632573c91509d6630dd4556a6a89f10bca2fe267514eec5b50dfc123d67b9ac`。记录的源码清单为 **4,840** 项，target absent 为 true。focused 验收明确标记 `full_class_runtime_accepted: false`。

Whole-class 对照共 **40 条命令、80 份 raw stream**，命令/编译/运行退出码均为 0。这只是观察命令成功。结果是：DV/DF 的四个 profile/JDK 组合均与原始运行输出逐字节一致；Retry v3 的四组也匹配；Retry v2 的四组全部不匹配。Retry v2 的差异原因已记录为 `exc()V` 缺少嵌套 `AssertionError` 消息构造，导致 uncaught-error case 错误。因此 `implementation_mvp_accepted` 为 false，产品验收仍拒绝。

## 验收边界与未完成项

此包捕获时任务为 **2/8**。后续补充已关闭2.2控制文本与2.3引用Load，父项当前 **4/8**；两个fragmented整类text及八retry transfer map见 ../control-text-and-transfer-map-root-v1。完整 workspace 双seed、Clippy（当前CI）、新增retry内部Stop及整个source-map门和自身CI仍未闭合。Focused 的385项不能替代这些工作，也不能使retry-v2的mismatch变为通过。

保留 focused 早期失败记录，避免把迭代过程误写成全程通过：v1 在 `cargo fmt --all` 退出 1 后停止，随后 `cargo clean` 退出 0；v2 格式化成功，但 lib test 退出 101；v3 格式化成功，lib test 汇总为 382 passed、2 failed、0 ignored，随后 clean 成功。v4/v5 的执行记录均为四条 focused 命令全零退出；v5 才有独立验收摘要和 CLI 指纹。

## 复核位置

证据包：`openspec/changes/recover-loop-body-try-catch/results/candidate-focused-and-comparison-root-v1/capture-and-tooling.tar.gz`

压缩包大小：**4,189,982 bytes**

压缩包 SHA-256：`74ddc48fe7a69737dad44747efe86da12ccffe611a22899a65045feb56309905`

清单：同目录的 `manifest.json`。以下是压缩包内可直接复核的条目路径：

- Focused v5 命令、测试汇总、源码清单和 CLI：`focused/v5/execution.json`、`focused-acceptance/summary.json`、`focused/v5/raw/2.stdout.raw`。
- Focused v1–v3 失败轨迹：`focused/v1/execution.json`、`focused/v2/execution.json`、`focused/v3/execution.json`；原始输出分别在对应 `focused/vN/raw/` 目录。
- 40 命令/80 raw streams 与逐组完整类对照：`comparison/execution.json`；摘要：`comparison/root-observation-summary.json`。
- 候选源代码快照：`candidate-source/crates/jarde-java/src/build.rs`、`candidate-source/crates/jarde-java/src/region.rs`、`candidate-source/crates/jarde-java/src/region/finite_loop_catch_tests.rs`。

包内清单为归档 entry 给出各自 byte count 与 SHA-256；应以这些 entry 指纹和上述 archive 指纹作为复核基准。对照 summary 的状态是 `accepted_observations_only_product_rejected`，不得改写成产品通过。
