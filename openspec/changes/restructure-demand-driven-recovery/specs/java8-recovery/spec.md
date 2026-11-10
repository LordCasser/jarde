## MODIFIED Requirements

### Requirement: One local variable's type is decided once, before its statements are built

一个局部变量（既有 `LocalVariable` 身份）的类型 MUST 在**语句被构建之前**、在既有的声明规划里决定一次，形成一个按该身份索引的类型结果，并由提升声明、就地声明、赋值、条件与返回共同消费；这些位置 MUST NOT 各自再作一次类型判定。
系统 MUST 遵守本 requirement 下所有“完整契约”场景中的全部约束。

#### Scenario: 完整契约 1

- **WHEN** 系统处理既有 LocalVariable 身份的类型决定、声明规划与语句构建
- **THEN** 系统 MUST 满足以下全部约束：

一个局部变量（既有 `LocalVariable` 身份）的类型 MUST 在**语句被构建之前**、在既有的声明规划里决定一次，形成一个按该身份索引的类型结果，并由提升声明、就地声明、赋值、条件与返回共同消费；这些位置 MUST NOT 各自再作一次类型判定。同一个值在两条声明路径上 MUST 得到同一个类型：写在包含全部使用的区域起点上的**提升**声明与写在首个写入处的**就地**声明 MUST 一致，结论 MUST NOT 取决于遍历顺序（同一变量的两个分支先后不得改变其类型）。

#### Scenario: 完整契约 2

- **WHEN** 系统处理既有 LocalVariable 身份的类型决定、声明规划与语句构建
- **THEN** 系统 MUST 满足以下全部约束：

决定的证据 MUST 来自有限且可复核的约束域：原始 descriptor、Frame、已证明局部复制、已校验泛型声明沿实际继承路径的类型实参替换、引用赋值与全部使用位置的要求，以及入边相容的引用 phi。引用关系 MUST 绑定同次解析环境中的定义身份；源码变量分组 MUST 在类型决定前完成，拆分物理槽须有独立生命周期与使用边界证据。boolean 与窄原语继续遵循已声明转换规则，MUST NOT 从任意值链或引用约束推断 boolean。`0`/`1` 字面量 MUST NOT 独立发起 boolean 判断；目标已有合法 boolean 约束时按 `true`/`false` 适配。每个写入及使用 MUST 按共同类型决定核对，MUST NOT 只检查第一次写入或只检查声明。推导 SHALL 有界并接入同次预算与取消；不完整依赖、未决类型、已证明不兼容和执行停止 MUST 可区分，MUST NOT 通过任意 cast 或猜测声明掩盖冲突。

#### Scenario: 完整契约 3

- **WHEN** 系统处理既有 LocalVariable 身份的类型决定、声明规划与语句构建
- **THEN** 系统 MUST 满足以下全部约束：

类型未知（没有可决定的类型）或冲突（写入与已决定的类型矛盾）时，本层 MUST 终止该结构的生成，而不是发布一个猜测：受影响结构按既有 refusal 契约处理（保留 bytecode 与 origin、representation=Mixed、quality=Fallback、诊断点名相关 BCI 与物理方法），MUST NOT 发布类型矛盾的赋值（例如 `int local3; … local3 = <boolean 值>; …`）。声明的结果 MUST 区分「没有声明到期」与「声明失败（fallback 已写）」两种含义；后者之后 MUST NOT 继续写出那条赋值。本要求 MUST NOT 被读作类型保真或语义等价的声明：字面量单独构成的提升声明按其与就地路径一致的既有答案呈现，这一类型差异 MUST 被记录为边界。


#### Scenario: The two declaration paths decide the same type

- **WHEN** 一个变量由提升声明定型（它的首个写入不在包含全部使用的区域起点上），而该写入存放的值是一次对**本 body 已证明 boolean 的局部**的读取（源码形状 `boolean a = b; boolean c; if (n == 0) c = a; else c = b; if (c) …`）
- **THEN** 该声明 MUST 与就地声明路径对同一个值的决定一致：呈现为 `boolean localN`，MUST NOT 出现 `int local3; … local3 = <boolean 值>;`；把正文放进成员自己的签名后 javac MUST 接受，`boolean cannot be converted to int` 不得出现（验收 A13）

#### Scenario: The conclusion does not depend on the traversal order

- **WHEN** 同一个方法里的两个形状以交换后的分支顺序出现（`if (n == 0) c = b; else c = a;` 相对 `if (n == 0) c = a; else c = b;`），或同一变量在两条路径上分别被提升与就地声明
- **THEN** 该变量的类型、它的声明形态与它在每个使用处的拼写 MUST 与交换前一致（只有分支文本本身按源码位置不同）；依赖遍历顺序作出不同结论 MUST 使验收失败并指名该变量与相关 BCI（验收 A13）

#### Scenario: A copy chain through a boolean local keeps it boolean

- **WHEN** 提升变量的值经另一个**本 run 已判为 boolean 的局部**间接得到（源码形状 `boolean x; boolean y; if (b) { x = b; y = x; } …`），链长在有限集合内
- **THEN** 传播 MUST 让链上的每个变量与其第一个变量同型（`boolean`），MUST NOT 因为「证据在看某个写入时还没写出来」而退化为帧的 `int`；链上的每个写入同样受一致性检查（验收 A13）

#### Scenario: Every write is checked against the decision

- **WHEN** 同一变量的不同分支各有一个写入（`if (n == 0) c = a; else c = b;`，或某个分支写入一个不能拼成该类型值的形态）
- **THEN** 每一个写入 MUST 按已决定的类型检查，MUST NOT 只检查第一次写入；与决定冲突的写入 MUST 终止该结构（见「A conflicting or unknown type terminates the structure」），MUST NOT 被静默写入（验收 A13）

#### Scenario: A conflicting or unknown type terminates the structure

- **WHEN** 某个写入的值与已决定的类型矛盾，或没有任何写入能决定出可拼写的类型
- **THEN** 受影响结构 MUST 按既有 refusal 契约终止：保留 bytecode 与 origin、representation=Mixed、quality=Fallback、诊断点名相关 BCI 与物理方法；MUST NOT 发布猜测的类型、MUST NOT 为了「保持一致」而临时换一个更强的类型，也 MUST NOT 只记录日志后继续发射（验收 A13）

#### Scenario: A failed declaration is not followed by its assignment

- **WHEN** 一次局部声明因未知或冲突而不能成立（若把这种结果与「没有声明到期」混淆，调用方可能继续写出赋值）
- **THEN** 两种含义 MUST 被区分，且「声明失败」之后 MUST NOT 继续写出那条赋值；产物 MUST NOT 出现一个没有声明却被赋值的名字，也 MUST NOT 出现与拒绝原因矛盾的语句（验收 A13）

#### Scenario: The propagation stays inside the run's budget and cancellation

- **WHEN** 证据传播（工作队列）在本次运行的预算或取消检查下走到停止
- **THEN** MUST 发布既有停止形态（`outcome = Stopped`、非 `Complete` 的 `execution`、诊断点名原因与位置、`content = not_produced`），MUST NOT 继续构建、继续传播或提交部分产物；规划本身 MUST NOT 新增预算维度、MUST NOT 跳过既有计费（验收 A13、A14）

#### Scenario: The literal does not initiate the decision

- **WHEN** 提升声明的首个写入值是 `0`/`1` 字面量，且没有别的 boolean 证据（源码形状 `boolean x; if (b) { x = true; } else { x = false; } if (x) …`）
- **THEN** 两条声明路径 MUST 对同一个值给出同一个答案（保持 `int`，其使用按整数形态拼写为一个自洽、可编译、两侧执行一致的文本）；MUST NOT 重新接纳字面量作为声明的发起证据，也 MUST NOT 声称该形状的类型已与源码一致——这一差异 MUST 被记录为边界（验收 A13）

#### Scenario: The decided type is consumed at every site

- **WHEN** 一个已决定的 `boolean` 局部同时出现在自己的声明、后续赋值、条件与 `Z` 方法的返回处
- **THEN** 各处的拼写 MUST 与该决定一致（`boolean localN;`、`localN = true;`、`if (localN)`、`return localN;`），MUST NOT 在任一位置退回帧的 int 拼写或与 `0` 比较；这些位置 MUST NOT 各自重新判定该变量的类型（验收 A13）

#### Scenario: The existing controls keep their types

- **WHEN** 修正后运行由 boolean 参数或返回 `Z` 的调用提升的局部，以及一个只被 `0`/`1` 填充的 `int` 局部
- **THEN** 前者 MUST 保持 `boolean`（既有 descriptor 证据路径），后者 MUST 保持 `int`，两者继续通过编译执行对照；本修正 MUST NOT 用普遍改写成 boolean 或普遍拒绝换取反例通过（验收 A13）

## ADDED Requirements

### Requirement: Runtime selection and source language target are independent

恢复请求 SHALL 独立声明运行时选择环境与源码语言目标，并在结果中分别回显。源码目标 SHALL 仅约束生成语法，不改变物理定义选择、依赖搜索顺序或解析身份；纯物理解析和纯 JVM 分析不因源码目标改变而引入源码阶段或无关失效。未注册的源码目标 MUST 明确拒绝；目标不支持的结构只能使用已证明的合法替代表达或显式降级。

#### Scenario: Same physical method with two source targets

- **WHEN** 相同 snapshot、runtime/loader、物理方法与足够预算分别以源码目标 7 和 8 恢复
- **THEN** 两次选择的 class、MR 变体及共同查询符号的依赖解析身份 SHALL 相同；输出语法遵循各自目标，报告分别回显两类版本配置。不同呈现需要的额外事实读取必须有理由，不得改变共同符号的绑定

#### Scenario: Source target cannot express a lambda

- **WHEN** Java 8 输入中的 lambda 请求以源码目标 7 呈现，且没有已证明的合法替代表达
- **THEN** 结果 SHALL 保留可定位缺口及低级事实，不发射 lambda 后声称满足目标 7，也不改选另一个 class

#### Scenario: Source configuration cannot select platform APIs

- **WHEN** 只改变源码目标，保持运行时环境及已提供依赖不变
- **THEN** 平台与用户定义的选择 SHALL 不变；源码配置不引入未提供的 Header 或联网下载

### Requirement: Source types agree across declarations and uses

恢复 SHALL 在已声明有限类型域内共同检查局部、字段、参数、返回值与调用位置，保持源码声明、赋值、必要转换和重载选择一致。该域 SHALL 包含 null/具体引用合流、引用数组、已解析父类/接口关系、已声明泛型类与接口的实参传递。未决关系与已证明不兼容 MUST 区分；普通请求的类型恢复不自动取得编译或语义验证状态。

#### Scenario: Declared generic type flows through independent containers

- **WHEN** 同次事实证明泛型接口及实现的实参替换，元素经过字段、局部、参数和返回使用，输入分别采用平台集合和自定义泛型容器
- **THEN** 恢复 SHALL 在上述位置保持相容的具体类型与必要转换，不能依赖容器类名特判才成立；缺少证明的位置保留可定位缺口

#### Scenario: Reference joins preserve all incoming constraints

- **WHEN** 一个变量从 null 与可赋值引用入边汇合，并在多个有明确类型要求的位置使用
- **THEN** 声明 SHALL 满足所有入边与使用；调换分支遍历顺序不改变类型决定，不以首个写入替代全部约束

#### Scenario: Distinct lifetimes share a physical local slot

- **WHEN** 同一物理槽承载两个已证明互不重叠且没有跨界使用的源码变量生命周期
- **THEN** 恢复 SHALL 为两个生命周期分别形成一致声明；存在跨界使用或不可区分入边时不得仅靠拆名掩盖冲突

#### Scenario: Published declarations determine overload checks

- **WHEN** 泛型或局部声明的呈现改变了调用实参的源码静态类型
- **THEN** 调用 SHALL 按最终声明重新核对重载与必要转换；无法保持字节码所选调用语义时撤销相关转换或明确拒绝

#### Scenario: Missing hierarchy facts are not incompatible types

- **WHEN** 某个赋值关系因缺 Header 或歧义无法决定
- **THEN** 结果 SHALL 保留原始符号和缺失/歧义原因，不将其报告为已证明不兼容，也不按同名类猜测关系

### Requirement: Source variables satisfy scope assignment and edge value semantics

在承诺的可恢复域内，源码变量 SHALL 根据值流与生命周期形成合法作用域，声明支配其全部使用，每条到达路径满足 definite assignment，合流与循环回边的赋值保持原始值关系。debug 信息 SHALL 只作为受校验命名/范围线索；系统 MUST NOT 用默认初始化、仅改名或合并物理槽掩盖未证明的使用关系。

#### Scenario: Branch values are used after the join

- **WHEN** 一个值由不同分支定义并在合流后使用，所有到达边均可证明
- **THEN** 声明 SHALL 对各分支及合流后使用可见，赋值和读取合法；缺少必要到达值时明确拒绝，不能填默认值来通过编译

#### Scenario: Loop edge assignments exchange live values

- **WHEN** 循环回边的两个或更多活跃值通过 phi 边赋值交换或形成复制环
- **THEN** 源码 SHALL 保持同时复制的值关系，必要时使用合法临时值，不能因顺序覆盖而改变下一轮值

#### Scenario: Catch or captured values cross a source scope

- **WHEN** 候选使用 catch 参数、循环局部或被捕获值，并可能跨越其源码作用域
- **THEN** 系统 SHALL 仅在合法声明/捕获关系可证明时呈现；否则保留明确缺口，不把同槽或同名误作同一源码变量

#### Scenario: Debug metadata is removed or renamed

- **WHEN** code 与值流保持不变，LVT/debug 缺失或名称变化
- **THEN** 恢复 SHALL 保持相同赋值/使用关系与行为，允许确定性名称差异；无 debug 不豁免声明合法性

#### Scenario: Edge copies do not execute on sibling transfers

- **WHEN** 一个多后继前驱到多前驱合流点的特定边需要传递 phi 值，而兄弟路径不需要这些赋值
- **THEN** 生成源码 SHALL 只在所属边执行赋值，必要临时值不能覆盖其他活跃值；不能在共享前驱或合流处无条件执行这些复制，改变未经过该边的路径

#### Scenario: A handler observes locals before the throwing instruction

- **WHEN** 已支持保护范围内的指令可能抛错，handler 使用此前局部值，而正常完成路径产生后继值
- **THEN** 源码 SHALL 保持 handler 可观察的 throw 前局部状态与原保护范围；catch 参数仅在进入 handler 后绑定，不能用它生成 throw 前赋值；无法证明该关系时明确拒绝，不重算可能抛错的定义以补值

### Requirement: Expression transformations preserve a complete conservative baseline

对可用顺序语句和临时变量合法表达的恢复域，系统 SHALL 先保持效果完整的表示，再提交表达式合并。一次转换 MUST 共同核对消费位置、所有吸收效果、执行次数、控制与异常归属及来源；候选拒绝不丢失已经存在的有效表示。全局预算或取消停止 MUST 保持真实停止，不通过回退继续额外分析或发射。

#### Scenario: Rejected sugar preserves a legal sequential representation

- **WHEN** 前后缀更新、赋值值或多消费点候选不能合并，但同次已有顺序语句与临时值完整表达原效果且满足源码目标
- **THEN** 恢复 SHALL 交付该保守表示，保留旧值与更新的次数和顺序，不仅因糖化失败而删除效果或伪造默认值

#### Scenario: A consumer fails after an update was considered for absorption

- **WHEN** 候选计划吸收字段、数组或局部更新，随后其消费语句不能成立
- **THEN** 系统 MUST 撤销该候选的吸收和改写；原更新仍有可靠语句或明确低级呈现与 origin，不能留下一份静默少执行更新的源码

#### Scenario: Short circuit and throwing operands retain their positions

- **WHEN** 候选涉及短路分支、调用、数组访问或可能抛错的操作数
- **THEN** 恢复 SHALL 保持条件执行及异常顺序，不为增加临时变量而提前执行分支效果，不重复求值左值地址或移动至另一 handler 范围

#### Scenario: Global stop is not a local candidate refusal

- **WHEN** 候选检查或输出准入发生预算耗尽或取消
- **THEN** 请求 SHALL 停止并报告原始原因，保留按已有合同已经提交的内容；未提交候选不得部分发布，也不得在停止后重新发射基线

#### Scenario: Throwing operands preserve the first observable failure

- **WHEN** 接收者、索引、RHS、cast 或调用实参在不同受控输入下分别抛错
- **THEN** 转换 SHALL 保留原求值次数、此前可见效果、首个异常及其 handler 归属；不能因正常输入返回相同就接受改变异常顺序的候选

#### Scenario: Numeric or lifecycle boundaries constrain a transformation

- **WHEN** 候选涉及窄化、溢出、装箱拆箱、浮点值、类初始化、volatile、monitor 或未初始化对象
- **THEN** 系统 SHALL 按实际涉及的操作保持原数值和生命周期语义，包括适用的 NaN/有符号零；没有足够证明的域保持原有表示或明确拒绝，不以普通纯表达式规则放宽

#### Scenario: A legal sequential baseline is available without expression sugar

- **WHEN** 承诺域的局部更新/赋值值多消费或普通字段/数组/调用参数组合已有合法类型、变量、控制及异常边界，表达式糖化尚未支持或被拒绝
- **THEN** 系统 SHALL 能交付完整的顺序语句及必要临时值，保持每次接收者/索引/RHS/参数计算的次数和先后、旧值、窄化与首个异常；不能仅因没有漂亮表达式退回整方法 fallback。未证明的特殊生命周期或类型域仍按原边界拒绝

### Requirement: Structured regions compose through owned entries and exits

在可约控制流、闭合普通 named catch、可证明入口与延续的域内，循环、分支、catch 及 break/continue/return/throw SHALL 根据实际块归属、出口目的地、跨边界值与异常范围组合。组合 MUST 保留操作次数、执行顺序与异常处理优先级，不以某个子结构单独成功推导整体成功。

#### Scenario: Supported loop and catch structures are nested

- **WHEN** 已支持循环与分支分别出现在普通 catch 的保护体或处理体中，且归属、入口、延续和异常边界满足声明域
- **THEN** 结果 SHALL 保留完整组合及非局部退出；增加无关前后语句或改变变量名字不使同一合同退回样例专用路径

#### Scenario: Transfer targets a particular enclosing loop

- **WHEN** 子区域通过 break 或 continue 指向特定外层循环，或经正常路径 return/throw
- **THEN** 呈现 SHALL 保持原目的地和先行效果；不能将外层退出误写成内层退出或丢失返回前操作

#### Scenario: Foreign entry or overlapping ownership invalidates composition

- **WHEN** 外部边进入子区域内部、两个候选重复拥有块、出口未决，或异常范围不满足组合条件
- **THEN** 系统 SHALL 拒绝该组合并保留低级事实和原因；候选失败不能污染后续归属或发布只覆盖部分实际路径的成功结构

#### Scenario: Nested regions retain handler and transfer semantics

- **WHEN** 域内区域组合包含多个 handler 或经过已支持的 finally/TWR/monitor 子结构
- **THEN** 系统 SHALL 保留 throw site 的保护范围、handler 优先级及 return/throw/清理行为，跨区域值仍满足变量和效果合同；未支持嵌套保持拒绝，不能由单个子结构成功推导整体成功

### Requirement: Class projections commit declarations and affected uses together

类级恢复 SHALL 在所声明的覆盖范围内共同核对声明、调用/创建点、隐藏成员、捕获和初始化顺序，然后提交一项完整投影。未证明所有相关使用、外部可见性或构造效果时，MUST 保留有效原成员或显式拒绝。源码投影 MUST NOT 删除原始物理事实与查询关系。

#### Scenario: One anonymous class has multiple allocation sites

- **WHEN** 同一匿名类有多个已纳入范围的创建点，其中一个点的捕获或构造顺序无法证明
- **THEN** 系统 SHALL 撤销整项相关匿名投影，保留各点有效原表示或缺口，不只隐藏声明而留下未改写引用

#### Scenario: Generic declaration and callers form one projection

- **WHEN** 类级恢复准备改变泛型方法或字段声明，且相关调用或赋值受该变化影响
- **THEN** 声明与所有受影响的已覆盖使用 SHALL 一起通过类型和调用检查后提交；任一失败不产生半项改写

#### Scenario: Synthetic member may have callers outside the covered range

- **WHEN** accessor 或 bridge 在当前范围没有剩余调用，但外部可见使用尚未排除
- **THEN** 恢复 SHALL 保留该可访问成员或明确受限输出范围，不将局部无引用报告为全局可删除

#### Scenario: Initializer projection preserves construction and exception order

- **WHEN** 字段初始化准备从静态初始化器或构造器中提取
- **THEN** 只有类初始化、构造链、可见效果与抛错顺序保持时才提交；失败保留原初始化位置，未隐藏的调用继续可解析

#### Scenario: Projection stops before commit

- **WHEN** 类级转换在读取相关使用、检查依赖或提交前耗尽预算或取消
- **THEN** 结果 SHALL 保留真实停止和已提交内容，未提交转换不得局部隐藏声明、修改调用或丢失物理来源

#### Scenario: Independent committed projections survive a later member failure

- **WHEN** 一项闭合投影已经提交，另一项互不依赖的成员或候选随后失败或停止
- **THEN** 系统 SHALL 保留允许交付的已提交内容及其来源并如实报告 partial；同一投影中相关使用失败则不能发布半项改写，全局停止后不读取新 Body 或重建整类基线

#### Scenario: Projection candidates overlap in edits or hidden members

- **WHEN** 两个候选准备改写同一声明/使用区间或共同隐藏一个成员
- **THEN** 系统 SHALL 在提交前检查兼容性和全部受影响使用，冲突候选不得交付相互矛盾的半项结果

#### Scenario: The observable domain is narrower than all Java behavior

- **WHEN** 使用闭包仅覆盖已声明类族或受控调用，ABI、反射、序列化或类名可观察性尚未验证
- **THEN** 结果及验收 SHALL 明确该范围，不把当前无引用、依赖缺失或扫描停止当作全局不存在使用，也不宣称生成包可直接替换原应用

### Requirement: Recovery inputs and entry contexts preserve trusted provenance

恢复入口 SHALL 以同次读取的物理方法、事实、环境与配置形成可核对决定。相同事实和等价词法/投影上下文下，不同入口 SHALL 作出一致语义决定；可选 evidence 的选择不改变已决定源码。公开底层入口接受的外供断言 MUST NOT 冒充本次读取证明，矛盾输入不得用于可信产物绑定。

#### Scenario: Equivalent method and class contexts are compared

- **WHEN** 方法、bulk 与类入口具有相同物理事实、完整环境、源码目标和等价词法/投影上下文及足够预算
- **THEN** 语义决定 SHALL 一致；类级声明折叠、缩进或匿名投影造成的合法文本差异有明确来源，各 map 绑定各自文本，不要求类单元与方法正文逐字相等

#### Scenario: A caller supplies inconsistent facts or proof context

- **WHEN** 底层恢复入口收到与实际方法/声明/环境矛盾的 facts、subject 或能匹配消费位置但未经该可信路径证明的输入
- **THEN** 系统 SHALL 拒绝矛盾输入，或明确隔离为不能参与可信决定和 evidence 绑定的来源；不能仅因字段可构造就把它报告为已验证事实

#### Scenario: Optional evidence stops after source commitment

- **WHEN** 相同恢复配置产生已提交正文，但额外选择的 evidence 在物化时预算停止
- **THEN** 核心正文及主体身份 SHALL 保留其原决定，证据载荷与请求 execution 如实报告停止；不得修改正文或把未完成证据标为完成

#### Scenario: A family projection needs origins without requesting a public map

- **WHEN** 等价类级上下文和充分预算下，essential、source_map-only、all 三种选择进入需要精确 BCI origins 的类族投影，分别覆盖有与无 projected emission 的路径
- **THEN** 必要证明 SHALL 独立于公开 map 发布按需取得，三者保持同一有效类族决定、正文和完整类编译/受控行为；真实缺失证明仍拒绝，不能以强制 all、常驻整套 AST 或删除有效类族恢复满足此合同

#### Scenario: Raw JVM names cross the recovery boundary

- **WHEN** 名称/descriptor 含合法 supplementary 编码、源码不可直接表达名称或损坏 MUTF-8
- **THEN** 精确身份与关系判断 SHALL 保留原始 JVM 字节；合法编码正确解码，不可拼写名称用有 raw evidence 的一致别名或明确拒绝，损坏输入按错误边界处理，不能用 lossy display 替代语义事实

#### Scenario: Selected parent definitions and Override use faithful source identifiers

- **WHEN** 注册的Java7/8目标下，直接Override方法或已选泛型父类/接口的名称含合法supplementary MUTF-8或单独`_`，其余既有证明边界成立
- **THEN** Java层 SHALL 复用忠实方法名/类名出口；被选直接父定义的单一Object-bound类型参数名称 SHALL 严格解码并允许合法`_`，该检查 MUST NOT 改变Child自身类型变量或局部alias策略。原始名称/descriptor、物理定义、环境/loader、flags/arity/erasure、收费/停止和提交 SHALL 保持可核对；MUST NOT 用display或alias授予证明，也不扩张一般泛型算法。损坏MUTF、孤立surrogate、关键字与IdentifierIgnorable SHALL 分组报告；前置拼写拒绝不得计作后续helper覆盖。完整生成Child可依赖明确声明的父定义，但其编译/受控元数据观察 MUST NOT 被表述为整包或父定义自身源码恢复完成。

#### Scenario: Type parameter declarations and uses share faithful raw spelling

- **WHEN** 注册Java7/8目标的既有闭合证明允许当前类、方法或构造器发射泛型声明，类型变量名称含合法supplementary MUTF-8或单独`_`
- **THEN** Java层 SHALL 使用同一严格类型参数出口拼写声明及字段、参数、返回、throws、数组和bounds引用，按raw作用域保留class/method/constructor归属与遮蔽；类名仍走类名语法。MUST NOT 全局放宽局部alias或以display代替物理身份，MUST 保留既有flags、erasure、完整使用、预算/停止和原子提交守卫。关键字、IdentifierIgnorable、损坏MUTF、孤立surrogate和重复变量 SHALL 保留各自实际拒绝边界，不扩张一般泛型算法。完整受控包验证 SHALL 生成全部目标类并排除原目标后备，语言目标7不等于JVM7运行覆盖。

#### Scenario: Java owns the selected member call source-name policy

- **WHEN** 已选root/member事实进入既有类族调用证明，门面需要把raw名称交付为源码路径
- **THEN** Java层 SHALL 决定名称解码、该证明子域的源码资格与路径拼写；门面 SHALL 保留物理关系、capture/header事实、完整调用/IR、费用、停止与提交。纯职责迁移 MUST 保留原拒绝优先级与有效结果，不用更宽的忠实类名资格替换当前证明域，也不把child仅需解码的其他证明强行并入更强的名称helper。源码路径决定 MUST NOT 取代物理身份或BCI来源。

#### Scenario: Static target source checks preserve their staged prerequisites

- **WHEN** 已选static member target在完整物理关系和构造器IR后检查既有空构造源码子域
- **THEN** Java层 SHALL 决定无handler/精确Object初始化形状、原恢复结果/sidecar资格及名称路径，门面 SHALL 保留方法身份、analysis完成、实际读取/IR/recover、费用、停止和target物理装配。名称检查 MUST 保持在既有前置之后，child仅需严格解码的域 MUST NOT 被较强member-call名称helper收紧；纯迁移不改变有效产物、来源或拒绝范围。未发布target的终态 MUST NOT 自动计为某个内部helper的定点负例。

#### Scenario: Metadata source eligibility preserves physical completeness and ordering

- **WHEN** static target或整型常量候选的既有元数据准入职责迁至Java层
- **THEN** Java层 SHALL 接管源码资格和有序唯一性判断，门面 SHALL 保留实际读取、物理身份、完整表、计费、停止与提交。读前static/root资格、读后child资格、完整方法表之后的raw构造器选择 MUST 保持原时序；字段不完整或全局停止 MUST 清空整批常量候选。全部物理字段 SHALL 先参与名称唯一性判断，再仅对名称唯一候选统计重复值；源码文本 MUST NOT 取代raw物理身份。纯迁移 MUST 保留既有接受域；严格名称语义修复 SHALL 单独绑定新恢复版本，以完整生成类、隔离编译和受控行为/元数据验收，不能以仅decoder单测或前端拒绝替代消费者闭环。

#### Scenario: Integer constant source names retain raw field uniqueness

- **WHEN** 既有整型常量消费者收到合法supplementary MUTF-8字段名，且原声明、static final、I、单ConstantValue与完整表守卫成立
- **THEN** Java层 SHALL 使用同一严格名称出口，唯一性 SHALL 使用原FieldItem的raw名称，在排重名后再判重复值；source String MUST NOT 回编码替代物理key。原switch label、switch arm直接int return和int[]叶子 SHALL 忠实投影，保留Field/Method/BCI与UTF-8 byte spans；损坏编码、不可写名称、遮蔽、同值、不完整表和全局停止 SHALL 保持各自拒绝/停止边界。完整生成类 SHALL 原样编译并在隔离目标加载下核对字段名称/flags/value与受控行为，MUST NOT 以新增名称位置计作新可编译类或JVM7覆盖。

#### Scenario: Selected enum child source checks preserve physical prerequisites

- **WHEN** 已选enum constant anonymous child通过同次类读取与完整性检查，进入源码资格或group重检
- **THEN** Java层 SHALL 拥有声明/无字段资格、该既有子域的方法名与flags、唯一base后的可见性及同次core恢复可展示判据；门面 SHALL 保留物理选择、Code coverage、raw name/descriptor唯一配对、constructor/clinit分类、预算/停止和group提交。资格 MUST 保持原阶段、拒绝优先级及可选evidence独立性，MUST NOT 以源码bool代替physical/uses/constructor证明或引入crate反向依赖。纯迁移 SHALL 保持旧名称域和完整产物；扩域 SHALL 独立版本化与完整类验收。前置lowering/格式失败或最终未投影 MUST NOT 自动计作某个内部helper覆盖。

#### Scenario: Generic source consumers share strict class-name spelling

- **WHEN** 既有闭合证明允许同类 raw local receiver 或 List 的单层精确 class 元素类型恢复泛型声明，而物理名称含合法 supplementary 或字面 `$`
- **THEN** 源码 Type SHALL 复用共享严格类名出口，raw Signature/descriptor 与物理身份保持；真实成员路径仅依据同次 AST 的 raw InnerClasses 关系闭合，不以 `$` 猜嵌套，不因可选 evidence 改变正文。flags、完整使用、擦除或名称合法性未闭合时 SHALL 保留拒绝/擦除，局部泛型收益不得抹去独立 Partial 原因

#### Scenario: Explicit-bound casts keep physical type identity

- **WHEN** 完整同类overload证明需给直接type-variable参数添加显式bound的源码cast，bound含合法supplementary名称
- **THEN** AST Type SHALL 在共享严格命名出口解码，后续候选/most-specific比较的Signature class binary_name SHALL 保留原MUTF字节；不得将源码UTF-8回编码替代物理身份。候选不完整、bound/flags不符或类型名不能拼写时 SHALL 保留拒绝/擦除，原始Code与方法descriptor保持，不通过放宽关系推断获得名称支持

#### Scenario: An array helper keeps its physical target until source emission

- **WHEN** 已证明的数组构造器 helper、声明 owner、工厂方法或元素类型含合法 supplementary 名称，类级展示连接候选并匹配最终声明
- **THEN** 连接 SHALL 使用同次不可变 IR 的物理 site 与 raw handle，Java 层以共享命名/descriptor 解析完成最后拼写和声明匹配，不从展示 UTF-8 文本逆造物理身份；原 flags、捕获、完整使用和原子提交守卫保持，物理成员及公开方法报告不因类级省略 helper 丢失。损坏编码按 reader 错误合同交付可靠前缀与显式跳过范围，不伪造完整成员枚举

#### Scenario: A write accessor preserves its exact descriptor components

- **WHEN** 值返回型 synthetic/static 写 accessor 的 owner、字段值或数组元素含合法 supplementary 名称，现有闭合指令/SSA/字段证明成立
- **THEN** 参数 receiver、值和返回组件以及字段/header/frame/发射复核 SHALL 消费同一 raw descriptor 关系，仅在共享源码命名出口形成 Type；赋值与返回共同提交。改变 flags、owner、值/返回关系或字段 raw 类型不得借相似展示文本获准；损坏字段编码按 reader 可靠前缀和显式跳过范围报告，未完成的成员枚举不得声称 Complete

#### Scenario: A provisional lambda implementation has no committed source spelling

- **WHEN** 不可拼写的 raw implementation 被暂存为 lambda/helper 候选，但最终入口没有提交消除或一致重命名该引用的同次投影
- **THEN** 公共方法 SHALL 明确拒绝该引用，保留 raw owner/name/descriptor/kind 与 BCI/CP；类入口仅在实际编辑闭合后交付对应正文，不以 `lambda$` 前缀或可选 evidence 授权非法 Java 调用，并保留既有合法 helper 内联/重命名能力

#### Scenario: Name finalization quotes an entire method

- **WHEN** 名称终结因一个未闭合引用将整个公共方法替换为 bytecode quote
- **THEN** quote SHALL 保留同次完整指令包络，包括捕获生产者与其它效果；被丢弃的合法 lambda/accessor 也 SHALL 撤销已呈现状态，并与具体非法目标的拒绝原因分列。私有类候选可保留 provisional Program，但不得借此发布未闭合公共正文

#### Scenario: Two dynamic sites share a constant-pool entry

- **WHEN** 同一物理方法的两个不同 BCI 引用同一 invokedynamic CP entry
- **THEN** 最终消费与来源 SHALL 按物理方法及各自 BCI/CP 区分，不因共享 CP 合并 site、漏掉拒绝或提交不完整 caller 投影；公开方法 map 保留 quote 覆盖的完整指令范围

#### Scenario: Generic void policy owns selection and declaration generation

- **WHEN** 同次恢复的泛型void候选携带已解析Signature、physical erasure与InnerClasses事实
- **THEN** Java层 SHALL 统一决定该分支是否适用及选中后的声明/拒绝/停止，保留原Signature throws门与候选顺序；root只供给事实和提交完整声明。合法supplementary sibling bound SHALL 通过共享严格命名出口，raw同族/唯一row/物理binary拼写校验不得被source display代替；不适用不能提前生成或计费，拒绝和全局停止不能发布半项改写


#### Scenario: Method-local generic declaration policies retain selected refusals

- **WHEN** 恢复进入method-local empty-void throws、instance null-return或static local throws的既有源码声明候选
- **THEN** Java层 SHALL 拥有路由、严格准入与完整声明拼装，路由不适用与选中后拒绝 MUST 区分；root保留同次事实、候选顺序、停止和提交，既有有效声明、预算与取消无半提交能力 MUST 保留

#### Scenario: Empty-void throws requires the proved class bound

- **WHEN** method-local empty-void throws的Signature仅有interface-bound，旧shape因其名称等于JDK throwable而误把它当作class-bound
- **THEN** 有限源码策略 SHALL 返回局部拒绝并保留物理方法和可用正文，MUST NOT panic或生成无class-bound证明的泛型声明；合法显式class-bound继续按既有门恢复，三种evidence选择不改变正文

#### Scenario: Same-class generic body declaration policy consumes a bound certificate

- **WHEN** 同次同类调用证明提供与选中物理方法对应的body certificate，方法含method-local泛型Signature
- **THEN** root SHALL 核对certificate的物理subject并保持既有候选顺序、停止和完整投影提交；Java层 SHALL 基于已供facts负责bounds/formals/return sources/throws的源码准入、严格名称与descriptor拼写及声明组装，不新增读取或以DTO复制root状态。预算停止时已完整恢复的同一调用组件不得只提交部分泛型声明；可选evidence不改变正文

#### Scenario: Ordinary parameterized declarations keep their proof and spelling policy together

- **WHEN** Reader已验证普通参数化方法的Signature erasure，同次恢复或physical NoBody事实被选中用于源码声明
- **THEN** Java层 SHALL 负责严格准入、返回来源/throws/参数注解/varargs/修饰符、完整声明和proof分类，descriptor拼写 SHALL 复用同一实现；root保留物理certificate身份核对、同次事实、有序候选和停止/提交。迁移 MUST 保留原拒绝/错误与计费顺序，逐evidence核对正文/physical方法/完整coverage/logical usage；构建磁盘采样 MUST NOT 被当成实际read/IR/资源退出验收

#### Scenario: NoBody generic declarations keep physical proof separate from body proof

- **WHEN** 物理方法没有 Code 属性，且已通过既有层级、Signature erasure、作用域和调用绑定门进入泛型声明投影
- **THEN** Java层 SHALL 负责 NoBody 的层级源码准入、声明策略与完整拼写；root 保留同次物理事实、原调用与计费顺序、停止和提交。证明说明 MUST 对应实际 NoBody/Signature 事实，不声称已产生 Program/SSA；方法级现有 NoBody 说明保持。迁移 SHALL 逐 evidence 保留声明、物理方法、完整 coverage、费用、停止阈值和来源坐标；正文说明变化须更新恢复语义身份并绑定实际构建，不能用历史构建指纹代替当前产物

#### Scenario: Generic constructor source policy consumes the selected physical facts

- **WHEN** 方法局部或类作用域泛型构造器进入既有声明候选，候选可来自同次 constructor candidate 或同类 body proof
- **THEN** Java层 SHALL 拥有类作用域策略选择、完整准入、candidate 优先与 body-proof 回退、InitRecord/槽位/作用域/字段写入证明、raw 字段唯一匹配、完整声明及 proof 分类；root SHALL 保留物理 subject 核对、erasure/同类调用与委托证明门、原候选顺序、停止和提交。迁移 MUST 保留成功声明、物理参数、拒绝与 partial 前缀、poll/charge 顺序及预算停止阈值，不通过扩大恢复准入或复制 root 状态来完成职责调整；纯迁移的全文和 proof marker 必须逐 evidence 保持

#### Scenario: Selected generic member-family source policy keeps physical orchestration separate

- **WHEN** 门面已按物理关系、capture、调用闭包选定泛型成员类族，需要恢复既有 Outer<T>.Inner make 或 inherited T id(T) 声明
- **THEN** Java层 SHALL 接管源码准入、method-local scope拒绝与完整声明/proof分类；属性读取前、parse后erasure前、erasure后的原边界 MUST 保持。root SHALL 保留同次raw身份/读取/erasure、停止和原子提交，不为清marker复制完整method record。三evidence下的成功、prepared_fold回退、家族refused与execution partial MUST 保持各自原状态，不能将回退写成拒绝，或用可选evidence补足核心证明；正文/marker、物理snapshot、logical费用/limits、coverage及来源坐标保持

#### Scenario: Ordinary method declaration spelling consumes selected physical metadata

- **WHEN** 普通方法、构造器、class initializer、NoBody成员或interface成员已有同次descriptor、flags、参数和annotation事实，需要形成源码声明
- **THEN** Java层 SHALL 负责descriptor源码准入、名称/alias、flags/default方法、完整参数名/槽位/varargs/final证明核验、throws/default/type decoration与声明/marker；root SHALL 保留物理属性读取/身份、既有阶段次序、预算/停止和原子发布。现有参数事实类型可迁移并重导出，但纯文本helper MUST NOT 冒充可信artifact构造入口。迁移 MUST 保留成功、不可拼写、annotation保守拒绝和partial范围，三evidence的正文/proof marker、raw与physical snapshot、limits/费用、coverage/来源一致；未支持的数组type_path不能被定向Driver PASS掩盖成完整反射等价

#### Scenario: Generic member family header source selection uses supplied facts

- **WHEN** 门面已选择root/child物理声明与可用generic scope，检查既有单T的generic family header子域
- **THEN** Java层 SHALL 决定Signature/flags/父类/接口/名称/scope的源码准入，门面保留声明/scope可用性和物理关系/闭包/停止事务；原helper的两处调用时点、短路结果及prepared_fold回退 MUST 保持，不扩大接受域或增加读取

#### Scenario: Annotation source policy owns alignment and placement over supplied facts

- **WHEN** 同次 Reader 已提供 class/member/parameter/type annotation 的原始属性、值、target/info/path 和 descriptor，需要决定可呈现的源码位置
- **THEN** Java层 SHALL 拥有 raw descriptor 重复域、descriptor 参数位置对齐、owner/target/path/声明冲突准入、值拼写与 field/method 实际类型放置和无声明拒绝；root SHALL 保留属性读取、shell/raw 事实、错误/计费/停止及报告事务。现有报告类型可迁移并重导出，MUST NOT 复制一套镜像事实或让 Java 新增读取。迁移 MUST 保留拒绝次序、独立属性和独立目标的有效结果、不可拼写的原事实、clinit 注解处理及所有 evidence 下的正文/marker/physical snapshot/limits/logical usage/coverage；wide 参数按 descriptor 位置对齐，不能按 JVM slot 偏移。非空 type_path 和数组 return 维度旧拒绝不得借迁移扩大接受域

#### Scenario: Package-info source certificate consumes selected physical metadata

- **WHEN** root 已选中 package-info 物理类及原 annotation 事实，需要判断现有窄源码投影是否成立
- **THEN** Java层 SHALL 拥有严格 JVM 包名、Java8 版本/flags/header/空成员表、完整执行、单一属性与可写注解及空 Deprecated 的源码证书准入、拒绝次序和文本；root SHALL 保留原触发时点、事实供给、Reader、预算/停止/diagnostics、fallback 和最终替换事务。MUST 使用既有事实和切片，不创建镜像 DTO 或新增读取。迁移 MUST 保留原合法投影、非目标注解拒绝、raw metadata、physical fallback 与取消/输出预算边界，不扩大包注解或 classfile 版本的恢复域；三 evidence 的同模式新旧完整报告 SHALL 只规范实际实现身份与 usage 耗时

#### Scenario: Admitted class families spell MUTF names through the shared Java boundary

- **WHEN** generic static member、nested enum 或 nested annotation 已满足现有窄域的物理关系与源码证明，root/child 的名称是合法 supplementary MUTF
- **THEN** 源码消费者 SHALL 使用共享严格名称出口进行声明、token 与 needle 匹配；descriptor 与 CP/Code 身份 MUST 保持 raw 字节，不以标准 UTF-8 或 lossy 字符串替代。合法名称恢复 MUST 保留既有结构、bridge、完整执行、闭包、预算/停止和同方法/BCI/span 守卫；源码不可表达或损坏 MUTF 仍保留原始事实并明确拒绝，不据此扩大一般泛型或编译器 lowering 的支持域

#### Scenario: A proved member edge does not reinterpret literal dollars

- **WHEN** direct member 的 InnerClasses 关系已证明 `child raw == root raw + '$' + MUTF(simple_name)`，而 root 或 simple_name 自身包含字面 `$`
- **THEN** 投影 SHALL 保留标识符内部的 `$`，只按已证明的该条关系构造嵌套路径；MUST NOT 使用全局 `$` 替换或最后一个 `$` 推断 simple name。正文与 derived outline SHALL 在三种 evidence 下保持相同；essential 的正确性不依赖公开 SourceMap，验收可复核同一输入与物理主体已保存的 source_map 证据，但不得改变请求或用伪造的 derived 身份自证

### Requirement: Recovery evidence bindings distinguish semantic context and implementation

恢复 evidence 的产物绑定 SHALL 包含适用物理主体及来源、完整运行环境、源码目标、词法/投影语义配置、确切文本身份及解析/恢复语义版本。格式 schema、cursor schema、包版本、支持的 major 上限和构建来源 SHALL 按其实际含义区分；同包版本或相同文本 MUST NOT 独立证明 evidence 可互认。

#### Scenario: Same text is recovered under a changed parent environment

- **WHEN** 输出文本相同，但父 domain 的 roots/prefix/delegation 或依赖 snapshot 改变，调用方持有旧产物绑定
- **THEN** 新请求 SHALL 按完整环境/依赖身份拒绝不匹配旧绑定，不把新证据附着到旧上下文的产物

#### Scenario: Proof semantics change without a package version change

- **WHEN** 两个构建的包版本与输出文本相同，但解析或恢复证明语义发生变化
- **THEN** 其适用语义标识 SHALL 可区分，旧绑定不得静默互认；运行收据保留实际构建来源，不能仅以 registered major 上限或未更新的规则表判定一致

#### Scenario: Presentation semantics change but evidence selection alone does not

- **WHEN** 请求改变源码目标或影响语义的词法/投影配置，或仅改变可选 evidence 类别
- **THEN** 前者 SHALL 按适用配置拒绝旧绑定；后者保持主体与正文身份，独立报告所选 evidence 的载荷范围，不因选择不同改变恢复决定

### Requirement: Class source provenance is bound to the complete source unit

请求类级来源时，系统 SHALL 为已交付的声明、语句、派生和降级片段提供绑定该类单元文本身份的来源范围，复用物理定义/方法/BCI 与 metadata 证据。范围 SHALL 使用明确的 UTF-8 byte 半开坐标；缺口与未完成映射显式报告，MUST NOT 把方法局部 map 当作整类 map 或仅凭局部 derived spans 声称覆盖完整。

#### Scenario: Formatting and repeated edits change class offsets

- **WHEN** 类文本经过缩进、声明折叠或多轮投影，含重复片段、非 ASCII 或非 BMP 文本
- **THEN** 已提交来源区间 SHALL 对应最终类文本且位于合法字节边界，文本身份和偏移变换一致，不将旧方法坐标直接应用于类单元

#### Scenario: One projection combines several physical sources

- **WHEN** accessor、匿名或成员类投影把不同定义/方法的片段组合成一段 Java
- **THEN** 映射 SHALL 保留多个实际物理来源及派生关系，相同 BCI/CP index 在不同主体下不合并为同一位置

#### Scenario: A declaration comes from metadata rather than an instruction

- **WHEN** 类/字段/方法声明、签名或注解来自 metadata
- **THEN** 映射 SHALL 使用其实际属性/CP/声明来源，不伪造 BCI；不可映射范围说明缺口，普通标点可随所属语法片段定位

#### Scenario: Class source mapping is not requested or cannot finish

- **WHEN** 类正文已决定，而映射未请求或在物化时停止
- **THEN** 正文 SHALL 不因可选映射改变，map 状态明确区分未请求、部分和完成，只交付绑定本份文本的有效范围，停止不引发新 Body 分析

## S1 类与字段源码策略检查点（2026-10-24，AE）

[结果](results/s1-class-field-policy-v1/README.md)、[收据](results/s1-class-field-policy-v1/acceptance-r1.json)、[后续职责](results/s1-class-field-policy-v1/next-boundaries.md)。

Java 接管 class/interface/enum/@interface 的 flags/name/父类型拼写、普通与泛型字段共享的 descriptor/type path、flags/alias/ConstantValue/marker，以及 field Signature 的源码准入。root 保留同次读取、身份、Reader erasure、same-class/SSA writer、CP 闭包、预算/停止、deferred 与原子提交，gate 顺序不变。四生产文件纯迁移，无新增 DTO/Session/cache/read/永久测试函数/产品资产，恢复35/schema12/解析4/inputs v9保持。

六项局部门禁通过：29 native 1173 passed/0 failed/2既有ignored；16上下文48complete与8非零预算报告保持；继承普通声明56、类族65、构造器248、NoBody24、body12/130cutoff/name3及65 compact error/32原子partial点；168全文/78声明保持；当前136×3正文与408完整JSON保持，6编译12受控JVM/真实两类4重编通过，真实目标未执行。完整JSON只排除已核验actual engine与usage.elapsed，snapshot/raw/正文/marker/limits/费用/来源/coverage不变。

四组非枚举ClassField/Iface/Anno原样生成源的隔离编译、反射及ConstantValue通过；当前16份完整源码相同后复用oracle。所选class-source域的FieldEnum四源整体编译仍失败，旧enum常量字段拼写及AD数组return维度注解拒绝均保留，不称完整枚举或反射等价。首partial采集与非枚举汇总器列解析失败保留，首组原流离线复用、后三组续采，exit重建依据和派生元数据纠错明确，不称首次全绿。

2735来源/归档/构建前/当前与30产物、五成功命令同次绑定，target清理；906665720bytes仅作构建磁盘护栏。AD5759payload核验未改。下一组先迁class Signature完整shape/source策略，继续保留物理父定义/nested证明与budgeted callback；随后迁annotation位置/重复与package-info，再完成2.9 raw/span/bridge闭包和2.10 actual read/IR/live/peak/RSS/停止/背压及八旧失败。各组复用直接前继固定输入与完整报告门，不以formatter迁移关闭完整策略。2.9/2.10/S1仍未验收，进度17/59，S2未进入。

## S1 完整 class Signature 源码策略检查点（2026-10-24，AF）

[结果](results/s1-class-signature-policy-v1/README.md)、[收据](results/s1-class-signature-policy-v1/acceptance-r1.json)、[剩余职责](results/s1-class-signature-policy-v1/remaining-source-responsibilities.md)、[后续边界](results/s1-class-signature-policy-v1/next-boundaries.md)。

Java承接class Signature的一次性shape分类、kind/name/type-use准入、interface-only own-header fallback、direct Parent<String>/interface源码决定、type variable/bound/Object省略及完整header拼写；已有父位置/证明枚举迁移并由root内部重导出。root保留属性读取/parse、raw Signature记录时机、nested/physical header与Reader erasure、所选环境父定义查询及原budgeted callback、最后OutputBytes charge和完整提交。两生产文件纯迁移，无新事实DTO/Session/cache/read/永久测试函数/产品资产，恢复35/schema12/解析4/inputs v9保持。

六项有效门通过：29 native1173 passed/0 failed/2既有ignored；14上下文42complete+10非零预算报告保持，实际5个header projected/5refused/4无投影；继承class-field56/普通声明56/类族65/构造器248/NoBody24/body12/130cutoff/name3与65compact error/32原子partial点保持；168全文/78声明、当前136×3与408完整JSON保持，核心6编译12受控JVM/真实两类4重编通过，真实目标未执行。仅规范actual engine与usage.elapsed，snapshot/raw/正文/marker/limits/费用/来源/coverage不变。

原44基线缺direct-parent proved控制，Cargo终态后用既有Child/Parent固定fixture补8报告，原44和JAR不重跑/重建；52行视图分别pin两源。当前16类/字段完整源码相同后复用AE四组非enum编译/反射/ConstantValue，保留首组exit控制流重建的原证据层级。FieldEnum所选域四源编译失败、数组return维度注解旧拒绝均保留，不称恢复域增益或全反射等价。self-test、R1机制/核心只读入口的导出/字段错误及静态mode不符保留；R2独立副本修正，原native九工具不改，无成功native/CLI/JDK重跑，不称首次全绿。

2735来源/归档/构建前/当前与30产物、五成功命令同次绑定，target清理；906653750bytes仅作构建磁盘护栏，AE5980payload未改。下一组先迁annotation现有类型所有权、重复/参数对齐/target/path和field/method实际源码位置，再迁package-info窄证书；root留Reader事实/停止/报告事务，Java接管源码准入，不能只搬formatter。其后完成2.9 raw/bridge/span闭包与2.10 actual read/Body/IR/live/peak/RSS/取消/停止/背压及八旧失败的原合同。2.9/2.10/S1仍未验收，17/59，S2未进入。

## S1 annotation 位置与重复策略检查点（2026-10-24，AG）

[结果](results/s1-annotation-position-policy-v1/README.md)、[收据](results/s1-annotation-position-policy-v1/acceptance-r1.json)、[覆盖范围](results/s1-annotation-position-policy-v1/coverage-scope.md)、[后续边界](results/s1-annotation-position-policy-v1/next-boundaries.md)。

Java接管八个既有annotation报告类型、raw重复域、descriptor参数对齐、owner/target/info/path/冲突与拼写、field实际类型放置和method的clinit/无声明拒绝；root保留Reader shells/facts、计费、错误/停止、报告装配和发布。四生产文件纯迁移，无新DTO/Session/cache/read/永久测试函数/产品资产，2735来源，恢复35/schema12/解析4/inputs v9保持。

六项有效门通过：29 native1173 passed/0 failed/2既有ignored；13注解上下文×3evidence的39份完整报告保持（36complete/exit0、Holder3份旧partial/exit4）；继承classSignature52/classField56/ordinary56/family65/constructor248/NoBody24及body12/130cutoff/name3、65compact error/32双body原子partial点保持；168全文/78声明、当前136×3及408完整JSON保持，核心6编译12受控JVM/真实两类4重编通过，真实目标及字节变异未执行。仅规范核验过的actual engine tuple与usage.elapsed，正文/marker/raw与physical snapshot/limits/费用/coverage/来源保持。

39份CLI矩阵不是完整畸形输入分母：类级重复与非法element name等由既有native补充，任意畸形info未穷尽。Holder记录static声明与旧anonymous nesting partial，空注解列表不能冒充拒绝收益。AE FieldEnum四源整体编译失败、AD数组return维度注解旧拒绝，以及八项既有失败继续保留；本轮无恢复域或实际资源性能收益声明。

2735来源/构建前/归档/当前与30产物、五成功命令同次绑定，target已清理；907163740bytes为构建磁盘采样峰值，仅用于构建守卫。AF6146冻结payload及精确目录清单核验未改。工具在native前补强真实input/process绑定并改为只读复用408保存报告；native后、产品调用前又发现跨evidence误用全JSON比较，以resume-r2复用AF原outline检查核心，并继续由独立408同evidence完整JSON门严格验收。原10工具保持native AST pin，修订前preflight保留，没有因修正重复CLI或冒充产品收益。

下一组独立迁package-info窄源码证书，保持Java8/flags/空members/单空Deprecated及原拒绝次序；随后继续2.9 raw/bridge/span真实来源闭包和2.10 actual read/Body/IR/live/peak/RSS/取消/停止/背压及八旧失败的原合同。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 package-info 源码证书检查点（2026-10-24，AH）

[结果](results/s1-package-info-policy-v1/README.md)、[收据](results/s1-package-info-policy-v1/acceptance-r1.json)、[覆盖分布](results/s1-package-info-policy-v1/coverage-scope.md)、[后续边界](results/s1-package-info-policy-v1/next-boundaries.md)。

Java接管package-info窄源码证书与raw JVM包名合法性，按原顺序检查版本/flags/header/空成员、完整执行、属性/注解和单空Deprecated，再拼写package文本。root仅供给既有事实/切片，保持触发、Reader、预算/停止/diagnostics、物理fallback和完整提交。三生产文件纯迁移，无新DTO/read/cache/永久测试函数/产品资产；2735来源，恢复35/schema12/解析4/inputs v9保持。

六项有效门通过：29 native1173 passed/0 failed/2既有ignored；package-info36条同模式完整报告保持（31exit0、4exit4、1exit2）；继承annotation39、classSignature52/classField56/ordinary56/family65/constructor248/NoBody24、body12/130cutoff/name3及65compact/32双body原子partial保持；168全文/78声明、136×3与408完整JSON保持，核心6编译12受控JVM/真实两类4重编通过，真实目标和字节变异未执行。只规范核验过的actual engine tuple与usage.elapsed，physical/raw/marker/limits/费用/coverage/来源保持。

36行是11上下文×三evidence加3预算观察：4行成功投影、27行证书拒绝、4行前置partial和1行compact输出错误；class_headers=1是成功控制，不称停止。截断annotation只证明Reader/完整执行门。另3条误改魔数的早期探针及原流独立保留，不计源码证书分母；不冒充合法包名反例。 合法fixture原始/生成源在JDK8和23下各编译并运行，共4编译4JVM，输出均true；没有compiler/runtime笛卡尔积。当前三个正常evidence及成功预算控制全文相同后复用该oracle，未重复JDK；生成侧只有2编译2JVM，其余2编译2JVM是原始侧。

2735源码/构建前/归档/当前与30产物、五成功命令同次绑定，精确target已清理；907193463bytes仅为构建磁盘采样峰值。AG6332冻结payload和精确目录清单核验未改。12个工具AST与离线preflight绑定；跨evidence核心/正文检查与同evidence408严格完整报告检查分开，两个保存流复核均不再调用CLI。

证书仍只接受Java8固定shape和单空Deprecated；FieldEnum整体编译、数组return维度注解、Holder匿名嵌套partial与八项旧失败继续保留。本轮不声称恢复域扩大、完整来源map或实际资源性能收益。

下一组按[raw消费者清点](results/s1-package-info-policy-v1/remaining-raw-consumers-audit.md)先复现完整入口，再统一实际raw到source消费；generic static/nested family、父定义/override和widening的静态可疑点不冒充已复现收益。span同run/BCI/唯一性属S1边界，整类map与编辑后重定位另归S5。继续2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压和八旧失败的原合同。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 类族 raw→source 名称边界检查点（2026-10-24，AI）

[结果](../../results/s1-family-source-name-boundary-v1/README.md)、[有效收据](../../results/s1-family-source-name-boundary-v1/acceptance-r2.json)、[覆盖边界](../../results/s1-family-source-name-boundary-v1/coverage-scope.md)、[后续职责](../../results/s1-family-source-name-boundary-v1/next-boundaries.md)。

generic static、nested enum、nested annotation 和 common fold 的实际名称消费者收拢至既有 Java 严格命名出口；descriptor/CP/Code 身份保持 raw，嵌套路径仅转换已证明的 InnerClasses 边，保留标识符内字面$。两个生产文件、两个已有测试增强，无新DTO/read/cache/test函数/产品资产；2735来源保持，恢复35→36，schema12/解析4/inputs v9不变。既有A<T>/compareTo切片、bridge/闭包、完整执行、BCI/span、预算/停止与原子提交不放宽。

六项有效门通过：30 native1181 passed/0 failed/2既有ignored；十二开发上下文三evidence共36当前报告，四个名称拒绝上下文恢复、四旧有效控制保持、四JDK8形状拒绝保持。当前24 projected、6 generic JDK8 prepared/refused、6 enum JDK8 family refused，均exit0；不是退出码通过率或真实包整体恢复率。八组完整生成源码原样编译/隔离执行成功，原始六组Driver复用、两组fold原始Driver补验，生成classpath无原目标后备。继承六cohort501完整JSON、annotation39/package36、body12/budget130/name3、65compact/32双body原子partial；168全文/78声明、136×3与408同evidence完整JSON保持，核心6编译12受控VM/真实两类4重编通过，真实目标和字节变异不执行。

R1脚本误要求essential公开SourceMap，且两fold原始Driver因文件名编译失败；原流/原21工具与弱版续验保留。最终离线复核使用同context已保存source_map peer，MethodPoint仅来自physical recovery source-map段，物理ID取冻结旧事实；旧JSON缺少的两个Minor ID另用2次旧AH CLI只读查询补齐（contract35，单列不并36恢复分母）。8生成oracle和36当前恢复全部复用，只补失败原始Driver，没有重跑成功native/CLI/JDK。新增验证器对完整报告的伪造digest/BCI/span拒绝，不允许derived自证；不得称首次全绿。

五构建命令同次成功，2735来源/归档/构建前与31产物绑定，精确target清理；907183333bytes仅是构建磁盘采样峰值。AH6575冻结payload及目录清单未变。下一组迁已选泛型父定义读取后的源码资格至既有Java module，再按原阶段迁直接Override方法资格，root保持解析/读取/费用/停止/提交。旧UTF8 gate的语义修复独立复现；enum anonymous-body、不同raw widening、integer constant候选和2.10实际read/IR/live/peak/RSS/取消/停止/背压及八旧失败继续承接。整类map/重定位归S5。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 已选泛型父定义源码资格检查点（2026-10-24，AJ）

[结果](results/s1-selected-parent-policy-v1/README.md)、[有效收据](results/s1-selected-parent-policy-v1/acceptance-r1.json)、[覆盖边界](results/s1-selected-parent-policy-v1/coverage-scope.md)、[后续职责](results/s1-selected-parent-policy-v1/next-boundaries.md)。

Java接管已选直接泛型父类和接口读取后的源码资格、Signature形状/arity及原预算化解析；门面保留环境/loader选择、实际读取、SingleClass读前短路、缺定义平台fallback、execution/停止合并与提交。父类bool和接口三态不合并，接口仍只证明现有kind/arity等窄条件，不声称bounds/实参与bound或interface erasure证明。两生产文件纯迁移，无新DTO/Session/read/cache/永久测试函数/产品资产；2735来源，恢复36/schema12/解析4/inputs v9保持。

六项有效局部门通过：30 native1181 passed/0 failed/2既有ignored；14上下文三evidence42份完整JSON保持（39exit0、3预算停止exit4）；继承六cohort501/annotation39/package36/AI family36共612份完整JSON，body12/budget130/name3、65compact/32原子partial保持；168全文/78声明，136×3与408同evidence完整JSON和跨evidence outline保持，核心6编译12受控VM/真实两类4重编通过。AI八份生成源码相同后复用隔离oracle，未重复家族JDK，真实目标及变异未执行。仅规范已绑定实际构建的engine tuple和usage.elapsed，raw/physical/正文/marker/limits/费用/coverage/来源/partial范围保持。

R1执行分支误用contract35 resolver，在第一份实际CLI后拒绝真实AI36；原16工具/失败流不改，R2离线复核并复用000后续验，无成功Cargo/native/该次CLI重跑。门后独立核验42selected与36family报告的非空engine、原流/进程/输入和完整JSON，闭合selected原工具只查tuple互相一致的缺口。基线三份最初SingleClass参数错误原流因路径覆盖丢失，excluded为原argv实际补采，明确不入42分母。

五构建命令成功，2735来源/归档/构建前/当前与31产物绑定，精确target清理；907426558bytes只作构建磁盘采样守卫。AI6976冻结payload与精确目录清单未改。父类变体在erasure前shape拒绝、接口结构变体在更早原始检查拒绝，不充后续helper覆盖；CLI无token取消入口，native只证明既有回调取消/投影原子性。

下一组按原候选阶段迁直接Override源码资格，root保留same-package前筛选、raw唯一配对、物理来源、读取/费用/停止及偏移事务。旧父定义UTF8 gate修复另经完整入口复现；widening、enum anonymous-body、integer constant、raw/BCI/span可信闭包及2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压与八旧失败继续承接。整类map/重定位属S5。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 直接 Override 源码资格检查点（2026-10-24，AK）

[结果](results/s1-direct-override-policy-v1/README.md)、[有效收据](results/s1-direct-override-policy-v1/acceptance-r1.json)、[覆盖边界](results/s1-direct-override-policy-v1/coverage-scope.md)、[后续职责](results/s1-direct-override-policy-v1/next-boundaries.md)。

Java declaration接管直接Override的child class flags/scope、parent class flags/scope、child method普通flags/name/Signature/已有声明与注解/正文/descriptor以及parent method资格/visibility。root保留环境/loader、读取/完整性、raw index/name/descriptor/flags绑定、同origin/同包、唯一配对、原计费/停止、唯一offset与output charge后的原子提交。child资格在父表扫描费前，parent资格在唯一匹配及计费后；descriptor是已证明与报告相等的同一raw member。两生产文件纯迁移，无新DTO/Session/read/cache/永久测试函数/产品资产；2735来源，恢复36/schema12/解析4/inputs v9保持。

六项有效局部门通过：30 native1181 passed/0 failed/2既有ignored；16上下文三evidence48份完整JSON保持（42exit0、3compact exit2、3partial exit4），三成功上下文的9份Override proof保持，11无提示上下文保持。继承六cohort501/annotation39/package36/family36/selected-parent42共654完整JSON、body12/budget130/name3、65compact/32原子partial保持；168全文/78声明、136×3与408同evidence完整JSON/跨evidence outline保持，核心6编译12受控VM/真实两类4重编通过。family同字节后复用8生成隔离oracle，未重复家族JDK；真实目标与变异未执行。仅规范已核验actual engine与usage.elapsed，raw/physical/正文/marker/limits/费用/coverage/来源/partial范围保持。

R1 AA wrapper旧变量名在产品调用前失败；R2复用六组501报告并通过三个AA门，随后首annotation查询后因收据pin类型错误停止。原19工具与两轮失败原流保留，R3复用九成功门和首annotation，只跑剩余五门，14项最终有效通过；门后201份父/Override报告逐条核对engine/输入/实际命令/原流/收据及完整JSON。前三份visibility setup误打public class的真实报告另存excluded，修正protected输入另存；不称首次全绿、参数错误或恢复收益。重复工具协议债务[单独记录](results/s1-direct-override-policy-v1/separate-harness-debt.md)，不混入产品迁移。

五成功构建、2735来源/归档/构建前/当前和31产物绑定，精确target清理；907830747bytes只作磁盘采样护栏。AJ6838冻结payload与精确清单未改。预算compact三行无artifact engine，analysis三行在方法恢复期停止，不冒称helper内部覆盖；native精确output charge原子性保留，内部定点取消仍不宣称穷尽。

下一组在实际入口统一父定义与Override的严格名称出口：U+10400方法名的旧无提示已复现，本组不修接受域；合法supplementary/Java7-8 `_`、源码不可表达、损坏MUTF/孤立surrogate分组，保留raw匹配/环境/loader/erasure/Signature/visibility及原预算事务。widening、enum anonymous-body、integer constant、raw/BCI/span可信闭包和2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压、八旧失败继续承接；整类map/重定位归S5。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 父定义与 Override 严格名称边界（2026-10-24，AL）

[结果](../../results/s1-parent-override-name-boundary-v1/README.md)、[验收收据](../../results/s1-parent-override-name-boundary-v1/acceptance-r1.json)、[独立名称审查](../../results/s1-parent-override-name-boundary-v1/name-results-independent-review.md)、[覆盖范围](../../results/s1-parent-override-name-boundary-v1/coverage-scope.md)、[后续边界](../../results/s1-parent-override-name-boundary-v1/next-boundaries.md)。

Java层既有方法/类名严格出口接入直接Override与已选父类/接口资格；父定义的单一Object-bound类型参数严格解码，并允许Java7/8的`_`，该变量不在Child发射。全局source_identifier/局部别名策略未改。root环境/loader、实际读取、raw唯一配对、物理来源、flags/arity/erasure、收费/停止和原子提交保持。三生产文件、两测试文件，无新产品helper/DTO/读取/缓存/资产，2735来源路径不变；恢复36→37，schema12/解析4/inputs v9不变。这是名称语义修复，与前继纯职责迁移分别验收。

六项局部验收通过：30 native1182 passed/0 failed/2既有ignored；21上下文三evidence共63份，12个合法名称上下文的36份修复、27份完整JSON保持。三组方法名新增9份精确raw Override proof，九组父类/接口解除header拒绝。新增12次完整生成Child重编、24次driver编译、Java8/23共48条VM命令及24对输出对照通过；生成classpath无原Child，父/接口为声明的原始依赖，观察限于方法反射调用和泛型继承元数据，不声称整包、父定义自身源码或JVM7覆盖。真实目标和字节变异未执行。

654份继承机制及45份旧Override完整JSON保持；body12/budget130/name3专项保持，另核65compact和32个0/2-header原子partial点，不冒称这些专项均为全JSON。168全文/78声明、136×3与408同evidence完整JSON/跨evidence outline保持；核心6编译12受控VM及真实两类4重编通过。family全文不变后复用8组生成隔离oracle，无新增家族JDK。全JSON保持门仅规范actual engine与usage.elapsed，名称修复的36份另核完整差异及raw/方法正文，不靠删除字段放宽。

旧基线15份方法名负例因generic基底提前拒绝，原流保留为setup，普通无Signature父子类另采；有效63份含9复用和54新查，历史新旧CLI调用69次。名称工具R1误把成功Signature说明注释当拒绝，已结束的31次查询全部复用，R2只新增余下32次；原20工具与失败流保留，不称首次全绿。非法父类名exit2是原始父类声明拼写失败，不是CLI选择参数错误或selected-parent helper覆盖；四字节名称负例是stdout failed ClassSourceReport。

五成功构建、2735来源/归档/构建前/当前与31产物同次绑定，精确target清理；907386668bytes仅为构建磁盘采样护栏，不代表实际read/IR/RSS收益。新名称进入更深的原证明路径，费用变化单列；内部定点取消未穷尽。当前类自身`_`泛型header/变量等静态剩余消费者先做完整入口复现，再闭合名称与相关使用；widening、enum anonymous-body、integer constant及同次raw/方法/BCI/span继续按2.9推进，整类map/重定位仍属S5。2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压与八旧失败按原协议承接。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 当前声明与类型变量名称闭包（2026-10-24，AM）

[结果](../../results/s1-own-generic-name-boundary-v1/README.md)、[收据](../../results/s1-own-generic-name-boundary-v1/acceptance-r1.json)、[覆盖边界](../../results/s1-own-generic-name-boundary-v1/coverage-scope.md)、[后续职责](../../results/s1-own-generic-name-boundary-v1/next-boundaries.md)。

Java层收拢当前声明与类型变量严格名称出口：类头、字段、方法、构造器、throws/数组及class/method同名遮蔽共同支持Java7/8合法`_`；raw scope、erasure、完整使用、预算/停止和提交守卫保持。新增两个names helper，三生产文件/一测试文件变化，未新增DTO/Session/cache/read/永久资产或测试函数；2735来源不变，恢复37→38，schema12/解析4/inputs v9保持。该名称语义修复与前继纯职责迁移分别验收。

六项局部门通过：30 native1182 passed/0 failed/2既有ignored；当前typed126中60预期修复/66完整JSON保持，owner30中12修复/18保持，继承762完整JSON保持。72是报告数，包含目标/evidence同源变体，不作为独立问题族分母。AA body12/budget130/name3子集合同保持，另核65compact/32双body原子partial（header0:31、2:1）；168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持。核心6编译12受控VM/真实两类4重编通过，真实目标和字节变异未执行。

新增18个合法受控程序、34次完整生成源码编译；生成全部输入目标类、空编译classpath、运行仅生成classes与Driver，无原目标或原依赖后备。Java8/23共104条VM命令、68对输出一致，观察GenericDeclaration归属、bounds/数组/throws及字段/构造/普通调用。仅为定向小型程序及两组完整Parent+Child，不外推任意JAR/WAR、任意行为或JVM7执行。family同字节后复用旧oracle，无重复家族JDK。

owner旧采集51条含18条重复argv，初版三份ASCII控制非泛型且原脚本未另存，旧工具无timeout；原流和偏差如实保留。新增typed工具先固定再执行并设置90s超时。offline preflight的evidence字典/选择字符串误判在产品执行前修正，13原工具在native前固定。native summary.scope误留AL标签，实际schema/来源/30二进制为AM，不以标签冒称额外覆盖。

五构建命令与31产物同次绑定，target清理；907408680bytes仅为磁盘采样护栏。AL7380冻结payload与目录清单保持。下一组仍按2.9审查实际源码策略/事实编排边界，widening、enum anonymous-body、integer constant与raw/方法/BCI/span分组推进；整类map/重定位仍属S5。2.10实际read/Body/IR/live/peak/RSS、取消/停止/背压与八旧失败继续按原协议承接。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 类族调用名称职责迁移（2026-10-24，AN）

[结果](../../results/s1-member-call-name-policy-v1/README.md)、[收据](../../results/s1-member-call-name-policy-v1/acceptance-r1.json)、[覆盖范围](../../results/s1-member-call-name-policy-v1/coverage-scope.md)、[后续职责](../../results/s1-member-call-name-policy-v1/next-boundaries.md)。

Java source_proof接管类族调用证明的root严格MUTF解码、child源码资格、root既有保守名称资格及dotted拼写；facade保留物理关系/capture/header事实、完整方法/IR、raw new/constructor/BCI、计费/停止和提交。三条拒绝文本与语义优先级不变，没有借纯迁移放宽`_`等名称。两个生产文件、一处既有cfg(test)断言扩展，新增一个纯helper，无新DTO/Session/cache/read/测试函数或永久资产；2735来源不变，恢复38/schema12/解析4/inputs v9保持，Java与facade实际实现指纹更新。dotted String和参数clone分配时机改变，不宣称内存/时间不变。

六项局部门通过：30 native1182/0/2；AM既有918份加新18份完整JAR入口，共936份完整JSON保持（仅规范actual engine与usage.elapsed，raw/正文/费用/coverage/partial/BCI/来源保留）。AA body12/budget130/name3子集门及65compact/32双body原子partial保持；168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持。核心实际6受控编译12VM、真实两类4重编通过，真实目标不执行。AM的34个完整源码编译变体/68对运行输出在text相同和pins有效后复用，本轮不重跑这组JDK，也不将新18份计为行为oracle。

有效R3为Corretto8编译的root+Inner完整JAR，ASCII/supplementary各6份calls proved；单独`_`6份最终prepared_fold而无calls字段，不充narrow调用证明成功或外部拒绝。R1模板guard误判、R2单CLASS缺成员/Outer_错误名称及javac23、工具parent路径修正和初次构建漏参均分别保留，不作为恢复收益。五构建命令/31产物同次绑定，target清理；907388435bytes仅为构建磁盘采样护栏，AM8545冻结payload与精确清单未变。

下一组先冻结static family target末尾名称资格的完整入口及停止前缀，再迁Java；该组child仅要求可严格解码，不能整套复用更强的member-call helper。调用仍位于物理/descriptor/IR/Java recovery证明之后。随后继续widening、enum anonymous-body、integer constant及同次raw/方法/BCI/span；整类map/重定位仍属S5。2.10实际read/Body/IR/live/peak/RSS、各类取消/停止/背压/panic与八旧失败保持原退出门。2.9/2.10/S1未验收，17/59，S2未进入。

## S1 static target 三段源码判据迁移（2026-10-25，AO）

[结果](../../results/s1-static-target-policy-v1/README.md)、[收据](../../results/s1-static-target-policy-v1/acceptance-r1.json)、[范围](../../results/s1-static-target-policy-v1/coverage-scope.md)、[下一组](../../results/s1-static-target-policy-v1/next-boundaries.md)。

static target的空Object构造器三指令/CP形状、原ClassSourceRecovery资格、root/child名称解码与root路径三个源码判据迁入Java source_proof。root保留物理选择/完整性、实际Header/IR/recover、依赖登记/计费/stop、target装配；各判据仍在原阶段，名称未提前到读取前，static child仍仅需严格解码。早期static/root-child Signature/fields与constructor selector的元数据资格仍未迁完，单列下一组，不宣称整个static函数策略完成。

相对AN，两生产文件及一处既有names cfg(test)断言扩展，新增3纯helper，无DTO/Session/cache/read、新测试函数或永久source资产。2735来源路径不变，恢复38/schema12/解析4/inputs v9保持；实际Java/facade指纹随构建变化。分配生命周期可变，不以纯迁移零文本差异推断性能不变。

六项局部门通过：30 native1182/0/2；前继936加新static24共960完整JSON保持，仅规范actual engine与usage.elapsed。AA body12/budget130/name3子集及65compact/32双generic body原子partial保持；168全文/78声明、136×3与408同evidence完整JSON/跨evidence outline保持；核心实际6受控编译12VM、真实两类4重编通过，真实目标不执行。AM的34源码编译变体/68运行对经AM→AN→AO两跳text/pins核验后复用，未当作本轮新增JDK或新24份的行为oracle。

新24为固定Corretto8的四完整root+Box JAR、release8与SourceTarget7/8×三evidence：12份明确static_proved；_和effectful的12份仅prepared_static、未发capture/calls，不充内部helper切点负例。五成功构建/31产物绑定，target清理；907410424bytes仅为磁盘采样护栏。AN7413封存payload与精确清单不变；继承工具标签依actual schema/source/build核对，不把组名当producer。

下一组按共享元数据源码准入机制批量迁static header资格与integer constant候选，保留读前/读后时序、reader完整性/物理绑定及预算提交；integer supplementary限制仍只是静态风险，先独立复现再修语义。随后继续widening、enum anonymous-body及raw/方法/BCI/span；整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic与八旧失败仍按原退出门，2.9/2.10/S1未验收，17/59，S2未进入。

## S1 共享元数据源码准入职责迁移（2026-10-25，AP）

[结果](../../results/s1-metadata-source-policy-v1/README.md)、[收据](../../results/s1-metadata-source-policy-v1/acceptance-r1.json)、[范围](../../results/s1-metadata-source-policy-v1/coverage-scope.md)、[下一组验收](../../results/s1-metadata-source-policy-v1/next-boundaries.md)。

static target的读前static/root Signature、读后child字段/Signature资格和完整表唯一raw构造器selector，以及integer constant的准入和名称先于值去重，成组迁入Java source_proof。facade保留物理读取/关系/identity、字段/方法完整性、CP/IR/recover、逐项计费/poll、停止清空及提交；selector返回原Header引用，mask保留原FieldItem和顺序，不以源码拼写授予物理身份。

相对AO仅两生产文件，Java文件同时新增一个cfg(test)测试函数；新增5纯helper，无DTO/Session/cache/read或永久source inventory资产。2735来源路径、恢复38/schema12/解析4/inputs v9保持，实际Java/facade指纹更新。局部分配与生命周期可变，不从纯迁移推断时间/RSS不变。

六项局部门通过：30 native1183/0/2；继承AO960加新合法90/变异30，共1080份完整JSON保持，仅规范actual engine与usage.elapsed_millis。AA body12/budget130/name3为字段子集，65compact与32个constructor+两个generic body均complete的partial点保持，header0:31/2:1；145份中170条engine identity全匹配。168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持；实际6受控编译12VM、真实两类4重编通过，真实目标未执行。AM34完整源码编译变体/68运行对经AM→AN→AO→AP三跳text与pins核验后复用，不计为本轮新增JDK或新120份的行为oracle。

新15合法JAR覆盖static六变体与integer九变体，固定Corretto8、release8、SourceTarget7/8×三evidence。合法90份均exit0；五query-only变异30份中duplicate ctor/truncated table合计12份exit4，是classfile前置拒绝，不充内部source helper定点负例；变异不编译或执行。KEY𐐀有完整字段声明和ConstantValue但0常量名投影，已从静态风险升级为完整入口复现的可读性缺口，不等于整类失败。本轮保持旧std UTF-8准入与display bytes比较域；后续必须同时改严格MUTF名称出口及candidate.field.name.raw()去重key，不能只替换decoder。

R1 ZIP排序误判和首次变异父verifier失败原流保留；R2复用首输入成功编译pins，另14输入各首次编译，15次成功fixture编译均只执行一次。父adapter错误lineage/current checkout比较缺失在freeze前修复；初版与diff保留，11原tools固定为native前AST版本，协议统一独立承接。五构建命令/31产物绑定，target清理，907843864bytes仅为构建磁盘护栏；AO7485封存payload与精确清单保持。

下一组按整型常量raw identity→忠实源码名称→switch/return/int[]消费者闭环推进：先保存旧完整入口，再同时修准入/raw key；保留声明/flags/I/ConstantValue/完整表/停止清空与anchors，接受域变更恢复38→39，建立新producer绑定，并以完整生成类的隔离编译、反射字段名/flags/value及行为对照验收。malformed/源码不可写/重名/同值/遮蔽/停止分组，变异仍只查询。随后按原阶段继续enum anonymous-body、widening及raw/方法/BCI/span；整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类取消/停止/背压/panic与八旧失败仍按原退出门。2.9/2.10/S1未完成，17/59，S2未进入。

## S1 整型常量 raw 名称恢复闭包（2026-10-25，AQ）

[结果](../../results/s1-integer-source-name-v1/README.md)、[收据](../../results/s1-integer-source-name-v1/acceptance-r1.json)、[范围](../../results/s1-integer-source-name-v1/coverage-scope.md)、[后续](../../results/s1-integer-source-name-v1/next-boundaries.md)。

Java准入复用既有严格MUTF-8 source_identifier，mask从既有FieldItem传raw key，全部物理字段先去重名、再计候选重复值；源码String不再回编码充物理身份。root保留Integer/声明/static final/I/单ConstantValue、完整field表、读取/预算/poll、停止清空与提交。三个生产文件含恢复版本，扩展一个既有Java单测，无新增helper/DTO/Session/cache/read或永久source路径。恢复38→39，schema12/解析4/inputs v9保持，实际实现指纹更新。

六项局部门通过：30 native1183/0/2；AP1080加新合法42/变异48共1170，1152完整JSON严格保持，18份精确名称/UTF-8 byte spans及原Field/Method/BCI增量通过（AP6由0→2，新12由0→4）。仅规范已验证engine与usage对象直属elapsed_millis；新增AST保留及后续累计预算快照的费用变化逐位置保存，方法级只允许steps/ir_items非负，不能对累计快照求和充实际资源。原比较18失败、R1路径标签TypeError和R2修正都保留，11原tools未改、CLI未重跑。6个审查负例均拒绝。

168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持；核心实际6编译12VM、真实两类4重编通过，真实目标未执行。AA body12/budget130/name3继续为原字段子集。新7合法contexts（名称×debug和同值控制）×SourceTarget7/8生成14完整源码编译变体，三evidence先核全文相同；JDK8/23下14原件VM/28生成VM形成28对反射/行为一致，额外Driver编译1次，隔离加载并校验实际CodeSource。新7原fixture各成功编译一次，变异只查询；12份编码前端拒绝不计Java helper覆盖，raw重名/同值先后次序及既有g()独立return边界保持。同源变体不充独立holdout，语言目标7不充JVM7覆盖，也不将名称可读性改善计作新增可编译类。

五串行构建/31产物/2735来源/11原tools绑定，target清理，907839575bytes仅为构建磁盘护栏；AP8468封存payload保持。下一组先冻结enum constant anonymous-body完整JAR与有效/拒绝/停止域，再迁源码/flags/可展示资格，root保留物理配对/coverage/read/recover/费用/提交，避免crate反向依赖和新DTO。该处旧UTF8仅为静态风险，未冒称已复现；若改接受域另声明相对39的恢复版本。其后widening和raw/方法/BCI/span按共享机制推进，整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic与八旧失败仍未验收；2.9/2.10/S1未完成，17/59，S2关闭。

## S1 enum child 源码资格职责迁移（2026-10-25，AR）

[结果](../../results/s1-enum-child-source-policy-v1/README.md)、[收据](../../results/s1-enum-child-source-policy-v1/acceptance-r1.json)、[范围](../../results/s1-enum-child-source-policy-v1/coverage-scope.md)、[后续](../../results/s1-enum-child-source-policy-v1/next-boundaries.md)。

Java source_proof接管四组分阶段判据：child声明/无字段；旧UTF8方法名、identifier与flags；raw唯一base配对后的override可见性；同次RecoveryReport/analysis execution/声明与marker事实的可展示资格。facade保留实际class_source_with_evidence、物理identity、完整Code coverage、constructor/clinit、raw name/descriptor唯一配对、method poll、停止/原错误与group提交。root wrapper只解包Outcome并传已有类型/两bool，无ClassSourceMethod穿层或新DTO。两生产路径，Java既有测试和root cfg(test) helper assert_positive扩展，0新测试函数；4新纯helper，无Session/cache/read/永久source资产。恢复39/schema12/解析4/inputs v9保持，实际实现指纹更新。

六局部门通过：30 native1183/0/2；AQ最终semantic-r2的1170 current报告加AR138，共1308完整JSON保持，仅规范校验后的engine与usage直属elapsed_millis，raw/费用/partial/BCI/来源和错误优先级不变。168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持；核心实际6编译12VM、真实两类4重编通过，真实目标未执行。AA body12/budget130/name3为原字段子集；AQ14完整源码编译变体/42源报告/28运行对经AQ→AR一跳text与pins核验复用，不计本轮新增JDK或新138份的行为oracle。

9同源contexts在Corretto8 source8/target8与JDK23 --release8各编译一次，共18成功编译、108报告（runtime8、SourceTarget7/8×三evidence）。现代54中ASCII两族/debug与BMP共30发布enum_projection，supplementary/_/fields/extra-method24无投影；旧compiler54缺$values形状，不能当源码helper覆盖。5 query-only变异30中18exit0、12exit4（instruction decode/method ambiguous），不编译/执行、不由终态断言内部拒绝切点。同源变体不充独立holdout，语言目标7不充JVM7覆盖。mutator初版parser失败、后续修正private/真duplicate及AA漏适配脚本的零产品调用失败均保留，未修改原11tools或重跑成功产品门。

五串行构建/31产物/2735来源/11原tools绑定，target清理，908144335bytes仅是磁盘护栏；AQ8887封存payload保持。下一组以严格MUTF与source_method_name统一child方法资格和既有忠实声明（不是保守source_identifier），保留raw配对/flags/Code/core证明与事务；接受域变更单列39之后版本及完整类/隔离行为/元数据/来源验收。现代supp/_12无投影仅为入口事实，不推断唯一根因；旧$values lowering差异独立归S5类级机制，不能混入名称修复。随后widening及raw/方法/BCI/span按共享机制推进，整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic与八旧失败仍未验收；2.9/2.10/S1未完成，17/59，S2关闭。

## S1 enum 方法忠实名称恢复闭包（2026-10-25，AS）

[结果](../../results/s1-enum-method-name-v1/README.md) · [收据](../../results/s1-enum-method-name-v1/acceptance-r1.json) · [范围](../../results/s1-enum-method-name-v1/coverage-scope.md) · [后续](../../results/s1-enum-method-name-v1/next-boundaries.md)

Java child 方法资格与既有方法声明共同消费 names::source_method_name：严格 MUTF-8，保留 Java 7/8 单独 _。raw name/descriptor、base 唯一配对、flags/visibility、Code/core 完整性、预算/停止及组提交保持。仅 source_proof.rs 与 artifact.rs 改变，facade 与 AR 逐字节一致；0 新 helper/测试函数/DTO/Session/cache/read/永久 source 路径。接受域改变使用恢复39→40，schema12/解析4/inputs v9保持。

六项局部门通过：30 native1183/0/2；1452报告中1440完整JSON保持，12份 supplementary/单独_由无enum_projection变为闭合投影。12份完整文本及完整projection_inputs由已有ASCII有效机制与独立child物理事实参数化核对，其余physical/method/BCI/artifact/status保持。唯一逻辑费用变化是既有constants_text计费：supplementary6份3190→4907、_6份3174→4859，只出现在usage及execution.usage的output_bytes；两者为累计快照，不求和充实际I/O或RSS。168全文、78声明、136×3核心、408同evidence完整JSON/跨evidence outline与AA原字段子集12/130/3通过。

七个有效现代contexts×SourceTarget7/8生成14完整源码程序（字符串族包含当前Jarde IOps），三evidence先核全文一致，空classpath原样编译；Driver另编译1次。复用冻结原class，在JVM8/23执行原件14次/生成28次，28对方法名/参数返回类型、enum name/ordinal、常量体调用、tag/getOp输出一致，校验root/children/IOps的Loader与CodeSource。核心另实际6编译/12VM、真实两类4重编；真实目标和变异不执行。新child108/名称变异24/接口12报告复用完整AR JAR，原fixture成功编译不重跑；4名称变异18exit0/6exit4，前端失败不充Java helper覆盖。同源变体不充独立holdout，语言目标7不充JVM7。

普通widening主体已在Java；后续先核raw/String handoff、bridge共同源码证明与同次方法/BCI/span，不重复迁移薄适配包装。Corretto8缺$values的旧lowering单列S5，整类map/重定位同属S5。2.10实际read/Body/IR/live/peak/RSS及取消/预算/elapsed/sink/backpressure/panic、八旧失败和API/双轨收尾仍待原门验收。2.9/2.10/S1未整体通过，OpenSpec17/59，S2未进入。

### Requirement: Enum child qualification shares faithful method spelling

Java 恢复 SHALL 以与声明相同的严格 JVM 名称解码和忠实 Java 7/8 方法拼写判断 enum child 资格，并保留 raw name/descriptor 做精确物理配对；flags、visibility、完整 Code、同次 core proof、停止和组原子发布条件不变。

#### Scenario: Legal supplementary and underscore override names

- **WHEN** 完整 enum 类族在已注册 SourceTarget7/8 下具有其他条件均满足、方法名为合法 supplementary 或单独下划线的 child override
- **THEN** 资格与声明一致，所有请求的 evidence 模式产出相同完整源码，生成源码原样编译并与隔离原件行为一致

#### Scenario: Invalid encoding or unfaithful source name

- **WHEN** 已选 child 名称是损坏 MUTF-8、Java 关键字，或现有源码名称规则无法忠实表达
- **THEN** 在对应解码或源码证明边界拒绝该类族，保留原 raw evidence，不提交半项 enum-body 投影

## S1 bridge 当前类源码准入职责迁移（2026-10-25，AT）

[结果](../../results/s1-bridge-source-policy-v1/README.md) · [收据](../../results/s1-bridge-source-policy-v1/acceptance-r1.json) · [范围](../../results/s1-bridge-source-policy-v1/coverage-scope.md) · [后续](../../results/s1-bridge-source-policy-v1/next-boundaries.md)

Java source_proof接管bridge当前类27个准入守卫、同次Report/analysis完整性、raw name/descriptor唯一配对、flags/MethodParameters、参数cast form及covariant return判据；两个既有Signature是否参数化函数一并迁入。门面供给借用事实iterator和selected-header callback，保留resolver/平台absence/访问检查、reloaded-current、Stop清空bridge proofs、原成员和staging/commit。当前类证明是完整阶段迁移；最终继承header再生gate仍由门面编排，后续单列收口。

六项局部门通过：30 native1183/0/2；AS权威1452加新bridge84，共1536完整JSON严格保持，0语义例外，仅规范校验后的actual engine及usage直属elapsed_millis。168全文、78声明、136×3核心、408同evidence完整JSON/跨evidence outline及AA12/130/3原字段子集保持。核心实际6受控编译/12VM、真实两类4重编；既有bridge完整源码/erased dispatch/跨层级协变与停止测试本轮实际通过。AS14完整程序/54源报告/28行为对经一跳text、输入和pins核验复用，该项新增JDK0，不把新84份计入旧Oracle。

新84来自14既有context×SourceTarget7/8×三evidence，共7完整JAR，0新fixture编译/目标执行/变异；54报告有投影、30拒绝。跨配置102条proof记录为72 admitted/projected、30 refused，不能当独立holdout或27内部守卫探针。两个生产文件、facade减少359行/Java增加390行，新增一个阶段证明入口、两既有函数迁移，无新DTO/Session/cache/read/Body/测试函数；恢复40/schema12/解析4/inputs v9保持，实际Java/facade指纹更新。五构建/31产物/2735来源/11原工具绑定，target清理，908139246bytes仅为构建磁盘峰值。

下一项先收口继承header再生gate，并复现平台Comparable契约未明确匹配raw方法名的静态盲区；只有完整入口反例支持后才修接受域和版本。随后核raw enum owner/descriptor、widening同次method/BCI/loader。2.10真实资源/各类停止/背压/panic、八历史失败当前复核、API/双轨收尾仍未验收。2.9/2.10/S1未整体通过，OpenSpec17/59，S2未进入。

### Requirement: Bridge source qualification consumes same-run borrowed facts

Java 恢复 SHALL 消费同次借用的物理成员、header、恢复结果及 analysis 判断当前类 bridge 源码资格；门面 SHALL 保留物理读取、依赖解析、停止传播和提交职责。迁移不得重读 Body、制造另一套方法身份、依赖可选 evidence 或改变既有拒绝优先级。

#### Scenario: Current-class source target and header walk

- **WHEN** bridge 候选需要同次当前类 source target、参数 cast 或 covariant-return 证明
- **THEN** Java 层按 raw name/descriptor、flags/metadata 和完整恢复事实决定资格，仅经原 selected-header callback 按同一 loader/depth/Budget 取依赖；门面保留原方法记录

#### Scenario: Refusal or global stop before bridge commit

- **WHEN** 当前类事实不满足原资格条件，或依赖 header walk 发生全局停止
- **THEN** 保留原局部拒绝次序；全局停止传回门面后清空该组先前 bridge proofs，原成员和未提交文本保持，不提交半项替换


### Requirement: Exact platform bridge member and inherited source contract ownership

系统 SHALL 在现有平台 clean-Missing 条件成立后，以 raw owner/name/descriptor/use-kind 精确识别 `java/lang/Comparable.compareTo(Ljava/lang/Object;)I`；平台不匹配 SHALL 继续现有成员解析及其他父接口搜索。提供了定义时 SHALL 由其真实声明决定契约。Java 层 SHALL 负责选定继承契约的源码 header 再生资格，门面保留事实读取、环境解析、预算、停止与提交。

#### Scenario: Missing Comparable with a different raw member name

- **WHEN** 同形 bridge 的 raw name 为 notCompare、单独下划线或合法 supplementary 名称，且环境未提供 Comparable
- **THEN** 不以 compareTo 平台事实准入；保留原物理方法与恢复报告、call BCI，并明确拒绝原因，不发布该 bridge 的隐藏投影

#### Scenario: Provided definition states the matching raw contract

- **WHEN** 输入提供的 Comparable 真实声明同一 raw 方法名/descriptor，且原 resolver 与源码 header 条件均成立
- **THEN** 依据提供的定义决定准入，不把平台 compareTo 名称条件套到普通输入定义；对应控制的完整报告保持

#### Scenario: Another interface supplies a legitimate erased bridge contract

- **WHEN** 合法 Dual 同时实现 Comparable<Dual> 与 Other<Dual>，other(Object) 的契约由输入 Other 提供
- **THEN** 平台 matcher 不匹配后继续解析 Other，两个合法桥均保持投影；完整生成 Dual/Other 原样重编并在 JVM8/23 核对擦除调用、bridge/synthetic flags、CodeSource 与 loader

#### Scenario: Source contract Signature or projection staging stops

- **WHEN** 原 Signature 读取条件触发停止，或 bridge 注释费用不足
- **THEN** 维持原读取顺序、拒绝与停止传播；停止清空原 bridge proofs 或保留未提交的原成员，不产生半项投影

该局部合同的验收见 [AU](../../results/s1-bridge-contract-source-v1/README.md)：1584严格保持+24指定变化，逻辑费用逐维核验；不据此宣告2.9/2.10或S1完成，S2不进入。


### Requirement: Enum child metadata source relation belongs to the Java proof stage

Java层 SHALL 对门面同次选择并读取的child metadata核对源码relation：child与parent的唯一anonymous InnerClasses记录、类型级EnclosingMethod以及唯一匹配构造器。系统 SHALL 复用已有reader facts/pool/raw owner/descriptor与借用header，保持原扫描计费、逐行取消轮询、重复项和无效CP行的原拒绝语义。门面 SHALL 继续拥有物理身份、读取、Code/使用闭包证明、停止传播和完整组提交。

#### Scenario: Selected child metadata qualifies for an enum constant body

- **WHEN** 同一物理child的两侧InnerClasses记录、enclosing owner/method及constructor descriptor满足原条件
- **THEN** Java阶段返回同一header及原顺序anonymous owner列表；caller继续原constructor/body/census证明，不直接发布投影

#### Scenario: Metadata relation is missing or ambiguous

- **WHEN** child或parent匿名行缺失/重复/带名字或outer，enclosing缺失/指错owner或method，或构造器缺失/重复/descriptor不匹配
- **THEN** 保留原局部拒绝/保守输出及物理facts；原reader提前拒绝的输入不计作Java helper命中，不跨候选半提交

#### Scenario: Scan budget or cancellation stops relation proof

- **WHEN** 原InnerClasses或method扫描收费点、逐行poll发生全局停止
- **THEN** 从同一Budget向门面传播原错误，不将停止变成普通缺失或成功relation，不新增Body读取或提交

#### Scenario: Baseline lowering does not reach the intended mechanism

- **WHEN** 拟用输入未包含该恢复阶段的必要lowering，或阳性控制不对应已验证的完整源码
- **THEN** 保留原流并单列边界，先以实际投影与编译oracle核定新的有效控制，不从总查询数推断guard覆盖；AV1740全严格保持不授予S1整体完成


### Requirement: Same numeric BCI does not share widening proof across methods or loader definitions

完整类恢复 SHALL 将 widening 的消费绑定同次物理方法、数值 BCI 与实际 loader/定义链。系统 SHALL 保留 Java 层现有纯证明和门面的读取/身份/停止职责，不为补测试新增独立证明入口或缓存。

#### Scenario: Two methods have different casts at BCI one

- **WHEN** alpha(S) 与 beta(U) 都在 BCI1 调用需要 T/V 的方法
- **THEN** 最终方法各自生成正确转换；只打断 alpha 所选链时 alpha 拒绝而 beta 保持成功，不能因数值 BCI 相同串用决定或 method-origin

#### Scenario: Parent defined source meets a child shadow

- **WHEN** source 由 parent 定义而 child 提供同名 Mid 或同字节 target
- **THEN** source 后继沿 parent 定义链；parent target 与 child target 保持不同 runtime identity，不用同名或 class bytes 代替 PhysicalDefinitionId 与 defining loader

本组公开入口的24报告及六项局部门见 [AW](../../results/s1-widening-entry-scope-v1/README.md)；不称该新组完整源码重编/行为等价、独立holdout或S1整体验收。


### Requirement: Java owns enum Code candidate capture and pure generated API checks

Java层 SHALL 拥有既有 enum Code candidate/指令/引用/使用记录及从同次 MethodIr 复制这些记录的完整生成阶段，并拥有 values/valueOf 的纯 Code 形状判断及共享原始 descriptor/operand helper。root SHALL 保留请求级读取、物理索引与member配对、完整性/handler检查、pending/census、停止传播与最终提交；MUST NOT 复制第二套模型或把metadata构造当可信artifact入口。

#### Scenario: Capture preserves the selected method facts and symbolic uses

- **WHEN** 原cheap eligibility允许enum候选捕获，同次IR给出Code/operands/pool/bootstrap facts
- **THEN** 保留原candidate名字集合、table index、实际member、effective opcode/width/local/immediate/branch目标及direct/bootstrap使用顺序；非候选方法仅保留原使用记录，不额外解码Code或读取Body

#### Scenario: Code is absent or capture stops

- **WHEN** 同次IR没有Code、Code不完整，或原IrItems收费/逐条poll及bootstrap收费触发停止
- **THEN** 保持原不完整候选与complete判据，预算/取消仍以原错误返回root，不伪造Code或将停止降为普通拒绝

#### Scenario: Generated API shape is checked without granting identity

- **WHEN** root已完成唯一物理method与完整Code的原资格检查后调用values/valueOf形状判断
- **THEN** Java按原raw owner/name/descriptor/opcode核对；仅外部构造metadata或调用形状bool不得获得Engine绑定，root仍负责完整组准入与原子提交

本轮[AX](../../results/s1-enum-code-candidate-policy-v1/README.md)以1740CLI与独立24完整类入口报告保持、既有机制测试和六局部门验收纯迁移；不宣告全部enum恢复、所有私有分支或S1整体完成。


### Requirement: Java owns enum parameter and source argument policy

Java层 SHALL 拥有既有enum用户参数分类、用户/固定int/String实参纯Code证明、相关字面量/条件/实参载体及原源码拼写。系统 SHALL 移动已有定义并共享唯一实现，保留raw符号、BCI、参数顺序与原接受域；MUST NOT 通过合并plain-int/user-int判据、放宽String解码或跨类求值扩大迁移范围。root SHALL 保留同次物理事实与ClassSource记录供给、初始化器编排、停止及完整组提交。

#### Scenario: Parameter descriptors classify the existing finite argument grammar

- **WHEN** constructor尾部具有原String/ordinal前缀及一至三项已支持用户参数
- **THEN** Java按原descriptor分类和窄化拼写生成相同实参，unsupported/过宽/无用户参数形状保持原拒绝；Z约束来自参数而非任意0/1值猜测

#### Scenario: Literal and conditional arguments retain execution order and origins

- **WHEN** 同次Code满足原int静态字段/加法/同类boolean调用条件或String字面量/条件形状
- **THEN** 保留每叶/操作的原BCI、连续与跳转目标检查、单次条件求值和raw成员唯一匹配，保持原ASCII String限制及符号字段来源，不查另一类字段的值

#### Scenario: Root still owns the request and complete constructor group

- **WHEN** 源码字段声明、同次完整性或整组条件不满足，或请求预算/取消停止
- **THEN** 原root守卫与事务保持，不能把迁入Java的公开metadata/格式化结果当可信artifact或绕过ClassSource/physical identity证明；同项失败不半提交

本轮[AY](../../results/s1-enum-argument-policy-v1/README.md)在一次依赖导入编译失败后以修正源独立构建验收；旧失败保留，1740CLI+独立24报告及六局部门通过，不宣告所有实参/构造器恢复或S1整体完成。


### Requirement: Java owns enum constructor Code and field policy

Java层 SHALL 拥有既有固定 int、scalar String、String-varargs 与任意参数尾部的构造器 Code/field 纯证明及既有尾部输出类型。root SHALL 保留同次物理配对、ClassSource 事实、请求/停止与整组事务，以原位窄事实供给拆除源码判断对 ClassSourceField 的依赖；MUST NOT 复制模型、增加读取或把公开形状结果当可信 Engine artifact。

#### Scenario: Arbitrary constructor checks a faithfully declared target field

- **WHEN** 原 raw owner/name/descriptor/flags 唯一字段匹配及不同参数存储不同字段检查已通过
- **THEN** Java 在原判定位置查询供应者的同次声明存在性，缺索引或缺 declaration 仍拒绝，随后才判参数源码类型；两个实际供应者只借用已有字段记录，不预扫描或恢复另一字段

#### Scenario: Constructor shape or budget refuses the complete group

- **WHEN** 原构造器形状、额外效果、handler、槽位或预算/取消判定失败
- **THEN** 保持各固定证明不同的原接受域和拒绝优先级，三个带 Budget 的路径继续逐相邻指令 poll，任意参数路径保留 IrItems/参数 poll；root 保持原失败/停止及完整组事务，不半提交

#### Scenario: Fixed constructor guards remain at their existing stages

- **WHEN** root 已按原物理配对与完整性阶段供给同次 Code 和字段 headers
- **THEN** Java 移用原固定 int/String/String-varargs 判断，不以迁移为由加固固定 int 的历史弱检查或省略上层守卫；各原 descriptor、BCI、field index 和 super-call 来源保持

本轮[AZ](../../results/s1-enum-constructor-policy-v1/README.md)以六局部门及逐函数源等价审查验收纯迁移。missing declaration 和逐 pair 取消点未逐 guard 独立动态覆盖，24 非 enum 入口不充构造器 helper 覆盖；不授予 2.9/2.10/S1 整体验收。


### Requirement: Java owns enum constructor chain and initializer policy

Java层 SHALL 拥有既有 source-tail Signature 分类、implicit/delegation/terminal 的源码/Code/AST 政策，以及共享 initializer-prefix/factory 证明，移动原输入/输出实体并保留唯一实现。root SHALL 保留 ClassSource 事实、同次物理配对、原停止编排和最终组事务；MUST NOT 复制镜像模型或从公开metadata获得可信Engine产物。

#### Scenario: Signature tail is classified at its original parsed fact stage

- **WHEN** root按原预算读取并解析物理构造器Signature
- **THEN** Java对原四种有限尾部的Signature数量、raw构造器名/descriptor、类型参数/参数/result/throws作同一判断，root安装同一个既有enum值并保留原失败回滚

#### Scenario: Delegation requests selected Code lazily after table accounting

- **WHEN** 原表计费/poll、raw header唯一性和资格阶段通过
- **THEN** Java按原delegating、terminal、clinit顺序请求root的method_code借用回调，root核同次table/member/raw身份，不提前批量配对、不读新Body或改变原失败映射

#### Scenario: Terminal source and Code AST facts form one existing proof

- **WHEN** root保留原source/header/index/raw配对及Code.member绑定后提供同次事实
- **THEN** Java保持Signature/refusal marker、Structured/ContainsStatements/Complete、完整Code/handler/效果/BCI及AST来源判据；字段观察仅在唯一Fieldref后原位提供，缺项或不忠实字段按原原因拒绝，结果不独立授权整组投影

#### Scenario: Initializer prefix retains original argument and backing member relations

- **WHEN** 已供给的常量与构造器调用规格进入共享prefix/factory证明
- **THEN** 保留原实参语法、raw唯一字段/构造器/factory/backing关系、BCI连续性、顺序与单次效果；prefix自身不新增收费/poll，继续原上层费用和停止合同

本轮[BA](../../results/s1-enum-constructor-chain-policy-v1/README.md)以六局部门、18项既有机制测试和源迁移审查验收；不宣告所有私有guard/公开helper任意伪输入鲁棒性、2.9/2.10或S1整体完成。


### Requirement: Java owns ordinary enum group policy and use census

Java层 SHALL 拥有既有ordinary-enum group完整源码政策及direct/bootstrap使用census，两个原输出类型 SHALL 移为唯一实现。root SHALL 保留ClassSource与body-family记录、同次物理表和Code绑定、原停止与最终事务；MUST NOT 复制镜像DTO、引入新读取或将公开metadata当可信Engine产物。

#### Scenario: Applicability precedes the physical table supplier

- **WHEN** 同次class scalar事实进入ordinary group
- **THEN** 非枚举仍NotApplicable，原版本/flags/super/owner拒绝仍先于表回调；root在原完整性检查后收首次表费用并按原顺序核raw/index/flags/owner，外层预算/取消保持Stop而非Refused

#### Scenario: Raw group qualifications and use census retain their order

- **WHEN** 物理配对成功后开始常量、backing、构造器与标准API检查
- **THEN** Java保留原唯一性、准入、五个预选Code和随后全方法census次序；通过root原位method_code和Signature/member事实供应核同次Code/handler/隐藏成员使用，不提前取Code或遗漏原direct/bootstrap引用

#### Scenario: Constructor prefix and sidecar facts form the existing result

- **WHEN** 原constructor/values/valueOf/factory及prefix条件成立
- **THEN** 保持edge后terminal、所有字段声明事实、String/任意参数费用和poll、clinit使用及唯一sidecar身份/handler判断，输出相同field/method/ordinal/BCI/参数/前缀字段；root原样包装普通组，并保留后续int-field certification、Refused body fallback和nested消费

本轮[BB](../../results/s1-enum-group-policy-v1/README.md)以六局部门、20项既有机制测试和源审查验收纯迁移，不授予所有guard/任意假raw helper输入、资源总门或S1整体验收。


### Requirement: Java owns enum terminal return and suffix policy

Java层 SHALL 拥有既有terminal-return、static-assignment和map suffix完整源码/Code/AST政策及原输出类型；root SHALL 只供原位ClassSource事实，保留原facade顺序、emission/停止/安装与事务。MUST NOT 复制镜像DTO、额外读/恢复Body或把公开metadata当可信Engine来源。

#### Scenario: Terminal return uses the original raw suffix certificate

- **WHEN** 已证明ordinary group进入末端检查
- **THEN** Java保持constructor tail/签名联动、唯一同次完整Code/initializer身份与无handler条件，raw suffix仍唯一精确return；不新增sidecar.steps或quality门槛

#### Scenario: Static and map sources keep their distinct fact gates

- **WHEN** root从原记录提供借用method与field facts
- **THEN** static仍核原强physical/declaration/marker/Code/AST条件且不新增recovered要求；map仍核原index/owner/声明文本/marker/refusal与recovered完整性，不能套static强guard或增加旧未检查的记录字段

#### Scenario: Fees and selected AST changes remain at their original stages

- **WHEN** suffix形状或任一原guard拒绝，或预算/取消触发
- **THEN** 保留两种不同首guard/费用/逐项poll、None与Stop区别、static优先于map及所有primary/derived来源检查，只有完整匹配后才clone/清空static receiver/重建原元素type；root不半提交

本轮[BC](../../results/s1-enum-suffix-policy-v1/README.md)在一次导入编译失败后以修正r2源独立构建验收，六局部门与14既有机制测试通过。旧固定名称/BCI切片和IndexLoop静态疑点归S5/完整入口复现，不主张本组泛化或S1整体完成。

### Requirement: Java owns body-family constructor and initializer Code policy

Java 层 SHALL 拥有既有 body-family constructor instructions/candidate、String constructor candidate、values factory 和 initializer prefix 五个完整纯 Code 政策及其五个原事实记录。root SHALL 保留同次选中 Code/member/field 的供给、物理读取和原费用/poll、跨定义需求/use census、类族与 ClassSource 状态及全组提交。MUST NOT 新增镜像 DTO、额外读取/恢复 Body，或把 raw 记录构造当作可信 artifact binding。

#### Scenario: Constructor forwarding and field identity retain their exact proofs

- **WHEN** root 在原阶段将候选、member 身份、目标和字段表借用给 Java
- **THEN** 保留候选唯一性、complete/exception、每槽参数与 marker 转发、连续 BCI/opcode/width/local/immediate/reference 和 exact target；String putfield 仍按唯一物理目标返回原字段索引，展示资格留待后续原检查

#### Scenario: Initializer prefix and factory preserve the original rejection domain

- **WHEN** Java 消费原 constants、values member 和同次 Code
- **THEN** 保留实际名称/ordinal、allocation/constructor/field-write BCI、原参数 grammar、常量顺序、$values/$VALUES 及 suffix 的 alias/transfer/early-exit 拒绝；原返回值、拒绝文本与来源字段不变，不泛化或加固旧切片

#### Scenario: Pure policy relocation preserves the outer physical and stop protocol

- **WHEN** 物理读取、预算、取消或任一原政策停止/拒绝
- **THEN** MethodBodies/AnalysisSteps/IrItems 和 poll 留在原阶段，root 的 arbitrary 分支、direct constants、子类关系/use census、恢复/事务保持；不存在新的 query、read、完整 AST 保留或半提交

本轮 [BD](../../results/s1-enum-body-code-policy-v1/README.md)以六局部门及 16 项既有机制测试验收纯职责迁移。完整 body-family shape/sidecar/展示资格仍待后续组；不主张全部私有 guard、任意伪输入、资源门、独立 holdout 或 S1 整体完成。

### Requirement: Java owns body-family group proof policy

Java层 SHALL 拥有既有完整body-family group政策及原GroupShape：shape/API/sidecar、成员展示、常量/子类关系与构造器源码资格。root SHALL 只供原记录的借用事实，并持有读取/query/census、ClassSource、原Proved输出及整组安装/提交。MUST NOT 复制镜像DTO、复制方法报告、增加读Body或使公开raw/helper获得可信artifact binding。

#### Scenario: Group proofs retain their complete original acceptance domain

- **WHEN** 原root入口在ordinary Refused、存在relations且整体execution Complete后调用group政策
- **THEN** Java按原顺序保留String/zero/arbitrary、same-shape/implicit API/terminal clinit/sidecar writes、abstract/implicit-use、constant/direct/anonymous/child唯一性与constructor-source全部资格和拒绝文本；String与arbitrary原marker/注解条件分开，不泛化或加固旧切片

#### Scenario: Borrowed facts preserve raw records and selected child handles

- **WHEN** Java读取method/field/relation facts并匹配same-run Code/sidecar与selected child
- **THEN** 独立保留raw name/descriptor和physical identity，child override仍调用原完整recovery谓词；root在原装配点复制原源参数、subclass身份及同一Arc方法handle，ClassSource类型不下移，也不建立替代Proved模型

#### Scenario: The original budget and transaction boundary survives relocation

- **WHEN** 无child、资格拒绝或预算/取消触发
- **THEN** 保留无child先拒绝、一次AnalysisSteps费用及两个主循环poll，内层拒绝/外层Stop不混同；root原fallback/nested/preview/emission/diagnostic/事务保持，Java不启动新读取或恢复

本轮 [BE](../../results/s1-enum-body-group-policy-v1/README.md)以六局部门和20项既有机制测试验收本组纯迁移。resolver候选准备、完整source projection资格、raw证据与实际资源门继续后续S1任务；不宣告全部私有guard、任意假raw输入、独立holdout或S1整体完成。

### Requirement: Java owns pre-child candidate and shape policy

Java层 SHALL 拥有原pre-child候选、已读接口资格及最终Code/constructor/prefix shape完整纯政策。root SHALL 在原条件与阶段显式执行interface与child读取，持有snapshot/loader/query、ClassSource/诊断和提交。MUST NOT 把resolver隐藏在Java回调或通过额外读/恢复、镜像DTO、报告复制补足阶段输入。

#### Scenario: Candidate qualification gates the original explicit interface read

- **WHEN** 同次root header/Code/scan进入候选准备
- **THEN** Java保留clinit/Code/scan身份与complete、allocations/常量数量顺序、implicit API与constructor/bridge/abstract资格，tuple转交原必需局部事实；只有原abstract无自身abstract方法且唯一interface才给root该owner，读取后再按原flags/唯一无Code方法资格继续

#### Scenario: Shape proof completes before the original child loop

- **WHEN** root给出原source-field声明存在性并消费已读interface事实
- **THEN** Java按原顺序计费/筛factory/prefix/constructor chain/direct BCIs并构造原GroupShape；root随后才执行原child/Code/pool/nesting/关系和恢复流程，不提前读child、不新建报告或候选实体

#### Scenario: Pure relocation preserves refusal and stop behavior

- **WHEN** 候选不适用、形状拒绝或预算/取消发生
- **THEN** 保留七处预child费用与四处poll、空关系/shape内String拒绝/外层Error的原区别，root事务不半提交；arbitrary constructor原Error转String和旧owner UTF8门槛不在纯迁移中改语义，需完整公开入口复现后另验

本轮 [BF](../../results/s1-enum-body-candidate-policy-v1/README.md)在一处返回包装编译失败后以修正r2独立构建验收，六局部门与24项既有机制通过。临时借用索引Vec较早释放不等于实际资源门；不宣称全部私有guard、任意伪helper、独立holdout或S1整体完成。

### Requirement: Java owns complete enum body source projection

Java层 SHALL 拥有完整body-source projection资格与拼写，消费同次原field/method/source/constant和shape借用；root SHALL 保留ClassSource、child方法Arc、retained/emission安装与事务，并只用Java返回的原局部值组装唯一既有Projection。MUST NOT 新建镜像报告/DTO/Session/cache或读取来满足纯投影输入。

#### Scenario: Physical fields and selected child texts retain their original checks

- **WHEN** 已证明body group与原shape进入源码投影
- **THEN** 保留常量/field/BCI/owner/name、child配对/非空methods、声明/markers/Recovered及闭合文本、backing/implicit成员的全部原资格；MethodItem raw name/descriptor与identity独立传递，不以同名字段替代原判据

#### Scenario: Existing constructor and indentation spelling stays single sourced

- **WHEN** 固定String或arbitrary user tail构造器参与投影
- **THEN** Java保持两路原descriptor/index/tail/text条件及原参数/赋值顺序，原共享构造器和indent拼写只有一个实现供全部原消费者使用；root不持child报告副本，retained与最终class assembly继续使用同一原Projection

#### Scenario: Refusal and stop cannot publish a partial body projection

- **WHEN** 后续child资格拒绝或任一预算/取消触发
- **THEN** 保留两处poll和四处OutputBytes费用、先收费后拒绝的原次序以及外层Stop/None区别，只在全部成功后返回原五个局部值，root原回退/诊断/事务保持

本轮[BG](../../results/s1-enum-body-source-policy-v1/README.md)只验上述职责迁移；r1主动中断与r2修正记录分列。六局部门不替代2.9 raw及2.10实际资源/停止门，S1未整体完成，S2未进入。

### Requirement: Java owns complete ordinary enum source projection

Java层 SHALL 拥有完整ordinary enum源码投影资格与拼写，消费同次已有group、物理raw成员和注解借用；root SHALL 只装配原Projection并保留读取/发射/retained、ClassSource与事务。MUST NOT 新建镜像报告/DTO/Session/cache或读取来满足纯政策输入。

#### Scenario: Signature markers and annotations retain distinct original rules

- **WHEN** 普通/无源/任意user-tail或delegating构造器进入投影
- **THEN** Java按原顺序验证物理index/raw descriptor/name、声明、各自Signature marker规则与隐藏/源码参数的成员/参数/type annotations；recognized与arbitrary的不同marker门槛不得合并或放宽，原共享annotation-prefix只有一个Java实现

#### Scenario: Initializer and empty enum placement keep their full eligibility

- **WHEN** 已证明group附带initializer或没有常量
- **THEN** Java保留字段扫描唯一index、原totalUnits/Map名称/descriptor/flags/owner/markers条件及原initializer文本；空enum仅在存在原可见用户成员或initializer时写分号，root不重新证明或替换为display身份

#### Scenario: Budget and refusal cannot publish a partial ordinary projection

- **WHEN** 字段缺失、注解/构造器不适用、预算/取消触发
- **THEN** 保留原None/外层Error区别、1处poll和IrItems/两个OutputBytes收费及原次序；terminal/initializer文本仅move，六个原局部值在全部成功后交root组装，回退/停止与原子提交保持

本轮[BH](../../results/s1-enum-ordinary-source-policy-v1/README.md)只验完整ordinary投影纯迁移。六局部门和40既有机制不代表穷尽全部guard/任意伪helper、独立holdout或2.10实际资源门；2.9/2.10/S1开放，S2未进入。

### Requirement: Java owns nested enum method reference rewriting

Java层 SHALL 拥有nested enum方法引用改写的完整纯资格与文本/范围政策。root SHALL 在原条件和位置显式读取IR并保留execution/stop、ClassSource放置、物理anchors和事务。MUST NOT 通过隐藏reader callback、镜像报告、额外read/cache/Session跨层。

#### Scenario: Required origins qualify candidates before a conditional IR read

- **WHEN** 同次Recovered方法及目标/声明进入引用改写
- **THEN** Java按原收费扫描pool/dotted token、选择同方法最小完整覆盖origins并合并并列BCIs、以MethodItem raw descriptor筛声明候选；使用必要projection_origins，原不适用先退出，声明only零IR

#### Scenario: Already read Code proves original physical constant identities

- **WHEN** root按原body edits条件读取IR并合并execution、校验identity与Complete
- **THEN** Java检查完整Code、普通enum group、getstatic/CP FieldRef raw owner与field.identity.member以及常量index，保留每token唯一匹配、原String refusal；IR硬Error及预算Stop不转换为普通拒绝

#### Scenario: Complete replacements preserve range and publication boundaries

- **WHEN** 引用匹配成功或出现overlap/offset/placement失败
- **THEN** Java保留排序、单份原RecoveryReport scratch clone、倒序替换与prefix delta/硬offset错误，root原放置callback与四类anchors装配只在相应成功点调用；最终MethodText/整组提交原子性保持，不新增报告副本或声称未测资源收益

本轮[BI](../../results/s1-nested-enum-reference-policy-v1/README.md)为完整引用机制纯迁移；正确直接前继静态复核与原误位/HEAD稿分列保存。六局部门与43既有机制不穷尽全部guard/任意伪输入或2.10资源门；2.9/2.10/S1开放，S2未进入。

### Requirement: Java owns complete static fold owner source rewriting

Java层 SHALL 拥有static-fold owner字段/方法/projected来源、引用资格与替换范围的完整纯政策。root SHALL 保留真实条件owner/Exceptions与IR读取、execution/identity、原ClassSource放置、anchors/输出与事务。MUST NOT 通过隐藏reader、镜像报告、额外read/cache/Session跨层。

#### Scenario: Selected projected or physical text keeps its original source evidence

- **WHEN** override/retained text/emission参与fold
- **THEN** Java保留原不支持组合拒绝和text-only投影不回退physical规则；扫描所有覆盖origins或原retained segments，保留不同于nested enum最短origin的并集及原budget，不强制all evidence

#### Scenario: Declaration and body demands retain explicit read boundaries

- **WHEN** 字段/方法声明或body token可能命名target
- **THEN** 原descriptor资格及收费先完成，只有原descriptor不命中才由root惰性读取Exceptions；声明only零IR，body edits才由root分析并合并execution/校验identity；Java按原CP首匹配、handler、SSA顺序证明来源，不改为唯一enum字段规则

#### Scenario: Complete token bridge and capture edits publish atomically

- **WHEN** token/bridge/capture候选齐备或发生overlap/offset/placement/预算失败
- **THEN** Java沿原helper及单一PendingEdit合并逆序替换、正序修正range，保留physical-only scratch/text clone和拒绝/硬Error；root创建原anchors与OwnerTexts并保持整组提交。5处poll/5处AnalysisSteps/28处clone及原原子性不削弱，不把未测临时对象早drop充作资源门

本轮[BJ](../../results/s1-static-fold-source-policy-v1/README.md)为完整机制纯迁移。六局部门和51既有机制不穷尽全部guard/任意伪输入、独立holdout或2.10实际资源门；2.9/2.10/S1开放，S2未进入。

### Requirement: Java owns complete member fold composition qualification

Java层 SHALL 拥有member-fold的完整源码组合资格、target拼写与row/binary一致性、re-recovery源码质量门和bridge owner拼写。root SHALL 保留实际IR/recovery/stop/identity、retained模型重放供给、原ClassSource/anchors/writer及事务；MUST NOT 引入镜像DTO/Session/cache/额外读取。

#### Scenario: Family preparation preserves ordered and lazy qualification

- **WHEN** root/children存在不完整物理准备、Signature或enum/initializer承载限制
- **THEN** 沿原三轮顺序拒绝；enum命中不计算initializer order，无order不扫描clinit。原成员物理顺序、raw身份和已有拒绝域 SHALL 保持，不能按单个fixture放宽

#### Scenario: Targets follow exact names and retained child replay

- **WHEN** 已证明的family准备完整target列表
- **THEN** root精确源码名先确定，每个child先确定源码名、再由root重放原retained channels，随后核对row与raw binary并创建原target；完整列表成功后才启动实例re-recovery。borrowed factory不得把临时投影或不同物理身份装入target

#### Scenario: Re-recovery and bridge names retain source publication gates

- **WHEN** root取得同次method的recovery结果或需要bridge owner拼写
- **THEN** 原execution合并后，Java按原Complete/produced/ContainsStatements/Java/structured/无fallback门判断，失败不追加override；bridge owner仅在原时点消费原raw MUTF字节。root真实调用、2处poll/1处OutputBytes/25个clone调用点与事务 SHALL 保持

本轮[BK](../../results/s1-member-fold-composition-policy-v1/README.md)为纯机制迁移。六局部门、51机制与11相邻回归不穷尽内部guard/任意callback，不充独立holdout或2.10资源门；2.9/2.10/S1开放，S2未进入。

### Requirement: Java owns complete nested member source rendering

Java层 SHALL 拥有嵌套static/instance member完整writer的header、成员文本来源优先级、constructor/capture消除、缩进范围和source span一致性纯政策。root SHALL 保留同次物理facts及原ClassDeclarationFacts clone、现有模型和anchors factory、NestedClassSourceText封装、真实恢复/停止/事务。MUST NOT 复制报告模型或隐藏reader/cache/Session进入Java。

#### Scenario: Header and members retain physical and staged fact sources

- **WHEN** 已准备的member child渲染嵌套声明
- **THEN** 原row flags覆盖物理source flags，raw父类型与重写父类型分开供给既有声明helper；字段与方法保持原staged、retained、physical优先级、omitted/hidden处理及markers/annotation次序，原clone和staged整对象clone不替换成镜像

#### Scenario: Constructor elision preserves source ranges and physical anchors

- **WHEN** 构造器改名或已证明capture的首参数被删除
- **THEN** 沿原泛型/注解/字面值/嵌套括号扫描和范围位移，hidden-write按原line_end截断，root在原点构造原物理anchors；缩进后UTF-8边界或source span不匹配继续拒绝整项，不发布半块，不放宽既有S5接受域

#### Scenario: Shared pure helpers have one implementation

- **WHEN** nested writer或其他现有根层writer需要declaration_member/comment_member/indented_offset
- **THEN** 所有调用消费Java中的同一原纯实现，root仅保留适配；原15个.clone/1个.cloned调用点及零poll/charge保持。事实供给accessor须纯借用，不能把任意公开helper当Engine可信binding，也不能将临时对象早drop冒充实测资源收益

本轮[BL](../../results/s1-nested-member-writer-policy-v1/README.md)为纯职责迁移。六局部门与67既有测试行不穷尽全部guard/任意回调，不充独立holdout或2.10实际资源门；2.9/2.10/S1开放，S2未进入。

### Requirement: Java owns nested enum rendering and annotation source gates

Java层 SHALL 拥有nested enum完整writer及annotation原单元素资格、owner源码引用排除和块拼写纯政策。root SHALL 保留同次物理family/complete/声明、原facts clone/模型/anchors、原child/root文本重放、预算/停止与事务；MUST NOT 引入镜像DTO/Session/cache/read或为表面相似统一接受域。

#### Scenario: Enum projections retain order and separate range contracts

- **WHEN** 已证明enum的constants、fields、initializer、constructor、implicit/clinit、method或descendant参与嵌套writer
- **THEN** Java沿原次序和来源选择拼写；method replacement继续验证原source span相等，descendant沿原非空与UTF-8边界合同，root保留原事实clone和最后declaration anchors factory，不放宽或收紧S5域

#### Scenario: Annotation gates consume the original raw element facts

- **WHEN** annotation物理准备完成且需要源码投影
- **THEN** Java仅接受原单元素value()F/default/NoBody/单no-body marker形态，消费Item的raw name/descriptor而不是display或identity别名；child重放后才核精确binary source与owner引用排除，随后root原owner文本重放，拒绝次序不改变

#### Scenario: Annotation block publication keeps budget and line behavior

- **WHEN** annotation块已由Java按原visibility/static和header规则拼好
- **THEN** 每一物理method行包括空行都补四空格，root沿原1poll→原anchors→外层拼接→1OutputBytes→commit；原6个clone及两个facts clone不变，阶段临时值早drop不得冒充资源收益或全S1验收

本轮[BM](../../results/s1-nested-enum-annotation-writer-policy-v1/README.md)仅纯职责迁移。六局部门、74既有测试行不穷尽全部guard/任意回调、不充holdout或2.10实际资源门；2.9/2.10/S1开放，S2未进入。

### Requirement: Java owns narrow member rendering and method placement

Java层 SHALL 拥有narrow/generic/capture member完整writer及单一append方法放置的纯政策。root SHALL 保留原validated declaration/facts clone、原物理模型及lazy anchors factories、真实读取/恢复/停止/外层事务；MUST NOT 复制DTO/Session/cache/reader或把static-fold域套入此域。

#### Scenario: Header and field selection retain the original narrow family rules

- **WHEN** 已证明family的capture/generic/header/fields进入writer
- **THEN** Java保留原普通class_declaration与generic header clone、static插入、capture查找时点；generic字段只在原Some分支换声明并用空markers，其anchors仍独立guard；root保留原物理anchor顺序和expect，不提前做capture lookup

#### Scenario: Bridge omission and constructor edits keep their own obligations

- **WHEN** generic bridge、hidden/omitted成员或projected constructor参与family输出
- **THEN** 原identity决定bridge省略，只有hidden_fields非空才跳omitted；projected整对象先clone，只有无capture时改名，保留原source token/clone/拒绝次序，不扩大S5接受域

#### Scenario: Shared method placement preserves nested and flat contracts

- **WHEN** 方法需追加到nested或flat源码
- **THEN** Java同一append先clone原D，nested沿原indent/range/source_span相等核对，flat只原偏移；root薄适配不重建另一套政策。原13个clone与零poll/charge保持，局部失败仍由原外层事务处理

本轮[BN](../../results/s1-narrow-member-writer-policy-v1/README.md)仅纯职责迁移。五项通过及原生两既有失败对照和94既有测试结果不穷尽guard/任意回调，不充新增恢复收益、holdout或2.10实测资源门；2.9/2.10/S1开放，S2未进入。

### Requirement: Java owns source-unit target refusal

Java declaration层 SHALL 拥有源码目标对整类接口方法体/类型注解的接受判断与拒绝提示。root SHALL 只适配现有借用事实，保留物理报告、方法恢复、anchors、停止和外层事务，MUST NOT 新增镜像DTO、Session或读取。

#### Scenario: Java 7 refuses unsupported whole-unit declarations with the original reason

- **WHEN** SourceTarget低于8且原interface default/static+Code或字段/方法type-use条件成立
- **THEN** 沿原release短路和type_uses计算位置拒绝，interface body理由优先，escaped/comment_text与原文换行保持；不丢物理声明和逐方法恢复

#### Scenario: Java 8 and independent method recoveries retain their original scope

- **WHEN** Java8或未命中整类门的Java7输入进入类文本装配
- **THEN** Java返回原接受决定，lambda/方法引用的逐方法限制仍独立处理；运行环境选择、物理身份、报告、预算和外层derived提交不变

本轮[BO](../../results/s1-source-unit-target-policy-v1/README.md)为纯职责迁移。1780完整JSON对及既有回归不充新恢复收益、穷尽分支、holdout或2.10实际资源门；native两既有失败仍开放，S1未整体验收，S2未进入。

### Requirement: Enum body descriptor matching preserves raw owner bytes

恢复资格判断 SHALL 以raw owner字节构造constant对象、数组字段、values/valueOf等物理descriptor，并与原field/method descriptor作唯一精确匹配。MUST NOT 使用标准UTF8、display或lossy字符串替代物理键；Java末端拼写仍严格MUTF且保留原输入/物理身份。

#### Scenario: Supplementary owners use UTF8 archive lookup and raw MUTF descriptors independently

- **WHEN** 合法supplementary enum owner及全部CP引用/descriptor一致，完整族按标准UTF8 archive名称声明
- **THEN** Engine选中同一物理root/interface/child并保留同次Code/scan及构造证明，三evidence和SourceTarget7/8产生一致完整常量体投影；源码原样可编译，物理字段与方法报告仍可独立查询

#### Scenario: Source refusal and malformed classfile bytes retain their original domains

- **WHEN** owner的MUTF合法但不可Java拼写，或classfile MUTF已损坏
- **THEN** 前者保持明确source-name refusal，后者保持reader Failed/incomplete selection与零Body/IR逻辑工作；不能用lossy替代、别名或缺省owner放行，也不能写成NotFound/Complete

本轮[BP](../../results/s1-raw-enum-owner-boundary-v1/README.md)恢复语义41→42，其他schema/解析/inputs域不变。新变体未执行，不充行为oracle/holdout；其他A1.6闭包和2.10停止/实际资源仍开放，S2未进入。


### Requirement: Java owns complete ordinary static initializer policy

Java 层 SHALL 持有普通 class/interface 静态初始化组的完整源码准入、RHS field@1 与初始化阶段判断、顺序政策和全部表达式发射暂存。门面 SHALL 只从同次已选择的物理成员与恢复结果供给借用事实，保留读取/IR、execution/diagnostic 和最终原子提交；不得为迁移建立新的 Session、事实副本或依赖 Body 编排。

#### Scenario: Class and interface initialization retain distinct acceptance domains

- **WHEN** 已完成成员表与同次 clinit 候选交给共享静态初始化证明
- **THEN** Java SHALL 保留既有 flags/ConstantValue、唯一静态Code、完整Structured无fallback、唯一写入和全组完整性守卫，以及class/interface各自的常量阶段与前向读取限制；纯职责迁移不得扩大接受域或改变首个拒绝理由

#### Scenario: Every RHS field read matches its same-run field claim

- **WHEN** RHS AST 包含field读取或效果节点
- **THEN** Java SHALL 核对唯一同BCI claim、raw owner/name/descriptor、静态/receiver/type、putstatic之前和消费一对一、效果不重复及原写入时序；缺失、重复或错配 SHALL 整组拒绝，不因表达式可打印就放行

#### Scenario: A later projection failure leaves every declaration untouched

- **WHEN** 首个RHS已发射，但后续write BCI、字段映射、完整排列或预算/取消检查失败
- **THEN** 已暂存的片段 SHALL 不被提交；门面只消费成功的完整暂存结果后追加字段初值，物理fields顺序不变，原clinit及停止/审计平面保持

#### Scenario: Shared semantic results do not grant caller facts artifact authority

- **WHEN** 既有proof结果类型由Java定义并由门面报告引用，跨crate helper可消费调用者提供的事实
- **THEN** 既有序列化与语义版本域 SHALL 保持；helper只返回verdict/片段/错误，不授予artifact绑定权限，Engine仍仅使用内部同次事实适配和提交路径；Rust-public与trusted输入面 SHALL 分别说明

### Requirement: Enum external int field source qualification belongs to Java

Java SHALL 持有 enum 有限 int 构造表达式引用外部 static 字段的整组源码资格、owner 拼写与遮蔽政策。门面 SHALL 通过既有按需路径提供同次属性、selected owner read 及 pool/属性事实，保留物理读取、预算/执行/诊断编排；纯职责迁移 SHALL 保持现有接受域和拒绝优先级。

#### Scenario: A selected field is legal in the current enum source scope

- **WHEN** 一个候选包含外部 static int 字段引用
- **THEN** Java SHALL 按原 raw owner/name/descriptor 唯一项次序验证 source owner、同包短名/遮蔽、完整 header、版本/顶层属性及唯一字段资格；保留 raw 证据、首次 source_owner 和初始化效果顺序，不新增 owner 缓存或重读策略

#### Scenario: A source refusal or execution stop occurs during qualification

- **WHEN** 原守卫拒绝、selected read 不完整或 budget/读取回调返回错误
- **THEN** 原资格理由/优先级和 caller Refused/Stopped 编排 SHALL 保持；执行错误经外层 Result 传播，不吞成字符串或提交部分 enum 参数；有限预算报告只对实际到达阶段作覆盖结论

#### Scenario: Cross crate visibility does not grant trusted binding authority

- **WHEN** 资格 helper 跨 crate 后为调用者提供 verdict/reason
- **THEN** 系统 SHALL 单列 Rust-public 可见面变化，helper 不授予 artifact 绑定或自动提交权限；Engine SHALL 继续只供给内部同次 selected-read 事实，schema/语义版本域与可选 evidence 选择保持独立

### Requirement: Java owns common primitive array constructor group staging

Java SHALL 持有共同 primitive-array 构造族的完整资格和暂存政策，消费门面供给的原物理header、字段、方法与保留AST及已有共享writer适配。门面 SHALL 保留读取/停止编排和成功暂存后的无失败整组提交；纯职责迁移不得扩大接受域。

#### Scenario: All direct constructors share the same proved field initialization

- **WHEN** 完整构造器 census 的同次AST/member唯一配对均成功，且各构造器写入及RHS、字段raw身份/primitive descriptor、static调用资格和源码放置顺序全部满足原守卫
- **THEN** Java SHALL 按原顺序完成全部字段与构造器成员文本暂存，保留物理方法文字/BCI来源及构造器各自super/后缀；可选evidence选择不得改变源码

#### Scenario: A late conflict or output stop occurs after partial staging

- **WHEN** 首个字段/成员已发射付费，后续已有成员冲突、原资格拒绝、预算或emitter停止发生
- **THEN** Java SHALL 不返回可提交的部分组；门面字段和成员文本均保持，原局部拒绝/typed Error/Stopped分流与计费/最终poll顺序保持

#### Scenario: Existing shared member writers remain an explicit migration dependency

- **WHEN** 构造族仍通过原共享current-body及envelope/markers/annotations writer拼装
- **THEN** 方案 SHALL 明示这些仍承载源码政策，不将所有callback称为纯物理facts或据此宣布S1完成；后续迁移整个共同writer，避免各语法复制拼装或授予调用者新的Engine绑定权限

### Requirement: Java owns shared member artifact rendering and matching

Java SHALL 持有共同artifact/block、marker/annotation placement、精确UTF-8长度、current-body/explanation判断及同一placement的span政策；门面 SHALL 仅为这些共同政策适配已有物理记录/展示事实并保留提交。构造族不得反向调用root的writer政策，原projector专用资格和停止合同保持。

#### Scenario: Rendered source bytes include Unicode markers blank lines and annotations

- **WHEN** 共同writer处理有/无envelope和body、包含non-BMP/空行/多行marker与声明注解的已有材料
- **THEN** 原Block/Without/Quoted与声明拼装的字节布局 SHALL 保持；长度预估 SHALL 与实际UTF-8渲染字节相符，迁移不得新增read/cache或改变output计费

#### Scenario: A current body is compared to a recovery artifact

- **WHEN** 有declaration、可解析recovery artifact和当前body region
- **THEN** Java SHALL 按原trim/nonblank规则比较唯一自身explanation marker、envelope/statements，不将Recovered强加为整个matching的新前置；物理report仅参与原marker特判，其他marker或正文改变仍按原合同处理

#### Scenario: Shared placement and provenance have separate authority

- **WHEN** 原borrowed Artifact/Placed与格式器跨crate公开以供root适配
- **THEN** 系统 SHALL 明示这些caller-supplied材料不授予Engine认证/绑定/自动提交，parser限制和span/indentation合同保持；projector资格及2.9/2.10原门不能由共同writer完成替代

### Requirement: Java owns whole class assert staging and nested classifiers

Java SHALL 消费同次fields/methods/AST/Fieldref/nesting事实作完整assert组准入、状态receiver/census、fold/current-text、clinit stripping及暂存；门面 SHALL 仅供借用事实并在完整plan成功后按同一未改序slice位置发布真实physical index。原nested分类候选只作数据，不获得Engine绑定权，纯职责迁移不得改变接受域。

#### Scenario: A later assert body or output stage fails

- **WHEN** 首个member已实际发射付费，但后续当前正文冲突或output停止
- **THEN** 所有methods/member_text/emission/omitted通道 SHALL 保持原样；位置计划由同一只读slice选出，发布不得再有可恢复的missing-index中途分支，物理报告与原member index保持

#### Scenario: Migration observes a pre-existing stop-plane defect

- **WHEN** 迁移审计实测nested scanner Err在private组边界变为普通拒绝或emitter stop成通用Unsupported
- **THEN** 方案 SHALL 单列实际边界/原流和未通过停止门，不以迁移/native测试通过宣称修复，不外推未复现Engine结果；后续按A1.3/2.10的typed stop及全局预算合同闭合，outermost关系也须有界终止

### Requirement: Assert staging preserves typed stops and bounds enclosing metadata

Java assert 整组政策 SHALL 保留 nested scanner 的 Error 与 emitter 的 StopReason；MUST NOT 将 budget/cancel 当作资格拒绝或通用 Unsupported。root SHALL 复用原执行/诊断合并，并只发布完整成功计划。outermost 关系遍历 SHALL poll、按实际检查行计费并有限终止；重复出边或环不能获得源码证明。新增计费 SHALL 独立语义版本并保留完整报告差异，不全局屏蔽 usage。

#### Scenario: Public Engine stops after completed method recovery

- **WHEN** 在真实 assert 暂存边界，由 caller ledger 触发 AnalysisSteps/OutputBytes/ElapsedMillis 或真实 charge 后取消
- **THEN** execution SHALL 保留相应 BudgetExceeded/Cancelled；原五个已完成成员和所有提交渠道保持，逻辑 Body 数及 JVM 六阶段实际 starts 不增长；AST 暂存的 ir_items 增量 MUST NOT 当成实际 IR 启动

#### Scenario: Enclosing metadata cannot prove a terminating owner

- **WHEN** typed InnerClasses 出边构成自环、环或重复出边，或其检查被预算/取消停止
- **THEN** SHALL 有界拒绝或传播原停止，不截断为成功 outermost；正常两层链及同一 clinit literal 的匹配保持

BW结果见[记录](../../results/s1-assert-stop-boundary-v1/README.md)。typed-row测试不充恶意class公开端到端证明；CLI适配器交付错误不充内部assert结果。2.9/2.10全门继续开放。

### Requirement: Java owns integer-name group placement and publication staging

Java SHALL 拥有 integer-name 整组/方法资格、原descriptor与同次AST选择、prior staged冲突、专用text/span、唯一源范围与候选字段匹配、整类普通重放一致性及发布前计费/暂存。root SHALL 保留原物理模型、借用事实和锚点factory、既有物理整类组装器与family编排及最终无失败发布；MUST NOT 新增镜像DTO、隐藏read/cache或由root回调提供源码资格。普通root重放不匹配时 MUST NOT 调用投影组装，匹配后 SHALL 在组装前释放普通重放String，保留原持有边界。

#### Scenario: Complete plan is required before publication

- **WHEN** 前项已发射，但整类普通重放不一致，或发布阶段末尾输出/取消失败
- **THEN** SHALL 保持整个report的成员、class text、derived及projection inputs；收费及poll次序与原准入保持

#### Scenario: Method refusal remains local to that candidate

- **WHEN** 一个method的needle不唯一，或array use遇到prior staged/body replay冲突或重复调用
- **THEN** SHALL 按原接受域跳过该method，其他独立method可继续；仅通过整类重放与完整publication暂存后的结果能够提交，MUST NOT 把array专用replay门扩展为所有method的新准入条件

#### Scenario: Public controls preserve actual prepared work

- **WHEN** 真实Engine integer边界的analysis/output/cancel/elapsed控制停止本次暂存
- **THEN** 已完成三成员及原物理field/text渠道 SHALL 保持，实际reader Body decoder attempts和JVM六阶段starts零增量；停止类型 SHALL 按独立A1.3/2.10门校验，MUST NOT 把prefix测试通过当作现有elapsed误归类已修复

BX实际结果见[记录](../../results/s1-integer-name-source-policy-v1/README.md)。纯职责迁移保持恢复44与完整报告；elapsed仍Unsupported的错误已复现，另归共享错误消费者闭包，S1全门未关闭。

### Requirement: Java owns complete enum-switch method-group staging

Java SHALL 拥有同方法候选聚合、完整结构资格、额外table use排除、map与同方法拒绝、marker预算/发射/placement和整类克隆暂存，以及纯map label/coverage决定。root SHALL 仅供同次成员事实、实际跨类解析/读取IR、执行诊断及成功计划最终提交；三个原模型 SHALL 只有一个canonical定义，不新增镜像DTO或隐藏read/cache。helper准备和shared-helper table-set政策仍 SHALL 按2.9另验收。

#### Scenario: A later site or placement fails

- **WHEN** 同方法后项map拒绝、额外table use、marker停止或后置placement失败
- **THEN** SHALL 保留原方法正文及marker，不提交部分成员，未发布proof不得标为projected；保留原拒绝顺序及费用，局部方法拒绝不扩大为所有独立方法拒绝

#### Scenario: A completed enum batch records a global stop

- **WHEN** 枚举阶段使本次execution进入全局预算或取消终止
- **THEN** root SHALL 刷新ended，后继array与initializer入口 SHALL 检查当前ended，传播停止用的token取消 MUST NOT 覆盖首个预算原因；已完成成员和实际Body/IR前缀保持。此局部验收 MUST NOT 代替2.10全部停止/消费者闭包

BY实测见[验收](../../results/s1-enum-group-source-policy-v1/README.md)，停止修复恢复44→45，随后纯迁移不增加revision。共享Interrupted及其他全局停止入口仍开放，S1未整体验收。

### Requirement: Java owns enum-helper preparation and shared-table proof policy

Java SHALL 持有完整selector/verifier/ordinal、helper与enum/field/values/clinit/构造器及常量身份准备政策，及原shared-helper table-set、同table enum身份/常量一致性、key合并和结果展开。root SHALL 仅适配原scope/target、同次ConfirmedRead借用facts、实际解析/读取IR与执行合并，按原index收集结果；PreparedEnumSwitchMap SHALL 只有一份canonical定义，不新增镜像DTO、Runtime或隐藏read/cache。

#### Scenario: Closed table-set or definition consistency fails

- **WHEN** 既有多候选helper的selected table-set不闭合，或同table输入的enum定义/常量矛盾
- **THEN** SHALL 保留原判断顺序：closed-set在helper IR前拒绝，definition冲突在同一次真实helper IR之后拒绝；原方法与marker不提交，拒绝理由、物理身份和计费保持，MUST NOT 为测试变异执行目标

#### Scenario: Request owner supplies same-run payloads

- **WHEN** Java准备证明请求已选中的Header或enum四段IR
- **THEN** SHALL 由root提供原物理环境中的ConfirmedRead完整载荷，仅借用facts，保持原两次resolve/read及四次IR后execution合并；helper IR原停止消费缺口 SHALL 另按2.10复现修复，MUST NOT 将纯迁移通过称作全停止门通过

BZ结果见[记录](../../results/s1-enum-helper-source-policy-v1/README.md)，2449继承报告与408核心保持，恢复45不变。2.9/2.10及S1整体未验收，S2未进入。

### Requirement: Enum source policies relay the supplier's original typed stop

Java enum准备、shared-helper证明与method-group staging SHALL 区分inner形状拒绝与outer供应方失败；outer失败 SHALL 保留既有Error/Report类型，经owned回调由root消费。root SHALL 保留真实依赖read/IR的原始execution和diagnostics，报告型全局预算/取消 MUST NOT 降成字符串继续后继需求，MUST NOT 构造原报告没有的Error quota字段。局部不完整仍遵循原独立拒绝域，已有合法成员前缀保持。

#### Scenario: A helper returns a global partial or cancelled report

- **WHEN** 实际helper IR因analysis/body预算、取消或deadline返回request-ending报告
- **THEN** SHALL 只完成当前helper需求，后继helper不得启动；原execution/diagnostics传至root并保留到class结果，已完成methods/markers保持、enum proof不标projected，后继实际Body/六IR增量零，initializer明确说明停止后未尝试

#### Scenario: A proved helper's map proof returns an error

- **WHEN** 完整实际helper IR后的map证明因IrItems预算或取消返回原typed Error
- **THEN** SHALL 保持该错误的code/text与原前缀，外层按既有错误通道消费一次，不因新增Report relay改变原正确路径

CA实测见[记录](../../results/s1-enum-helper-stop-v1/README.md)，恢复45→46。共享Interrupted、其余相邻入口与所有read切点仍另验收，MUST NOT 以本六控制替代S1全停止/资源/API门。

### 2026-10-26 / CB：共享成员放置政策归 Java

[结果](../../results/s1-anonymous-member-placement-v1/README.md)、[验收](../../results/s1-anonymous-member-placement-v1/acceptance-r1.json)、[六项](../../results/s1-anonymous-member-placement-v1/six-gates.md)、[后继职责/依赖/验收](../../results/s1-anonymous-member-placement-v1/next-boundaries.md)。八个关联body/span/匿名成员方法的声明/marker/注解准入、child flags、默认缩进、root-return/current-prefix放置与UTF-8范围核验归Java；root仅供原字段/Recovered变体事实并构造原index/kind/anchors，私有parts移除，无新模型/trait/read/cache。各接受域差异、clone/文本顺序和预算/发布保持，恢复46/schema12/解析4/v9不变。高层匿名interface/nested/super/double-brace候选、Header/owner/capture资格与专用暂存/冲突决策仍在root逐成员编排中，不能称整族已迁完；物理提交继续留root。

原政策同三新边界测试3/0；当前16目标473/0/0，原11政策/停止控制完整保持。2449继承（已含168枚举）及408核心全JSON/退出码严格相等，零语义/计费例外；CA42份metadata修正沿接受基线核对，不改其原false流。168全文/78声明/136×3及核心6编译12受控VM、真实两类4重编通过，真实目标未执行；新测试只复用旧JAR和布局事实变体，不计新恢复或holdout。2744源/资产、归档、17产物与正常构建绑定，target清理，磁盘仅构建护栏。第三轮历史六项与原JDK8/旧native失败限制保持。共享Interrupted、其他停止/资源/API、raw闭包及高层匿名族仍按2.9/2.10承接，17/59，S1未整体验收，S0原记录保持，S2未进入。

### 2026-10-26 / CC：匿名接口跨 IO 阶段资格与 emission 归 Java

[结果](../../results/s1-anonymous-interface-source-policy-v1/README.md)、[验收](../../results/s1-anonymous-interface-source-policy-v1/acceptance-r1.json)、[六项](../../results/s1-anonymous-interface-source-policy-v1/six-gates.md)、[后继职责/依赖/验收](../../results/s1-anonymous-interface-source-policy-v1/next-boundaries.md)。site/root-child typed nesting、capture参数与返回类型、平台/实际接口契约、构造器/完整方法/scan闭集和捕获AST重写—child文本—root匿名返回emission归Java；共享唯一allocation/nested-site/complete-method政策及三个既有模型同迁。root仍负责实际resolve/read/IR准备、执行合并、锚点/物理装配和发布；flags/facts/bytes getter只借用原数据，placement调用CB已迁政策，无新DTO/trait/read/cache，恢复46/schema12/解析4/v9保持。共享capture/owner census仍含政策、nested/super/double-brace专用决策仍待迁，不称纯IO或整族完成。

旧政策同三新测试3/0，当前16目标476/0/0；4正例/7早晚拒绝/3停止共14控制逐项保持，原11控制保持，三停止后实际Body/六IR全0且无半提交；elapsed仍Unsupported(jre_budget_interrupted)，单列2.10，不冒称首因正确或全部read点通过。2449继承+20匿名profile（5既有JAR×source7/8×essential/all，40CLI）=2469完整报告/退出码严格相等，408核心相同，零语义/计费例外；168全文/78声明/136×3及核心6编译12受控VM、真实两类4重编保持，真实目标未执行。新控制只用既有冻结class/JAR和内存facts变体，不计恢复增量/holdout。r1选错capture夹具2pass/1fail原流保留，r2修正输入不改断言；2744源/资产/归档、17当前产物及正常构建绑定，target清理，磁盘不充运行资源。临时row/definition所有权范围提前仅静态观察，S0原门/历史失败不豁免。2.9/2.10/S1开放，17/59，S2未进入。

### 2026-10-26 / CD：完整共享 capture 证明簇归 Java

[结果](../../results/s1-member-capture-source-policy-v1/README.md)、[验收](../../results/s1-member-capture-source-policy-v1/acceptance-r1.json)、[六项](../../results/s1-member-capture-source-policy-v1/six-gates.md)、[后继任务/依赖/验收](../../results/s1-member-capture-source-policy-v1/next-boundaries.md)。member_capture模块接管完整Outer/instance/double/val捕获、constructor参数分区、local单写、SSA使用/唯一写/MethodHandle/exception及两receiver域，两个原类型同迁，root仅重导出同一实现；anonymous闭表/Code预检、method调度与三分流归Java，root callback留实际analyze/执行合并/可选原Report relay。无镜像DTO/trait/runtime/cache/read，恢复46/schema12/解析4/v9保持；一般family/instance IR编排、call-site/bridge/owner census与专用匿名族仍待迁，不称整族完成。

原代码同三测3/0，当前16目标479/0/0；新14控制（2成功/4早拒/8真实IR stop）与原25控制逐项保持。四早拒wrapped-reader/Body/六IR为0；八stop均1请求、停止后三类计数全0。legacy四次Ok(None)无原Report/diag sink已复现，opt-in四次原execution/diagnostics保留；两种局部execution均已合并，尚未证明公开假Complete或修typed消费者，2.10继续开放。2469继承全JSON/退出码及408核心严格保持，零语义/计费例外，含168枚举/20匿名子集不另跑矩阵；168全文/78声明/136×3及核心6编译12受控VM、真实两类4重编保持，真实目标未执行，不计新holdout。2745源/资产（仅新增机制模块）、归档/17当前产物与构建绑定，target清理。r1 lib递归格式化三无关文件被scope门拦在Cargo前并按封存字节恢复；r2仅旧单测缺两helper导入而编译失败，r3补同实现别名/隐藏pub，不改证明或断言，原流保留。S0原门/历史失败不豁免，2.9/2.10/S1开放，17/59，S2未进入。

### 2026-10-26 / CE：capture 原始停止报告消费者闭包

[结果](../../results/s1-capture-stop-consumers-v1/README.md)、[验收](../../results/s1-capture-stop-consumers-v1/acceptance-r1.json)、[六项](../../results/s1-capture-stop-consumers-v1/six-gates.md)、[后继任务/依赖/验收](../../results/s1-capture-stop-consumers-v1/next-boundaries.md)。统一 capture 依赖的实际 IR/原始停止返回通道。Java anonymous capture 调度透传泛型供应方错误；Root 复用同一 DependencyProofFailure，集中实际分析、执行合并及原 Report/diagnostics relay，anonymous/val 的 optional sink 与三个 legacy/strict 包装删除。普通 member、instance、匿名 interface/nested/super 和 double-brace constructor partition 的公开消费者均接收同一通道；源码形状拒绝保持原域，capture certificate 判断未改变。恢复46→47，schema12/解析4/inputs9，无新DTO/Runtime/cache/read。

先在原代码同三测试复现2pass/1fail：48公开控制中8充分预算正例、8后置proof Error及double-brace两路8个Report stop原已通过；其余24个Report stop失败。member/instance原总体停止类型正确但原diag丢失，interface/nested/super另有停止原因被Unsupported覆盖；没有公开假Complete证据。修后16目标482/0/0；48控制全通过，32真实IR Report逐字消费原execution/diag一次，8 proof Error保留AnalysisSteps，均保持已发布源码/方法prefix，停止后actual wrapped-reader calls/bytes、Body decoder及六IR starts为0。原25政策控制保持，CD原6成功/早拒与4 strict报告保留，删除4 legacy可选sink控制，现CD10；不是新增独立程序/holdout。

2469继承全JSON/退出码严格保持，含168枚举与20匿名profile子集，无额外矩阵CLI；408核心全JSON保持，零语义/计费例外。168完整源码/78声明、136×3核心、核心6编译12受控VM、真实两类4重编保持，真实目标未执行。2745源/资产、归档与17当前产物及三正常构建绑定，target清理；baseline-r1切点修正和current-r2静态scope纠正均主动终止，原流/归档/exit -15保留，无native运行。构建磁盘护栏不作S0运行资源证据。

共享capture证明簇已归Java，typed供应/消费闭包现补齐；剩余ordinary/instance方法需求编排、owner census、call-site/bridge与nested/super/double-brace专用资格/暂存事务仍按机制迁移。shared emitter Interrupted、其他read/bridge/sink/资源/API及旧native失败保持开放，第三轮历史六项原JDK8/失败限制不豁免。2.9/2.10未勾选，OpenSpec17/59，S1未整体验收，S0原记录保持，S2未进入。

### 2026-10-26 / CF：普通 member capture 需求政策归 Java

[结果](../../results/s1-member-capture-demand-policy-v1/README.md)、[验收](../../results/s1-member-capture-demand-policy-v1/acceptance-r1.json)、[六项](../../results/s1-member-capture-demand-policy-v1/six-gates.md)、[后继依赖/验收](../../results/s1-member-capture-demand-policy-v1/next-boundaries.md)。承接CE已封存的capture原始Report消费者闭包，继续迁移普通member capture的方法需求政策。Java member_capture::prove_member_capture接管原Vec::new、物理methods原顺序、每项先poll再筛Code、PhysicalMethodId构造、IR身份/Complete校验、IR借用及同一prove_family_capture调用。Root仅保留同一实际analyze_capture_dependency callback、typed停止/执行diag合并与ClassSourceMemberCapture枚举拼装。无新DTO/trait/Runtime/cache/read，原拒绝文案、严格六指令Outer证明域、计费/poll/clone顺序保持；所有旧Java证明与anonymous pipeline、Root其他函数/测试逐字未动。恢复47/schema12/解析4/inputs9保持，instance的普通构造体扩展域未混入。普通证明的Root重导出现仅测试使用，新增unused import警告已逐项对照记账，随member_inner剩余调用者/重导出收口清理，不声称零警告。

当前16目标482/0/0，原25政策、CD10 typed与CE48公开控制共83份完整记录严格保持；48包括8正例、32真实IR Report及8 proof Error，原报告/诊断一次消费、prefix与实际停止工作截断保持。CE原代码2/1和24公开Report传递失败、修后全通过的原证据不改，不重复算新恢复/holdout。本轮无新测试/目标输入。

2469继承全JSON/退出码、408核心全JSON严格相同，零语义/计费例外，含168枚举/20匿名profile子集不另跑矩阵；168完整源码/78声明、136×3、核心6编译12受控VM及真实两类4重编保持，真实目标未执行。2745源/资产、归档与17当前产物及三正常构建绑定，target清理。r1脚本parent路径预检错误发生在Cargo/native启动前，原流保留；r2纠正路径后正常构建，不改生产代码。构建磁盘不充S0运行资源。

下一组依次处理instance需求编排及IR持有期、owner census/call-site/bridge资格与nested/super/double-brace专用事务。共享Interrupted、全部read/bridge/sink停止/资源/API和历史native失败仍归2.9/2.10，不能用482/0覆盖。第三轮六项原JDK8/历史限制、S0记录保持；2.9/2.10/S1开放，OpenSpec17/59，S2未进入。

### 2026-10-26 / CG：instance capture需求与IR持有期

[结果](../../results/s1-instance-capture-demand-policy-v1/README.md)、[验收](../../results/s1-instance-capture-demand-policy-v1/acceptance-r1.json)、[六项](../../results/s1-instance-capture-demand-policy-v1/six-gates.md)、[后继依赖/验收](../../results/s1-instance-capture-demand-policy-v1/next-boundaries.md)。Java member_capture::prove_instance_capture接管instance物理methods的原顺序需求、每项先poll再筛Code、PhysicalMethodId、IR身份/Complete门与同一prove_family_instance_capture调用；Root保留物理重读、实际IR callback/typed执行诊断、fold target/bridge/census与提交。原Vec::new、拒绝文案及instance capture refused前缀、允许普通构造体/实际capture write BCI的扩展域保持，不误用普通member六指令证明。所有旧Java证明/ordinary/anonymous政策及Root其他流程逐字不变；普通proof的Root导入限定cfg(test)，instance proof不再Root重导出，CF新增unused import已清。恢复47/schema12/解析4/inputs9保持，无新产品DTO/Runtime/cache/read。

先增加仅test-support的真实MethodIr构造/drop计数，正常构建无该字段/状态；已有事件Counts和产品报告不扩展。基线同两新测试1pass/1fail：完整N1捕获后的2个IR及2个ClassFacts在proof、bridge、census到done仍存活，returned才归0；五种实际IR/后置proof停止已释放且停止后实际reader calls/bytes、Body、六IR全0。迁移后Java只返回自有MemberCaptureProof，分析/借用缓冲在证明返回前释放；同一五阶段IR和ClassFacts均2→0，五stop保持、所有六控制通过。ClassFacts为值载荷数而非物理类数/Arc引用数；只证明该受控路径的对象持有期，不当heap字节、RSS/时间或完整S0资源改善。

当前17目标489/0/0：Root308、原14 integration174、Java assert2、追加既有inner_class_static_mixed_folding5；其中只有两项是本轮新测试，五项为已有整族源码/重编/受控行为/停止回归。原83完整控制记录严格保持，CG6的请求、身份、执行、diagnostics、usage及fold全文保持；仅normal五阶段的两个存活计数变化逐点保存，原预算/取消与已发布prefix保留。复用旧N1三class及原fixture，不计新holdout或恢复增量。

核验实际engine、registry.jvm来源指纹并仅规范直属usage.elapsed_millis后，2469继承完整JSON/退出码与408核心完整JSON保持一致，零语义/计费例外；原r1规范下377份相同、2092份共6538处仅JVM来源身份变化，均与父/当前实际Cargo绑定；r2复用原2469次CLI输出，不增加CLI。168枚举/20匿名profile已含，无额外矩阵CLI；168源码全文/78声明、136×3、核心6编译12受控VM和真实两类4重编保持，真实目标未执行。2745源/资产、归档、18当前产物与三正常构建绑定，target清理。r1迁移静态断言误拒report.execution读取，source scope门在Cargo前拦截未完成转移，原脚本/错误和空目录保留；r2修正断言后构建，无证明或测试断言调整。观测基线与迁移后用同一测试源码，21项纯文本迁移核对及正常CLI无test-support features验证通过。两次审查脚本误判及原false输出、纠正收据分列保留，不改生产/测试断言；构建磁盘不充运行资源。

下一组owner census/自调用准入与call-site/bridge资格，然后nested/super/double-brace完整资格及暂存事务；shared emitter Interrupted、其余read/bridge/sink停止、资源/API、raw消费者闭包和历史native失败仍开放。第三轮六项原JDK8/历史限制、S0原记录保持；2.9/2.10/S1未关闭，OpenSpec17/59，S2未进入。
