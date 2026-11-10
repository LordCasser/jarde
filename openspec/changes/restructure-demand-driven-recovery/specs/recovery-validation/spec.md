## ADDED Requirements

### Requirement: Recovery stages are accepted by shared mechanism coverage

每个机制阶段的验收 SHALL 固定输入、范围、引擎/编译器版本、运行环境、源码目标、预算、基线和独立保留集。类型、表达式、区域、类级恢复阶段 SHALL 分别覆盖至少两个独立问题族，并以参数化组合与保留集证明共享机制的恢复收益。生产规则 MUST NOT 依赖 fixture 名称、固定 BCI 或输入摘要进行专用放行。

#### Scenario: A mechanism passes a development example only

- **WHEN** 某阶段只证明原始反例成功，没有其类型/消费者/结构变形、独立保留集或失败边界结果
- **THEN** 该阶段 MUST 保持未验收，不能按单个样例或已审阅测试文件数量报告机制完成

#### Scenario: Mechanism coverage is evaluated on complete classes

- **WHEN** 类型、表达式、区域或类级投影被计入受控机制矩阵的正向验收成功
- **THEN** 验收 SHALL 使用完整生成源码原样重编并对受控输入执行原 class 对照，检查返回、可见效果、异常和清理顺序；手工补桩或只编译摘录不能满足此项

#### Scenario: A known successful input regresses

- **WHEN** 冻结基线中的有效成功输入变为拒绝、编译失败、行为不同或来源丢失
- **THEN** 阶段 SHALL 记录并阻止无解释退化；若基线本身错误，必须提供原 class 证据和显式修订，不能直接改预期隐藏差异

#### Scenario: Compiler and debug variants exercise the same mechanism

- **WHEN** 承诺域内同一机制经已固定的不同编译来源、debug/no-debug、变量与结构组合产生不同 lowering
- **THEN** 适用输入 SHALL 通过同一机制合同验收，不按具体编译器版本猜测原始语言或跳过不利变体

#### Scenario: A holdout example is used to tune a production rule

- **WHEN** 一个保留样本或其同源改名/常量/flags 变体被用于调整准入条件
- **THEN** 验收 SHALL 标记来源关联与污染，不再以该样本声称独立泛化，并用未参与调参的独立评估集验证；生产规则仍不得按样例身份专用放行

#### Scenario: JADX disagrees with the original controlled class

- **WHEN** 固定 JADX 输出无法编译或在受控输入上与原 class 行为不同
- **THEN** 结果 SHALL 单独记录对照实现的失败，Jarde 的预期来自原 class 的受控 oracle，不能通过跟随 JADX 错误宣布正确

#### Scenario: Generated source execution could fall back to original classes

- **WHEN** 对恢复后的完整源码进行编译或受控执行验收
- **THEN** 生成源码 SHALL 原样编译，运行 classpath 隔离原始目标类并声明依赖范围；发现实际执行了原目标或手工补桩时，该次结果不能计为恢复成功

#### Scenario: A known wrong result still reports unproven validation

- **WHEN** 已有受控反例证明生成结果错误，而报告仍使用 Unproven 或 NotPerformed
- **THEN** 验收 MUST 将其计为已知失败；未证明标签不能豁免错误，修正错误与新增恢复覆盖分别统计

### Requirement: Real artifact gains and resource costs use frozen denominators

阶段报告 SHALL 同时给出固定真实 JAR/WAR 的方法与完整类分母、适用与排除范围、含语句/可编译/实际行为对照/停止分类，以及请求读取、Body/IR 工作、时间、内存和取消结果。阶段收益 MUST 由同输入同配置的新旧产品对照支持；诊断减少、Produced 比例或文本相似度不能替代恢复和行为证据。

#### Scenario: A repair reveals a second blocker

- **WHEN** 类型修复后同一方法显露原先未到达的区域或类级问题
- **THEN** 报告 SHALL 保留多重阻塞归因，直到相应交付层真正成功才计收益，不因首条诊断消失计为恢复完成

#### Scenario: An artifact or oracle is missing

- **WHEN** 冻结真实包无法获得，或某输出没有实际可用的执行 oracle
- **THEN** 报告 SHALL 分别记录未执行与未验证；不得换分母伪造阶段通过，不得将恢复或编译成功算成行为一致

#### Scenario: Shared mechanism changes the billed work

- **WHEN** 新机制改变计费量或请求内复用导致读取减少
- **THEN** 验收 SHALL 说明实际工作变化并对照预先冻结的资源门；不能仅更新精确计数预期，也不能把 AnalysisSteps 当作耗时或取消延迟测量

#### Scenario: Optional evidence and sufficient cache paths are compared

- **WHEN** 同一已完成恢复分别选择不同可选证据，或在足够预算下使用关闭、冷、热物理事实缓存
- **THEN** 源码语义决定、物理身份与核心缺口 SHALL 一致；可选证据和实际资源消耗按各自路径报告，证据选择不能改变恢复能力

#### Scenario: Physical records and attempted recoveries have different denominators

- **WHEN** 包中存在 NoBody、重复/被遮蔽定义、runtime 未绑定方法、输入拒绝或预算停止
- **THEN** 报告 SHALL 分别给出物理 class/method、具有 Code、可绑定、已尝试及交付层分母，固定排除原因；不能通过只统计已成功尝试项提高恢复率

#### Scenario: Diagnostic tags overlap or class compilation is sampled

- **WHEN** 同一方法具有多个诊断标签，或只编译了部分选中类
- **THEN** 报告 SHALL 说明标签重叠、首个阻塞与根因的证据等级，并公开类采样算法/样本量；诊断条数不得相加成互斥方法分布，样本类结果不外推为全包编译率

#### Scenario: A baseline measurement batch is interrupted

- **WHEN** 必需资源样本、取消测量或输入核验未完成
- **THEN** 基线 SHALL 保持未验收并保留已有原流，不能补写成功、混合配置凑次数或挪用历史 SHA 数字；后续按声明协议完成批次。已测得的产品失败单独记录为后续门的输入，不伪装成缺测或已修

#### Scenario: Cache capacity and tight budgets change completed scope

- **WHEN** 相同输入在 cache 禁用/冷/热/容量不足及不同紧预算下完成范围不同
- **THEN** 验收 SHALL 核对当前限额和真实执行范围；充分预算下语义/身份/来源一致，紧预算停止不变成完整否定，不把成本不同本身当作语义错误

### Requirement: Resource and failure-path gates distinguish retention cancellation and delivery

资源验收 SHALL 分别测量逻辑计费、实际读取/解码工作、时间、峰值内存、cache/live facts 与候选/输出保留以及消费者背压。token 取消、预算/elapsed 停止、sink 停止与 worker panic SHALL 独立记录触发点、合法交付前缀、未处理范围和返回行为；一种停止的结果不得替代另一种。

#### Scenario: Required work and retention ports are missing

- **WHEN** S0 所需的真实 read/IR 或 live facts/候选/AST 峰值没有现成端口
- **THEN** S0 SHALL 通过 test-support 在实际工作与对象生命周期位置补齐观测，并校验构造/共享/clone/drop 及观测开关不改变恢复语义；对象数、权重代理与实际字节 SHALL 明确区分，不用逻辑计费或 RSS 替代缺项，补齐后完整重跑已声明协议

#### Scenario: Consumer stop is faster than token cancellation

- **WHEN** 基线已有 sink stop 结果但没有在工作阶段置位取消 token 的测量
- **THEN** 取消延迟 SHALL 保持未验证，不能用 sink stop 时间填充；token 测量记录置位到返回及停止后的实际工作

#### Scenario: Slow delivery overlaps retained results and optional evidence

- **WHEN** 慢消费者使 bulk 窗口、候选文本及可选 evidence 同时存活
- **THEN** 验收 SHALL 记录有效并发/窗口配置和实际峰值，核对背压与预算；cache 容量通过不能替代总保留/RSS 检查，默认栈不可用扩大线程栈掩盖失败

#### Scenario: A bulk class is refused before preparation

- **WHEN** 准备前拒绝或关闭故障发生，流使用已声明的 Prepared 或 Refused 分支
- **THEN** 故障验收 SHALL 要求非 Refused 正常结束具有已交付 Prepared，Refused 按其独立协议判断；sink 拒绝后不得强制补发事件，合法前缀和协调器退出状态分别保存

#### Scenario: A known independent debt affects a stage gate

- **WHEN** 某阶段的资源、坐标、格式或批量故障结论依赖已登记但未解决的独立债务
- **THEN** 该项验收 SHALL 保持未通过并关联独立修复证据，不把失败从分母删除或以计划完成代替实现；基线可以如实冻结已测失败，不能宣称故障路径已合约通过

### Requirement: Recovery fuzzing reaches source and class projection stages

受控 fuzz SHALL 覆盖方法 Java 恢复与类级投影，核对终止、预算、来源、转换完整性和报告状态；必须记录有效进入目标阶段的输入数量。fuzz MUST NOT 编译或执行任意变异目标，行为差分使用另外声明的受控 fixture 流程。

#### Scenario: Most mutations fail before recovery

- **WHEN** 一轮 fuzz 主要在格式或 IR 阶段提前拒绝
- **THEN** 报告 SHALL 单列拒绝位置与实际到达 Java/类级阶段的输入数，不能把总迭代数称为恢复层覆盖

#### Scenario: Candidate failure or stop occurs during projection

- **WHEN** 有界有效输入在效果吸收、区域归属或类级修改期间产生拒绝或全局停止
- **THEN** 检查 SHALL 验证没有半项提交、丢失效果来源、越预算或伪造 Complete，并把真实违规保留为可重放反例

### Requirement: Recovery milestone acceptance has finite scope and explicit prerequisites

每个恢复里程碑的验收 SHALL 在实现和产品测量前固定入口范围、不变量、输入/配置与必需门，并列出不适用项的证据和独立归属。保持行为不变的职责整理 MUST NOT 成为无限增加的退出条件；实际破坏适用正确性、身份、停止或资源门的问题仍 SHALL 阻塞相应里程碑。局部里程碑通过 MUST NOT 被报告为更大阶段或独立债务已完成。

#### Scenario: Another ownership cleanup is discovered before a milestone exits

- **WHEN** 已声明退出清单之外发现一项仅改变源码政策所在模块的整理，没有证据表明它破坏适用不变量
- **THEN** 验收 SHALL 将其登记为独立整理，保留现有有效结果与历史证据，不以清空所有门面判据为退出前置；影响尚未核实的风险必须保留未决，不能伪称无影响

#### Scenario: Deferred cleanup contains a reachable correctness or stop defect

- **WHEN** 独立整理范围内的反例或可复核调用路径证明当前验收入口存在身份串用、错误源码、停止丢失、半提交或适用资源超门
- **THEN** 对应里程碑 SHALL 保持未通过并修复最小因果闭包；不能因“已知失败”或独立整理标签豁免，也不以修复该缺陷为由要求完成全部整理

#### Scenario: A resource row belongs to an independent delivery path

- **WHEN** 原冻结资源矩阵的一项只依赖未被本次修改或用作证据的 bulk 协调/慢消费者路径，且其独立归属已在产品运行前确定
- **THEN** 原输入、分母、失败和阈值 SHALL 完整保留，对应全负载门继续未验收；当前里程碑只能声明已通过的适用范围。若本次修改或依赖该路径，该项仍为阻塞门，不能按运行好坏排除配置或放宽阈值

#### Scenario: Source variable legalisation is ready before generic expansion

- **WHEN** 变量分组、全部写入/使用约束、声明/definite assignment、普通与异常边值关系的有限域已通过完整类、行为、两问题族、独立保留集、固定包净收益及适用资源门，而新增泛型传播尚未验收
- **THEN** 该域 SHALL 具备独立里程碑证据，允许其已证明类型/调用合同被顺序恢复消费；不能宣称完整类型阶段完成，不能对依赖未交付泛型能力的输入猜测 cast 或放宽调用绑定

#### Scenario: Sequential recovery succeeds while sugar remains unfinished

- **WHEN** 保持原求值次数、顺序、条件执行、异常与数值边界的完整顺序源码通过两问题族、独立保留集、固定包净收益、停止及适用资源门，而表达式压缩仍未完成
- **THEN** 该基线 SHALL 独立报告实际恢复收益，糖化留独立里程碑；不能只比较中间计划充当源码成功，也不能把局部通过当成整个表达式或区域阶段完成

#### Scenario: Planning changes split existing gates without executing the product

- **WHEN** 修订依赖和退出清单但没有新的实现或产品验收
- **THEN** 状态 SHALL 保留原已完成项、失败原流和阈值，新增或重述任务保持未完成；规划完整或复选分母变化不能增加产品通过数
