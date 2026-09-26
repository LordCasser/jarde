## ADDED Requirements

### Requirement: Proved member type names in local declarations

系统 SHALL 在选定物理类的成员关系、类型身份及其 Java 源路径唯一且完整时，为局部声明使用可解析的 `Outer.Member` 类型名。该源级拼写 MUST 不改变局部值的原始 JVM 类型、写入/读取关系、声明位置、物理来源或其他类型出现位置的语义；不得仅凭 `$` 字符或目标类简单名推断成员关系。

#### Scenario: Local declaration beside qualified member construction

- **WHEN** 根类与非静态成员类已证明可组成同一源码家族，局部槽的准确成员类型与已证源路径一致，右侧是已证 `outer.new Member()`
- **THEN** 完整类源码的局部声明使用与嵌套声明一致的 `Outer.Member` 类型名，原/JADX/Jarde Java 8 重编后的值、分派和效果顺序一致

#### Scenario: Declaration in a for header or hoisted region

- **WHEN** 同一准确成员类型的局部声明位于循环头或提升后的区域
- **THEN** 该声明与普通语句使用同一已证源名且来源范围仍落在正确的物理方法和 BCI

#### Scenario: Unproved or conflicting source name

- **WHEN** 源路径缺失或不唯一、选定定义有歧义、成员类型的泛型源实参未证明，或类型不是该准确物理成员
- **THEN** 系统 MUST 不发布猜测的成员类型名；已有物理文本和拒绝/降级状态保持可追踪，预算/取消不得留下部分改写

#### Scenario: Other binary dollar names

- **WHEN** 一个独立顶级类的合法名称含 `$`，或成员类的二进制名与另一个选定类型相似
- **THEN** 没有对应成员关系证明的类型声明 MUST 保持原来的准确类型，不被批量改成点分层级
