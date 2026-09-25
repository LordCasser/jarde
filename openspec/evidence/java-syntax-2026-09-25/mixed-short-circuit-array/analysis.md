# 混合短路值写入 `boolean[]` 元素

冻结的 [Java 8 源码](MixedArrayValue.java)、[class](MixedArrayValue.class) 和 [Runner](Runner.java) 覆盖 `array(nullArray)[index(pos)] = (a && b()) || c()`。`javac --release 8 -g:none` 生成的 class SHA-256 为 `f6856cc228d8efebe60aecfa2f903290a10e9d3f14ed4197ea3fa0b8a30c9351`；无 LVT。Runner 对 `a/bValue/cValue` 八组、正常数组/null 数组/越界下标三种目标各跑一次，共 24 行。原始 [JVM 轨迹](original-run.txt)用 `java -Xverify:all` 得到。

[`javap` 跟踪](javap.txt)中，BCI 1 `array:(Z)[Z`、BCI 5 `index:(I)I` 先求出数组与下标；BCI 9/15/21 为短路测试，24/28 生产 1/0，BCI 29 `bastore` 消费 stack depth 2 的值 Phi，同时读 depth 0/1 的数组与下标。BCI 30 返回。正常数组写入结果符合 `(a && b) || c`；null 与越界路径都先执行数组、下标及适用的 `b()/c()`，然后分别在 `bastore` 抛出 `NullPointerException` 和 `ArrayIndexOutOfBoundsException`。三种目标的数组/下标调用数都为 1；`b()` 仅在 `a` 真时运行，`c()` 在 `a` 假或 `b()` 假时运行。这是 RHS 求值和错误时点的验收真值，不能只比较最终数组值。

安装的 JADX 1.5.6 输出[完整类](jadx-MixedArrayValue.java)，其 `one` 为 `array(z2)[index(i)] = (z && b()) || c();`。只在临时重编副本去掉 JADX 给无包 class 加的 `package defpackage;`，Java 8 编译和 JVM 验证均通过，[24 行结果](jadx-run.txt)与原始轨迹逐行一致。它展示了可用的源码呈现，但不是缺口修复的极性或求值顺序证明。对照本地 checkout `2fb1b163`：`TypeUpdate.arrayPutListener` 从数组组件向写入值传播类型；`InsnGen` 的 `APUT` 按数组、下标、值顺序生成 `a[i] = v`。Jarde 已有数组组件证明和 `IndexAssign` 发射，不必移植 JADX 的通用类型更新机制。

从网关变更验收后、局部值变更开工前的 CLI 导出[全证据](jarde-report.json)和[完整类](jarde-MixedArrayValue.java)。`one(ZZI)V` 是 `fallback/mixed/explanation_only`，整个方法以 15 个已解码 BCI 引用；首个可见拒绝为 `jre_region_ownership_overlap`，指出 BCI 24 被多处 Region 认领。因此不能声称运行已经到达数组消费者并因它失败。[Jarde 完整类](jarde-MixedArrayValue.java)本身可用 Java 8 重编，但 `one` 是空壳，[24 行执行](jarde-run.txt)全与原 class 不同：首行原为 `0:0:OK:false:1:1:0:1`，Jarde 为 `0:0:OK:false:0:0:0:0`；null/越界路径也不抛原异常。可编译的引用文本不算语义恢复。局部值变更完成后须从最新 CLI 复测，不能把此时点快照当成之后源码状态。

代码审计给出与上述**观测拒绝不同**的下游边界：`Region::short_circuit_value` 的 consumer-anchor 名单目前只承认静态字段写入、`ireturn`、调用和 `Store`，不含 `ArrayStore`；即使图遍历此前均成功，此处也必定返回 `None`，普通 `if` 构造随后可能重访共享 producer。仅靠源码阅读尚未排除更早的拒绝，所以 BCI 24 overlap 的首因仍待 trace 定案。`prove_short_circuit_value` 的消费者也仅匹配静态 `putstatic Z`、`ireturn Z`、单参数静态调用（正在实施的局部 `istore` 另案），且要求消费者 `reads()` 仅有值 Phi；`bastore` 同时有数组/下标/值三个读取。普通 `array_write` 已用 `array_element` 证明 `[Z`，按 `bastore` 选择 Boolean 表示并发射现有 `IndexAssign`。应先定位这份闭合图的实际首个失败点，再决定能否在原私有图中新增严格数组消费者：Phi 唯一直接 use 必须是值操作数，数组/下标须有独立的 SSA、类型、唯一求值和来源证明；`bastore` 同时用于 `[B` 与 `[Z`，opcode 单独不能证明 Boolean。候选整体提交前需核对测试/producer/consumer 与先求出的数组/下标无额外 owner、异常边或隐藏效果。无需预设公开 AST/IR 或全局类型推断 pass。

这组正例不同于实例 `putfield`：数组多一个下标操作数，还有 bounds fault；两者都要求源码赋值在 RHS 之后触发目标检查。分别建 change 与拒绝控制，再比较可共享的局部证明，不把一个样例的通过推成另一类的准入。

局部 Boolean Store 实现后，根代理重新构建 CLI 的[报告](jarde-after-local-root-report.json)和[完整类](jarde-after-local-root-MixedArrayValue.java)显示 `one` 仍为 `fallback/mixed/explanation_only`，在 BCI 24 报相同 `jre_region_ownership_overlap`，15 个已解码起点都在 whole-method quote 中。局部消费者变化未偶然放开数组写入；24 路径的原 class 轨迹仍是未来验收目标。

实例字段修复后的[再次独立重放和只读 Region/SSA 跟踪](region-trace.md)现已消除上述“是否更早失败”的不确定性：外层 BCI 9 的短路候选到达 BCI 29 `bastore` 所在 SSA 块后，因 consumer-anchor 未列 `ArrayStore` 而退出；通用 `If` 树随后在两个位置认领 BCI 24。当前 Jarde 的 24 行仍与原类不等价。后续需保持 owner validator，先扩闭合候选，再单独证明 `[Z` 的三操作数写入。
