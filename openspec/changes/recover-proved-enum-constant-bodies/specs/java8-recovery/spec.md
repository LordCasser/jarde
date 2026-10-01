## ADDED Requirements

### Requirement: 匿名子类构造的枚举常量折叠为常量专属体

系统 SHALL 在枚举常量经匿名子类构造（`new Sub; dup; ldc; iconst; invokespecial Sub.<init>` → `putstatic`）、该子类经家族通道准备且满足关系/委托 ctor 体/纯覆盖成员集义务时，将常量折叠为 `NAME { <覆盖方法恢复文本> }` 并从输出中抑制该子类的独立呈现。子类义务不满足、其体方法恢复非 structured、或常量体带字段构造的混合形态 SHALL 保持逐字段呈现；无匿名体路径 SHALL 逐字不变。

#### Scenario: 常量专属体折叠

- **WHEN** `enum Operation implements IOperation { PLUS { @Override public int apply(…) { … } }, MINUS { … } }` 家族三方 Java 8 重编运行
- **THEN** 常量体完整呈现、匿名子类不再独立出现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 义务不满足与混合形态保守

- **WHEN** 子类 ctor 体含额外语句、成员集含非覆盖成员、体方法恢复拒绝，或常量体带构造实参
- **THEN** 整枚举保持逐字段呈现与既有诊断

#### Scenario: 既有路径零回退

- **WHEN** 输入为无匿名体枚举（四固定形、任意实参、嵌套家族）
- **THEN** 输出与本变更前逐字一致
