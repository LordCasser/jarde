# CF-02：比较谓词与提前返回后的可达路径

固定队列项为 [CF-02](../../jadx-feature-inventory-2026-09-27/control-flow.md)。[replay.py](replay.py) 校验 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestCmpOp`、`TestCmpOp2`、`TestConditions7`、`TestTernary3` 和五个条件实现入口的哈希。[input/cf02](input/cf02) 用合法 Java 8 顶级 `Predicates` 与共同 Runner 缩小测试：浮点比较含 NaN、无穷、负零；数组边界含负数、合法和越界索引；`named(Object)` 是固定 `TestTernary3` 的判空、`instanceof`、cast、提前 `return false` 与后续布尔返回形状。本片未覆盖该测试的 `InsnArg` 外部类层级、`getName` 多分支和其它 `TestCmpOp` 的全部文字断言。

[baseline/summary.json](baseline/summary.json) 记录原 class、JADX 和 Jarde 的输入/生成源码哈希。三份**完整 `Predicates` 类源码**各自与相同 Runner 用 `javac --release 8 -g:none` 重编，均以 `java -Xverify:all` 运行。前九行逐字一致：浮点 NaN 比较为 false、正无穷大比较为 true、`-0.0f < 0.0d` 为 false、边界检查为 false/true/false，判空、非 String 和空串为 false。最后 `named("x")` 原 class 与 JADX 为 `true`，Jarde 为 `false`。这是一项已证的行为偏差，即使 Jarde 源码本身能通过编译。Jarde 对该方法的报告是 `quality=fallback`、`semantic_validation=unproven`，源码也写出 `// @bytecode 28` 和未恢复标记；不能将“能编译”误计为完成恢复。

[Jarde 完整源码](baseline/source/jarde/cf02/Predicates.java)把 `named` 输出成两个空 `if` 后的无条件 `return false`，并在其后注释“live block [28]”；[JADX 完整源码](baseline/source/jadx/cf02/Predicates.java)保留 `obj != null && obj instanceof String && ((String)obj).length() > 0`。`javac` 的 BCI 13 进入 cast/`String.length()`，BCI 20 比较，23/27 产生布尔值，28 返回；这个正常路径绝非可忽略的异常 handler。

根因落在现有 Region 候选/遍历边界。[region-probe.log](baseline/region-probe.log)显示方法的 canonical blocks 为 `[0,4,11,13,23,27,28]`，区域 walk 仅访问 `[0,4,11,13,23,27]`；BCI 13 的条件以 BCI 28 为后继合流，而外层一个 `if` 以提前返回块 BCI 11 为 join。`region.rs::region_at` 的 one-armed `if` 路径调用内层 `region_at` 后丢弃 `arm_next`，于是把可达的 BCI 28 留给尾部 `UncoveredBlocks`，同时已输出了可编译的无条件假返回。`short_circuit_value` 当前仅接收方法级或受证 try 头，不能直接把这个嵌套返回值接成已证明的布尔表达式。比较运算本身在同一夹具可重编且运行正确，差距在**有提前出口的谓词尾路径归属**，不需要重写浮点比较。

[窄 OpenSpec](../../changes/recover-proved-early-return-predicate-tail/proposal.md)先要求阻止丢后继还发表面完整的错误控制流，再在同一物理 CFG/SSA/Region 与 builder 证据内恢复此 Java 8 形状。若某个后继、返回值、异常边或预算不能完整证明，输出必须原子回退并保留全部可达 BCI，不得在真实分支前写无条件 `return false`。JADX 的逻辑条件合并可参考，但三方运行行为和来源闭合是接受标准。

重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf02-predicates/replay.py \
  --jarde /tmp/jarde-cli-accepted-em06 \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/cf02-audit-replay
```

`--out` 必须为空目录。脚本目前把 Jarde 最后一行错误值作为冻结预期，修后验收须改为三侧逐字一致。原 class/JAR 和 Java 编译目录置于自动清理的临时目录，仓库仅保存源码、诊断和日志。
