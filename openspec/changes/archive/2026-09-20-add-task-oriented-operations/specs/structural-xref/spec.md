## ADDED Requirements

### Requirement: Reference results keep derivation classes and group by owner

引用结果 SHALL 保留既有语义分离：常量池出现（未被任何 consumer 使用的 CP 常量候选）、结构/指令引用（consumer 实际使用的 use-site），以及在给出声明环境时解析到声明的引用，三者 MUST 作为不同发现分别表示，MUST NOT 压平为单一“引用”种类或互相代替。呈现 SHALL 按 owning method 组织结构引用，并保留 relation、derivation、certainty、resolution、evidence、物理位置与 BCI；类级、字段级与 resource 位置命中 SHALL 保留自身位置，MUST NOT 被归入任何方法。分组、排序与过滤 MUST NOT 改变 coverage、execution、诊断或解析状态。

#### Scenario: Constant-pool occurrence stays a candidate

- **WHEN** classfile 常量池含未被任何 Code consumer 使用的 `Methodref`
- **THEN** 结果只给出常量池候选，不被呈现为调用或结构引用，分组与排序不改变其 derivation（验收 A01、A17）

#### Scenario: Structural reference is not a resolved declaration

- **WHEN** 实际调用的 CP owner 为 Sub 而声明在 Base，且调用方未提供运行环境
- **THEN** 结果保留原始符号与结构 use-site，不出现 resolves_to；在显式环境下请求声明引用时，同一 use-site 才可解析到 Base 声明（验收 A11）

#### Scenario: Grouping keeps class-level and resource positions

- **WHEN** 结果同时包含方法体内命中、类级 metadata 命中与 resource 命中
- **THEN** 方法体内命中按 owning method 组织并保留 BCI 与物理方法身份，类级与 resource 命中保留各自位置，不被分配到任何方法（验收 A03）

#### Scenario: Partial scan is not completed by grouping

- **WHEN** 扫描因预算或取消未完成
- **THEN** 已发布分组保留真实 coverage/execution 与未扫描范围，不补造完整分组或补全未决候选（验收 A14）
