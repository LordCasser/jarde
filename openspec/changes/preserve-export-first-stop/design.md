## Context

ledger::record 先写 BulkStop 再取消共享 token，后续 checkpoint 的 Cancelled 不覆盖该记录。bulk::publish 保留同一 first stop。错误位于 export::run：无条件优先取 Stream::stopped 的 error。worker 停止可发生在 coordinator 的 ResultItems charge 与 sink 内 OutputBytes charge 之间。

## Goals / Non-Goals

恢复首次停止的可观察原因，保留真实输出预算用量与 I/O 错误。非目标：新取消机制、重新执行、并发调度调整、测试放宽、库层返回类型迁移。

## Decisions

复用报告 BulkStop 的 kind/dimension/owner。Output 错误继续直接返回；Allowance 只有实际 BudgetExceeded/OutputBytes 与首次 Budget/OutputBytes/Delivery 匹配时直接返回，其余走现有 unfinished_error(report)。外部取消仍由报告中的 Cancelled 表达，不能误归预算。无需第三方库；现有 serde 与报告足够，不增加许可或维护负担。

提取或修改现有私有选择函数，在原始 first-stop 与次生回调错误组合上确定性测试；不依赖运行时恰好撞上 race。集成测试仍验证紧 ir_items 输出 cli_export_unfinished、原始维度和无 final，宽预算完整；已有输出预算测试保持数字断言。

## Risks / Trade-offs

对全部 Allowance 一刀切会丢失真实 OutputBytes 的详细数字；只特判 Cancelled 会漏掉先发生其他预算、后出现本地预算错误的并发组合。因此以首次记录与本地错误相符为条件。无 first stop 的异常组合保守引用报告，不猜测预算归属。

## Migration Plan

独立提交，和在飞构造实参片共同 root 验收后合入。fmt/clippy、export 集成全族、两个固定 seed workspace 及既有 ignored oracle 通过后推送并核对 CI。
