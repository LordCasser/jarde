## Context

见[三方构造器对照](../../evidence/java-syntax-2026-09-25/constructor-conditional-delegation/analysis.md)。`ConstructorPairProbe(String,int)` 的第一个条件在 BCI 3 起分叉，BCI 12 汇合；第二个条件在 BCI 13 起分叉，BCI 22 执行 `this(String,String)`。`init::prologue` 和 `constructor_call` 已识别真实委托调用与参数描述符。`prove_conditional_value` 当前要求变化栈 Phi 的唯一读取为**同一 join 块的指令**，因此无法公布 BCI 12 的第一个条件值；BCI 22 第一个实参仍是不可渲染的栈 Entry/Phi。实际 SSA 中第一个 Phi ValueId 7 的 uses 为 BCI 22 的真实调用读取，以及同一 BCI 的同值 Phi ValueId 12 的内部读取；现有代码首先在 `PhiUseCount` 拒绝，接着 BCI 22 的报错只是失败的后续表现。基线 source map 还丢了已部分折叠第二条件图的八个 BCI，证明按单个分支逐项发布不足以保证调用整体失败时的来源完整性。

## Decision

1. 在既有 `prove_conditional_value` 与准备/渲染链上加入**有界传递证明**，从第一个变化栈 Phi 沿普通 CFG 边追踪保持同一值的栈槽和同值 Phi，直到同一次 `invoke*` 的确定实参槽。必须记录每跳的实际 SSA 输入/输出、栈深度、物理前驱和可达性；不得因类型相同、BCI 接近或目标相同推断身份。首个直接 join 消费的现有路径保持原语义。
2. 传递路径只准入不改变该值的栈操作与后续**已独立证明**的条件实参图；核对中途没有原本应在第一个实参求值之后、调用之前独立执行而会被挪到 Java 参数求值之前的可观察语句。最终调用按 descriptor 及原栈顺序消费各值。构造器的 `uninitializedThis` 身份与唯一 `this/super` 前导语义必须由既有 init 事实保留，不能把调用移出前导位置。
3. 只有沿链每项证明、两个条件表达式的 Java 类型和调用实参转换都成功后，才原子登记条件值及折叠的 Region 分支。传递用既有 `conditional_values`/来源结构表示；避免新公开 IR 或构造器专用 Region。若现有 SSA 将同值栈汇合表示为 Phi，其等价关系也必须逐输入证明并受预算约束。失败需引用全部相关图与调用 BCI，不能留下已折叠的第二分支而只引用 BCI 22，也不能生成空 `if` 后丢委托调用。
4. JADX 的 `ConstructorVisitor` 将 invoke 转为 constructor node，并在后续移动/内联指令；这里只借用实参求值顺序与 SSA 使用关系的思路。Jarde 以 class 的真实 JVM 栈、CFG/SSA 和 Java 8 重编/JVM 执行作为准入依据；不复制其移动启发式。单实参和双实参的原/JADX 输出当前都正确，不将这些样例描述成 JADX 错误。

## Risks and gates

- **重复或重排效果**：第一实参可能调用方法，第二实参可能抛异常。只允许能按 Java 从左到右参数求值重放的路径；新增可观察计数控制，对不闭合的独立语句拒绝。
- **虚假同值传递**：两臂写入不同但同类型的栈值、额外前驱、循环或异常边均可能使身份错配；核对每个 Phi 输入和 exact predecessor，保持拒绝。
- **半提交与停止**：候选和表达式先完整证明再修改发布表；每段图和 SSA 值遍历计费、轮询取消，低预算不得产出部分 Java。
- **重编约束**：无效委托调用会使 final 字段报初始化错误。完整类原/JADX/Jarde 均须 Java 8 重编并运行六条 `-Xverify:all` 路径；报告 status 与全部物理 BCI 同时核对。
