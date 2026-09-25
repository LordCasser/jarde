## ADDED Requirements

### Requirement: Proven class-variable throws preserves generic checked-exception semantics

对 Java 8 顶层普通类的无正文方法，若其 `Signature` 中的 `throws` 类型变量属于已发布类头、泛型签名与物理方法及 `Exceptions` 属性逐位置擦除一致，而且该变量的异常类型约束可证明，完整类源码 SHALL 在该位置写出同一个类型变量。输出类与同一泛型调用方 MUST 通过 Java 8 重编；方法的 `getGenericExceptionTypes()`、物理异常和可执行行为 SHALL 与原 class 一致。

#### Scenario: Class-bound unchecked specialization

- **WHEN** 类声明 `E extends Exception`，无正文方法签名声明 `throws E`，物理 `Exceptions` 为 `Exception`，调用方把接收者静态限定为该类的 `RuntimeException` 实例
- **THEN** 完整类源码 SHALL 写出 `throws E`，无需检查异常处理的强类型调用方 SHALL 可重编，反射中的泛型异常变量 SHALL 与原 class 一致

#### Scenario: Debug information is immaterial

- **WHEN** 相同源码分别以 `-g` 和 `-g:none` 编成 Java 8 class
- **THEN** 两份完整类源码 SHALL 恢复相同的泛型异常声明，重编及运行结果 SHALL 相同

### Requirement: Unproved throws does not overwrite a physical declaration

异常变量未绑定、与 `Exceptions` 的擦除不一致、不能证明为异常类型、有不可保留的源码位置，或泛型候选未完整构造时，系统 MUST 保留该方法的物理异常声明与局部拒绝。若方法 `Signature` 没有泛型 `throws` 后缀，已有的物理 `Exceptions` 声明 SHALL 保留。预算耗尽或取消 MUST 遵守请求停止契约，不得发布半个候选；成功请求的 essential/all Java 正文 SHALL 一致。

#### Scenario: Verifier-valid contradictory signature

- **WHEN** classfile 的方法 descriptor 和 `Exceptions` 不变，但泛型异常变量的类级擦除与物理异常不符，classfile 仍通过 JVM 验证
- **THEN** 该方法 SHALL 保留物理 `throws` 并报告局部拒绝，不得据不一致的签名改写异常声明

#### Scenario: Signature omits generic throws suffix

- **WHEN** 方法存在物理 `Exceptions` 属性，而 `Signature` 仅含泛型参数或返回类型、没有 `throws` 后缀
- **THEN** 方法源码 SHALL 继续写出物理异常声明，不得将其清空

#### Scenario: Request stops during projection

- **WHEN** 泛型异常属性读取或投影期间触发预算耗尽或取消，或同一 class 分别请求 essential/all
- **THEN** 停止请求 SHALL 不发布部分声明；成功请求的两种证据选择 SHALL 产生同一 Java 正文
