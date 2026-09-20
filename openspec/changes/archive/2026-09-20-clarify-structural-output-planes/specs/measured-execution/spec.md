## ADDED Requirements

### Requirement: Counting dimensions are read within one request shape

每个计数字段 SHALL 被读作「本次请求在其声明范围与请求形状下实际做了多少工作」的代理，MUST NOT 被当作跨请求形状可比的单位。以 `archive_entries` 为例，standalone class、flat jar 的根目录枚举、WAR 中单个 container、以及 active container tree 的整树遍历具有不同 scope——实测两个 flat jar 为 28 与 29、一个大 flat jar 约 6,009、一个 WAR 为 500–1,296、另一个为 3,702，且 container tree root 会走子容器。比较行 MUST 声明请求形状、声明的 roots/profile（含 multi-release/layout 策略）与引擎版本，并明确该 scope 下这个数「数的是什么」；MUST NOT 跨形状直接比较计数，也 MUST NOT 用未声明形状的数字得出收益或回归结论。

#### Scenario: Counter rows state their shape

- **WHEN** 报告或文档并列 `archive_entries` 等计数
- **THEN** 每一行 MUST 声明请求形状、范围与声明（roots/profile），不同形状的行 MUST 标注不可直接比较；缺失这些声明的数字 MUST NOT 作为结论进入发布材料

#### Scenario: A tree traversal counts what it walked

- **WHEN** 请求从 active container tree root 出发并走到子容器
- **THEN** 报告 MUST 说明该计数包含被接受的子容器条目，与只声明单个 container 的请求并列时 MUST 标注范围不同；不得让读者按同一单位相减

#### Scenario: A counter is a work proxy, not a claim about time or memory

- **WHEN** 报告给出计数字段
- **THEN** 该数字 MUST 保持「实际工作代理」的含义，MUST NOT 被表述为墙钟占比、RSS 或收益本身；时间与内存结论仍按既有测量协议单独测量
