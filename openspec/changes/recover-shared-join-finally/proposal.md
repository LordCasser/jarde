## Why

固定 JADX `TestTryCatchFinally.TestCls` 的 `test(Object)` 有一个具名 catch 和共用 catch-all，三条完成路径各执行一次 `this.f = true`，两条正常路径在清理后汇入同一个 `return this.f`。原类及固定 JADX 的完整 Java 8 源码在正常、异常路径和 `check()` 中一致；当前 Jarde 在 BCI 31 整方法拒绝，生成的完整类缺返回而不能重编。已实现的 CF-16 共享 finally 证书只证明“清理前保存两个返回值、清理后分别返回”的形态，不能以虚构的保存返回套用这个固定测试。

## What Changes

- 在现有共享 finally 恢复路径内增加“清理后共用正常汇合点”的互斥完成形态。逐一证明三条异常表行、正常/具名 catch/catch-all 的三份实例布尔字段赋值、`this` 接收者与常量值、两条 goto 的同一汇合点、原异常重抛及全部物理来源。
- 用有界 try/catch 子 Region 和原子 Builder checkpoint 输出一份 `finally { this.f = true; }`，然后在语法层继续恢复共用 `return this.f`；不能把它吞进 finally，也不能重复执行赋值。既有双返回共享证书保持原行为。
- 以固定完整类的原 class、固定 JADX、实时 Jarde 三方 Java 8 重编和 `-Xverify:all` 执行为验收；对 verifier 有效的副本效果不等价及异常/汇合边界变体保持安全拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：共享 catch-all 的三份相同清理若在正常路径汇入一个后续块，也可在来源、控制流和异常效果完整获证时输出唯一 `finally`；证据不闭合时继续原子拒绝。

## Impact

主要涉及 `crates/jarde-java/src/guard.rs` 的私有共享证书、`region.rs` 的有界子区域和 join 续接、`build.rs` 的同一个 `StmtKind::Try` checkpoint。`this.f = true` 可由现有字段赋值 Builder 呈现；不新增公开 AST、CFG 重写、通用异常 IR、依赖或 Java 方言。`FinallyOnce.handled/escaping` 的保存返回与其它保护形态、任意实例字段操作及多 catch 不在本任务范围。固定测试证据见 `openspec/evidence/java-syntax-2026-09-27/cf16-finally/fixed-test-triage-2026-09-28.md`。
