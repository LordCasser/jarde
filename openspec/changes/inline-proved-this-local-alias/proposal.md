## Why

固定 JADX 的 `TestInlineThis` 和 `TestInlineThis2` 能把仅保存 `this` 的局部变量消去，Jarde 在 Java 8 完整源码中仍输出 `local1 = this` 及其所有读取。运行语义已正确，但这种机械别名降低源码可读性。`TestDontInlineThis` 同时要求保留可能变为新对象的局部变量；按文本或槽号无条件消除会写错接收者和返回对象。

## What Changes

- 在现有同次方法 builder 中，对唯一、纯粹从实例接收者槽 0 写入、且全部读取仍指向该 SSA 值的非参数局部，证明它可以直接以 `this` 呈现。
- 原子省略该局部的声明/赋值，并把已证明的调用接收者、字段接收者及 `Objects.isNull` 实参读取写为 `this`；其它结构和调用保持原来的求值位置。
- 多来源、复赋值、逃逸/不支持的读取、异常/区域证据不完整以及预算停止保留现有局部或拒绝，不发布一半消除的源码。
- 以冻结的原/JADX/Jarde Java 8 全类源码重编及验证运行和定向负例验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对已证明只指向当前实例的局部别名，输出直接 `this` 接收者和参数表达式，同时保留真正参与值流动的局部。

## Impact

只影响 `jarde-java` 既有局部身份/声明计划、表达式构造和定向测试；不改变 JVM IR、AST 枚举、公开 CLI/schema 或依赖。[EM-21 冻结对照](../../evidence/java-syntax-2026-09-27/em21-this-alias/report.md)记录基线与边界。
