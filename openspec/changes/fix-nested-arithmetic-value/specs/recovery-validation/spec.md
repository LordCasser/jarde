## ADDED Requirements

### Requirement: Presented arithmetic is accepted by an executed comparison

算术呈现的验收 SHALL 以受控执行对照为准：同一输入集合分别运行原 class 与呈现文本（或核对拒绝边界），并把原 class 自己的答案作为基准。对照 MUST 在隔离环境中运行受控 fixture，MUST NOT 执行未受控输入；MUST NOT 用单一样本、单一输入、quality/content/execution 平面或两侧共享同一错误的比较代替。任一分歧 MUST 使验收失败并指名输入、本方法与其对应 BCI；被拒绝而不是呈现的路径 MUST 以拒绝范围、被引用 BCI 与物理方法映射作为其边界证据。

#### Scenario: A set of inputs, both sides

- **WHEN** 对照运行受控 fixture 的输入集合（含 `d = -1` 与至少一个正向输入）
- **THEN** 每个输入的返回值/可观察行为在两侧逐项比较并记录；`d = -1` 时原 class 的 `-1` 与呈现文本的 `-81` 是不可接受的分歧，必须被该对账捕获（验收 A13）

#### Scenario: The baseline driver states the original's answers

- **WHEN** 对照的基准侧运行原 class 自带的 driver
- **THEN** 基准输出被记录并与呈现侧逐项比较；基准 MUST NOT 来自呈现侧、恢复中间事实或人工复算的表达式

#### Scenario: A refused presentation passes only with its boundary

- **WHEN** 修正后该形状不再生成可执行文本，而是 Mixed/Fallback 的拒绝
- **THEN** 验收核对拒绝范围、被引用 BCI 与物理方法映射；仅「没有可执行文本」不构成通过，也没有产物可以在无证明的情况下声称结构化（验收 A13）

#### Scenario: Mutation restores the wrong presentation

- **WHEN** 变异恢复错误呈现（例如去掉外层运算的呈现）后重跑对照
- **THEN** 对照在至少一个输入上以值不同变红；变异恢复后树中不遗留调试改动

#### Scenario: Correct arithmetic stays accepted

- **WHEN** 无跨写入的嵌套算术、`p3-local-rewrite` 语料与既有 controls 在修正后运行
- **THEN** 它们保持原有的 Structured 呈现并通过既有执行对照；本修正 MUST NOT 用普遍拒绝换取反例通过（验收 A13）
