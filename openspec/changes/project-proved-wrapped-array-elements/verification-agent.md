# Wrapped-array 实施验收（任务 2.1–2.3）

2026-09-24 在既有 `array_for_each_candidate` 中加入受限的表达式内元素绑定，没有增加 AST 形态、全局 pass 或依赖。候选仍复用已验收的 `ForHeader`、数组与索引 SSA 身份、长度独占消费及异常 handler 比较。首条语句的元素读取只允许沿整数加法抵达；它前面的操作只能是纯整型局部读取或整数常量。调用、字段／数组读取、写入和控制流一律不能作为前缀。循环所有块只准出现一次数组元素读取，原 `Index` 的栈结果必须只有一个真实消费。根 `Index` 局部赋值保持原形，避免把循环后仍可读的元素局部搬入增强 `for` 头。

在完整证明后，候选先克隆首条语句，仅将该 `Index` 子树替换为预算计费的新鲜元素名，随后才移除长度缓存及计数循环头。原数组捕获、元素读取之后的 `tick()` 和其它语句保留原顺序。折叠的长度、索引、更新和元素读取 BCI 进入增强 `for` 来源；保留的首句继续持有自己的来源。失败时不修改原计数 `for`。

## 结果

| 验收 | 结果 |
| --- | --- |
| `CARGO_TARGET_DIR=/tmp/jarde-wrapped-array-target cargo test -p jarde-java --lib` | 140/140 |
| `CARGO_TARGET_DIR=/tmp/jarde-wrapped-array-target cargo test --no-default-features --test p3_wrapped_array_foreach --test p3_array_foreach --test p3_loop_transfers --test p3_for_add_store --test p3_array_access` | 27/27 |
| `cargo fmt --all -- --check` | 通过 |
| `openspec validate project-proved-wrapped-array-elements --strict` | 通过 |

新定向测试断言 `plusArrayFirst` 与 `wrappedOnlyRead` 成为增强 `for`，`plusTickFirst`、两种调用实参、前置写数组与同轮两读仍为计数 `for`。旧数组回归中的 `effectInBinding` 正是先读数组再调用的安全形状，现改为增强 `for` 正例；索引、数组错配及元素局部逃逸等拒绝仍通过。完整原 class 与新 Jarde 类分别用 `javac --release 8 -g:none` 编译 runner、用 `java -Xverify:all` 运行，13 行的结果、调用次数和数组最终状态一致。测试只归一 helpful-NPE 中与局部变量编号相关的消息文本，保留异常类型与调用轨迹比较。来源测试核对正例数组读取 `19`/`20`，以及对应的长度、索引、更新、后续调用和首句存储 BCI；essential/all 的完整正文一致。低预算测试覆盖一个分析步限额点，并检查任何已发布增强 `for` 不带旧长度、索引读取或重复 `tick()`；取消测试检查无正文发布，不声称穷举全部中间预算点。

`CARGO_TARGET_DIR=/tmp/jarde-wrapped-array-target cargo test -p jarde-java` 全量集成测试仍被并发 `recover_for_class_source` 增加第三个 `bool` 参数阻断：`class_initializer_candidates.rs` 七处、`p3_patterns.rs` 一处仍以双参数调用。本切片未改这些文件。root 的任务 3.1–3.2 独立验收尚未执行。

独立 Cargo target 在清理前 `du -sh` 为 2.7 GiB。`cargo clean --target-dir /tmp/jarde-wrapped-array-target` 移除 12,588 个文件，目录消失；`df -h /tmp` 的可用空间由 14 GiB 增至 16 GiB。
