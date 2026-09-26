## Why

普通 Java 8 嵌套条件值 `outer ? (inner ? a() : b()) : c()` 在同一栈汇合点形成三条值来源。现有恢复只证明单层双臂 Phi，面对这个已编译且可执行的形状，jarde 输出空分支与不可编译的 Phi 引用；原 class 和 JADX 结果在冻结样本上行为一致。需要沿现有区域与 SSA 边界扩展证明，而不是把任意多前驱 Phi 猜成三元表达式。

## What Changes

- 对由 `Region::If` 递归组成、叶子最终流向同一栈值消费者的有界条件树，证明每条叶子路径及其 Phi 输入的精确对应，再恢复嵌套 `?:` 值。
- 保留外层与内层测试顺序、仅选中臂的调用和异常、静态类型与实际消费目标；整树证明失败时保留既有保守回退和物理来源。
- 用原 class、JADX、jarde 三方的 Java 8 编译与执行对照验收正例，并以多入口、独立副作用、异常边界及类型不明等反例验证拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明的嵌套条件区域可作为一个 Java 条件值恢复，并保持求值语义与未证明形状的保守回退。

## Impact

涉及 Java 恢复层的 `Region::If`、SSA Phi 到值表达式的交接、已有条件表达式构建和来源归属，以及相邻回归测试。前置条件是已完成的单层条件值恢复；本项不新增通用 Phi 求解器、不改变 JVM 解析/验证、不扩展为一般控制流重构，也不宣称覆盖所有 Java 条件表达式类型组合。证据保存在 `../../evidence/java-syntax-2026-09-26/nested-conditional-value/`。
