## Context

见[冻结证据](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-instance-field/analysis.md)。样例的 `one(ZZ)V` 在求值短路图期间，将 `target(nullReceiver)` 的结果保留在操作数栈上。BCI 25 的 `putfield ...:Z` 从栈深度 0 读取接收者，从栈深度 1 读取汇合后的 1/0 值。该 Phi 只有一个直接值消费者，但这条指令还会额外读取接收者。

稳定版与当前 Jarde 报告均在规范 BCI 20 因 `jre_region_ownership_overlap` 停止。[Region/SSA 跟踪](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-instance-field/region-trace.md)已确定因果顺序：`short_circuit_value` 已收集到完整闭合图及 BCI 25 的两个 1/0 producer，却因消费者锚点只接受 `putstatic` 等操作而在 `putfield` 返回 `None`；随后通用 `if` 构造真实地两次认领 BCI 20。所有权校验正确地拒绝了该树。私有字段消费者匹配器目前还只接受 `putstatic Z` 和单个 Phi 栈读取；实例指令须额外证明接收者栈读取。解决位置是专用候选的锚点与消费者证明，而非 Region 所有权例外。

## Goals / Non-Goals

**Goals:** 定义最小且安全的实例字段消费者边界，保留接收者与 RHS 的求值顺序，并通过永久测试矩阵观察 null/non-null 两种行为。

**Non-Goals:** 本规划任务不实现恢复逻辑。没有独立证明时不得放宽 Region 所有权；不得新增公开 AST/IR/pass；不接纳复合字段写入、字段读取、其它描述符、数组、局部变量或调用消费者。

## Decisions

1. **保持 Region 所有权不变量。** 已证明图内 Canonical 正常边完全闭合、BCI 25 具有唯一消费块，且不存在异常边或外部前驱。当前通用 `if` 树中的 BCI 20 重叠是真实表述冲突，不能放宽校验。仅当专用短路图通过既有入口、前驱、边、SSA 和消费者检查后，才使其先于通用 `if` 获得一次所有权；任何检查失败仍由完整候选回退。
2. **绑定 `putfield` 的两个操作数。** 要求已证明消费者 BCI 上的字段指令是 `putfield`，描述符为 `Z`，且字段 owner/name/descriptor 与现有字段计划所认领的一致。将 `shape.receiver` 匹配到栈深度 0 的接收者 SSA 值，将 `shape.value` 匹配到栈深度 1 的唯一短路 Phi。即便 `putfield` 还读取接收者，也必须保留 Phi 仅有一个直接值使用的证明，该使用就是本次写入的值操作数。
3. **证明接收者所有权及单次求值。** 接收者 SSA 值在该 `FieldAssign` 中必须只有一个表达式所有者，并与 frames/字段计划证明的精确声明类类型一致，还必须可由现有值渲染器呈现。`target(nullReceiver)` 这样的接收者调用必须且只能嵌入 `FieldAssign.receiver` 一次；不得同时作为前置语句发射，也不得因另一 SSA 使用而重复发射。使用/所有权事实不完整时必须拒绝。默认不得将接收者移到更早的独立语句。
4. **保留赋值的异常顺序。** 发射现有 `receiver.result = <conditional>;` 形状。接收者表达式先于 RHS 求值且只求值一次。RHS 保持惰性，并在 `putfield` 最终因接收者为 null 而出错前执行；8 条 null 接收者轨迹中，b/c 次数必须与对应非空路径相同。只有等价求值顺序得到证明后，才可采用其它求值策略。
5. **原子提交或引用。** 只有图、接收者和字段 claim 全部成功后，才构建字段语句。保留接收者调用、所有测试/producer 和 `putfield` 的来源；预算扣减、取消或所有权拒绝时，必须保留整个候选原始引用，不得发射部分赋值。
6. **明确拒绝边界。** 遇到第二个 Phi 使用、接收者有第二次使用/所有者、字段非 `Z`、字段身份不匹配、接收者类型未解析、额外所有者/入口/边/效果、复合字段更新，或尚未单独解决的重叠时均拒绝。报告真实的先行拒绝原因；不得把所有权重叠改写成 `putfield` 类型拒绝。

## Risks / Trade-offs

- `[风险]` 接收者调用被移动或重复 → 要求 SSA 表达式只有一个所有者，并验证每条路径的 `receiverCalls` 都恰为 1。
- `[风险]` null 检查早于有副作用的 RHS → 对照原始 null 路径；NPE 前 b/c 调用次数必须全部一致。
- `[风险]` Phi-only 使用证明误拒所有实例写入 → 明确区分 Phi 的值使用次数与 `putfield` 的完整双操作数读取集合。
- `[风险]` 局部所有权修复掩盖真实 Region 重叠 → 将重叠调查作为阻断性设计决策，并保留未解决重叠时的拒绝控制。
