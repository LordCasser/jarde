# java8-recovery Specification

## Purpose
在 Java 8 RuntimeProfile 下，将有可靠 IR 证据的历史与 Java 8 字节码恢复为可读 Java 结构，同时保持求值顺序、异常、副作用和原始引用事实。

## Requirements

### Requirement: Evidence-gated Java 8 recovery

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

恢复 MUST 保留异常优先级、资源关闭/抑制异常、monitor、类初始化和副作用顺序；不可约 CFG、非法 Java 名称或不满足模式时 MUST 降级，不生成伪等价空代码。

#### Scenario: TWR and exceptional close

- **WHEN** 方法包含 try-with-resources 的正常和异常 close 路径
- **THEN** 输出保留 close 顺序、primary/suppressed exception 关系；无法证明时返回 representation=Mixed/Bytecode、quality=Conservative/Fallback 和诊断

#### Scenario: Unstructured control flow

- **WHEN** 方法的区域恢复遇到不可约 CFG 或交叉异常区域
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

恢复报告 SHALL 返回 `content`，取值为 `not_produced`、`explanation_only`、`contains_statements`。该字段 SHALL 描述最终交付产物：Stopped 为 not_produced；Produced 且只有包装、理由、BCI 引用等说明为 explanation_only；Produced 且含至少一个实际发射的 Java 语句为 contains_statements。Produced MUST 仅表示产物已交付，不能解释为存在语句、语义完整、可编译或已验证。

声明、赋值、调用、return 和控制流语句均可构成 contains_statements；该分类不改变 representation、quality、syntax、coverage、execution 或验证状态。分类 MUST 不依赖注释剥离、token 相似度或调用方二次解析文本。库与 CLI SHALL 返回同一分类。

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

恢复调用 MUST 返回 `RecoveryReport`，MUST NOT 因递归耗尽进程栈而让进程以 signal 终止。恢复层 SHALL 只在能证明递归会完成时继续加深；每次递归下降前 MUST 检查一个显式的深度界，界内递归照常完成，超过界 MUST 在继续加深之前结束为已发布停止：`outcome = Stopped`、非 `Complete` 的 `execution`（Partial 或 Cancelled）、以及点名停止原因与位置的诊断，全部复用既有停止词汇与平面。停止 MUST 保留本次运行的真实 usage 与停止位置，MUST NOT 提交部分构建的文本/段表（`content = not_produced`，与既有停止语义一致），MUST NOT 以空产物冒充成功，也 MUST NOT 只记录日志后返回成功。结构性平面（representation/quality/content/execution）MUST NOT 被当作本要求的证据：本要求的证据是入口返回的报告与其中的停止平面。

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

### Requirement: Boolean contexts are presented as booleans

值出现在 boolean 上下文时，呈现 MUST 按该上下文的类型定型，而不是按值在帧里的 int 形状拼写：返回类型为 `Z` 的方法的 `return` MUST 以 boolean 呈现（`return true;`/`return false;`，MUST NOT 为 `return 1;`/`return 0;`）；条件的分支测试在其操作数被证明为 boolean（返回 descriptor 为 `Z` 的调用结果，或既有 P3-R5 证据所证明的 boolean 参数、由它声明的局部、字面量与否定）时 MUST 写成真值测试（`if (flag())`、`if (arg0)`），MUST NOT 写成与 `0` 的整数比较（`if (flag() != 0)`）。层不能证明上下文所要求的值是 boolean 时 MUST 拒绝该区域并保留 bytecode 与 origin，MUST NOT 发布另一个编译器拒绝的文本；上下文没有 boolean 要求的位置（真正 `int` 返回、两个 int 值的比较）MUST 保持原有整数形态。本要求只使用返回 descriptor、参数槽类型与既有 callee descriptor 证据，MUST NOT 由它引入通用类型系统或推断。

本要求同样约束判定的**顺序与操作数范围**：某个位置是否要求 boolean MUST 在该位置的**全部**操作数被拼写之前判定，且 MUST 由该位置的形态与已证明的证据作出，而不是由操作数各自的值形态作出。值自身的类型证据（`Z` 参数槽的读取、返回 descriptor 为 `Z` 的调用结果、descriptor 为 `Z` 的字段读取、被本 run 证明 boolean 的局部的读取，以及真正 `int` 操作数的整数形状）只回答「这个值是什么」；目标位置的要求回答「这个位置要不要 boolean」；`0`/`1` 到 `false`/`true` 的拼写只是后者的**适配**，MUST NOT 反过来成为前者的证据。因此整数二元比较（`if_icmp*` 形态）的两个操作数 MUST 保持各自的整数文本：`1 == n`、`n == 1`、`0 < n`、`n > 0` 的常量操作数写成 `1`/`0`，变量操作数按自己的值写出；常量在左、常量在右、相等与大小比较一律相同，MUST NOT 出现 `true == arg0`、`false < arg0`、`true < arg0` 一类 javac 拒绝的文本。**比较的结果是 boolean，这一事实 MUST NOT 被当作它操作数的类型证据。** 操作数被**证明**为 boolean 的零值测试（`ifeq`/`ifne` 形态）仍 MUST 写成真值测试（`if (b)`、`if (!b)`、`if (flag())`），而不是 `if (b != 0)`。同一个 `iconst_1` 在二元比较里 MUST 是 `1`，在真值测试与 `Z` 返回里 MUST 是 `true`。

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

一个局部变量（既有 `LocalVariable` 身份）的类型 MUST 在**语句被构建之前**、在既有的声明规划里决定一次，形成一个按该身份索引的类型结果，并由提升声明、就地声明、赋值、条件与返回共同消费；这些位置 MUST NOT 各自再作一次类型判定。同一个值在两条声明路径上 MUST 得到同一个类型：写在包含全部使用的区域起点上的**提升**声明与写在首个写入处的**就地**声明 MUST 一致，结论 MUST NOT 取决于遍历顺序（同一变量的两个分支先后不得改变其类型）。

决定的证据 MUST 是一份有限、写清的清单：类文件 descriptor 的 boolean 事实（`Z` 参数槽的读取、返回 descriptor 为 `Z` 的调用结果、descriptor 为 `Z` 的字段读取）、一次对本 run **已判为 boolean 的局部**的读取（传播，MUST NOT 跟随值链：一个 store 的 own value、由多个 push 合并出的值仍不证明）、以及帧为其它类型给出的既有类型事实。`0`/`1` 字面量 MUST NOT **发起** boolean 判断（`int x = 0;` 与 `boolean c = true;` 是同一份字节），但目标已经确定为 boolean 时 MUST 按其拼写（`true`/`false`）适配。**每个**写入（不只是第一次写入）MUST 按已决定的类型检查：写入的值不能拼成该类型时即为冲突。证据的传播 MUST 有界（不随输入规模无限增长），并 MUST 接线到本 run 既有的预算与取消检查；规划 MUST NOT 成为绕过预算的无限工作量，预算或取消在规划期停止时 MUST 发布既有停止。

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
