## ADDED Requirements

### Requirement: 自定义参数枚举构造的常量折叠

系统 SHALL 在枚举构造 descriptor 形如 `(Ljava/lang/String;I` 前缀 + 至多三个用户参数（int 族 / String / 对象引用）时，按既有 ctor 体与常量步骤纪律折叠常量列表：每实参为 int 族字面量（按参数类型拼写窄化）、String 字面量、静态字段引用（限定名）或 null 之一。四固定形与既有 enum 证书 SHALL 逐字不变；实参种类不符、ctor 体含额外语句或超上限 SHALL 保持逐字段呈现。

#### Scenario: 任意实参折叠

- **WHEN** `ONE((byte) 1, NumString.ONE)`、`A(Simple.A)`、`B((byte) 2, "y")` 形态的枚举以三方 Java 8 重编运行
- **THEN** 常量列表按源形态折叠（含窄化与限定名拼写），整类可编译，`java -Xverify:all` 运行与原 class 一致

#### Scenario: 基线与负例不变

- **WHEN** 输入为 N0 基线、四固定形态，或实参不符/额外语句/超限形状
- **THEN** 前两者与本变更前逐字一致；后者保持逐字段呈现与既有诊断
