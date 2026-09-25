## Why

合法 Java 8 类 `ClassVariableBoundary<U>` 的类 `Signature` 声明了 `U`，方法 `identity(U):U` 也在自己的 `Signature` 中引用它。当前 Jarde 类头只写裸类名，方法头把 `U` 回退为 `Object`；原类和 JADX 保留类变量，泛型调用方能编译并通过反射核对，而 Jarde 单类输出不能编译同一调用方。[现有三方边界证据](../../evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/boundaries.md)已把它与方法自有 `<T>`、缺失依赖和内类路径问题区分。

## What Changes

- 从类自身的 `Signature` 解析、验证并投影已证明的类级类型变量及其边界，保持物理父类/接口身份、名称和声明来源；只在完整类头可拼写时发布。
- 将已发布的类变量作用域传给同类方法 `Signature` 的擦除与源码拼写证明，使 `U identity(U)` 与 `<U> class` 同时成立；有正文和无正文成员分别沿用既有证明路径。
- 对 class `Signature` 与物理父类/接口不符、变量未绑定、复杂内类或正文/调用不能证明的情形报告局部拒绝，不发明类型变量来让文本碰巧可编译；预算、取消和 essential/all 仍保持原子性。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明的类级 `Signature` 类型变量可以出现在类声明及其方法头中，并保持重编后的泛型反射与调用行为。

## Impact

主要沿 `jarde-reader::signature` 现有语法树增加类级擦除/作用域证明，沿 `src/class_source.rs` 的类声明与 `project_method_signature` 接缝传递该证据；`src/facade.rs` 保持类级与成员级候选的发布顺序。复用既有 reader/query、预算和类源码结果，不新增 crate、依赖或全局泛型求解器。字段泛型、匿名/内类捕获、构造器合成参数、外部类型存在性及增强 `for` 元素类型由独立任务处理；[嵌套缺类对照](../../evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/nested-missing-signature-reference.md)已证实后者是通用编译 classpath 边界，而非本项应顺手修的类变量语法。
