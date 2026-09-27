## Why

CF-16 的单正常出口 `finally` 已受证恢复，但 `FinallyOnce.handled(boolean)` 仍因具名 `IllegalArgumentException` catch 与两个保护范围共用同一 catch-all handler 而整体引用，完整 Jarde 源码缺返回。原 class 在正常/catch 路径各清理一次；固定 JADX 源码虽可重编，却在正常路径清理两次。继续只按相似调用去重会损坏语义。

## What Changes

- 对**一个具名 catch、两个按半开 BCI 范围排列的 catch-all 行、一个共用直线异常清理 handler、两个正常返回副本**建立私有多出口证书，证明每条完成路径恰执行一次同一清理。
- 复用现有 `guard` 的 SSA 副本比较、`Plan`/Region 有界子正文及 `StmtKind::Try` 的 catch/finally 输出；只在行优先级、返回值快照、异常身份、物理所有权与来源全部闭合后发布。
- 构造只含该 `handled` 形态和计数器的 Java 8 类族，分别重编运行原 class、固定 JADX、Jarde 完整源码；Jarde 与原 class 必须一致，记录而非复制 JADX 的重复清理差异。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加共享 catch-all 的具名 catch/finally 多出口准确恢复与原子拒绝边界。

## Impact

限于 `crates/jarde-java/src/guard.rs` 的 finally/typed catch 证书、`region.rs` 的保护范围所有权，以及 `build.rs` 的现有 `Try` 输出。不新增异常 IR、通用 CFG pass、AST 节点或依赖。固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的算法与测试账本见 `openspec/evidence/java-syntax-2026-09-27/cf16-finally/README.md`。
