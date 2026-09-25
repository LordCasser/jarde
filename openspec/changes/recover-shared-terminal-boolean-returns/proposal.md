## Why

Java 8 方法中的多个条件可共享 `return true`、`return false` 两个终结块，而没有栈 Phi 或单一普通汇合块。冻结的 `TernaryInIfProbe.bothMatch` 因 BCI 62 被两个 Region arm 认领而整体引用，完整 Jarde 类无法编译；原 class 与 JADX 的八条验证执行路径一致。需要在保持物理边和唯一所有权约束的前提下恢复这一有界控制图。

## What Changes

- 对入口、全部有序条件测试、纯转接和两个布尔终结返回均可证明的无环子图，输出一个等价的 Java 布尔返回表达式；不要求逐字还原原源码中的 `?:`。
- 在 Region 认领前一次提交图中每个物理块，保留每条测试的真实 taken/fallthrough、惰性求值、异常顺序和全部 BCI 来源。
- 对非布尔返回、第三出口、额外入口、循环/异常边、独立效果、极性或 SSA 身份不明及预算/取消作原子拒绝；保留现有 owner overlap 检查。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加有界、共享两个终结返回块的布尔控制图恢复及拒绝/来源约束。

## Impact

前置依赖为现有 Java 8 reader、canonical CFG/SSA、Region、Boolean 表达式、Return AST 和 source map，不增加公开 API、外部库或目标代码执行。改动限于 Jarde 的 Region 候选与 Builder 证明/发射及永久 Java 8 对照测试；JADX 只作本地算法参照和离线 oracle。普通 `if`、异常范围内分支、任意多出口 DAG 和源码原样重构不在此 change 范围。
