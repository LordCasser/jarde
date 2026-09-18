## Purpose

在 Java 8 RuntimeProfile 下，将有可靠 IR 证据的历史与 Java 8 字节码恢复为可读 Java 结构，同时保持求值顺序、异常、副作用和原始引用事实。

## ADDED Requirements

### Requirement: Evidence-gated Java 8 recovery

系统 SHALL 仅在对应 IR、descriptor、异常/effect 和编译器模式前置条件满足时启用 Structured recovery；证据不足时 MUST 返回 representation=Mixed/Bytecode、quality=Conservative/Fallback，并说明缺失条件。

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
- **THEN** 结果保留可靠低级结构和 origin，不能输出空 body 或成功 Structured 标志；若扫描完整，coverage 仍可为 CompleteWithinScope，只有扫描失败才为 Partial（验收 A13）
