## ADDED Requirements

### Requirement: Proven no-body method-local Signature preserves its source declaration

对没有 `Code` 的顶层类或接口抽象方法，当方法自有类型参数、参数、返回与异常位置可以从该方法的 `Signature` 完整拼写，所有位置与物理 descriptor 和 `Exceptions` 的擦除一致，且类型层次与本类型调用不会改变源级绑定时，完整类源码 SHALL 写出该泛型方法声明。方法的类型参数、泛型参数/返回/异常反射信息、强类型调用方及覆写关系 MUST 与原 class 一致。

#### Scenario: Exception variable belongs to the method

- **WHEN** 方法声明 `<X extends Exception> void raise() throws X`，物理异常为 `Exception`，调用方以 `<RuntimeException>` 显式实参调用并有一个泛型覆写实现
- **THEN** 完整类源码 SHALL 同时保留 `<X>` 和 `throws X`；类、覆写实现及调用方 SHALL 通过 Java 8 重编，异常泛型反射与原 class 一致

#### Scenario: Type variable appears in parameter and return

- **WHEN** 无正文方法的自有类型参数同时出现在可拼写参数和返回位置，且擦除与物理 descriptor 一致
- **THEN** 输出 SHALL 在同一个方法声明中保留类型参数及对应参数/返回；强类型调用方与方法反射 SHALL 和原 class 一致

#### Scenario: Root interface method

- **WHEN** 顶层接口不继承其它接口，抽象方法的自有类型参数与参数、返回或异常位置均有完整签名和擦除证明
- **THEN** 接口源码 SHALL 保留同一泛型方法声明；实现类和调用方 SHALL 可重编，方法反射 SHALL 与原 class 一致

#### Scenario: Debug tables are absent

- **WHEN** 相同源码分别以 `-g` 和 `-g:none` 编译
- **THEN** 两份完整类源码 SHALL 恢复相同泛型声明；重编、反射和运行 SHALL 相同

### Requirement: Unproved no-body Signature remains a physical method

如果方法签名语法或变量作用域不闭合、任一擦除或异常上界不符、类层次/本类调用绑定未证明、注解位置不能保留，系统 MUST 保持物理方法声明和可查的局部拒绝。缺少泛型 `throws` 后缀时物理 `Exceptions` SHALL 继续呈现。预算耗尽或取消 MUST 停止候选发布，成功请求的 essential/all Java 正文 SHALL 相同。

#### Scenario: Verifier-valid scope or erasure contradiction

- **WHEN** 不改变方法 descriptor 和物理异常、仍通过 JVM 验证的 classfile 带有未绑定变量或不一致的泛型异常上界
- **THEN** 输出 SHALL 保留物理方法声明并报告局部拒绝，不得发布不可重编的半个泛型方法头

#### Scenario: Inherited contract is unknown

- **WHEN** 类继承一个尚未证明的父类/接口方法契约，而重写泛型方法头可能改变源级覆写关系
- **THEN** 方法泛型投影 SHALL 被局部拒绝，保持物理声明和原属性来源

#### Scenario: Generic Signature has no throws suffix

- **WHEN** 方法 Signature 描述类型参数但没有泛型异常后缀，物理 `Exceptions` 列有异常
- **THEN** 泛型方法头若能独立证明 SHALL 保留物理异常声明，不得误清空
