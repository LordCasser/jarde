# CF-07：标准循环的首片与循环内提前返回

固定队列项为 [CF-07](../../jadx-feature-inventory-2026-09-27/control-flow.md)。[replay.py](replay.py)固定 JADX 提交、`TestLoopCondition2`、`TestLoopDetection2`、`TestLoopCondition5` 和对应 `LoopRegionVisitor`、`LoopRegionMaker` 的内容哈希。[LoopCases](input/cf07/LoopCases.java)只取三项能作为独立 Java 8 程序重放的正例：布尔与上界共同作循环头条件、含 if 的计数循环、倒序查找循环内提前返回。`TestLoopCondition5` 的另一个 Smali 断言允许 `for`/无限 `while`/`do while` 三种呈现，只是弱文本边界；`TestComplexWhileLoop` 和 `TestLoopConditionInvoke` 含 CF-06 的条件内赋值，仍由独立任务处理。

[基线](baseline/summary.json)中原 class 与固定 JADX 的完整类源码都以 `javac --release 8 -g:none` 重编、`java -Xverify:all` 运行，八行结果同为 `0,10,39,63,9,3,-1,3`。Jarde 的 `andWhile` 与 `counted` 已恢复成能编译运行的 `while`。`lastIndexOf` 的[物理字节码](baseline/javap.log)在 BCI 5 进入倒序循环，BCI 16 条件跳转，BCI 19 读循环变量并 `ireturn`，BCI 22 递减并回边，BCI 28 返回 -1。[基线区域](baseline/regions.json)只把 5/11/22 归给循环，19 留作 `UncoveredBlocks`；跨 quote 的局部变量词法计划正确拒绝完整正文。

修复在现有循环 `Frame` 中仅接纳一个终止返回叶：它从自然循环内比较分支进入，只有该分支的一条 normal 入边，没有外出边；SSA 块恰为单次 `iload; ireturn`，且返回只消费这次加载。候选扫描按预算计费并传播取消。循环体必须实际遍历该叶并完成所有权覆盖，否则沿原拒绝路径回退。`build.rs` 的局部作用域规则、`Region`/AST 形状均未改变。[修后区域](after/regions.json)把 BCI 5/11/19/22 一次归给循环，BCI 28 保持外层归属；定向测试还检查 BCI 19/21 来源与一次 `return local4;`。

[修后重放](after/summary.json)以同一固定输入和 JADX 提交验证三份**完整类源码**：原始、JADX、Jarde 的 Java 8 重编和 `-Xverify:all` 均成功，八行输出完全一致。原 class SHA-256 为 `004e99baeb062861f95a4dac13e3a78399a66efd31f2589b0028cfe50432ae33`；三份源码 SHA-256 依次为 `3184f43aa78ba2f26752d15e0b706fe7b0cc9cf25bca34dded34bf603ecb3c38`、`8dc5b1a14452ee65c82702c480c2e9aa5ecbdfe1f8fd1db0c4d384f55aefdc4d`、`ecbd892b39e3f36d634509de8bec9c4a5b811e96fe5a20033cc38ce896655b23`。[修后 `javap`](after/javap.log)与[完整 Jarde 类源码](after/source/jarde/cf07/LoopCases.java)随结果保存。

[负例源码](../../../../tests/fixtures/p3-loop-terminal/LoopTerminalNegatives.java)覆盖共享叶、非终止 `break`、异常路径及返回值缺证；经 verifier 验证的 `LoopTerminalExtraEntry.class` 把循环前的独立入口指向循环内叶。未证明的终止叶保留 bytecode 和来源，普通 `break` 保持原有恢复行为。CF-03 共享尾、CF-07 定向测试与预算/取消原子性测试通过。CF-06 条件内赋值仍待单独实现，CF-08 多 `break` 循环也不属于本项。

重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf07-basic-loops/replay.py \
  --jarde /path/to/jarde-cli \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /empty/output/directory
```

`--out` 必须为空目录；脚本硬性要求三方完整源码重编、验证运行和八行结果一致，Jarde 源码不能含 `@bytecode`。
