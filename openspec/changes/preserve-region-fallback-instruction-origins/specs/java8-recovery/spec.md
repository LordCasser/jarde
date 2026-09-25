## ADDED Requirements

### Requirement: Region 拒绝的物理指令来源

当 Java 8 恢复在 Region 阶段拒绝一个或多个 canonical 块时，恢复报告 SHALL 为可证明属于拒绝范围且已解码的每条物理指令保留来源。报告 MUST NOT 用块起点替代整块指令，也 MUST NOT 为未解码或不属于该范围的字节虚构来源。拒绝代码、正文语义和停止状态仍由现有恢复合同决定。

#### Scenario: 异常表跨越后置更新链
- **WHEN** 合法 `PostfixHandlerBoundary.update()` 在 BCI 7 开始保护，Region 提前拒绝包含 BCI 0–12 的候选块及 BCI 13 的 handler 块
- **THEN** 报告 SHALL 保留这些块内实际已解码指令的来源，包括 BCI 3、6–12，并保留 BCI 0、13；不得改写成错误的单一 `try` 后置表达式

#### Scenario: 融合、克隆与部分解码
- **WHEN** 一个被拒 canonical 节点代表多个原始块、`jsr` 克隆路径，或方法只解码了可信前缀
- **THEN** 报告 SHALL 根据实际块归属列出可信指令起点、稳定去重；不得以连续数字区间或不同路径的指令推测缺失来源

#### Scenario: 来源预算或取消
- **WHEN** 逐指令来源采集在预算、取消或输出限制处停止
- **THEN** 结果 SHALL 遵守现有受限产物契约，不把未完成的来源报告标为完整，默认与完整来源模式的正文 SHALL 相同
