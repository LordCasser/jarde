## Why

2026-10-08 双编译腿已证明泛型字段投影有源码类型缺口：ObjectSetter/ObjectHold 的 Object 值与 CrossSetter 的 U 值被写入投影后的 T 字段，输出无法编译。当前字段使用证明仅核对读位，错误地把同一擦除当作源码写入合法；先修复这个证明契约，再单独恢复 Hold(T) 构造器。

## What Changes

- 泛型字段发布必须证明全部已呈现写位的 RHS 在**实际发布**的源类型下可赋给字段，不能仅验证物理 descriptor。
- T/U 或不同 scope 的变量不得因同一 Object 擦除被合并；拒绝成员的 Signature 不得冒充已经发布的参数源类型。
- 保留 TypedSetter(T) 与 null、已有合法 raw→参数化赋值正例；不完整证明安全回退，并标明字段投影拒绝。
- 前置条件为同类使用清单、Signature/擦除与已恢复成员事实齐备。非目标：恢复泛型构造器、补隐含unchecked cast、外部类型层次推测、泛型语法总覆盖。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 约束泛型字段写入的源码类型证明和保守回退。

## Impact

`src/facade.rs` 的 same-run field use scan/commit 与 `src/class_source.rs` 的字段绑定证明；复用 SSA 与现有 Signature 类型，不引入新 pass/IR/依赖。原始字节码、引用与物理 identity 不变。冻结证据：`openspec/evidence/java-syntax-2026-10-08/generic-holder-patrol/`。构造器(T)覆盖独立登记，不能混入本片。
