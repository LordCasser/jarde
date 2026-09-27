# CF-10 数组长度循环条件：主线独立验收

root 在 `0a506b75`（已包含先前的 List→Iterable 调用修复）独立构建 CLI，SHA-256 为 `c838c3d2651de54a53c5c2408004876fb0ed420e6d896b9c092d143d4f580f2c`。分别重新以 `javac --release 8 -g` 编译 [`StepIndex.java`](step-index/original/StepIndex.java) 和 [`ForeachCases.java`](../input/ForeachCases.java)，所得 class SHA-256 与冻结输入一致：`64b5baa6270ce4604c0a70fa0a6d3438902bdeaa9f3bc3f7cb5105b40c512344`、`6b65ab7efe39ce153eb008f626ed2f0cd382b2260c3c10a241fef8031b0c6864`。固定 JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f` 重新反编译两类；原、JADX、Jarde **完整类源码**分别按 Java 8 重编并以 `java -Xverify:all` 运行。

`StepIndex` 三方均输出 `4`，Jarde 源码 SHA-256 `61d6d63bafdb9ccfb1b8ad750df7d46df5df7af719df0be23b73aa3b98de63a3`，恢复保留索引步长 2、数组长度条件和返回，无 `@bytecode`。组合 `ForeachCases` 三方均输出 `10 / abc / 4`，当前 Jarde 完整源码 SHA-256 `1a05995a82a0c979f1a36075c3c279db52466a465b6fef1d9b4fab66089693b5`，无 `@bytecode`；这同时验证旧工作树上因独立 `List→Iterable` 差距丢失的 `abc` 已在主线恢复。另以各自完整源码编译 `everyOther(null)` 探针，三方均为 `NullPointerException:everyOther:2`（异常类型、首个方法帧、栈深一致）。

Region 仅允许条件同块 SSA 消费链里的 `ArrayLength`，其输出在测试块只有一个读取位置；latch-tested loop 不扩门。额外 `iinc`、未消费长度和复制复用负例仍引用。审阅时专门补测把长度值复制后留到两个后继的形状：`dup` 成为条件链生产者，`StatementFree` 门在 BCI 6 拒绝；相同字节码按旧 class 版本由 JVM 验证后在空数组读处按预期抛 NPE。数组长度不会被提升到循环外或重复求值。

root 检查通过：`cargo test -p jarde-java --locked`（全 crate）、`cargo test --test p3_arraylength_loop_condition --locked`（2 项）、补充 `p3_java_recovery`（37 项）、`cargo fmt --all -- --check`、`openspec validate recover-arraylength-loop-condition --strict`。CF-10 的这两个隔离缺口和组合重放已闭合；JADX 的禁编译八形态负例只断言无冒号，不能据此宣称 CF-10 全单元追平。
