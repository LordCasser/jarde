# Design

## Context

见 proposal.md 的实际记录。本地 OpenSpec 1.14.1 strict 将过长 requirement 正文及无 checkbox 的任务视为失败；本机两个 1.14.1 副本的 210 个 dist JavaScript 文件摘要相同。CI 固定 1.11.0 已实际通过，不升级或修改 CI 工具版本；分别保留两个版本的真实结果，不能混称同一门禁。

## Goals / Non-Goals

**Goals:** 文档通过真实 strict，旧契约完整可审，未完成任务在统计中如实出现。

**Non-Goals:** 修改 validator、扩大 lint 例外、删除要求或场景、改变产品行为、重命名 requirement、重写历史 archive、同步尚未满足前置的 delta。

## Decisions

1. 使用现有 Markdown 结构和 OpenSpec CLI。正文保留完整的简明主张，详细约束迁入同一 requirement 的明确场景；正文用 SHALL/MUST 绑定全部迁移约束。原始量词、身份、拒绝门、预算、来源及终止契约逐条保持，不以“背景”“示例”降为可选行为。无需新库、parser 或文档生成框架。
2. 场景的 WHEN 覆盖原要求的操作和输入，不增加一个“适用时”条件门。表格与多段规则按原有行为关系组织；既有 scenario 保持全文，任何内嵌标题不得逃逸到另一条 requirement。
3. 两份 tasks 只把既有编号行改成未勾选复选框。之前的零任务是格式事实，不能作为实现完成证据；新的任务状态须由各自后续验收决定。
4. 私有候选绑定基线 SHA、修改清单及原正文→新正文/场景的对应。reviewer 检查全部 38 条长要求、两个任务文件，与 coding root 协调后在不影响已冻结 CLI 的文档路径应用；源码工具链和 Git 提交继续由 coding root 串行处理。来源变化先重新核对，不覆盖。
   主规范新增完整契约场景也进入 MODIFIED 的完整替换规则。私有两版本 CLI 首轮实际为 362 passed / 3 failed，确定上述三个依赖 change 的四个 MODIFIED 块需要同样整理：迁移其自身原正文，不以主规范旧文本覆盖其已规划扩展，不漏主规范的场景名称。最终核验为 38 条长正文加四个依赖块、共 28 份文档。
5. 保留 validator 的 missing-MODIFIED-header INFO。这些涉及其他计划的前置与同步状态，不能为本次文档门禁随意重命名或伪造主 spec。当前只验收 24 个实际 strict 失败对象的明确修复。

## Risks / Trade-offs

- 移动正文弱化义务 → 保留全部原文条款并用显式 SHALL/MUST 绑定，逐条核拒绝条件和强制范围。
- Markdown 结构改变解析归属 → 核 requirement 标题、数量、全部既有场景及新增场景的归属；不能仅凭 exit 0 验收。
- 任务统计被误认进展 → 新增 14 个可识别任务全为未完成，不更改历史完成记录或产品能力计数。
- coding root 验收期间发生来源漂移 → 独立候选、摘要检查、串行应用，并保存新一轮全量 strict raw。

## Migration Plan

保留失败 raw → 准备并审读私有候选 → 对照来源摘要应用 → 单项与全量 strict 实跑 → 核两份任务各为 0/7、旧场景和 archive 摘要保持 → 独立提交。规划完成或候选存在均不勾选实施任务。若语义或结构核验失败，只撤回本次文档修订并保留失败证据。
