## Why

完整 workspace 门禁复现 export_cli 的既有并发失败：worker 首先耗尽 ir_items 并记录 Budget/IrItems，取消 token 唤醒其他执行者；CLI 输出额度回调随后得到 Cancelled，却将这个次生错误作为最终失败文档，遮住报告中正确的首次停止原因。交接记录中的五次 CI flake 因此已有生产根因，不能放宽断言。

## What Changes

- 导出失败原因以操作报告的 first stop 为准；真实文件 I/O 失败仍优先。
- 仅当回调自身输出预算错误与首次 Delivery/OutputBytes 预算停止相符时保留该错误的 limit/consumed/requested。
- 冻结次生取消、外部取消、实际输出预算和输出 I/O 的确定性回归；保留 ir_items 紧/宽预算集成测试。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `measured-execution`: 导出适配层不得覆盖操作的首次停止原因。

## Impact

仅 jarde-cli export 的失败文档选择与测试；复用现有 ledger/report，无新状态机、库 API、依赖或重试。先决条件是既有 first-writer ledger；不改调度、预算数字、源码恢复或其他架构债务。
