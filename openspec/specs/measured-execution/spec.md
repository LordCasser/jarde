# measured-execution Specification

## Purpose
让优化决策建立在可复现的真实成本数据上，并在冷/热、局部/全范围、正常/退化输入和取消场景下保持已有分析结果语义。

## Requirements

### Requirement: Reproducible measurement protocol

系统 SHALL 为 benchmark 记录输入 fixture fingerprint、snapshot/view/profile、registry/recovery 配置、query scope、预算、并发、缓存状态、耗时、读取字节、物化范围、内存代理和结果摘要。优化 MUST 先有基线和可重复对照。

#### Scenario: Cold and warm comparison

- **WHEN** 同一 snapshot/view/query 分别以关闭和开启缓存执行
- **THEN** 报告冷/热测量上下文和结果 fingerprint；性能差异不能隐藏输入或语义配置变化（验收 A15）

#### Scenario: Local versus full-range query

- **WHEN** 单方法请求和全范围 XRef 请求在相同语料上运行
- **THEN** 分别报告物化类/方法/字节范围，局部请求不得因优化预热而读取全局 Body（验收 A16）

### Requirement: Semantic-preserving scheduling

调度、合并和并行 SHALL 保持 snapshot/view identity、origin 顺序、coverage、Unknown/Partial、预算和取消语义；消费者取消不得无条件取消仍被其他订阅者使用的工作。

#### Scenario: Cancelled shared query

- **WHEN** 一个订阅者取消而另一个订阅者继续等待相同 single-flight 请求
- **THEN** 保留请求工作给仍在等待的订阅者，取消者只得到自己的终止状态，结果不混合不同 snapshot（验收 A14/A15）

#### Scenario: Reordered parallel results

- **WHEN** 多 worker 以不同完成顺序扫描同一范围
- **THEN** 分页、evidence 和 aggregate coverage 以稳定排序发布，不因并行顺序改变结果（验收 A15）
