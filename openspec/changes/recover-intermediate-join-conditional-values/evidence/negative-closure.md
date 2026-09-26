# 两层 join 负例闭包（2026-09-26）

本记录区分两种证据：**JVM 级**是 major version 52 的 class 经 `java -Xverify:all` 执行，并由真实 reader/CFG/SSA/Region/Builder 完整走通；**proof-unit 级**只改变一条边、use 或 Region owner 事实，调用生产证明门槛，不宣称该注入状态来自 verifier 有效的 class。冻结正例的 Phi18 唯一 use 在 BCI 19，Phi26 唯一 use 在 BCI 26；外真臂从内 join 18 经 `iconst_3; iadd; goto` 进入外 join 26，两个 Phi 都为 `int`。

| 反例 | 证据等级及事实 | 拒绝位置和完整性 |
| --- | --- | --- |
| 桥接独立效果 | JVM 级：`IntermediateJoinNegative.independentEffect(I)I` 在 BCI 18 调用 `side()`，随后 21 `iadd`、22 `goto`；class 经 verifier 与运行检验 | 桥接仅允许已证明的 `Push(int) → iadd → Transfer`，组合证明拒绝；完整 `@bytecode` 引用，18/21/22/28 均有来源 |
| 可抛错运算 | JVM 级：`throwingBridge(I)I` 的 BCI 19 是 `idiv` | 桥接运算门槛拒绝；18/19/20/26 均有来源 |
| 未知运算 | JVM 级：基于冻结 class 将 BCI 19 的 `iadd` 换成 `ixor`，其余两层 join 与两个值类型不变；新 class major version 52、含 StackMapTable、六条 runner 路径经 verifier；SHA-256 `8befd5c4763826d43b7b790983e7a32927993cbefa0657530654c7136bb46fe6` | `prove_intermediate_join_value` 返回 `Refused`；完整类结果不含部分三元式，0/4/9/15/18/19/20/23/26 均有来源 |
| 外部入口 | JVM 级：`externalEntry(I)I` 有 9→28，另有 19→28 与 25→28；28 不能当作内层独占 join | Region 在 `jre_region_arms_do_not_meet` 引用整个候选；28–30 与相关前驱的来源保留 |
| 异常边 | JVM 级：`caughtBridge(I)I` 的异常表有 `[0,28)→29`，指向 `ArithmeticException` handler | Region 以 `jre_region_arms_do_not_meet` 引用分支，handler 与桥接指令均有来源 |
| Call、异常、Return 入边 | proof-unit 级：对同一个续走块的两条预期入边 9、15，分别把 15 的 `CanonicalEdgeKind` 改为 `Call`、`Exception`、`Return` | `exact_normal_predecessors` 在 Region 续走的边闭包处逐一拒绝；未把旧 `jsr` fixture 假称为这个双 join 的 verifier 有效 Call 反例 |
| 重复/外部前驱 | proof-unit 级：把预期 9、15 的实际入边改为 9、9，或额外加入 23；canonical 发布边排序但该层不去重 | Region 续走要求边数、全 Normal、**不同**预期前驱集合与实际集合一致，均拒绝；JVM 级外部入口另验证整段引用 |
| owner 重叠/visited 差集 | proof-unit 级：在冻结 Region 假臂再放入桥接块 18，使该物理块出现两次；另以尾段 `[18,18]`、visited `{18}` 注入重复 claim，并验证少 claim/多 claim | 组合证明因 owner 不唯一 `Refused`；Region 续走的 claim 闭包在发布前拒绝重复或不相等的 visited 差集 |
| 额外 SSA use | proof-unit 级：由冻结 class 读出真实 Phi18 的 `use@19` 和 Phi26 的 `use@26`，把后者作为内层 Phi 的第二条 use 事实交给生产唯一消费者门槛 | `exactly_one_use` 精确返回 `PhiUseCount`。这个单元检验只证明门槛；它不声称修改后的 use 列表是完整有效的 SSA 表。JVM `dup` 会产生新 SSA identity，不能替代该负例 |
| 预算/取消 | JVM 级冻结样本，恢复预算设为 `ir_items=1` 或提前取消 | 完整恢复停止，正文和来源映射均为空；不发布部分组合 |

Region 续走还逐块核对同一 canonical path、原 `Frame` 的 scope、原外臂边界、唯一后继、`tail_next=None` 以及尾段首块等于未认领内 join。正例覆盖外真臂 BCI 18–20 唯一归属；外部入口、handler、边种类、重复前驱、owner 与停止负例分别压住闭包的关键拒绝条件。Builder 只在子 Phi、桥接步骤、外 Phi 输入与最终返回全部核对后发布根折叠；既有完整类六路径运行对照见 `verification-root.md`。

未知运算 fixture 的生成与验证命令：

```text
javac --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED -d /tmp/jarde-intermediate-bridge-generator tests/fixtures/p3-intermediate-join/GenerateBridgeControls.java
java --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED -cp /tmp/jarde-intermediate-bridge-generator GenerateBridgeControls openspec/evidence/java-syntax-2026-09-26/conditional-intermediate-join/ConditionalIntermediateJoin.class /tmp/jarde-intermediate-bridge-rebuilt/ConditionalIntermediateJoin.class
cmp tests/fixtures/p3-intermediate-join/unknown-operation/ConditionalIntermediateJoin.class /tmp/jarde-intermediate-bridge-rebuilt/ConditionalIntermediateJoin.class
javac --release 8 -Xlint:-options -cp tests/fixtures/p3-intermediate-join/unknown-operation -d /tmp/jarde-intermediate-bridge-runner openspec/evidence/java-syntax-2026-09-26/conditional-intermediate-join/Runner.java
java -Xverify:all -cp tests/fixtures/p3-intermediate-join/unknown-operation:/tmp/jarde-intermediate-bridge-runner Runner
```

最后一条依次输出 `30:f3,`、`9:f1,`、`23:f2,` 与三个只执行被选调用的 `IllegalStateException` 路径；`cmp` 完全相同。`IntermediateJoinNegative.class` 的 Java 8 编译、verifier 和来源核对见 `evidence/region-proof-validation.md`。
