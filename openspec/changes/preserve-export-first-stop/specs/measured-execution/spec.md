## ADDED Requirements

### Requirement: 导出保留首次停止原因

导出适配层 SHALL 保留操作报告记录的首次停止原因，MUST NOT 将唤醒导致的次生取消误报为原始停止。真实输出文件错误 SHALL 保留其失败优先级。

#### Scenario: 内部预算停止后输出回调看到取消
- **WHEN** worker 首先耗尽 ir_items 并取消共享操作，输出回调随后因该取消拒绝输出额度
- **THEN** 导出 SHALL 返回未完成状态，失败文档 SHALL 包含原始 ir_items 首次预算停止；没有交付 final 的文件 SHALL 仍为已确认前缀

#### Scenario: 真正的输出额度拒绝
- **WHEN** 首次停止为 Delivery 的 OutputBytes 预算拒绝，且输出回调记录该拒绝
- **THEN** 导出 SHALL 保留该错误的维度、limit、consumed、requested；未获许可记录 MUST NOT 写入文件

#### Scenario: 外部取消与文件失败
- **WHEN** 外部取消是首次停止，或输出文件写入关闭失败
- **THEN** 外部取消 SHALL 保持取消语义且不得改称预算耗尽；真实文件错误 SHALL 保持失败状态及实际 I/O 原因
