# 共享 true 的短路字段写入：原件 / JADX / Jarde

日期：2026-09-25。输入是 [Java 8 源与冻结 class](../../../../tests/fixtures/p3-conditional-values/short-circuit-shared-true/)，`SharedTrueShortCircuit.assign(boolean)` 写 `result = left || rhs()`；`rhs()` 递增 `calls`。冻结 class 的 SHA-256 为 `4b3632dd7bc9bd26e640112a67a080d10f2171d0e83bf88f3190f67de412e935`。架构师重新用 `javac --release 8 -g:none` 编译，产物逐字节等于冻结 class；临时 runner 在 `java -Xverify:all` 下成功运行。`javap` 的 `assign` 为 BCI 1 `ifne 10`、BCI 7 `ifeq 14`、BCI 10/14 的 1/0 生产者、BCI 15 `putstatic result:Z`，StackMapTable 含目标帧。外层真边与内层真边共用 BCI 10，正好是当前 shared-false 证明的镜像控制骨架。

用冻结 class 单独打 jar，JADX 1.5.6 完整反编译出 `result = z || rhs();`（[源码](jadx-SharedTrueShortCircuit.java)）。Jarde 当前整类输出把 `assign` 的 BCI `0 1 4 7 10 11 14 15 18` 放在一条可见引用中（[源码](jarde-SharedTrueShortCircuit.java)）；[JSON 报告](jarde-report.json)为 `outcome=performed`、执行完整，但该方法 `quality=fallback`、`representation=mixed`。这些 BCI 同时出现在其全证据 source map 中；没有伪造一个已证明的字段赋值。

三者均以 `javac --release 8` 编译被测完整类及同一临时 runner，再以 `java -Xverify:all` 运行。原 class 与 JADX 输出相同：

```text
left=true,result=true,calls=0
left=false,result=true,calls=1
```

Jarde 的引用行只是注释，所以生成类虽能编译，运行时字段保持默认值，也不调用 RHS：

```text
left=true,result=false,calls=0
left=false,result=false,calls=0
```

各次编译和执行的实际输出保存在本目录的 `original-*`、`jadx-*`、`jarde-*` 日志中。此差异是一个**未恢复且明确降级**的 Java 8 `||` 字段写入；不能因源码可编译而当作语义等价。当前 Region 已能一次认领两生产者及写入，所以预计只需把现有 `ShortCircuitValue` 的 CFG/SSA 证明和 `Conditional` 发射推广到共享 true 的极性，不需要新 Region 或通用表达式机制。

本地 JADX 的 `IfRegionMaker.mergeIfInfo` 在可合并路径上按继续走的分支选择 `AND` 或 `OR`，`TernaryMod.makeTernaryInsn` 又要求两臂结果汇入同一个 Phi 才折叠；这是可参考的判定顺序。Jarde 应直接用物理 CFG 边、Phi 各前驱出口、唯一 `putstatic` 消费和无额外效果证明，而不是仅按地址/方法使用者集合猜极性。JADX 这份输出通过本样本的重编和行为对照，不能由一个正例推断其对额外入口、第二消费者或异常边也安全。

## 实现后的独立复核

现有 `ShortCircuitValue` 证明现在由 decode target 和 canonical 边同时确认外层跳转抵达 true 生产者，并保留原来的 shared-false 路径。架构师重新运行 `cargo test -p jarde-java --lib frozen_shared_true_write_proves_and_emits_one_delayed_rhs_assignment`，1 项通过；用更新后的 CLI 对冻结 class 做一次完整 `class-source --evidence all`，`assign(Z)V` 报告为 `quality=structured`、`representation=java`、`fallbacks=[]`。生成源码见 [Jarde 实现后输出](jarde-after-SharedTrueShortCircuit.java)，仅有一次 `result = (!arg0 ? rhs() ? 1 : 0 : 1) % 2 != 0`。全证据 source map 覆盖 BCI 0/1/4/7/10/11/14/15，没有把 RHS 调用提前执行。

架构师将原始 class、JADX 1.5.6 的完整类源码、Jarde 的完整类源码分别与同一临时 Runner 用 `javac --release 8` 编译，再以 `java -Xverify:all` 运行。三者均输出：

```text
left=true,result=true,calls=0
left=false,result=true,calls=1
```

实际编译与运行日志为 `accepted-{original,jadx,jarde}-{javac,run}.log` / `.txt`；实现后的 [JSON 报告](jarde-after-report.json)包含字段写入及逐指令来源。这里验证的是这个受限静态字段消费形状，不将一次成功运行扩展为任意 `||`、异常保护区或多消费者 Phi 的证明。
