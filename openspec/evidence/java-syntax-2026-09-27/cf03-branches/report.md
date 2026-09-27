# CF-03：else-if 链、嵌套分支与共同出口

固定队列项为 [CF-03](../../jadx-feature-inventory-2026-09-27/control-flow.md)。[replay.py](replay.py) 固定 JADX 提交及 `TestElseIf`、`TestNestedIf`、`TestConditions3`、`TestOutBlock` 与 `IfRegionMaker`/`IfCondition`/`RegionGen` 的哈希。前三个测试分别给出 else-if 赋值后共同出口、嵌套判定后的共同尾、顺序早退结构；`TestOutBlock` 是 Smali 测试，明确 `disableCompilation()`，只能供容错分析，不能算 Java 8 完整源码正向能力。

[baseline/summary.json](baseline/summary.json) 记录两个隔离夹具。`BranchShapes` 的原 class 与固定 JADX **完整类源码**分别通过 `javac --release 8 -g:none`、`java -Xverify:all`，15 行结果一致。Jarde 的完整类源码在 `nested` 整方法回退后缺少返回，编译失败。[物理 `javap`](baseline/javap.log) 显示 true gate 的 BCI 9 和 false gate 的 BCI 19 都可能抵达 BCI 24 的一次 `hits++` 及 `return true`；Jarde 报告 `canonical block at BCI 24 ... more than one owner in the completed Region tree` 并整方法引用。这是共享尾区域归属问题，不能靠复制尾效果或把 `return true` 提到任一分支修复。它与 CF-02 正在修的内层可达后继接近，需先在 CF-02 合入后重放，再决定是否另开区域任务。

独立的 `ChainOnly` 消除了嵌套共享尾的影响。其原 class、JADX、Jarde **完整类源码**均重编、验证运行五行一致：`10:1`、`20:2`、`30:3`、`40:4`、`10:14`。末项还证明每个 `matches` 只按分支顺序求值，并在默认分支额外执行一次 `hits += 10`。JADX 输出三个 `else if`；[Jarde 完整源码](baseline/chain/source/jarde/cf03/ChainOnly.java)把完全相同的 AST 链排成三个逐层嵌套的 `else { if (...) ... }`，`else if` 计数为零。这是独立的源码可读性差距，现有 `StmtKind::If` 已保留嵌套单语句 else 的结构，不需要新的 CFG 机制。[窄 OpenSpec](../../changes/spell-proved-else-if-chain/proposal.md)拟仅在 emit 层把此已证形状写成 `else if`，同时保留每个子 `If` 的 origin/source-map span、预算与停止契约。

重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf03-branches/replay.py \
  --jarde /tmp/jarde-em12-root-target/debug/jarde-cli \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/cf03-branches-replay
```

`--out` 必须为空目录。脚本保留修前 `BranchShapes` 编译失败作为账本事实；修后若区域共享尾闭合，再改三侧运行验收预期。Jar 与编译物在临时目录自动清理，仓库只保存源码、摘要、诊断和 `javap` 文本。

## else-if emitter 修后重放

`post-else-if-chain/summary.json` 是 CF-03 else-if 拼写改动后的固定脚本结果，完整日志和三侧生成源码也保存在该目录。`ChainOnly` 原 class、JADX、Jarde 完整类源码均通过 `javac --release 8 -g:none` 和 `java -Xverify:all`；五行输出逐字相同：`10:1`、`20:2`、`30:3`、`40:4`、`10:14`。Jarde 输出三个 `else if`，与 JADX 数量相同；Jarde 源码 SHA-256 为 `07d19e0854e8842832d9e8c078d1988751744d6c589448eba1cc0900703e4b8a`。

同次完整 `BranchShapes` 重放中，原 class 与固定 JADX 仍通过 Java 8 编译和验证运行，15 行结果相同。Jarde 完整类源码仍在 `nested` 缺少返回语句而编译失败；`post-else-if-chain/jarde/javac.log` 记录该结果。else-if 呈现没有改变 AST，也没有修复或掩盖 BCI 24 的共享尾区域归属差距，后者仍需 CF-02 区域修复集成后重验。
