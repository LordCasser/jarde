## Why

现有 `invokespecial InterfaceMethodref` 恢复只检查池项 owner 是当前类的直接接口，便写出 `I.super.m()`。冻结的 verifier-valid class 同时直接实现 `Parent` 和 `Child extends Parent`，却调用 `Parent` 的默认方法：JVM 可运行，Jarde 文本因 `Parent` 是冗余直接接口而不能用 Java 8 重编。即使其它直接接口无关，父类链已实现 `I` 时，patched JVM 调用仍可运行而 Java 8 拒绝 `I.super`。另一个只替换目标接口声明的 class 能通过验证、在调用处抛 `AbstractMethodError`，Jarde 同样写出无法重编的 `Child.super`。JADX 在正常的接口显式选择场景还会把 `I.super` 错写成父类 `super`，所以验收需要同时守住已正确的分派和这些源码合法性边界。

## What Changes

- 在现有特殊调用选择上增加有界的类/接口继承及方法声明证明：其它直接接口和父类链均不能使目标接口冗余，目标限定名还须选到唯一可访问的 default 方法；抽象覆盖、歧义或证据不足时保留带物理 BCI 的恢复缺口。
- 对有独立源码表示的双接口正常调用继续保留各自的限定接口和执行语义；不为不合法源码形状改写成另一种调用。
- 沿用请求的解析环境、预算、取消和已存在的调用/回退机制，不新增 Java AST、通用继承图 pass 或生产依赖。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：收紧 `I.super` 的源码合法性门，并要求无法证明时的可追溯拒绝。

## Impact

以 [`preserve-special-call-dispatch`](../preserve-special-call-dispatch/) 已完成的特殊调用表达式和生产者回退为前提。涉及 `src/facade.rs` 的按需选定依赖读取、`crates/jarde-java/src/report.rs` 到 `build.rs` 的窄证明交接，以及定向测试。[冗余接口证据](../../evidence/java-syntax-2026-09-25/redundant-interface-super/analysis.md)与[抽象方法证据](../../evidence/java-syntax-2026-09-25/abstract-interface-super/analysis.md)限定了正反例。非目标包括任意特殊调用的 JVM 验证、method resolution 全图、非直接接口 `super`、未证明类/接口源码装配和旧变更的全库 Clippy 债务。
