# Proposal

## Why

2026-10-14 实际重跑本地 OpenSpec 1.14.1 全量 strict 校验为 341 passed / 24 failed（365 items）：22 个既有 spec/change 的 38 条要求正文超过 500 字符，另两个 change 的编号任务没有 checkbox。仓库 CI 固定的 1.11.0 在本轮验收为 365 passed / 0 failed；本变更修复本地工作流文档卫生与零任务可见性，不将新版本警告误报为当前 CI 失败。原始证据与 CLI 摘要已保存至持久目录。

## What Changes

- 整理失败清单中的 4 个 main spec 与 18 个 active change：保持每个 requirement 标题和契约，缩短正文并把细节放入可检验的场景；保留全部既有场景。
- 同步整理三个依赖 change 的四份 MODIFIED 文档：`add-demand-driven-core-results`、`make-required-conversions-explicit`、`recover-proved-local-reference-write-compatibility`，保留各自正文与扩展行为，使其完整场景集合继续覆盖主规范。实际私有首轮校验已证明只修改原 24 份文件会留下这三项失败，最终修订范围为 28 份文档。
- 将 `recover-return-through-effectful-finally`、`recover-switch-case-nonlocal-exits` 的既有编号任务转为未勾选 checkbox，保留原文和编号；不推断完成状态。
- 逐条记录正文与场景的语义对应，再用真实全量 strict 校验和任务统计验收。历史 archive 与产品实现不修改。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

无行为变化；本变更声明 `skip_specs: true`，不创建 delta spec。既有 spec 的排版调整不得改变准入、拒绝、身份、来源、预算或执行状态契约。

## Impact

仅修改上述既有 Markdown 规划文档及本变更的验证记录。前置是保留当前失败 raw 与来源摘要，由 reviewer 核语义、coding root 串行应用和验收。当前仅立项与私有候选准备，实施任务未完成。主 spec 与旧 delta 的 missing-header INFO 单独记录；不在本修复中重命名契约或改变 archive 状态。
