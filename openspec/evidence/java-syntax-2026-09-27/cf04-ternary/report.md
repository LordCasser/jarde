# CF-04：条件值与构造器实参

固定队列项为 [CF-04](../../jadx-feature-inventory-2026-09-27/control-flow.md)。[replay.py](replay.py) 固定 JADX 提交、`TestTernary`、`TestTernaryInIf`、两份 `TestTernaryOneBranchInConstructor` 与 `TernaryMod`/`InsnGen` 的哈希。测试代表普通数值、布尔和嵌套条件值，以及构造器 `this(...)` 中的一或两个条件实参；第二份构造器测试是 Smali 输入，不能单凭文本断言证明完整 Java 8 源码自洽。

[baseline/summary.json](baseline/summary.json) 记录两份合法 Java 8 隔离类。`TernaryBasic` 覆盖数值/布尔返回、两种构造器委托和有副作用的二选一调用。原 class、固定 JADX 与 Jarde 的**完整类源码**均用 `javac --release 8 -g:none` 重编、以 `java -Xverify:all` 运行，11 行逐字一致；`1:1`、`2:1` 证明两个分支中只执行被选择的 `arm` 一次。Jarde 已正确写出 `this(arg1 == null ? 0 : arg2)` 和双条件 `this(...)`，不需要重做构造器机制。

完整 `TernaryCases` 增加 `nested(boolean,boolean,boolean)`，源码形状为 `return (!first ? third : second) ? 1 : 2;`。原 class 与 JADX **完整类源码**编译、验证运行的 13 行一致；Jarde 的其它方法仍可读，但 `nested` 整方法引用、缺返回，使完整类编译失败。[物理 `javap`](baseline/javap.log)显示 BCI 1/5/12 的条件测试汇到 BCI 15/19 的整数 1/2 生产者，BCI 20 唯一 `ireturn`。Jarde 已由 `region.rs::short_circuit_value` 识别共享值消费者，却在 builder 报“no SSA proof for that value”。现有 `build.rs::prove_short_circuit_value` 把叶值固定为 `1/0`，且只把 `ireturn` 判为布尔返回；因此即使该 SSA 图可以闭合，整数 `1/2` 也被当前证明子集拒绝。后续布尔简化还假设叶值为 `1/0`。差距是**嵌套条件值的整数返回证明/拼写**，不需要新 CFG 探测层。

[窄 OpenSpec](../../changes/recover-proved-nested-int-conditional-return/proposal.md)要求只在现有 Region/SSA 边、两个精确 int 叶值和唯一 `ireturn` 全闭合时沿用条件表达式 builder；整数结果必须保留 `?:` 的值，不能走布尔化简。未知 producer、额外消费者、异常边、预算停止仍整段拒绝。CF-02 提前返回尾路径已合入后，root 以相同脚本复放，`TernaryCases` 差距仍在，故不是 CF-02 的内层后继问题。

重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf04-ternary/replay.py \
  --jarde /tmp/jarde-cli-main-46968a97 \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/cf04-ternary-replay
```

`--out` 必须为空目录。这里保留修前完整类失败和隔离基本形状通过两类事实；输入 JAR 与编译目录自动清理，仅保存源码、`javap`、日志和哈希。
