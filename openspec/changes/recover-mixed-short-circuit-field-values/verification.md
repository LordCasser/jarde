# 实施验收（2026-09-25）

`ShortCircuitValue` 现在以私有测试边表记录每个比较的 fallthrough/taken，并按前向拓扑和精确前驱集合收集闭合无环图。生产者极性取自解码的整数常量，不能由末次跳转的位置推断。SSA、唯一 Phi、唯一静态 `Z` 字段写入和测试值树仍逐项证明；超出展开上限或请求预算时不发布部分赋值。

- `cargo test -p jarde-java --lib --test p3_short_circuit_chain_controls`：164 个库测试、1 个三测试双消费控制通过。库测试包含已有两测试 `&&`/`||`、纯三测试 OR、非布尔生产者与独立效果拒绝。
- `cargo test -p jarde --test p3_region_owner_overlap --test p3_region_fallback_origins --test p3_short_circuit_exception_chain --test p3_short_circuit_exception_edge`：5 个额外入口、低来源预算和真实异常边控制通过；三元 OR 额外入口保持一个完整拒绝 owner。
- `cargo test -p jarde --test p3_mixed_short_circuit_field -- --include-ignored`：默认结构/来源测试及需 JDK 的完整类测试均通过。原源码以 `javac --release 8 -g:none` 重编的 class 与冻结 class 逐字节相等；恢复完整类经 `javac --release 8`、`java -Xverify:all` 后，两方法的 16 行字段值及 `b()`/`c()` 次数与冻结轨迹逐字一致。
- `rustfmt --check`（三个相关 Rust 文件）和 `openspec validate recover-mixed-short-circuit-field-values --strict` 通过。`cargo clippy -p jarde-java --lib` 与 `cargo clippy -p jarde --test p3_mixed_short_circuit_field` 退出 0，仍报告仓库其他位置的既有告警。全包 `--tests` Clippy 被并行变更的 `class_initializer_candidates.rs` 调用参数缺失阻断；本变更没有改动该文件。

保留边界：transfer-only 外部入口、真实异常边、额外 Phi 消费者、独立测试效果与未证明的值仍整体引用。基础修复的独立验收由主任务补在下段；本变更未归档。

主代理独立复验：先按 [来源阶段验收](../preserve-region-fallback-instruction-origins/verification.md)及[所有权独立验收](../refuse-overlapping-region-ownership/verification.md)确认前置负例仍是单一 whole-body 引用、16 个已解码 BCI 皆可追，故勾选 1.1。随后从当前 CLI 重新导出[完整类与报告](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-field/jarde-after-graph-report.json)，`javac --release 8` 成功，`java -Xverify:all` 的[16 行](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-field/jarde-after-graph-run.txt)同时与原 class、JADX 冻结输出逐字一致；`andOr/orAnd` 均为 `structured/java`、无 fallback。主代理再跑 `jarde-java --lib` 164/164、完整类永久回归 2/2（含 ignored JDK 项）、额外入口、异常边、Postfix 来源、loop/switch/try/jsr 与双消费者控制均通过。全局 `cargo fmt --all -- --check`、`git diff --check` 和三个相关 OpenSpec strict validation 通过。新的[短路值直接返回](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-return/analysis.md)仍是独立 parity 缺口，本 change 不延伸到 `ireturn`。

测试后以 `cargo clean --target-dir /tmp/jarde-mixed-short-circuit-target` 清理独立编译目录，共移除 5.3 GiB；临时重编目录与报告也已删除。
