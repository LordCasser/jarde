## MODIFIED Requirements

### Requirement: Independent analysis planes
系统 SHALL 允许直接打开 artifact 请求事实或单个方法，不要求全程序加载、全局索引、持久数据库或预先反编译。Query 与 Decompiler MUST 共享输入解码和身份契约，但不得隐式互相启动；依赖默认只扩展 Header，Body 升级必须附理由和预算。任务导向操作 SHALL 在同一契约下组合这些平面：打开 artifact 自身保持轻量，组合操作 SHALL 在一次请求内复用同一次类读取与成员列举，并让每个方法保留自己的阶段结果、覆盖、执行状态与诊断，MUST NOT 为每个方法重新扫描、重新读取或重建已经取得的类事实。组合类视图的顶层 execution SHALL 汇总本次搜索、成员读取与所有已请求方法体的停止；任一请求部分未完成时不得发布顶层 Complete。各 coverage 维度 SHALL 仍按实际读取范围报告，不能因方法体失败而否认已完整读取的成员表。

同次组合操作在完成目标绑定和可信类准备后，随后选择的 driver 方法及必要的同类 callee SHALL 消费该次已取得的定义事实；对同一 selected physical definition，不得因从绑定进入准备、driver 解码或 callee 证据而再次读取或准备。未选择的方法和未被依赖的定义不得因此提前解码。该复用 MUST 保留每个方法独立的 stage、coverage、execution、diagnostics，并按既有报告边界发布完整 `Limits`、`UsageSnapshot`，不要求在每个条目重复复制共同配置；复用只能减少重复物化，不能把未请求的事实变成已读取或把局部停止改成 Complete。


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

#### Scenario: Selected driver and callee share one prepared definition

- **WHEN** 同次操作已绑定并准备一个物理类，随后恢复一个选定 driver 方法并按需检查该类中被它命名的 callee
- **THEN** driver 与 callee 消费同一可信类定义和成员事实，不重复读取或准备该 selected definition；每个读取 reason、body 费用和证据仍按实际消费者报告

#### Scenario: Unselected methods remain undecoded

- **WHEN** 类视图或组合操作只选择部分方法体及其必要 callee
- **THEN** 未选择方法不解码、不构造 IR 或 Java AST；为搜索候选而读取的声明事实与为已选 body 读取的事实分开报告，不能因共享准备把其它方法宣称为已分析

#### Scenario: Reuse does not collapse per-method status

- **WHEN** 共享准备后一个选定方法完整结束，另一个选定方法损坏、取消或耗尽共享预算
- **THEN** 每个方法保留自己的阶段、coverage、execution 与 diagnostics，顶层 execution 汇总真实未完成状态；完整 `Limits` 和 `UsageSnapshot` 仍按既有契约发布，复用不得隐藏重复费用或停止原因

#### Scenario: Repeated consumers over one immutable artifact

- **WHEN** 同一不可变 snapshot/view 上先完成目标选择，再追问 driver 正文或局部 callee 证据
- **THEN** 后续消费者复用仍持有的可信定义事实并保持身份、顺序和 coverage 一致；若事实已释放，才可在新预算内重建并记入当前请求，不得把旧请求的费用当作当前命中
