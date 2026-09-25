## ADDED Requirements

### Requirement: Complete class source preserves proved method-local generic signatures

对于声明了可完整解释且与真实方法 descriptor 擦除一致的 Java 8 方法泛型 `Signature`，若已恢复方法体及受影响调用在该泛型声明下仍可证明为合法且保持原绑定，完整类源码 SHALL 在方法声明中保留该方法自己的类型参数、上界和参数/返回位置的类型变量。该投影 MUST 保留原物理方法身份、descriptor、属性来源及独立方法体恢复报告；不能由字节码源码输出猜测泛型。

#### Scenario: Bounded type variable in parameter and result

- **WHEN** `choose` 的 descriptor 是 `(Number, Number, boolean)Number`，同一物理成员的签名为 `<T extends Number> T choose(T,T,boolean)`，方法体由两个 `T` 参数选择并返回，且该类型关系和调用绑定均已证明
- **THEN** 完整类源码 SHALL 写出方法局部的 `<T extends java.lang.Number>`、两个 `T` 参数及 `T` 返回；Java 8 重编后普通调用结果和反射可见的类型变量名称、界及参数/返回类型 SHALL 与原 class 一致

#### Scenario: Evidence selection leaves the Java declaration unchanged

- **WHEN** 同一合法 class 分别请求 essential 与 all 证据
- **THEN** 两次成功的完整 Java 正文 SHALL 相同；详细来源可随证据选择改变，但泛型准入、物理方法与独立方法体结论 MUST 相同

#### Scenario: Ordinary checked exception is absent from the generic Signature throws suffix

- **WHEN** 同一方法的 `Signature` 为 `<T extends Number> T choose(T)` 且未带 `^` throws 后缀，物理 `Exceptions` 属性另列 `java.io.IOException`，正文及调用已证明兼容
- **THEN** 泛型投影 SHALL 保留 `<T extends Number>` 与从 `Exceptions` 得到的 `throws java.io.IOException`；MUST NOT 因 `Signature` 的可选 throws 列表为空而拒绝或丢失异常声明

### Requirement: Generic projection refuses unproved or unrepresentable signatures

泛型方法的声明投影 SHALL 对完整语法、类型变量作用域、擦除结果、已恢复方法体的源级类型关系、源码名称绑定、调用目标和目标版本进行证明。无法证明时 MUST 保留按真实 descriptor 拼写的声明并记录拒绝原因；MUST NOT 发布会编译失败、改变 JVM 方法 descriptor/调用目标或遗漏已知元数据的泛型源码。

#### Scenario: Erasure disagrees with physical descriptor

- **WHEN** 方法签名的类型变量第一界或任何参数/返回擦除与物理 descriptor 不一致
- **THEN** 系统 MUST 拒绝泛型投影，保留物理 descriptor 的类源码声明，并报告不一致的成员及位置

#### Scenario: Unknown type variable or unsupported syntax

- **WHEN** 签名引用未在方法或已呈现的类声明中定义的类型变量，或签名含当前投影无法完整拼写的嵌套类型、通配符、数组、throws、type-use 注解等结构
- **THEN** 系统 MUST 拒绝整项泛型投影，保留原 descriptor 声明；MUST NOT 只写出可识别的一半泛型语法

#### Scenario: Bound method call cannot be preserved

- **WHEN** 当前完整类中其他方法对这个物理方法的调用，可能因泛型声明改变 Java 重载/类型推断绑定，而该调用位置没有得到保持原 Methodref 的证明
- **THEN** 系统 MUST 拒绝会改变该绑定的类级泛型投影，保留原调用和物理来源

#### Scenario: Erasure matches but the method body is not valid under the type variable

- **WHEN** 同一物理方法的签名擦除与 `()Number` descriptor 一致，字节码正文返回一个新建 `Integer`，但签名要求 `<T extends Number> T`，或正文中的赋值、表达式合流、方法调用无法证明在 `T` 下保留源级类型与目标绑定
- **THEN** 系统 MUST 拒绝整项泛型声明投影并记录正文不兼容位置；MUST NOT 仅凭 descriptor 擦除一致输出无法重编或调用目标变化的泛型源码

#### Scenario: Budget or cancellation during signature proof or publication

- **WHEN** 属性读取、签名解析、擦除核对、来源记录或新声明输出遇到预算耗尽或取消
- **THEN** 系统 SHALL 按现有停止契约结束，MUST NOT 发布半个泛型头或失去物理成员记录；成功时 essential/all 正文仍相同

#### Scenario: Independent method recovery

- **WHEN** 调用方只请求单方法恢复
- **THEN** 方法体 SHALL 继续以物理 descriptor 和现有 SSA 事实恢复，不因完整类的泛型声明投影改变既有方法结果
