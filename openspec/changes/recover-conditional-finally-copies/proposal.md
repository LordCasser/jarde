## Why

固定 JADX `TestTryCatchFinally14.TestCls.test()V` 的正常与异常完成路径各执行一份带判空分支的清理。原 class 与固定 JADX 在七条可观察路径上相同，Jarde 在该方法入口保守拒绝；现有 finally 副本证明只支持直线清理，无法把两份带分支的清理安全折成一份。

## What Changes

- 以已冻结的同布局目标方法和七路径回放为基线，构造不受无关 synthetic accessor 缺口影响的最小完整类验收载体。
- 对单行 catch-all、一个条件判断加一处可选调用的两份清理建立有界 CFG/SSA 等价证明，核正常/异常入口、字段重新读取、唯一出口和原 Throwable 身份。
- 复用现有 FINALLY pass、区域 `If` 和 `Try` AST，在一个原子构建中输出正文与唯一结构化条件 finally；证据不足继续保守拒绝。
- 比较原/JADX/Jarde 的完整 Java 8 源码重编、`-Xverify:all` 七路径，以及 verifier 有效的不同字段/调用、异常行自保护、异常值改变等近邻。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：证明两份有界条件清理等价时恢复单行 catch-all 的 `try/finally`，保持字段重复读取及异常覆盖行为。

## Impact

影响 `jarde-java` 的私有 finally 证书、Region 有界子区域和 Builder 的 finally 正文构造；不新增公开 IR、通用异常重写或另一套语法 pass。Test12/Test13、`FinallyOnce` 的不同保护形态及 fixture 的无关 synthetic accessor 不属于本 change。实施在当前 CF-16 exception-only 与下一项 concat-saved shared-finally 改动验收后排队，避免并发触碰同一核心文件。
