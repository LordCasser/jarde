## Why

CF-16 的 `SharedFinallyCall.handled` 已能恢复一个具名 catch、两处返回和共用 catch-all 的三份清理调用，但固定 `SharedFinally.handled` 把同一 `static int` 字段的读、加一、写回复制在三个出口，Jarde 仍整体引用且完整源码缺返回。原 class 每条路径只清理一次；固定 JADX 在正常路径执行两次。现有共享 finally 的控制流证书已覆盖此结构，差距集中在四指令清理效果的逐份等价证明和呈现范围。

## What Changes

- 在现有共享 catch-all 证书中支持三份相同的静态 `int` 字段加常量清理；按字段身份、算术操作、SSA 生产/消费和异常表边界整体证明，不能仅凭指令外形或 slot 号折叠。
- 复用已受证的双返回、具名 catch、两个有界子 Region 和 `StmtKind::Try`，把唯一字段更新放进 `finally`，保留全部物理 BCI 来源；任何一份副本、返回、异常路径或所有权不闭合时仍原子拒绝。
- 用固定原 class、JADX 和实时 Jarde **完整类源码**分别按 Java 8 重编、验证和运行，以原 class 的正常/catch 路径计数验收，并加入 verifier 有效的字段、增量和保护范围负例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩充已受证共享 finally 的可观察清理效果，要求静态整数字段增量在多个完成出口只执行一次且无法证明时安全拒绝。

## Impact

涉及 `crates/jarde-java/src/guard.rs` 的私有清理证书、`region.rs` 的现有共享 finally 选择及 `build.rs` 的已有 `Try` 输出范围。字段表达式呈现能力已由单独 `count++` 方法确认；不新增 AST、通用异常 IR、CFG 重写或依赖。`FinallyOnce.escaping()`、实例字段、非整数/复合清理和更多 catch 仍不在本次范围。固定 JADX revision 与原/JADX/Jarde 基线见 `tests/fixtures/p3-shared-catchall-finally/README.md`。
