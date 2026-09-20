## ADDED Requirements

### Requirement: Bounded environment policies are explicit and non-inferring

任务导向操作 SHALL 通过少量环境策略构造解析环境：单 `.class`（standalone snapshot 根）、plain JAR（该 snapshot 的真实 root container）与显式 classpath（调用方按顺序声明的 roots）。每个策略 MUST 显式声明 roots、delegation 与 module mode，并产生可由既有环境 validator 校验、与手写等价的环境；MUST NOT 从 Manifest `Class-Path`、layout 检出、嵌套库或宿主环境推断 classpath，也 MUST NOT 自动下载、预加载或激活任何依赖。策略不改变既有显式请求的语义，被 validator 拒绝的环境保持既有的不可用状态与原始符号。WAR 布局策略 MUST NOT 在本阶段提供；后续显式策略只允许按显式 root prefix 组织 `WEB-INF/classes/` 与嵌套库的加载位置，且 MUST NOT 声称复现容器的真实加载行为。

#### Scenario: Single class environment

- **WHEN** 调用方用单类策略对 standalone CLASS 发起恢复
- **THEN** 环境绑定该 snapshot 的 root，不伪造 container/entry 身份；报告与同一请求的手写环境一致

#### Scenario: Plain JAR environment

- **WHEN** 调用方用 plain JAR 策略
- **THEN** root 是该 snapshot 的真实 root container，扫描到的嵌套库不被自动激活；嵌套类仍需要显式的 artifact-tree root（验收 A08）

#### Scenario: Explicit classpath order decides

- **WHEN** 调用方给出显式 root 顺序，且同名定义分布在不同的已声明 root
- **THEN** 按声明顺序选择并保留选择依据，不因存在多个定义就报 Ambiguous（验收 A07）

#### Scenario: Layout evidence is not a load policy

- **WHEN** 范围是 WAR/Boot 布局且调用方未显式声明 prefix 或 root
- **THEN** 策略不生成对应 roots，报告保持未绑定/未解析状态，不按路径或布局检出猜测加载位置（验收 A07、A14）

#### Scenario: A layout policy is not claimed

- **WHEN** 调用方请求本阶段未提供的 WAR 布局策略
- **THEN** 以明确 unsupported/无效输入作答，不返回一组声称等价于容器加载的 roots
