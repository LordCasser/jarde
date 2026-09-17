## Purpose

在明确的 RuntimeView、loader domains 和平台 Header providers 中按需解析声明及已知派发候选，保留缺失、歧义和开放世界边界。

## ADDED Requirements

### Requirement: Explicit resolution environment

解析请求 SHALL 显式绑定不可变 snapshot、RuntimeView、平台/provider 身份、loader domain 映射和调用方上下文。系统 MUST 校验 root/parent/provider 映射，不得从宿主 classpath 或网络隐式补齐。没有运行环境的 physical X0/X1 请求 MUST NOT 启动 resolver 或方法 IR。

#### Scenario: Missing parent binding

- **WHEN** 请求包含 parent LoaderId 但未提供对应 domain，或 domain parent graph 有环
- **THEN** 返回可定位的环境/策略诊断和未完成的解析范围，不按扁平 root 列表产生唯一解析结果

#### Scenario: Unsupported runtime policy

- **WHEN** 请求使用未支持的 module mode、Custom/Unknown loader 或超出当前能力的运行 profile
- **THEN** 返回 UnsupportedPolicy 或明确的 unsupported 能力诊断，保留原始符号，不静默模拟 Java 8 classpath

### Requirement: Demand-bound symbol resolution

系统 SHALL 区分 Resolved、Missing、Ambiguous、Inaccessible、IncompatibleClassChange、UnsupportedPolicy、BudgetExceeded，并另行记录输入损坏、取消和实际 execution。缺失定义 MUST 保留 SymbolRef、descriptor 和 origin，不伪造空成员。解析 MUST 根据字段/方法种类、调用方、访问条件和 invocation kind 应用相应规则。

#### Scenario: Missing platform definition

- **WHEN** 引用需要未提供的平台 Header
- **THEN** 返回 MissingDependency 证据及未完成的解析覆盖，不把结构扫描完整当解析完整

#### Scenario: Ordered roots choose a definition

- **WHEN** 同名定义分布在有明确 ParentFirst/ChildFirst 策略和 root 顺序的不同位置
- **THEN** 按声明策略选择并返回 loader、物理定义和顺序依据，不仅因存在多个定义就报 Ambiguous

#### Scenario: Indistinguishable definitions

- **WHEN** 同一选择位置存在无法区分的重复定义，且给定策略不能确定唯一选择
- **THEN** 返回 Ambiguous 与各自 origin，不按 hash 或遍历偶然顺序覆盖

#### Scenario: Invocation kind affects resolution

- **WHEN** 同一 owner/name/descriptor 被不同字段/方法或调用指令种类引用
- **THEN** 独立检查访问、静态/实例、class/interface、构造器和 invokespecial 等适用规则；冲突或未支持语义明确报告，不用递归找同名成员代替规范解析

### Requirement: Declaration reference queries preserve symbolic evidence

系统 SHALL 提供携带运行环境、declaration identity 和显式扫描范围的声明引用查询。候选读取 MUST 复用结构 consumer，不能先按声明 owner 精确筛掉可能经继承解析到该声明的 use-site，也不能把未使用的 CP 常量当引用。结果 SHALL 分开保留原始 SymbolRef、物理 use-site、解析后声明和扫描/解析覆盖。

#### Scenario: Inherited declaration owner

- **WHEN** 范围内实际调用的 CP owner 为 Sub，foo 声明在 Base，查询目标为 Base.foo
- **THEN** 查询发现该调用并解析到 Base 声明，保留 Sub 符号和实际 BCI；P1 的 MentionsSymbol 仍按原始符号区分二者（验收 A11）

#### Scenario: Unresolved candidate cannot be excluded

- **WHEN** 已找到潜在 use-site，但依赖缺失或解析预算耗尽
- **THEN** 保留未决候选和不完整的解析覆盖，不能把它当已排除并报告完整空结果

### Requirement: Resolution and dispatch are separate

系统 SHALL 先解析声明，再在显式范围内返回 KnownCandidates；缺失依赖、外部子类或未知 loader/transformer 下，唯一已知候选 MUST NOT 被标为唯一运行时目标。

#### Scenario: Interface default conflict

- **WHEN** 接口调用在当前范围存在多个实现或 default conflict
- **THEN** 分开返回声明解析状态、已知候选和冲突/open-world 依据，保留 invocation kind（验收 A11）

### Requirement: Bounded Header closure

解析 SHALL 默认只扩展必要 Header，Body 升级必须有显式 reason。闭包 SHALL 受类数、方法数、依赖深度、字节、步骤、时间和取消约束，同请求中同一物理定义/loader 绑定去重；不同 origin/loader 不得因字节相同合并。目录或依赖读取停止后 MUST 保留 skipped/未决范围。

#### Scenario: Hierarchy expansion without unrelated bodies

- **WHEN** 单方法分析需要父类/接口，或显式 CHA 请求需要范围内候选 Header
- **THEN** 仅展开相应 Header 闭包与目标 Body；每次读取记录 reason/usage，不加载其他方法 Body（验收 A14、A16）

#### Scenario: Closure budget or cancellation

- **WHEN** 长继承链、循环引用或高扇出依赖到达限制，或请求被取消
- **THEN** 在下一次扩展前停止，区分 BudgetExceeded 与 Cancelled，保留可信前缀；依赖深度不得使用容器 nested_depth 代替
