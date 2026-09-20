## MODIFIED Requirements

### Requirement: Independent analysis planes
系统 SHALL 允许直接打开 artifact 请求事实或单个方法，不要求全程序加载、全局索引、持久数据库或预先反编译。Query 与 Decompiler MUST 共享输入解码和身份契约，但不得隐式互相启动；依赖默认只扩展 Header，Body 升级必须附理由和预算。任务导向操作 SHALL 在同一契约下组合这些平面：打开 artifact 自身保持轻量，组合操作 SHALL 在一次请求内复用同一次类读取与成员列举，并让每个方法保留自己的阶段结果、覆盖、执行状态与诊断，MUST NOT 为每个方法重新扫描、重新读取或重建已经取得的类事实。组合类视图的顶层 execution SHALL 汇总本次搜索、成员读取与所有已请求方法体的停止；任一请求部分未完成时不得发布顶层 Complete。各 coverage 维度 SHALL 仍按实际读取范围报告，不能因方法体失败而否认已完整读取的成员表。

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

#### Scenario: A requested body stops while another succeeds

- **WHEN** 成员表完整，一个被请求方法的指令解码在 BCI 0 停止，另一个被请求方法完整读取
- **THEN** 两个方法各自保留真实结果、coverage 与停止位置，顶层 execution 非 Complete；局部损坏不删除正常方法，已完成的类/成员结构覆盖仍按自身证据报告（验收 A13、A14）

#### Scenario: A body exhausts the shared operation budget

- **WHEN** 方法体处理中耗尽共享预算或收到取消
- **THEN** 组合操作保留真实终止原因和已发布前缀，不重置预算、不继续启动后续方法体工作；顶层与逐方法执行状态一致（验收 A14、A16）


### Requirement: Task-oriented target selection binds real physical identity

任务导向操作 SHALL 接受两种目标输入：friendly 名称（点分隔类名、成员名、descriptor 与可选过滤条件），或调用方已有的物理身份；两种输入 MUST 收敛到同一物理身份（definition location、class bytes、variant 与成员 descriptor），并复用与物理导航相同的名称匹配与歧义规则。匹配不唯一时操作 SHALL 返回候选及各自物理依据并不执行，MUST NOT 静默取第一个；调用方给出的身份不属于本次 artifact 时 MUST 返回可定位的输入错误，不得改用同名定义。名称搜索 SHALL 在范围与相关成员表完整后才作唯一或缺失判断；若搜索因损坏、预算或取消未完成，操作 MUST 返回独立的未完成选择结果，保留可靠候选、coverage、execution、diagnostics 与实际用量，不启动方法分析/恢复，不把零候选当成未找到，也不把一个候选当成唯一。

#### Scenario: Friendly name selects one definition

- **WHEN** 调用方用 friendly 名称发起操作，且范围内只有一个匹配的物理定义
- **THEN** 操作绑定该定义并发布所选择身份；结果与直接给出同一物理身份的请求一致，不以显示名作为身份

#### Scenario: Ambiguous name is not resolved silently

- **WHEN** 名称匹配多个物理定义或同类内多个同名 descriptor
- **THEN** 操作返回候选与选择依据，不执行分析、不选择第一个；调用方回传所选身份后才继续（验收 A07）

#### Scenario: Existing identity is used as given

- **WHEN** 调用方给出既有 `PhysicalMethodId`
- **THEN** 操作按该身份执行；身份属于另一 snapshot 或其他物理定义时返回可定位的输入错误，不替换为同名定义（验收 A18）

#### Scenario: One candidate precedes a damaged candidate

- **WHEN** 名称搜索已确认一个匹配方法，随后同名候选读取失败
- **THEN** 返回未完成选择及已确认候选和失败来源，不执行该方法、不丢弃搜索诊断；该候选只有在调用方显式选择其物理身份后才可用于独立操作（验收 A07、A14）

#### Scenario: No candidate is confirmed before a stop

- **WHEN** 损坏、预算或取消使搜索在确认任何目标之前停止
- **THEN** 返回零可靠候选的真实停止结果，不报告完整缺失或用法错误；完整搜索确实无匹配时才允许未找到结果（验收 A14）

#### Scenario: Member selection has an unread suffix

- **WHEN** 类声明已确认，但相关成员表在目标选择完成前中断
- **THEN** 保留成员表停止和已确认前缀，不把未到达的成员宣称为不存在，也不把部分重载集合宣称为唯一匹配


### Requirement: Task-oriented CLI is a thin adapter over library operations

CLI SHALL 提供任务链命令，每条命令直接调用相应的库操作，并把该操作的报告（含 coverage、execution、diagnostics 与 usage）作为唯一结果来源；CLI MUST NOT 重新扫描 artifact、重新选择目标、重新分析文本或补造结果。同一请求 SHALL 支持文本与 JSON 两种输出，两者 MUST 由同一份报告派生且语义一致。诊断 MUST 与正文分离：正文进入标准输出或输出文件，诊断进入独立字段或标准错误，不得混入代码正文。输出文件选项 SHALL 在写入前执行与标准输出相同的 `output_bytes` 检查，文件内容 MUST 与标准输出模式一致，执行未完整时 MUST NOT 写出伪装成功的文档。命令 SHALL 使用显式、稳定、可区分的退出状态：成功为 0，用法/输入错误、需要调用方选择（名称歧义）与执行未完整（Partial/Cancelled/Stopped）各自给出不同的非零状态。任务链内的后续命令 SHALL 接受前一条命令返回的物理身份，MUST NOT 要求调用方手工重拼身份。

#### Scenario: Text and JSON come from one report

- **WHEN** 同一请求分别以文本与 JSON 运行
- **THEN** 两者的方法身份、content/quality、coverage、execution 与诊断集合一致，仅渲染不同，且没有额外的 artifact 读取（验收 A13、A16）

#### Scenario: Diagnostics are separated from the body

- **WHEN** 恢复产生诊断并以文本模式输出
- **THEN** 诊断不进入代码正文；JSON 输出把 diagnostics 放在独立字段，正文与诊断都可定位（验收 A13）

#### Scenario: Output file matches stdout

- **WHEN** 同一请求使用输出文件选项
- **THEN** 文件内容与标准输出模式一致；预算不足或取消时保留真实停止状态，不写出部分成功文档（A14）

#### Scenario: Exit statuses are explicit

- **WHEN** 请求成功、用法/输入错误、名称歧义需要选择、或执行未完整
- **THEN** 退出状态分别为 0 与三个相互不同的非零值，原因可定位；执行未完整即使有可靠前缀也 MUST NOT 以 0 退出（验收 A14）

#### Scenario: Task chain reuses returned identities

- **WHEN** 调用方先列类、列方法取得物理身份，再用该身份发起恢复
- **THEN** 后续命令按该身份执行，读取范围与库操作一致，不要求手工重拼或重新查找（验收 A16）

#### Scenario: Incomplete selection is an incomplete command

- **WHEN** 库操作返回名称选择未完成，可靠候选可以是零个或一个
- **THEN** CLI 的 JSON 与库结果同源并保留停止证据，文本呈现同一事实；命令退出 4，不映射为成功 0、输入错误 2 或确定歧义 3，也不自行选择候选继续恢复

#### Scenario: Library and CLI agree on a stopped class view

- **WHEN** 类视图包含一个停止的方法体和一个正常方法体
- **THEN** 库的顶层执行状态已体现未完成，CLI 据该报告以未完整状态退出；直接库调用与 CLI 得到相同的逐方法事实，停止判定不只存在于适配器中

