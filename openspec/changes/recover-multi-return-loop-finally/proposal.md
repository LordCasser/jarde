## Why

固定 JADX `TestTryCatchFinally5` 的 Java 11 class 在正常可达的 do-while 中构造返回值，并在两个 return 与异常完成前运行同一份 `close()`。Jarde 目前安全拒绝该方法，完整类源码不能重编；既有 Test11 的异常 handler 循环证书和具名 catch 的保存返回证书都不覆盖这个布局。

## What Changes

- 对固定的三行 catch-all、正文循环、两处保存返回值和一处异常重抛布局，证明三份清理调用的值、目标、异常覆盖及每条完成路径。
- 在既有 Guard、Region、Build 层交付一个带两个 return 的普通 `try/finally`，保留循环、求值顺序、Throwable 身份和全部物理来源；不能完整证明时继续安全拒绝。
- 用固定原 class、原源码、JADX、Jarde 的 Java 8 重编与 `-Xverify:all` 行为对照，以及 verifier 有效的近邻变异验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加有界的正文循环加双保存返回 finally 恢复契约。

## Impact

影响 `jarde-java` 的共享 finally 证明、区域所有权、源码构建及来源映射；复用现有 IR/AST 和预算、取消机制，不新增 crate、通用 pass 或对外 API。固定证据位于 `cf16-test5-multi-return`；Test2 的 void 循环 finally、Test9 的可空资源及其他编译 profile 均不在此变更内。
