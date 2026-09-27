## MODIFIED Requirements

### Requirement: Historical and Java 8 compiler patterns

系统 SHALL 覆盖已验证的 synthetic accessor、inner/local/anonymous capture、bridge、enum/enum switch、try-with-resources、default/static interface、synchronized/finally 和构造器/字段初始化模式；编译器来源未知时 MUST 使用 generic JVM semantics。对于嵌套 enum，系统 SHALL 仅在完整证明父类/enum 与 child enum 的词法成员关系、选定物理定义和 child 的完整 enum 声明组后，将 enum 声明放回其实际 owner；类名中的 `$` 本身 MUST NOT 被用作成员关系证据。无法证明任一环节、表被截断或请求停止时 MUST 拒绝整个相关子树的内联投影并保留物理类报告，不得发布部分嵌套源码。普通 enum 的接口列表仍按 class header facts 输出；该要求不把枚举常量匿名类体视为普通 enum 声明的一部分。

#### Scenario: Nested enum hierarchy is restored at its lexical owners
- **WHEN** 一个完整 Java 8 类家族包含 `Outer.Inner` 和 `Outer.Inner.Deep` 两级嵌套 enum；每级选定定义唯一，class/member facts 具有相互一致的 `InnerClasses` owner/name/flags，且每个 enum 的常量组完整获证
- **THEN** 完整源码 SHALL 在 `Outer` 与 `Inner` 的词法位置声明 `Inner`、`Deep`，对外可通过 `Outer.Inner` 与 `Outer.Inner.Deep` 引用；三方源码按 Java 8 编译后，原始 API consumer SHALL 编译并与原始 class 行为一致

#### Scenario: Dollar-named top-level class remains top-level
- **WHEN** 物理类名包含 `$`，但没有完整且一致的 InnerClasses 成员关系证明其属于当前父类型
- **THEN** 系统 MUST NOT 根据 `$` 将其改写为嵌套 enum；保持独立物理类输出或按现有策略拒绝项目级内联

#### Scenario: Incomplete or conflicting nested enum evidence is rejected atomically
- **WHEN** owner 与 child 的关系行缺失、重复、冲突，child 定义不唯一，enum 组不完整，class/member table 截断，或预算/取消使证明停止
- **THEN** 系统 MUST 拒绝该 owner 下相关 enum 子树的源投影，不得只内联部分深度或部分 enum 常量，并 SHALL 保留每个可读取物理 class 的独立报告及拒绝/停止事实

#### Scenario: Interface declaration remains attached to the enum
- **WHEN** 完整 enum class header 声明一个或多个接口，且没有涉及常量专属匿名类体
- **THEN** 源声明 SHALL 保留 `implements` 接口列表；无接口 enum SHALL 不获得推测接口，普通实现接口的 class 仍按 class kind 输出

#### Scenario: Synthetic accessor recovery
- **WHEN** accessor 的 flags、body 和调用适配满足已验证模式
- **THEN** Java 输出可显示直接字段访问，同时 X1 保留调用 accessor 和 accessor-to-field 两条原始边（验收 A12）

#### Scenario: Missing debug metadata
- **WHEN** Java 8 方法没有 LVT 或 LineNumberTable
- **THEN** 恢复使用确定性 arg/local 名称或 Conservative 输出，不因缺少 debug metadata 伪造源码作用域（验收 A10）
