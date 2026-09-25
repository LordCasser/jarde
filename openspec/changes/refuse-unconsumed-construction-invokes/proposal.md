## Why

一份 JVM 可验证的 Java 8 class 在 `new Target; dup; Side.effect()V; iconst_1; Target.<init>(I)V` 中先初始化 `Target`，再运行独立调用。当前 Jarde 与 JADX 1.5.6 都输出 `Side.effect(); return new Target(1);`，重编后把原运行结果 `CST` 改为 `SCT`。Jarde 还把错误移动的构造站点报告为 `new@1 presented=true`，因此需要收紧既有证明，而不是扩展可恢复语法。

## What Changes

- 普通 `new@1` 只允许构造器实参表达式中的调用；没有栈结果、无法成为实参值的独立调用必须使该站点拒绝，保留原始字节码来源和准确的拒绝证据。
- 锁定原 class、JADX、Jarde 的源码及重编执行差异，并加入有独立调用的负例和真正作为构造器实参的调用正例。
- 不改变成员内部类的语义证明、构造器 AST 或类级声明装配；这些由各自的变更处理。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 普通构造恢复不得移动 `new` 与 `<init>` 之间不属于构造实参的调用效果。

## Impact

影响 `crates/jarde-java/src/init.rs` 的 `new@1` 站点验证、对应报告与回归测试。沿用现有 SSA、`StatementFree` 拒绝和来源保留路径；无需新 IR、AST、遍历阶段或对外 API。
