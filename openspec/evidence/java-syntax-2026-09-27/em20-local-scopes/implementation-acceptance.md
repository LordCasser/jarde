# EM-20 同步 guard 槽位复用实现验收

本文件记录 [split-proved-monitor-slot-reuse](../../changes/split-proved-monitor-slot-reuse/) 的实现验收；[report.md](report.md) 与 [acceptance/](acceptance/) 继续保留修前事实。实现只在已证 `Shape::Monitor` 的精确清理异常表行内准入异常边；引用访问必须全部落在该 guard 的物理拥有块，后续整数访问必须在 guard 外。循环 φ 的每个输入须来自后段已核验写入，φ 块与后段访问块都参与“不可回到前段”检查。其它异常边、Call/Return 边及证据不完整的槽位仍拒绝。

以 `CARGO_TARGET_DIR=/tmp/jarde-em20-impl-target cargo build -p jarde-cli` 构建的 CLI SHA-256 为 `c9e710ca111a762326be83d60c07d9305e8c2f319a05a7979c2f9f9155f1ca0b`。[accepted/summary.json](accepted/summary.json) 保留固定 JADX 修订及五个测试、四个生产入口、输入/原 class/CLI 哈希。用同一 [replay.py](replay.py) 分别写入 `accepted/` 和第二个空目录，两次摘要与 JADX/Jarde 完整类源码逐字节相同。原 class、固定 JADX、修后 Jarde 的完整 `LocalScopes` 与共同 Runner 均以 `javac --release 8 -g:none` 重编，并经 `java -Xverify:all` 输出相同的七行：`5, 3, 0, 10, 6, 2, 3`。Jarde 的 `synchronizedLoop` 将 `local2_2` 和 `local3_2` 分别声明后在同步区域外使用，循环和返回不再丢失；完整文本见 [accepted/source/jarde-LocalScopes.java](accepted/source/jarde-LocalScopes.java)。

定向 `p3_scope` 回归以冻结的 [Java 8 class 与来源](../../../../tests/fixtures/em20-monitor-reuse/README.md) 验证两个正例槽位和三个拒绝控制：普通 catch 的异常边，后续 catch 对复用整数的读取，以及外层循环从后段回到同步 guard。预算测试在已经完成 IR 分析、正在计费 monitor handler BCI 14 时停止，确认不发布半份源码；预取消请求的分析层保持 `Cancelled`，正文为空。`p3_guard` 与 `p3_sync_return` 保持既有 monitor/资源拒绝边界。本验收不覆盖其它 EM-20 Smali/泛型样例，也不将循环样式差异视作作用域恢复。

相关测试 `p3_scope`、`p3_guard`、`p3_sync_return`、`p3_java_recovery`、`p3_patterns` 与 `class_source` 均通过；`cargo check --workspace`、`cargo fmt --all -- --check`、`git diff --check` 和 OpenSpec strict 校验亦通过。额外运行 `cargo test --workspace --all-features` 时，只有 `member_family_identity::outer_super_method_bridge_is_not_projected` 失败；从未包含本次实现的审计提交 `dd3f4fea5fd978a4ec4d3e83ca9ee23323b1182e` 独立归档源码中单测也以同一断言失败，因此不属于本次同步槽位分割。
