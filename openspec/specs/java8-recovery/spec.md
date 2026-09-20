# java8-recovery Specification

## Purpose
在 Java 8 RuntimeProfile 下，将有可靠 IR 证据的历史与 Java 8 字节码恢复为可读 Java 结构，同时保持求值顺序、异常、副作用和原始引用事实。

## Requirements

### Requirement: Evidence-gated Java 8 recovery

系统 SHALL 仅在对应 IR、descriptor、异常/effect 和编译器模式前置条件满足时启用 Structured recovery；证据不足时 MUST 返回 representation=Mixed/Bytecode、quality=Conservative/Fallback，并说明缺失条件。值的正确性 SHALL 按生成代码实际求值的位置与顺序验证，嵌套表达式不能以其原始生产位置代替当前求值位置；fallback MUST 保留被省略表达式所依赖的可观察生产者及其物理 origin，包括字段读取、类初始化与可能抛异常的操作。

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
