## ADDED Requirements

### Requirement: Explicit composite requests connect query resolution and recovery

系统 SHALL 提供显式阶段组合请求，声明物理筛选、可选声明解析或有界派发条件、可选源码交付、运行环境、源码目标和预算。各阶段 SHALL 保留独立身份、范围、证据与状态，结构事实不因源码投影而改变。独立物理查询 SHALL 保持现有解析边界，不因解析与恢复能力存在而自动启动它们。

#### Scenario: Structural selection requests source delivery

- **WHEN** 调用方明确请求先筛选引用候选，再在给定环境解析匹配成员并恢复其源码
- **THEN** 系统 SHALL 按显式计划执行，只为确认需要的成员及有理由依赖启动方法分析；每个交付项同时保留原始引用、解析依据和恢复状态

#### Scenario: Structural or declaration results are sufficient

- **WHEN** 调用方只请求物理关系，或组合请求只到声明解析而没有源码交付
- **THEN** 前者 SHALL 不启动 resolver/IR/恢复，后者 SHALL 不构建 Region 或 Java AST；未请求的更高阶段报告 NotRequested

#### Scenario: A filter cannot decide a semantic predicate

- **WHEN** 缺依赖或开放世界条件使候选是否匹配未决
- **THEN** 结果 SHALL 保留未决候选与原因，不把 Unknown 过滤成 NoMatch，也不将已知派发候选声称为全部运行目标

#### Scenario: A projection hides an accessor in generated source

- **WHEN** 匹配成员在源码交付中被安全投影隐藏
- **THEN** 组合结果 SHALL 保留其物理声明、原始引用和到投影的来源关系，查询匹配不因隐藏而消失

### Requirement: Composite pagination and stopping preserve the declared plan

组合请求的游标 SHALL 绑定完整阶段配置、谓词、snapshot/view、全部参与 domain/provider 的有效环境声明、适用的源码目标及词法/投影配置、稳定顺序和相关游标/解析/恢复语义版本。预算和页大小为执行参数；全局停止 SHALL 保留已提交前缀并终止后续工作，未分析范围不得解释为否定结果。

#### Scenario: A caller changes the stage plan while continuing

- **WHEN** 调用方使用原游标但改变谓词、运行环境、阶段集合或源码目标
- **THEN** 系统 SHALL 在继续扫描前拒绝游标，不按旧边界跳过新请求的结果

#### Scenario: Paged and unpaged composite operations finish

- **WHEN** 同一计划在足够预算下分别分页读到末尾与不分页执行
- **THEN** 拼接后的语义结果、物理身份、稳定顺序和源码映射 SHALL 一致；页间实际资源使用分别报告，不伪造相同计费

#### Scenario: Recovery stops after structural selection completed

- **WHEN** 结构筛选已完成，而后续请求的恢复阶段遇到取消或预算停止
- **THEN** 结构 coverage SHALL 保留其实际完成事实，整体 execution 非 Complete，未开始方法标为未处理；不能继续启动后续 Body 或把它们当作不匹配

#### Scenario: Composite and separate execution have the same declared context

- **WHEN** 相同输入事实、环境、源码目标与充分预算通过组合操作和对应分立操作执行
- **THEN** 两者 SHALL 得到相同语义决定、物理身份与适用来源；复用造成的实际读取/计费差异单独说明，不能隐藏结果差异

#### Scenario: A parent loader changes without changing the caller

- **WHEN** 使用旧游标时 caller domain、loader/provider ID 列表和内容顺序未变，但父 roots、prefix、delegation 或有效 provider 内容声明改变
- **THEN** 系统 SHALL 在继续工作前拒绝环境不匹配的游标，不以简化 identity 相等跳过新环境下的候选

#### Scenario: Semantic versions or projection policy change while continuing

- **WHEN** 游标续读时相关解析/恢复语义版本或影响源码交付的投影配置改变，即使包版本与先前文本相同
- **THEN** 系统 SHALL 拒绝不匹配计划；只改变页大小或预算仍按执行参数处理，不把格式版本、实现版本与运行参数混为一项

#### Scenario: A composite source unit carries class provenance

- **WHEN** 显式组合请求交付具有类级投影的源码及所选映射
- **THEN** 结果 SHALL 保留同分立类恢复一致的物理来源、文本绑定与映射状态；方法局部 map 不得被解释为整类坐标，未请求映射不改变筛选或源码语义

#### Scenario: Global stop preserves completed stages without starting later bodies

- **WHEN** 组合操作在候选需求、恢复或交付期间被取消或耗尽全局预算
- **THEN** 已完成阶段及合法提交前缀 SHALL 保留各自状态，后续 Body/候选需求不启动，未处理项不变成 NoMatch；局部候选拒绝与全局停止不能混淆
