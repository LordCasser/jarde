## ADDED Requirements

### Requirement: Proven simple generic constructor preserves its source signature

对顶层、直接继承 `java.lang.Object` 的类，当构造器自己的 `Signature` 类型变量、界和参数与物理 descriptor 完整一致，构造器正文经证明只调用 `Object()` 并返回，且本类没有未证明的构造器调用绑定时，完整类源码 SHALL 写出构造器类型参数及其泛型参数。该声明重编后的构造器泛型反射、合法显式类型实参调用和错误实参的编译拒绝 MUST 与原 class 一致。

#### Scenario: Bound constructor parameter

- **WHEN** 构造器为 `<T extends Number> C(T value)`，物理参数为 `Number`，正文仅有无参 `Object` 构造器调用和返回
- **THEN** 完整类源码 SHALL 写出 `<T extends java.lang.Number> C(T value)` 的等价声明；Java 8 重编后的构造器类型参数与泛型参数反射 SHALL 与原 class 一致

#### Scenario: Explicit caller type arguments

- **WHEN** 调用方以 `<Integer>` 显式实参和 `Integer` 参数调用该构造器，同时另一个调用方以 `<Integer>` 搭配 `Double` 参数
- **THEN** 前者 SHALL 重编并运行，后者 SHALL 被 `javac --release 8` 拒绝，与原 class 作为依赖时相同

#### Scenario: Debug tables are absent

- **WHEN** 相同构造器分别以 `-g` 和 `-g:none` 编译
- **THEN** 两份完整类源码 SHALL 恢复相同泛型约束，重编、反射与调用方编译结果 SHALL 一致

### Requirement: Unproved generic constructor keeps its physical declaration

如果 `Signature` 语法、作用域或擦除不成立，正文使用参数、包含 `this(...)` 链或其它未证明效果，或者本类构造器调用/源级层次关系无法证明，系统 MUST 保持物理构造器声明及其属性来源，并给出局部拒绝；不得只投影类型参数或部分参数位置。预算耗尽或取消 MUST 停止候选发布，成功请求的 essential/all Java 正文 SHALL 相同。

#### Scenario: Verifier-valid signature contradiction

- **WHEN** 仍可通过 JVM 验证的 classfile 保持物理参数 `Number`，但构造器 `Signature` 使用未绑定变量或与 `Number` 不同的第一界擦除
- **THEN** 完整类源码 SHALL 保持物理构造器参数并报告局部拒绝，不得写出不可验证的半个泛型声明

#### Scenario: Constructor body consumes the parameter

- **WHEN** 构造器将参数写入字段、传给另一构造器或执行其它未纳入简单正文证明的动作
- **THEN** 系统 SHALL 保留物理声明并报告未证明的泛型投影，不得根据 `Signature` 单独改写参数类型

#### Scenario: Same-class constructor binding is unproved

- **WHEN** 本类另一个正文调用该构造器，且改变源级泛型声明可能改变重载或类型检查
- **THEN** 系统 SHALL 拒绝泛型构造器投影；方法正文和物理成员身份 SHALL 保留
