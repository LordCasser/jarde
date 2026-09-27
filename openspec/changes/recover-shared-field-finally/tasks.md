## 1. 固定字段型基线

- [x] 1.1 在当前主线重新编译归档 `SharedFinally` 原源码，核 class SHA、异常表、三份清理 BCI；分别重编运行原/JADX 完整类并记录当前 Jarde 引用和 javac 失败，见 [CF-16 字段型基线记录](../../evidence/java-syntax-2026-09-27/cf16-finally/shared-field-baseline-root-2026-09-28.md)。

## 2. 共享证书支持字段更新

- [x] 2.1 将共享 finally 的清理锚点扩为三份准确 span，保持原单调用切片通过；用调用型定向测试和 `SharedFinally` 正例测试核三行、两个返回与 handler 所有权。
- [x] 2.2 为三份 `getstatic; int Push; iadd; putstatic` 证明同字段、同常量、同算术及各副本内唯一 SSA 生产/消费；以一处字段目标/增量/中途入口变异和扩围异常行的 verifier 有效负例逐一断言不折叠。
- [x] 2.3 在现有有界 Region 与 Builder checkpoint 中仅呈现首份完整清理 span 为一个 `finally`，核 `SharedFinally.handled` 完整文本没有 `@bytecode`，source map 覆盖全部物理 BCI 和三条异常行，两个返回及原异常重抛不变。

## 3. 三方运行与回归

- [x] 3.1 将原/JADX/实时 Jarde **完整 Java 8 类源码**分别 `javac --release 8 -g:none` 与 `java -Xverify:all`；确认 Jarde 与原 class 均为 `normal:1 / caught:1`，固定 JADX 为 `normal:2 / caught:1`，并将结果写入 fixture README。
- [x] 3.2 复跑 `SharedFinallyCall` 的三方运行及负例、已受证单出口 finally/typed catch、字段更新的预算和取消原子性；`cargo test -p jarde-java --tests --locked`、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`openspec validate recover-shared-field-finally --strict`、`git diff --check` 全通过并清理专用 Cargo target。
- [x] 3.3 root 独立重放三方完整源码、审阅所有权/SSA/来源证书与负例，并记录[验收](../../evidence/java-syntax-2026-09-27/cf16-finally/shared-field-root-acceptance-2026-09-28.md)；仅在通过后更新 CF-16 清单，保留 `FinallyOnce.escaping()` 等剩余差距。
