## ADDED Requirements

### Requirement: A proved mixed short-circuit value may initialize one Boolean local

当 Java 8 方法中的闭合无环混合 `&&`/`||` 测试图将精确的 `1/0` producer 汇入一个栈 Phi，且该 Phi 的唯一直接 use 是写入同一局部变量时，恢复结果 SHALL 在类型/声明决策阶段先证明该 local 为 Java `boolean`，然后在该写入位置发射一次惰性布尔表达式。Boolean 证据 SHALL 同时覆盖 Phi 唯一 Store、同一 SSA local/slot 的完整定义-使用链，以及所有后续 local load 均到显式 Boolean 描述符消费者；不能把 JVM int-like verifier 类型、`istore`、`1/0` producer 或名字本身当作充分类型证据。该局部的名字与声明位置 SHALL 由现有局部变量计划决定；后续对已存局部值的读取 SHALL 写局部名，不得重新发射或重复求值短路 RHS。不能证明类型、名字、作用域、唯一 use 或完整区域所有权时 MUST 原子拒绝相关闭合候选并保留其已解码来源。

#### Scenario: One stored Phi feeds two later local reads
- **WHEN** Java 8 `one(Z)Z` 编译 `boolean value = (a && b()) || c(); result = value; return value;`，BCI 21 的 `istore_1` 是栈 Phi 唯一直接 consumer，BCI 22 与 26 分别读取已写入的 local
- **THEN** 完整类 SHALL 可用 Java 8 重编；八组输入中返回值、`result`、`b()` 和 `c()` 的调用次数 SHALL 与原 class 一致，短路表达式 SHALL 只发射在一次局部声明/赋值中

#### Scenario: The local's Boolean type or lexical lifetime is unproved
- **WHEN** 现有 `decide_types` 对 Phi 给出 `BooleanEvidence::None` 且普通 Int 类型回落，或完整 local use 集合不能证明所有 load 都流向 Boolean 描述符消费者，或任一后续 local read 不能证明位于这次声明的可见范围内
- **THEN** 系统 MUST 保留整体候选的 bytecode 来源，不得猜测 local 类型、把声明移出证明范围或留下未声明的 local 名；报告 SHALL 保持非结构化降级

#### Scenario: The existing plan cannot infer Boolean from the Phi alone
- **WHEN** `-g:none` 的 `one(Z)Z` 样本中 BCI21 的 Phi store 当前会经 `boolean_proof(Definition::Phi) = None` 与 `value_type(Value::Int)` 计划为 `Type::Int`，但局部完整 use 链仅到 BCI23 `putstatic result:Z` 和 BCI27 `(Z)Z` `ireturn`
- **THEN** 实现 SHALL 在现有类型/声明决策前置加入上述局部 Boolean-use 证据，再由原 placement 决定声明/赋值；不能声称无需扩展类型决策，也不能只因 0/1 值把 local 降为 Int 后继续声称满足 Boolean-local 能力

#### Scenario: The Phi has another direct consumer or the graph has an extra owner
- **WHEN** 栈 Phi 有第二个 use、store 未唯一写入可命名 local、测试/producer/consumer 还有额外正常入口、异常边、独立效果，或 Region 所有权重叠
- **THEN** 系统 MUST 原子拒绝该候选并保留所有已解码来源；不得把局部写入作为理由放宽既有闭合图、异常边或所有权证明
