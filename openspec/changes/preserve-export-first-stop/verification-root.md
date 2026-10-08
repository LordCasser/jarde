# Root 独立验收

候选生产修复 `7a296303`。修复前 root 对 c42275ff 跑完整 all-targets/all-features、seed 5350648285461741569、no-fail-fast，唯一失败目标为 export_cli。原错误全文见 [before-fix.txt](results/root/before-fix.txt)：ir_items=1 的操作返回 Cancelled，失败文档缺 cli_export_unfinished，与此前五次 CI 的诊断症状一致。

根因链已逐段核对：ledger::record 先保存首次 Budget/IrItems 再 cancel token；coordinator 已获 ResultItems 许可，sink 的 OutputBytes charge 随后在 checkpoint 看到取消。Stream 记为 Allowance，原 export::run 无条件将这次生 Cancelled 当最终原因。库 report 的首次记录始终正确，没有调度或预算计费错误。

修复复用 Stopped::error 与 report.stop：文件 I/O 保持优先；只有 BudgetExceeded/OutputBytes 且首停为 Budget/OutputBytes/Delivery 时保留该错误的完整数字，其他回调停止由现有 unfinished_error 引用原报告。既不更改 ledger 也不重新执行，不放宽 ir_items 集成断言。5 个确定性单元测试分别覆盖次生取消、外部取消、晚到输出预算、真实输出预算与 I/O。

共同门禁结果记录在 [构造片 gates-summary](../recover-functional-constructor-arguments/results/root/gates-summary.txt)。root 验收还要求 export_cli 全族原有 output_bytes 数字及前缀断言通过，以及原紧/宽 ir_items 用例重复回归；不能只靠某一次随机调度绿灯认定根因修复。

最终本地验收：5 个确定性单测与 export_cli 全族（11 项）在两个固定 seed 均通过；原紧/宽 ir_items 集成精确重复 20/20。完整失败前后与重复凭证见 results/root。首轮/次轮各 3,224 个测试通过，fmt/clippy、ignored P3/构造整类/绑定引用整类与 strict OpenSpec 均通过。远端 CI 状态按 handoff 的命令核对。
