## Purpose

让按需分析能够在明确的 RuntimeView、LoadDomain 和平台定义范围内解析符号、继承和可能派发，同时把缺失、歧义和开放世界限制作为结果的一部分。

## ADDED Requirements

### Requirement: Demand-bound symbol resolution

系统 SHALL 绑定每次解析到 snapshot、RuntimeView、LoadDomain 和依赖 provider，并区分 Resolved、Missing、Ambiguous、Inaccessible、IncompatibleClassChange、UnsupportedPolicy、BudgetExceeded。缺失定义 MUST 保留原始 SymbolRef，不得伪造空成员。

#### Scenario: Missing platform definition

- **WHEN** 方法引用依赖未提供的 Java 平台 Header
- **THEN** 结果保留 descriptor、来源和 MissingDependency 证据，不返回虚构的 resolved member

#### Scenario: Explicit loader order

- **WHEN** 多个 loader/module root 提供同名定义
- **THEN** 解析结果携带候选 origin、loader/order 条件和 Ambiguous/OpenWorld 状态，不静默覆盖定义

#### Scenario: Inherited declaration owner

- **WHEN** 调用点常量池 owner 为 Sub，但查询目标是在 Base 声明的 `foo`
- **THEN** 解析按预算扩展候选并返回 Base declaration 与 Sub use-site 的关系，保留完整 symbol evidence，不把两者混为同一 symbol（验收 A11）

### Requirement: Resolution and dispatch are separate

系统 SHALL 先解析声明，再在有界候选集合中计算可能 dispatch；仅一个已知候选不得在缺失依赖、外部子类或自定义 loader 时被标为唯一运行时目标。

#### Scenario: Interface default conflict

- **WHEN** 接口调用在当前范围存在多个可能实现或 default conflict
- **THEN** 结果返回已知候选和冲突/开放世界诊断，并保留原始 invocation kind（验收 A11）

#### Scenario: Bounded closure expansion

- **WHEN** 单方法请求需要父类/接口 Header
- **THEN** 系统仅扩展记录的最小依赖闭包；超过类数、深度、字节或时间预算时返回 BudgetExceeded/Partial（验收 A14、A16）
