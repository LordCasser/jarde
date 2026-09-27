# CF-08：双层 `if` 中的带效果双出口循环基线

该夹具在已验收 `NestedEffectful` 形态上增加一个有界空数组分支：`pick(int[] xs)` 对 `null` 返回 `-1`；非空时缓存 `xs.length`，长度为零也返回 `-1`；其余路径执行无限循环，在越界出口调用会增加 `calls` 的 `cost(7)`，或在命中 `3` 时提前退出。内层汇合后只执行一次 `result += calls`，最后与外层 `null` 臂汇合。Runner 的 Java 8 输出为 `-1:0 / -1:0 / 3:0 / 8:1`。

固定输入和 Runner 分别由 SHA-256 `c98dd5aa3d633c96ea9b894ef85cf9665bb7365b7dc74d29418d1255244f44ce`、`303e2d371c2d724ab33397bc946bd57e0fa954bf60f35ed7341ea3c6af04b368` 锁定。`javac --release 8 -g:none` class SHA-256 为 `b5a7541fc6dc6a6eaff229f3559968cd7e20a4f48c063e4fd019186f92b4a240`，保存于 [`baseline/classes`](baseline/classes)；[`javap`](baseline/javap.log) 冻结了实际 BCI：外层判空在 0–9，长度判断在 9–21，循环头为 23，循环边界判断在 23–37，`cost(7)` 及退出为 28–34，数组读取和命中判断为 37–46，递增回边为 49–52，内层汇合并加 `calls` 为 55–60，外层共同返回为 61。

固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`；脚本还校验与 NestedEffectful 相同的七个源文件哈希。原 class 与固定 JADX 产生的完整 `TwoLevelIf` 源码均通过 Java 8 编译和 `java -Xverify:all`，四个结果逐项相同。当前 Jarde CLI SHA-256 是 `567a04f18dce6e015ac1bff1e714b39d28aba97adf89f9be4bfa49d8f843355c`。

真实 Region 证据见 [`regions.json`](baseline/regions.json) 和原始 [`region-report.log`](baseline/region-report.log)。Jarde 将外层 BCI 0 报为 `jre_region_arms_do_not_meet`，具体诊断是 `the arms of the branch in block 0 do not meet at one join`；此外 BCI `[23, 28, 37, 46, 49]` 是 `jre_region_uncovered_blocks`，报告它们只能经 normal-flow view 遗漏的边到达。生成源码对 `pick([I)I` 给出 `@bytecode 0 4 9 16 21 23 28 37 46 49 55 61`，随后精确拒绝原因为 `local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice`。因此完整 Jarde 源码编译在第 46 行以“缺少返回语句”失败；这是安全拒绝红基线，而不是等价恢复结果。

复放会先构建当前 Jarde CLI，并要求输出目录为空：

```sh
cargo build -p jarde-cli --target-dir /tmp/jarde-cf08-two-level-target
python3 openspec/evidence/java-syntax-2026-09-27/cf08-endless-loops/two-level-if-baseline/replay.py \
  --jarde /tmp/jarde-cf08-two-level-target/debug/jarde-cli \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/jarde-cf08-two-level-replay
cargo clean --target-dir /tmp/jarde-cf08-two-level-target
```

`baseline/summary.json`、逐步日志和三方源码保存本次运行证据；默认脚本校验这个预期拒绝。`--require-jarde` 可在未来恢复后要求 Jarde 完整源码也通过编译和验证运行。
