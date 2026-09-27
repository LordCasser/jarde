## Why

固定 JADX `TestXor` 将布尔 `x ^ true`、`x ^ false` 分别投影为 `!x` 和 `x`。EM-22 冻结 Java 8 全类对照表明，Jarde 已正确重编运行，却仍保留这两种机械异或写法；参数及带副作用调用均可复现。已有布尔类型证明与 `ExprKind::Not` 足以闭合这个源码质量差距。

## What Changes

- 在既有 `Operation::Bitwise` 表达式构造处，仅对准确 `ixor`、独立证明为 boolean 的左值和右侧 `0`/`1` 常量分别输出已有 `Not` 或原左表达式。
- 保留左表达式原执行位置及次数、XOR BCI 来源、返回/参数类型证明；其它整型/长整型、非常量或类型不明的 XOR 仍保持原路径。
- 以固定原/JADX/Jarde Java 8 完整类重编、验证运行和负例验收，尤其核对有副作用调用恰好执行一次。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证明的布尔值与字面量 `true`/`false` 异或可恢复为直接否定/原表达式。

## Impact

只改 `jarde-java` 既有位运算表达式选择与测试，不改 JVM 解析、AST、公开 CLI/schema 或依赖。冻结证据见 [EM-22 审计](../../evidence/java-syntax-2026-09-27/em22-arithmetic/report.md)。
