## ADDED Requirements

### Requirement: Proved direct member annotation source declaration

系统 SHALL 在完整的 Java 8 类家族中，只有当直接成员注解的词法 owner、唯一物理定义、注解种类与完整元素声明均有证据时，才将 `@interface` 声明放在 owner 的成员位置。成员名中的 `$` 自身 MUST NOT 成为词法关系证据；无法证明或请求停止时 MUST 保留可读取的物理类结果而不发布部分根级成员声明。

#### Scenario: Direct annotation member restores the original API
- **WHEN** `Holder` 与 `Holder$A` 的成员关系双方一致，`A` 是完整的直接成员注解，且元素 `float value()` 有完整默认值
- **THEN** 根级源码 SHALL 声明 `Holder.A`，独立 child 查询 SHALL 仍可见原物理 `Holder$A`；根级完整源码与原 API consumer SHALL 在 Java 8 重编并通过 `-Xverify:all`，反射默认值 SHALL 与原 class 一致

#### Scenario: Top-level defaults and dollar name remain independent
- **WHEN** 一个顶级注解带 float/int 默认值，另一个顶级注解的源码名包含 `$` 但没有成员关系行
- **THEN** 默认值 SHALL 保持可重编且反射一致；`$` 类型 MUST 仍作为顶级类型输出，不得嵌入其他 owner

#### Scenario: Unproved member annotation does not create partial source
- **WHEN** 任一 owner/child 关系行缺失、重复或冲突，child 定义有歧义，注解标志或声明/默认值属性不完整，或预算及取消使证明停止
- **THEN** 根级结果 MUST 不包含该成员注解的局部声明或已改写的类型引用；可读的物理 child 报告和拒绝/停止事实 SHALL 保留
