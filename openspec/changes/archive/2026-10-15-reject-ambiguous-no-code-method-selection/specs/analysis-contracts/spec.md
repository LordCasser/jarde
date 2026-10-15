# Spec Delta

## ADDED Requirements

### Requirement: Physical method selection precedes body admission

对于物理 classfile 的 direct/prepared 方法分析和 callee 读取，系统 SHALL 在完整成员表中按 raw name 与 descriptor 确认唯一匹配，之后才判定是否存在 Code 或准入方法体。存在多个匹配时 MUST 返回既有 `classfile_method_ambiguous` 输入错误，无论成员是否有 Code、是否 abstract/native、记录顺序如何；不得选择首条或输出确定的 no-body。

#### Scenario: Duplicate with a no-Code first record

- **WHEN** 同名且同 descriptor 的完整物理成员中，一条为 abstract/native 无 Code 声明，另一条具有 Code，调用者以 direct 或 prepared 输入分析或读取 callee
- **THEN** 两种记录顺序均返回 `classfile_method_ambiguous`，不进入方法体读取或返回 no-body

#### Scenario: Both duplicate declarations share body availability

- **WHEN** 完整成员表中的同名同 descriptor 两条声明均无 Code，或均有 Code
- **THEN** direct/prepared 方法分析与 callee 读取均按歧义拒绝，abstract/native 组合与记录顺序不影响结果

#### Scenario: Unique and unrelated members retain their outcomes

- **WHEN** 请求的 raw name 与 descriptor 只有一个匹配，或没有匹配，或仅其他 descriptor/其他方法出现重复声明
- **THEN** 系统保留原有唯一有 Code、唯一无 Code、缺失结果；其他成员的歧义不得污染本次请求

### Requirement: Ambiguous selection retains header evidence without body work

在完整成员表证明选择歧义后，方法分析 SHALL 将 RawFacts 阶段报告为 Failed，错误码为 `classfile_method_ambiguous`，保留已取得的 Header 证据，后续阶段 NotPerformed、方法体 NotInspected。方法分析与 callee 读取 MUST 不为该选择准入 body、消费 Code 字节或生成 IR；prepared 输入以 preparation 完成后的新增用量衡量本次读取。

#### Scenario: Ambiguity stops before any body budget is consumed

- **WHEN** direct/prepared 输入中的重复声明被完整成员表识别为歧义
- **THEN** 方法分析的 execution 为 Failed/Error/`classfile_method_ambiguous`，RawFacts 为 Failed、后续阶段 NotPerformed、body 为 NotInspected；本次 method_bodies、code_bytes 和 ir_items 增量均为零
- **AND** callee 读取返回既有输入错误且同样不消费 body、Code 或 IR 额度

#### Scenario: Existing binding and completeness failures stay authoritative

- **WHEN** prepared 输入绑定不一致或尚未取得完整成员表
- **THEN** 系统保留既有绑定/完整性拒绝，不能以局部成员推断唯一、歧义或确定的 no-body
