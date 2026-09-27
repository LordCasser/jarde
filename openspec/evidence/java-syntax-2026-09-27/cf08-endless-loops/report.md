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

## 双网关窄修复验收

[修后重放](simple-after/summary.json)启用 `replay-simple.py --require-jarde`：原 class、固定 JADX、Jarde 的**完整** Java 8 类均重编成功，在 `java -Xverify:all` 下六行同为 `0,1,2,3,3,3`；`limit=4` 正常结束，没有 `@bytecode`。本次 CLI SHA-256 为 `f36d347f11cca0e6ce37f27018aa27875f8195c42fc8a773a37592c609ff6c7e`，三份源码 SHA-256 依次为 `e657d1e5571cd3bf2ebd64f2b6d0ffdc761bb738b94143488fbb83abbc46778a`、`7f563d32983c0949231b7445f44d70f893379a56899f33e172b9ccdef3191aea`、`9fff4ace10f7da20544a4cd5852daa73d21c10f2f4e4c54c13ef9864d880c2e0`。

[修后区域](simple-after/regions.json)将 BCI 2/10/15/18 一次归给循环，BCI 7/24 留在外层。头部 BCI 7 和体内 BCI 15 是分别由精确单一正常入边到达、直接转接到同一个真实后继 BCI 24 的纯 `goto`；循环唯一回边的终止转移为 BCI 21。两网关证明后，`Region::Loop.exit` 仍记物理 BCI 7，`LoopTarget.break_target` 改指 BCI 24，体内 BCI 15 才有循环 `break` 所有权。定向测试核实 BCI 4/7/12/15/18/21/24/25 都在 source map；纯转移 BCI 7/21 只在双网关证书成立时附于 loop 语句的派生来源。

额外入口的 verifier 有效负例、不同最终目标、带效果网关与异常边都保留物理引用，不能被循环误认领；预算和取消没有半份源码。完整 `jarde-java` 测试、`p3_loop_transfers`、CF-07 固定三方八行回放、CF-09 `Grid` 与 `OuterContinue` 三方回放均通过。`cargo fmt --all -- --check`、`cargo check -p jarde-java -p jarde-cli`、`git diff --check` 和 OpenSpec strict 校验通过；独立 Cargo target 经 `cargo clean` 删除 17,801 个文件、6.8 GiB。复杂 `TestNotIndexedLoop` 的外层分支和局部汇合仍未处理。
