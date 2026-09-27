## Why

固定 JADX `TestTryCatchFinally11.TestCls.test(List)` 的 Java 8 字节码在正常与异常出口各有一份遍历列表的清理循环。原类和固定 JADX 完整源码行为一致；Jarde 却在 FINALLY 证明之前，把只由异常 handler 进入的单入口循环误判为不可约，因而整个方法安全拒绝。隔离实验表明单独修正此误判后仍缺清理循环的所有权证明，必须把两个边界一起闭合才可宣称恢复。

## What Changes

- 在现有 normal-flow 视图中为异常才可达、具有唯一正常入口的组件建立有界的循环事实；保留方法入口 dominance 的既有含义，真正多入口或零入口循环继续拒绝。
- 在现有 FINALLY Guard/Region/Builder 通道证明固定两行 catch-all 布局的两份可观察迭代清理：同一列表入口值、同一 `iterator/hasNext/next` 和逐项调用、正常退出与原 Throwable 重抛；一次输出 `try/finally` 中的循环。
- 以[固定 Test11 class 与探针](../../evidence/java-syntax-2026-09-28/cf16-test11-loop-finally/README.md)重编原/JADX/Jarde 完整 Java 8 源码、运行正常和异常路径，核全部物理 BCI 来源及 verifier 有效近邻；任一证明失败或停止则保留拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证的异常 handler 循环可以作为 finally 的完整清理副本恢复，同时不把单入口 handler 循环误判为不可约。

## Impact

仅修改 `crates/jarde-java` 现有 normal-flow、FINALLY 证明、区域与源码投影；不新增 JVM IR、公共 API、CLI 开关或依赖。固定证据是 Java 8 classfile，DEX/D8、任意循环体以及普通多入口不可约循环不在本次范围。Test11 的组件循环事实是本功能前提，不能单独当作 finally 已恢复；旧两/三/四/五行 finally 证书和普通循环须回归。
