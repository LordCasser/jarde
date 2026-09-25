## ADDED Requirements

### Requirement: Special invocations preserve their declared selection

非构造器 `invokespecial` 的恢复 SHALL 保留其非虚调用选择，不得默认为 `this.name` 或普通实例虚调用。当指令引用的类型是当前声明的直接父类，且接收者已证明是方法入口的实例 `this` 时，文本 SHALL 写成 `super.name(...)`。当指令引用的是当前声明的直接父接口，且接收者同样为该 `this` 时，文本 SHALL 写成 `接口类型.super.name(...)`。当前类声明的 private 方法 SHALL 使用实际接收者，不能改成 super。超出这些证据的特殊调用 MUST 保留有来源的缺口。

构造器调用 SHALL 继续使用已证明的 `super(...)`、`this(...)` 或创建表达式，不受本要求的非构造器选择影响。普通 virtual/interface/static 调用 SHALL 保持原调用类别。恢复 MUST NOT 为证明这项选择隐式展开类型闭包或读取其它方法体。

#### Scenario: A superclass override keeps its base implementation

- **WHEN** 父类 `value` 返回 7，子类 `value` 的字节码特殊调用该直接父类实现再加 1
- **THEN** 恢复文本 SHALL 含 `return super.value() + 1`；重编译执行 SHALL 返回 8，不得自递归

#### Scenario: An explicit interface default call keeps its interface

- **WHEN** 同一子类直接实现接口，接口默认 `value` 返回 11，而父类 `value` 返回 7，某方法特殊调用接口实现
- **THEN** 文本 SHALL 含 `接口类型.super.value()`；重编译执行 SHALL 返回 11，不能调用父类或当前类的覆写

#### Scenario: A private call can use another instance

- **WHEN** 当前类的 private 方法分别通过 `this` 和另一个同类参数接收者被特殊调用
- **THEN** 文本 SHALL 保留各自接收者，执行结果及空参数导致的异常与原 class 一致，MUST NOT 把参数接收者改成 super

#### Scenario: Missing target or receiver proof remains visible

- **WHEN** 特殊调用的目标不是当前类已声明的 private 方法、不是有同源声明证明的直接父类/接口，或者 super 调用的接收者不是已证明的入口 this
- **THEN** 文本 SHALL 保留指名调用及所需生产者的缺口，而不是制造一次可能改变调度的 Java 调用

#### Scenario: Argument evaluation and source mapping are retained

- **WHEN** 已证明的 super 调用参数包含一次可计数或抛异常的调用
- **THEN** 恢复文本 SHALL 保持参数求值顺序、次数与异常结果，并保留特殊调用和接收者的物理方法/BCI 来源

#### Scenario: Refused nested calls retain argument producers

- **WHEN** 可被 JVM 接受的特殊调用因缺少目标选择证据被拒绝，而它的实参或接收者含已延期的调用生产者
- **THEN** 无论外层调用作为语句还是返回表达式被拒绝，缺口 SHALL 保留这些生产者的来源，不能只引用外层调用和返回而隐藏实参效果
