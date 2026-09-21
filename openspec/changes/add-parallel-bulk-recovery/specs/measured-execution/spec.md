## MODIFIED Requirements

### Requirement: Semantic-preserving scheduling

调度、合并和并行 SHALL 保持 snapshot/view identity、origin 顺序、coverage、Unknown/Partial、预算和取消语义；消费者取消不得无条件取消仍被其他订阅者使用的工作。

足额完整的批量恢复 SHALL 保持逐方法语义内容与物理发布顺序。并行预算/取消中止时，实际完成子集可受调度影响；MUST 如实报告执行孔洞、交付前缀、实际 usage 和停止原因，不承诺相同部分集合。这个例外只适用于显式并行操作的执行范围，不允许完整执行时遗漏或改变语义结果。

#### Scenario: Cancelled shared query

- **WHEN** 一个订阅者取消而另一个订阅者继续等待相同 single-flight 请求
- **THEN** 保留请求工作给仍在等待的订阅者，取消者只得到自己的终止状态，结果不混合不同 snapshot（验收 A14/A15）

#### Scenario: Reordered parallel results

- **WHEN** 多 worker 以不同完成顺序足额完成同一范围的扫描
- **THEN** 分页、evidence 和 aggregate coverage 以稳定排序发布，不因并行顺序改变结果（验收 A15）

#### Scenario: Parallel stop does not claim a serial prefix of work

- **WHEN** 同一并行工作负载在不同调度下中止
- **THEN** 分别核对实际执行和交付覆盖及资源总账；交付有序不等于执行没有孔洞，不能用相同的已交付排序伪造同一执行过程（B04/B07）

### Requirement: Optimization comparison preserves full determinism checks

评测 SHALL 区分同执行配置的完整报告确定性 fingerprint 与跨加速路径的语义 fingerprint；执行配置 MUST 包含 cache policy、容量、初始预热状态以及 worker 数和调度模式。既有串行入口及新批量单 worker 的完整报告重复对照 SHALL 继续只排除 elapsed_millis，保留其它字段及预算计数；跨 direct/cold/warm 的语义对照仅排除实际工作 usage、cache 状态/计数和耗时。两类记录 MUST 分别命名，不能用语义子集通过替代完整报告的确定性门禁。

新批量多 worker 另有明确的 parallel 记录：足额完整时 SHALL 对展开共享证据后的逐方法语义、顺序及覆盖做全量对照；实际 usage、cache 计数、worker 配置及耗时进入独立的资源记录。资源归属可能随共享准入/调度变化，不承诺这些字段逐字节相同，但 MUST 检查全局额度、归属求和及差异来源。对照允许排除的路径须固定白名单，不能排除正文、身份、来源、规则/拒绝、语义诊断或语义覆盖。此项不得放宽旧串行门禁，也不允许把非完整运行混入完整语义对照。

#### Scenario: Identically warmed repeats

- **WHEN** 两个独立串行运行使用相同输入/profile/limits，并从相同配置及预热步骤建立相同 cache 初始状态
- **THEN** 完整报告除 elapsed_millis 外逐字段一致，预算计数差异必须作为确定性问题暴露；另保留与 direct 路径的语义对照

#### Scenario: Parallel payload and accounting are checked separately

- **WHEN** 以 1/2/4/6 worker 完整导出同一清单
- **THEN** 逐方法语义 fingerprint 一致；资源记录按实际计数独立验证守恒与上限；更高拒绝率、缺失方法或仅解释占比上升不能当作加速（B07/B10）

## ADDED Requirements

### Requirement: Bulk performance includes the complete export lifecycle

全量性能 SHALL 以固定输入清单的冷端到端总时间为主指标，包含启动/open、发现、准备、恢复、编码、全部写出及最终 flush/close；同时记录总 CPU、峰值 RSS、首次结果、方法延迟分布和产出分类。cache 初态、容量、worker、OS page cache 状态及 sink 条件 MUST 登记。串行直接/共享 store/类内复用/并行 SHALL 单因素与组合对照；每配置至少十个独立交错重复。请求 p50 不得代替总时间除以完成工作量，analysis_steps 不变不得作为分析耗时不变的证明。

#### Scenario: A warm median appears competitive

- **WHEN** retaining store 的方法请求 p50 接近另一个工具的整包总时间除以方法数
- **THEN** 仅报告该延迟观察，直到实际测得同范围冷导出总时间才声明吞吐收益；不把 p50 乘数量当作实测总账（B10）

#### Scenario: A comparison with jadx is published

- **WHEN** 发布接近或超过 jadx 的结论
- **THEN** 保存两侧工具/构建/JVM/线程配置和完整命令，核对同一物理类/方法集合、构造器/内联差异、nested/resources 范围、输出单位及真实产出质量；无法等价的整类源码与方法产物流单列产品场景，不声明无条件等价或胜出；完整运行的重复总时间及区间支持该结论（B10）

#### Scenario: A candidate saves time by doing less recovery

- **WHEN** 候选总时间下降但有更多失败、提前停止、遗漏或 explanation_only
- **THEN** 正确性/覆盖门禁拒绝将其认定为性能收益，独立登记能力变化；性能测量不修改正文来补齐质量（B10）
