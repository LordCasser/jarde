## Why

固定 JADX `TestTryCatchFinally16` 的 Java 8 正例要求空 `catch (Exception e)` 与唯一 `finally`，但关闭了其源码重编检查。我们构造的同布局可观察类证明原源码和固定 JADX 的完整源码在六条验证路径上一致；当前 Jarde 对目标 `test()V` 仍安全拒绝，无法恢复这个常见的三份调用型清理。

## What Changes

- 在现有 finally Guard/Region/Builder 路径内，增加只针对两条异常行、空具名 catch、正常/具名 catch/异常 handler 三份相同 `invokestatic ()V` 清理的有界证明与唯一 `try/catch/finally` 投影。
- 证明三份物理副本的目标、异常覆盖、CFG 入口出口、原 Throwable 身份和完整 BCI 所有权；任何改变这些事实的有效近邻保守拒绝，预算/取消原子停止。
- 用[固定 Test16 基线](../../evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/README.md)对照原 class、固定 JADX、Jarde 的完整 Java 8 类重编和六路径验证运行。只闭合此 Java 8 lowering；D8/DX 与其它空 catch/finally 形态另验。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在受证的空具名 catch 与三份调用型清理中输出唯一 finally，并保持异常优先级与物理来源。

## Impact

影响 `crates/jarde-java` 的既有 Guard 证书、Region 有界走访及 Builder 输出；不新增解析层、公共 API、CLI 选项或依赖。现有 Test12–14、共享 finally 与具名 catch 证书必须保持行为。`Test15` 的错误寄存器合并负例及固定 `FinallyOnce.main` 属独立边界，不并入本改动。
