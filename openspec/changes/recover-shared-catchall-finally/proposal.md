## Why

CF-16 的单正常出口 `finally` 已受证恢复，但 `SharedFinallyCall.handled(boolean)` 因具名 `IllegalArgumentException` catch 与两个保护范围共用同一 catch-all handler 而整体引用，完整 Jarde 源码缺返回。原 class 在正常/catch 路径各调用一次清理；固定 JADX 源码虽可重编，却在正常路径清理两次。现有单出口证书不能靠放宽边界复用。

## What Changes

- 首片只针对一个具名 catch、两个按半开 BCI 范围排列的 catch-all 行、共用 handler、两个正常返回和三份相同无参静态 `cleanup()V` 调用，建立不可分割的私有多出口证书。
- 复用现有 `Plan`、有界子 Region 与 `StmtKind::Try`；行优先级、两份返回快照、异常身份、物理所有权与来源全部闭合后才发布。
- 重编运行已冻结的 `SharedFinallyCall` 原 class、固定 JADX 和 Jarde 完整源码；Jarde 与原 class 必须一致，记录 JADX 的重复清理差异。字段自增清理和 `FinallyOnce.escaping()` 留作独立后续范围。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加共享 catch-all 的具名 catch/finally 多出口准确恢复与原子拒绝边界。

## Impact

限于 `crates/jarde-java/src/guard.rs` 的 finally/typed catch 证书、`region.rs` 的保护范围所有权，以及 `build.rs` 的现有 `Try` 输出。不新增异常 IR、通用 CFG pass、AST 节点或依赖。固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的算法与测试账本见 `openspec/evidence/java-syntax-2026-09-27/cf16-finally/README.md`。
