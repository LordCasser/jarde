## Purpose

把性能、资源、取消和正确性对照变成可审查的发布门槛，避免用未固定的数字或单一样本宣称引擎已经优化或普遍更快。

## ADDED Requirements

### Requirement: Evidence-based optimization acceptance

每项优化 SHALL 关联基线、代表性语料、测量方法、结果置信范围/重复策略和回归样本；若收益不稳定或正确性对照失败，项目 MUST 保留未启用状态并记录原因。

#### Scenario: Optimization regresses a corner case

- **WHEN** 优化降低平均耗时但在 ZIP bomb、condy 图、不可约 CFG 或缺失依赖样本改变状态/资源边界
- **THEN** 优化不能进入默认路径，报告回归并保留安全 fallback（验收 A13/A14/A18）

#### Scenario: No universal threshold yet

- **WHEN** 代表性语料不足以支持固定 P95、内存或吞吐阈值
- **THEN** 发布记录只声明测得范围和未决门槛，不制造预设性能承诺

### Requirement: Correctness and resource regression gates

发布前 SHALL 对优化开关与关闭状态比较 evidence、coverage、representation、diagnostics、snapshot/view identity、峰值内存代理、取消和预算结果；任一语义差异 MUST 阻断默认启用。

#### Scenario: Optimized versus direct path

- **WHEN** 相同配置分别执行直接、cache、index、parallel 或 merged query path
- **THEN** 在允许的稳定排序差异之外结果等价，被选目标阶段的适用验收集合和共享不变量回归必须通过；若优化跨阶段，再覆盖所有受影响阶段

#### Scenario: Cold and warm results

- **WHEN** 相同完整请求分别走冷路径和热路径
- **THEN** evidence、coverage、representation、diagnostics 和结果 identity 一致；预算耗尽或取消的部分子集允许受调度影响，但必须报告实际已处理范围与终止原因

#### Scenario: Cancellation under pressure

- **WHEN** 并发、解压或 IR 工作达到预算时收到取消
- **THEN** 返回已扫描范围和终止原因，不 hang、不把 Partial 标为 Complete（验收 A14/A18）
