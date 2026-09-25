## Why

Java 8 的泛型成员类创建可在字节码中保留非静态外层实例、合成构造首参和成员泛型 `Signature`，但当前 Jarde 的成员构造证明一遇目标类或构造器 `Signature` 就拒绝。本地 JADX 1.5.6 在两层泛型样例中丢失外层实例；即使外层类非泛型，只将调用方返回类型改为 `Object`，它也会把合成首参当源码实参，生成不能重编的 `new Outer.Inner(outer, ...)`。这里应沿已证明的物理绑定扩展，而非复制该输出。

## What Changes

- 在**非泛型外层类、泛型非静态成员类**的受限调用点，复用现有成员关系、SSA 身份、空值检查和顺序证明，核对成员类及构造器 `Signature` 的作用域与物理参数尾部，恢复 `outer.new Inner<>(args)`。
- 使构造 AST 明确表达经证明的 diamond 拼写，同时保留完整二进制类型、物理实参及 BCI 来源；证据不足时保留已有拒绝，不靠泛型签名或 `$` 名称猜测外层绑定。
- 以冻结的原始外层/成员 class 为依赖，只重编调用方并执行对照；类族源码声明和整 jar 重编单列为后续工作。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加已证明的泛型成员类调用点的限定构造及显式拒绝边界。

## Impact

涉及已选 class-source 环境的成员目标证明、`jarde-java` 的 `new@1` 事实交接与 `New` 发射；不增加通用 pass、运行时依赖或类级嵌套声明装配。首片不覆盖泛型外层类 `A<T>`、外层变量捕获、复杂界、重载绑定未证、匿名/局部类或完整类族源码投影。Java 8 编译器和本地 JADX 只作受控对照，原 class 行为为语义判据。
