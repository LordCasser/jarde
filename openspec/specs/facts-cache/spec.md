# facts-cache Specification

## Purpose
为已证明有收益的 facts cache 或索引提供明确身份与失效边界，使缓存只加速读取而不改变结构、解析、恢复和降级结果。

## Requirements

### Requirement: Complete cache identity and invalidation

任何 cache/index entry SHALL 按所在缓存层绑定其实际依赖的语义维度：CP/Header 使用 class 内容摘要、parser/registry 版本和 parse policy；X1 使用 class/resource 摘要与 consumer/scanner schema；resolution 再加入 symbol/source context、view/domain/platform/dependency snapshot；IR/source 按需增加方法内容、analysis/recovery 版本、依赖事实、output level 与命名配置。物理 origin 在返回结果时单独绑定，MUST 不因内容缓存共享而合并；不完整结果不得覆盖完整项。输入或语义依赖改变时 MUST 失效或返回不命中。

#### Scenario: Dependency becomes available

- **WHEN** 缺失依赖导致的 negative/partial cache entry 之后补齐 Header provider
- **THEN** 旧 entry 不得覆盖新的可解析结果，系统必须重新查询或明确使用不同 key

#### Scenario: Runtime profile changes

- **WHEN** 同一物理 artifact 以另一个 RuntimeProfile 或 output level 查询
- **THEN** 不复用会改变选择/语法的旧 entry，结果携带新的 view/output key

#### Scenario: Cache key follows layer dependencies

- **WHEN** 同一 snapshot 的原始 CP/Header/X1 查询与 resolution、IR 或 source 查询分别建立缓存
- **THEN** 原始层 key 不因无关 RuntimeProfile、output level 或 recovery 变化而失效；resolution/IR/source 层按各自实际依赖加入 RuntimeView、platform、registry、IR 或 recovery 上下文

### Requirement: Cache is optional and transparent

cache/index 不得成为正确性前置条件；关闭、损坏或版本不兼容时系统 SHALL 在剩余预算内回退到同一语义的直接扫描，并报告 cache 状态。请求已取消或预算耗尽时 MUST 返回终止状态，不得重置预算重跑，也不得伪造 Complete。

#### Scenario: Corrupt cache entry

- **WHEN** cache entry 校验失败或格式版本未知
- **THEN** 丢弃该 entry 并在剩余预算内执行直接路径；相同配置的完整执行与无缓存路径语义一致，无法完整执行时返回 Partial（验收 A15）

#### Scenario: Index candidate requires verification

- **WHEN** 索引只提供潜在 candidate 而未证明 consumer/definition 关系
- **THEN** 查询仍需经过结构 consumer/解析验证，不把索引命中直接作为 XRef 事实（验收 A01）

#### Scenario: Cancelled or exhausted execution cannot reset the budget

- **WHEN** 直接路径或加速路径已经取消或耗尽预算
- **THEN** 系统返回已处理范围和终止原因；禁用或损坏缓存可在剩余预算内有界回退，取消或预算耗尽不得重置预算并自动重跑
