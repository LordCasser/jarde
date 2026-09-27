## Why

EM-12 的已证 `Outer.super` 桥在父类出现任何同名重载时一律拒绝源码投影，即使成员调用传入的静态类型使另一重载必然不可适用。[冻结对照](../../evidence/java-syntax-2026-09-27/em12-super-dispatch/baseline/summary.json)中原 class 与 JADX 完整源码均重编运行输出 `number`，Jarde 因这一过宽拒绝留下不可重编的桥方法。

## What Changes

- 复用现有桥体、捕获接收者、调用闭包和家族投影证书；仅在成员实参的**生成源码静态类型**与桥参数一致、另一同名重载经所选完整类层级可证明不适用时放行该桥的源码绑定。
- 对重载可适用性不明、泛型/varargs/装箱/原始类型转换、checked exception 不明或依赖读取不完整的情形维持原子拒绝，不以 JVM `MethodRef` 代替 Java 8 重载解析。
- 将旧 `OuterReceiverCases` 回归从“预期拒绝”修为正向可重编/可运行断言；额外覆盖真正的同型 `other` 和有竞争力重载负例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：当重载竞争者已证明对生成实参不适用时，恢复精确绑定的词法 `Outer.super.m(args)`；证明不足仍拒绝。

## Impact

前置条件是现有 `project-proved-outer-super-bridges` 的桥/调用/闭包/家族证明。主要修改 `src/facade.rs::prove_outer_super_source_binding` 并在必要时读取现有 SSA 与已选类声明；不增加通用 Java 类型求解器、IR 层或后台依赖，也不顺带处理 EM-01 的多成员家族、EM-20 局部声明或泛型重载。JADX 的 `InlineMethods` 与 `InsnGen` 仅作固定测试和源码呈现对照，正确性以原 class 与 Java 8 重编运行相等为准。
