# CF-02：比较谓词与提前返回后的可达路径

固定队列项为 [CF-02](../../jadx-feature-inventory-2026-09-27/control-flow.md)。[replay.py](replay.py) 校验 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestCmpOp`、`TestCmpOp2`、`TestConditions7`、`TestTernary3` 和五个条件实现入口的哈希。[input/cf02](input/cf02) 用合法 Java 8 顶级 `Predicates` 与共同 Runner 缩小测试：浮点比较含 NaN、无穷、负零；数组边界含负数、合法和越界索引；`named(Object)` 是固定 `TestTernary3` 的判空、`instanceof`、cast、提前 `return false` 与后续布尔返回形状。本片未覆盖该测试的 `InsnArg` 外部类层级、`getName` 多分支和其它 `TestCmpOp` 的全部文字断言。

[baseline/summary.json](baseline/summary.json) 记录原 class、JADX 和 Jarde 的输入/生成源码哈希。三份**完整 `Predicates` 类源码**各自与相同 Runner 用 `javac --release 8 -g:none` 重编，均以 `java -Xverify:all` 运行。前九行逐字一致：浮点 NaN 比较为 false、正无穷大比较为 true、`-0.0f < 0.0d` 为 false、边界检查为 false/true/false，判空、非 String 和空串为 false。最后 `named("x")` 原 class 与 JADX 为 `true`，Jarde 为 `false`。这是一项已证的行为偏差，即使 Jarde 源码本身能通过编译。Jarde 对该方法的报告是 `quality=fallback`、`semantic_validation=unproven`，源码也写出 `// @bytecode 28` 和未恢复标记；不能将“能编译”误计为完成恢复。

[Jarde 完整源码](baseline/source/jarde/cf02/Predicates.java)把 `named` 输出成两个空 `if` 后的无条件 `return false`，并在其后注释“live block [28]”；[JADX 完整源码](baseline/source/jadx/cf02/Predicates.java)保留 `obj != null && obj instanceof String && ((String)obj).length() > 0`。`javac` 的 BCI 13 进入 cast/`String.length()`，BCI 20 比较，23/27 产生布尔值，28 返回；这个正常路径绝非可忽略的异常 handler。

根因落在现有 Region 候选/遍历边界。[region-probe.log](baseline/region-probe.log)显示方法的 canonical blocks 为 `[0,4,11,13,23,27,28]`，区域 walk 仅访问 `[0,4,11,13,23,27]`；BCI 13 的条件以 BCI 28 为后继合流，而外层一个 `if` 以提前返回块 BCI 11 为 join。`region.rs::region_at` 的 one-armed `if` 路径调用内层 `region_at` 后丢弃 `arm_next`，于是把可达的 BCI 28 留给尾部 `UncoveredBlocks`，同时已输出了可编译的无条件假返回。`short_circuit_value` 当前仅接收方法级或受证 try 头，不能直接把这个嵌套返回值接成已证明的布尔表达式。比较运算本身在同一夹具可重编且运行正确，差距在**有提前出口的谓词尾路径归属**，不需要重写浮点比较。

[窄 OpenSpec](../../changes/recover-proved-early-return-predicate-tail/proposal.md)先要求阻止丢后继还发表面完整的错误控制流，再在同一物理 CFG/SSA/Region 与 builder 证据内恢复此 Java 8 形状。若某个后继、返回值、异常边或预算不能完整证明，输出必须原子回退并保留全部可达 BCI，不得在真实分支前写无条件 `return false`。JADX 的逻辑条件合并可参考，但三方运行行为和来源闭合是接受标准。

## 修后验收

[accepted/summary.json](accepted/summary.json) 与同目录日志和完整生成源码记录了修后重放。原 class、固定 JADX、修后 Jarde 的 `Predicates` 完整类源码分别由 `javac --release 8 -g:none` 重编，并以 `java -Xverify:all` 运行；三方十行逐字一致，末行 `named("x")` 都是 `true`。修后 Jarde 源码 SHA-256 为 `014bdae95be4e1ae43a3426efa7d18e5dd82a05bb05cc4880b1c4b09601c9dd0`，原 class 为 `b5ee52d137db638aefdd2f6554c440b38d59e83e6cf4b42c0de28f93f4e176d5`，JADX 源码为 `02a33a1d6ed3e61367d2504e1e2dacc1cf71d26e4aecdbf3df06d701b34008fc`。三个 `javac.log` 与 `runtime.log` 均记录 `exit=0`；前六行同时锁定 NaN、无穷、负零和数组下上界。Jarde 的 `named` 现为 `quality=structured` 且无 fallback；`semantic_validation=unproven` 仍表示引擎本身没有执行目标代码，本次行为一致是独立重编运行的外部证据。

另一个独立类 [PredicateEffects.java](input/effects/cf02/PredicateEffects.java) 在安全 cast 后通过 `probe` 计数，配合同一个四输入顺序的 [EffectsRunner.java](input/effects/cf02/EffectsRunner.java) 检查效果次数。原 class 与修后 Jarde 完整类源码各自重编运行，输出均为 `false:0`、`false:0`、`false:1`、`true:1`：判空和错误类型不调用，空串与 `"x"` 各调用一次。[accepted/effects-jarde/runtime.log](accepted/effects-jarde/runtime.log) 和 `summary.json` 保留诊断、输出与源码哈希。区域前驱单元负例额外拒绝重复前驱、来自提前返回块 BCI 11 的第三前驱以及异常/子程序边；无法接续的尾块按整方法字节码引用，不发表可执行的无条件假返回。builder 在该尾形状中遇到值或效果拒绝时也按整方法原子引用。[accepted/budget-stop.log](accepted/budget-stop.log) 另记录极低输出预算下退出码 2、标准输出 0 字节的原子停止。

本次修复只准入已证单一布尔 `ireturn` 的提前返回尾路径。固定队列中的 `TestCmpOp`、`TestCmpOp2`、`TestConditions7` 的其他比较/条件变体和 `TestTernary3` 外部 `InsnArg` 类型层级仍待各自扩验；这些形状的通过不能由本十行对照推断。root 的独立三方验收另行执行。

重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf02-predicates/replay.py \
  --jarde /tmp/jarde-cli-accepted-em06 \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/cf02-accepted-replay
```

`--out` 必须为空目录。脚本现在要求三方十行逐字一致，并额外检查效果次数。原 class/JAR 和 Java 编译目录置于自动清理的临时目录，仓库仅保存源码、诊断和日志。
