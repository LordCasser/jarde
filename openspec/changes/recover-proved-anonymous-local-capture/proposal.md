## Why

固定 JADX `TestAnonymousClass7` 要求匿名 `Runnable` 引用创建方法的 `double` 参数。DT-08 隔离对照中，原源码和 JADX 完整源码都能在 Java 8 重编运行；Jarde 保留物理 child，而其构造器先写 synthetic 捕获字段、后调用 `super()`，导致完整源码无法重编。

## What Changes

- 在唯一直接返回匿名接口实例、一个 synthetic-final `double` 捕获字段、一个准确构造器参数的闭合形态下，证明根参数、创建点实参、构造器写入及覆盖方法的所有字段读取是同一值。
- 把这些已证字段读取呈现为根方法参数引用，原子输出 `new Runnable() { ... }`；物理 child 继续可独立查询。
- 用原/JADX/Jarde 完整源码的 Java 8 重编与验证运行，以及二次写入、错误槽、跨类使用、预算停止等负例验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：恢复一个受限 Java 8 匿名接口实现对创建方法 primitive 参数的捕获，保证不能证明时不发布部分内联源码。

## Impact

沿用 `src/member_inner.rs` 的捕获字段/构造器/SSA 证明结构、`src/facade.rs` 的匿名接口整类投影和 `crates/jarde-java` 的同次 AST 读取替换；不新增 crate、外部依赖、执行目标代码或公开 API。本变更只验收 `double` 参数直接捕获；多字段、局部变量复杂赋值、闭包链以及 DT-08 的外层实例子形态另行处理。
