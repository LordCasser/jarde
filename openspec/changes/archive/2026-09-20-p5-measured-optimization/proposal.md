## Why

以某个被优化阶段（P1/P2/P3/P4）的结果契约和真实基线稳定为进入条件；不要求等待后续阶段完成。本 change 规划通过可复现实验选择优化，确保任何 cache、index、并行或 query 合并都不改变 evidence、coverage、fallback 和 runtime 语义；当前尚未实现。

## What Changes

- 建立代表性语料、冷/热、局部/全范围、成功/退化输入的基准与资源测量协议。
- 基于实测瓶颈选择多查询合并、可选 facts cache/index、细粒度并行或退化输入优化；具体实现与依赖不预设。
- 增加缓存键、snapshot/view/profile/registry/recovery 失效规则和取消/预算一致性门槛。
- 将性能收益、内存峰值、并发、取消和正确性回归纳入发布门槛。

## Capabilities

### New Capabilities

- `measured-execution`: 以可复现实验驱动 query/decompile 调度和资源预算优化。
- `facts-cache`: 提供可选、可失效、不会改变语义的 facts cache/index 行为。
- `performance-gates`: 发布基准、稳定性、内存、取消和正确性对照门槛。

### Modified Capabilities

无。P5 不改变 P1–P4 的结果契约；任何性能优化必须通过相同的 evidence/coverage/diagnostic 语义验证。

## Impact

影响 `jarde` 的 scheduler、cache/index、benchmark harness、metrics 和 CLI diagnostics；是否新增依赖、是否落盘、是否启用持久索引均由实测结果和单独设计决定。本 change 不预设阈值、工期、索引格式或库选型，也不宣称已实现；验收重点为 A14–A18 及被选目标阶段的回归；若优化跨阶段，再覆盖所有受影响阶段。
