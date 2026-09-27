# CF-03 共享尾修后验证

固定 [summary.json](summary.json) 与生成的 [Jarde 完整类源码](source/jarde/cf03/BranchShapes.java)记录最终 CLI 二进制 SHA-256 `5d14ece13f98dfe78a899e65e4e3c9141adc834bcd386fd69322b57f384f803d`。原 class、固定 JADX、Jarde 的完整 `BranchShapes` Java 8 源码均通过 `javac --release 8 -g:none` 和 `java -Xverify:all`；15 行输出逐字相同。`nested` 的 BCI 24 仅在外层续接，`hits` 赋值与 `return true` 各一次。独立 `ChainOnly` 三方均通过五行对照，Jarde/JADX 各有三个 `else if`。

`p3_shared_tail` 检查 BCI 0、4、8、12、14、18、22、24 的单一 Region 所有权，测试块和尾部效果的 source map、`hits` 赋值一次、`guards` 非回归、取消及 `AnalysisSteps` 恰少一步时的原子停止。候选扫描先按每条 canonical edge 建邻接表并计费，再为每侧走访的块与边计费；共同入口选择对块、边、共同候选计费；共享早退叶的入边逐条计费，最终领取两个新块按 `IrItems` 计费。预算不足时文本和 source map 均为空。

[负例夹具说明](../../../../../tests/fixtures/p3-shared-tail/README.md)记录两份 `java -Xverify:all` 可运行的修改后 class。额外入口从外层 branch 之外进入尾部，不满足候选前驱的支配证明；BCI 23/30 两个互不可达候选不具备唯一最近共同入口。两者均保留 `@bytecode` 引用及物理来源；不会为候选虚构一个共享结构。异常、`jsr`/`ret`、回边或循环头仍不能通过此证明：扫描逐边要求正常且严格前向，候选只从无 loop/try/switch/finally Frame 进入；P3 原有异常/循环拒绝测试在完整 `jarde-java` 测试中保持通过。此处的 `Frame.shared_tail` 只保存已证外层尾的停止界，不改变循环体 owner，也不领取循环中的提前 `ireturn` 叶；CF-07 的循环 BCI 19 不在此形状内。一般异常、循环、switch 内共享尾及任意多入口结构仍在本变更范围之外。

验证命令：`cargo test -q -p jarde-java`、`cargo fmt --all -- --check`、`cargo check --workspace`、`openspec validate own-proved-shared-early-return-tail --strict`，均通过；Cargo 使用独立 `CARGO_TARGET_DIR=/tmp/jarde-cf03-target`。
