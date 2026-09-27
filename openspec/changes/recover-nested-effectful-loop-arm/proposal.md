## Why

CF-08 已验收的带效果双出口循环在顶层方法中能保留两条不同的退出路径；同一循环放进外层 `if` 后，当前 Region walker 在外层臂内留下循环与汇合块，整方法引用字节码并缺少返回。[冻结的嵌套样例](../../evidence/java-syntax-2026-09-27/cf08-endless-loops/nested-effectful-baseline/)已确认原始 class 与固定 JADX 完整 Java 8 源码的四组验证运行一致，而 Jarde 拒绝。这是现有循环证书的嵌套边界，不能靠放宽局部变量引用检查解决。

## What Changes

- 在已验收的 `EffectfulExits.pick` 精确双出口形态上，增加单个外层 `if` 臂的有界 Region 所有权证明：循环、两个出口、共同后继和随后直线尾段都必须落在该臂内，外层另一臂独立汇合。
- 保留效果调用仅在越界路径执行、直接 `break` 跳过效果路径的既有证书；让现有 `Region::Loop`、`LoopBreak` 与外层 `If` 组合原子发布完整方法和逐 BCI 来源。
- 固定原/JADX/Jarde 全类 Java 8 重编、`java -Xverify:all`、四组输出、负例和预算/取消门槛；任何额外入口、出口、异常边或未证局部消费仍拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：恢复受证的嵌套外层分支中的带效果双出口无限循环和其独占直线续接。

## Impact

主要涉及 `crates/jarde-java/src/region.rs` 的现有 `effectful_dual_exit_loop`、`Frame`/Region 走访与来源所有权，及聚焦回归测试。优先复用既有 Region/SSA/预算机制，不增加公共 pass、AST/Region 种类或依赖。固定 `TestNotIndexedLoop` 仍叠加 `new File`、循环内虚调用与内层长度分支，不作为本次完成条件；CF-16/CF-18 与其它循环形态也不混入。
