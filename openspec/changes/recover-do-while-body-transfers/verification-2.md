# 2.1–2.3 本层 do-while 转移验证

`region.rs` 先按真实后继判断头块比较是否是体内分支：两臂留在天然环内，或其中一臂经独占普通转移桥到唯一闩锁出口时，优先尝试单闩锁 `do-while`；经典头测循环仍由原头测路径处理。`withContinue` 的 BCI 10 是到闩锁的纯 `goto`，体内分支写成等价的单臂效果；BCI 24 的条件只出现一次。

`withBreak` 的 BCI 10 桥由 canonical 原边确认唯一 Normal 入边 `2→10`、唯一 Normal 出边 `10→29`，桥块仅含 `Transfer`，无外部或异常入边，也无未认领效果。BCI 29 是闩锁分支的唯一出口；BCI 10 归属循环臂并锚到生成的 `break`。闩锁块有体内效果时先写该块的体前缀，再写条件，避免提前退出时运行体尾或条件。循环覆盖核账包含天然环外的桥块。嵌套 `switch` 和额外测试入口仍保守拒绝。

永久 `p3_loop_boolean_do` 测试已从 ignored RED 转成常规门：原 class 重编字节一致，原/恢复核心 runner 的九行返回值与 trace 一致；默认、essential+SourceMap 与 all 请求正文相同，`withContinue`/`withBreak` 的 BCI 7、10、13、21、24、29、32 在后两种带来源证据的请求中均有锚，桥 BCI 10 分别映到 `if` 与 `break`。恢复类经 `javac --release 8` 编译并在 `java -Xverify:all` 下执行。新增带副作用的闩锁调用对照：提前退出时输出 `12:2`，原/恢复 class 一致，说明第 3 轮 `break` 跳过闩锁调用。低分析步数与预取消请求保持停止状态，不发布未经证明的循环。

隔离 target 下运行 `cargo test --locked --all-features --test p3_loop_boolean_do`，8 项通过；相邻 `p3_loop_boolean_exit` 3 项、`p3_loop_test_values` 4 项、`p3_loop_transfers` 5 项、`p3_loop_try_handler_entry` 6 项、`p3_switch_loop_exits` 3 项通过；`jarde-java` 的 `region::tests` 3 项通过。`openspec validate recover-do-while-body-transfers --strict` 通过。此记录只覆盖 2.1–2.3；完整混合类和健康类的整类重放、跨层转移以及 root 的 3.x 验收仍由后续任务处理。
