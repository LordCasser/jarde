## Why

固定 JADX `TestTryCatchFinally2.TestCls.test(OutputStream)` 的三个正常可达循环位于同一 `try/finally` 受保护正文内，正常与异常路径都关闭同一个 `DataOutputStream`。Jarde 目前在 handler 证明和区域所有权处拒绝整方法，完整类输出不可重编；[固定证据](../../evidence/java-syntax-2026-09-28/cf16-test2-loop-finally/README.md)已将它与 Test5 的双返回、Test11 的异常 handler 循环区分。

## What Changes

- 对固定双行、void 完成、正文三循环和同一资源的两份 `close()`，增加有界物理证明，在证据完整时恢复一份 `try/finally`。
- 保持所有循环迭代与写入顺序、正常/异常各一次关闭、`close()` 对待传播异常的覆盖、原 Throwable 身份及全部物理 BCI 来源。
- 对有效近邻及预算/取消安全拒绝，并用原 class、原转写、固定 JADX Java-input、Jarde 完整类运行对照。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩展固定 JVM classfile 的 void 正文循环 finally 恢复与拒绝边界。

## Impact

仅涉及 `jarde-java` 私有 Guard/Region/Builder 的 finally 证明、局部活性和来源呈现，以及固定回归/反例。CLI、宿主接口、依赖和公开 IR 不变。前置条件是现有 CF-16 已验收证书、Test2 固定物理与行为证据；Test5 双返回证书不被放宽。DEX 默认测试、其他编译器 lowering、TWR 和 Test9 可空清理均不在本变更内。
