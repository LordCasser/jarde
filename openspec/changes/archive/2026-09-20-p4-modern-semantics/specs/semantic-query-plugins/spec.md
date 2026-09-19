## Purpose

为框架资源和自定义动态模式提供受控扩展点，使规则覆盖、版本、配置和证据显式化，避免把任意字符串或配置值默认解释为精确 JVM 引用。

## ADDED Requirements

### Requirement: Versioned query plugin contract

每个 Query plugin SHALL 声明 id、rule version、输入类别、配置路径、输出 schema、evidence、coverage 和预算类别；plugin 不得执行目标代码或改变 P1/P2 的原始结构事实。

#### Scenario: Framework resource rule

- **WHEN** 启用某个明确版本的 framework resource rule 并扫描匹配配置
- **THEN** 结果携带规则版本、来源 entry、匹配范围和 coverage，并与通用 structural XRef 分开

#### Scenario: Unrecognized configuration

- **WHEN** 配置格式未注册或 plugin 前置条件不满足
- **THEN** 系统返回 NotRequested/Unsupported/Unknown 诊断，不把任意 YAML/XML/Groovy 字符串当精确类引用

### Requirement: Plugin isolation from runtime execution

plugin 执行 SHALL 在同一信任域内只读取已授权的 snapshot/resource 输入，不自动联网、启动 launcher、加载 JNI 或调用 bootstrap；未来不可信扩展必须使用独立进程/Wasm 方案而非假设 Rust trait 自带沙箱。

#### Scenario: Plugin sees bounded input

- **WHEN** plugin 请求超出资源或时间预算
- **THEN** 返回 Partial/Skipped 和已读取范围，主查询仍能完成其独立结构结果
