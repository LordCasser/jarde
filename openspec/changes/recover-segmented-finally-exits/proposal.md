## Why

固定 JADX `TestTryCatchFinally13.TestCls.test(I)V` 的五条异常行覆盖两段不连续的 try 正文、一个具名 catch，以及早退、正常汇合、catch 汇合和异常重抛上的四份清理。原 class 与 JADX 的有效路径已有冻结对照；Jarde 在 handler BCI 56 以 `jre_guard_finally_copy` 安全拒绝。现有两/三行 finally 证书不能组合处理这五行：它们会重复认领共用 handler，且不能同时表达四种出口。

## What Changes

- 在现有 Guard/Region/Builder 链中增加仅覆盖受证五行、两段保护区、四份等价清理与四种出口的私有证书，并输出一次 Java `try`/`catch`/`finally`。
- 逐一证明异常行顺序、保护范围、分支必经清理、清理副本的调用/SSA 身份及原 Throwable 重抛；保留每个物理 BCI 和异常行来源。证据不全时整方法保持安全拒绝。
- 用固定 class 的完整 Java 8 源码和运行路径对照原 class、pinned JADX、Jarde；用四个 verifier 有效近邻负例与既有 finally 回归约束证书边界。扩围负例以原 class 为语义基准，不能沿用 JADX 少执行一次清理的错误输出。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对获证的两段保护范围、具名 catch 与四副本 finally，恢复语义等价且来源完整的 Java 8 结构。

## Impact

修改 `crates/jarde-java` 内部 Guard 证书、Region 有界遍历和 Builder 既有 `try/catch/finally` 构造，以及定向测试/证据；不新增 pass、公共 IR、crate 或依赖。固定 Test13 的相邻负例与预算/取消路径在本次验收内；`FinallyOnce.handled/escaping`、任意多段异常图、TWR、同步块及 Test12 的 `runTest` 均不在本变更范围。
