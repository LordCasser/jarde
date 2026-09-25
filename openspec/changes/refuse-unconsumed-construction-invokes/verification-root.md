# Root 验收：构造区间中的独立调用

## 结论与对照

[冻结的 JVM 可验证输入](../../evidence/java-syntax-2026-09-25/ordinary-new-void-effect/analysis.md)中，`VoidBetween.make` 的 BCI 为 `0 new Target; 3 dup; 4 Side.effect()V; 7 iconst_1; 8 Target.<init>(I)V; 11 areturn`。输入 class 的 SHA-256 为 `cba6ef63e220e3b8ec1d1abead07b2e7acb93182b8790bb6ced27b5a237989b7`。Root 独立重编和 `java -Xverify:all` 重放确认：原 class `CST`，JADX 1.5.6 旧源码 `SCT`，Jarde 旧源码 `SCT`。因此不能以“能编译”或旧报告的 `structured/presented=true` 作为语义验收。

本变更在既有 `new@1` 中从构造器物理实参沿 SSA 值定义追溯区间调用；没有实参依赖的调用拒绝。Root 重新构建 CLI 后，对同一输入分别请求 essential、all、`--evidence all --evidence-bci 0..11`，冻结[三份结果](../../evidence/java-syntax-2026-09-25/ordinary-new-void-effect/analysis.md)：三份正文一致，均为 `mixed/fallback`，诊断 `jre_new_interleaved_effect` 点名 BCI 4。all/range 的 BCI 0 记录为 `presented=false`；essential 按选择规则不物化该详情。正文保留 BCI 0、3、8 的 bytecode 引用和 BCI 4 的调用，未发射错误的 `return new Target(1);`。真实调用实参的 `CountingRunner.check` 控制仍被接受，调用在构造表达式中各出现一次。

## 验证

- `cargo test --locked -q -p jarde --test p3_ordinary_new_invokes`：2/2，通过负例、正例、质量和 BCI 来源验收。
- `cargo test --locked -q -p jarde --lib`：23/23；其中成员目标事实的预算与取消测试也通过。
- `cargo test --locked -q -p jarde --test class_source`：47/47；`cargo test --locked -q -p jarde --test d1_evidence_selection`：8/8；`cargo test --locked -q -p jarde-java --lib`：176/176。
- `cargo test --locked -q -p jarde --test p3_compound_lvalue_updates`：4/4，其中既有证据阶段预算/取消停止控制通过；本修复只读取已发布的 SSA，未新增预算操作。
- CLI `--budget result_items=1` 对同一 JAR 返回 `outcome=incomplete`、`execution.status=partial`，没有成员正文或无证明构造结论；预算充足时的三种证据选择保持上述同一判定。
- `cargo fmt --all -- --check`、`git diff --check`、`openspec validate refuse-unconsumed-construction-invokes --strict` 均通过。

全套 `jarde-java` integration 测试在共享工作树中的 `p3_patterns.rs:389` 因旧 `recover_for_class_source` 两参调用与现三参 API 不匹配而不能编译；整个 `jarde` 测试集合还被 `bulk_recovery_cancel.rs` 使用已不存在的 `BulkProbe`/`with_probe` 阻断。本变更未触碰这些文件，已用上述定向及相邻测试验收。严格 Clippy 在 `enumswitch.rs`、`region.rs`、`report.rs`、`build.rs`、`reuse.rs` 的 17 项既有警告处停止，未报 `init.rs` 新代码问题；这些债务另案处理。

本任务的代理私有 Cargo target 已清理；Root 随后对 `/tmp/jarde-void-new-target` 执行 `cargo clean`，删除 9804 个文件、约 3.9 GiB 构建产物，磁盘可用空间从约 77 GiB 回升至 80 GiB。
