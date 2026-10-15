# Spec Delta

## ADDED Requirements

### Requirement: Interface-super proof stops remain visible in recovery results

接口限定 `super` 调用的源码证明在本请求中因预算、取消或期限停止时，公共恢复结果 SHALL 保留该次停止的原原因、预算维度、可用位置与真实 usage，返回非 Complete 的恢复 execution 和对应停止呈现。MUST NOT 以空证明、普通源码拒绝或 Complete fallback 隐藏停止；停止前已完成的 raw analysis SHALL 保留，停止后 MUST NOT 继续恢复或提交该方法的文本及 source map。

#### Scenario: Dependency depth refuses a public method proof

- **WHEN** 同一真实接口特殊调用在宽预算下能完成证明并呈现，公开恢复请求仅将 dependency_depth 限额降到证明所需深度以下
- **THEN** 结果 MUST 保留已完成 analysis，恢复 execution 为 Partial 且预算原因明确为 DependencyDepth；恢复 outcome 为 Stopped、content 为 not_produced，text/source map 为空，presentation 显示同一停止。MUST NOT 返回仅含缺证明说明的 Produced/Complete，也 MUST NOT 将合法请求变成没有恢复报告的裸错误

#### Scenario: Counted proof budget keeps its actual dimension

- **WHEN** 接口源码证明已开始且其实际元数据读取或分析计费因本请求预算拒绝继续
- **THEN** 公共报告 MUST 命名原拒绝维度，保留停止时 usage 和可用定位，不得因后续正文使用另一种预算而改报 OutputBytes、IrItems 或一个普通缺证明理由；调用方能够区分实际预算停止与完整读取后未能证明

#### Scenario: Cancellation occurs during proof work

- **WHEN** 本请求的取消 token 初始未置位，并在接口源码证明的一次真实收费中被确定性置位，随后的真实 checkpoint 观察到取消
- **THEN** 公共恢复报告 MUST 为 Cancelled，保留停止前 analysis 与实际已准许的收费，停止后没有正文或 source-map 产物；MUST NOT 用入口预取消、测试线程 sleep 或纯内部证明函数替代这一公共路径验收

#### Scenario: Deadline interrupts proof before recovery starts

- **WHEN** 请求已抵达接口源码证明，证明中的 checkpoint 首次观察到真实期限停止
- **THEN** 公共恢复结果 MUST 保留既有期限停止语义及可用定位，非 Complete 且未提交文本，MUST NOT 伪装成正常空证明、Cancelled 或正文输出预算耗尽；验收 SHALL 能证明期限停止发生在证明路径中

#### Scenario: The first proof stop is not replaced by later work

- **WHEN** 接口证明因 DependencyDepth 或其它原预算停止，而继续正文恢复还可能触发不同的预算或期限
- **THEN** 恢复 SHALL 在原停止处终止，不再启动正文恢复或证据物化；最终原因和 usage 不得被这些本不该执行的工作改写，已完成 analysis 不得被重跑或重复计费

#### Scenario: Missing or ambiguous evidence remains an ordinary refusal

- **WHEN** 实际证明读取完整且未发生执行停止，但层级定义缺失、选择歧义、限定接口冗余或目标被抽象声明遮蔽
- **THEN** 恢复 SHALL 保持既有保守拒绝、物理方法/BCI 来源和接收者/实参生产者效果；MUST NOT 将这种普通拒绝伪造为预算或取消停止，也不得借修复停止传播放行非法接口限定调用

#### Scenario: Wide public recovery preserves a legal interface call

- **WHEN** 实际选定环境能够完整证明合法接口限定调用，且请求预算足够
- **THEN** 公共恢复 SHALL 仍正常完成，保留原限定调用、物理来源和声明事实；合法 fixture 的完整生成源 SHALL 继续通过既有编译执行对照。MUST NOT 用普遍停止或普遍拒绝替代本修复

#### Scenario: Public consumers share the recovery stop

- **WHEN** 单方法恢复或 class-source 成员恢复在接口源码证明处停止
- **THEN** 恢复报告、任务 presentation 及已有 CLI 消费 SHALL 表达同一非 Complete 停止；class-source SHALL 合并该成员停止并保留允许保留的其他成员与 raw 事实。MUST NOT 依赖调用方解析文本重建原因，质量降级不能替代 execution
