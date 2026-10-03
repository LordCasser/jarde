## Why

[共享 latch 巡查](../../evidence/java-syntax-2026-10-04/shared-latch-patrol/README.md)以纯 int 单变量判别钉死：**外层 `continue` + 同体内嵌套循环**整方法拒绝（`jre_region_loop_shape` + 未覆盖块）——外层 latch 块被两条语义不同的边共享（外层 continue 边、内层循环退出边），既有形状证明要求 latch 归属单一所有者。单层含 continue、内层无 continue、双层无跳转均健康。真实业务代码（Map 缓存双层 for-each + 前缀过滤 `continue`）即此形，`Svc.lookup` 整方法不发布 → 完整类不可编。

## What Changes

- 循环形状证明接受 latch 共享：判据为 CFG 结构事实（内层循环完全包含于外层体 ∧ 内层退出目标 == 外层 latch ∧ continue 边目标 == 同一 latch），归属按边语义分派（continue→外层 latch；内层退出→离开内层后落入外层 latch）。不新增机制。
- S5 两形（无标签/带标签 `continue outer`）恢复且行为一致（`13`/`13`）；S3/S4 的 Map.Entry 双层形与 `Svc.lookup` **体**恢复（泛型投影属另一域，由既有 same-class/nested-headers 通道处理，本片只锚体）；单层/无跳转/既有循环形逐字不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：嵌套循环与外层 continue 共享 latch 时可证明并呈现，双层过滤循环完整恢复。

## Impact

`crates/jarde-java/src/region.rs`（循环形状证明与 latch 归属）及测试；与 `recover-loop-body-double-jumps`（同体双边归类）相邻但不同层，复用其边语义归类思路。既有循环/labeled/dj 全部切片零回退。
