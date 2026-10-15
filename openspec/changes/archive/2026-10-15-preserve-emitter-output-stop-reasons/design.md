## Context

`Emitter::put` 先调用 `stop::poll`，随后通过 `.is_ok()` 压缩 `Budget::check` 与 `Budget::charge` 的错误，并重建 `Budget(OutputBytes)`。`Budget` 的后续检查仍可能观察取消或 deadline，故最初 poll 不能证明两个后续操作一定成功。ledger observer 在操作执行后接收通知，允许确定性复现中间取消，无需线程竞态或 sleep。

进一步审读发现 `stop::charge` 目前只区分 `Error::Cancelled`，其余错误均重建请求计费维度的 Budget；真实 `BudgetExceeded(ElapsedMillis)` 因而同样会被错误分类。必须补齐这一必要分支，并让只读 check 与 charge 共用一个转换函数，避免 emitter 专属转换器。

当前已正式立项，生产入口尚未修复。两文件源码与回归候选保存在 reviewer 的持久私有目录，未编译或运行；共享工具链、Git 索引及源码快照由 coding chat 串行管理。

## Goals / Non-Goals

- 保留每次输出许可的真实停止类别，确保原子丢弃。
- 不改 reader 的先获许可再计费语义，不退还已获许可，不重解释其它阶段的 first stop。
- 仅处理提交 `put`；replay 继续校验既有正文，不重新申请输出许可。

## Decisions

1. 在现有 `stop::charge` 的错误分支中保留当前 Cancelled 处理，增加真实 `BudgetExceeded(ElapsedMillis)` 到既有 `Interrupted(BUDGET_INTERRUPTED_CODE)` 的精确映射；实际计费维度超限保持原 Budget 口径。抽取单一私有错误转换，供 typed `check` 与 `charge` 共用；输出入口保持先只读 Probe、再原子 charge 的顺序。Probe 超额只停止当前方法，真正 charge 的竞争失败才记录 operation stop。不得引入第二个错误转换器、pending-stop 状态或新结果框架。
2. 仅对真实 `Budget(OutputBytes)` 补回 emitter 的 `written` 与既有 `limit`，保留原输出额度报告口径。其它 `StopReason` 原样传播并带当前 `at`。任一失败先执行已有 `discard`，再返回 `Halt::Stop`。
3. 确定性测试在一次直接 `put` 的第二个及第三个 ledger Checkpoint 回调分别取消：初始 poll 已通过，原代码的后续检查会误归类，新代码的 Probe 或 charge 检查均应返回 Cancelled。partial-write 测试在前一次写入成功后开启 observer，验证此前真实已写进度与 buffer 清空。对真实 `elapsed_millis=0` Budget 直接调用共享 charge，分别请求 IR 与输出维度；reader 使用 `elapsed >= limit`，故无需 sleep 即可证明实际 deadline 应为 Interrupted，而非请求维度的额度错误。既有精确额度、UTF-8、嵌套表达式边界和 replay 回归继续使用原断言。
4. 复用 reader/Java 已有模块与标准库测试 observer，许可证和维护依赖不变。无需新库；本项不涉及 classfile 解析、验证、方言或运行选择规则。

## Risks / Trade-offs

- 2026-10-15 集成门禁证实：移除只读 Probe 会把串行方法局部额度停止升级成 operation stop，破坏既有批量恢复契约。已撤销该设计假设，恢复 typed Probe；保持 `bulk_recovery_delivery` 的全部原断言，以真实回归验证两种停止边界。
- 取消在许可已经获准后才到达，仍由下一个真实检查观察。本项不要求回溯取消已经获准的许可，也不应把该场景误当成本次分类缺陷。
- emitter 的额度停止保留原写入进度口径，shared ledger first-stop 的选择由既有操作报告负责；不在本片重写 CLI 导出错误策略。
- 共享 charge 的 deadline 分类会纠正其它 Java 计费调用的相同误报；使用零 elapsed 的真实回归及既有正常计费、取消与库测试核验必要传播边界，不更改错误类型或预算数字。

## Migration Plan

进入共享串行窗口后，先运行旧实现上的取消复现并保存失败，再应用单一 emitter 修复，运行受影响回归、当前 CI 检查与报告入口停止检查。按独立 OpenSpec 切片提交；不夹入普通 Try/List 或前向 carry 的源码交付。
