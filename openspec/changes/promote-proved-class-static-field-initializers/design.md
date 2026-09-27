## Context

[EM-06 对照](../../evidence/java-syntax-2026-09-27/em06-field-init/report.md)中，普通类五个静态字段的 `<clinit>` 与源码声明顺序一一对应，Jarde 已恢复有序 `static` 赋值并收集同轮 `ClassInitializerStep`、RHS AST、字段读取身份及准确 BCI。现有接口字段初值路径在 `src/facade.rs` 已验证同类候选并由 `src/class_source.rs` 原子写字段声明、跳过 `<clinit>`，但入口按 `ACC_INTERFACE` 限制。

## Goals / Non-Goals

**Goals:** 一个普通类的完整、单路径、无异常的静态运行时字段写入组，按原写入顺序输出声明初值，保留运行、副作用和来源身份；有实例字段与构造器时保持它们不变。

**Non-Goals:** 实例字段初值移动、构造器合并、部分静态组、ConstantValue 混合、前向读、异常块、继承字段与数组特殊语法。这些不能从本首片推定。

## Decisions

1. **复用同轮事实，不增提取机制。** 将接口证明中的字段表、同轮候选、RHS 字段读取、表达式效果及预算检查整理为可按静态字段子集使用的证明。普通类必须是唯一完整选定定义，且 `<clinit>()V` 唯一、结构和正文完整。实例字段只维持其原声明与方法正文，不进入静态组的覆盖计数。

2. **整组准入。** 普通类首片只接受没有 `ConstantValue` 的静态运行时字段，每个字段在 `<clinit>` 恰好写一次，候选序列只含有序字段写和尾 `return`；无 handler、未知操作、重复/遗漏写、未证 RHS 读或前向同类静态字段读即拒绝。动态调用只允许作为准确 RHS 树中已恢复的效果，原 BCI 次序须仍由声明求值实现。拒绝时保留原 `static` 块，不先投影前缀字段。

3. **一次性提交最终文本。** 现有 field fragment 发射先在预算内全部完成，再按证明的写顺序附到对应声明；writer 仅在整组成功时省去原 `<clinit>`。字段与 `<clinit>` 的物理报告、BCI 和 source map 不冒充彼此；产出的根源码是派生视图。任何停止都保留未投影的已有结果并按现有执行状态报告。

4. **固定正反验收。** [replay.py](../../evidence/java-syntax-2026-09-27/em06-field-init/replay.py) 的原/JADX/Jarde 完整类源重编与 `-Xverify:all` 输出应一致，Jarde 的 `trace/a/b/c/result` 初值顺序匹配固定 bytecode；`field = initField()` 仍在构造器。定向测试覆盖重复写、额外调用/写入、前向读、ConstantValue、异常边、截断字段表、预算/取消，接口既有字段投影不回归。

## Risks / Trade-offs

- 静态依赖读取可能观察默认值，源码前向引用还有合法性差异：首片直接拒绝前向读。
- 搬动 RHS 可能移动副作用：只允许整组按原顺序一一对应的候选，不做通用可重排分析。
- 类字段和接口字段 flags 不同：准入分别校验，表达式/来源/原子投影逻辑可共享，不通过放宽接口规则顺带扩大范围。
