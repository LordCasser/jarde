# 1.2 永久 fixture 与 RED 定向门

## 固定输入

永久 fixture 位于 [`tests/fixtures/p3-do-while-body-transfers/`](../../../tests/fixtures/p3-do-while-body-transfers/)。`DoWhileCore.java` 与 runner 复用已冻结的 [`java-syntax-2026-09-22/do-while/core`](../../evidence/java-syntax-2026-09-22/do-while/core/) 输入；新增 `DoWhileSwitchBoundary.java` 专门验证嵌套 switch 的无标记 `break` 不得被误作外层循环出口。

本机 `javac 23.0.1 --release 8 -g:none` 重编后，`DoWhileCore.class` SHA-256 为 `9339878366306e65530aee489e15ca5ead92c4ff637b614141d9a8e552cb7a60`，与历史冻结 class 相同，且测试比较完整 class 字节。switch 控制 class SHA-256 为 `80b32c4b68f6409f236f4f7e049966dd30576602a82ac71224c56217b51b4fc8`。

`javap -c -p` 钉住两个转移：`withContinue(I)I` 的条件分支 BCI 7 跳过 `trace` 更新，BCI 10 `goto 24` 进入闩锁；`withBreak(I)I` 的 BCI 10 `goto 29` 跳过闩锁并到达循环后返回。runner 在 `java -Xverify:all` 下输出：

```text
basic:0=1:trace=1
basic:1=1:trace=1
basic:4=1234:trace=1234
continue:1=1:trace=1
continue:4=134:trace=134
break:1=1:trace=1
break:5=12:trace=12
switch:2=199:trace=0
switch:3=19939:trace=0
```

switch 负例的 `lookupswitch` 位于 BCI 8，case 2 的 switch-break bridge 从 BCI 28 到 BCI 38，仍在 do-while 内；条件回边从 BCI 48 回到 BCI 4，循环后出口为 BCI 51。当前报告把它保持为带真实来源的 fallback，正文没有 `do` 或 `break;`，不会错误地退出循环。

## 定向测试

在 `tests/p3_loop_boolean_do.rs` 中，`body_continue_and_break_edges_recover_with_original_trace_and_sources` 是 `recover-do-while-body-transfers` 2.1/2.2 的 ignored RED gate：它重编并比对永久输入、执行原 runner、检查 javap 边和两个方法的关键 BCI source map，然后要求未实现的完整结构化输出。忽略是有意的：本任务只建立 RED 测试，不提前让尚未实现的 2.1/2.2 使常规测试失败。`a_switch_break_inside_do_while_is_not_emitted_as_an_unmarked_loop_break` 是常规负边界测试。

使用独立临时 Cargo target 的结果：

- `CARGO_TARGET_DIR=/tmp/jarde-do-while-task-12-target cargo test --locked --test p3_loop_boolean_do`：5 passed，1 ignored。
- `CARGO_TARGET_DIR=/tmp/jarde-do-while-task-12-target cargo test --locked --test p3_loop_boolean_do a_switch_break_inside_do_while -- --nocapture`：1 passed。
- 显式运行 `body_continue_and_break_edges_recover_with_original_trace_and_sources -- --ignored --nocapture`：按预期 RED（exit 101）。fixture 重编、runner 输出及 `javap` 检查均先通过；当前来源缺口为 `withContinue` 的 BCI 7/21/32 和 `withBreak` 的 BCI 7/21/24/32。后续恢复任务应使这些有效指令均可追溯并通过其结构化输出断言。

Root 用独立的 `/tmp/jarde-root-do-while-accept-target` 与 `--all-features` 重放，常规套件仍为 5 passed、1 ignored；单独启用 ignored gate 仍按预期 exit 101，并精确报出上述 7 个缺失 BCI。验收后 `cargo clean` 清除了该 target 的 1.1 GiB 编译残留。

未改生产代码。既有 `p3_loop_boolean_do` 测试覆盖复合闩锁条件、另一条进入闩锁的路径及纯度负例；`p3-loop-transfers` 覆盖 while 标签/控制转移。它们没有涵盖该永久核心的 do-while 体内 continue、break 及 switch 负边界，因此本 fixture 不是重复冻结。
