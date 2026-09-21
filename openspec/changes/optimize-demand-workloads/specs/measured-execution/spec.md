## ADDED Requirements

### Requirement: Optimization investigations have explicit scope and disposition

性能专项 SHALL 为每组方案保留可审查的调查档案，包含目标工作负载、必要工作与重复工作、固定证据、收益与代价假设、备选方案、受影响行为契约、实验、回退和最终处置。调查 MUST 覆盖容器访问、操作内 class/方法复用、按需查询/续扫、批处理生命周期、分层缓存、预热/预取、并行/相同计算合并以及字节/摘要/输出路径；未经测量或不存在目标需求的项 SHALL 明确标记证据缺口，不得伪装成已实现或已验证收益。

#### Scenario: A plausible optimization has no workload evidence

- **WHEN** 某方案在理论上能减少等待，但尚无实际重复访问或并发工作负载
- **THEN** 调查记录缺失证据、暂缓原因和重新进入条件，不默认实施、不填写虚构命中率或加速比

#### Scenario: An implementation is admitted

- **WHEN** 调查决定进入会改变产品行为的优化实现
- **THEN** 记录负责该方案的独立 change、受影响 specs、验证与回退范围；总专项和其它子项不重复拥有同一实现

#### Scenario: Full export has an explicit parallel workload

- **WHEN** 调用方已声明全量导出及多 worker 需求
- **THEN** 调查将其登记为具体工作负载，指定批量/并行实现的唯一子 change；其它宿主的跨请求合并缺少需求时可以单独暂缓，不能把已声明需求也标成缺失或把完成规划标为交付

#### Scenario: Investigation closure differs from feature completion

- **WHEN** 所有方案已完成调查，但部分被否决或暂缓
- **THEN** 交付记录分别列出调查结论、实际交付和未交付能力；已准入但未完成的实现不得计为完成

### Requirement: Workload measurements preserve end-to-end boundaries

评测 SHALL 区分冷单次、同 snapshot 多 class、同 class 多方法、查询与分页、超过 cache 容量的扫描、已明确的全量并行导出，以及有独立需求时的跨请求并发宿主。每组 MUST 记录进程/快照/请求生命周期、工作单位、目标顺序、预算、cache 初态和主指标，并分别给出准备/预热、请求或首次结果、完整序列总成本。全量导出 MUST 包含打开、发现、准备、计算、编码和输出关闭，不以逐方法 p50 替代总耗时。不得把库内请求、CLI 进程、不同输出单位或不同语义配置的数字混为同口径结论。

#### Scenario: Warm navigation has nonzero preparation cost

- **WHEN** 多方法导航在已打开 snapshot 和已填充 store 上测得较低请求延迟
- **THEN** 报告打开、发现和填充成本及全序列时长，warm latency 不被直接报告成冷启动或完整序列加速

#### Scenario: A one-shot scan exceeds retention capacity

- **WHEN** 工作负载只访问每个 class 一次且数据超过 cache 容量
- **THEN** 单独报告总时间、容量拒绝/淘汰/重建和内存代价，不用同方法反复命中的结果代替该工作负载

#### Scenario: CLI continuation starts a new process

- **WHEN** 每页查询都由独立 CLI 进程执行
- **THEN** 全序列成本包含重复打开和准备，不能声称复用了上一个进程的内存缓存

### Requirement: Amdahl estimates use attributable time and declared assumptions

优化优先级记录 SHALL 使用同一目标工作负载的互斥阶段时间占比、局部加速和新增开销，或明确记录缺失项。阿姆达尔估计 SHALL 区分理想上界、带假设预测和实测端到端结果；目录条目、读取字节、命中率与分析步数 MUST NOT 直接替代墙钟占比。重叠阶段和多项优化不得重复计入收益，后续决策 MUST 基于前一阶段完成后的剩余成本。

#### Scenario: Work counts prove duplication but not wall-time share

- **WHEN** 记录表明一次请求多次枚举同一目录，但没有阶段计时
- **THEN** 可以据此准入访问范围或重复工作修正，耗时占比与整体倍数仍标为未测

#### Scenario: A small fraction has a bounded ideal benefit

- **WHEN** 目标部分互斥耗时占总时长 5%，且估计假设完全消除该部分、零新增开销
- **THEN** 记录的理想整体加速上界约为 1.053 倍，并标注假设，不将局部无限加速表达为整体无限加速

#### Scenario: Several changes affect the same cost

- **WHEN** 定向访问与 cache 都减少原来的目录遍历成本
- **THEN** 通过分阶段和组合消融归因，不相加重复占比或相乘独立测得的整体加速比

### Requirement: Optimization experiments separate algorithm and policy effects

实验 SHALL 固定基线与候选构建、输入及语义配置，分开比较算法直接路径、显式保留的冷/热路径和容量/失效退化路径。与目标无关的 cache 产品、并发和输出策略 MUST 保持一致或作为独立因素报告。实验 SHALL 保存原始样本、预先声明的统计方法及退化/资源界限，并校准仪表开销；无法区分的差异必须标为收益未证实。

#### Scenario: Container and class facts caching are enabled together

- **WHEN** 实验同时减少目录读取和 CP/Header parse，工具尚不能独立控制两个因素
- **THEN** 结果只能标为组合收益，容器单项归因保持缺失，不能通过该项归因验收

#### Scenario: Instrumentation changes measured cost

- **WHEN** 阶段计时自身产生可见开销，或父子阶段计时重叠
- **THEN** 报告校准与未归因部分，采用互斥或扣除子阶段后的时间，不能将重叠时间用于占比推导

#### Scenario: The candidate does not separate from baseline variation

- **WHEN** 按预先声明的方法重复测量后，候选收益仍无法与波动区分
- **THEN** 保留所有有效样本并报告未证实；不事后挑选样本或改阈值宣称成功，启用与继续投入按调查门禁重新决定

### Requirement: Reuse and lazy execution admission preserve evidence boundaries

复用、索引、lazy 执行或续扫方案进入实现前，验收计划 MUST 覆盖 snapshot/物理 origin、原始名字、解析策略、实际语义依赖、完整性、确定性顺序、coverage、预算和取消。局部性或结果生产时机变化引起可观察语义变化时 SHALL 先由实施 change 声明，不能通过删除诊断/coverage、合并重复来源或减少验证来获得透明性结论。

#### Scenario: A lazy page stops before a malformed suffix

- **WHEN** 可停止查询未访问后缀，而原始实现曾在返回该页前访问后缀
- **THEN** 验收明确比较观察范围，未扫描范围如实保留；page、coverage 与 execution 按其各自契约报告，`page.has_more` 不等于搜索完成，不把页满自动等同取消、预算耗尽或完整无命中；完整扫描且确实无命中的空结果仍然合法

#### Scenario: A candidate index cannot prove absence

- **WHEN** 索引或预过滤无法安全排除目标，或目录仅构造出不完整前缀
- **THEN** 验收要求保留候选/未决范围并继续必要验证或真实停止，不允许跳过后报告完整 Missing/NoMatch

#### Scenario: A cached product depends on a changed environment

- **WHEN** 类名相同但物理内容、loader/view、依赖或适用规则发生影响该产品的变化
- **THEN** 验收证明旧项不代答新身份；纯物理事实不因无关配置扩大 key，环境相关结论也不因省略依赖而复用

### Requirement: Speculation and concurrency require lifecycle-specific evidence

预热/预取、批处理、并行或相同计算合并方案进入实现前 SHALL 指定独立的工作/保留/在途资源边界、输出顺序、排队与取消行为。相同计算合并 MUST 明确产品 key 的身份及实际语义依赖，并保持每个订阅者的预算、coverage、结果过滤和取消状态独立，不以共享整份报告替代共享事实。评测 MUST 包含准备及放弃成本、资源竞争、容量退化和部分输出，不得仅以 warm latency 或吞吐证明全部用户场景获益。取消隔离、失败重试及最后需求释放必须有可复现验证。

#### Scenario: Prefetched data is never used

- **WHEN** 宿主提前准备后用户改变目标或关闭输入
- **THEN** 统计未使用工作及驻留代价，验证可停止/释放，且不扩大前台请求的已扫描 coverage

#### Scenario: One shared-work consumer cancels

- **WHEN** 多个请求等待同一事实，其中一个取消而其它仍有合法需求
- **THEN** 取消者终止，其它请求仍按各自限制完成或真实失败；共享生产者失败不得使 key 永久挂起或污染后续请求

#### Scenario: Batch or parallel execution reaches a total limit

- **WHEN** 某些方法或 worker 已消耗额度且总量到达请求上限
- **THEN** 验证停止与已发布前缀，不通过更换方法、worker 或 fallback 重置总预算，不因完成顺序改变已声明的确定性契约

### Requirement: Performance evidence keeps distinct determinism checks

专项 SHALL 分别保存同执行配置的确定性对照、跨策略的语义对照及外部性能记录。既有串行及批量 workers=1 在未触发 elapsed 截止或外部取消时，同配置完整领域报告对照 MUST 继续只排除 `elapsed_millis`，保留预算计数；配置包含 cache 产品、容量及初态。新增并行路径 MUST 先由实施 spec 明确完整语义与资源归属的确定性边界，并保留独立资源总账；不得把语义 fingerprint 冒充原始报告字节一致。跨策略足额完整结果的语义对照只排除实际工作 usage、cache 状态/计数和耗时，MUST 保留身份、顺序、语义诊断、coverage、规则及输出证据。harness 阶段时间不得写入领域报告以规避该边界。

#### Scenario: Warm and direct paths spend different work

- **WHEN** 两个策略足额完成且 warm 实际少读了数据
- **THEN** 跨策略语义对照允许真实 usage 不同；两个相同 warm 初态的串行完整确定性对照仍核对预算计数，不能用语义子集替代

#### Scenario: Parallel admission changes resource ownership

- **WHEN** 新并行路径的 cache 准入次序或总额度竞争随调度发生变化
- **THEN** 按其已声明的契约保留足额完整语义、实际 usage 及中止范围；受控调度扰动必须证明总预算不突破、已执行未交付工作不丢失，差异白名单不能移除语义诊断或来源证据

#### Scenario: A deadline terminates at different physical points

- **WHEN** 真实墙钟截止在不同重复中发生于不同计算位置
- **THEN** 将其作为取消/截止响应和真实前缀的实验，另用受控停止点验证终止语义，不从完整确定性对照中任意删除结果字段

### Requirement: Optimization rollback cannot restart terminated work

每个准入方案 SHALL 提供可验证的实验撤销、运行期降级与发布回退路径。可选 cache/索引失效或容量不足时，只能在同一 snapshot、同一语义和剩余预算内转直接路径；取消或预算耗尽 MUST 保持终止。已发布前缀不得因回退自动重复发布。语义、身份、预算或确定性回归 MUST 阻止启用，不能以性能收益抵消。

#### Scenario: Optional acceleration fails with budget remaining

- **WHEN** 可选事实失效/损坏且直接路径仍有可用预算
- **THEN** 丢弃对应可选状态，有界回退并报告实际工作；不丢弃当前仍有效的已取得事实，不重复计成一次成功命中

#### Scenario: An exhausted request encounters an available fallback

- **WHEN** 请求已经终止，但禁用 cache 或改用串行可能重新开始计算
- **THEN** 返回实际停止与已处理范围，不重新创建预算或自动重跑

#### Scenario: A release candidate violates evidence or resource limits

- **WHEN** 候选更快但改变物理身份/语义结果，或超过预先声明的资源/退化界限
- **THEN** 阻止该策略启用或回退其独立改动，保留其它已验证正确性修复和可复现失败证据

### Requirement: Optimization programs close with auditable decisions

专项 SHALL 为每项调查和每个准入实现分别记录状态、证据、收益/代价及后续进入条件；每阶段后重测剩余成本。调查否决/暂缓不等于功能交付，工作计数下降不等于已证明耗时改善；尚未完成的准入子项不得在专项归档时隐藏。总专项不自动启用新策略或自动实施未准入候选。

#### Scenario: The required locality correction passes but timing benefit is unproven

- **WHEN** 不必要读取已消除、行为与资源门禁通过，但端到端耗时差异未超过测量波动
- **THEN** 交付记录分别说明访问范围修正和未证实的速度收益，保持未获启用证据的策略关闭，不编造性能成功阈值

#### Scenario: The previous optimization changes the dominant cost

- **WHEN** 一个子项完成后原热点占比下降
- **THEN** 重新排列剩余调查优先级，必要时停止后续策略，不继续沿用原热点的占比和收益预估

#### Scenario: A delivered child change does not finish the investigation program

- **WHEN** 某独立子 change 已归档并通过其正确性和工作计数门禁，但专项工作负载、原始样本、归因或其余调查处置尚未完成
- **THEN** 记录该子项已交付与专项仍未完成，不重复实施子项，也不把归档、格式验证或工作计数下降当成整个专项完成

#### Scenario: A frozen candidate has an independently reproduced correctness gap

- **WHEN** 已冻结候选在声明的目标工作负载中被反例证明会丢失搜索停止证据、误报完整结果，或输出类型非法/行为不等价的代码
- **THEN** 保留该提交作为注明缺口的历史比较臂，由独立正确性 change 修复并验收后重新冻结当前候选；性能专项不得降低行为契约或删除失败样本以通过准入


### Requirement: Agent workloads remain demand scoped across preparation and delivery

性能专项 SHALL 将核心库的导航、选定方法、查询首屏/续页及证据追问作为独立于全量导出的工作负载。核心 crate SHALL 维持按请求范围读取/计算的边界，不得通过全量恢复再过滤来实现普通单点请求。候选设计 SHALL 区分必要分析、证据构造、序列化投影与跨请求保留，复用已验证事实不能依赖可选 cache 恰好准入。默认策略的决策 MUST 同时验证继续与放弃、首条结果、实际工作、资源和真实停止。

#### Scenario: An agent requests members and then one method body

- **WHEN** 同一 snapshot 上先列举成员，再请求一个确定物理方法的正文
- **THEN** 成员步骤不恢复 Body；后续只执行所选方法及既有恢复规则实际要求的依赖，能够复用的 prepared 实现不分叉，未选择的其它方法不因性能策略被全部恢复

#### Scenario: Compact delivery and full evidence have different cost

- **WHEN** 相同请求提供必要结果及显式完整证据视图
- **THEN** 两者分别记录实际序列化与必要分析成本，最小结果保留身份、四平面和真实停止；较少输出字节不得被宣传为同产出恢复算法加速

#### Scenario: Product implementation has one owner

- **WHEN** 普通 prepared 交接、增量查询或可选证据产品进入实现
- **THEN** 由 add-demand-driven-core-results 的产品契约与任务验收，本专项记录归因和处置；bulk 的 worker/总账/窗口仍由原 change 负责，不重复实施或重复计完成
