## Purpose

使核心库按调用方当前问题交付必要事实与可定位的判断边界，并将计算依赖、证据明细与保留生命周期分开。小范围读取、结构查询、方法恢复和证据追问共享可信事实，但不要求预先完成更大范围或更强分析，也不把省略的信息误报为不存在。

## ADDED Requirements

### Requirement: Requested facts determine the necessary computation

核心请求 SHALL 分别表达目标范围、事实类别或分析阶段，以及需要交付的详细证据。系统 MUST 只执行取得所请求事实所必需的计算与验证依赖，不以默认全量分析后过滤实现局部请求。读取容器目录、解压完整压缩 entry、检查 class 布局等物理必要工作 SHALL 如实计费，不承诺请求一个事实只读取对应几个字节。所选事实的计算依赖 MUST NOT 因省略输出证据而删除。

#### Scenario: Members without bodies

- **WHEN** 请求某类的声明和成员，未选择任何方法体
- **THEN** 方法体指令解码、方法 IR 和 Java 恢复的构造次数均为零，成员表自身的读取与不完整性仍真实报告

#### Scenario: Invocation sites without recovery

- **WHEN** 请求结构调用位置及其符号目标
- **THEN** 只执行对应消费者与必要读取验证，不构造 resolver、CFG、SSA、Region 或 Java AST；调用指令不被升级为已解析声明或运行时必然执行

#### Scenario: Source text still requires semantic facts

- **WHEN** 请求方法源码而不请求详细分析或规则证据
- **THEN** 恢复所需的类型、控制流、值流、effect 和规则前提仍被计算与检查，省略的仅为未请求的证据产品，不能放宽恢复接受集

#### Scenario: Unsupported fact is not substituted

- **WHEN** 请求当前入口不支持的分析或证据类别
- **THEN** 明确报告不支持，不以结构引用、空集合或较弱分析冒充所请求答案，也不隐式启动其它入口的全局工作

### Requirement: Every result preserves the facts needed to interpret it

任何证据选择下，结果 SHALL 保留准确目标身份、请求含义与生效选择、实际产物、适用的 representation/content/quality/validation 平面、覆盖范围、执行状态，以及影响答案的缺失、歧义、拒绝和停止。缺口 SHALL 至少携带版本范围内的原因标识、受影响方法与可知位置，以及已经确定的缺失条件。完整配置与实际使用量 MUST 遵守既有核心报告契约。重复正文或相同上下文可以通过所属结果结构共享，但 MUST 不丢失关联或把不同物理来源合并。

#### Scenario: An explanation-only artifact

- **WHEN** 恢复只交付解释而没有语句
- **THEN** 默认结果仍明确为 ExplanationOnly，包含原因与相关位置，不以 produced 或执行完成推导可用源码

#### Scenario: Unrequested is not an empty finding

- **WHEN** 某类证据没有请求，另一类证据完整检查后没有记录
- **THEN** 前者为明确的 NotRequested，后者为 Complete 的空结果；未开始、请求后中止与不支持同样不能伪装为空结果

#### Scenario: A local refusal remains visible without rule details

- **WHEN** 一个方法部分恢复成功，某条赋值或表达式因证据不足被拒绝，调用方未选择详细规则记录
- **THEN** 正文及核心缺口都能定位被拒区域与原因，关闭详细证据不能消除拒绝、升级质量或隐藏可观察生产者

#### Scenario: Unknown denominators

- **WHEN** 方法表或查询范围未读完，完整数量无法确定
- **THEN** 返回已观察数量和未知剩余范围，不用零作为未知总数，不以“0 of 0”或空集合宣称完整

### Requirement: Detailed evidence is selected independently of semantic decisions

普通恢复 SHALL 默认交付必要结果而不物化完整可选证据表；调用方 SHALL 能显式选择完整证据，或选择源码映射、区域明细、规则明细、命名明细、读取明细中的类别及当前方法的 BCI 范围。未请求且不被算法依赖的详细记录 MUST 不被构造、复制或保留。内部算法必要的计划、来源与证明事实不属于可跳过的工作。完整证据选择 SHALL 保留既有审计范围；不支持的选择与非法范围 MUST 显式拒绝。

#### Scenario: Essential recovery has no optional table construction

- **WHEN** 对足够大的可恢复方法执行默认恢复
- **THEN** 完整映射段表及其它未选报告明细的拥有型记录构造计数为零，正文和必要缺口完整交付；仅在完整报告生成后删除字段不能通过验收

#### Scenario: An explicit full-evidence request

- **WHEN** 调用方显式选择所有详细证据且预算充足
- **THEN** 全部适用规则接受与拒绝记录、映射、区域、命名和读取明细按其已有语义交付，非适用类别为 Complete 的空结果，不能用精简模式替代该选择

#### Scenario: A local evidence range includes its supporting origins

- **WHEN** 选择当前方法某 BCI 范围的证据，相关表达式由范围外或同类 callee 的事实派生
- **THEN** 仅物化所选位置相关的记录及解释它所必要的来源闭包，跨方法来源保留各自完整身份；不能把 callee 的相同数值 BCI 当作当前方法的位置

#### Scenario: Invalid or unsupported selection

- **WHEN** 类别不被支持、范围反向、超出目标 Code，或无方法体却要求方法位置证据
- **THEN** 返回可识别的输入或适用性错误，不静默扩大为完整证据、不静默忽略选择；需读入才能判断的错误保留实际读取费用

### Requirement: Evidence selection cannot change a completed semantic answer

在同一不可变输入、确切物理方法、运行环境、恢复规则与输出配置下，且各请求预算足够完成相同事实计算时，改变证据选择 MUST NOT 改变源码字节、名字、类型、恢复/拒绝决定及其影响答案的缺口。执行成本、证据范围与对应证据状态 SHALL 按实际工作独立报告。系统 MUST 区分产物的语义质量、事实覆盖、证据交付覆盖与请求整体 execution。

#### Scenario: Essential and full results agree

- **WHEN** 相同方法分别执行默认恢复和完整证据恢复，两者都完整结束
- **THEN** 正文逐字节相同、核心语义平面与缺口一致，新增明细不扩大恢复接受集；全配置各自的重复运行仍通过完整确定性检查

#### Scenario: Detail stops after an artifact was committed

- **WHEN** 可信正文已提交，所请求的详细证据构造随后耗尽预算或被取消
- **THEN** 保留该正文及原有质量，证据标为实际前缀或未完成，整体 execution 为真实停止，不能删除已经提交的正文，也不能因正文存在宣称请求全部完成

#### Scenario: Tight budgets complete different amounts of work

- **WHEN** 相同有限额度下默认恢复完成而完整证据恢复中止
- **THEN** 报告各自真实费用和完成程度，不强求相同 usage/execution，也不将这种预算差异当作足额结果语义一致性的反例

### Requirement: Evidence expansion is bound to the exact artifact it explains

需要解释已有文本的证据请求 SHALL 绑定输入身份、物理方法及必要的成员序号、运行环境、规则/引擎 schema、输出配置和确切文本身份。展开可以在新请求预算内重建派生事实，但 MUST 验证对应产物一致后才把文本位置证据关联到原产物。已释放的内部状态 MUST NOT 被当作仍可用的引用；来源不匹配、规则变化、内容改变或无法重建 SHALL 显式回答，不能悄悄改指向新结果。

#### Scenario: Evidence is rebuilt after all temporary facts were dropped

- **WHEN** 调用方保存首次产物身份，释放全部临时事实后以同一 snapshot 和配置请求证据
- **THEN** 系统在新预算内重建必要事实，验证正文身份，再返回匹配证据；重建费用不被隐藏为命中或记入旧请求

#### Scenario: Formatting or rules changed

- **WHEN** 请求把新规则、输出配置或不同正文的映射附着到旧产物
- **THEN** 返回产物不匹配，不能仅凭相同 class 名、方法签名或 BCI 接受旧文本位置

#### Scenario: Duplicate physical definitions or member records

- **WHEN** 同一声明名存在多物理来源，或输入中同一签名存在多个成员记录
- **THEN** 证据只绑定被明确选择的来源与记录；选择仍歧义时先返回候选，不取第一个记录生成证据；相同字节的不同来源仍为不同定义，共享解析事实不能混用已经绑定来源的记录

### Requirement: Reuse ends with its explicit consumers and retention policy

同一操作的定位、读取和事实消费者 SHALL 复用仍被持有的可信产品，避免已持有事实的重复物化。跨请求复用 SHALL 受已有显式、有界保留策略约束，未保留事实允许在新预算内重新读取。局部请求不得因可能追问而预先构建其它方法、全局索引或所有证据；请求结束、取消及最后消费者释放后，不得留下无主任务或无限驻留的 IR。

#### Scenario: Selection and recovery share the selected class read

- **WHEN** 同一操作已为目标选择取得可信类事实，随后恢复该类选定方法及必要同类 callee
- **THEN** 所选定义不因进入下一个消费者再次读入或准备，未选择方法不被提前解码；搜索其它候选和必要依赖的实际工作仍独立计费

#### Scenario: No store and zero-capacity store

- **WHEN** 关闭保留或 store 无法准入，而当前消费者仍持有类或容器事实
- **THEN** 当前消费者继续使用同一可信产品，正确性与局部性不依赖缓存命中；最后消费者释放后允许后续请求重建

#### Scenario: A navigation sequence is abandoned

- **WHEN** 调用方只列成员便结束，或恢复一个方法后不再展开证据
- **THEN** 未请求的方法和明细构造次数为零，请求临时数据释放，显式保留的数据仍受容量约束

### Requirement: Demand-driven acceptance measures computation and delivery separately

验收 SHALL 同时记录实际读取、类准备、body 解码、分析阶段、可选记录构造、返回字节、在途/保留权重与时间。简洁/完整证据、首次结果/完整序列、请求内/跨请求复用 SHALL 分别比较。时间收益只有在固定构建、范围、行为与输出单位下的可重复测量才能宣称；工作计数下降不能代替耗时证据。已知非法或行为不同的恢复产物 MUST 作为独立未关闭项保留，不通过删除样本、隐藏证据或增加拒绝获得性能验收。

#### Scenario: Smaller evidence does not prove a faster recovery algorithm

- **WHEN** 精简结果减少了证据记录与输出字节，内部 IR 工作相同
- **THEN** 将收益归于证据物化或交付，保留相同计算范围的对照，不宣称恢复算法获得相同倍数的加速

#### Scenario: A sequence resumes and then changes direction

- **WHEN** 同一 snapshot 上执行成员读取、一个方法恢复、局部证据追问、换方法及放弃
- **THEN** 分别报告首次结果与全序列成本、复用与重建、峰值数据和释放；不可仅用同一方法反复命中的单点延迟代答
