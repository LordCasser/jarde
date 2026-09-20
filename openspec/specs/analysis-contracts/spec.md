# analysis-contracts Specification

## Purpose
让库调用方和命令行用户准确解释分析结果的来源、覆盖边界和终止原因，避免把未扫描、取消、解析失败或不支持的内容误解为不存在或已经验证成功。本契约覆盖 artifact 枚举与物理导航、classfile Header/bytecode inspection、结构 XRef、显式运行环境下的解析与声明引用、方法分析报告，以及任务导向组合操作和 CLI。方法分析自身交付 IR 与独立报告平面；恢复入口另行交付 Java 呈现及其 content、quality、coverage 与 execution，不能把分析完成等同于恢复或语义验证成功。

## Requirements

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

### Requirement: Physical symbolic and resolved identities
系统 SHALL 区分物理定义、SymbolRef 和运行视图下的 ResolvedDefinition；物理定义身份包括 class bytes 身份、物理变体，以及显式的 standalone snapshot root 或真实 archive entry。Archive entry 身份 SHALL 保留 snapshot、当前容器、entry ordinal/raw name，以及逐层记录“父容器中的哪个 entry 产生哪个 child container”的有向 origin chain。成员身份保留完整 descriptor。相同字节可共享解析 facts，但 MUST 不合并不同物理 origin 或 loader binding；系统 MUST NOT 为 standalone CLASS 伪造 ZIP entry，也不得用无法复核边的容器 ID 列表代替 nested origin chain。

#### Scenario: Identical bytes at distinct origins
- **WHEN** 两个 entry 含完全相同的 class 字节但位于不同位置
- **THEN** 内容摘要可以相同，物理来源仍可独立定位，不能因解析缓存复用而合并定义

#### Scenario: Symbol without definition
- **WHEN** classfile 引用了没有提供定义的成员
- **THEN** 原始 owner/name/完整 descriptor 保留；定义解析和运行时派发不能从该符号的存在自动推断

#### Scenario: Standalone class root identity
- **WHEN** 调用方打开 standalone CLASS 并为其建立物理定义身份
- **THEN** 身份绑定真实 snapshot root 和 class bytes，不创建虚假的 container、entry ordinal 或 archive name

#### Scenario: Duplicate nested archive entries
- **WHEN** 父容器中两个不同 ordinal 的同名 entry 各自产生一个 child container
- **THEN** 两条 origin chain 分别保留其父 entry ordinal/raw name 和 child container ID，内层同名定义不得因容器名称相同而合并

### Requirement: Provenance and execution are explicit

系统 SHALL 为枚举、artifact-tree、Header、bytecode、解析/闭包和方法分析返回快照/entry 身份及适用的 container origin、class offset 或方法 BCI，并分别返回 coverage、execution、diagnostics 和预算消耗。非累加的高水位预算维度 SHALL 进入 limits、usage 和 BudgetExceeded 结果：`nested_depth` 表示已接受的容器嵌套深度（root container depth 为 0，直接 child 为 1），`dependency_depth` 表示已接受的依赖闭包深度；两者相互独立，任一维度不得代替或改写另一维度。任何计费维度 SHALL 同时出现在 `Limits`、`UsageSnapshot`、终止维度枚举以及库与 CLI 两个请求 schema 中，请求 schema 保持显式——缺失维度是协议错误，不静默取默认值。

#### Scenario: Cancelled enumeration

- **WHEN** ZIP 根容器已经建立，且枚举开始前或过程中收到取消
- **THEN** 返回 `Ok(EnumerationReport)`，execution 为 Cancelled，并保留已完成的可靠前缀及其 coverage，不能将其标为 Complete

#### Scenario: Partial or failed enumeration after root open

- **WHEN** ZIP 根容器已经建立，枚举过程中耗尽预算或后续 entry 结构损坏
- **THEN** 返回 `Ok(EnumerationReport)` 并保留已验证前缀；预算耗尽为 Partial 和对应 BudgetExceeded 维度，结构损坏为 Failed 和错误 diagnostic，artifact coverage 为 Partial，未完成范围明确标为 skipped

#### Scenario: Root container cannot be established

- **WHEN** 输入不是 ZIP，或 ZIP 根容器无法建立
- **THEN** 枚举可以返回 `Err`，不得伪造可枚举的根容器或部分前缀

#### Scenario: Nested depth is a high-water limit

- **WHEN** artifact-tree 已接受 depth 1 的 child，随后请求 depth 2 而 `nested_depth` limit 为 1
- **THEN** usage 的 nested depth 高水位保持 1，execution 为 Partial 且 reason 指向 `nested_depth`，depth 2 child 标为未扫描而不是累加消费或空成功

#### Scenario: Dependency depth is an independent high-water limit

- **WHEN** 同一请求已接受容器深度 2（`nested_depth` limit 为 3），随后依赖闭包扩展到深度 5 而 `dependency_depth` limit 为 4
- **THEN** execution 为 Partial 且 reason 指向 `dependency_depth`，保留已接受的容器深度与依赖深度 usage；`usage.nested_depth` 与 `usage.dependency_depth` 分别记录各自高水位，任一维度超限不改写另一维度的 usage 或判定

### Requirement: Independent evidence and coverage dimensions
结构引用 SHALL 分开表示 relation、derivation、resolution、applicability 和 validity。结果 MUST 分开描述 artifact structural coverage、runtime resolution coverage、dynamic analysis coverage、execution 和分页；“对已声明 schema 完整”不得扩张为理解任意未知 attribute 或完整运行图。

#### Scenario: Complete structure with missing runtime definitions
- **WHEN** 声明范围内结构扫描完整，但运行时定义缺失或未请求解析
- **THEN** 结构覆盖可为 complete-within-schema，解析状态为 Missing/NotRequested，不能用一个 supported/complete 布尔值替代二者

### Requirement: Pure Rust library first
系统 SHALL 提供同步、可取消的库 API 和调用相同语义的 JSON CLI；运行不需要 JVM、外部反编译器、网络或持久索引。

#### Scenario: Offline inspection
- **WHEN** 未安装 JDK 的调用方打开本地 CLASS/JAR 并检查某方法
- **THEN** 库及 CLI 均可完成声明能力内的读取，且不会启动外部进程

### Requirement: Honest release matrix
项目 SHALL 分别列出 parse、X1、resolution、decompile-quality、output-level 的已实现状态；未完成阶段 MUST 保留未完成标记。

#### Scenario: P0 release
- **WHEN** 用户查看本阶段能力
- **THEN** 文档声明 Header/bytecode 能力及其限制，并将 X1、resolver、Java recovery 标记为未实现

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

### Requirement: Diagnostic codes are version-scoped identities

诊断码 SHALL 标识「该引擎版本在本次运行中记录了哪个事实」，MUST NOT 被读作跨版本可比的量。同一个码在不同版本的含义变化（例如声明事实交接前 `jre_declaration_class_not_in_run` 在单个 artifact 上出现 10,720 次、交接后语料全局 0 次，而 `jre_declaration` 出现在每个 artifact 上）MUST 被登记为词汇/事实来源的变化；消费方、比较报告与文档 MUST NOT 把某个码的出现次数下降直接读作质量或覆盖改善，比较 MUST 同时声明两个引擎版本与各自的 code 词汇。

#### Scenario: A declaration code changes meaning

- **WHEN** 两个版本或两份报告之间某个诊断码的出现次数发生变化，或一个码消失而另一个码出现
- **THEN** 报告与文档 MUST 说明这是词汇/事实来源变化而不是结论；比较 MUST 带上两个引擎版本与 code 词汇，MUST NOT 只给出「从 N 次降到 0 次」的差值

#### Scenario: A code count is not a quality metric

- **WHEN** 报告按码或按 artifact 呈现诊断计数
- **THEN** 这些计数 MUST 被表述为某声明版本下记录的事实数量，MUST NOT 被表述为正确性、覆盖或语义改善；某个码缺席 MUST NOT 在没有版本与词汇上下文时被当作对应风险消失的证据

#### Scenario: Diagnostic codes are stable within one version

- **WHEN** 同一引擎版本对同一受控输入的重复运行
- **THEN** 诊断码集合与计数保持确定；本 requirement 要求的是把码读作带版本的词汇身份，不改变任何码在单个版本内的含义或确定性
