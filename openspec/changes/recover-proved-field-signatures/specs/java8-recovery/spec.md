## ADDED Requirements

### Requirement: Proven field Signature is presented as a Java field type

对 Java 8 顶层类中的字段，当自身 `Signature` 的变量引用能绑定到已发布的类作用域、擦除与物理 descriptor 完全相同、类型可拼写且字段未参与本类尚未证明兼容的正文使用时，完整类源码 SHALL 发布该泛型字段类型。发布后的源码和同一泛型调用方 MUST 通过 Java 8 重编；字段的 `getGenericType()`、物理 descriptor 和可执行行为 SHALL 与原 class 一致。

#### Scenario: Parameterized and wildcard field without body use

- **WHEN** 字段 `Signature` 分别给出可拼写的参数化类型和通配符类型，擦除与各自 descriptor 一致，且本类方法不引用这些字段
- **THEN** 类源码 SHALL 保留泛型实参与通配符；重编后的字段泛型反射、调用方编译和原 class 可执行行为 SHALL 一致

#### Scenario: Class variable field and array

- **WHEN** 类头已发布类型变量 `T`，字段 `Signature` 以 `T` 或 `T[]` 引用它，字段 descriptor 与类变量第一边界擦除一致且正文无未证明的字段使用
- **THEN** 字段声明 SHALL 使用同一个 `T` 作用域；完整类与泛型调用方 SHALL 可重编，类/字段泛型反射 SHALL 与原 class 一致

### Requirement: Unproved field Signature keeps the physical declaration

字段 `Signature` 的擦除不符、类型变量未绑定、类型注解路径不可保留或字段在本类正文中的泛型静态类型兼容性未获证明时，系统 MUST NOT 仅因该属性可解析就替换物理字段类型；字段声明 SHALL 保留 descriptor 拼写及可查拒绝。预算耗尽或取消 MUST NOT 发布部分字段候选；成功请求的 essential/all Java 正文 SHALL 相同。

#### Scenario: Verifier-valid erasure mismatch

- **WHEN** classfile 保持可验证且字段 descriptor 不变，但字段 `Signature` 被改为擦除不同的类型
- **THEN** 该字段 SHALL 保留物理类型并报告签名拒绝，不得从签名生成与 descriptor 矛盾的声明

#### Scenario: Generic field conflicts with method body

- **WHEN** 字段 descriptor 仍是原始 `List`，其 `Signature` 声称 `List<String>`，而本类正文通过该字段执行需要另一元素类型的操作，字节码仍通过 JVM 验证
- **THEN** 在缺少完整正文兼容证明时该字段 SHALL 保留物理类型及局部拒绝，不能发布使同一完整类无法通过 Java 8 编译的泛型声明

#### Scenario: Signature projection stops

- **WHEN** 字段属性读取或候选证明遇到预算耗尽/取消，或者同一输入分别选择 essential/all 证据
- **THEN** 停止请求 SHALL 遵守既有停止契约且无半个泛型字段；成功请求的正文 SHALL 相同，物理字段身份、属性和来源仍 SHALL 可查
