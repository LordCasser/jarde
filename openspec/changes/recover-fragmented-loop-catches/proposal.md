## Why

固定 CF-18 的完整 Java 8 类在循环中的内层 catch/continue 与外层分段同 handler catch 相交，Jarde 于 BCI 0 报 `jre_region_irreducible` 并使整类源码不可编译。当前门把仅经异常边进入的内层 handler 当作普通循环区块要求头部支配；即便解除此首拒绝，现有 catch 所有权也无法把分段异常表认作同一外层 catch。固定 JADX 完整源码虽可编译，却把外层 catch 放进负值分支，漏掉受外层行 `[28,52)` 保护的 BCI 28–35 `work(value)` 调用，运行语义错误，必须以原 class 为基准。

## What Changes

- 为循环内嵌套的具名 catch 和同 handler 分段保护区构造有界的异常行、物理块和词法所有权证明；只有异常根的每条正常完成路径向既有循环续接点汇合、每条表行顺序及每条可抛指令的保护范围完整闭合时，才排除假普通流不可约并恢复一个外层 catch。
- 保持 handler、循环更新与普通/异常续接块恰好一个结构所有者，按同轮 SSA 证明 catch 参数、局部写入和后继读取；任一边界不闭合即保留整方法可定位引用。
- 以固定完整 `ExceptionRegionsAudit` 的原/JADX/Jarde 三方 Java 8 重编和运行，以及缩小的 `HandlerLoopProbe` 三方结构对照作为验收；保留 JADX 在完整类上错放 catch 的反例，不把其文本当正确性 oracle。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩充可证明的循环内嵌套具名 catch 与同 handler 分段异常范围恢复，同时保持异常派发、效果顺序、来源和拒绝边界。

## Impact

主要涉及 `jarde-java` 现有 normal-flow 循环例外、Guard catch 候选、Region/Builder 的 try 与循环归属；优先复用已有异常表事实、Canonical CFG、SSA 与 Region/AST。若确需新的候选内证明数据，只附着于此受限形态，不引入全局异常重排器或新通用 IR。排除 finally、TWR、交叉异常区、任意 handler 重排及无本地证据的 Java profile。已证阻碍及反例见 [架构分析](../../evidence/java-syntax-2026-09-27/cf18-handler-region-triage/architecture.md)；提案不表示样本已经恢复。
