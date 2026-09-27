# CF-08：带效果的双出口隔离基线

输入为 [`EffectfulExits.java`](../input/effectful/cf08effects/EffectfulExits.java)（SHA-256 `1d8d906d5bb3bf10d7220d46af6453ef2baaa096a228f4d1a7a5ad3d4f857eaf`）与同目录 `Runner.java`（SHA-256 `ba71a9f1a2b557e5dc63f9163c446de60076273ce3d804fd4ca5d6ec26cbec0d`）。用 `javac --release 8 -g:none` 生成的类 SHA-256 为 `2e27cffb9361cbd78a689b404457b68d6bc31f1739362659ce71c0e276c0bae8`。固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`；Jarde 使用已验收的主线 CLI SHA-256 `e2c9e66b56ca335e6460cbfece233207eba8381d91c4eed472856893fbe34ab6`。该输入独立于 `NotIndexedLoop` 的外层 `if` 和 `File` 对象，仅保留循环头的带效果退出、体内另一条 `break` 与循环后同一局部消费。

`pick` 的物理循环头从 BCI 2 比较 `i` 与数组长度。越界边到 BCI 8–14，执行 `cost(7)`、写入结果局部并转 BCI 35；命中边从 BCI 26 直接跳 BCI 35，必须跳过 `cost`；BCI 29–32 自增回边。原 class 与固定 JADX 的**完整类源码**均通过 Java 8 重编和 `java -Xverify:all`，三组输入 `[]`、`[1,3,4]`、`[1,2]` 都输出 `8:1 / 3:0 / 8:1`。Jarde `class-source` 成功返回，但 `pick` 引用 BCI `0 2 8 17 26 29 35`，诊断 `local 2 crosses a quoted fallback region`；完整源码因缺少返回语句而编译失败。这是安全拒绝，不是运行语义正确。

现有 `Region::loop_exit_bridge` 只接受唯一入边、唯一出边且仅一条无效果 `goto` 的网关；`loop_exit_gateway_pair` 用它证明两个纯转接网关的共同后继。BCI 8–14 有调用和局部写，不能套用纯网关证书。`header_tested_loop` 目前把 BCI 8 作为循环正常出口；若直接把体内 BCI 26 写成 Java `break`，它会落到循环后的 BCI 8 并错误多执行 `cost`。因此不能靠放宽 `loop_exit_bridge` 的指令白名单解决；下一项设计必须先证明如何在现有 Region/Frame/AST 中保留“越界执行效果、命中跳过效果”的两个不同出口和共享后继，或者明确需要一个更窄的新表示。`NotIndexedLoop` 还叠加外层分支汇合，不能由此最小反例推定整体可恢复。
