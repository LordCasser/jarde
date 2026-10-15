# Proposal

## Why

当前共享 facts store 的 container 与 CP/Header 新项准入漏计 definition-read 条目，可能突破 `FactsCapacity.entries`；CP/Header 不兼容项被丢弃时又未扣减权重，使空 store 仍被误判为字节容量不足。这两处记账缺陷违反既有容量和透明回退契约，应在新增缓存能力前修复。

## What Changes

- 三类已保留条目统一参与总 entry 容量判断与拒绝原因统计。
- CP/Header 项因 format、registry 或 product 不兼容被移除时同步扣减其 store 权重。
- 增加不同插入顺序、满容量回退、失效后重新保留和仍存活请求引用的回归验证。
- 保持已取得完整事实可由当前请求消费；不引入淘汰、全局缓存、依赖或新公开 carrier。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `facts-cache`：明确共享层总条目边界和失效移除后的 store 驻留权重，补充可观察的容量回归场景。

## Impact

主要涉及 `crates/jarde-reader/src/facts_cache.rs` 与现有 P5 缓存测试。前置是当前三类 store 已存在；修复可独立于普通 try/List 恢复实施。共享 Git、Cargo、rustfmt 和最终门禁仍由 coding root 串行管理，review agent 准备私有候选并独立审查；规划完成不代表实现或验收完成。性能收益、并行缓存、缓存 key/schema 变更及一般内存测量不在范围内。
