## Purpose

为 JVM 操作数栈方法建立保真、分阶段、带 origin 和不变量的中间表示，使历史子程序、异常控制流和 category-2 值能够在有界条件下进入后续 Java 恢复。

## ADDED Requirements

### Requirement: Phase-ordered JVM IR

系统 SHALL 按 raw bytecode facts、raw CFG、dialect normalization、CanonicalCFG、Frame、stack/local SSA、type/effect 和 Region 前置关系创建 IR；每个 Pass MUST 声明输入 facts、输出 facts、失效分析、支持 dialect、scope 和 budget class。

#### Scenario: Pass dependency violation

- **WHEN** 配置尝试在 Frame facts 产生前运行依赖 Frame 的 pass
- **THEN** 启动或请求校验拒绝该顺序，并报告缺失依赖而不运行半初始化 IR

#### Scenario: Analysis invalidation

- **WHEN** 某 pass 改变 CFG 或异常边
- **THEN** 相关 dominator、liveness、SSA 分析被标记失效，后续使用必须重新计算或明确拒绝

### Requirement: JVM frame and exception semantics

Frame 分析 SHALL 处理 category-1/category-2、long/double 双槽、dup/swap、uninitializedThis、new-site、handler entry、null/数组/引用合流和 returnAddress。异常边 SHALL 保留 handler 顺序、保护区间和 throwing instruction 的 effect。

#### Scenario: Java 8 legacy finally

- **WHEN** Java 8 之前的 class 使用 jsr/ret 实现 finally
- **THEN** 系统先保留 raw returnAddress/调用上下文，再在预算内规范化或返回明确 fallback，不把线性替换当作完整语义（验收 A09）

#### Scenario: Overlapping exception regions

- **WHEN** 多个 throwing instruction 位于重叠保护区间且 handler 顺序有意义
- **THEN** CanonicalCFG 保留 instruction-level throw origin、handler order 和 effect sequence，不将异常边简化为最后一条指令

### Requirement: IR invariants and origin preservation

每一层 IR SHALL 保留 OriginSet、diagnostics 和 effect order；BytecodeIR 的原始 opcode、operand、BCI 和异常表顺序 MUST 不被覆盖。Frame/SSA 构建不得把未执行的 verifier 当作已验证。

#### Scenario: Missing debug metadata

- **WHEN** 方法没有 LVT、LineNumberTable 或 StackMapTable 的一部分
- **THEN** Frame/SSA 仍按 descriptor 和数据流分析，并独立报告 verification=NotPerformed/diagnostic（验收 A10）
