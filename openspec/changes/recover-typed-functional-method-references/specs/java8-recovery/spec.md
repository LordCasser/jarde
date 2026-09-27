## ADDED Requirements

### Requirement: Proven parameterized functional return and method reference

系统 SHALL 在 Java 8 目标下，仅当完整方法正文直接返回受支持的 LambdaMetafactory 函数值、方法 `Signature` 的参数化函数目标与该站点的实例化调用类型相容，且方法引用写法能保持实现成员及转换语义时，输出可重编的参数化返回声明和方法引用。声明、正文与物理来源 MUST 作为同一次证明的结果发布；任一条件无法证明时 MUST 保留拒绝或不完整标记，不得发布可编译但改变行为的 raw 类型替代品。

#### Scenario: 泛型 Function 的静态引用
- **WHEN** 完整 Java 8 方法声明返回 `Function<String,Integer>`，正文仅直接返回一个以 `Integer.parseInt(String)` 为实现目标且结果装箱相容的已验证函数站点
- **THEN** 输出保留 `Function<String,Integer>`、`Integer::parseInt` 及对应物理来源，完整类以 Java 8 重编，调用、结果和反射泛型返回类型与原 class 一致

#### Scenario: 泛型 Function 的绑定实例引用
- **WHEN** 完整方法声明返回 `Function<String,Integer>`，正文仅捕获当前实例并直接返回其 `length(String)` 方法引用，目标、参数及结果相容
- **THEN** 输出保留参数化声明和绑定的 `this::length`，创建时捕获当前实例且调用时使用同一实例，完整类重编及运行与原 class 一致

#### Scenario: 零参泛型 Supplier 引用
- **WHEN** 完整方法声明返回 `Supplier<String>`，正文仅捕获当前实例并直接返回相容的零参成员引用
- **THEN** 输出保留 `Supplier<String>` 与该成员引用，反射返回类型、调用结果和来源与原 class 一致

#### Scenario: 泛型声明与站点证据不相容
- **WHEN** 方法 `Signature` 的参数化函数目标与函数站点的实例化参数或结果不相容，或正文有未证明的额外语句、异常处理或非直接返回路径
- **THEN** 系统 MUST 拒绝参数化声明与方法引用的组合，并给出不完整/拒绝证据；不得仅凭 `Signature` 或 bootstrap handle 猜测可编译源码

#### Scenario: 接收者在创建时可观察
- **WHEN** 绑定引用的接收者可能为空，或其求值有副作用、可在引用创建后被重新赋值，而当前证书无法证明创建时空值失败、恰好一次求值与对象别名
- **THEN** 系统 MUST 安全拒绝该方法引用恢复，不得把空值失败或副作用移至函数调用时，也不得引用后来替换的对象

#### Scenario: 中止与预算
- **WHEN** 证据构建或源码投影被取消，或任一适用预算耗尽
- **THEN** 系统 MUST 返回停止状态，不得发布半个参数化声明、半个函数正文或伪造来源
