## Why

JADX 原件 `TestBreakInLoop2` 的具名 catch 包含“条件重抛，否则清理后重试”。完整基线中 Jarde 因 BCI 0 重复 owner 拒绝，整类不能重编；同轮 IR 已确认单块 handler 的 retry 证明拒绝后，普通 handler 遍历沿回跳再次进入保护区。

## What Changes

- 在已有具名 catch 异常重试能力内，支持一个条件 handler、一个终止 throw 分支和一个效果前缀后回到精确保护区入口的分支。
- 独立证明 handler 三块的边、终结、物理行与所有权闭合；用现有 Endless Loop、Try、If、Throw、LoopBreak/LoopContinue 表达结构。
- 保留现有拒绝、停止、预算与源码来源约束，以完整原件及可达异常路径控制例验收。
- 前置：真实 IR 与最终重复 owner 已定位。非目标：任意 handler 图、嵌套/多 catch/finally、自然 CFG 改写、Builder 新机制、尾跳来源债务及覆盖账本整项关闭。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：恢复已证明的具名 catch 条件重抛/重试循环，保留效果、转移与所有权。

## Impact

主要修改 `jarde-java` 的 `RetryLoopFacts`、retry 证明及受证明约束的 Try 组装；增加真实 class、证明级拒绝/停止与公共入口对照。无新依赖、公共协议、Region 种类或 CFG 改写。完整证据见 `results/same-run-ir-root-v1/` 与既有 Loop2 baseline。
