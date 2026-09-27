## Why

[CF-05 固定对照](../../evidence/java-syntax-2026-09-27/cf05-numeric-condition/report.md)显示，原 class 与固定 JADX 的完整 `ConversionCases` Java 8 源码运行 20 行一致，Jarde 因四处 byte/short 条件调用拒绝而无法重编。简单 int/long/byte/float/double 条件返回已正确；缺口集中在窄类型条件值的来源证明与调用目标类型。

## What Changes

- 对 `byte`/`short` 形参的条件值，只有两臂分别能证明为该类型字段/值或范围内 int 常量，且消费者调用目标准确时，才生成保留同一重载选择的 Java 条件表达式。
- 在现有条件值和调用参数适配路径内处理“字段一臂 + 常量一臂”及“两常量臂”，保留求值顺序、物理来源、预算和原子拒绝。
- 以完整 `ConversionCases` 三方 Java 8 重编、验证运行及越界/错签名/额外使用反例验收；`ConversionBasic` 的五种简单数值返回保持通过。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证窄数值条件值作为准确重载调用实参时的源码恢复。

## Impact

主要影响 `crates/jarde-java/src/build.rs` 的既有条件值构建、字面量窄化和调用实参适配，以及必要时 `ast.rs` 的条件表达式类型计算。JADX 的 `ModVisitor`、`FixTypesVisitor` 与 `InsnGen` 可参考类型修正入口；不能照搬仅凭目标形参或 int 栈形状强制 cast 的结论。本任务不处理泛化 boolean-to-number DEX cast、任意数值变换、CF-04 任意 producer、`final` 字段初始化或其它声明问题。
