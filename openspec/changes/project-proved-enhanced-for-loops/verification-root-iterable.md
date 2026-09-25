# `Iterable` 子切片的 root 独立验收

## 结论与准入范围

已验收的投影只接受 Java 8 中可证明的直接 `java/lang/Iterable.iterator()Ljava/util/Iterator;`、`java/util/Iterator.hasNext()Z`、`next()Ljava/lang/Object;` 组合。既有 SSA 值证明 iterator 没有逃逸或额外消费者，元素 `next()` 是每轮首个可观察动作；原 cast 仍在循环体原位，初始化、测试、取值及 cast 的异常处理器集合一致。`ForEach` 复用数组切片的 AST 节点，字段统一名为 `iterable`。不满足证明时保留已构造的普通 `while`，不借调试局部变量表、短方法名或推断泛型改写来凑语法。

root 审读了 `crates/jarde-java/src/build.rs` 的候选与提交分支：调用目标按 owner、name、descriptor 和 invoke kind 精确匹配，iterator 初始化结果只有 store 消费，header phi 是入口值加自身回边，真实消费者只在 test/next 路径；新元素名由既有名称表分配。候选先验证接收者源类型、cast/SSA、处理器与预算，再移除 iterator 的声明和赋值、提交新循环。来源包括初始化、`hasNext`、`next`、绑定及 loop test 的 BCI。此切片只接受局部 `Iterable` 接收者；`List`、`Collection` 和自定义子接口的 owner 留给独立类型证据工作。

## 冻结完整类对照

root 在独立 `/tmp/jarde-iterable-root-target` 执行 `cargo build -p jarde-cli --locked`，CLI SHA-256 为 `f1bfcff286e0dc0cf08071de7d5375cbfebbb396acccbf6809fe506e16fac0b9`。对 [4.1 冻结输入](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/remaining-4-1/analysis.md)的 raw `Iterable` 有/无调试信息 class，以及[泛型原始 class 与调试对照](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/jadx-iterable-for-claim/analysis.md)的有/无调试信息 class，分别生成完整 Jarde 类；每组把冻结 runner 与原 class 或生成的 Java 源分开以 `javac --release 8 -g:none -Xlint:-options` 构建，并以 `java -Xverify:all` 执行。

| 输入 | 原/Jarde 行数 | Jarde 增强 `for` | 完整类执行 |
|---|---:|---:|---|
| raw，`-g` | 6/6 | 1 | 逐行相同 |
| raw，`-g:none` | 6/6 | 1 | 逐行相同 |
| 泛型，`-g` | 4/4 | 1 | 逐行相同 |
| 泛型，`-g:none` | 4/4 | 1 | 逐行相同 |

四份生成源码 SHA-256 依次为 `5c521e14138254c07e7f6db14acd0d701f900bfcc61894eef2fa5c83e519efed`、`e9237dcc789a16493668212c2064f569d9faad95101fb71114a199503d65586a`、`a9e31f512e958b654c5c2ea4df560b3bce6a2b4199aeeb7fb1064a739c8a176f`、`406b2bfaa1c02fed1c0921ce94aa133698bb1123feda910ac8c4a6a53c7cc7f9`。例如 raw 无调试信息源码写 `for (Object iteratorElement19 : arg0)`，然后保留 `Object local3 = iteratorElement19; String local4 = (String) local3;`；坏元素的 `ClassCastException` 仍发生在原位置。此前[三方负例复核](verification-root-iterable-boundaries.md)已证实 JADX 1.5.6 在无调试信息的另一条 raw while 路径把副作用移到 cast 前，以及在有调试信息时把受保护的 `next()` 移出 `try`；这两种 JADX 输出不能作为投影真值。

root 还从 [边界 fixture](../../../tests/fixtures/p3-iterable-foreach/IterableBoundary.java)独立以 Java 8 编出 class（SHA-256 `402c035dfb905cb52390a8103dea7fd953bd8fc6790ffe374a27528ec9e5f1db`），Jarde 完整类 SHA-256 `176872efa9fc52fc2ae1cf08bd765b441fc375ee039611df2447e8ac1c43f290`。两者与同一 runner 重编/运行均为 7 行逐字相同，覆盖容器供应者只调用一次、空/null、`continue` 以及 `iterator`/`hasNext`/`next` 抛错。`capturedOnce` 与 `skipEmpty` 写增强 `for`；`List`、`Collection` owner、同轮两个 `next` 和循环后 iterator 再使用均保留 `while`。4.1 的非 `Iterable` 同名方法、首动作前副作用、受保护 `next` 亦未被误投影；其中受保护区的 Jarde 普通结构仍 explanation-only，不能宣称其运行等价，独立 P0 结构债务见路线图。

[额外异常边界探针](../../evidence/java-syntax-2026-09-24/iterable-exception-boundaries/analysis.md)把 `iterator()`、`hasNext()`、`next()` 放入不同的 catch/finally 集合。root 独立将冻结的原 class 和 JADX Java 源各自以 Java 8 编译/`-Xverify:all` 执行，12 行里 JADX `-g` 有 4 行、`-g:none` 有 3 行不同：错误 catch、跳过 finally 或重复执行 finally。Jarde 对三条目标方法均 explanation-only，缺返回语句，故这组证据验证的是候选必须拒绝跨处理器移动的边界，并暴露独立结构债务，不是 Jarde 运行等价的声称。

## 测试与限制

root 独立运行新增 `p3_iterable_foreach`：4/4；相邻 `p3_array_access`、`p3_array_foreach`、`p3_for_add_store`、`p3_loop_transfers`、`p3_wrapped_array_foreach`：27/27；`cargo test -p jarde-java --lib --locked`：140/140。`cargo fmt --all -- --check`、`openspec validate project-proved-enhanced-for-loops --strict` 与 `git diff --check` 均通过。新增测试还检查真实来源 BCI、essential/all 正文一致、低预算与取消时不发布半折叠结构。实施代理的全量 `jarde-java` 集成测试仍被并发 facade API 变更留下的八处旧调用编译错误阻断；本次定向和库测试没有把该债务混入循环改动。

泛型 `Iterable<String>` 的方法头在当前通用 Signature 投影中仍会退回 raw `Iterable`，因此这里合法地用 `Object` 增强 `for` 加原 cast；恢复 `String` 元素声明须有独立泛型类型证明，不能删除 cast 假装原始类型已恢复。数组正例的多余顶部局部声明也继续作为独立文本质量债务。root 已以 `cargo clean --target-dir /tmp/jarde-iterable-root-target` 删除 7459 文件/2.7 GiB；实施代理和异常探针的独立 target 分别删除约 4.2/1.2 GiB，最终可用空间约 19 GiB。冻结证据与约 656 KiB 的复放目录保留用于审计。
