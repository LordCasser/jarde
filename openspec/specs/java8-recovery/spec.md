# java8-recovery Specification

## Purpose
在 Java 8 RuntimeProfile 下，将有可靠 IR 证据的历史与 Java 8 字节码恢复为可读 Java 结构，同时保持求值顺序、异常、副作用和原始引用事实。

## Requirements

### Requirement: Evidence-gated Java 8 recovery

系统 SHALL 仅在对应 IR、descriptor、异常/effect 和编译器模式前置条件满足时启用 Structured recovery；证据不足时 MUST 返回 representation=Mixed/Bytecode、quality=Conservative/Fallback，并说明缺失条件。
系统 MUST 遵守本 requirement 下所有“完整契约”场景中的全部约束。

#### Scenario: 完整契约 1

- **WHEN** 系统尝试从 IR、descriptor、异常/effect 与编译器模式证据恢复并呈现 Java 8 方法
- **THEN** 系统 MUST 满足以下全部约束：

系统 SHALL 仅在对应 IR、descriptor、异常/effect 和编译器模式前置条件满足时启用 Structured recovery；证据不足时 MUST 返回 representation=Mixed/Bytecode、quality=Conservative/Fallback，并说明缺失条件。值的正确性 SHALL 按生成代码实际求值的位置与顺序验证，嵌套表达式不能以其原始生产位置代替当前求值位置；当一个算术的结果作为另一个算术的操作数时，呈现 MUST 使用该操作数自己的值并保留这一嵌套，MUST NOT 省略外层运算、也不得把某个操作数改写成另一种运算（例如把 `i * (2 - d * i)` 写成 `i * 2 - d * i`）。**该嵌套也 MUST 体现在输出文本的分组上**：表达式树到文本的转换 MUST 按运算优先级与结合性补足括号（或写成不依赖默认结合的形式），使文本解析回同一棵树；仅凭「树是对的」不满足本要求，因为调用方收到的是文本。分组 MUST 按子表达式所在的**语法位置**判定，而不只按它的父运算符：同一个子表达式写在调用或字段读取的 receiver、方法引用限定符、一元操作数与数组位置时 MUST 同样保持自己的分组（例如 `return (a + b).substring(1);` 的接收者），而 Java 语法已经界定该子表达式的位置（调用/构造实参、下标、lambda 体）MUST NOT 增加无谓括号。层不能证明某操作数的值或求值点时 MUST 拒绝该区域而不是呈现，并保留被拒区域的 bytecode 与 origin；quality、content、execution 等结构性平面 MUST NOT 代替这一证据。fallback MUST 保留被省略表达式所依赖的可观察生产者及其物理 origin，包括字段读取、类初始化与可能抛异常的操作。


#### Scenario: Java 8 lambda and method reference

- **WHEN** invokedynamic 符合已验证的 LambdaMetafactory 形态且实现 handle、SAM descriptor 和 capture 可追溯
- **THEN** 输出可读 lambda/method reference，并保留 bootstrap/use-site/origin evidence；任意 bootstrap 不得套用该模式（验收 A04）

#### Scenario: String concatenation order

- **WHEN** IR 证明 StringBuilder/StringBuffer 拼接模式及每个转换的求值顺序
- **THEN** 输出拼接表达式或等价结构，不重复调用、移动可能抛异常的操作或丢失转换语义

#### Scenario: Generic output without a compiler-specific match

- **WHEN** 方法已满足支持范围内的普通控制流、类型与 effect 前提，但没有命中编译器语法糖规则
- **THEN** 仍能生成带稳定名称与 source map 的通用 Java 表达；不得仅因没有语法糖匹配就放弃已证明的基础结构或声称恢复了原始源码写法

#### Scenario: Loop header has observable effects

- **WHEN** 可恢复循环的条件或 header 包含调用、读取或可能抛异常的操作
- **THEN** 输出保持这些操作在每条正常/异常路径上的执行次数与次序；不能把每轮执行的操作移到循环外，证明不足时保留可靠表示与降级原因

#### Scenario: A loaded value survives a later local write

- **WHEN** 一个值经 load 留在 operand stack 上，原 local 随后被 iinc/store 覆盖，旧值之后才被 return、调用或条件消费
- **THEN** 恢复 SHALL 使用 load 时的 SSA 值，不重新读取已经改变的 slot；例如 `return x++` 必须返回递增前的值，不能因结构可识别而标记语义不同的 Java 为 Structured

#### Scenario: A consumer falls back after reading a call result

- **WHEN** 调用结果的消费者属于未证明的 cast/field/其他形态，消费者最终不能生成表达式
- **THEN** 生产者调用及其求值顺序 SHALL 保留为可靠语句或完整 fallback 范围与 origin；不能先省略生产者，再仅引用消费者 BCI，不能因 quality=Fallback 就丢失 effect

#### Scenario: A nested expression is evaluated after a local write

- **WHEN** 一个局部变量先参与子表达式求值，其后被写入，而该子表达式的结果更晚才被消费，例如 `(x + 1) + ++x`
- **THEN** 输出 SHALL 保留先前子表达式的值；输入 7 时若生成可执行 Java，结果必须为 16。无法证明时 MUST 明确降级并保留相关生产者/消费处证据，不能生成返回 17 的 Java/Structured，也不能因子表达式原始 BCI 位于写入前而放行

#### Scenario: A refused cast depends on a static field read

- **WHEN** `getstatic External.value; checkcast String; areturn` 的 cast 不能可靠呈现，字段读取可能触发类初始化或异常
- **THEN** 输出 SHALL 保留字段读取的语句或完整 fallback 引用，并为读取 BCI、消费 BCI 和物理方法提供 source map；仅保留 cast/return 或独立的字段识别记录不满足产物的 effect 与来源完整性。无需执行类初始化或读取无关 Body 来保留此证据

#### Scenario: An arithmetic operand is the result of another arithmetic

- **WHEN** 字节码把一个算术的结果作为另一个算术的操作数，例如 `iload_1; iconst_2; iload_0; iload_1; imul; isub; imul; istore_1`（源码形状 `i = i * (2 - d * i)`）
- **THEN** 呈现 MUST 使用该操作数自己的值并保留嵌套，输出 `local1 * (2 - arg0 * local1)`；MUST NOT 省略外层 `imul` 或把 `isub` 的常量左操作数写成乘法（`local1 * 2 - arg0 * local1`），因为那会计算另一个值

#### Scenario: A presented body computes what its bytecode computes

- **WHEN** 受控 fixture 以一组输入（至少包含 `d = -1` 与一个正向输入）分别执行原 class 与呈现文本
- **THEN** 每个输入的返回值/可观察行为 MUST 与原 class 相同；`d = -1` 时原 class 返回 `-1`，呈现文本不得返回 `-81`。呈现被拒绝时改由拒绝边界验收，不给「未生成 Java」留通过路径

#### Scenario: An unproven arithmetic operand is refused

- **WHEN** 层无法证明某个算术操作数的值或它实际被求值的位置
- **THEN** 该区域 MUST 拒绝并保留 bytecode 与 origin（representation=Mixed、quality=Fallback、拒绝诊断点名相关 BCI），MUST NOT 省略该操作数或换一个定义后继续呈现；拒绝产物按既有 refusal 契约可定位

#### Scenario: The structural planes do not stand in for this

- **WHEN** 一份产物报告 quality=Structured、content=contains_statements、execution=complete 且没有诊断
- **THEN** 这些平面 MUST NOT 被当作值正确性的证据；本 requirement 的证据只能是受控执行对照，或对相关操作数值与求值点的证明。已知反例表明同时满足上述三个平面的产物仍可计算与字节码不同的值

#### Scenario: Grouping survives printing

- **WHEN** 表达式树把一个优先级更低或结合方向不同的子表达式放在某个运算的操作数位置，例如 `a * (2 - b * a)`、`a - (b - c)`、`a / (b * c)`、`a - (b + 1) * 2`
- **THEN** 呈现文本 MUST 保持该树的分组：打印时按运算优先级与结合性补足括号（或等价地写成不依赖默认结合的方式），使文本按 Java 语法解析回同一棵树；MUST NOT 依赖默认优先级与结合性直接拼接子表达式。四个形状中的每一个都必须解析回原树，且以受控执行对照验收（`a * (2 - b * a)` 已实测被打印成 `arg0 * 2 - arg1 * arg0`，其余三个同样丢失分组）

#### Scenario: A receiver keeps its own grouping

- **WHEN** 呈现把某个子表达式写在调用或字段读取的 receiver 位置，例如受控样本 `return (a + b).substring(1);`（receiver 是拼接链构造的加法表达式）
- **THEN** 文本 MUST 保持该 receiver 自己的分组，输出 `return (arg0 + arg1).substring(1);`；MUST NOT 直接拼接成 `return arg0 + arg1.substring(1);`——后者按 Java 语法解析为 `arg0 + (arg1.substring(1))`，是与字节码不同的程序（已实测：该文本编译执行后在 `("a", "bc")` 上返回 `"ac"`）

#### Scenario: Grouping is decided per position

- **WHEN** 打印机把子表达式写入任意位置（二元操作数、调用与构造实参、调用 receiver、字段 receiver、方法引用限定符、数组与下标、一元操作数、lambda 体）
- **THEN** 该位置需要分组时才补括号，Java 语法已经界定该子表达式的位置 MUST NOT 增加括号；判定 MUST 按位置作出，不能只看父运算符是哪种二元运算，也不能因为某个位置由同一个打印臂写出就认为它已受检

#### Scenario: A receiver's text is executed, not inferred

- **WHEN** receiver 受控样本的原 class 与呈现文本在各自正确签名下分别编译执行，输入为 `("a", "bc")`
- **THEN** 两侧返回值 MUST 相同：原 class 返回 `"bc"`，呈现文本 MUST NOT 返回 `"ac"`；呈现被拒绝时改由拒绝边界验收（拒绝范围、被引用 BCI 与物理方法映射），不给「未生成 Java」留通过路径

### Requirement: Historical and Java 8 compiler patterns

系统 SHALL 覆盖已验证的 synthetic accessor、inner/local/anonymous capture、bridge、enum/enum switch、try-with-resources、default/static interface、synchronized/finally 和构造器/字段初始化模式；编译器来源未知时 MUST 使用 generic JVM semantics。

#### Scenario: Synthetic accessor recovery

- **WHEN** accessor 的 flags、body 和调用适配满足已验证模式
- **THEN** Java 输出可显示直接字段访问，同时 X1 保留调用 accessor 和 accessor-to-field 两条原始边（验收 A12）

#### Scenario: Missing debug metadata

- **WHEN** Java 8 方法没有 LVT 或 LineNumberTable
- **THEN** 恢复使用确定性 arg/local 名称或 Conservative 输出，不因缺少 debug metadata 伪造源码作用域（验收 A10）

### Requirement: Semantics-preserving fallback

恢复 MUST 保留异常优先级、资源关闭/抑制异常、monitor、类初始化和副作用顺序；未获本规范明确的完整源码变换证明的不可约 CFG、非法 Java 名称或不满足模式时 MUST 降级，不生成伪等价空代码。明确受证的条件初始int前缀循环仅按其独立要求恢复，不将该例外推广至其它不可约图。

#### Scenario: TWR and exceptional close

- **WHEN** 方法包含 try-with-resources 的正常和异常 close 路径
- **THEN** 输出保留 close 顺序、primary/suppressed exception 关系；无法证明时返回 representation=Mixed/Bytecode、quality=Conservative/Fallback 和诊断

#### Scenario: Unstructured control flow

- **WHEN** 方法的区域恢复遇到没有明确完整变换证明的不可约 CFG 或交叉异常区域
- **THEN** 结果保留可靠低级结构和 origin，不能输出空 body 或成功 Structured 标志；若扫描完整，coverage 仍可为 CompleteWithinSchema，只有扫描失败才为 Partial（验收 A13）

### Requirement: Read-only IR handoff owned by jarde-jvm

恢复层 SHALL 只通过 `jarde-jvm` 自有的只读载荷取得一个方法请求的真实 IR（canonical CFG、frames、SSA/effects 及其 origin），该载荷 MUST 就是该次运行发布的那三张表本身，MUST NOT 由 `MethodAnalysisReport` 的摘要字段重建，也 MUST NOT 给恢复侧留下可变访问或伪造表的构造路径。所有权 SHALL 为“一次请求产生一份载荷、调用方在其作用域内以 `&` 交给恢复层”；交接 MUST NOT 重复运行 P2、重复计费或改变既有停止/取消语义。恢复侧实现（`jarde-java`）只在 1.3 随首个真实闭环创建，1.1 不为它建通用 backend trait、动态 pass 注册或跨层 IR 抽象。

#### Scenario: Recovery input is the published table

- **WHEN** 一个方法请求跑到某阶段并发布了 canonical/frames/SSA 表
- **THEN** 恢复侧能读到这些表的只读内容（块/边/throw site/handler row/帧/值/phi/effect 与 origin），且其中至少一个量（例如块数、BCI 或 phi 数）在报告里没有对应字段

#### Scenario: Stopped or refused run

- **WHEN** 请求因预算、取消或环境被拒而停止
- **THEN** 载荷只含停止前已发布的那部分表（环境被拒时为空载荷），报告保持原有阶段/执行/诊断语义，且两个入口对同一请求的计费与停止一致

#### Scenario: Accessor evidence is demanded through the public entry

- **WHEN** 公开恢复入口决定检查某个 accessor 候选
- **THEN** callee 声明/Body/CP SHALL 绑定到实际物理定义并由同一请求预算按需读取，报告读取 reason；普通方法不因此预装其他 Body。没有可靠 callee 证据时保留原调用和拒绝原因，低层 API 的独立成功不能替代公开入口的呈现验收

### Requirement: Recovery artifact content is observable independently

恢复报告 SHALL 返回 `content`，取值为 `not_produced`、`explanation_only`、`contains_statements`。
系统 MUST 遵守本 requirement 下所有“完整契约”场景中的全部约束。

#### Scenario: 完整契约 1

- **WHEN** 系统处理恢复报告与产物交付
- **THEN** 系统 MUST 满足以下全部约束：

恢复报告 SHALL 返回 `content`，取值为 `not_produced`、`explanation_only`、`contains_statements`。该字段 SHALL 描述最终交付产物：Stopped 为 not_produced；Produced 且只有包装、理由、BCI 引用等说明为 explanation_only；Produced 且含至少一个实际发射的 Java 语句为 contains_statements。Produced MUST 仅表示产物已交付，不能解释为存在语句、语义完整、可编译或已验证。

#### Scenario: 完整契约 2

- **WHEN** 系统处理恢复报告与产物交付
- **THEN** 系统 MUST 满足以下全部约束：

声明、赋值、调用、return 和控制流语句均可构成 contains_statements；该分类不改变 representation、quality、syntax、coverage、execution 或验证状态。分类 MUST 不依赖注释剥离、token 相似度或调用方二次解析文本。库与 CLI SHALL 返回同一分类。

#### Scenario: 完整契约 3

- **WHEN** 系统处理恢复报告与产物交付
- **THEN** 系统 MUST 满足以下全部约束：

`Produced` 与 `content` MUST 分开统计与表述：同一轮 sweep 的 182,883 个请求中有 30,452 个是 Produced 而产物不含语句；报告、README、支持矩阵与 benchmark 协议 MUST NOT 把这类计数表述为语句恢复率或语义恢复率，MUST 分别命名「引擎 content 分类」与旧 token/启发式口径。content 与其余平面一样是结构性的，MUST NOT 被读作值等价或语义正确的证据。


#### Scenario: Explanation-only produced artifact

- **WHEN** 方法产物只有 BCI 引用、降级理由和成员包装说明
- **THEN** outcome 仍为 Produced，content 为 explanation_only，不因文本非空或存在 fallback 节点被统计为 contains_statements

#### Scenario: Partial structure has a statement

- **WHEN** 产物包含 `if (arg0) {}` 或已证明的 `return;`，同时仍有未恢复区域的说明
- **THEN** content 为 contains_statements；既有 Mixed/Fallback 与未证明语义保持原状，不标为完整 Java 恢复

#### Scenario: Text emission stops

- **WHEN** 构建已产生语句但输出阶段被预算或取消终止，最终未提交产物
- **THEN** content 为 not_produced，与 Stopped 及实际 text/source map 契约一致，不报告未交付 AST 的内容

#### Scenario: Formatting and literals do not change classification

- **WHEN** 同一结构改变注释措辞、空白，或字符串字面量包含注释分隔符
- **THEN** 内容分类不因这些文本变化而改变；公开库和 JSON CLI 对同一请求报告相同分类

#### Scenario: A produced count is not a statement count

- **WHEN** 报告或文档给出某组请求的 Produced 数量或比例
- **THEN** MUST 同时给出 content 分类（或明确声明该数不含内容分类），MUST NOT 把它标注为语句恢复率、语义正确率或行为等价率；两个口径分别命名，不合并成一个数字

#### Scenario: Produced without a statement stays visible as such

- **WHEN** 一组请求的产物只有包装、理由与 BCI 引用
- **THEN** 它们计入 Produced 且 content 为 explanation_only；报告与文档 MUST 能同时读出这两个数，不得因「有产物」就把它们计成含语句

### Requirement: Driver class declarations use the already-read physical evidence

公开恢复入口 SHALL 向声明恢复提供本次 driver Header 已取得的 class internal name 和 class access flags，并绑定该读取的物理定义；MUST NOT 要求调用方再次读取或手工补充这些已有事实。事实交接 SHALL 不新增 Header/Body 扫描，也不从 entry 显示名、CP 引用或宿主 classpath 推断声明类。名称的安全显示不能改变原始身份。

#### Scenario: Ordinary and interface method declarations

- **WHEN** 公开入口分析普通类实例/static 方法及 interface default/static 方法，相关 Header 已完整读取
- **THEN** 声明记录基于实际 class/member flags 给出对应 form，不产生因交接丢失导致的 class-not-in-run 诊断；正常方法不额外读取其它 Header 或 Body

#### Scenario: Same name with different physical declarations

- **WHEN** 两个物理定义或 loader 下存在相同内部名但 class flags 不同的类
- **THEN** 每个恢复请求使用其实际绑定定义的 class facts，不因同名或相同显示文本串用声明

#### Scenario: No declaration was published

- **WHEN** 本次分析在取得可靠 driver 声明前停止，或低层恢复输入本来缺少声明类事实
- **THEN** 继续报告缺失事实与原停止语义，不通过请求参数猜测 flags 或伪造 body

#### Scenario: Declaration evidence is not a body quality claim

- **WHEN** 声明类事实交接成功，但方法内部某个结构仍无可靠恢复证据
- **THEN** 声明可正确呈现，方法继续保留原 fallback/未证明状态，不仅因声明诊断消失升级 body quality、编译或验证标志

### Requirement: Recovery presentation leads with delivered content

任务导向的恢复结果 SHALL 先表达交付内容（`contains_statements`、`explanation_only`、`not_produced`），再表达 quality，最后表达任何停止原因；上述三者 MUST 从报告的既有字段读取，MUST NOT 通过重新分析 `text`、剥离注释、统计 token 相似度或第二次构建结构来判定。呈现顺序 MUST NOT 改变任何既有契约：content 仍不证明完整恢复、可编译或语义等价，representation、quality、syntax/compile/semantic/verification、execution 与 text/source map 保持原语义，停止时继续遵循既有停止契约。

#### Scenario: Statement-bearing result leads with content

- **WHEN** 产物含实际发射的语句
- **THEN** 呈现先给出 contains_statements 再给 quality；不得把该分类读作完整恢复、验证通过或可编译（验收 A13）

#### Scenario: Explanation-only and stopped results

- **WHEN** 产物只有说明，或运行在交付产物前停止
- **THEN** 呈现分别先给 explanation_only 或 not_produced，再给 quality 与停止原因；停止时 text/source map 与 execution 保持原有契约（验收 A13、A14）

#### Scenario: Presentation does not re-analyse text

- **WHEN** 交付文本的注释措辞或字面量内容变化而结构不变，例如说明中出现 `return`
- **THEN** 呈现的 content、quality 与停止原因不变，库与 CLI 对同一请求给出相同字段（验收 A13）

### Requirement: Recovery recursion is bounded and a stop is published

恢复调用 MUST 返回 `RecoveryReport`，MUST NOT 因递归耗尽进程栈而让进程以 signal 终止。
系统 MUST 遵守本 requirement 下所有“完整契约”场景中的全部约束。

#### Scenario: 完整契约 1

- **WHEN** 系统执行恢复调用（包含递归下降）
- **THEN** 系统 MUST 满足以下全部约束：

恢复调用 MUST 返回 `RecoveryReport`，MUST NOT 因递归耗尽进程栈而让进程以 signal 终止。恢复层 SHALL 只在能证明递归会完成时继续加深；每次递归下降前 MUST 检查一个显式的深度界，界内递归照常完成，超过界 MUST 在继续加深之前结束为已发布停止：`outcome = Stopped`、非 `Complete` 的 `execution`（Partial 或 Cancelled）、以及点名停止原因与位置的诊断，全部复用既有停止词汇与平面。停止 MUST 保留本次运行的真实 usage 与停止位置，MUST NOT 提交部分构建的文本/段表（`content = not_produced`，与既有停止语义一致），MUST NOT 以空产物冒充成功，也 MUST NOT 只记录日志后返回成功。结构性平面（representation/quality/content/execution）MUST NOT 被当作本要求的证据：本要求的证据是入口返回的报告与其中的停止平面。

#### Scenario: 完整契约 2

- **WHEN** 系统执行恢复调用（包含递归下降）
- **THEN** 系统 MUST 满足以下全部约束：

本要求同样约束恢复层**自己构造的表示**：一条拼接链在输入里的 append 数 MUST NOT 变成这一层的递归深度——链的构造、发射、克隆与释放（以及任何复制该表示所在结构的路径）的递归深度 MUST NOT 随该链的 append 数增长，增加一个片段 MUST NOT 加深这些路径的嵌套。片段**内部**真正嵌套的子表达式仍按既有的值嵌套界受限，本要求不改变那条界（真正的递归下降前仍 MUST 检查它），也不得用「更大的界」或「放大线程栈」（更大的 `RUST_MIN_STACK`、大栈线程或等价手段）替代上述性质。可观察结果：链长达到复核记录的区间（debug 构建在 N = 1536/2048 以 signal 结束、N = 1024 完成）与更大规模时，公开入口与 CLI MUST 返回报告而不是 signal；链**能**呈现时 MUST 正常完成（MUST NOT 把「2048 个片段必须停止」写成契约），不能呈现时 MUST 结束为既有已发布停止；预算或取消在深链的构建/发射过程中生效时，MUST 发布既有停止形态并在释放该结构后返回，MUST NOT abort。debug 与 release 是同一份代码的两条边界证据，两个构建 MUST 都返回报告；MUST NOT 用 release 对更深输入完成代替 debug 的 signal 证据，也 MUST NOT 用任一构建的完成深度代替界的依据。


#### Scenario: A request that used to abort now answers

- **WHEN** 对固定复现（默认预算、公开入口、standalone class）发起恢复，且修正前同一输入使进程以 `stack overflow, aborting` 结束
- **THEN** 调用返回报告：`outcome` 为 Stopped、`execution` 非 Complete、诊断点名递归界与位置；CLI 以既有「执行未完整」状态（exit 4）退出且 stdout 为报告。MUST NOT 出现 signal 终止、空 stdout 或无报告的失败（验收 A13、A14）

#### Scenario: The bound is checked before the descent

- **WHEN** 下一次递归会超过显式界
- **THEN** 在该次递归开始之前结束并发布停止，位置与原因可定位；不能在递归内栈耗尽后再补救，也不能先写出部分产物再撤回

#### Scenario: Re-entering a state inside the bound

- **WHEN** 递归在界内重新进入本次运行已经访问过的状态（同一 SSA 值、同一块或同一表达式节点的重复进入），说明该递归不能自行完成
- **THEN** 结果 MUST 是已发布停止，或该层能把它局部化时按既有 refusal 语义处理：被拒区域保留 bytecode 与 origin、不得标记结构化，其余工作如实报告；两种结果都 MUST NOT 继续加深直到栈溢出，诊断 MUST 说明是重新进入而不是输入过深（验收 A13）

#### Scenario: In-bound recovery is unchanged

- **WHEN** 已提交语料与受控 fixture 的递归在界内完成
- **THEN** 结果与修正前逐字段一致（仅 elapsed_millis 可不同），不因新增界升级拒绝、降级 quality、改变 content 或改变停止/取消语义（验收 A13）

#### Scenario: The stop is the existing stop shape

- **WHEN** 一个运行因超界停止
- **THEN** 其 `execution`、诊断形态、`content = not_produced` 与 text/段表为空的性质与其他既有停止一致（仅 code 与位置不同）；不得新增 `StopReason` 变体、预算维度或报告平面

#### Scenario: A chain's length does not become this layer's depth

- **WHEN** 一条已验证的拼接链含数千个 `append` 片段
- **THEN** 构造、发射、克隆与释放的递归深度 MUST NOT 随 append 数增长（增加一个片段 MUST NOT 加深这些路径的嵌套）；片段内部真正嵌套的值仍按既有值嵌套界受限，本要求 MUST NOT 被表述为取消该界（验收 A13）

#### Scenario: A deep chain answers without a signal

- **WHEN** 链长落在复核记录的区间（N = 1536/2048；N = 1024 是修正前的完成对照）或更大，且分别在 debug 与 release 构建上恢复
- **THEN** 两个构建 MUST 都返回报告（能呈现时为产物，不能呈现时为既有已发布停止），MUST NOT 出现 signal、exit 134、空 stdout 或 `fatal runtime error: stack overflow, aborting`；MUST NOT 通过放大线程栈取得该结果（验收 A13、A14）

#### Scenario: A deep chain may complete normally

- **WHEN** 深链的每个片段都能按本要求呈现
- **THEN** 该运行 MUST 正常完成并给出链自己的文本与 content 分类，MUST NOT 被一个防御性界停止；「N 个片段必须停止」MUST NOT 被写成产品契约（验收 A13）

#### Scenario: A stop inside the deep path still answers

- **WHEN** 预算或取消在深链的构造或发射过程中生效
- **THEN** MUST 发布既有停止形态（`outcome = Stopped`、非 `Complete` 的 `execution`、点名原因与位置的诊断、`content = not_produced`），并在释放该结构后返回；MUST NOT abort、MUST NOT 提交部分文本（验收 A13、A14）

### Requirement: Boolean contexts are presented as booleans

值出现在 boolean 上下文时，呈现 MUST 按该上下文的类型定型，而不是按值在帧里的 int 形状拼写：返回类型为 `Z` 的方法的 `return` MUST 以 boolean 呈现（`return true;`/`return false;`，MUST NOT 为 `return 1;`/`return 0;`）；条件的分支测试在其操作数被证明为 boolean（返回 descriptor 为 `Z` 的调用结果，或既有 P3-R5 证据所证明的 boolean 参数、由它声明的局部、字面量与否定）时 MUST 写成真值测试（`if (flag())`、`if (arg0)`），MUST NOT 写成与 `0` 的整数比较（`if (flag() != 0)`）。
系统 MUST 遵守本 requirement 下所有“完整契约”场景中的全部约束。

#### Scenario: 完整契约 1

- **WHEN** 系统尝试呈现 boolean 返回、分支测试与其它值上下文
- **THEN** 系统 MUST 满足以下全部约束：

值出现在 boolean 上下文时，呈现 MUST 按该上下文的类型定型，而不是按值在帧里的 int 形状拼写：返回类型为 `Z` 的方法的 `return` MUST 以 boolean 呈现（`return true;`/`return false;`，MUST NOT 为 `return 1;`/`return 0;`）；条件的分支测试在其操作数被证明为 boolean（返回 descriptor 为 `Z` 的调用结果，或既有 P3-R5 证据所证明的 boolean 参数、由它声明的局部、字面量与否定）时 MUST 写成真值测试（`if (flag())`、`if (arg0)`），MUST NOT 写成与 `0` 的整数比较（`if (flag() != 0)`）。层不能证明上下文所要求的值是 boolean 时 MUST 拒绝该区域并保留 bytecode 与 origin，MUST NOT 发布另一个编译器拒绝的文本；上下文没有 boolean 要求的位置（真正 `int` 返回、两个 int 值的比较）MUST 保持原有整数形态。本要求只使用返回 descriptor、参数槽类型与既有 callee descriptor 证据，MUST NOT 由它引入通用类型系统或推断。

#### Scenario: 完整契约 2

- **WHEN** 系统尝试呈现 boolean 返回、分支测试与其它值上下文
- **THEN** 系统 MUST 满足以下全部约束：

本要求同样约束判定的**顺序与操作数范围**：某个位置是否要求 boolean MUST 在该位置的**全部**操作数被拼写之前判定，且 MUST 由该位置的形态与已证明的证据作出，而不是由操作数各自的值形态作出。值自身的类型证据（`Z` 参数槽的读取、返回 descriptor 为 `Z` 的调用结果、descriptor 为 `Z` 的字段读取、被本 run 证明 boolean 的局部的读取，以及真正 `int` 操作数的整数形状）只回答「这个值是什么」；目标位置的要求回答「这个位置要不要 boolean」；`0`/`1` 到 `false`/`true` 的拼写只是后者的**适配**，MUST NOT 反过来成为前者的证据。因此整数二元比较（`if_icmp*` 形态）的两个操作数 MUST 保持各自的整数文本：`1 == n`、`n == 1`、`0 < n`、`n > 0` 的常量操作数写成 `1`/`0`，变量操作数按自己的值写出；常量在左、常量在右、相等与大小比较一律相同，MUST NOT 出现 `true == arg0`、`false < arg0`、`true < arg0` 一类 javac 拒绝的文本。**比较的结果是 boolean，这一事实 MUST NOT 被当作它操作数的类型证据。** 操作数被**证明**为 boolean 的零值测试（`ifeq`/`ifne` 形态）仍 MUST 写成真值测试（`if (b)`、`if (!b)`、`if (flag())`），而不是 `if (b != 0)`。同一个 `iconst_1` 在二元比较里 MUST 是 `1`，在真值测试与 `Z` 返回里 MUST 是 `true`。

#### Scenario: 完整契约 3

- **WHEN** 系统尝试呈现 boolean 返回、分支测试与其它值上下文
- **THEN** 系统 MUST 满足以下全部约束：

这条修正是对本项目自己上一轮 boolean 修正（已归档的 `type-boolean-contexts`，`5a8c36a`）所引入回归的纠正：该修正之前（`fa6dc6e`）`1 == n`、`0 < n`、`1 < n` 三个形状输出正确的整数文本；本要求 MUST NOT 被表述为新发现的缺陷，MUST NOT 重开该 change 的其它结论。前一轮已经落地的 boolean 形状与 int 形态 MUST 逐字保留。


#### Scenario: A boolean return presents true or false

- **WHEN** 方法 descriptor 的返回类型是 `Z`，正文是 `if (x == 0) { return true; } return false;`（`iconst_1`/`iconst_0; ireturn`）
- **THEN** 呈现 MUST 为 `return true;`/`return false;`，MUST NOT 为 `return 1;`/`return 0;`；把正文放进方法自己的签名后 javac MUST 接受，`int cannot be converted to boolean` 不得出现

#### Scenario: A boolean call result as a condition

- **WHEN** 分支测试的操作数是返回 descriptor 为 `Z` 的调用结果（源码形状 `if (flag()) { return 1; } return 0;`）
- **THEN** 呈现 MUST 是真值测试 `if (flag())`，MUST NOT 是 `if (flag() != 0)`；javac MUST 接受该正文，`incomparable types: boolean and int` 不得出现，且执行结果与原 class 相同

#### Scenario: An unproven boolean context is refused

- **WHEN** 上下文要求 boolean，而该值的 boolean 类型没有证据（不是返回 `Z` 的调用结果、不是 descriptor 声明为 `Z` 的参数或由其声明的局部、也不是 boolean 字面量/否定）
- **THEN** 该区域 MUST 拒绝并保留 bytecode 与 origin（representation=Mixed、quality=Fallback、拒绝诊断点名相关 BCI 与物理方法），MUST NOT 以 `1`/`0` 或与 `0` 比较的形式发布；拒绝产物按既有 refusal 契约可定位

#### Scenario: An int context keeps its int shape

- **WHEN** 方法真正返回 `int`（descriptor 返回 `I`），或分支测试比较两个 int 值（`if (x != 0) { return 1; } return 0;`）
- **THEN** 呈现 MUST 保持 `return 1;`/`return 0;` 与 `if (arg0 != 0)` 的整数形态，MUST NOT 改成 `true`/`false` 或真值测试；本要求 MUST NOT 用普遍改写成 boolean 换取反例通过

#### Scenario: An integer comparison keeps its integer literals

- **WHEN** 分支测试是整数二元比较，且其中一个操作数是 `0`/`1` 字面量（源码形状 `if (1 == n)`、`if (0 < n)`、`if (1 < n)`，字节形态为 `iconst_1; iload_0; if_icmpne` 一类）
- **THEN** 呈现 MUST 是 `if (1 == arg0)`、`if (0 < arg0)`、`if (1 < arg0)` 的整数文本，MUST NOT 是 `if (true == arg0)`、`if (false < arg0)`、`if (true < arg0)`；把正文放进成员自己的签名后 javac MUST 接受

#### Scenario: Either side of the comparison keeps its own spelling

- **WHEN** 常量 `0`/`1` 出现在整数二元比较的右侧（源码形状 `if (n == 1)`、`if (n > 0)`），或该比较是相等与大小两个方向中的另一个
- **THEN** 两个操作数 MUST 各自保持整数文本（`if (arg0 == 1)`、`if (arg0 > 0)`），MUST NOT 出现 `if (arg0 == true)`、`if (arg0 > false)`；本要求 MUST NOT 只检查左操作数，MUST NOT 只以一个比较方向作为证据

#### Scenario: The same literal is spelled by its context

- **WHEN** 同一段字节里的一个 `iconst_1` 分别被二元比较、被证明 boolean 的操作数的零值测试、以及 `Z` 方法的 `return` 读取
- **THEN** 它在二元比较里 MUST 是 `1`，在真值测试与 `Z` 返回里 MUST 是 `true`（`if (arg0)` 与 `return true;`）；文本由上下文决定，MUST NOT 由字面量自身的形状或渲染顺序决定

#### Scenario: A proven boolean operand keeps the truth test

- **WHEN** 零值测试的操作数是一次返回 descriptor 为 `Z` 的调用，或一个由 descriptor 证明 boolean 的参数/局部（字节形态 `invokestatic flag()Z; ifeq` 一类）
- **THEN** 呈现 MUST 仍是非比较的真值测试（`if (flag())`、`if (!flag())`、`if (arg0)`），MUST NOT 因为本修正退回 `if (flag() != 0)`；本条与整数比较的修复 MUST 同时成立

### Requirement: A type position is spelled as a legal Java type

类型位置上的类型 MUST 以合法 Java 源类型拼写，而不是原样输出 class file 的 descriptor：数组 descriptor MUST 拼成 `元素类型[]`（原始、引用与多维形态，例如 `byte[]`、`java.lang.String[]`、`int[][]`、`java.lang.String[][]`），元素名的拼写 MUST 复用既有的对象名拼写（`L…;` 去掉包装、`/` 换成 `.`）；对象类型与基本类型的既有拼写 MUST 逐字不变。层无法把某个 descriptor 拼成合法 Java 类型时 MUST 拒绝该区域并保留 bytecode 与 origin，MUST NOT 把 descriptor 原样写进文本，也 MUST NOT 用占位类型替换一个已读到的类型。representation/quality/content/execution 等结构平面 MUST NOT 被当作本要求的证据。

#### Scenario: A primitive array in a type position

- **WHEN** 局部声明由数组参数或返回数组的调用填充，帧事实给出的 descriptor 是 `[B`（源码形状 `byte[] local = value;`）
- **THEN** 呈现 MUST 是该位置的合法 Java 数组类型（`byte[] local1 = …;`），MUST NOT 含 `[B`；产物声称 Java 时 javac MUST 接受该声明

#### Scenario: A reference array in a type position

- **WHEN** 同一位置上的 descriptor 是 `[Ljava/lang/String;`
- **THEN** 呈现 MUST 是元素名按既有对象名规则拼写后的数组类型（`java.lang.String[] local1 = …;`），MUST NOT 含 `[Ljava.lang.String;`；javac MUST 接受

#### Scenario: Multi-dimensional arrays in a type position

- **WHEN** 同一位置上的 descriptor 是 `[[I` 或 `[[Ljava/lang/String;`
- **THEN** 呈现 MUST 分别是 `int[][]` 与 `java.lang.String[][]`，MUST NOT 含 `[[I` 或 `[[Ljava.lang.String;`；javac MUST 接受

#### Scenario: A legal non-array type is unchanged

- **WHEN** 类型位置上是对象类型或基本类型（例如 `Ljava/lang/String;` → `java.lang.String`）
- **THEN** 呈现 MUST 与修正前逐字相同；本要求 MUST NOT 用普遍改写拼写换取反例通过

#### Scenario: An unspellable descriptor is refused

- **WHEN** 类型位置上的 descriptor 不能被拼成合法 Java 类型（畸形、截断或非字段 descriptor 形态）
- **THEN** 该区域 MUST 拒绝并保留 bytecode 与 origin（representation=Mixed、quality=Fallback、拒绝诊断点名相关 BCI 与物理方法），MUST NOT 原样输出该 descriptor；拒绝产物按既有 refusal 契约可定位

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

决定的证据 MUST 是一份有限、写清的清单：类文件 descriptor 的 boolean 事实（`Z` 参数槽的读取、返回 descriptor 为 `Z` 的调用结果、descriptor 为 `Z` 的字段读取）、一次对本 run **已判为 boolean 的局部**的读取（传播，MUST NOT 跟随值链：一个 store 的 own value、由多个 push 合并出的值仍不证明）、以及帧为其它类型给出的既有类型事实。`0`/`1` 字面量 MUST NOT **发起** boolean 判断（`int x = 0;` 与 `boolean c = true;` 是同一份字节），但目标已经确定为 boolean 时 MUST 按其拼写（`true`/`false`）适配。**每个**写入（不只是第一次写入）MUST 按已决定的类型检查：写入的值不能拼成该类型时即为冲突。证据的传播 MUST 有界（不随输入规模无限增长），并 MUST 接线到本 run 既有的预算与取消检查；规划 MUST NOT 成为绕过预算的无限工作量，预算或取消在规划期停止时 MUST 发布既有停止。

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

- **WHEN** 一次局部声明因未知或冲突而不能成立（今天的 `declare()` 在这种情形下返回与「没有声明到期」相同的 `Ok(None)`，调用方随后仍会写出赋值）
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

### Requirement: A presented concatenation keeps each append's own conversion

一条被判为拼接链（`StringBuilder`/`StringBuffer` 链）的呈现 MUST 逐片段保留该 `append` overload 自己的转换语义：文本 MUST 计算与该链相同的值，且每个片段的转换 MUST 发生在该片段自己的位置。
系统 MUST 遵守本 requirement 下所有“完整契约”场景中的全部约束。

#### Scenario: 完整契约 1

- **WHEN** 系统尝试呈现 StringBuilder 或 StringBuffer 的 append 拼接链
- **THEN** 系统 MUST 满足以下全部约束：

一条被判为拼接链（`StringBuilder`/`StringBuffer` 链）的呈现 MUST 逐片段保留该 `append` overload 自己的转换语义：文本 MUST 计算与该链相同的值，且每个片段的转换 MUST 发生在该片段自己的位置。链的呈现 MUST 从一个**字符串上下文**开始：需要经 `String.valueOf` 转换的片段（int/long/float/double/boolean/对象）出现在第一个字符串操作数之前时，字符串拼接 MUST 从这些片段处开始（例如在它们之前插入一个空字符串片段，或用同等语义的 `String.valueOf` 形态），MUST NOT 先把相邻的数值片段加成数值、再对求和结果做一次转换——`append(a).append(b).append("!")` MUST NOT 呈现为 `arg0 + arg1 + "!"`（输入 `(1, 2)` 时原 class 返回 `"12!"`，该文本返回 `"3!"`）。

#### Scenario: 完整契约 2

- **WHEN** 系统尝试呈现 StringBuilder 或 StringBuffer 的 append 拼接链
- **THEN** 系统 MUST 满足以下全部约束：

MUST 按 `append` 的参数 descriptor 拼写每个片段：参数类型为 `boolean` 的片段里的 `0`/`1` 字面量 MUST 呈现为 `true`/`false`，MUST NOT 呈现为 `1`/`0`（`append(true)` 的 `iconst_1` 在文本里不是数值 `1`）；`null` 片段 MUST 呈现为 `null` 字面量；对象片段 MUST 按其 overload 的转换呈现（对象经 `String.valueOf` 等价的转换）。呈现 MUST 保持每个片段的求值次数与左到右顺序（不重复调用、不移动可能抛异常的操作、不改变异常次序），每个片段 MUST 保留自己的 BCI/source-map 锚点，插入的空字符串片段 MUST 只携带由链派生的锚点而不是一条它没有执行的指令。

#### Scenario: 完整契约 3

- **WHEN** 系统尝试呈现 StringBuilder 或 StringBuffer 的 append 拼接链
- **THEN** 系统 MUST 满足以下全部约束：

片段序列 MUST NOT 被任意平衡、合并或重排：把相邻片段合并、把片段移出它自己的求值位置、或为「少一层」改变某个片段被转换的时机都会改变值或副作用，MUST NOT 出现。片段自己的表达式 MUST 保持自己的分组，MUST NOT 被拆进链的加法：链 `append(int a).append(int b)`（**两个**片段）MUST 呈现为 `"" + arg0 + arg1`（两个转换各自发生，输入 `(1, 2)` 得 `"12!"`），而链 `append(int a + int b)`（**一个**片段，其表达式本身是加法）MUST 呈现为 `"" + (arg0 + arg1)` 的形态（该加法先按数值求出再转换，同一输入得 `"3!"`）；把后者的片段拆成两个片段、或把前者两个片段合并成一次数值加法，都改变值，MUST NOT 出现。链的识别与本规则接受的 overload 集合 MUST NOT 因此改变。某个片段的转换无法用呈现表达时，该区域 MUST 按既有 refusal 契约拒绝并保留 bytecode 与 origin。representation/quality/content/execution 与「两侧都能编译」MUST NOT 被当作本要求的证据：本要求的证据是原 class 与呈现文本在同一输入集合上的执行对照。


#### Scenario: A numeric fragment keeps its conversion where the chain converts it

- **WHEN** IR 证明确认的链是 `append(int a).append(int b).append(String "!")`（源码形状 `new StringBuilder().append(a).append(b).append("!").toString()`），且该链被呈现为表达式
- **THEN** 文本 MUST 在第一个数值片段处进入字符串拼接，MUST NOT 是 `arg0 + arg1 + "!"`；输入 `(1, 2)` 时原 class 返回 `"12!"`，呈现文本 MUST NOT 返回 `"3!"`（验收 A13）

#### Scenario: A boolean fragment is spelled by its append descriptor

- **WHEN** 链的某个 `append` 的参数类型是 `boolean`，且该片段的值是 `0`/`1` 字面量
- **THEN** 文本里该片段 MUST 是 `true`/`false`，MUST NOT 是 `1`/`0`；两侧以 `true` 与 `false` 两个输入执行时返回值 MUST 相同（`"" + 1` 的 `"1"` 与 `append(true)` 的 `"true"` 的差异 MUST 被该对照捕获）

#### Scenario: Null and object fragments keep their conversion

- **WHEN** 链里有 `append((Object) null)` 或 `append(someObject)` 形态的片段
- **THEN** `null` 片段 MUST 呈现为 `null`，对象片段 MUST 按 `String.valueOf` 等价的转换呈现；两侧执行（含 `null` 输入与一个非 `null` 对象）返回值 MUST 逐项相同（验收 A13）

#### Scenario: A part that is itself an addition keeps its own grouping

- **WHEN** 链的**一个**片段的表达式本身是一次数值加法（源码形状 `new StringBuilder().append(a + b).append("!")`，两个 int 参数），或**两个**片段各自是一个数值（`append(a).append(b).append("!")`）
- **THEN** 前者 MUST 呈现为 `"" + (arg0 + arg1)` 一类保持该加法分组的形态（输入 `(1, 2)` 得 `"3!"`），后者 MUST 呈现为 `"" + arg0 + arg1`（同一输入得 `"12!"`）；MUST NOT 把前者的片段拆成两个片段、也 MUST NOT 把后者的两个片段合并成一次数值加法，两种改写都会让执行对照在至少一个输入上以值不同变红（验收 A13）

#### Scenario: The order and count of evaluations are unchanged

- **WHEN** 链的某个片段是可能抛异常或可观察的求值（调用、字段读取），且它夹在需要转换的片段之间
- **THEN** 插入的字符串上下文 MUST NOT 重复、移动或删除该片段自己的求值；两侧的调用次数、顺序与异常 MUST 逐项相同（验收 A12）

#### Scenario: Chains already in a string context are unchanged

- **WHEN** 链的每个 `append` 参数都是 `java.lang.String`（全字符串），或首个片段已经是字符串、需要转换的片段在其后（源码形状 `append("x").append(a)`）
- **THEN** 文本 MUST 与修正前逐字相同，MUST NOT 增加空字符串片段或其它装饰

#### Scenario: An unstateable conversion is refused

- **WHEN** 某个片段的 `append` overload 的转换无法用呈现表达（既有识别规则不接受的 overload，或任何拼不出同一转换的形态）
- **THEN** 该区域 MUST 按既有 refusal 契约拒绝并保留 bytecode 与 origin（representation=Mixed、quality=Fallback、拒绝诊断点名相关 BCI 与物理方法）；本要求 MUST NOT 通过扩大接受范围、猜测类型或改写转换来换取反例通过

### Requirement: Output admission preserves the actual stop

恢复输出 SHALL 保留检查与计费期间的真实停止类别及位置。取消 MUST 报告为 Cancelled；deadline 中断 MUST 保持既有 Interrupted 形态。只有实际输出额度不足时 SHALL 报告 OutputBytes 预算停止，并保留真实已写字节及适用限值。停止 MUST 丢弃未提交的正文与来源，不能交付部分产物。

#### Scenario: Cancellation during output admission

- **WHEN** 取消在最初轮询通过之后、输出额度检查或计费期间被观察到
- **THEN** 输出 SHALL 返回 Cancelled 与当前写入位置，MUST NOT 报告虚构的 OutputBytes 额度不足；正文与来源不提交

#### Scenario: Interrupted output admission

- **WHEN** deadline 在输出检查或计费期间终止该次工作
- **THEN** 输出 SHALL 保留既有中断类别、原因与位置，MUST NOT 将其重新归类为 OutputBytes 额度不足

#### Scenario: Actual output quota refusal

- **WHEN** 已写入一段正文后，下一段因实际输出额度不足而被拒
- **THEN** 输出 SHALL 报告 OutputBytes 预算停止及真实已写进度、限值与拒绝位置；未提交的正文和来源 MUST 全部丢弃

#### Scenario: Exact quota and replay remain consistent

- **WHEN** 完整输出正好适合额度，且同一已决定产物进行来源重放
- **THEN** 输出 SHALL 正常完成，正文、物理来源、额度费用与重放一致性 SHALL 保持既有契约；停止分类修复 MUST NOT 改变正常产物


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


### Requirement: Class-source snapshot proof stops before body recovery

class-source 的同快照层级 widening 证明因预算、取消或期限停止时，系统 SHALL 保留原执行原因、用量及诊断，并在方法正文恢复前交付既有 Stopped/not_produced。系统 MUST 保留本次已完成 raw analysis 与可靠前缀，MUST NOT 将停止转换为空证明后继续正文或后继投影，也不得把合法请求变成无报告的裸错误。

#### Scenario: Snapshot header budget stops the pending body

- **WHEN** 真实公共 class-source 请求已取得方法分析，但所需同快照层级证明在 Header 预算处停止
- **THEN** 该方法尚未交付的恢复产物 MUST 为 Stopped、content=not_produced、无未提交文本或段表；execution 为 Partial 且保留原 ClassHeaders 维度与诊断，raw analysis 与已发布兄弟前缀保持，类 execution 非 Complete，CLI 以既有未完整状态退出4

#### Scenario: A stopped dependency report is not absence

- **WHEN** 同快照层级证明所需依赖读取已返回非 Complete 执行报告，其中承载结束共享请求的原预算/取消/期限停止，且没有产生可用证明事实
- **THEN** 系统 MUST 将该原始报告的执行停止及诊断交给恢复与类汇总，MUST NOT 仅凭无证明事实判为普通拒绝；原预算字段仅在原报告/错误确实提供时保留，不编造 limit、consumed、requested 或位置

#### Scenario: No recovery work follows the first proof stop

- **WHEN** 前置证明在本次真实收费或检查点首次观察到执行停止
- **THEN** 原停止 SHALL 保留，之后不再启动正文、optional evidence 或后继类投影收费；为发布停止保留原有报告与可靠前缀，不重跑分析、重置预算或重试收费以改换原因

### Requirement: Class-source postbody dependency stops retain committed content

class-source 的 double-brace 依赖证明在正文已交付后因预算、取消或期限停止时，系统 SHALL 保留已提交方法正文、逐方法结果及可靠兄弟/raw/source 前缀，将原停止汇入类 execution 与 diagnostics，结束后继投影。系统 MUST NOT 因已有正文而发布类 Complete，也不得将已提交产物改成 not_produced 或清空。

#### Scenario: Header budget stops only the postbody projection

- **WHEN** 公共 class-source 请求已交付含语句的 main，随后 double-brace 依赖 Header 读取因预算拒绝，且前置证明已完成
- **THEN** main 的已提交正文、content、执行结果与 source map SHALL 保留，类 execution MUST 为 Partial、reason 保留原 ClassHeaders、diagnostics 保留原停止；类汇总不能因 main Complete 而变 Complete，CLI 退出4

#### Scenario: Later projections do not resume a stopped request

- **WHEN** double-brace 依赖证明停止且报告仍有可读正文
- **THEN** 系统 MUST 保留这些已提交结果并终止后继 anonymous-interface 等本次恢复投影，不再花费其预算或以其后续结果覆盖首次停止；结构 coverage 仍按实际已读范围报告，不伪造未完成或已完成范围

### Requirement: Dependency proof absence and execution stop remain distinct

上述两条证明消费路径 SHALL 区分完整读取后的普通保守拒绝与实际执行停止。缺失、歧义、不适用或固定算法搜索界造成的无证明事实 MUST 保持既有 refusal；原错误或依赖报告承载的预算/取消/期限共享请求停止 MUST 保留原执行语义，不从文本推断或伪造原因。content、quality、syntax、compile 和 verification MUST NOT 代替 execution。

#### Scenario: Wide proof and ordinary refusal keep their source behavior

- **WHEN** 宽预算下同快照层级/double-brace 证明完整完成，或完整读取后因缺失、歧义、不适用、无关联、固定算法界而普通拒绝
- **THEN** 系统 SHALL 保持既有合法正文或拒绝边界、物理来源及效果，不将所有无证明事实变成 Stop；普通拒绝可为 Complete，不能普遍停止以通过预算反例；value-level Unsupported/decode 不完整 SHALL 按现有隔离与汇总契约处理，不一律升级为共享请求结束

#### Scenario: Cancellation is observed during real proof work

- **WHEN** token 初始未取消，真实公共请求在上述任一路径的实际证明收费中收到取消并于检查点观察到
- **THEN** 执行 SHALL 为 Cancelled 并保留真实停止位置/诊断，按前置或后置交付边界保留对应结果，不改为 BudgetExceeded、不再继续后继恢复工作

#### Scenario: Deadline and earlier causes are not rewritten

- **WHEN** 实际证明检查点观察到期限耗尽，或在此之前已发生另一真实预算停止
- **THEN** 系统 SHALL 保留实际先发生且已观察到的停止；期限保持原 ElapsedMillis 语义，其他预算保持原维度，不将非累加维度改成输出/IR预算，也不以稍后的 poll 覆盖先前原因

### Requirement: Direct-parent dependency proof stops remain visible

class-source 的直接父类 accessor 调用与字段写入证明 SHALL 保留实际预算、取消、期限停止及原诊断，涵盖 source 直接父类关系和 owner 成员表的必需读取。停止 MUST 在正文恢复前结束后继工作，保留本次分析、真实用量及已合法交付的成员前缀，MUST NOT 降格为空证明或 Complete。完整读取后的普通拒绝和值级停止仍遵循既有隔离契约。

#### Scenario: A field dependency header stops the pending body

- **WHEN** 真实公共 class-source 在无更早停止时，直接父类字段证明取得依赖读取的原 ClassHeaders 预算停止报告
- **THEN** 恢复与类汇总 MUST 保留该原维度及全部原诊断，正文尚未提交，恢复为 Stopped/not_produced，类 execution 非 Complete，JSON CLI 退出4；不得凭当前用量编造原错误未提供的额度数字

#### Scenario: An accessor stop ends later field proof work

- **WHEN** 直接父类 accessor 证明先观察到本请求的实际停止
- **THEN** 系统 MUST 保留该原停止，不再执行字段证明、正文恢复、可选证据或后继收费投影；修复前继续发生的第二次拒绝不构成应保留的工作

#### Scenario: Required source and owner reads share the stop contract

- **WHEN** 任一直接父类消费者核对实际 source 的直接父类关系或读取 owner 的方法/字段表时发生共享请求停止
- **THEN** 原停止 SHALL 在公共报告可见，不能只保护 owner 读点而将 source 读取中断解释为普通缺证；已由本次 IR 提供的关系不要求额外读取

#### Scenario: A directly returned failure preserves its cause

- **WHEN** 实际证明检查或收费直接返回结束请求的原错误
- **THEN** 原错误具备的维度和额度数字 MUST 保留；Cancelled 不得改写为预算，非 counted 期限不得冒充 IrItems 或 OutputBytes；合法请求仍须返回带分析与恢复停止的报告

#### Scenario: Cancellation or deadline is observed inside proof work

- **WHEN** token 初始未取消，实际证明收费内发生取消并由 checkpoint 观察，或真实期限在实际证明边界到期
- **THEN** 系统 SHALL 保留实际 Cancelled 或 ElapsedMillis 停止，结束后继工作，不再次 poll/charge 重建原因，不回滚停止前已获准用量；预取消、sleep 或伪造错误不能替代该验收

#### Scenario: Publication keeps the reliable prefix and admission boundary

- **WHEN** 较早成员已合法交付，而后续直接父类证明在正文恢复前停止
- **THEN** 较早 text/source map MUST 保留，当前正文不得伪称已产出；无法合法支付当前 stopped member 发布费用时，不强迫发布该 record，类汇总仍须保存原诊断与真实 coverage 缺口

#### Scenario: Ordinary absence and value-level stops retain isolation

- **WHEN** 完整读取后 source/owner 缺失或歧义、字段/accessor 不满足原身份访问条件，或遇既有值级 Unsupported/decode 情况
- **THEN** 系统 MUST 保持既有普通拒绝、物理来源与隔离，不制造预算或取消停止，不将所有非 Complete 无差别终结为共享请求停止

#### Scenario: Wide recovery preserves field ownership and helper behavior

- **WHEN** 合法同包直接父类字段/accessor 在宽预算下完成证明
- **THEN** 原字段身份、访问约束、receiver 与 BCI 来源、显式父 owner cast 及物理 helper 调用 SHALL 保持；完整生成类族与原 Runner 的编译执行行为须一致，不以普遍拒绝或停止通过反例

### Requirement: Reference-overload proof stops before later recovery work

class-source 的引用实参重载证明 SHALL 保留其必需依赖读取与平台缺失检查中实际发生的共享请求预算、取消、期限停止，以及原诊断、用量和可用定位。停止 MUST 在正文恢复前结束后继证明、正文与可选证据工作，保留已完成 raw analysis 和合法发布的成员前缀，MUST NOT 转换为空证明、普通拒绝或由后续停止改写原因。完整读取后的普通拒绝和值级隔离仍遵循既有契约。

#### Scenario: The first overload proof stop ends later proofs

- **WHEN** 公共 class-source 的引用实参重载证明首次在实际收费或检查点观察到共享请求停止
- **THEN** 系统 MUST 保留原停止，结束后继证明、正文及可选证据收费；不得继续到另一次拒付才把请求标为未完整。尚未交付的当前正文 SHALL 为 Stopped/not_produced，text/source map 为空，已完成 analysis 不重跑、不重复计费

#### Scenario: A direct failure retains its original cause

- **WHEN** 实际证明检查或读取直接返回结束本请求的预算、取消或期限错误
- **THEN** 公共报告 MUST 保留原原因、实际维度、原有额度数字与可用位置；Cancelled 不得改写成预算，期限不得变成输出或 IR 额度停止。合法请求 SHALL 返回分析及恢复报告，不能以空证明或无报告裸错误代替

#### Scenario: A dependency stop report remains distinguishable from absence

- **WHEN** 该证明必需的 owner/source 读取或平台缺失检查返回承载共享请求停止的原执行报告
- **THEN** 原 execution、全部原诊断与真实 usage MUST 进入恢复及类汇总；无可用事实不得成为普通缺失，原报告未提供的 limit/consumed/requested/位置不得被编造。预算报告仍保留原维度

#### Scenario: Cancellation and deadline are observed within real proof work

- **WHEN** 初始未取消的 token 在实际证明收费中置位并被真实 checkpoint 观察，或请求真实期限在该证明内首次被观察到
- **THEN** 系统 SHALL 保留原 Cancelled 或期限停止，停止后无后继证明/正文/证据工作及新增收费；停止前已获准用量不退款。入口预取消、sleep 或伪造错误不能替代该验收

#### Scenario: Publication preserves the paid prefix

- **WHEN** 较早成员已合法交付，而当前重载证明在正文恢复前停止
- **THEN** 已交付正文、来源和逐方法结果 SHALL 保留；当前正文不得伪称已产出。当前 stopped record 仅在既有发布费用允许时交付，否则类汇总 MUST 保留原诊断与真实 coverage 缺口；库及 JSON CLI 表达同一停止，CLI 使用既有未完整退出码4

#### Scenario: Complete absence and isolated value failures keep their meaning

- **WHEN** 读取完整但缺失、歧义、访问/descriptor/调用形态不满足原条件，或出现既有值级 Unsupported/decode 不完整
- **THEN** 系统 MUST 保持既有普通拒绝、物理来源和隔离；不得将所有无事实或所有非 Complete 都升级为共享请求停止。真正完整无候选的平台缺失判据 SHALL 保持，不能把读不到 Header 当作平台不存在

#### Scenario: Wide proof preserves overload binding and source behavior

- **WHEN** 宽预算下原有合法引用实参重载证明完成
- **THEN** 系统 SHALL 保持原调用目标、descriptor、必要显式转换、物理方法及 BCI 来源，不增加可接受重载或放宽缺证。受控完整生成源码按原样重编后的固定输入行为 MUST 与原始类一致，不能用结构平面、单一方法片段或原始源码可编译代替生成源码验收

### Requirement: A proved static int prefix update returns its new value

系统 SHALL 恢复可证明的本类非final、非volatile静态int字段加一后直接返回新值，在完整无异常处理单块内保持同一字段身份、一次读取、一次写回及相同新值的返回。输出 MUST 保持int溢出、求值顺序与物理来源，MUST NOT 以写回后再次读取字段代替原返回值。无法证明时遵循既有拒绝契约，实际预算或取消停止保持原执行语义。

#### Scenario: A static increment returns the computed sum

- **WHEN** 方法完整地读取已证明的本类可更新静态int字段，加一后将同一新值写回并直接返回，没有其它消费者、插入效果或异常范围
- **THEN** 输出 SHALL 呈现该次前置更新的新值返回，字段读取、写入和返回分别执行一次；初值-1及7分别返回并保存0及8，初值MAX_VALUE返回并保存MIN_VALUE，不得返回旧值、常量或另一次字段读取

#### Scenario: A saved return precedes the finally effect

- **WHEN** 完整FO的 `evalThenFin` 先调用上述 `bump` 并保存返回值，随后finally把字段置为100
- **THEN** 完整恢复源码 SHALL 保持原有return/finally顺序，初值7得到8/100，初值MAX_VALUE得到MIN_VALUE/100；固定main初值0的输出为1/100。MUST NOT 通过删除辅助方法、改原Java或oracle、仅编译方法片段来通过验收

#### Scenario: Field identity and value consumers must be proved

- **WHEN** 读写字段的物理声明、owner/name/descriptor/staticness或可更新性不能闭合，或写回与返回并非同一新加法值、存在额外alias消费者/插入效果/跨块或handler范围
- **THEN** 本能力 MUST NOT 将候选呈现为已证明的前置更新；受影响范围保留可定位的bytecode与物理来源，不能只比较显示字段名、猜测目标或认领部分链。其它既有规则能正确恢复的形态继续由其原契约处理

#### Scenario: Origins retain the read and the returned write

- **WHEN** 上述静态前置更新经公共库或JSON CLI恢复
- **THEN** 正文与source map SHALL 保留同一物理方法的字段读、常量、加法、复制、写回和return锚点；复制或算术不能因被折叠而丢失来源，字段producer与写入/返回consumer身份不得串用

#### Scenario: A stop commits no partial update

- **WHEN** 候选证明或发射期间观察到本次真实预算、取消或期限停止
- **THEN** 结果 MUST 保持既有停止原因、位置与真实usage，不发布部分前置更新正文或source map；不得将停止当普通缺证后继续后继构建，也不得用入口预取消代替证明期停止验证

#### Scenario: Existing updates keep their own returned values

- **WHEN** 同时运行既有实例字段前置/后置更新、静态postfix旧值消费者及无值字段单位更新
- **THEN** 原更新的返回值、求值次数、拒绝边界及物理来源 SHALL 保持；本片不得将旧值返回改成新值，也不得把减一、窄/宽型或不满足原条件的字段纳入本能力

#### Scenario: Whole source execution proves recovery

- **WHEN** 受控javac23 --release8与真实javac8的FO分别经最终冻结CLI恢复，并将每份完整生成Java原样重编后在Java8/23运行
- **THEN** 固定main与独立driver的非零/溢出/return-before-finally结果 SHALL 与对应原class逐项相同；rebuilt classpath不得混入原class/JAR，不允许改生成文本或删方法。CLI exit0、Complete或语法/质量平面不能替代实际重编与运行证据

### Requirement: A proved conditional return and fallthrough share one finally

系统 SHALL 恢复可证明的无循环条件try，其一个正常出口先保存int返回值、另一个正常出口落穿，两个出口与唯一catch-all异常出口各执行同一条直线副作用清理。输出 MUST 保持值求得位置、每路径一次清理、原异常或清理异常的优先级与物理来源；缺证保持既有拒绝，真实执行停止保持原原因和用量。成功 MUST 以完整生成源码原样重编及隔离运行对照验收。

#### Scenario: Return and fallthrough keep separate completions

- **WHEN** 固定条件体对空数组或[2]正常落穿，对[3]在try内取得返回字段的值，并在两条出口各执行同一清理
- **THEN** 恢复 SHALL 保持落穿后返回-1及条件成立时返回已取得的3000；每次清理一次，MUST NOT 把正常落穿改为条件返回或丢失方法后继

#### Scenario: The return value is captured before cleanup

- **WHEN** 未改driver把返回字段置3000，清理中原有println的受控实现把该字段改为99
- **THEN** 原类与完整重建类 SHALL 都返回3000且字段变99、清理次数为1；MUST NOT 在清理后重新读取字段来取得返回值

#### Scenario: An exception runs cleanup once and keeps its identity

- **WHEN** null输入在受保护数组读取处抛NullPointerException，清理正常完成
- **THEN** 系统 SHALL 保持一次清理及原异常的传播，不把该路径转为返回值、不生成吞异常的catch，也不重执行抛异常的producer

#### Scenario: A cleanup exception replaces the pending completion

- **WHEN** 固定driver令原println调用抛专用清理异常，分别覆盖待返回3000与受保护NullPointerException两条路径
- **THEN** 两条路径 SHALL 都只执行一次清理并传播专用清理异常；MUST NOT 让自身handler再次捕获清理操作、重复执行清理、恢复原返回或优先抛原NullPointerException

#### Scenario: Copies and physical ownership are complete

- **WHEN** 三份清理经证明折叠为一个finally
- **THEN** 正文与来源 SHALL 保留受保护条件、值保存/返回、两条正常清理与异常保存/重抛的实际方法及BCI；没有被证明的入口、额外消费者、跨范围效果或副本差异 MUST 阻止本规则认领。其它规则已正确恢复的形态保持原契约

#### Scenario: Proof work preserves actual stops

- **WHEN** 新候选证明已开始，真实预算、取消或期限在该证明或正文提交前被观察到
- **THEN** 恢复 MUST 保留原停止原因、位置与已准许用量，不发布部分证书、ownership或未提交正文/来源，不把停止当普通缺证继续后继恢复；真实证明收费中的预算/取消验证不能由入口预取消或伪造错误代替

#### Scenario: Full source and independent driver prove recovery

- **WHEN** 真实javac8及javac23 --release8生成的独立完整类分别经最终CLI恢复
- **THEN** 每份完整report.text SHALL 原样重编，未改driver仅对rebuilt类编译后在Java8/23运行，六组固定观察逐字匹配原类oracle；classpath不得混原class/JAR，不得修生成文本或删方法，CLI exit0或结构平面不替代执行证据

#### Scenario: The parent finally work keeps its remaining gaps

- **WHEN** 本片验收独立完整类及原FY.loopless方法
- **THEN** 原完整FY与FX的所有方法 SHALL 继续如实报告并另存整类重编结果；只恢复loopless不得被标为noCall/finReturn或整个父计划已完成。既有FO、直线/条件清理及其它finally控制保持原契约，不通过删除失败方法、调整oracle或改旧计费表完成本片

### Requirement: A proved loop-local finally preserves its pending return and next iteration

系统 SHALL 恢复有完整证据的int数组循环内条件try：命中条件时先保存int返回值，未命中时继续，finally在每条离开try的路径上对同一int局部作相同更新。输出 MUST 保持求值顺序、每路径一次清理、循环更新与真实异常范围及来源；缺证保留原拒绝，实际执行停止保留原原因与用量。成功 MUST 经完整生成源码原样重编及隔离执行对照验收。

#### Scenario: A pending return retains the value before local cleanup

- **WHEN** 固定noCall形状从c=0开始，每轮x==3时取得c+7、finally令c加100
- **THEN** 输出 SHALL 让[3]返回7、[1,3]返回107、[1,2,3]返回207、[3,1]和[3,3]返回7；MUST NOT 在清理后重新计算c+7、漏掉return或继续执行后续轮次

#### Scenario: Fallthrough performs cleanup and loop update exactly once

- **WHEN** 输入为空、[1]、[1,2]或[1,2,1]，没有触发提前返回
- **THEN** 输出 SHALL 分别返回0、100、200、300，保持每个有效轮次一次清理及一次索引更新，不丢失方法最终return、不重复或提前清理

#### Scenario: Array reads keep their actual exception boundary

- **WHEN** 原方法在进入try前读取数组长度与本轮元素，null输入于循环准备处抛NullPointerException
- **THEN** 恢复 SHALL 保持读取在try外并传播原异常；MUST NOT 扩大保护范围、把数组读取移入finally或为假定的受保护异常执行额外清理

#### Scenario: Incompatible copies or value flow remain unclaimed

- **WHEN** 清理副本的局部或增量不同、待返回值有额外消费者、存在未解释的写入/入口/异常范围，或清理所读的实际值流无法完整证明
- **THEN** 本规则 MUST 不认领未证明结构，保留bytecode、拒绝原因和物理来源；同slot或显示名称相同不能代替值流证据，不能普遍放宽其它finally规则

#### Scenario: Physical origins include the lead and every completion

- **WHEN** 公共库或JSON CLI交付该已证明循环finally
- **THEN** source map SHALL 保留同一物理方法的try外数组读取、受保护条件、返回值保存与消费、三个清理副本、handler绑定/重抛以及循环更新的实际BCI；折叠副本不得丢失异常表事实或原始分析

#### Scenario: A late proof stop commits no partial ownership

- **WHEN** 候选已经开始实际证明，预算、取消或期限在证明或正文提交前首次被观察到
- **THEN** 系统 MUST 保留实际原因、位置和已批准用量，不发布部分证书、块ownership或未提交正文/来源，不将停止当普通缺证继续；预算与取消须有真实证明期检查验证

#### Scenario: Whole source and the unchanged driver prove the bounded slice

- **WHEN** 真实javac8及javac23 --release8的独立NoCallFinally完整类经最终冻结CLI恢复
- **THEN** 两份完整text SHALL 原样重编，未改driver只对rebuilt类编译后在Java8/23运行，十组固定观察逐字匹配84字节原oracle；不得混用原class/JAR、改生成文本、删方法或改oracle。运行观察不直接证明待返回之后不可观察的局部清理次数，该结论还须独立结构和值流证明

#### Scenario: The broader finally parent retains unresolved cases

- **WHEN** 本片独立完整类通过验收，并重新观察原FY.noCall
- **THEN** 原FY/FX所有方法及整类重编结果 SHALL 继续完整记录，finReturn及其它未证形状保留真实缺口，不把局部成功当父计划全部完成；既有FO、LooplessFinally与其它finally控制保持原行为和计费契约

### Requirement: A proved loop call return keeps its multi-call finally

系统 SHALL 恢复可证明的数组循环条件try：一臂保存int调用结果后返回，另一臂继续循环，两臂与catch-all重抛路径执行同一多调用清理。输出 MUST 保持受保护调用、值保存位置、每路径清理顺序及异常优先级，保留物理来源；缺证或真实停止遵循既有契约。成功 MUST 经过完整生成源码原样重编和隔离运行验收。

#### Scenario: The loop keeps normal and early return paths

- **WHEN** 固定方法对数组逐项检查，等于3时返回`c0(x)`，否则继续，finally输出`fin:`与当前元素，循环耗尽返回-1
- **THEN** 恢复 SHALL 对空数组返回-1且无输出，对[1,2]返回-1且依次清理1、2，对[3]及[1,3]返回3000且分别清理3或1、3；MUST NOT 丢失提前返回、循环后继或把每轮清理移出循环

#### Scenario: A call result is captured before cleanup

- **WHEN** try内具有may-throw效果的调用正常完成，其int结果在清理前保存并在清理后返回
- **THEN** 调用、保存与返回 SHALL 保持同一值链和原求值位置，不能重执行调用、在清理后改读另一个值或把调用移出try；固定纯c0的运行结果不能单独证明其调用次数和时点

#### Scenario: All cleanup operations keep their order

- **WHEN** 三份清理均按System.out读取、StringBuilder分配/初始化、字符串与int追加、toString、println的同一实际依赖链执行
- **THEN** 折叠后的finally SHALL 保持每份自身的对象生命周期、输入值、调用目标与求值/异常顺序；副本差异、错误未初始化alias或额外消费者不得被本规则认领，不能只比较显示调用名与opcode数量

#### Scenario: Cleanup exceptions replace a pending return or continuation

- **WHEN** 未改driver在返回路径或继续循环路径的原println调用抛专用清理异常
- **THEN** 原类与完整重建类 SHALL 都传播该异常，只清理至发生异常的那一轮，不返回3000、不继续后继轮次，也不由catch-all再次捕获清理而重复执行

#### Scenario: Exception scopes and pretry effects remain distinct

- **WHEN** 原始受保护调用有catch-all边，而数组读取在try前、三份清理及重抛均不在保护区内
- **THEN** 恢复 SHALL 保留相同异常边界与原异常保存/重抛身份，null输入在数组取长度处失败且不执行清理；不得因清理相似而扩大保护范围或把可能抛异常的try体当作不抛异常。固定driver未触达的受保护异常路径须由真实IR证明，不伪称已运行覆盖

#### Scenario: Origins cover every folded physical path

- **WHEN** 公共库或JSON CLI成功恢复固定方法
- **THEN** 正文及source map SHALL 保留同一实际方法全部物理BCI的来源，涵盖调用/保存/return、三份清理的构造器与复制、异常保存/重抛、循环尾与退出；不得通过生成一个看似相同的finally而丢弃未证明的路径

#### Scenario: Real proof stops publish no partial ownership

- **WHEN** 候选已进入实际证明工作，预算、取消或期限在证明或正文提交前被观察到
- **THEN** 系统 MUST 保留原停止原因、位置与已准许用量，不发布部分证书、ownership或未提交正文/source map，不把停止当缺证继续；入口预取消、伪造错误或扩大stack不能替代真实证明期验证

#### Scenario: Whole source execution and parent gaps stay explicit

- **WHEN** 真实javac8与javac23 --release8原独立类经最终产品CLI恢复并原样重编，未改driver仅针对rebuilt类编译后在Java8/23运行
- **THEN** 四次217B固定输出 SHALL 逐字等于原oracle，classpath不得混原class/JAR，不改生成文本或删方法；另存完整FX/FY/FO的实际源码重编及可运行结果，剩余失败不得被本片成功或结构平面掩盖，本产品自身CI须绑定实际产品提交

### Requirement: A proved loop transfer retains complete finally handler evidence

系统 SHALL 完整恢复有充分证据的int数组循环continue或break清理形：本轮正常转移与落穿均对同一int局部作同一更新，catch-all出口保存原异常、清理并重抛。输出 MUST 保持转移目标、每路径效果、真实保护范围及物理来源；不能仅因handler无正常前驱而省略未证明的异常语义。缺证保持原拒绝，实际停止保留原原因与用量，成功须经完整生成源码原样重编及隔离对照验收。

#### Scenario: Continue advances while break exits

- **WHEN** 固定两方法从c=0开始，遇x==3分别continue或break，其它轮次先c加x，离开try时均c加100
- **THEN** 输入[3,4] SHALL 分别返回204与100，[1,3,4]分别返回305与201，[3,3]分别返回200与100；MUST NOT 合并转移目标、遗漏清理或额外运行break后的轮次

#### Scenario: Ordinary iterations keep both local updates

- **WHEN** 固定方法输入为空、[1]或[1,2]且没有命中转移条件
- **THEN** 两方法 SHALL 分别返回0、101、203，并保持int溢出及原每轮本体更新与清理更新的顺序和次数；不得因去除fallback而丢失一次局部写回

#### Scenario: Protection starts after array reads

- **WHEN** 原数组长度和本轮元素读取位于try外，null在循环准备时抛出NullPointerException
- **THEN** 恢复 SHALL 保持该真实边界；不能把数组访问移入try、为try外异常执行清理，或把未发生的受保护异常声称为运行已覆盖

#### Scenario: The exceptional completion needs full evidence

- **WHEN** 系统认领catch-all保存/清理/重抛出口，或基于受保护操作不可抛异常而采用等价正常表示
- **THEN** 所有物理保护范围、入口、实际effects及异常值身份 SHALL 完整证明；有可达异常边时须保留原异常经过一次清理后重抛的语义。MUST NOT 仅凭没有正常前驱、没有运行触发或原handler只剩注释就宣称完整恢复

#### Scenario: Different effects or values remain unclaimed

- **WHEN** 副本局部/增量不同、保护范围或handler身份改变、异常保存/重载不是同值、存在额外消费者/正常handler入口/可能抛出的新操作
- **THEN** 本能力 MUST 不认领未证明部分并保留诊断、bytecode及物理来源；原named-catch与其它finally规则不得因此放宽，已有正确能力仍按其原契约处理

#### Scenario: Every physical operation keeps its origin

- **WHEN** 公共库或JSON CLI交付上述已证明方法
- **THEN** 方法 SHALL 为Java/Structured且无该handler的fallback，完整text及source map保留同一物理方法的try外读取、条件、两条正常清理、转移、handler保存/更新/重抛、循环更新与最终return的全部实际BCI；不能靠空语句或伪造来源覆盖未证明指令

#### Scenario: Real proof stops publish no ownership

- **WHEN** 候选已经开始真实证明，预算、取消或期限在证明或正文提交前被观察到
- **THEN** 系统 MUST 保留原原因、位置及已批准usage，不发布部分证书、块ownership或未提交正文/来源，不把停止降为普通缺证后继续；真实证明期预算/取消检查不能由入口预取消或伪造错误代替

#### Scenario: Complete source execution defines the bounded acceptance

- **WHEN** 两真实编译器原独立完整类经最终冻结CLI恢复，未改driver只对原样重编的完整生成类编译后在Java8/23运行
- **THEN** 固定多组continue/break/落穿/null/溢出观察 SHALL 逐字等原class oracle，rebuilt classpath不混原class/JAR，不改文本/driver/方法/oracle。原FX/FY/FO完整类与JADX对照须如实保留，运行不可观察的handler事实仍依赖独立静态证明；本片不代表父计划或任意JAR/WAR全部完成

### Requirement: Typed List foreach preserves the existing body binding

系统 SHALL 在同一物理方法的完整源码证明已确定 iterable 为精确 `List<E>`、逐元素转换与 E 完全匹配时，将受支持的 Object foreach 与正文首句元素 cast 呈现为 typed E foreach。系统 MUST 保持逐元素检查先于原目标写入和任何用户效果。默认保留已有元素合成名字、原目标声明作用域、正文绑定位置及其余语句；仅在唯一元素绑定、全部局部读写和词法可见性完整证明原目标只属于该循环时，系统 SHALL 将已有原目标名字提升为头变量，删除对应首句和无其它用途的外提空声明，不改变其余引用的变量含义。

#### Scenario: Nested declaration and assignment bindings

- **WHEN** 完整输入包含两层受证 foreach，外层正文首句给循环外空声明变量赋值，内层正文首句声明变量，二者均从唯一元素作相同精确 E 转换
- **THEN** 完整类源码 SHALL 同时呈现两个 E foreach；全部使用范围闭合时，两头分别使用已有外层与内层目标名字且不再保留中间复制或无用空声明。任一绑定缺证时仅保留该绑定的已可靠 typed 头和原声明/赋值，不为另一层的美化删掉它

#### Scenario: Empty iteration and externally observed local

- **WHEN** iterable 为空，或循环前/后可观察原赋值目标，或正文内有同名但不同作用域的变量
- **THEN** 系统 MUST 保持零次迭代、最后成功元素绑定及循环退出后的可观察值和变量解析关系；有初始值、循环外用途或无法区分的同名绑定时 MUST 保留原目标声明及正文写入，不把这些变量重新绑定为头变量。无外部用途的空声明仅能在完整作用域证明后删除

#### Scenario: Typed foreach is the only source-body projection

- **WHEN** 受支持方法的泛型参数与唯一逐元素转换已完整证明，但正文没有其它调用实参转换或包装需要改写
- **THEN** 系统 SHALL 仍完成 typed foreach，并在绑定用途也完整证明时提升已有元素名字；两者均不能依赖无关正文改写偶然触发

#### Scenario: Element conversion fails

- **WHEN** null iterable、null 元素、污染列表中的错误类型元素或 iterator/hasNext/next 抛异常
- **THEN** 完整生成 Java 的异常类型、已完成目标写入、列表状态和事件顺序 SHALL 同原 class；失败转换之前不得写原目标，正文效果不得提前执行，移除中间复制不得改变循环外可见的最后值

#### Scenario: Source or binding evidence is incomplete

- **WHEN** 泛型源码证明未完成、源码元素与物理 cast 不等、源不是受支持的直接参数、首句转换不唯一或位于其它异常范围、或原 next/cast 结果在受证绑定之外还有消费
- **THEN** 系统 MUST 保留已有可靠泛型/循环/正文表示，不猜测元素类型、不吞掉 cast、不发布部分 typed 投影。仅变量用途或可见性证明不足时 MUST 保留已证明的 typed 头和原正文绑定，不能因此降级可靠元素类型

#### Scenario: Evidence scope and transactional stopping

- **WHEN** 相同输入使用 default/all/no-rule-details，或新投影遇到预算、取消或期限停止
- **THEN** 完成产物的完整类/方法投影 SHALL 一致，原始方法恢复文本与其自身来源映射 SHALL 保留；停止 MUST 原样传播，当前事务不得发布半完成的声明删除、头变量替换或正文，方法 map 不得冒称属于投影后的类文本

#### Scenario: Visible declaration and all uses belong to the loop

- **WHEN** 元素来自唯一首句绑定，原目标只有该物理写入，全部读取都位于该循环正文，原声明在真实词法祖先中可见且没有其它同名绑定
- **THEN** 系统 SHALL 使用原目标已有名字呈现头变量，保留正文其它表达式、语句、求值次数及访问对象；有调试名时保留该名，没有调试名时保留已有确定性局部名，不猜原始源码名

#### Scenario: Sibling scopes and reused locals do not authorize a fold

- **WHEN** 相似声明位于另一个 if/else、catch/finally 或 switch 分支，局部槽位在别处复用，有额外写入、循环外读取、未闭合值合流，或存在无法完整区分的捕获/遮蔽
- **THEN** 系统 MUST 不借相同名字、槽号或指令来源把不同词法绑定合并；缺证的可选折叠保持原 typed 头与正文绑定，不删另一个分支的声明

#### Scenario: Removed bindings retain physical origins

- **WHEN** 唯一首句及其无用空声明被提升到已证明的 foreach 头
- **THEN** 投影 SHALL 保留这些被折叠节点的同一物理方法来源；原方法 text 及其 source map 仍逐字保持，只在其自身文本坐标中验证全部物理指令，不伪造投影全文映射

### Requirement: A committed iterator foreach removes only its wholly consumed ancestor declaration

系统 SHALL 在已成功恢复的 iterator foreach 完整消费某个局部变量、并且唯一无初值声明位于该循环可见的真实祖先作用域时，清除这条失用迭代器声明。系统 MUST 保持循环的全部效果、元素绑定、其他声明及后续使用关系；单个 SSA 值被消费不能代替整个局部变量失用的证据。缺证时 MUST 保留已有可靠循环和声明。

#### Scenario: Nested iterator has an unused ancestor declaration

- **WHEN** 完整方法中内层 iterator foreach 已证明，迭代器只在该循环的创建、条件与 next 访问使用，唯一祖先空声明确实只服务于该循环
- **THEN** 方法恢复文本及完整类源码 SHALL 不再包含该空声明，嵌套 foreach 及其他正文保持，零次迭代、异常、状态和事件顺序 SHALL 与原 class 一致

#### Scenario: Another value of the same local remains observable

- **WHEN** 同一个局部变量在循环外、另一个异常分支或循环前后还有读写，即使当前 iterator 值的 uses 已经闭合
- **THEN** 系统 MUST 保留该变量所需声明和所有其他读写，不把不同 SSA 值误当成不同声明，也不能为此撤回已经可靠的 foreach

#### Scenario: Slot reuse and sibling declarations remain distinct

- **WHEN** 相同槽号承载另一个已区分的局部变量，或存在同名兄弟分支声明、有初值声明、同来源锚点的其他节点或重复匹配
- **THEN** 系统 MUST 按实际变量、节点形态与词法范围区分；证据不唯一时保留相关声明，不删除另一个变量或分支的声明

#### Scenario: A speculative foreach is discarded

- **WHEN** 子循环候选成功后，其所在分支或整方法被既有恢复逻辑回滚、拒绝或替换为 bytecode quote
- **THEN** 该候选 MUST 不能授权删除最终输出中仍需保留的声明；只有最终实际存在的受证循环能够消费相应空声明

#### Scenario: Origins and output planes remain truthful

- **WHEN** 一条受证祖先空声明被清理，并以 default、all 或显式包含 SourceMap 的证据请求恢复同一输入
- **THEN** 各请求的代码决定 SHALL 一致，被消费指令来源仍可在对应循环找到；raw 方法 map SHALL 对应新 raw 方法文本并保持全部物理指令和合法 UTF8 区间，不能把旧坐标或方法 map 冒称为完整类投影 map

#### Scenario: Cleanup encounters a real execution stop

- **WHEN** 变量用途核验、最终作用域匹配或清理遇到预算、取消或期限停止
- **THEN** 系统 MUST 沿既有停止契约报告，不能转成普通缺证并继续发布半完成方法；成功发布的其他内容继续遵循既有操作停止和输出承诺

### Requirement: Proved List body calls omit redundant Object presentation wrappers

在已有可靠结构化 List 方法中，对精确 `String.equals(Object)` 和 `List.remove(Object)` 调用，若实参为物理元素转换、局部存取和来源均闭合的引用局部值，且去掉 Object 展示包装后源级绑定仍与原调用一致，系统 SHALL 在完整方法及类源码中省略该包装。实参求值次数、顺序、异常和方法选择 MUST 保持一致。remove 的实参类型 MUST NOT 用作 List 元素类型推断依据。

#### Scenario: Fixed nested String loops match their source calls

- **WHEN** 固定 JADX TestNestedLoops 的两层 String foreach、局部绑定和调用均已证明
- **THEN** 完整方法及类源码 SHALL 输出 `s1.equals(s2)` 和 `l1.remove(s2)`，继续保留 `l2.add(s1)`、两层 typed foreach 和原名；完整源码重编译后的运行结果 SHALL 与原始 class 一致

#### Scenario: Reference Integer removal keeps the Object overload

- **WHEN** 同范围内已证明的局部值类型为 Integer，物理调用为 List.remove(Object)，且已有其它事实独立支持泛型声明
- **THEN** 系统 SHALL 允许省略 Object 展示包装并保持 remove(Object) 选择；MUST NOT 将该引用实参误当 primitive int，也 MUST NOT 据此推断 List<Integer>

#### Scenario: Unknown optional proof preserves reliable recovery

- **WHEN** 可选清理缺少唯一声明、赋值、物理值链、精确调用或唯一 AST 包装证据
- **THEN** 系统 SHALL 保留该包装及已有可靠泛型、typed foreach 和原名；MUST NOT 因可选清理缺证让完整方法退回原始擦除形式

#### Scenario: Overloads and physical conversions remain observable

- **WHEN** 调用接收者具有 equals(String)/equals(Object) 重载，实参为需要保持选择的 primitive int 装箱表达式，或实参含真实 CHECKCAST
- **THEN** 系统 MUST 保留原重载和物理转换语义；真实 CHECKCAST 的错误类型异常和 null 行为 SHALL 与原 class 一致；MUST NOT 用通用 Object 包装删除规则处理这些表达式

#### Scenario: Evidence planes and stops remain intact

- **WHEN** 同一输入通过 default/all/essential evidence 请求输出，或证明及投影发生预算、期限或取消停止
- **THEN** 成功的完整源码 SHALL 一致，raw RecoveryReport 文本、原始 source map 及物理事实 SHALL 保持不变；Stop SHALL 沿已有契约传播，MUST NOT 发布半完成方法或部分投影

### Requirement: Proved indexed List bodies preserve generic parameter declarations

对已完整恢复、结果仍为原物理 void 或精确 primitive 的方法，若 List 参数的全部用途与其合法 Signature、元素转换及源级调用均已证明一致，系统 SHALL 在完整方法及类源码中恢复相应 `List<E>` 参数声明。系统 MUST 保持原方法选择、返回类型、读取次数、求值顺序和异常时序；MUST NOT 仅凭 Signature 或一个结果转换猜测安全的泛型正文。

#### Scenario: Fixed indexed String loop retains its generic signature

- **WHEN** 固定 TestNestedLoops2 的 List<String>、size/get 调用、String 元素转换和 boolean 方法体均已证明
- **THEN** 完整方法和类源码 SHALL 输出 `java.util.List<java.lang.String>` 参数，保留已恢复的外层索引循环、内层循环及返回表达式；完整源码重编译后的正常和异常结果 SHALL 与原始 class 一致

#### Scenario: Physical get conversion remains observable

- **WHEN** 已证明的 get(int) 结果通过真实元素 CHECKCAST 被使用，输入含 null、错误元素、越界或会抛错的 List 实现
- **THEN** 系统 SHALL 保留该真实转换及其原有执行位置，完整生成源码的 size/get 事件、返回值和异常 SHALL 与原始 class 一致；MUST NOT 把该转换当展示包装删除

#### Scenario: Primitive result remains exact

- **WHEN** 泛型参数声明被恢复且方法结果为原物理 boolean、int、long 或其它 primitive
- **THEN** 输出 SHALL 保留同一精确结果类型及正文；MUST NOT 因若干类型共享 JVM return opcode 而互换其源类型，也 MUST NOT 据此接纳未证明的引用、数组或类型变量结果

#### Scenario: Missing body or element proof keeps reliable erased output

- **WHEN** 缺少唯一的参数用途、get/转换关联或源节点证据，存在额外写入、别名逃逸、未支持的成员/签名、错元素转换或无法闭合的值合流
- **THEN** 系统 SHALL 保留原先可靠的循环、正文、物理转换和擦除声明，并准确报告泛型投影未证明；MUST NOT 发布部分改写或凭同名/相邻位置匹配其它值

#### Scenario: Evidence selection and execution stops preserve output contracts

- **WHEN** 同一输入以 default/all/essential 请求完整源码，或证明和提交遇到预算、取消、期限停止
- **THEN** 成功请求的代码决定 SHALL 一致，raw 方法文本及对应 source map SHALL 保持同一恢复平面的真实性；真实 Stop MUST 原样传播，不能转为普通缺证后发布半完成声明

### Requirement: Proved assignment conditions permit int widening from narrow numeric values

当条件内局部赋值的值、执行路径、目标变量及其 `int` 声明均已证明，且右值有确切的 `byte`、`short` 或 `char` Java 类型时，系统 SHALL 接纳合法扩大转换，恢复原条件位置的一次赋值。系统 MUST 保留右值的值、求值次数、异常时序及赋值后变量的读取；MUST NOT 仅因共享 JVM 栈类别而认定源类型可转换。

#### Scenario: Fixed character-producing loop is completely recovered

- **WHEN** 固定完整 `TestLoopConditionInvoke` 的字符调用、局部赋值和循环早返路径均已证明
- **THEN** 系统 SHALL 输出完整结构化的 `test(C)Z` 与可重编译的完整类，保留一次条件调用、终止字符判断、命中返回与退出后位置恢复；MUST NOT 因合法 `char` 到 `int` 赋值而撤回整个方法

#### Scenario: Dynamic input preserves values and effects

- **WHEN** 独立完整控制输入覆盖立即命中、先不匹配再命中、非零起始位置遇终止符、终止符等于查找值、数组越界及字符65535
- **THEN** 完整生成类重编译后的返回、异常类别、位置、调用次数及事件顺序 SHALL 与同一原始输入一致，右值 MUST 在原条件位置只求值一次，MUST NOT 插入冗余强制转换或提前求值

#### Scenario: Exact assignments retain their existing behavior

- **WHEN** 已证明的右值类型与目标声明完全相同
- **THEN** 系统 SHALL 保留既有赋值准入、源文本和物理来源；新增扩大转换能力 MUST NOT 改变原精确类型路径

#### Scenario: Missing type or incompatible conversion remains refused

- **WHEN** 右值类型未知、boolean 与不同类型混用、引用与基本类型混用、引用类型不匹配、发生窄化，或局部复制/使用/路径证据不完整
- **THEN** 系统 MUST 保留原拒绝与物理来源，MUST NOT 将缺证或 JVM int 栈形当成扩大转换证明，也 MUST NOT 发布部分条件赋值

#### Scenario: Evidence and stops preserve atomic output

- **WHEN** 同一输入请求 default/all/essential 证据，或赋值构建遇到预算、取消、期限停止
- **THEN** 成功请求的代码决定 SHALL 一致，raw方法与source map SHALL 对应真实物理方法和BCI；真实停止 MUST 继续经原事务传播，MUST NOT 变成缺证后发布半完成源码或映射

### Requirement: A proved entry-split integer prefix loop preserves initial and recurring updates

系统 SHALL 恢复有完整物理拓扑、SSA及精确局部类型证明的条件初始int前缀循环：入口分支选择先更新值或直接检查计数；计数非零时先更新计数、再执行同一值更新并重新检查；退出返回实际值。首片只覆盖三个不同局部绑定、无异常/调用/内存效果、单前缀更新/单计数更新/单条件/单返回闭合的形状。输出 MUST 保留初始路径、每轮次序、int溢出、退出值和物理来源，缺证保持原整方法拒绝。

#### Scenario: Skipping the first prefix does not skip recurring prefixes

- **WHEN** 值初始17、前缀加3、计数每轮减1，入口选择跳过或不跳过首次前缀，计数为0、1、2
- **THEN** 完整重建类 SHALL 分别得到跳过时17/20/23、不跳过时20/23/26，与原class相同；输出包含对应条件初始更新、循环及最终return，不能丢掉首次或重复某轮更新

#### Scenario: Integer wrapping and structural variants remain faithful

- **WHEN** 已证明相同图形的值靠近int上下界，或只改变有符号更新常量、合法局部槽/参数位置及入口分支极性
- **THEN** 恢复 SHALL 依据实际操作、槽绑定和分支sense呈现，与原class的32bit环绕结果及原始选择一致，不匹配固定类名、descriptor、BCI、槽号或+3/-1常量

#### Scenario: One physical prefix retains both displayed origins

- **WHEN** 受证前缀在条件初始位置和循环尾各展示一次
- **THEN** 两个源码span MUST 指向同一物理方法与真实前缀BCI，入口/条件/尾更新/goto/return的全部物理BCI可追；不得伪造第二物理节点或丢掉重复展示中的来源

#### Scenario: Additional edges effects or ambiguous locals preserve refusal

- **WHEN** 图存在额外入口/出口/live块、异常或调用/monitor/内存效果、别名更新、非Int或Unknown更新类型、SSA/绑定缺证、栈逃逸、jsr上下文或未accounted指令
- **THEN** 本片 SHALL 不发布该结构化候选，保留完整拒绝、原始物理事实与来源，不放开一般不可约图或输出只有一半效果的源码

#### Scenario: Budget and cancellation do not publish half a duplicated prefix

- **WHEN** 候选拓扑/SSA证明、复制前缀、任一AST输出或来源物化遭遇真实预算拒付或取消
- **THEN** 系统 MUST 原样保留停止原因和实际用量；正文构建或提交前的停止不发布正文/映射，不能回退成成功空代码、只留初始更新或仅留循环尾更新，重复展示的真实输出分别计费；正文完整提交后的可选SourceMap物化停止 SHALL 沿用既有EvidencePhase契约，保留完整正文并仅交付完整节点的可靠映射前缀，显式标记Partial/None及原Stop，不发布半个节点或伪称完整映射

#### Scenario: Complete comparison does not imply whole-family coverage

- **WHEN** 交付本片的公共库/CLI与原始/JADX/Jarde完整类对照
- **THEN** 成功源码 MUST 原样经两编译器与隔离JVM对照，保留全部失败与负向边界；原Smali/DEX、其它不可约图及整个CF08/71单元账本不得由该构造JVM正例自动标为完成

### Requirement: A proved if-arm entry transfer retains its nested-loop origin

已恢复if的非空臂若以前导唯一无条件跳转块进入紧邻的嵌套循环，且完整物理控制流和唯一归属证明该关系，系统 SHALL 在真实if源码span上保留该跳转的派生物理来源。变换 MUST 只增加真实来源，保持Java文本、方法/类身份、物理事实及既有拒绝不变，不把该要求推广为任意goto或所有结构的完整BCI映射保证。

#### Scenario: Original Raung entry edge is traceable

- **WHEN** 请求真实TestLoopRestore2原class或原JAR中 `test([I[I)V` 的来源映射，其十条物理指令包含从if臂进入循环的goto
- **THEN** 已恢复源码 SHALL 保留十条指令的可靠来源，入口goto作为if的派生origin；每个span均为有效UTF8边界并指向精确原方法/类，不制造独立Java跳转语句

#### Scenario: Incomplete or different entry proof adds no guessed origin

- **WHEN** 前导块含其它指令/效果、存在额外或异常出口、目标不是紧邻循环入口、物理身份或控制流证据不足
- **THEN** 本次规则 MUST 不附加猜测来源，保持既有源码与拒绝，不挪用其它块、循环出口或上下文clone的来源

#### Scenario: Evidence selection and real stops retain their contract

- **WHEN** 同一输入分别请求default/all证据，或证明、来源构建、后置映射物化遭遇真实预算/取消停止
- **THEN** 成功请求的Java文本与精确来源 SHALL 一致；停止 MUST 原样传播并保留实际用量，正文提交前不发布半成品，正文提交后映射停止沿用可靠完整节点前缀及显式Partial/None，不改变既有发布协议

#### Scenario: Comparison preserves the upstream evidence boundary

- **WHEN** 验收该原始Raung形态的完整原始/JADX/Jarde输出
- **THEN** 生成源码 SHALL 原样交两编译器，所有失败保留；不执行上游未提供oracle的目标方法，不以编译成功声称运行等价或JADX精确while文本通过，亦不关闭整个CF08或71/612账本

### Requirement: Proved literal boolean local flags preserve their consumers

系统 SHALL 在非参数局部标志的全部写入、合并和读取均被证明属于同一布尔用途，且存在独立布尔描述符消费时，将其 0/1 初始化与更新恢复为布尔声明和赋值，并保留相应消费语句；调试名称或整数形态本身 MUST NOT 构成布尔证明。

#### Scenario: Loop flag retains printing and labeled exit
- **WHEN** 完整输入含初始化 false、命中后置 true 并带标签跳出嵌套循环的局部标志，退出后参与布尔拼接与布尔返回，且其生产消费证明闭合
- **THEN** class/JAR 的 default/all 结果 SHALL 保留布尔赋值、带标签跳出、打印和返回；完整源码重编后，在有限原始用例上的返回、异常类别与 stdout SHALL 与原件相同

#### Scenario: Debug metadata does not supply missing proof
- **WHEN** 相同方法 Code 分别携带或不携带局部变量调试表
- **THEN** 两种输入 SHALL 获得相同的布尔类型结论；缺少代码证明时 MUST NOT 仅凭变量名称或调试描述符提升类型

#### Scenario: Mixed integer use or incomplete producer refuses promotion
- **WHEN** 存在非 0/1 或未证明写入、整数消费、歧义复用、未知合并输入，或只有整数分支测试而没有独立布尔消费
- **THEN** 系统 MUST NOT 通过本规则将该局部变量提升为布尔，也 MUST NOT 将消费语句伪报为已结构化恢复

#### Scenario: Proof stops preserve reporting and publication boundaries
- **WHEN** 布尔证明遇到真实预算耗尽或取消
- **THEN** 系统 SHALL 沿既有停止契约传播原因和实际用量，MUST NOT 发布未完成证明的声明、正文或来源映射，也 MUST NOT 把停止降为缺证成功
