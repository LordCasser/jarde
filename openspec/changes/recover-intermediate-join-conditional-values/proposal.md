## Why

Java 8 的 `a > 0 ? ((a > 1 ? f1() : f2()) + 3) : f3()` 有两个不同的栈汇合点：内层值在 BCI 18 汇合，经过 BCI 19 的加法后才进入 BCI 26 的外层汇合。冻结的[三方样本](../../evidence/java-syntax-2026-09-26/conditional-intermediate-join/analysis.md)中，原 class 和 JADX 完整类的六条运行路径一致，而 Jarde 遗留未覆盖的 BCI 18，输出缺少返回；现有“所有叶子直达同一 Phi”的证明按定义不处理这类值。

## What Changes

- 在外层分支臂内，继续遍历内层 `If` 已证明的前向 join，直到该臂自己的边界，完整认领内层汇合后的直线尾段；只有闭合的正常边、单一物理 owner 和有效作用域才提交结构。
- 对内层 Phi、汇合后唯一值运算、外层 Phi 及最终消费者建立有界的私有组合证明，复用现有两臂证明与表达式 AST，成功后一次性发布整个表达式和来源。
- 用正常值、选中调用抛错与未选中调用不执行的六条路径做完整类 Java 8 重编对照；额外消费者、独立效果、异常/Call 边、外部入口、重叠 owner、预算/取消继续保守引用。

本项以前置的普通两臂条件值和同一 join 嵌套条件值恢复为基础。它只处理有界前向、单个中间 join 后接可证明直线值运算的形状；不把任意 CFG、循环、switch 或异常处理器改写为条件表达式。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 内层条件值先汇合、经直线运算再参与外层条件值汇合时，可在证明完整后恢复为 Java 表达式，并保留失败来源。

## Impact

主要在 `jarde-java` 的 Region 臂内续走、现有条件值证明/构建和来源归属，以及定向 Java 8 三方测试。无需公开 IR、AST、reader/SSA、CLI 接口或外部依赖变更；本地 JADX 只作为算法和行为对照，不成为运行依赖。
