## ADDED Requirements

### Requirement: Proven class type variables are presented with their method uses

当类自身的 `Signature` 给出可由物理父类与接口验证、且可写成 Java 8 的类型变量声明时，完整类源码 SHALL 在类头发布该变量及其已证边界；同类方法 `Signature` 中引用该变量的位置 SHALL 使用同一类型变量，而非独立回退为擦除类型。完整类重编后 MUST 保留类与已发布方法可观察的泛型反射签名、泛型调用方的编译结果和原 class 的执行行为。

#### Scenario: Direct identity through a class variable

- **WHEN** 合法 Java 8 类声明 `<U>`，方法的物理 descriptor 为 `(Object)Object`，类和方法 `Signature` 分别声明及引用 `U`，且正文直接返回输入参数
- **THEN** 类源码 SHALL 写出 `<U>` 与 `U identity(U)`；完整类、相同的泛型调用方均 SHALL 通过 `javac --release 8`，`java -Xverify:all` 的值和泛型反射 SHALL 与原 class 相同

#### Scenario: Bounded class variable and no-body member

- **WHEN** 类变量有可证明的 class/interface 边界，且一个无正文成员的方法 `Signature` 引用该变量，物理父类、无泛型实参的接口及方法 descriptor 与各自擦除一致
- **THEN** 类头与成员头 SHALL 在相同作用域中写出边界和变量；同一成员的物理身份与 `Signature` 来源 SHALL 保留，重编后的泛型反射 SHALL 与原 class 一致

### Requirement: Unproved class variable scope does not leak into source

类 `Signature` 的变量、父类或接口与物理 class 不符，或方法变量引用无法绑定到已发布的类作用域时，系统 MUST NOT 为了生成可编译文本而发明变量声明；相关类头或方法头 SHALL 保留物理 descriptor 可证明的形状与可查拒绝。请求在预算耗尽或取消时 MUST NOT 发布一半泛型类头或孤立的方法类型变量，essential/all 成功请求的 Java 正文 SHALL 相同。

#### Scenario: Conflicting class signature and orphaned method variable

- **WHEN** 类 `Signature` 的父类/接口擦除与物理 class 不一致，或类头无法发布而方法 `Signature` 仍引用该类变量
- **THEN** 类级泛型候选 SHALL 被拒绝，方法 MUST NOT 单独写出未声明的类型变量；报告 SHALL 指出拒绝来源，不能把原类不存在的变量补进头部

#### Scenario: Parameterized parent requires inherited-member proof

- **WHEN** 类 `Signature` 在父类或接口位置使用类变量作为泛型实参，而当前投影没有证明整组继承成员的 Java 声明兼容性
- **THEN** 类级泛型候选 SHALL 局部拒绝，成员不得单独写出该类变量；物理父类、接口和成员 descriptor 的来源仍 SHALL 可查

#### Scenario: Stop and evidence selection

- **WHEN** 类型变量证明中预算耗尽或请求取消，或者对同一个已证 class 分别请求 essential/all 证据
- **THEN** 停止请求 SHALL 遵守现有停止契约，成功请求的类/方法 Java 正文 SHALL 一致，物理属性和来源仍可查
