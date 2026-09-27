## Why

[CF-08 隔离样例](../../evidence/java-syntax-2026-09-27/cf08-endless-loops/join-baseline/report.md)把剩余 `NotIndexedLoop` 差距拆出一条独立控制流边界：外层 `if` 的一臂含内层 `if`，内层一臂为单出口循环，两臂在 BCI 26 汇合后还需执行 `value += 10`，才到外层 BCI 34。原 class 与固定 JADX 完整 Java 8 源码均编译运行 `4 / 13 / 11`；Jarde 在 BCI 0 报 `jre_region_arms_do_not_meet`，完整源码缺返回。现有 `continue_inner_join_arm` 只允许两个直线臂，不能消费已结构化循环的同臂续接。

## What Changes

- 在现有 `Region::If` 与 `Region::Loop` 表示内，为**一个单出口循环臂**和另一个直线臂证明同一个内层 join，并继续走访内层 join 到外层 arm 边界的独占直线尾部。
- 仍要求完整 normal/exception predecessor、实际出口、Region owner、来源和预算证明；不靠局部 slot 错误或文本拼接推断流程。
- 用冻结三方完整 Java 8 重编与验证运行关闭这条首片，保留原 `NotIndexedLoop` 的带效果双出口拒绝，直到它有单独证据和任务。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：内层 `if` 的单出口循环臂与直线臂能在获证局部 join 续接，并保留外层 arm 的语句顺序。

## Impact

主要修改 `crates/jarde-java/src/region.rs` 中现有 `continue_inner_join_arm` 的私有边界证明和定向测试；复用当前 Region、canonical CFG、来源、预算与拒绝合同，不增加公共 IR 或通用 CFG 改写。
