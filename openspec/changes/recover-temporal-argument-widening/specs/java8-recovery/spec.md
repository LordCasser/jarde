## ADDED Requirements

### Requirement: java.time 类型到 Temporal 族的实参扩宽

当调用实参的呈现类型是 java.time 具名类型（以 release-8 header 实测行集为封闭），且目标是 `java.time.temporal.Temporal`/`TemporalAccessor`（或行集声明的其它接口）时，系统 SHALL 按既有扩宽表机制呈现该调用。

#### Scenario: DateTimeFormatter 双向主锚
- **WHEN** 输入为固定 `JT`（`f.format(dt)` 与 `LocalDateTime.parse(f.format(dt), f)`，`javac --release 8`）的 class 并恢复
- **THEN** `fmt()` SHALL 完整呈现且行为与原一致（`2026/01/02 03:04 -> 3:4`）

#### Scenario: Duration/Period/ChronoUnit 锚
- **WHEN** 输入为固定 `JT.spans`
- **THEN** SHALL 完整呈现且行为一致（`287`）

#### Scenario: 直接参数与 parse 位
- **WHEN** 输入为固定 `DT`
- **THEN** SHALL 完整呈现且行为一致（`03:04/03:04`）

#### Scenario: 非行集成员仍拒与零回退
- **WHEN** 实参类型不在行集，或输入为既有四表/单边机制的 fixtures
- **THEN** 保持现行拒绝/渲染逐字节不变
