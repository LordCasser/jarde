## Why

[CF-03 固定对照](../../evidence/java-syntax-2026-09-27/cf03-branches/report.md)中，`ChainOnly` 的原 class、JADX、Jarde 完整 Java 8 类源码重编和运行一致，但固定 `TestElseIf` 期望的三个 `else if` 在 Jarde 被写成三层 `else { if (...) ... }`。现有 AST 已有准确的单语句嵌套 else；差距在文本呈现，不需改 CFG 或发明新语法节点。

## What Changes

- 仅当 `StmtKind::If` 的 `else_body` 恰为一个直接子 `StmtKind::If` 时，将原有两层文本拼成 `} else if (...) {`，可继续链式输出；其它 else 块保持既有形式。
- 保留每个子 `If` 的物理来源、source-map span、语句计数和预算/停止行为，不把表达式或副作用移到新区域。
- 用隔离完整类三方重编/验证运行及条件调用计数证明文本变化不改行为，另测含多句 else、fallback/guard 边界保持不折叠。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证嵌套 else-if 链的 Java 源码拼写与来源保持。

## Impact

主要是现有 `crates/jarde-java/src/emit.rs` 的 `StmtKind::If` 分支和目标测试；`ast.rs`、Region/SSA 与 class-source 门不变。该变更不处理同一 CF-03 对照中的 BCI 24 共享尾归属或完整 `BranchShapes` 编译失败；后者须在 CF-02 的区域修复集成后重验。JADX 的 `IfRegionMaker` 可作输出参照，不能照搬其区域拥有权假设。
