## Context

见 [proposal.md](proposal.md)、[行为合同](specs/java8-recovery/spec.md)和[三方证据](../../evidence/java-syntax-2026-09-26/conditional-intermediate-join/analysis.md)。原 class 的 `choose(I)I` 在 BCI 0 作外层测试，在 BCI 4 作内层测试，内层调用 BCI 9/15 汇合到 BCI 18，`iconst_3; iadd; goto` 位于 BCI 18–20；外层假臂 BCI 23 与桥接结果在 BCI 26 汇合并返回。原 class 与 JADX 1.5.6 重编类的六条正常/异常路径一致，Jarde 完整类缺返回。

[独立 Region trace](../../evidence/java-syntax-2026-09-26/conditional-intermediate-join/region-trace.txt)进一步定位首因：`normal_flow` 正确保留普通分支/goto，外层 `If` 的 post-dominator 是 BCI 26，内层的是 BCI 18；但外臂构造只收下内层 `region_at` 返回的 `If`，没有在同一臂继续消费 `next=18`，使 BCI 18 最后作为 uncovered 引用。即使修复这一步，现有 `prove_conditional_tree_value` 仍要求所有内部 `If` 共享根 join、叶子直接供给同一个 Phi，不包含 `Phi18 → iadd19 → Phi26`。

## Goals / Non-Goals

**Goals:** 对有界前向的两层条件、一个内层 join 和一段直线值桥接，完整恢复结构与值；保持真/假边、求值顺序、异常传播、类型、来源、唯一物理归属及有界停止。先用冻结 `iadd` 形状闭环，再只接纳现有表达式构建可证明的同类无独立效果运算。

**Non-Goals:** 不改 class 解析、Java 8 方言/descriptor 校验、运行时选择或 JVM verifier；不改 canonical CFG/SSA，也不新建公开 Region/AST 或通用 Phi 求解器。循环、switch、嵌套 try/handler、回边、多层任意 join DAG、可能抛错或有独立效果的桥接指令不在本切片。不能证明时完整引用，不为得到可编译文本放宽边界。

## Decisions

1. **先在原臂内续走内层前向 join。** 外 `If` 的一臂若返回内部 `If` 与未认领的 `next`，且 `next` 不等于外臂边界，则在同一个 `Frame` 中续走到原边界，形成 `Sequence(If(join=18), Straight(18→20))`；不另起顶层 owner，也不更改 `normal_flow`。只接受单一前向正常出口、同一 canonical path、无外部前驱/重叠 owner 的闭包。遍历前暂存 visited，逐段检查 scope、边种类和物理块集合；不能完整抵达边界时恢复状态并引用整个候选。备选是把 BCI 18 作为顶层 Region，虽然能消除 uncovered，却会把外层表达式真臂拆开，并可能重复发射。
2. **复用两臂证明的事实，而不假定现有函数能直接接收外臂。** 内层 `If` 的两臂仍为 `Straight`，可沿既有证明核对 BCI 4 的真实边、Phi18 的 slot/输入与唯一 BCI19 消费。外层真臂已经是 `Sequence(If, Straight)`，现有 `prove_conditional_value` 的形状门槛会拒绝；组合证书须复用它的边、Phi、输入与唯一消费不变量，专门核对外层 BCI 0、BCI 23 假臂、Phi26 的两个真实输入及唯一 BCI26 返回，不能假装对整棵外臂直接调用旧证明。证书只连接这两个 join：记录桥接的有序指令/SSA 读写、从子 Phi 到桥值的单 use、桥值到父 Phi 的单 use、完整物理 BCI/块与合法边集合。桥接只允许现有 `render_value` 能写、不会独立产生效果或改变异常边界的直线运算；`iadd` 加常量是冻结正例。逐条扫描参与值的所有 uses、块指令和 incident edges，拒绝额外读者、独立调用/存储、异常/Call 边、外部入口和第二个变化的栈 Phi。备选是放宽共同 join 树 proof；它没有记录中间 Phi 的唯一消费者和运算位置，不能证明这个形状。
3. **局部构建整棵表达式，成功后才发布。** `prepare_conditional_region` 在识别这种 `Sequence` 真臂时，先暂存子条件表达式，再以该子 Phi 为桥接运算的确切操作数构造桥值，最后构造外条件表达式并检查返回位置的类型/转换与来源。复用 `conditional_dependencies`、`render_value`、`ExprKind::Conditional` 和现有 AST 发射；子 Phi 是显式组合边界，不能只依赖会在 Phi 处停止的普通递归。测试、叶子调用、`iconst_3/iadd/goto`、两个 join 和返回的 origin 一起校验。全部完成后才写入根 Phi 的 `conditional_values` 与根分支折叠计划；失败恢复暂存状态，并把候选闭包整段引用，不能让内层局部折叠独自可见。备选是先发布内层再尝试外层，目前恰会留下空外臂与无生产者的返回。
4. **把 JADX 当作算法对照而非证明来源。** 本地 `TernaryMod` 在两臂结果都唯一进入同一 Phi 时，将指令包成三元值并从块中移除，之后的区域遍历能把结果交给后续算术；这说明“先组合值、再继续求值”的次序有效。它依赖可变 Insn/Phi、line hints 和 visitor 迭代，不能直接移植到 Jarde 的不可变 SSA、预算、来源与原子回退约束。本项不用新增依赖或复制 JADX 代码。

## Risks / Trade-offs

- [臂内续走把外层 join 或另一 owner 吸入子树] → 以原 `Frame` 边界、路径、前驱和 visited 差集核对闭包，失败整体回滚。
- [内层调用因局部折叠与外层值再次发射] → 子表达式只在暂存组合中存在，根发布前核对每个 producer、Phi 与物理 BCI 恰有一个呈现 owner。
- [运算被移到错误的求值位置或括号改变 AST] → 检查桥接前后无独立效果/异常边，使用既有值构建和打印分组；以六条 trace/异常路径的完整类对照验收。
- [停止或类型失败留下半个条件值] → 证明、构建、来源均按有界预算/取消逐段检查；停止保持既有状态，拒绝则完整引用候选。
- [旧两臂、同 join 嵌套或相邻 Region 回归] → 分别运行冻结正例及外部入口、多消费者、独立效果、异常/Call 边负例。

## Migration Plan

无公开格式或依赖迁移。可回退到现有 Region/条件值保守引用；冻结样本、物理来源和负例仍可用于验证回退边界。
