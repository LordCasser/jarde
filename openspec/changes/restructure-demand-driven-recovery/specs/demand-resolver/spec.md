## ADDED Requirements

### Requirement: Recovery dependencies are explicit and reusable within an operation

恢复所需依赖 SHALL 由具体候选提出，绑定物理来源、解析环境、需求类别与 reason。系统 SHALL 默认只读取必要 Header；读取辅助 Body 必须定位到具体物理成员并说明恢复用途。相同操作内重复需求 SHALL 复用已取得的完整事实，且不要求开启跨请求缓存。

#### Scenario: A type relation requires headers only

- **WHEN** 恢复仅需证明父类、接口或成员签名关系
- **THEN** 读取范围 SHALL 限于必要 Header 闭包，相关依赖 Body 解码次数为零，来源和需求理由可核对

#### Scenario: Two candidates demand the same helper

- **WHEN** 同次操作的两个候选在相同环境下需要同一个辅助方法 Body
- **THEN** 系统 SHALL 复用一次已完成解码及其身份与来源，仍分别保留两项需求的理由，不为每项候选重复读取或解码

#### Scenario: No recovery candidate needs another body

- **WHEN** 一个普通方法请求的候选检查不需要外部成员 Body
- **THEN** 未请求 Body 的读取与分析次数 SHALL 为零，不预扫描整包以建立候选闭包

#### Scenario: Static and captured-this helpers share the same class use inventory

- **WHEN** 完整类源码请求中的同类 synthetic helper 候选包含 static 或 captured-this 实现，投影需要证明完整成员使用闭包
- **THEN** 需求阶段 SHALL 不因实例 helper 缺少 static 标志而遗漏既有同类 census；header 只提出需求，投影仍须核验精确 raw owner/name/descriptor、handle kind 与访问标志、完整使用及同次 AST，复用已请求的成员 Body/IR，不为该证明重复读取或构建 IR；未知使用或全局停止不得提交不完整投影

#### Scenario: Cache capacity does not change complete facts or physical origins

- **WHEN** 相同需求分别在禁用、冷、热或容量不足的物理事实缓存下以足够预算执行
- **THEN** 取得的完整事实与绑定的物理来源 SHALL 一致，同次 live facts 可继续复用；同 bytes 不同 origin 不合并，实际读取和计费按真实路径报告

### Requirement: Recovery fact decisions preserve resolution uncertainty and stops

恢复依赖结果 SHALL 区分取得事实、完整搜索后缺失、歧义、尚未决定的关系与执行停止。类型关系 MUST 依据实际 defining loader 与物理定义解释；平台契约与已读取物理 Header 的来源必须可区分。

#### Scenario: Same name is defined by different loaders

- **WHEN** 两个 loader 下存在同名且不同物理定义的类型
- **THEN** 每次恢复 SHALL 使用实际解析出的定义关系，不以名字相等合并继承事实或候选

#### Scenario: Input definition overrides a platform assumption

- **WHEN** 显式环境选中某个输入定义，而其名字与内建平台契约重合
- **THEN** 输入定义的真实选择 SHALL 被保留；不以平台表覆盖输入，不伪称读取了未提供的 JDK Header

#### Scenario: A dependency search stops before absence is established

- **WHEN** Header 或辅助 Body 需求因预算、取消或损坏未完成
- **THEN** 结果 SHALL 保留停止及未决范围，不将其变成 Missing、模式不匹配或安全的负结论，不继续该失败需求链；全局预算/取消后不启动后续需求，局部损坏仍按既有成员隔离合同保留其他独立结果

#### Scenario: Nested source proofs preserve execution stops as typed errors

- **WHEN** 类级源码证明调用一个同时可能返回源码形状拒绝与预算/取消错误的嵌套证明
- **THEN** 预算/取消 SHALL 保留外层错误并返回供给方，由既有 execution/diagnostic 编排处理，不转换成候选不匹配字符串；合法形状与局部拒绝仍分别保留，已完成独立成员前缀不被撤销，共享 ledger 的首次停止及后续需求截断保持

#### Scenario: Required context is unavailable after a completed lookup

- **WHEN** 完整搜索确认缺少候选所需 Header 或 Body
- **THEN** 结果 SHALL 指明缺少的物理/符号目标及用途，保留未转换表示或拒绝；不自动下载依赖或执行目标补足事实

#### Scenario: A cyclic demand is still in progress

- **WHEN** 一项恢复需求沿依赖链再次遇到本次尚未完成的需求
- **THEN** 系统 SHALL 有界地保留进行中/未决或停止状态，不无限重试，不把环或中途失败当成完整搜索后的 Missing

#### Scenario: A later operation changes a dependency

- **WHEN** 后续操作改变依赖 snapshot 或有效 loader 配置，而某次旧需求曾给出缺失、未决或部分事实
- **THEN** 新结论 SHALL 按当前环境重新决定，不把旧 Unknown/Stop 复用成不存在；仍可共享与环境无关的完整物理事实

### Requirement: Environment identity preserves every effective resolution declaration

解析与恢复结果所引用的环境身份 SHALL 区分全部参与 loader domain 的 parent、delegation、有序 roots、raw prefix、container origin、module/uncertainty、provider 内容声明及提供的 snapshot 身份。相同调用方、loader/provider 名称列表和内容列表 MUST NOT 被视为完整环境相同的充分条件。provider 的声明 SHALL 不凭空增加搜索位置或优先级。

#### Scenario: A parent domain changes ordered roots

- **WHEN** 两次请求具有相同调用方 domain、content 顺序及 loader/provider ID，但父 domain 的两个 roots 顺序不同，roots 中存在不同的同名定义
- **THEN** 环境身份 SHALL 区分两种声明，解析按各自有效顺序选择，旧环境的结论或证据绑定不能静默复用

#### Scenario: Prefix delegation or uncertainty changes outside the caller domain

- **WHEN** 非调用方 domain 的 raw prefix、parent、delegation、module mode 或 uncertainty 改变
- **THEN** 身份 SHALL 反映变更；当前不支持的声明仍按既有环境诊断拒绝，不能因旧 identity 相等沿用已支持环境的答案

#### Scenario: Provider content bindings change without renaming the provider

- **WHEN** provider ID 不变，但其绑定内容声明或提供的 snapshot 身份改变
- **THEN** 环境身份 SHALL 区分新旧声明，校验 root/provider 绑定；不把同名 provider 当成同一可复用解析事实

#### Scenario: Source presentation changes without changing physical content

- **WHEN** 只改变源码目标或可选呈现证据，物理内容、解析策略和运行环境保持不变
- **THEN** 物理解析事实与共同符号的解析身份 SHALL 保持不变，纯结构/纯 JVM 请求不因此引入源码阶段；额外源码需求仍有明确理由


### Requirement: Task environment policy can preserve an explicit loader graph

系统 SHALL 允许现有 EnvironmentRequest 通过 ExplicitDomains 政策声明现有 LoadDomain/HeaderProvider 图，并由 request.loader 精确选取唯一 caller domain。系统 SHALL 保留原 domains/providers 顺序、scope 和 RuntimeProfile，继续原环境校验、完整身份与定义选择，不从 provider 或布局猜测 root，不把多个 root 冒充多个 loader。

#### Scenario: Class source receives an explicit parent and child domain

- **WHEN** caller 为 ChildFirst，parent/child 对同名类提供不同物理定义，完整类入口请求恢复两个同 BCI 方法
- **THEN** 同一次请求保留完整图；每个方法仅消费当前实际定义链的证明，parent 定义后继从 parent 查询，目标身份不能因同名字节相同而合并

#### Scenario: Explicit graph has no unique caller

- **WHEN** domains 中没有 request.loader 或有多个同名 caller domain
- **THEN** build 明确返回 environment_policy_caller_domain_not_unique，不合成默认 domain；完整类入口不启动 Body 分析

#### Scenario: Other unsupported graph declarations remain validator findings

- **WHEN** caller 唯一而其余 graph/root/provider 声明超出原支持域
- **THEN** 原样交给现有 validator 与停止/Unknown 合同，不将其改写成受支持的单 loader 环境
