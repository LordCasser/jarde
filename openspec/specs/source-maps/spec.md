# source-maps Specification

## Purpose
让 Java 输出中的每个声明、表达式和诊断都能回到原始 class、method、BCI 或 metadata 证据，并在无 debug 信息时保持稳定可复核的命名。

## Requirements

### Requirement: Deterministic names and scopes

变量和成员显示名 SHALL 优先采用可信 MethodParameters/LVT/Signature 证据，并结合 descriptor、SSA usage 和作用域校验；缺失证据时 MUST 使用稳定的 `argN`/`localN` 等名称，避免关键字和冲突。

#### Scenario: Slot reuse across ranges

- **WHEN** 同一个 local slot 在不同 BCI 区间承载不同变量
- **THEN** source map 和 Java AST 建立不同作用域/名称，不把整个 slot 合并成一个变量

#### Scenario: Invalid source identifier

- **WHEN** 原始名称是 Java 关键字、非法标识符或与同作用域名称冲突
- **THEN** formatter 产生确定性安全别名，并保留原始名称 evidence

#### Scenario: A local is defined in both arms and used after the join

- **WHEN** then/else 对同一可恢复变量赋值，合流之后使用该值
- **THEN** 声明 SHALL 位于对所有使用可见的合法作用域，不能只在 then 中声明却在 else 或合流后引用；无 debug 信息不豁免该规则，证据不足时应明确降级

### Requirement: BCI-to-source evidence

系统 SHALL 为恢复的声明、表达式、语句、异常路径和降级片段提供 origin set，能够映射到原始 BCI/attribute/CP 位置；每个位置 MUST 绑定实际物理定义及适用的完整方法身份，复用既有身份词汇，不能把 BCI 当作跨方法全局键。派生 accessor/lambda 映射 MUST 标注派生关系和覆盖范围。

#### Scenario: Accessor-derived field expression

- **WHEN** Java 输出把 accessor 调用呈现为外层字段访问
- **THEN** 映射同时包含 caller 的物理方法/调用 BCI 与 callee 的物理方法/字段 BCI，并标记派生关系；两者 BCI 相等时仍为不同位置，CP index 也按所属定义解释

#### Scenario: Partial method output

- **WHEN** 一个方法只有部分 region 成功恢复
- **THEN** 成功区域和 fallback 区域各自有映射与诊断；representation 可为 Mixed，quality 按区域独立为 Structured、Conservative 或 Fallback；只有扫描未完成时整体 coverage 才为 Partial（验收 A13）
