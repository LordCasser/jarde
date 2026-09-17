## MODIFIED Requirements

### Requirement: Provenance and execution are explicit

系统 SHALL 为枚举、artifact-tree、Header、bytecode、解析/闭包和方法分析返回快照/entry 身份及适用的 container origin、class offset 或方法 BCI，并分别返回 coverage、execution、diagnostics 和预算消耗。非累加的高水位预算维度 SHALL 进入 limits、usage 和 BudgetExceeded 结果：`nested_depth` 表示已接受的容器嵌套深度（root container depth 为 0，直接 child 为 1），`dependency_depth` 表示已接受的依赖闭包深度；两者相互独立，任一维度不得代替或改写另一维度。任何计费维度 SHALL 同时出现在 `Limits`、`UsageSnapshot`、终止维度枚举以及库与 CLI 两个请求 schema 中，请求 schema 保持显式——缺失维度是协议错误，不静默取默认值。

#### Scenario: Cancelled enumeration

- **WHEN** ZIP 根容器已经建立，且枚举开始前或过程中收到取消
- **THEN** 返回 `Ok(EnumerationReport)`，execution 为 Cancelled，并保留已完成的可靠前缀及其 coverage，不能将其标为 Complete

#### Scenario: Partial or failed enumeration after root open

- **WHEN** ZIP 根容器已经建立，枚举过程中耗尽预算或后续 entry 结构损坏
- **THEN** 返回 `Ok(EnumerationReport)` 并保留已验证前缀；预算耗尽为 Partial 和对应 BudgetExceeded 维度，结构损坏为 Failed 和错误 diagnostic，artifact coverage 为 Partial，未完成范围明确标为 skipped

#### Scenario: Root container cannot be established

- **WHEN** 输入不是 ZIP，或 ZIP 根容器无法建立
- **THEN** 枚举可以返回 `Err`，不得伪造可枚举的根容器或部分前缀

#### Scenario: Nested depth is a high-water limit

- **WHEN** artifact-tree 已接受 depth 1 的 child，随后请求 depth 2 而 `nested_depth` limit 为 1
- **THEN** usage 的 nested depth 高水位保持 1，execution 为 Partial 且 reason 指向 `nested_depth`，depth 2 child 标为未扫描而不是累加消费或空成功

#### Scenario: Dependency depth is an independent high-water limit

- **WHEN** 同一请求已接受容器深度 2（`nested_depth` limit 为 3），随后依赖闭包扩展到深度 5 而 `dependency_depth` limit 为 4
- **THEN** execution 为 Partial 且 reason 指向 `dependency_depth`，保留已接受的容器深度与依赖深度 usage；`usage.nested_depth` 与 `usage.dependency_depth` 分别记录各自高水位，任一维度超限不改写另一维度的 usage 或判定
