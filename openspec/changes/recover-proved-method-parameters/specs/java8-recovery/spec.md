## ADDED Requirements

### Requirement: 以成员参数属性保留可观测的 Java 方法参数声明

当 Java 8 方法的成员级参数属性完整声明与方法 descriptor 一一对应的合法源名及 `final` flags，且本次源码恢复可将该命名用于方法声明与所有正文引用时，类源码视图 SHALL 使用这些名字和 `final` 修饰。输出的完整类源码 MUST 可由 Java 8 重编，并在 `-parameters` 重编及反射中保持参数名称和 `isFinal` 的原有观察。方法自身的 `throws`、descriptor、物理成员身份及正文求值语义 MUST 不变。不得只替换声明文本而让正文保留另一套槽位名。

#### Scenario: 无 LVT 的完整参数属性

- **WHEN** Java 8 方法由 `-g:none -parameters` 编译，`MethodParameters` 准确列出 `paramStr` 和 `final number`，正文直接读取 `paramStr`
- **THEN** 完整源码的方法声明与正文都使用 `paramStr`，第二参数写 `final int number`；重编及 `-Xverify:all` 运行的反射结果为 `paramStr:false:number:true`

#### Scenario: 没有参数属性和 debug 名称

- **WHEN** 方法没有 `MethodParameters` 且没有可用的本地变量调试名
- **THEN** 继续使用确定性槽位名，不声称存在原始参数名或 `final` 修饰

#### Scenario: 参数属性不能闭合

- **WHEN** 成员参数属性不完整、重复、计数或参数位置与 descriptor 不一致，名字不可作为安全 Java 源名，或与当前方法的已证命名冲突
- **THEN** 不得从该属性发布局部参数名或 `final` 的半份投影；拒绝或保守回退可定位到物理成员，正文与声明的名称仍须一致

#### Scenario: 中途停止

- **WHEN** 属性读取、命名交接或源码输出遇到预算耗尽或取消
- **THEN** 现有执行状态和物理来源保持可见，不发布只有声明或只有正文使用新参数名的部分源码
