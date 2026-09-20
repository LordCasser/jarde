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
