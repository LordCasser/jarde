## Why

当前 Jarde 对无正文方法只要发现方法自有类型参数就跳过整个 `Signature`，导致 `<X extends Exception> void raise() throws X` 退化为 `void raise() throws Exception`：原本合法的泛型覆写类无法编译，反射也丢失方法类型参数。JADX 1.5.6 保留 `<X>`，却仍把 `throws X` 写成 `throws Exception`，使强类型调用方失败；[三方对照](../../evidence/java-syntax-2026-09-24/method-local-generic-throws/analysis.md)说明这里需要比 JADX 更完整的投影。

## What Changes

- 在无正文方法上，从已证方法 `Signature` 完整恢复可拼写的方法自有类型参数、参数、返回及泛型异常位置；保留物理 descriptor 和 `Exceptions` 的逐位置擦除约束。
- 只在顶层类或接口的层次事实及同类调用关系足以排除未证明的覆写/调用绑定冲突时发布；局部签名或异常上界无法证明时保留物理声明及拒绝。
- 以 Java 8 原/JADX/Jarde 的泛型覆写、显式异常实参调用、方法反射、`-g`/`-g:none` 和 verifier-valid 负例验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：无正文方法自身的已证泛型声明进入完整类源码，保持泛型调用方、覆写关系及反射信息。

## Impact

复用 `jarde-reader::signature` 唯一 parser/作用域/擦除证明与现有类源码投影接缝；主要影响 `src/class_source.rs` 和传入类层次事实的 `src/facade.rs`。不修改有正文方法的泛型推断，不新增 pass、crate 或第三方依赖；缺失父类、复杂继承契约及有正文的异常控制流留作独立证据和任务。
