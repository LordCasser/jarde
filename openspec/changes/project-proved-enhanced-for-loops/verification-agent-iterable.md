# Iterable 4.2 实施验收

2026-09-24 在现有 `Region::Loop` 已恢复的 `while` 后增加候选。准入只认精确的 `invokeinterface java/lang/Iterable.iterator()Ljava/util/Iterator;`、`java/util/Iterator.hasNext()Z` 与 `next()Ljava/lang/Object;`，并要求容器 AST 的真实源类型是 `Iterable`。数组与 Iterable 共用的 `ForEach` 字段由 `array` 改为 `iterable`，没有兼容层、新 IR、pass 或依赖。Java 方法头仍可呈现 raw `Iterable`；增强 `for` 使用新鲜 `Object` 元素名，原 `(String)` cast 留在体内原位置，不借 LVT/LVTT 猜泛型，也不更改 SSA 推断类型。

候选要求 `hasNext()` 是测试块唯一可观察调用，`next()` 前的体内物理指令仅是其接收者局部读取；`next()` 的结果只进入首句绑定或其原有 cast。iterator 槽从初始化到结束无其它写入，其 SSA 值只由测试与首句 `next()` 读取；header phi 只能有唯一初始化输入，其余回边必须全为自身透传。原始 `iterator()`、`hasNext()`、测试和 `next()` 的异常处理集合一致，移动过的 inline cast 亦如此。初始化与循环之间只许无异常的常量局部赋值／空声明，且不得改写容器或 iterator 局部。生成完整候选、来源和经预算计费的新鲜名称后，才在同一提交分支移除旧 iterator 声明／赋值并发布 `ForEach`；拒绝时原 `while` 不变。

定向测试覆盖以下结果：

| 输入 | Jarde 4.2 输出和 Java 8 行为 |
| --- | --- |
| `RawIterableIterator` `-g`/`-g:none` 的 `rawCast` | 均为 `for (Object … : values)`，体内保留 `Object item` 与原 `(String) item`；各完整类原/Jarde `-Xverify:all` 六行逐字一致 |
| `IterableForEach` `-g`/`-g:none` | 均为 `Object` 元素绑定加原位 `String` cast；各完整类原/Jarde四行逐字一致，包括空序列、null 元素和 null 容器 |
| `IterableBoundary.capturedOnce` 与 `skipEmpty` | 捕获供给者一次求值，`continue` 的多回边仍保留语义；原/Jarde 完整类 Java 8 重编并运行七行一致，包括空/null 与 iterator/hasNext/next 抛错 |
| 负例 | `List`、`Collection` owner、同轮两次 `next()`、循环后 iterator 消费、`touchBeforeNext` 及原 next 值额外用途均维持 `while`；自定义非 Iterable `iterator()` 不误投影，受保护 `next()` 维持原保守输出 |

验证命令及结果：

| 命令 | 结果 |
| --- | --- |
| `CARGO_TARGET_DIR=/tmp/jarde-iterable-array-target cargo test -p jarde-java --lib` | 140/140 |
| `CARGO_TARGET_DIR=/tmp/jarde-iterable-array-target cargo test --no-default-features --test p3_iterable_foreach --test p3_array_foreach --test p3_wrapped_array_foreach --test p3_loop_transfers --test p3_for_add_store` | 20/20；含新集成 4/4 与数组／循环相邻 16/16 |
| `cargo fmt --all -- --check` | 通过 |
| `openspec validate project-proved-enhanced-for-loops --strict` | 通过 |

来源测试对 `rawCast` 的 iterator 初始化、测试、next、元素存储与 cast 的真实 BCI `1/6/9/10/15/18/19/24/25/26/29` 查询到正文，essential/all 正文一致。一个低 `analysis_steps` 限额与预先取消的测试证明无已发布半折叠；该测试不声称穷举所有中间限额。完整类均以 `javac --release 8 -g:none -Xlint:-options` 重编并以 `java -Xverify:all` 对照原 class。当前实现 CLI SHA-256 为 `e4c18b9fbb4dec07627c031cbabcc8496fdac0423ce26d06c1fc589efdd1757a`，来自独立 Cargo target，root 仍须执行任务 4.3 的独立重建与复核。

全量 `CARGO_TARGET_DIR=/tmp/jarde-iterable-array-target cargo test -p jarde-java` 仍被无关的并发 facade 签名变更阻断：`recover_for_class_source` 已需第三个 `bool` 参数，而 `class_initializer_candidates.rs` 七处、`p3_patterns.rs` 一处仍按两个参数调用。本切片没有编辑这些测试。

独立 target 清理前 `du -sh` 为 2.9 GiB；`cargo clean --target-dir /tmp/jarde-iterable-array-target` 移除 14,031 个文件并删除目录。`df -h /tmp` 的可用空间从 11 GiB 增至 14 GiB。
