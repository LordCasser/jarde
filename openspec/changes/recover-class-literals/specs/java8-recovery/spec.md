## ADDED Requirements

### Requirement: A proved Class constant is a Java class literal

合法 `ldc`/`ldc_w` 指向真实 Class 常量池项且类或数组类型可无歧义写出时，系统 SHALL 将该值恢复为 Java 8 的 `T.class`，保留它在返回、局部或调用参数位置的类型和值身份。其它常量种类不得因本要求被猜成类字面量。

#### Scenario: Reference, self and array classes

- **WHEN** 同一合法 class 的方法分别返回 `String.class`、本类 `.class`、`String[][].class` 与 `int[][].class`
- **THEN** 完整恢复源码 SHALL 可编译执行；每个返回值与原 class 为同一 `Class` 对象，数组层数和基本元素类型相同

#### Scenario: A class literal is a call argument

- **WHEN** 方法将一个由 `ldc Class` 产生的值传给可呈现调用，调用可观察地计数
- **THEN** 恢复源码 SHALL 把字面量放在原参数位置，调用执行一次，返回对象与原 class 相同

#### Scenario: Existing primitive and void class paths remain stable

- **WHEN** 普通 Java 8 编译的 `int.class` 和 `void.class` 通过既有静态 `TYPE` 字段产生
- **THEN** 恢复源码 SHALL 保留其可编译且值相同的既有结果，本要求 MUST NOT 伪造一条不存在的 `ldc Class`

### Requirement: Class literal recovery preserves evidence and refusal boundaries

Class 字面量 SHALL 由真实指令/池项来源锚定；无法验证池名、类型写法或类型限定符绑定时 SHALL 来源完整地拒绝，不产生歧义的 Java 文本。默认与完整来源请求 SHALL 给出相同正文，预算与取消 SHALL 沿既有停止合同生效。

#### Scenario: The type cannot be spelled or resolved without ambiguity

- **WHEN** Class 池名无法按合法 Java 类型呈现，或者当前 class 已读声明证明同名类型会把必要的限定符绑定到另一类且当前恢复无法安全避让
- **THEN** 系统 SHALL 保留实际 `ldc` 和消费位置的引用，MUST NOT 写出可能指向另一值的 `X.class`；同名局部变量在类型上下文中不应单独触发拒绝

#### Scenario: A non-Class constant is not admitted

- **WHEN** `ldc` 指向 MethodType、MethodHandle 或不可解析池项而非 Class 项
- **THEN** 系统 SHALL 保持现有来源完整的拒绝，MUST NOT 写成类字面量

#### Scenario: Source mapping and output mode

- **WHEN** 同一合法类字面量分别请求默认与完整来源证据
- **THEN** 正文 SHALL 一致，字面量的指令 BCI 与池项 SHALL 可追溯；较低预算或取消 SHALL 按已有停止结果返回
