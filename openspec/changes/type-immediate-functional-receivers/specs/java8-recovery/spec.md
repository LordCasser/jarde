## ADDED Requirements

### Requirement: Immediate functional receivers have a Java target type

系统 SHALL 在已验证的 Java 8 lambda 或方法引用所生成的函数对象立即接收接口调用时，输出具有该函数式目标类型且可由 Java 8 编译的表达式。调用的目标方法、参数求值次数与顺序、返回值和异常 MUST 与原 class 一致；MUST NOT 仅以“正文零引用”证明源码可编译。

#### Scenario: Array constructor reference is immediately applied
- **WHEN** `int[]::new` 产生的 `IntFunction` 立即调用 `apply`，并由调用者检查返回数组的类型
- **THEN** 完整恢复类 SHALL 编译，零长度、正长度及负长度输入 SHALL 与原 class 分别得到同样长度或 `NegativeArraySizeException`

#### Scenario: Ordinary method reference is immediately invoked
- **WHEN** `Math::abs` 产生的 `IntUnaryOperator` 立即调用 `applyAsInt`
- **THEN** 完整恢复类 SHALL 编译，正数、零及负数的结果 SHALL 与原 class 相同；方法引用 MUST NOT 裸露为不能接后缀调用的 Java 文本

#### Scenario: Bound local control remains unchanged
- **WHEN** 相同数组构造器引用或方法引用先存入有类型的局部，再调用其接口方法
- **THEN** 完整恢复类 SHALL 继续编译执行，并保留局部求值、异常及调用顺序；MUST NOT 因修复即时接收者而增加无依据的检查或重复调用

### Requirement: Immediate functional typing remains proven and attributable

目标类型缺失、不是可拼写的函数式引用类型、或无法证明即时接收者满足被调用成员的类型要求时，系统 MUST 明确拒绝该结构，保留真实函数对象创建与调用的来源，MUST NOT 猜测强制转换。可恢复情形的正文和来源 SHALL 遵守既有预算、取消和证据选择契约。

#### Scenario: Functional target is not proved
- **WHEN** 恢复只知某值来自函数式工厂，却无法证明其 Java 目标类型与当前接口调用相容
- **THEN** 系统 SHALL 拒绝发布正常的即时调用正文，并在 fallback 中保留工厂与调用的物理来源

#### Scenario: Complete evidence traces creation and consumption
- **WHEN** 同一个已证明的即时调用分别请求 essential 与完整证据
- **THEN** 两次正文 SHALL 相同，完整证据 SHALL 保留真实工厂和调用的 BCI/成员来源，MUST NOT 虚构运行时 `checkcast` 指令

#### Scenario: Budget or cancellation stops atomically
- **WHEN** 接收者类型补足、表达式发射或来源构造触发预算耗尽或取消
- **THEN** 系统 SHALL 按现有停止契约响应，MUST NOT 发布一半已加目标类型、一半仍为非法裸露 lambda/方法引用的正常正文
