## ADDED Requirements

### Requirement: Member listing binds physical method identity

类成员列举 SHALL 在一次有界 Header 读取内返回类声明事实（`this_class`、class access flags、super/interface）与成员项；每个方法项 MUST 包含 raw name、descriptor、access flags 和可直接用于方法请求的 `PhysicalMethodId`，其 owner 是该次读取的物理定义（location、class bytes、variant），MUST NOT 要求调用方另行拼装身份。字段项、类级事实与普通 resource 命中 SHALL 保留各自的 kind 与物理位置，MUST NOT 被呈现为方法。列举 MUST NOT 读取任何方法 Body，也 MUST NOT 构建 CFG、SSA、Region 或 Java AST。abstract/native 方法 SHALL 作为声明返回并标明没有 Code，MUST NOT 伪造 Body。Header 读取在成员表处停止或某成员结构损坏时 SHALL 返回可靠前缀与带物理 origin 的诊断，其他成员继续可用。

#### Scenario: A listed method is directly usable as a request target

- **WHEN** 调用方列举某类成员，并选择其中一个方法项作为后续方法请求的目标
- **THEN** 该 `PhysicalMethodId` 直接可用，请求读取的物理定义与列举读取的定义一致，且列举本身没有读取任何 Body（验收 A16）

#### Scenario: Fields, class facts and resources keep their own kind

- **WHEN** 列举范围内包含类级命中、字段命中与普通 resource
- **THEN** 每一项保留自己的种类与物理位置，不被包装成方法项，方法集合只包含真实成员表中的方法（验收 A13）

#### Scenario: Declaration without a body

- **WHEN** 类成员表中的方法为 abstract 或 native
- **THEN** 列举返回其声明、descriptor 与 flags 并标明没有 Code，不伪造空 Body（验收 A13）

#### Scenario: Damaged member does not erase a class

- **WHEN** 类可读取但某个方法的结构损坏，或 Header 读取在成员表处停止
- **THEN** 返回可靠前缀、该成员或该位置的诊断与物理 origin，其他成员继续可用；不返回完整空列表，也不把损坏成员标为正常（验收 A13、A14）

#### Scenario: Listing does not start analysis

- **WHEN** 调用方只请求类成员列举
- **THEN** 读取记录只有 Header 读取，resolver、CFG、SSA、Region 与 Java AST 的构造为零；列举不启动方法分析或恢复（验收 A16、A17）
