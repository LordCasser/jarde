## Why

[八路径完整类对照](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-argument/analysis.md)显示，`sink((a && b()) || c())` 的三测试、两个常量 producer 与唯一 Phi 与已恢复的混合字段值同型，Phi 由 `invokestatic sink:(Z)V` 消费。原 class/JADX 的字段值及 `b/c/sink` 调用次数八行相同；Jarde 当前引用整段虽然能重编，却在全部八条路径漏掉 `sink()`。可编译性不能充当调用恢复证明。

## What Changes

- 在既有 `ShortCircuitValue` 闭合图和唯一 Phi 证明上，限定开放一个静态、无接收者、恰好一个 `Z` 参数且返回 `V` 的真实 Methodref 调用消费者；不复制布尔图算法。
- 按真实调用目标描述符及 SSA 参数绑定，复用现有 invocation argument 和 `Call`/`Expr` AST 发射惰性布尔值并只调用一次目标。
- 对额外参数/接收者、Methodref 不明、求值顺序无法证明、第二消费者、异常边/外部入口、独立效果或预算停止继续完整引用。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对可证明的闭合短路值作为单个布尔静态调用实参，保留值及所有调用次数和顺序；证明不足时原子拒绝。

## Impact

只扩展 `jarde-java::region` 的现有 consumer anchor、`jarde-java::build` 的 SSA/真实 Methodref 准入与现有调用表达式发射。依赖[字段图](../recover-mixed-short-circuit-field-values/verification.md)和[直接返回](../recover-short-circuit-return-values/tasks.md)的独立验收，不增加公开 AST/IR/pass；字段报告误标与局部作用域债务另列。
