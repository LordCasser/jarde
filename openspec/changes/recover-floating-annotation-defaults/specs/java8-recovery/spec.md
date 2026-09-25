## ADDED Requirements

### Requirement: 忠实拼写注解成员的浮点默认值

当成员的 `AnnotationDefault` 属性声明 F/D 值且其常量池项保存可由 Java 8 注解常量表达式忠实表示的 raw bits 时，类源码 SHALL 写出相应的 `default`，重编译后的反射 raw bits MUST 与原 class 相同。此规则 MUST 不从字段 `ConstantValue` 或方法体浮点指令推断注解默认值；不能证明的 NaN sign/payload MUST 保守拒绝，而不是归一化输出。reader 的原始属性事实、class-source 的物理成员身份及预算/取消结果 SHALL 保持可追溯，不新增 class-source JSON 值树。

#### Scenario: 有限值、负零和次正规值
- **WHEN** F/D 的真实池项为有限位模式，包括负零与次正规值
- **THEN** 默认值 SHALL 用保留类型的精确 Java 8 表达式拼写，完整生成类经 javac 和反射 SHALL 逐项得到原始 raw bits

#### Scenario: 标准无穷与 canonical NaN
- **WHEN** F/D 为正负无穷或标准正 quiet NaN 的确切位模式
- **THEN** 默认值 SHALL 是 Java 8 注解默认值允许的常量表达式，重编译反射结果 SHALL 保持原始 raw bits

#### Scenario: 无法证明的 NaN 或数组后代
- **WHEN** F/D 带其他 NaN sign/payload/signaling 位模式，或数组任一后代没有忠实拼写
- **THEN** 该成员的整个 `default` MUST 不被部分输出，reader 的原始属性事实及 class-source JSON 的物理成员 SHALL 仍在，不得声称已恢复该默认值

#### Scenario: 来源请求及相邻声明
- **WHEN** 同一类请求默认/完整 evidence、输出预算或取消，或含普通整数/字符串/类/枚举/嵌套注解默认值及字段 ConstantValue
- **THEN** 成功恢复时正文 SHALL 不因 evidence 选择改变，停止仍按既有契约呈现；相邻默认值与字段声明 SHALL 保持原有语义，不增公开报告字段
