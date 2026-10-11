## ADDED Requirements

### Requirement: 无 child report 的成员 family refusal 不阻止同类整数常量名称

当完整 Java 8 类源码请求的 root report 执行为 Complete、成员 family 为 `Absent` 或 `Refused { child: None }`，且 nested enum、nested annotation、anonymous-interface 三个 sibling family 均为 `Absent` 时，系统 SHALL 继续执行已有同类整数常量名称投影。该例外 MUST NOT 依赖或解析 `Refused.reason`。投影 MUST 继续使用当前 root 的完整物理字段候选与同轮 root method AST，并保持唯一值/名称、词法遮蔽、current-text/full-writer equality、物理 anchors、精确范围、预算和原子发布契约。

该规则 MUST 保持 `Refused { child: Some(..) }`、`RefusedPair`、所有 Prepared member-family variants 及任何非 Absent sibling family 对该投影的阻止作用。它 MUST NOT 增加 child/nested source text，亦 MUST NOT 将引用解析到 enclosing/outer/inherited class。

#### Scenario: root self row 与 direct child 同时存在

- **WHEN** frozen `TestSwitchLabels$TestCls.class` 的 `InnerClasses` 同时包含 `TestCls` 相对于 `TestSwitchLabels` 的 self row 和 direct-child `Inner` 相对于 `TestCls` 的 row，root execution Complete，member family 为无 child report 的 Refused，三个 sibling family 均 Absent，root 自身 `CONST_ABC=2748` 与 `CONST_CDE=3294` 为唯一可引用字段
- **THEN** Jarde root 完整源码 SHALL 在 `f1(int)` 中输出 `case CONST_ABC:` 与 `return CONST_CDE;`；member family仍保持 Refused且不因此声称 `Inner` 已装配。`integer_constant_projections` SHALL 对两个名称各给一个精确范围，并分别锚到对应 root 物理字段 identity/index 与 `f1(I)I` 的 switch BCI 1 / return BCI 20；物理 method recovery text仍保留数字 `case 2748` / `return 3294`

#### Scenario: 独立选中的 Inner 仅恢复本类私有常量

- **WHEN** 单 class 输入选择冻结 `TestSwitchLabels$TestCls$Inner.class`，执行完整、成员 family 无 child report 而 Refused、三个 sibling family 均 Absent，唯一可验证的本类整数常量为 `CONST_CDE_PRIVATE=3294`，外层 class 字节不在请求内容中
- **THEN** 组装源码 SHALL 输出 `case CONST_CDE_PRIVATE:` 并继续输出 `return 2748;`；派生名称 SHALL 锚到 Inner 自己的字段 identity/index 与 `f1(I)I` 的 BCI 1，物理方法正文 SHALL 仍保留数字 case/return；系统 MUST NOT 仅凭数值和 InnerClasses 信息输出外层 `CONST_ABC`

#### Scenario: 有 child report 或另一 family claim

- **WHEN** member family为 `Refused { child: Some(..) }`、`RefusedPair` 或任何 Prepared variant，或 nested enum/nested annotation/anonymous-interface任一状态不是 Absent
- **THEN** 本规则 MUST NOT 触发整数名称投影；既有类文本、child report、family状态和所有物理来源继续依照原有对应通道发布

#### Scenario: name/value与完整 writer条件仍适用

- **WHEN** 候选字段有重复名称/重复值、名称被当前方法参数或局部变量遮蔽、方法 AST不完整/非Structured Java/有fallback，或 ordinary writer无法逐字重建当前完整 root text
- **THEN** 对受影响位置 MUST 保留数字或维持既有拒绝/停止；MUST NOT 仅因 member family 是 RefusedNone而越过原有安全检查或输出无效 derived span

#### Scenario: 仅有 outer 同值字段

- **WHEN** root 没有本类唯一的完整 `static final int ConstantValue` 候选，唯一同值字段只在 enclosing/outer/inherited class 中
- **THEN** 系统 MUST NOT 使用 outer/inherited字段名，继续保留数字；本规则不改变独立的 `Inner → outer` 绑定债务
