## ADDED Requirements

### Requirement: Repeated request optimization has attributable measurements

重复请求优化 SHALL 在固定输入和候选提交上记录 direct、cold、warm、容量不足的对照，区分同方法重复与同 snapshot 多方法。报告 MUST 分开记录实际目录解析、nested 物化、读取/展开字节、保留内存权重、请求 Budget 总量及墙钟分布，并携带结果 fingerprint 与环境配置。工作量计数不得直接解释成墙钟占比，retained weight 不得标成 RSS。

#### Scenario: Warm multi-method workload

- **WHEN** 同一 snapshot 连续请求多个方法，目标 container 的完整 facts 在预算容量内一直保留
- **THEN** 测量能验证首次构造与后续复用的区别，后续没有该 container 的目录重建或父容器重复物化，并对完整结果比较语义 fingerprint

#### Scenario: Constrained budget comparison

- **WHEN** 同一低预算下 direct 未完成而 warm 完成
- **THEN** 报告实际 usage、复用状态与两种完成程度，不伪造相同 usage，也不把该差别计成足额完整结果的语义不一致

#### Scenario: Performance claim remains bounded

- **WHEN** 比较含机器负载波动、不同输出单位或未测缓存路径的结果
- **THEN** 报告这些限制，只有固定并记录基线/候选构建、同工作单位与语义配置、隔离待比较因素且超出重复测量波动的证据才能支持相应耗时收益；同一候选的 cold/warm 比较不混入代码变更，不由当前基线发布固定加速倍数或延迟阈值

### Requirement: Optimization comparison preserves full determinism checks

评测 SHALL 区分同执行配置的完整报告确定性 fingerprint 与跨加速路径的语义 fingerprint；执行配置 MUST 包含 cache policy、容量及初始预热状态。完整报告的重复对照 SHALL 继续只排除 elapsed_millis，保留其它字段及预算计数；跨 direct/cold/warm 的语义对照仅排除实际工作 usage、cache 状态/计数和耗时。两类记录 MUST 分别命名，不能用语义子集通过替代完整报告的确定性门禁。

#### Scenario: Identically warmed repeats

- **WHEN** 两个独立运行使用相同输入/profile/limits，并从相同配置及预热步骤建立相同 cache 初始状态
- **THEN** 完整报告除 elapsed_millis 外逐字段一致，预算计数差异必须作为确定性问题暴露；另保留与 direct 路径的语义对照
