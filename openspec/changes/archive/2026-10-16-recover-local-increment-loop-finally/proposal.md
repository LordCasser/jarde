# Proposal

## Why

已交付的条件返回 finally 能恢复无循环完整类，但原 FY.noCall 的循环内局部自增清理仍输出 fallback，完整独立 NoCallFinally 生成源码重编译因 missing return 失败。两种编译器的实际 CFG/SSA 与 Region 探针已明确该缺口，适合继续一个有界叶片。

## What Changes

- 在已完成 conditional-return finally 主线与自身 CI 基础上，恢复固定 int[] 循环中条件提前返回、其它轮次继续、finally 对同一 int 局部加100的完整控制流，保持先保存 c+7、再清理、再返回。
- 完整保留 try 外的数组元素读取与循环更新，证明三份局部更新、真实 phi 与物理异常表；不足时保守拒绝，真实停止不降格。
- 用未修改原样例、独立驱动、两种 javac 输入及 Java8/23 全类重建运行验收；另保存原 FY/FX 整类现状。
- 非目标：通用局部清理语法、任意循环finally、finally自身return、重写P2异常/SSA事实、强制漂亮的for语法、完成整个父计划或扩大旧计费表。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增有证据的循环内局部更新 finally 与条件已保存返回的恢复契约。

## Impact

预计改动 jarde-java 私有guard证书与region局部范围接线，复用既有 Finally/SavedReturn、尾部发射及事务回滚；仅确有必要时调整builder。增加实际类fixture、公开入口/CLI回归与原样重编运行收据。无新增依赖、公开枚举形状或预算维度，不改变宿主无关分层。当前仅有基线与探针证据，尚无该能力的产品实现。
