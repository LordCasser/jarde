## Why

CF-08 已能恢复一层 `if` 内的带效果双出口循环，但 [二层分支冻结基线](../../evidence/java-syntax-2026-09-27/cf08-endless-loops/two-level-if-baseline/report.md) 中，循环的两个出口与内层空数组臂直接汇合。Jarde 在外层 BCI 0 报 `arms_do_not_meet`、漏掉循环块，随后局部引用安全拒绝；原 class 和固定 JADX 的完整 Java 8 源码可重编并运行一致。该缺口是固定 `TestNotIndexedLoop` 更复杂失败路径的前置结构问题，值得先独立闭合。

## What Changes

- 在现有带效果双出口循环证书上，增加“内层 `if` 的另一臂与两个循环出口共用一个 join”的受限归属和三来源局部值证明。
- 让二层条件臂在现有 `Region::If`、`Region::Loop`、`LoopBreak` 和直线续接中闭合；不把 join 写进循环，也不跨越外层边界。
- 对冻结 `TwoLevelIf` 做原/JADX/Jarde 完整类 Java 8 重编与验证运行，核四组结果、物理来源及负例拒绝。对象构造型效果块、循环体虚调用和固定 `TestNotIndexedLoop` 整体恢复不在本片。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加受证的二层条件分支内带效果双出口循环及共享 join 的准确恢复。

## Impact

限于 `crates/jarde-java/src/region.rs` 的既有 Frame/Region 循环与分支证书；仅当源码构建的局部声明确实无法按已证 Region 关系呈现时，才在 `build.rs` 做该形态所需的最小修正。不新增 Region/AST 类型、通用 CFG 重写、依赖或放宽跨引用局部安全门。
