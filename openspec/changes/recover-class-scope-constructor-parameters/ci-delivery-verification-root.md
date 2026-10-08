# 主线 CI — output 预算并发验收修正

`698a219e` 的远端 [run 37804470415](https://github.com/LordCasser/jarde/actions/runs/37804470415) 中，fmt/clippy、第一固定 seed、MSRV、fuzz smoke 与 supply chain 通过；第二 seed 在 `bulk_recovery_delivery::one_declaration_bounds_the_librarys_own_presentation_too` 的四 worker 分支失败，方法 stop 集合为空。失败原文保存在 `results/local-gates/remote-ci-delivery-failure.log`。后续 JDK25 oracle/P3/OpenSpec 等步骤当次未执行，不能据此报绿。

root 与 Luna 独立核对 emitter、Budget、OperationLedger、bulk closing 与原 OpenSpec。`emit::write` 先 check，再 charge；check 不预留、不记操作停止。多个 worker 可同时探测成功，随后一个原子 charge 拒绝，账本准确记录 `Methods/Budget/OutputBytes` 并取消派发。该结果可能在方法交付之前关闭流，方法级 stop 因而未必交付。单 worker 则可以先探测到余额不足，在当前方法丢弃产物并继续流。

原设计第 4 节明确全局额度耗尽关闭派发，bulk spec 的预算条款和汇总条款允许并行中止集合受调度影响。旧测试把单线程的探测拒绝当成所有调度的唯一结果，与实际扣费合同不符。没有为此新增不记录 operation stop 的准入 API，也没有改变生产预算或语法恢复逻辑。

修改仅涉及两个测试文件。bulk 用例分别严格验证局部拒绝与全局扣费拒绝：总额不越限、四类归属求和、停止维度与 owner、完整有序流或真实有序前缀、Partial 汇总、最终记录是否实际交付。p1 用两份 Methods budget 先同时探测同一字节、再依次扣费，确定性确认探测不预留、第二次扣费被原子拒绝、拒绝未计费、取消与首停止归属保持准确。未移除拒绝路径或单 worker 的全部交付要求。

本地复验：`cargo test --test bulk_recovery_delivery --test p1_budget_ledger --all-features --locked` 3 + 18 项通过；最终 all-features 二进制的预算用例连续 30 次通过；fmt、两个修改目标的 CI 同口径 clippy 与 strict OpenSpec 321/321 通过。使用主仓共享 target，单 Cargo worker/无 incremental。日志在 `results/local-gates/ci-delivery-*.log`。远端全工作区两 seed 与实际 JDK25 oracle 仍须在最新 HEAD 验收。

修正提交为 `d6598866`，完整远端门禁见 [run 37808909595](https://github.com/LordCasser/jarde/actions/runs/37808909595)。根 handoff 提供最新 main HEAD 查询命令；后续文档提交只更新交接记录，不能用旧失败 run 代替当前状态。
