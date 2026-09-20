# demand-resolver Specification

## Purpose

在明确的 RuntimeView、loader domains 和平台 Header providers 中按需解析声明及已知派发候选，保留缺失、歧义和开放世界边界。

## Requirements

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

#### Scenario: Delegation changes the hierarchy lookup loader

- **WHEN** ChildFirst 的 child 委托 parent 找到 Owner，Owner 的父类 Base 在 child 和 parent 均有定义
- **THEN** 从 Owner 的 defining loader 解析 Base，返回 parent Base 的 loader/物理声明；不得重用请求初始 child loader 而返回 child Base 并声称完整

#### Scenario: Same name is not the same hierarchy node

- **WHEN** 闭包、访问检查或已知范围 dispatch 遇到同名但 defining loader/物理定义不同的类
- **THEN** 查找 memo 保留 initiating loader，已解析节点、环检测、祖先比较和读取去重保留定义身份，不仅按 owner 字符串合并或建立继承关系

#### Scenario: Physical caller or driver has no declared runtime binding

- **WHEN** caller 或分析目标的物理定义虽然在 content 中，却不能绑定到声明 loader 的 root/选择结果
- **THEN** 明确报告绑定缺口，不把该定义直接记为调用方 loader 并继续运行时语义分析；合法依赖可来自不同 snapshot，不能用 snapshot 相等作为替代校验

已知范围 dispatch 的候选规则是**结构性**的：范围内自身声明同 kind/name/descriptor、且声明 owner 严格位于其超类型路径之上的类即候选，不筛 private/static/abstract 标志，也不排除 `<init>` 之类特殊名字——调用方必须结合 `open_world` 与候选证据判断，不得把候选集合读作已按规则筛选的运行时目标集合。解析的已知语义边界（P2 的显式近似，不得被读作 JVMS 的完全实现）：maximally-specific 集合按 JVMS 5.4.3.3/5.4.3.4 排除 `ACC_STATIC`/`ACC_PRIVATE` 的接口声明；interface owner 不隐式继承 `java/lang/Object` 的方法（未命中即 Missing）；default conflict 在解析期即报告（JVMS 8 把它放在 invocation selection 阶段）；只检查成员自身声明的可访问性，不检查声明类的可访问性（JVMS 5.4.3.1）；`InvokeDynamic` 的 owner 只是搜索起点，不施加调用种类规则；同一 owner 内同名同描述符的重复声明只能表达为无法区分的候选（Ambiguous）。

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

### Requirement: Container lookup obeys the declared search position

对已显式声明的 container，类名定位 SHALL 只访问解析该位置必需的祖先链、该 container 的目录与实际候选；MUST NOT 为单位置查名展开未搜索的 sibling 或 descendant container。冷路径允许为证明候选集合完整而读取该 container 的完整中央目录。查名得到 Missing 或 Ambiguous 之前 MUST 证明对应位置的目录完整；局部成功不能声称整棵 artifact tree 完整。

#### Scenario: Unrelated nested sibling

- **WHEN** 调用方只声明一个 nested container 为查名位置，同一外层归档还含大量 sibling JAR
- **THEN** 查名读取该位置和必要祖先，不展开 sibling；新增 sibling 内容不增加 sibling 物化量，候选选择和顺序保持不变

#### Scenario: Incomplete selected directory

- **WHEN** 被搜索 container 的目录因损坏、预算或取消未完成，已读前缀没有目标名称
- **THEN** 返回未决范围及实际停止原因，不返回完整 Missing，也不转向后续 root 伪造唯一命中

#### Scenario: Unrelated damage remains outside local coverage

- **WHEN** 未搜索 sibling 已损坏，但被搜索 container 及其祖先可完整读取
- **THEN** 局部请求能完成其声明范围；显式整树请求仍报告坏 sibling，不将局部结果扩大成整树健康声明

### Requirement: Locating and reading share live container facts

单次操作的定位、绑定复核与所选 entry 读取 SHALL 消费同一组仍被该操作持有的完整权威目录和验证 backing，不得仅因阶段切换重建相同事实。此行为 MUST 不依赖跨请求 cache 已启用；操作结束释放局部持有，所选 entry 的身份及内容完整性检查仍须成立。

#### Scenario: Cross-request retention is disabled

- **WHEN** 跨请求保留关闭，当前操作已取得目标 container 的完整目录与 backing，随后读取刚定位的 class
- **THEN** 操作复用仍持有的事实，不重新枚举该目录或重复展开父容器；实际 class 读取与校验继续按剩余预算执行

#### Scenario: Local reuse cannot validate caller-supplied metadata

- **WHEN** 当前操作持有完整目录，但调用方传入的 entry metadata 与权威记录不符
- **THEN** 拒绝不匹配身份，不能因为局部事实已存在就跳过 locator/metadata 检查

### Requirement: Explicit entry prefixes define container load positions

系统 SHALL 允许调用方以物理 container origin 和 raw entry prefix 声明一个加载位置。prefix SHALL 为空或以 `/` 结尾；查名以 `prefix + internal_name + .class` 作逐字节精确匹配，不归一化路径或改写物理身份，不要求 ZIP 中存在显式目录 entry。不同 prefix 属于不同运行环境声明；单独发现布局节点 MUST NOT 自动激活对应位置。Standalone CLASS 的身份读取保持独立，不用文件名推断内部名。

#### Scenario: WAR application class binding

- **WHEN** 同一 container 有 `WEB-INF/classes/com/demo/A.class`，请求显式声明 `WEB-INF/classes/` 并绑定其真实物理定义
- **THEN** 对 `com/demo/A` 的选择和 driver/caller binding 可以命中该 entry，结果保持原始带前缀 raw name、ordinal、snapshot 和 origin chain

#### Scenario: Prefix is absent or malformed

- **WHEN** 调用方只声明空 prefix 或仅提供布局识别结果，或声明非空但不以 `/` 结尾的 prefix
- **THEN** 空 prefix/布局识别不能自动绑定带前缀类；不合法的 prefix 返回环境诊断，不通过补分隔符猜测用户声明

#### Scenario: Raw bytes are not normalized

- **WHEN** 两个 entry 的 raw name 在大小写、非 UTF-8 字节、反斜杠或点段上不同
- **THEN** 仅精确匹配组合名称的 entry 成为候选，不因显示转义或路径规范化合并物理定义

### Requirement: Prefixed roots preserve definition selection discipline

前缀加载位置 SHALL 遵循既有 loader 委派与 root 声明顺序、重复项歧义、缺失和不完整范围语义。选中候选的 Header 内部名 MUST 与所请求名称一致；损坏或不一致候选不得通过继续搜索后续位置隐藏。改变 prefix 后不得复用旧环境的绑定或缺失结论；共享物理目录不等于共享解析结果。

#### Scenario: Root order and duplicates

- **WHEN** 同名类同时出现在带 prefix 的应用目录及 nested library，或在同一位置重复出现
- **THEN** 跨位置遵循调用方声明顺序；同位置重复保持各自 origin 并报告 Ambiguous，不按 archive 遍历顺序或内容摘要静默消歧

#### Scenario: Header name disagrees with the entry path

- **WHEN** 前缀匹配到 `p/A.class` 但其 Header 声明另一个内部名
- **THEN** 返回该物理候选上的不一致诊断，不把该文件绑定为 A，也不绕过它选取后续 root 的 A

#### Scenario: Environment changes after an unbound result

- **WHEN** 同一物理 snapshot 先以空 prefix 请求得到 unbound，随后显式声明正确 prefix
- **THEN** 新请求重新应用其环境并可建立真实 binding，不被旧 unbound/missing 结论污染

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
