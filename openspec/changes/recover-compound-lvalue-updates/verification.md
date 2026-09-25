# 验收记录

## 结果与边界

root 审阅了字段/数组写入从最终 store 反向建立的同块证明：Fieldref 身份、`I`/`[I` 限定、SSA 复制值与单一消费者、接收者/下标/RHS 完整依赖区间，以及区间内独立效果拒绝。只有整条链成立才认领 `dup`/`dup2`、旧值读取、RHS、`iadd` 与写入，并发射结构化 `+=`；未证明的形状保留原引用和 BCI。预算按已扫描的 SSA use 计费，取消和 source-map 走既有停止路径。未引入通用栈复制机制。

root 在最后一版生产代码及 Clippy 修正后重新构建并冻结 CLI，SHA-256 为 `7a33dbed5b390009cb65802fd9264367e1d5854b9cc70dbced5ae1878109c822`；回放前后哈希相同。完整 `CompoundProbe.class` 为 1113 字节、12 个 Code 方法，SHA-256 为 `889f36d3a5c06951d32762c8914829c059d8b1d6692001c08df73404c92f673f`。原 class、JADX 与 Jarde 的**完整** Java 8 类均经 javac 和 `java -Xverify:all`，七行输出逐行相同，包括接收者/下标单次求值、同左值快照及 null/越界时 RHS 不执行。永久脚本、三方生成源码、编译/执行日志和最终 `summary.json` 在 `../../evidence/java-syntax-2026-09-22/compound-assignments/post-fix-fixture-replay/`。

19 个边界的原 class 与 JADX 全部等价，Jarde 有 6 个等价。其余 13 个不被误认成 `+=`，来源锚点保留；但部分引用正文即使碰巧通过 javac，运行仍不等价。比如字段复制前插入独立效果时，原 class 为 `value=9:select=1:rhs=2`，Jarde 为 `value=7:select=1:rhs=1`。这些是拒绝边界和后续 class-source 状态契约债务，**不计为语义恢复成功**；见 `../../evidence/java-syntax-2026-09-22/architecture-debt.md`。

## 门禁

- `cargo test -p jarde-java --locked`、复合赋值/字段自增/数组访问/局部改写/延期求值定向集成测试通过；JDK-only 延期顺序测试显式执行通过。
- `cargo test -p jarde-reader --locked` 160 项通过。六份新 JVM 合法 gap class 使固定 census 从 `(123, 896, 98, 340, 8)` 变为 `(129, 1040, 98, 346, 8)`，已按实际 class/Code/handler/branch/subroutine 数更新。`p5_corpus_fingerprint` 常规测试 5 通过、1 ignored，指纹包含新增 fixture。
- `cargo fmt --all -- --check`、`git diff --check` 通过。`openspec validate --all --strict` 当次 62/62 通过。
- 初次严格 Clippy 发现既存 `region.rs:1736` `type_complexity` 及本项 `collect_expression_bcis` 的两个新警告。后两项已通过局部 `ExpressionBciContext` 整理参数和直接返回 `Ok(true)` 修正；复合定向测试重跑 4/4。`cargo clippy -p jarde-java -p jarde-reader --all-targets --locked -- -D warnings -A clippy::type_complexity` 通过，唯一豁免是既存 `region.rs` 警告；源码未添加 lint allow。
- `p5_bulk_corpus` 当前另有 3 个 RED：`Guarded.boom` 已从 explanation-only 迁移为可恢复 `throw new`，flat-mixed 与 direct arm 计数 pin 漂移。语义审阅和重钉属于独立验证维护，记录在 `../../evidence/java-syntax-2026-09-22/bulk-recovery-pin-drift/analysis.md`，未混入本项功能改动。
