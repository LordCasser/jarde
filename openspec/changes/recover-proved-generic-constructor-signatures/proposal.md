## Why

当前 Jarde 将已带方法 `Signature` 的构造器 `<T extends Number> GenericConstructor(T value)` 写成 `GenericConstructor(Number value)`。原 class 与 JADX 1.5.6 均保留构造器类型参数及泛型反射，并拒绝显式 `<Integer>` 搭配 `Double` 的调用；Jarde 输出却接受该错误调用。[三方对照](../../evidence/java-syntax-2026-09-24/generic-constructor-signatures/analysis.md)表明这是声明约束丢失，不是复杂正文恢复问题。

## What Changes

- 对可证明的简单构造器复用 reader 已有方法 `Signature` 解析、作用域和擦除证明，完整投影构造器自己的类型参数及参数类型。
- 仅在同轮正文证明构造器只调用 `Object` 的无参构造器并返回、且不存在本类构造器调用绑定风险时发布泛型声明；否则保留物理声明和局部拒绝。
- 以 Java 8 完整类重编、正常与错误调用方、泛型反射、`-g`/`-g:none` 以及 verifier-valid 负例验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在已证无参数使用的简单构造器上，完整类源码保留构造器类型参数及其参数约束。

## Impact

影响 `jarde-reader::signature` 的现有证明消费、`jarde-java` 的同轮 AST 候选、`src/class_source.rs` 的构造器声明投影及 `src/facade.rs` 的候选交接。不新增 Signature parser、恢复 pass、crate 或依赖；有 `this(...)` 链、参数参与正文、继承类构造器、复杂注解或泛型异常的扩围另行取证。
