# 验证记录

## 实现边界

`Builder::invocation_argument` 只在当前恢复配置的 Java release 为 8、已呈现源类型完整名为 `java.util.List`、descriptor 目标完整名为 `java.lang.Iterable` 时接受标准平台上溯。它复用既有 `cast_argument`，目标 descriptor 仍决定参数类型；cast 包住原表达式一次，并保留 producer 与 invoke 的来源。单元测试同时拒绝其它 release、短名相似类型、`ArrayList`、`Collection`、参数化拼写、反向关系及无关目标类型。未证明用户引用关系仍经过原拒绝分支。

## 定向运行证据

输入只有 `consume(Iterable)` 与 `main` 中一次 `Arrays.asList` 调用，不含循环、重载或泛型 helper。`array_invocation_widening` 集成测试确认恢复表达式保留物理目标类型 `Iterable`，producer `asList` 仅出现一次；source-map 中 producer span 保留 BCI 19，调用 cast span 保留 invoke BCI 22。该测试还确认同一输入在输出预算不足和预先取消时不发布文本或 source-map。相邻测试覆盖数组到 `Object`/marker interface 的既有上溯、重载保留、效应实参只出现一次，以及未证明用户数组层级仍整体 fallback。

完整类由原始、JADX 与 Jarde 源分别以 `javac --release 8 -g -Xlint:-options` 编译，再用 `java -Xverify:all` 运行；三方 stdout 都是单行 `called`。原始类重编后与记录的输入 class SHA-256 相同。JADX 版本为 1.5.6，JDK 为 23.0.1。

复现命令（从仓库根目录运行；Jarde 源先由 `class-source` 命令生成）：

```sh
CARGO_TARGET_DIR=/tmp/jarde-cf10-list-target CARGO_INCREMENTAL=0 cargo run -q -p jarde-cli -- class-source --input openspec/evidence/java-syntax-2026-09-27/cf10-foreach/isolation/list-iterable-call/original/ListToIterable.class --class ListToIterable --policy single-class --release 8 --format text > /tmp/ListToIterable.java 2> /tmp/ListToIterable.report
javac --release 8 -g -Xlint:-options -d /tmp/cf10-list-accept/original openspec/evidence/java-syntax-2026-09-27/cf10-foreach/isolation/list-iterable-call/original/ListToIterable.java
jadx -d /tmp/cf10-list-accept/jadx-output /tmp/cf10-list-accept/original/ListToIterable.class
javac --release 8 -g -Xlint:-options -d /tmp/cf10-list-accept/jadx-classes /tmp/cf10-list-accept/jadx-output/sources/defpackage/ListToIterable.java
javac --release 8 -g -Xlint:-options -d /tmp/cf10-list-accept/jarde-classes openspec/evidence/java-syntax-2026-09-27/cf10-foreach/isolation/list-iterable-call/jarde/ListToIterable.java
java -Xverify:all -cp /tmp/cf10-list-accept/original ListToIterable
java -Xverify:all -cp /tmp/cf10-list-accept/jadx-classes defpackage.ListToIterable
java -Xverify:all -cp /tmp/cf10-list-accept/jarde-classes ListToIterable
```

## SHA-256

| Artifact | SHA-256 |
|---|---|
| Original source | `a6880b2a064ec1390bca8245101c3bcdf0ac58d01b69cc0c5d0f4ce69eedb274` |
| Original class, including Java 8 recompilation | `bbae590902f24b48fa064c25b59247ec71fcb1534ec0818a54ce437b5f1ba0ce` |
| JADX source | `9ac68d263a1ef3e8e0af896fdba0161982de5989d410a51d30b8bb23c8ef0e62` |
| Jarde source | `16da9f57b4555c3eebe8f5f64443764baec74137dbce4498db2d47a114aa86b7` |
| Original stdout | `04d540922ee4c86ca1c488ee84bd212f8ec5deceb222eef48701938a669394c3` |
| JADX stdout | `04d540922ee4c86ca1c488ee84bd212f8ec5deceb222eef48701938a669394c3` |
| Jarde stdout | `04d540922ee4c86ca1c488ee84bd212f8ec5deceb222eef48701938a669394c3` |

## Rust 与 OpenSpec checks

- `cargo fmt --all -- --check` 通过。
- `cargo check --workspace --all-targets --all-features --locked` 通过。
- `cargo test -p jarde-java --test array_invocation_widening`：6 项通过；包括新加入的 List 实参 source-map、单次求值、预算和取消断言，以及原数组、重载和拒绝回归。
- `cargo test -p jarde-java --test p3_conditional_values`：2 项通过。
- `cargo test -p jarde --test p3_iterable_foreach`：5 项通过；`cargo test -p jarde --test p3_loop_test_values`：4 项通过。
- `cargo test -p jarde-java platform_reference_argument_widening_is_one_java_8_relation`：准入边界单元测试通过。
- `openspec validate preserve-list-iterable-invocation-widening --strict --no-interactive` 通过。

默认 feature 下的 `cargo check --workspace --all-targets --locked` 因 `tests/bulk_recovery_cancel.rs` 使用 feature-gated `BulkProbe` 而失败；启用所有 features 后完整 all-targets check 通过。Workspace clippy 目前有既有 `-D warnings` 问题（包括 `jarde-reader` 的测试 lint 与 `jarde-java` 的历史 lint），没有把这些无关代码带入本 change。
