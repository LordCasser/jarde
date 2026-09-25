## ADDED Requirements

### Requirement: Proven non-generic member construction retains its enclosing instance and evaluation order

在公开可访问、非泛型的目标成员类可从请求的选定环境唯一确认、目标与外层定义各自的 `InnerClasses` 关系一致、其非静态成员关系与隐式外层实例绑定均有证据时，恢复结果 SHALL 将 Java 8 的成员类创建写为限定构造 `outer.new Inner(args)`，并 SHALL 只把源级普通参数写入括号。可观察的外层求值、空值异常、普通参数及构造调用顺序 MUST 与输入字节码一致。无法证明任何必要关系或时序时，恢复 SHALL 保留保守拒绝及字节码来源，不得根据二进制名中的 `$` 或构造器首参类型猜测。

#### Scenario: A simple member construction is recovered

- **WHEN** 已选环境含唯一的 `SimpleOuter.Inner` 成员类定义，调用点以该外层实例构造 `Inner`，且其物理首参、捕获字段和调用点检查都得到证明
- **THEN** `UseInner.make` 的源码 SHALL 含 `outer.new Inner(SimpleOuter.mark("A", value))` 或同义的已绑定限定表达式，普通参数列表 SHALL 不含隐式外层实例；用原依赖类进行 Java 8 重编和验证运行所得结果 MUST 与原 class 相同

#### Scenario: Null receiver precedes nested argument effects

- **WHEN** 限定构造的普通实参包含有序的嵌套效果，或构造表达式之前另有已完成的效果
- **THEN** null 限定接收者 MUST 在普通实参效果之前抛出，先前效果 MUST 保留；非 null 路径的实参效果 MUST 仍按字节码顺序出现一次

#### Scenario: Target relation or binding is unproved

- **WHEN** 目标定义缺失、在选定环境中不唯一，或成员关系、隐式首参/捕获字段、接收者身份、空值检查位置或效果顺序任一项未获证明
- **THEN** 该构造站点 MUST 保留拒绝理由和原始来源，且不得发射一个可能绑定到错误对象或改变异常次序的限定构造表达式

#### Scenario: Evidence selection and stops do not alter the decision

- **WHEN** 相同方法分别请求必要证据、全部规则详情和 BCI 范围，或者目标读取与构造证明期间预算耗尽或被取消
- **THEN** 完整运行的 Java 正文和构造判定 SHALL 不依赖证据选择；停止的运行 MUST 报告未完成且不得发布无证明的完整构造结论
