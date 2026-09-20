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

### Requirement: Counting dimensions are read within one request shape

每个计数字段 SHALL 被读作「本次请求在其声明范围与请求形状下实际做了多少工作」的代理，MUST NOT 被当作跨请求形状可比的单位。以 `archive_entries` 为例，standalone class、flat jar 的根目录枚举、WAR 中单个 container、以及 active container tree 的整树遍历具有不同 scope——实测两个 flat jar 为 28 与 29、一个大 flat jar 约 6,009、一个 WAR 为 500–1,296、另一个为 3,702，且 container tree root 会走子容器。比较行 MUST 声明请求形状、声明的 roots/profile（含 multi-release/layout 策略）与引擎版本，并明确该 scope 下这个数「数的是什么」；MUST NOT 跨形状直接比较计数，也 MUST NOT 用未声明形状的数字得出收益或回归结论。

#### Scenario: Counter rows state their shape

- **WHEN** 报告或文档并列 `archive_entries` 等计数
- **THEN** 每一行 MUST 声明请求形状、范围与声明（roots/profile），不同形状的行 MUST 标注不可直接比较；缺失这些声明的数字 MUST NOT 作为结论进入发布材料

#### Scenario: A tree traversal counts what it walked

- **WHEN** 请求从 active container tree root 出发并走到子容器
- **THEN** 报告 MUST 说明该计数包含被接受的子容器条目，与只声明单个 container 的请求并列时 MUST 标注范围不同；不得让读者按同一单位相减

#### Scenario: A counter is a work proxy, not a claim about time or memory

- **WHEN** 报告给出计数字段
- **THEN** 该数字 MUST 保持「实际工作代理」的含义，MUST NOT 被表述为墙钟占比、RSS 或收益本身；时间与内存结论仍按既有测量协议单独测量
