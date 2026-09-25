# 纯局部变量嵌套循环的转移目标

这是 `present-proved-java-structure` 3.1 的隔离输入，从早期第 70 批 `/tmp` 探针固定为本目录的 `Grid.java`、`Grid.class`、`GridRunner.java`。`javac --release 8 -g:none` 重编 `Grid.java` 与冻结的 Java 8 class 逐字节相同，class SHA-256 为 `c81221e1a70ee1336d78053b43e843c9afe213f4180b06430b1ff8c8f6e83d46`。三个方法只用整数局部：内层 `break`、`continue outer`、`break outer`；不依赖数组、泛型或异常处理。

root 在独立临时目录分别用原 class、JADX 1.5.6 反编译源码与当前 Jarde CLI `class-source --policy single-class --release 8 --format text` 编译并运行 `GridRunner`，Java 源兼容级别 8，运行用 `java -Xverify:all`。本地 JADX 源码参考版本为 `2fb1b1638694`；本次 Jarde CLI SHA-256 为 `4d8bd10e0d6ff09ab765e05242e9d0922687e372b18ea1cc6f56cddba03fd00a`。

| n | 原 class | JADX 重编 | 当前 Jarde |
| ---: | --- | --- | --- |
| 0 | `0:0:0:0` | 相同 | 整类 javac 失败 |
| 1 | `1:1:1:11` | 相同 | 整类 javac 失败 |
| 3 | `3:9:6:39` | 相同 | 整类 javac 失败 |
| 6 | `6:21:12:3` | 相同 | 整类 javac 失败 |
| 9 | `9:21:18:3` | 相同 | 整类 javac 失败 |

Jarde 输出有 3 处 `@bytecode`，`javac` 报 3 个“缺少返回语句”；JADX 与原 class 各 5 行完全相同。这证明即使没有数组读取干扰，内层循环的汇合仍被当前循环体走访截断，属于 3.1 的后继归属与转移目标问题。JADX `LoopRegionMaker` 先把 loop condition exit 和其余 exit edges 分开，再沿真实出口边插入 `break`/`continue`，这一分类可借鉴；Jarde 仍须用 Region 嵌套与实际边目标证明是否需要外层标签，不能照搬 JADX 对含 `switch` 路径直接拒绝插入 break 的启发式。

补充字节码边界：`Grid.labeledContinue` 的 BCI 21 `goto 34` 到达的 BCI 34 同时是内层循环自然出口和外层 `iinc` 的入口；内层循环后没有其它语句。因此这份 class 无法区分源码的 `continue outer` 与等价的内层 `break`，不能拿它验收“必须输出外层 continue 标签”。`nestedBreak` 的 BCI 23 `goto 36` 是内层出口，`labeledBreak` 的 BCI 21 `goto 45` 是独立外层出口。要验收外层 `continue` 标签，须另造内层循环后有可观察语句、且 `continue outer` 跳过该语句并指向独立外层闩锁的冻结 class；按真实目标而不是源码标签作判据。

## 当前工作树的 3.1 子集复核

root 用 SHA-256 `4218aed9ca51f06e2f362c6aab67cba4cfb7524f345e2ac3857ea7383fbb0f9f` 的 Jarde CLI 独立重放冻结 class。安装的 JADX 1.5.6 对默认包输入添加 `package defpackage;`；比较前仅去掉这个包装行，使三份编译产物与测试 runner 处于同一默认包。原 class、JADX 源码和 Jarde 源码各自以 `javac --release 8 -g:none` 重编并用 `java -Xverify:all` 运行。

| 输入 | class SHA-256 | 三方执行 | 当前 Jarde 呈现 |
| --- | --- | --- | --- |
| `Grid` | `c81221e1a70ee1336d78053b43e843c9afe213f4180b06430b1ff8c8f6e83d46` | 5/5 行相同 | 三个方法无 `@bytecode`；内层 `break`、带外层标签的 `break` 各在条件臂内；源码中的 `continue outer` 因目标重合，等价写为内层 `break` |
| `ObjectArrayForeach` 的循环体内条件 | `e0b6e524da02db14fae69bb6924c7dd30c23d8b42c81b2e68c996eeee444c4db` | 7/7 行相同 | `if`、循环后继和返回值完整呈现，无 `@bytecode`；仍用下标 `while`，不宣称已恢复增强 `for` |

独立负例 `tests/fixtures/p3-loop-transfers/OuterContinue.java` 由 `javac --release 8 -g:none` 重编后与冻结 class **逐字节相同**，SHA-256 `b756d4fcebe65bed901ef368d3ae93ed71b16cc122d995f44926d9117d8551b5`。它在内层循环后有 `total += 100`，`continue outer` 直接跳到外层 `iinc` 的 BCI 36，不能等价写成内层 `break`，也不能把更新留在 `while` 体尾后写外层 `continue`。`OuterContinueRunner` 对 `n=0,1,2,3,5` 的原 class/JADX 输出均为 `0:0`、`1:101`、`2:204`、`3:6`、`5:10`；Jarde 仍引用字节码，未输出错误的 `break`/`continue`。另一冻结反例 `SwitchLoopTransfer`（SHA-256 `f89d59cd8c56c295c20a8f0c25f9b1799216dbaa8e05e1c3ea161df312888595`）中 switch 与循环出口归属尚不能证明，也继续引用。

定向 Rust 回归 16 项通过：`p3_loop_transfers` 4、`p3_loop_test_values` 4、`p3_switch_fallthrough` 2、`p3_one_armed_if` 3、`p3_switch_value` 2、`p3_exception_scope` 1。3.1 保持未勾选：带 switch/try 的正例、真正外层 `continue` 和其它出口反例仍未闭合；外层更新链目标与 3.2 的 `for` 更新表达相依赖。

邻接检查中 `jarde-java --lib` 121 项通过，`p3_char_switch` 1、`p3_forward_join` 3、`p3_guard` 13 及 `p3_twr_catch` 的非忽略项 1 通过。全量门禁尚未绿：`p3_stated_rows` 的一项因 catch 参数槽位逃逸被拒绝，`p3_sync_return` 的精确文本断言期望 `return this.n`，当前实际等价地写成 `int saved0 = this.n; return saved0`。两项与本轮循环证明的因果关系需另行隔离，不能把定向通过写成全套通过。`enumswitch.rs` 的未使用 `values_factory` 警告也属于已记录的枚举证明缺口，不混进 3.1。

## 后续工作树状态（2026-09-24）

上面的旧 CLI SHA、`OuterContinue` 仍引用、`p3_sync_return` 红项和枚举警告都是当时的快照，不再代表当前工作树。[计数 `for` 的独立复核](../for-latch/analysis.md)已确认 `OuterContinue` 的 `iinc` 闩锁被移入 `for` 头，带标签的外层 `continue` 正确执行，原源码/JADX/Jarde 的 5 行结果一致；同步返回与枚举数组证明也已另行修复和复核。[七行 switch/loop 反例及验收](../switch-loop-exits/analysis.md)进一步定位了 switch 局部汇合与外层循环出口混同；Jarde 现已恢复该类与旧 `SwitchLoopTransfer`，JADX 1.5.6 对新类仍输出不可编译的重复 `break`。3.1 仍未完成。
