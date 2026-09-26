## ADDED Requirements

### Requirement: 成员类家族投影的物理来源

家族源码 SHALL 把根及每个嵌套成员的声明、字段、方法、构造捕获和被投影调用映射到各自的完整物理定义/成员身份与位置。被源码省略的 synthetic 构件 MUST 保留为明确的派生投影事实；两个不同定义中相同的 BCI 或成员索引 MUST NOT 合并为一个来源。根类与 child 的覆盖、停止和方法 source map SHALL 分别可检查，合成的家族文本不得把 child BCI 记成根类 BCI。

#### Scenario: 根与成员 BCI 相同

- **WHEN** 根方法和成员方法都有 BCI 0，成员方法在嵌套源码中呈现
- **THEN** 两处映射 SHALL 保持不同 `PhysicalMethodId`、定义身份与源码作用域；按 BCI 单独查找不得混淆两者

#### Scenario: 捕获字段和隐式首参被投影

- **WHEN** 完整证明允许从家族文本隐藏 synthetic 捕获字段、构造器首参及其写入
- **THEN** 原始字段、物理构造器 descriptor、写入 BCI 与源级 `Outer.this`/成员构造位置 SHALL 能由派生关系交叉追溯，而不是从报告中消失

#### Scenario: child 停止或证明拒绝

- **WHEN** child 方法恢复或家族投影停止/拒绝
- **THEN** 根与 child 已完成的物理成员报告和各自覆盖 SHALL 保留，家族文本与执行状态 MUST NOT 将缺失的 child 来源报告为根类完整来源
