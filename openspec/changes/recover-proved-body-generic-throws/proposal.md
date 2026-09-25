## Why

Java 8 的 `void run() throws E {}` 可把类级异常变量保留在方法 `Signature` 中，即使物理 `Exceptions` 只写 `Exception`。当前 Jarde 与本地 JADX 1.5.6 都输出 `throws Exception`，使原 class 可编译的 `BodyThrows<RuntimeException>` 强类型调用方失败；[三方重放](../../evidence/java-syntax-2026-09-24/body-generic-throws/analysis.md)在 `-g` 和 `-g:none` 下复现。

## What Changes

- 对有正文且同轮证明为单条 `return` 的顶层类方法，在 reader 已证 `Signature` 异常后缀与物理 `Exceptions` 逐位置擦除一致、类头变量已发布时，原子恢复 `throws E`。
- 限制首片为无参数、`void`、无异常处理器、无隐含效果、直接继承 `Object` 且无接口/本类同名调用的简单方法；其余形状保留物理异常和局部拒绝。
- 冻结正反例、反射和强类型调用方对照，以及预算/取消和相邻泛型投影回归。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 有正文方法在严格同轮正文证明下保留已证类级泛型异常变量。

## Impact

复用 `jarde-reader::signature` 的解析与擦除证明、`jarde-java` 现有 class-source 候选接缝和 `src/class_source.rs` 的结构化方法声明投影。改变的仅是受控 class-source 输出；不增加 crate、外部依赖或 CLI 协议。此阶段不涵盖非空/复杂正文、方法自有异常变量、自定义异常界、继承/覆写求解及跨类调用绑定。
