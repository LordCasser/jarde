## Why

对 `bafdcec`（行为提交 `85828c4`）的独立复核发现：名称搜索遇到损坏候选后，任务操作会把已确认的一个候选当成唯一目标并返回 Complete，或把零候选误报为未找到；类视图也会在方法体解码 Partial 时返回顶层 Complete。已有回归全绿不足以关闭这两条反例，必须先修正操作边界，再确认本轮任务链完成。

## What Changes

- 名称选择保留搜索的 execution、coverage、diagnostics 和可靠候选；只有完整搜索才作唯一/缺失判断，未完成时不启动方法分析或恢复。
- **BREAKING**：任务操作结果增加明确的未完成选择分支，库调用方和 CLI 一次性迁移，不用 Ambiguous 或输入错误冒充停止。
- 类视图在库内汇总所请求方法体的停止状态，同时保留逐方法结果、独立 coverage 与局部错误隔离。
- 以库/CLI 反例、正向对照及预算/取消边界验收；CLI 从库报告读取停止，名称选择未完成退出 4。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `analysis-contracts`：明确不完整目标搜索与组合操作停止状态的传播，以及相应 CLI 退出语义。

## Impact

前提是已归档的 `add-artifact-navigation`、`add-task-oriented-operations` 和 `add-task-oriented-cli`。影响根库 `src/facade.rs` 的选择/组合报告、CLI 适配及对应回归；证据见 [独立完成复核](../../completion-review.md)。不新增 crate、第三方依赖、持久状态或兼容层。

非目标：恢复算法、container cache、分页或性能优化；不重开 R8/R9，不重构整个 facade，不修复 body 解码重新解析类等独立架构债务。当前仅完成修正规划，实施任务全部待办；历史归档保持原状。
