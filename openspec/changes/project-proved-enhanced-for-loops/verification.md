# 数组子切片实现验收（2.1–2.4）

2026-09-24 在当前工作树完成数组投影。独立构建目录为 `/tmp/jarde-enhanced-for-array-target`；CLI 为其 `debug/jarde-cli`，SHA-256 `4f07d05e7029802f9c468044ef63a81faf26d9d3f89fde8f7a2a6411121cf715`。root 开始独立重建与验收后，已清理此独立构建目录。

## 实现与拒绝边界

- `ForEach` 仅在既有 `Region::Loop`、`ForHeader` 已形成计数 `for` 后尝试。证明零初值、单步 `+1`、`index < cached length`、长度与元素读取的同一 SSA 数组值、元素类型及局部绑定、长度/索引的独占真实消费和同值 join phi 透传。空值可能抛出的 `arraylength`、元素读取和循环入口必须有相同的异常 handler ordinal 列表。
- 原数组捕获语句留在原位，保留 `Supplier.get()` 一次求值；长度缓存、索引初值/更新及元素读取只在完整证明后移入增强 `for`。折叠的真实 BCI 合入语句来源。不能证明时不修改计数循环。
- 同形负例均先缓存数组和长度，进入计数 `ForHeader` 接缝：体内额外索引用途、不同数组、元素绑定含额外调用、元素局部逃逸均保留计数 `for`；索引退出后用途保留保守形态。`break` 的相邻转移例保持普通 `while`，已可证明的 `continue` 例投影增强 `for`。

## 定向结果

| 命令 | 结果 |
| --- | --- |
| `CARGO_TARGET_DIR=/tmp/jarde-enhanced-for-array-target cargo test -p jarde-java --lib` | 140/140；含独立增强 `for` 打印与来源单测 |
| `CARGO_TARGET_DIR=/tmp/jarde-enhanced-for-array-target cargo test --test p3_array_foreach --no-default-features` | 4/4；正反例、来源、低预算/取消、完整类执行 |
| `CARGO_TARGET_DIR=/tmp/jarde-enhanced-for-array-target cargo test --test p3_loop_transfers --no-default-features` | 5/5 |
| `CARGO_TARGET_DIR=/tmp/jarde-enhanced-for-array-target cargo test --no-default-features --test p3_for_add_store --test p3_loop_test_values --test p3_loop_try_handler_entry --test p3_array_access` | 24/24 |
| `cargo fmt --all -- --check` | 通过 |
| `openspec validate project-proved-enhanced-for-loops --strict` | 通过 |

完整类执行测试对冻结 `IntArrayForeach`、`ObjectArrayForeach`、`ArrayForeachRefusal`、`ArrayForeachTransfers` 分别复制原 class、用 `javac --release 8` 重编 Jarde 全类、运行相同 runner 的 `java -Xverify:all`；原/Jarde 输出分别为 6、7、5、4 行且逐行一致，唯 helpful-NPE 的编译器临时变量文字先归一到异常类型。运行覆盖空/null/多元素、Supplier 一次调用与抛错、元素 `hashCode` 调用与抛错、错配数组越界和 `continue`/`break`。来源测试分别核对四个正例的数组捕获、长度、索引、元素读取/存储与更新 BCI；essential/all Java 正文一致。低分析预算和取消均没有发布半折叠方法；该预算测试覆盖一个低限点，未声称穷举所有中间限额。

`cargo test -p jarde-java` 全集在本轮被并发的 `recover_for_class_source` 新增布尔参数挡住：`class_initializer_candidates.rs` 七处、`p3_patterns.rs` 一处仍按双参数调用。此错误与数组子切片无关；本轮没有编辑这些测试。root 的 3.1/3.2 独立验收仍待进行。
