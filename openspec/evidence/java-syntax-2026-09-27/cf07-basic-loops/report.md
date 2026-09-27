# CF-07：标准循环的首片与循环内提前返回

固定队列项为 [CF-07](../../jadx-feature-inventory-2026-09-27/control-flow.md)。[replay.py](replay.py)固定 JADX 提交、`TestLoopCondition2`、`TestLoopDetection2`、`TestLoopCondition5` 和对应 `LoopRegionVisitor`、`LoopRegionMaker` 的内容哈希。[LoopCases](input/cf07/LoopCases.java)只取三项能作为独立 Java 8 程序重放的最小正例：布尔与上界共同作循环头条件、含 if 的计数循环、倒序查找循环内提前返回。`TestLoopCondition5` 的另一个 Smali 断言允许 `for`/无限 `while`/`do while` 三种呈现，只是弱文本边界；`TestComplexWhileLoop` 和 `TestLoopConditionInvoke` 含 CF-06 的条件内赋值，待该依赖厘清后再测。

[基线](baseline/summary.json)中原 class 与固定 JADX 的**完整类源码**都以 `javac --release 8 -g:none` 重编、`java -Xverify:all` 运行，八行结果同为 `0,10,39,63,9,3,-1,3`。Jarde 的 `andWhile` 与 `counted` 已恢复成能编译运行的 `while`，顺序、副作用和结果未发现差距；固定 JADX 把后者的一次 if 写成条件值，Jarde保留 if/else，在本项中是合法等价形式。

Jarde 的完整类源码因 `lastIndexOf` 没有方法正文而不能重编。[物理字节码](baseline/javap.log)在 BCI 5 进入倒序循环，BCI 16 条件跳转，BCI 19 读循环变量并 `ireturn`，BCI 22 递减并回边，BCI 28 返回 -1。[同轮区域详情](baseline/regions.json)显示循环区只持有 5/11/22，BCI 19 被标为 `UncoveredBlocks`；它是只从循环体条件分支到达的终止叶，但不在自然循环集合，因为它没有回边。局部 4 在循环与该 quote 两处使用，`build.rs` 的词法计划因此按现有规则原子拒绝整个方法。这是区域所有权遗漏先触发、局部作用域拒绝随后正确阻止半份源码的情况；不应直接放松局部规则。

后续窄任务应在现有 Loop 区域中证明这类终止叶由循环内唯一分支拥有，且值来源、异常边、其他入口与退出路径均闭合，再由局部作用域复用既有证书。不要借自然循环集合直接收下任意出口叶；多入口、共享终止块和带额外副作用的出口需保持拒绝。CF-06 条件内赋值另立任务。

重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf07-basic-loops/replay.py \
  --jarde /tmp/jarde-root-cli-cf04-791641c2 \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/jarde-cf07-basic-replay-20260927
```

`--out` 必须为空目录。当前脚本固定原 class 与 JADX 完整源码成功；保留 Jarde 的失败作修前事实，修后再提高三方硬门槛。
