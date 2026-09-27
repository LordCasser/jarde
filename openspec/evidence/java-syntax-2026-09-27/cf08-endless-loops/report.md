# CF-08：无头无限循环与显式出口

本轮只审计 CF-08，不改生产代码。固定输入来自 JADX `2fb1b16386941660fda07e9017285aec40fcb37f`。`replay.py` 固定了四个代表测试和 `FixMultiEntryLoops`、`LoopRegionMaker`、`LoopRegionVisitor` 的 SHA-256；输入 Java 是从 `TestNotIndexedLoop.TestCls.test` 逐句抽出的可执行 Java 8 正例，并增加一个 runner 覆盖空输入、命中和未命中。

四个固定测试的断言强度不同：

- `TestEndlessLoop` 是真实 Java 源类，但只断言输出包含 `while (this == this)`。`test1` 和 `test2` 都能满足该子串，断言没有约束第三个 `while (true)` 的结构或语义。
- `TestEndlessLoop2` 是 Smali 测试，禁用编译，只断言输出中有两个 `while (true)`。它是文本形状证据，不是可重编运行的 Java 正向样例。
- `TestMultiEntryLoop` 也是 Smali 测试，只断言一个 `while (true)`，不单独检验入口复制或出口语义。
- `TestNotIndexedLoop` 是可编译 Java 正例：普通 `while (true)` 在索引越界和找到目标时都 `break`，随后做 `deleteOnExit` 并返回结果。两组测试（含 `noDebugInfo`）断言不生成 `for` 且恰有一个 `while (true)`。其内嵌 `check()` 还由 JADX `IntegrationTest` 自动在原类和编译后的反编译类上运行，覆盖 `null`、空数组、命中 `f`、未命中返回 `h`。

对应实现也表明 CF-08 不是单纯输出 `while (true)`：`FixMultiEntryLoops.process` 用 DFS 标记 back/cross edge，再对头部不支配回边尾部的循环尝试两种受限块复制；无法唯一确认入口模式时保留警告。`LoopRegionMaker.makeEndlessLoop` 为无条件 loop region 查找有效公共出口，并把出口边转成循环 `break`。后续 `LoopRegionVisitor.processLoopRegion` 在条件为 null 时直接不做 indexed/foreach 改写，因此不会把这类无限循环强制改成 `for`。四个代表测试中只有 `TestMultiEntryLoop` 直接触及多入口修正；本次 Java 8 正例更具体地审计无头循环的多个 break 出口和循环后局部值汇合。

## 三方 Java 8 回放

固定 runner 对四种输入输出 `null / null / f / h`。原 class 与 JADX 完整类源码均以 `javac --release 8 -g:none` 编译成功，并以 `java -Xverify:all` 运行成功，输出逐行相同。JADX 的源码保留一个 `while (true)`，其两个出口都写成 `break`。

Jarde 在完整类恢复中将 `test(File[])` 标为 `@bytecode`，并报告 `local 2 crosses a quoted fallback region`；返回的源码因此缺少方法体，Java 8 编译因缺少返回语句而失败。完整回放输出、各源码和 `javap` 指令均保存在 `baseline/`。物理字节码中索引循环是 BCI 19–61，索引越界出口在 BCI 34–35 写入 `File("h")` 后跳到公共后继 BCI 64，命中出口在 BCI 55 也到 BCI 64；之后 BCI 69、73、77 对结果做空检查、`deleteOnExit` 和返回。slot 2 同时承载循环出口的赋值及循环后的消费者，Jarde 当前方法级拒绝边界阻止了不完整局部变量切片。

这个差距落在 CF-08 可终止的 `while (true)` 加显式 `break` 正例上。它与 CF-07 已记录的循环出口所有权/局部变量范围问题相邻，但物理形态是两个循环内 break 赋值汇合到循环后清理，而不是循环体内唯一分支到达的终止 `return` 叶。这里仅记录实测证据；是否复用或扩展现有 `own-proved-loop-terminal-return` 工作项，应在该工作项中判断，不另起实现机制。

## 缩小后的双出口反例

为了把上述复杂样例的外层空值分支、`File` 调用与局部汇合分开，本目录另固定 `input/simple/cf08/EndlessInts.java`：整数 `i` 的 `while (true)` 中，`i >= limit` 与 `i == 3` 分别 `break` 到同一个返回块。`replay-simple.py` 使用同一 JADX 提交与源码哈希固定条件，记录在 `simple-baseline/`。原 class 与 JADX 完整源码以 Java 8 重编、`-Xverify:all` 运行，六行同为 `0,1,2,3,3,3`。JADX 将其等价写为 `while (i2 < i && i2 != 3)`。

Jarde 的 Region 证据先把 BCI 2/10/18 认作循环，随后把第二个出口的 BCI 15 `goto 24` 列为未覆盖块；输出的 `if (local1 == 3) {}` 没有 `break`，尾部附带 `@bytecode 15`。这份带引用标记的类**恰好仍能通过 javac**，但在 `limit=4` 的运行中没有按原 class 于 3 停止，五秒内不退出。故“能重编”不能越过 `@bytecode` 和执行对照来宣称正确。这个最小反例表明 CF-08 至少有一条独立的循环出口所有权缺口；复杂 `NotIndexedLoop` 的方法级 `jre_region_arms_do_not_meet` 与跨引用局部范围是相邻的另一个形态，后续任务不得把它们一起用放宽局部检查解决。

重放命令：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf08-endless-loops/replay.py \
  --jarde /path/to/jarde-cli \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /path/to/empty-output-directory
```

当前固定回放记录的 Jarde CLI SHA-256 为 `6b94a3e7c5d640d7034c96827913db08f7d473c88008418037f776bc1d80d8cd`。`javac` 对 Java 8 选项给出的弃用提示不影响退出状态或回放结果。
