## Why

固定 JADX `TestTryCatchFinally4.TestCls.test()V` 的四行异常表表示：正文 `write(1)` 后，无论正常或异常退出，都执行 `close(); delete()`，并只在清理内部吞掉 `IOException`。JADX 把两份清理恢复成一个含内层 `try/catch` 的 `finally`；Jarde 在 BCI 38 的 guard handler 证明失败，随后因局部跨 fallback 区域拒绝整个方法。固定证据见 [Test4 对照](../../evidence/java-syntax-2026-09-28/cf16-test4-nested-cleanup/README.md)。

## What Changes

- 在既有 FINALLY Guard 通道增加固定四行、两份嵌套清理的联合证书：证明异常范围、两个 `IOException` catch、原异常重抛、清理调用顺序与接收者相同，且所有物理指令/边均有归属。
- 复用既有 `Region::Guard`、`StmtKind::Try` 和 catch/finally writer，一次呈现外层 `try/finally` 与 finally 内层 `try/catch`。只为证书携带不可从现有 `Shape::Finally` 推出的内层 catch 与异常副本事实；不建通用异常控制流重写机制。
- 用固定 class 的 BCI/异常表及 Java 8 同形编译输入做原/JADX/Jarde 三方验收；另以不同字节码的可注入 control 对照原/JADX 在正常、正文异常和清理异常下的源级语义。Jarde 对该三行 control 可继续安全拒绝；不为它增设第二个证书。错误表、错调用、错接收者及停止均须安全拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：恢复有完整证明的 finally 内嵌清理 catch，保留受检异常吞掉和未检异常覆盖原异常的次序。

## Impact

范围限于 `crates/jarde-java` 的 FINALLY Guard/Region/Build 与固定 JVM classfile 对照，不新增公共 API、IR、CLI 开关或依赖。Test4 的固定目标只可观察正常路径；control 的异常路径不能冒充固定目标的行为证明。Test11 的 handler 循环证书是独立 OpenSpec，本变更不得顺手扩成通用 finally 归并。
