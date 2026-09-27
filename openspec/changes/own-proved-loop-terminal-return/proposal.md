## Why

[CF-07 固定对照](../../evidence/java-syntax-2026-09-27/cf07-basic-loops/report.md)中，循环体内由唯一条件分支到达的 `ireturn` 叶没有回边，故不在自然循环集合；Jarde 将该块留给未覆盖 quote，随后正确地拒绝跨 quote 的循环局部变量，完整类无法重编。原 class 与固定 JADX 的完整 Java 8 类源码均能运行且结果一致。

## What Changes

- 在现有循环区域证明中，让由循环内唯一正常入口到达、独立终止方法的返回叶归循环体持有，不把它误当成循环外共同出口。
- 保留返回值的定义使用、循环更新、正常退出、物理来源和预算停止；多入口、异常/子例程边、共享叶或无法闭合的值继续拒绝。
- 用固定 CF-07 夹具的完整原/JADX/Jarde Java 8 重编与验证运行验收，并保留普通循环首片的行为。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：证明循环内唯一终止返回叶的所有权，并恢复包含该提前返回的循环源码。

## Impact

主要影响 `crates/jarde-java/src/region.rs` 的循环 `Frame` 可达范围与终止叶归属。`build.rs` 应继续使用已有局部变量作用域、返回值、来源和预算机制，不放宽跨 quote 拒绝。JADX `LoopRegionMaker`、`LoopRegionVisitor` 提供测试与算法参考，但不会直接移植其结构猜测。本任务不处理 CF-06 条件内赋值、一般多入口循环、不可约循环、异常退出或 `for` 文本美化。
