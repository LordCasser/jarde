# CF-09：嵌套循环转移目标

本轮只审计并记录证据，不改生产代码或新增 OpenSpec。固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的五项账本测试，以及 `LoopRegionMaker`、`LoopRegionVisitor`、`LoopLabelAttr` 的 SHA-256 由 [replay.py](replay.py) 固定。

五项现有断言强度并不相同：

- `TestBreakInLoop` 断言一个计数 `for`、内部 `if`、无标签 `break` 和循环后的字段递增，还要求没有 `else`；`a[i]++` 断言被注释掉。它没有 `check()`，因此 IntegrationTest 不会执行转移语义检查。
- `TestContinueInLoop` 断言计数 `for`、一个 `continue`、一个 `break` 和循环后字段递增，但不含 `check()`，也不涉及标签或循环目标。
- `TestBreakWithLabel` 断言 `loop0:` 与 `break loop0;`。它另有公开 `check()`，IntegrationTest 会在原类与已编译的 JADX 类上分别执行，命中和未命中两种结果都受检查；这是五项中最直接的运行语义证据，但只覆盖向外层循环 `break`。
- `TestDoWhileBreak` 只断言源码含一个 `while (`，没有断言 `break` 所属的循环，也没有 `check()`。
- `TestSequentialLoops` 断言两个循环、一个 `break` 和尾部 `return c;`，没有 `check()` 或标签目标。

JADX 的实现先把循环边作为转移候选处理：`LoopRegionMaker.insertLoopBreak` 按实际出口边添加 break，`addBreakLabel` 只在目标跨出多个循环且不是父循环自然出口时附上标签；`insertContinue` 根据回边前驱和出口节点确认 continue。`LoopLabelAttr` 保存目标 `LoopInfo`，后续代码生成按该目标发出标签。`LoopRegionVisitor` 还会把循环更新归入 `for` 头，因此外层 `continue` 必须跳过体尾语句并执行正确的 update。

## 三方 Java 8 回放

回放重用 `Grid` 与 `OuterContinue` 两组已有 Java 8 输入和 runner，并先验证重编 class 与各自冻结 class 字节相同。默认包下 JADX CLI 会添加合成的 `package defpackage;`；回放只剥掉这一行，使输出回到 class 文件真实的默认包，其他源码保持原样。

原始源码、JADX 完整类源码、Jarde 完整类源码均通过 `javac --release 8 -g:none` 重编，并由 `java -Xverify:all` 运行。两组输出逐行一致：`Grid` 的五个输入为 `0:0:0:0`、`1:1:1:11`、`3:9:6:39`、`6:21:12:3`、`9:21:18:3`；`OuterContinue` 为 `0:0`、`1:101`、`2:204`、`3:6`、`5:10`。完整源码、命令日志和原始 `javap` 结果保存在 `baseline/`。

判定标签时按字节码目标，而不是只看原 Java 标签。`Grid.labeledContinue` 的 BCI 21 跳到 BCI 34；该块同时是内层循环自然出口和外层递增入口，内层 `break` 与原标签 `continue` 等价，因此 Jarde 输出 `break;` 是正确恢复。`Grid.labeledBreak` 的 BCI 21 跳到外层独立出口 BCI 45，Jarde 保留外层标签 `break jarde_loop_4;`。`OuterContinue.run` 的 BCI 21 跳到外层递增 BCI 36，越过内层后的 `total += 100`（BCI 33）；Jarde 输出带外层标签的 `continue jarde_loop_4;`，并把递增保持在 `for` 头。运行值确认两条标签路径都命中真实目标。

这两种目标在本轮固定样例上没有发现 Jarde 与原/JADX 的源码编译或运行差距。范围仍是窄首片：其他固定测试多为编译后文本形状断言；本回放没有证明所有 break/continue 与 switch、try/catch、共享出口或其他条件合流组合。`p3_loop_transfers` 五项结构/source-map 测试也在当前工作区通过，但它们不替代更广泛的原 class/JADX/Jarde 语义矩阵。

重放命令：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf09-loop-transfers/replay.py \
  --jarde /path/to/jarde-cli \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /path/to/empty-output-directory
```

本次重放使用 Jarde CLI SHA-256 `32bf54586fe54655f3d21e1f94c71548aa280b9b442e3918db36b67cd1d41b3a`。Java 8 参数的过时提示来自当前 JDK；所有编译和运行退出码仍为 0。

root 在含 CF-07 的主线重新构建 CLI（SHA-256 `c738c75b0834390de9e8bab46bfc1d7376f0ce90bd9319a0e6804708b3dbb976`），独立重放到 `/tmp/jarde-cf09-root-acceptance`。两组原/JADX/Jarde 完整类仍各自 Java 8 重编、验证运行且五行逐字相同；`p3_loop_transfers` 5/5 通过。故此处只验收 CF-09 标签目标首片，不把未测组合算作整个单元追平。
