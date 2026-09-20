## MODIFIED Requirements

### Requirement: Independent analysis planes
系统 SHALL 允许直接打开 artifact 请求事实或单个方法，不要求全程序加载、全局索引、持久数据库或预先反编译。Query 与 Decompiler MUST 共享输入解码和身份契约，但不得隐式互相启动；依赖默认只扩展 Header，Body 升级必须附理由和预算。任务导向操作 SHALL 在同一契约下组合这些平面：打开 artifact 自身保持轻量，组合操作 SHALL 在一次请求内复用同一次类读取与成员列举，并让每个方法保留自己的阶段结果、覆盖、执行状态与诊断，MUST NOT 为每个方法重新扫描、重新读取或重建已经取得的类事实。

#### Scenario: Artifact inspection before indexing

- **WHEN** 调用方首次打开一个 JAR，仅请求某 entry 的 Header
- **THEN** 系统不构建全局 XRef、CFG、SSA 或 Java AST，并记录实际读取/物化范围

#### Scenario: Class view shares one read

- **WHEN** 调用方请求一个类的视图（声明、字段、方法列表与按需方法体）
- **THEN** 同一请求内只有一次类 Header 读取与一次成员列举，每个方法的结果与阶段状态独立保留；为每个方法重新读取同一 Header 或重复列举必须使该计数检查失败（验收 A13、A16）

#### Scenario: Open stays lightweight

- **WHEN** 调用方只打开 artifact 而不请求任何事实或方法
- **THEN** 不读取任何 class Header/Body，也不构建 resolver、CFG、SSA、Region 或 Java AST；读取只发生在随后的按需操作中（验收 A16、A17）

#### Scenario: Repeated operations over one immutable artifact

- **WHEN** 同一不可变 snapshot 上重复执行同一操作
- **THEN** 报告的事实、身份、顺序、coverage 与 execution 保持一致，snapshot 的字节不因源文件或外部状态变化而改变（验收 A18）；该路径存在复用时，启用与未启用复用得到相同结果（验收 A15）

## ADDED Requirements

### Requirement: Task-oriented target selection binds real physical identity

任务导向操作 SHALL 接受两种目标输入：friendly 名称（点分隔类名、成员名、descriptor 与可选过滤条件），或调用方已有的物理身份；两种输入 MUST 收敛到同一物理身份（definition location、class bytes、variant 与成员 descriptor），并复用与物理导航相同的名称匹配与歧义规则。匹配不唯一时操作 SHALL 返回候选及各自物理依据并不执行，MUST NOT 静默取第一个；调用方给出的身份不属于本次 artifact 时 MUST 返回可定位的输入错误，不得改用同名定义。

#### Scenario: Friendly name selects one definition

- **WHEN** 调用方用 friendly 名称发起操作，且范围内只有一个匹配的物理定义
- **THEN** 操作绑定该定义并发布所选择身份；结果与直接给出同一物理身份的请求一致，不以显示名作为身份

#### Scenario: Ambiguous name is not resolved silently

- **WHEN** 名称匹配多个物理定义或同类内多个同名 descriptor
- **THEN** 操作返回候选与选择依据，不执行分析、不选择第一个；调用方回传所选身份后才继续（验收 A07）

#### Scenario: Existing identity is used as given

- **WHEN** 调用方给出既有 `PhysicalMethodId`
- **THEN** 操作按该身份执行；身份属于另一 snapshot 或其他物理定义时返回可定位的输入错误，不替换为同名定义（验收 A18）

### Requirement: Task-oriented operations publish their stages and effective configuration

任务导向操作 SHALL 按固定 pass 表选择其所需 stage 并在报告中发布实际执行的 stage 集合，`Engine::analyze_method` 的显式 stage 列表保持为可用的底层控制且语义不变。操作 SHALL 应用有界默认预算、只允许少量显式覆盖，并在结果中发布生效的完整 `Limits` 与 `UsageSnapshot`；未知或不可用的覆盖 MUST 是输入错误，MUST NOT 静默取默认，也 MUST NOT 用默认值掩盖预算造成的停止。

#### Scenario: Operation selects its stages and publishes them

- **WHEN** 调用方不提供 stage 列表而使用任务导向操作
- **THEN** 报告发布实际执行的 stage 集合，且与同一 fixture 上用显式 stage 复现的调度一致；显式 stage 的 `analyze_method` 请求保持原校验、调度与停止语义

#### Scenario: Effective configuration is published

- **WHEN** 调用方只覆盖部分预算维度
- **THEN** 结果包含生效的完整 `Limits` 与 `UsageSnapshot`；覆盖导致工作中断时返回真实 Partial/Cancelled 与终止维度，不伪装成功（验收 A14）

#### Scenario: Unknown override is rejected

- **WHEN** 请求包含未知维度或非法覆盖值
- **THEN** 返回输入错误，不静默取默认值继续执行
