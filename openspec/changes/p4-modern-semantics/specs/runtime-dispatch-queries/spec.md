## Purpose

让跨 Java 版本、模块和 loader 的查询能够表达运行时选择与开放世界不确定性，并在有限入口和局部常量证据下提供可解释的 dispatch/reflection 结果。

## ADDED Requirements

### Requirement: Runtime matrix and dispatch

系统 SHALL 根据 RuntimeProfile、LoadDomain、module policy 和 MR/layout 规则生成可复核的 RuntimeMatrix，并把 declaration resolution、possible dispatch、missing dependency、ambiguous 和 unknown-loader 状态分开。

#### Scenario: Runtime matrix selection

- **WHEN** 同一 physical snapshot 在 Java 8、11、17 profile 下查询同名定义
- **THEN** 每个 view 返回选择函数、候选 origin 和未选择物理 entries；结果不得把某一 profile 的选择当成全部版本事实（验收 A06/A07）

#### Scenario: Open-world dispatch

- **WHEN** 接口/虚方法存在外部子类、动态生成类或缺失 loader 定义
- **THEN** 结果返回 KnownCandidates 加 open-world/unknown 状态，不声称唯一 runtime target

### Requirement: Bounded semantic patterns

X3 SHALL 只对有界局部常量/值传播启用 Class.forName、Class.getMethod/getField、MethodHandles.Lookup.find*、ServiceLoader.load 等已声明模式，并返回 API overload、常量输入、传播范围、loader 假设和 rule version。

#### Scenario: Constant reflection target

- **WHEN** Class.forName 的输入是可证明的常量 string 且 propagation 未超预算
- **THEN** 结果标为 pattern_inferred_target 并附证据与假设，不改写 X1 原始调用边

#### Scenario: Dynamic reflection input

- **WHEN** 反射目标来自网络、加密、复杂拼接、JNI 或自定义 loader
- **THEN** 结果为 Unknown/Unresolved，并报告停止原因，不输出任意 confidence 数字冒充确定性
