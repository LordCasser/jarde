# 验证记录

## 实现范围

本次只完成任务 1.2、2.1–2.3、3.1。`field_value` 先沿用已有位置转换规则；只有真实 `putfield`/`putstatic` 或已验证的 synthetic accessor 写入、字段描述符为 `B`/`C`/`S`、且值事实为 int-shaped primitive 时，才在实际写入 BCI 使用已有 `Cast`。`Z`、未知值和通用调用参数仍走原拒绝边界。字段写入失败改用既有 `quoted_bcis(at)`，保留生产者与 put 指令来源。

## Cargo 测试

使用 `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1` 串行运行：

- `cargo test -p jarde --test p3_narrow_field_stores -- --nocapture`：2 passed，1 ignored；ignored JDK 对照另行通过。
- `cargo test -p jarde --test p3_narrow_field_stores recovered_narrow_field_stores_match_patched_runtime -- --ignored --nocapture`：通过；267 行 recovered 输出与 patched JVM 输出一致。
- `cargo test -p jarde-java --test p3_patterns -- --nocapture`：48 passed；包含 verifier-valid B accessor 写入正例及既有无效 accessor 拒绝正例。
- `cargo test -p jarde --test p3_accessor_edges -- --nocapture`：4 passed。
- `cargo test -p jarde --test p3_field_increment -- --nocapture`：2 passed。
- `cargo test -p jarde --test p3_array_access -- --nocapture`：11 passed。
- `cargo test -p jarde-java --lib`：104 passed。

root 复跑相邻 `p3_accessor_edges` 4 项、`p3_array_access` 11 项、`p3_deferred_value_order` 2 项、`p3_field_increment` 2 项、`p3_invocation_arguments` 3 项、`p3_narrow_field_stores` 2 项、`p3_partial_array_allocation` 2 项、`p3_reference_cast` 5 项、`p3_required_conversions` 8 项，均通过；五个需要 JDK 的运行对照在默认 Cargo 运行中按既定标记忽略，字段和部分维度数组已单独执行。root 复跑 `jarde-java --lib` 104 项、`p3_patterns` 48 项通过。`p3_narrow_integer_returns` 当前两个预期 RED 用例属于尚未实施的独立 change，不计入字段回归通过项。

`p3_required_conversions` 最初 7/8：`castPart(C)` 没有字段写入，但其旧的 `ir_items=140` 断言未包含已验收的 deferred-binding SSA 扫描。root 将该测试基线修订为实测 224，并明确扫描按 SSA value 收费；`normalization_clones=0` 与转换文本 224 字节保持原断言。修订后 8/8 通过，未改变字段生产代码以掩盖预算差异。

## 真实 class 重放

冻结 CLI 为 `/tmp/jarde-cli-narrow-field-stores-48ed`，SHA-256 为
`48edb9d2e3eec451983aabb4affcbcaf6d723c8a75b0284605f729743088cc76`。两个审计脚本均用该 SHA 前后校验：

- `openspec/evidence/java-syntax-2026-09-22/numeric-conversions/narrow-field-stores/p3-core/run_audit.py`：源/补丁 class 均 `javac --release 8`、`java -Xverify:all` 成功；267 行 patched JVM 与 jarde 输出逐字一致；jarde `@bytecode` 为 0，完整 jarde `javac`/运行成功。JADX 完整源码 `javac` 状态为 1，未作为运行 oracle。汇总见同目录 `summary.json`。
- `openspec/evidence/java-syntax-2026-09-22/numeric-conversions/narrow-field-stores/core-bcs/run_audit.py`：285 行 patched JVM 与 jarde 输出逐字一致（B/C/S 各 95 项），jarde `@bytecode` 为 0，完整 jarde `javac`/运行成功；49 个 Code 属性字节完全不变。JADX 完整源码 `javac` 状态为 1。汇总见同目录 `summary.json`。

两套输入均由同一精确字段描述符补丁生成，原 class 与 patched class 均经过 `java -Xverify:all`；来源、边界异常身份、producer 次数和 null receiver 顺序由永久 runner 的逐行输出校验。完整 380 项含 `Z` 的低位语义仍是独立边界，本次没有放宽该规则。

root 在 `root-p3-48ed/` 和 `root-core-48ed/` 从冻结 CLI 独立重放：267/267 与 285/285 行分别与 patched JVM 逐字一致，整类 `javac`/`java -Xverify:all` 成功，jarde 引用 0；JADX 整类 `javac` 均失败，未臆测其运行结果。另在 `root-z-48ed/` 重放完整 380 行：B/C/S 的 285 行全部相同，Z 的 95 行有 70 行不同。这里 jarde 整类仍 `javac`/JVM 成功，因为拒绝后的方法可编译为空操作；`Z` 值、null 异常和 producer 调用均可能不同。它是待独立处理的布尔存储恢复债务，不能算本 change 成功，也不能以编译通过当作语义正确。

root 最终门禁：reader 的仓库 class census 通过；P5 corpus fingerprint 259 文件、5 项通过 1 项显式重生成忽略；`cargo fmt --all -- --check`、`git diff --check` 通过；`openspec validate --all --strict --no-interactive` 为 52/52。当前磁盘可用约 24 GiB，未扩大 Cargo 并行编译。
