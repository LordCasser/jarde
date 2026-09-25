# 计数 `for` 的单闩锁投影

`OuterContinue.run` 的 BCI 3 初始化局部 2；BCI 4/6 测试它；BCI 36 `iinc 2,1` 是唯一外层回边更新。内层 BCI 21 直接跳到 BCI 36，正常走完内层则先经过 BCI 33 `iinc 1,100` 再到 BCI 36。因此只把 BCI 36 放进 `for` 更新位，BCI 33 仍在循环体。Jarde 文本在 `for` 前已有 `int local2;` 声明，头部使用 `local2 = 0` 是合法的 Java 赋值。改成尾部含更新的 `while` 加 `continue outer` 会跳过 BCI 36，不能接受。

Region 在认领头部之前核验唯一前置初值、条件读取同一 SSA phi、phi 的初值/更新输入与使用次数、条件中的其他局部在循环内不变、单一自然闩锁、闩锁所有入边都在循环内、更新块末尾回头，以及归纳局部在循环出口后不再读取。闩锁带其它效果时，仅允许从测试头直接进入，防止 `continue` 跳过效果。证明成功后，Region 将跨层 `continue` 的目标绑定到该闩锁；Builder 原子地将两条指令移入 `For` 头部，Emitter 保留 BCI 来源。证明不完整时仍使用已有 `while`/引用。

冻结反例 `ForBoundaries`：`observedAfter` 在退出后读取局部；`twoUpdates` 有两次更新；`varyingBound` 修改条件界限；`noUpdate` 不更新归纳局部。这四个方法均仍是 `while`，没有 `for` 或 `@bytecode`，`twoUpdates` 的两条更新都保留。正例 `counted` 为 `for`。本次只认领 `iinc`；设计 2d 的等价加后存储及更广的 3.2 验收尚未实现，任务不能勾选完成。

三方源码均用 `javac --release 8` 重编，并以 `java -Xverify:all` 运行冻结 Runner。冻结输入的编译选项须分开核对：`OuterContinue.class` 由 `javac --release 8 -g:none` 重编得到相同 SHA；`ForBoundaries.class` 保留 javac 默认的 `LineNumberTable`/`SourceFile` 属性，须用 `javac --release 8`（不带 `-g:none`）重编才逐字节相同，虽然两种调试选项的执行结果一致。root 从两份 fixture 源码独立重建 class，再对同一输入分别运行 JADX 1.5.6 与当前 Jarde CLI：两组的原源码、JADX、Jarde 全部重新编译成功，每组 5 行 `-Xverify:all` 输出完全相同。`OuterContinue` 的五行输出为：

```text
0:0
1:101
2:204
3:6
5:10
```

`ForBoundaries` 的五行输出为：

```text
-1:0:0:0:0:-1
0:0:0:0:0:0
1:0:1:0:1:0
2:1:2:0:1:0
5:10:5:6:3:0
```

输入 class SHA-256：`OuterContinue` 为 `b756d4fcebe65bed901ef368d3ae93ed71b16cc122d995f44926d9117d8551b5`，`ForBoundaries` 为 `9d580094884645a9acd495f604a54f7e8b42e00da837ae99ee184003e560fafe`。三方重编与输出暂存于 `/private/tmp/jarde-for-latch-replay/{outer,boundaries}/{original,jadx,jarde}`，完整 Jarde 文本分别在其中的 `jarde/OuterContinue.java` 和 `jarde/ForBoundaries.java`。JADX 给默认包加了 `package defpackage;`，重编时只移除了这一行。

`tests/p3_loop_transfers.rs` 检查 essential/all 正文一致，BCI 3 初值、BCI 21 `continue`、BCI 36 更新均可由 source map 定位；`tests/p3_for_latch.rs` 检查正反语法、低分析步数预算与预取消。`cargo test --test p3_for_latch --test p3_loop_transfers --locked` 通过 8 项；相邻 Grid、object-array 内层条件及 switch 归属拒绝也在此测试中。另跑 `p3_loop_test_values`、`p3_switch_fallthrough`，分别通过 4、2 项。`cargo fmt --all -- --check`、`git diff --check` 均通过。

root 使用独立 Cargo target 复核：上述四组集成测试合计 14/14，`jarde-java` 单元测试 121/121；相邻 `p3_guard` 13/13、`p3_sync_return` 3/3、`p3_twr_catch` 1 通过且 1 个既有忽略。当前只验收 `iinc` 单闩锁子集及它让 `OuterContinue` 的外层 `continue` 可执行；等价加后存储、带 switch/try 的转移正例和任务 3.2 的完整边界仍未闭合。
