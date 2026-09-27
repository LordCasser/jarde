## Why

固定 `FinallyOnce.handled(Z)String` 的三行共享 catch-all 与三份相同静态字段清理，已经落在现有共享 finally 证书的结构边界内；剩余关键差异是具名 catch 的返回值由 `StringBuilder.toString()` 产生，而证书目前只接受 `Push` 字面量。Jarde 因而保守拒绝，完整源码不能重编；固定 JADX 虽能重编，却在正常路径重复清理，不能作为行为真值。

## What Changes

- 冻结该方法的原 class、字节码、原/JADX/Jarde 完整源码及运行差异，再制作保持目标方法 BCI、opcode 和三行异常表不变的最小完整验收类。
- 在现有共享 finally 私有证书中，为 catch 的保存返回值补足一个有界的 Java 8 字符串拼接生产者证明；保留已受证的字面量路径与三份字段清理证明。
- 复用现有 Region/Builder 的 `Try`、具名 catch、两个保存返回值和唯一 finally；只有求值时机、异常覆盖、SSA 值和所有权均闭合才发布源码。
- 用原 class 的正常/catch/异常运行行为验收 Jarde；把 JADX 正常路径二次清理明确记为参考实现的语义差异。增加 verifier 有效的错误近邻及资源停止回归。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：共享 finally 的保存返回值可来自经证明的有界字符串拼接表达式，且返回表达式仍在清理前求值一次。

## Impact

限于 `jarde-java` 已有共享 finally Guard 证书、必要的返回值呈现接缝及其定向测试；不新增通用表达式机制或异常 IR。实施应在单行 exception-only catch-all 改动合入后基于主线进行，避免并发修改同一 Guard/Region 文件。`FinallyOnce.escaping()` 和 Test14 条件清理各自独立，不随本 change 声称完成。
