## ADDED Requirements

### Requirement: Complete Java 8 class source preserves proved ordinary parameterized method types

对同一物理方法声明的普通参数化 `Signature`，若其语法完整、每个参数与返回的擦除和真实 descriptor 一致，且类源码能完整表达这些类型并保持正文及调用绑定，完整类源码 SHALL 在对应方法声明中写出参数化类型。投影 MUST 保留物理成员身份、descriptor、异常及注解事实、原独立方法恢复结果；重编后的泛型反射类型 SHALL 与原 class 一致。

#### Scenario: Concrete type argument and raw control

- **WHEN** 一个 Java 8 类分别声明 `Iterable<String> strings(Iterable<String>)` 和没有 `Signature` 的 raw `List raw(List)`，两者正文直接返回参数
- **THEN** 完整类源码 SHALL 仅对前者保留 `String` 类型实参；两者重编后的泛型参数/返回反射类型、调用结果及真实 JVM descriptor SHALL 各自与原 class 一致

#### Scenario: Wildcard, nested arguments and parameterized array

- **WHEN** 同一类有可完整拼写并已证明绑定的 `List<? extends Number>`、`Map<String,List<Integer>>` 与 `List<String>[]` 参数和返回位置
- **THEN** 完整类源码 SHALL 保留通配方向、嵌套层级与数组维度；原类和恢复类经 Java 8 重编、`-Xverify:all` 执行及泛型反射对照 SHALL 一致

#### Scenario: Bodyless declaration

- **WHEN** abstract、interface 或 native 方法没有 `Code`，但其普通参数化 `Signature` 与物理 descriptor 一致且其 Java 8 声明可完整表达
- **THEN** 完整类源码 SHALL 在该无正文成员的方法头保留参数化类型，并维持原有 abstract/interface/native 成员形态与反射签名

#### Scenario: Independent method output and enhanced for remain separate

- **WHEN** 同一方法另外请求独立方法恢复，或其正文含已经恢复的 raw 元素增强 `for`
- **THEN** 类级泛型声明投影 MUST NOT 改变独立方法的物理 descriptor/报告，也 MUST NOT 在没有单独元素和异常边界证明时删除正文原有 cast、合成 unchecked 参数化转换或改写增强 `for` 元素类型

### Requirement: Ordinary parameterized projection refuses unsupported or unsound source declarations

参数化声明投影 SHALL 原子核对来源、Java 8 类型拼写、上下文类型变量、参数槽和注解/varargs/异常位置、已恢复正文的源级类型关系及受影响调用的目标绑定。任何这些前提不能证明时 MUST 保留按真实 descriptor 拼写的声明及可查拒绝原因；MUST NOT 发布部分泛型头或改变物理调用目标。单类请求未提供 Java 8 平台和外部 classpath 的完整类型存在性证明时，语法合法的嵌套引用是否可在另一环境重编不属于本条的保证；该已知边界必须在验收中陈述。

#### Scenario: Verifier-accepted Signature disagrees with descriptor

- **WHEN** 一份 JVM 可验证的 class 在方法 `Signature` 中把真实 `List` 参数/返回写成另一个原始类型，而物理 descriptor 仍是 `List`
- **THEN** 完整类源码 MUST 拒绝该泛型投影、维持 `List` descriptor 声明并报告冲突位置，不得照搬元数据中的不匹配类型

#### Scenario: Unpresented class variable or ambiguous source name

- **WHEN** 参数化 `Signature` 引用类头未呈现的类型变量、不能唯一拼成 Java 源的内部类名，或含当前声明不能完整定位的 type-use 注解
- **THEN** 系统 MUST 拒绝整项投影，不得写出游离类型变量、误指另一类或遗失注解

#### Scenario: Changed source overload binding

- **WHEN** 参数化参数或返回会改变已恢复正文或同类调用位置的源码重载选择，而系统无法证明该位置仍绑定原 JVM 方法引用
- **THEN** 系统 MUST 保留 descriptor 声明与原调用来源；单凭 `Signature` 擦除相同或简单返回值相同不足以准入

#### Scenario: Budget or cancellation before publication

- **WHEN** 属性读取、语法/擦除证明、声明构造、来源记录或输出计费期间预算耗尽或任务被取消
- **THEN** 请求 SHALL 按现有停止契约结束，MUST NOT 发布半个参数化头或丢失物理方法；成功时 essential 与 all 的 Java 正文 SHALL 相同
