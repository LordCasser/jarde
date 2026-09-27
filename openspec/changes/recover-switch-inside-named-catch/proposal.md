## Why

固定 JADX `TestTryCatchFinally12.TestCls.runTest(II)String` 在单条 `[11,61)→64 IllegalArgumentException` 异常行内执行整数 `switch`，随后从 BCI 61 跳到 BCI 79 返回。当前主线已能恢复同类的 `test1/2/3`，但 `runTest` 仍因 `jre_region_ownership_overlap` 只输出解释。隔离诊断定位两处同一路径上的边界：保护区前 BCI 8 的完整实例字段赋值被误当作 TWR 资源头；排除此假候选后，具名 catch 和 switch 都可恢复，但保护区外的唯一出口 `goto` 块 BCI 61 未被认领。

## What Changes

- 在现有 Guard 资源候选判定中，只对已证明完整、栈闭合的保护区前字段赋值排除 TWR 假候选，使真正的具名 catch 得以解析；保留真实 TWR 候选的优先级与拒绝。
- 在现有 Region 的 try 正文续走中，证明异常范围 exclusive end 上的唯一、无效果 transfer 确实汇合了本次正文所有正常出口，再由现有 `region_at`/`split` 认领它；保持 switch arm、catch handler、后续 return 各一个物理 owner 和完整来源。
- 用固定字节码和 Java 8 可运行完整类，三方比较原 class、pinned JADX、Jarde 的正常 case、异常 case 与 default 路径；用 JVM verifier 有效近邻及已有 TWR/switch 回归守住拒绝边界。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对受证完整字段前缀、整 switch 保护范围和唯一出口 transfer 的具名 catch，输出可运行的 `try { switch ... } catch (...)` 结构。

## Impact

仅涉及 `crates/jarde-java` 既有 Guard/Region/Builder 来源路径及定向证据、测试。无需新 pass、CFG 改写、公共 IR 节点、crate 或依赖。固定类中的 InnerClasses family、其它 switch/catch 嵌套、真实 TWR 的新形态和 `TestTryCatchFinally13` 多段 finally 均不在本变更范围。
