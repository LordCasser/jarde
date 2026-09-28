## ADDED Requirements

### Requirement: 具名 catch 的共享字段清理与返回值保持一致

系统 SHALL 仅在具名 catch、两条正常完成、三份相同的可观察字段增量、catch-all 重抛及 handler 自保护范围均有完整物理证明时，将固定四行布局恢复为一份 `try/catch/finally`。输出 MUST 保持返回值在清理前求值、字段增量恰好一次、异常覆盖、原 Throwable 身份及全部字节码来源；证据不足时 MUST 安全拒绝。

#### Scenario: 正常返回

- **WHEN** 固定 Test7 同布局方法的 `exc(obj)` 正常返回布尔值
- **THEN** 恢复源码 SHALL 先保存该值、把 `f` 加一，再返回相同值；原 class、原源码、固定 JADX 和恢复源码的可观察行为 SHALL 一致

#### Scenario: 具名 catch 与未捕获错误

- **WHEN** 保持目标 `test` 方法物理字节码不变的探针让 `exc(obj)` 抛出 `Exception`，或固定源中让其抛出 `AssertionError`
- **THEN** 前一路径 SHALL 由具名 catch 保存 false、仅增量一次后返回 false；后一路径 MUST 穿过 catch-all、仅增量一次并传播同一 `AssertionError`，不得误吞或重复清理

#### Scenario: 完成值、清理或异常边不再同形

- **WHEN** 一份 `f++` 的接收者、字段、读写链或增量不同，返回局部的保存/重载不同，具名 catch 多入口/出口，异常表扩大到清理，或 handler 重抛不再是进入的 Throwable
- **THEN** 系统 MUST 拒绝唯一 finally 投影，保留未证字节码与异常行的物理来源

#### Scenario: 来源、预算和取消

- **WHEN** 固定方法恢复成功、证明预算耗尽或请求取消
- **THEN** 成功输出 SHALL 为每个目标 BCI 提供来源且完整类可按 Java 8 重编；停止 MUST 原子地不交付半成品源码和来源映射
