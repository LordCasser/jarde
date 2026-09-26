# Root 条件值证明与边界验收

2026-09-26 在隔离 checkout `bfad1b9c` 审读 `crates/jarde-java/src/build.rs` 的 `prove_conditional_value_with_forward`、`build_conditional_value`，以及 `ast.rs` 的条件值类型门和 `emit.rs` 的 `?:` 优先级。证明只接受两条各自为直线区域的正常臂、精确 join、唯一变化的 stack Phi、与前驱出口栈槽一一对应的两个输入、各臂生产者及 join 中的唯一消费；其它 stack Phi 须为同值传递。构造表达式还核对两臂全部指令归属，不把额外副作用藏进 `?:`。类型只在两臂相同或 null/引用组合时形成表达式；发射器按 Java 优先级括号化。来源包含分支、两臂生产者、传递和消费者，不从一般 Phi 猜值。

隔离 `CARGO_TARGET_DIR` 下运行四个 `jarde-java` 条件值/字段/跨条件实参/布尔返回测试文件，共 10 项通过；根 crate 的 deferred、switch value、switch loop exits 共 7 项通过，另运行 deferred 的 JDK 执行对照 1 项通过。`p3_execution_comparison` 默认 3 项为需要 JDK 的 ignored 行，本次不把它们算作通过；整类执行已由 [3.1](verification-3.1.md) 的独立重放覆盖。

以 `--policy single-class --release 8` 对冻结的 `ConditionalBoundaryCases.class`（SHA-256 `04d7291d6da56dc802a3717dc5e2b78ccbab552b15f4e50f7b4208b5dbb96eb5`）与 `ConditionalBoundarySwitch.class` 重跑当前 CLI。switch 仍是三分支 switch 后读取局部变量，没有被折成两臂 `?:`。前者的循环体内合法 `again ? n : -n` 现在能恢复，而循环回边仍保持 `while`，布尔赋值缺类型证据处留 `@bytecode 29`；`guarded` 的跨 handler 局部值整方法拒绝，缺依赖层级的 `unknownType` 仍拒绝。循环输出与 2026-09-24 冻结的 `evidence/boundaries/jarde.java.txt` 有进展差异，不能将旧文本当本次精确输出；这不改变循环/异常/类型不明不得冒充简单双臂 join 的边界。`guarded` 的词法局部作用域债务属于另项工作。

本项没有生产代码变更；更一般的 Phi、异常区域与局部作用域恢复不随条件值准入放宽。
