## ADDED Requirements

### Requirement: 唯一同类整数常量可作为 switch 标签及直接返回的源级别名

完整 Java 8 类源码恢复中，若一个 switch 整数 key 或该 case 的直接整数返回与本类唯一、可在该源码位置安全引用的 `static final int` 常量字段值相同，系统 SHALL 使用该字段名呈现该位置；MUST 保留字节码 key、实际返回值和物理来源，不得把推测的原始源码拼写当作字节码事实。

#### Scenario: 固定整数 switch 的两个常量名

- **WHEN** 完整类中 `LOW` 与 `HIGH` 是值分别为 2748 与 3294 的唯一可引用整数常量字段，`labelConstant(int)` 的 switch key 为 2748，命中 case 直接返回 3294
- **THEN** 恢复源码 SHALL 含 `case LOW:` 和 `return HIGH;`，且原 class、固定 JADX 和 Jarde 完整源码 MUST 均通过 Java 8 重编与 `-Xverify:all`，固定 runner 的 11 行输出 MUST 逐行相同

#### Scenario: 值或名字无法唯一绑定

- **WHEN** 同值字段有两个以上、字段不是可用的 `static final int` 常量、字段声明无法完整写出，或源级名字在使用位置发生冲突
- **THEN** 该位置 MUST 保留原有整数写法，不得选一个字段名猜测，也不得输出不能重编的 case 标签或返回表达式

#### Scenario: 只有方法体证据

- **WHEN** 请求只恢复一个方法，未提供同一物理类的完整字段声明与常量属性
- **THEN** 整数 switch key 与直接返回 MUST 保留原数值写法，不能搜索其它类的同值字段

#### Scenario: 投影被预算或取消中断

- **WHEN** 字段候选核对、源码位置检查或最终输出期间预算用尽或请求取消
- **THEN** 系统 MUST 停止或保持未投影的完整源码，不得发表只替换部分常量名的误导性类源码，已知物理字段与方法事实 MUST 继续可追溯
