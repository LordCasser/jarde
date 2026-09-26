## Why

现有 `finally@1` 已能证明直线 `try { return value(); } finally { cleanup(); }`，但受保护正文含 `if`/`throw` 时仍整段引用。冻结的 Java 8 `ImplicitCleanup.run()` 就是此缺口：正常臂的值保存与正常清理、返回同处一个 canonical 块，平坦发射会丢分支；JADX 1.5.6 虽输出可编译源码，却在清理调用抛错时把清理执行两次，不能作为语义正确性的捷径。

## What Changes

- 在已有正常/异常清理副本、异常表半开范围、旧返回值和原异常身份的证书上，恢复可由现有 Region 结构证明的受保护分支正文；证书不足仍完整引用。
- 在恢复层内精确分割融合块的受保护指令与正常清理/返回，正常返回在其实际分支内呈现，保持 Java 局部作用域和求值顺序。
- 原子检查子 Region、物理块归属、异常边、可呈现正文和来源，再发布一个 `try/finally`；用原 class、JADX、Jarde 三方完整类和 `-Xverify:all` 对照四种完成路径，并固定 JADX 的重复清理反例。

本项以前置的 [recover-proved-finally-cleanup](../recover-proved-finally-cleanup/design.md) 直线证书为基础；不扩展到多正常出口、循环、switch、嵌套 guard、清理正文显式 `return`/`throw` 或通用异常语义求解。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 在已证 `finally` 清理与完成流之上，增加可证明的结构化受保护正文与融合块边界恢复。

## Impact

主要涉及 `jarde-java` 的 `guard::Plan`/`FinallyCopyProof`、`region::Walker`/`Region::Guard`、builder 的有界正文发射与来源/声明规划。复用现有 Java `Try`、`If`、`Throw`、`Return` AST；不改变 reader、SSA、canonical CFG、公开持久格式或生产依赖。测试使用仓内冻结 Java 8 class、JADX 输出与 runner。
