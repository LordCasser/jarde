## Why

[冻结的 Java 8 八路径对照](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-local/analysis.md)表明，同一混合短路图可先由唯一 `istore` 写入局部变量，再由后续两次 `iload` 使用。原 JVM 与 JADX 保持八条结果和 RHS 次数，当前 Jarde 因无法把 Phi 证明到局部写入而引用方法，生成类缺少返回语句。

## What Changes

- 在既有闭合混合短路图和唯一栈 Phi 证明上，增加受限的局部 `istore` 消费边界；一次写入声明或赋值该局部，后续读取沿用既有局部变量呈现。
- 冻结样本的当前类型计划会把该 Phi local 决为 `Int`：`boolean_proof(Definition::Phi)` 无证据，普通 `value_type(Value::Int)` 回落为 `Type::Int`；`-g:none` 下也没有 LVT descriptor 可用。实现必须在现有类型/声明决策前增加窄的 Boolean 证据：精确 `1/0` Phi 唯一写入、同一局部的完整 def-use 集合、且全部后续 load 都到显式 `Z` 描述符消费者（样本 BCI23 `putstatic result:Z` 与 BCI27 `(Z)Z` `ireturn`），并证明名字及声明范围。只有这组证据成立时才让现有 plan 输出 `Type::Boolean` 并发射 local Store；否则保持原子拒绝。保留 RHS 惰性、一次求值和完整来源。
- 类型、名称、作用域、Phi 唯一 use、Region 所有权或预算任一无法证明时，继续原子引用完整候选；不扩大到数组或实例字段消费者。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对闭合混合短路图仅在唯一 Phi 经一次类型与作用域可证的 boolean 局部写入时恢复；局部后续读取复用其名字且不重复求值。

## Impact

实施范围限于 `jarde-java::region` 的消费者识别，以及 `jarde-java::build` 现有类型/声明决策前置的局部 Boolean use 证明和 Store 发射路径。复用现有 `Declare`、`Assign`、`Conditional` 与 local-name AST；需要扩展既有类型决策证据，不增加公共 AST、IR 或通用推断 pass。独立基线和拒绝边界见 [三方证据](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-local/analysis.md)；调用实参消费者仍由 `recover-short-circuit-invocation-arguments` 单独约束。
