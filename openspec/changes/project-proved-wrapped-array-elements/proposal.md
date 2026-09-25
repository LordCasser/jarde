## Why

已验收的数组增强 `for` 只接受循环体入口的独立 `T value = array[i]`，因此 `array[i] + tick()` 等可安全投影的表达式仍写计数循环。[三方执行证据](../../evidence/java-syntax-2026-09-24/array-wrapped-binding/analysis.md)同时发现 JADX 将元素读取提前到副作用之前，令三条合法方法重编后改变结果；扩展 Jarde 时必须把求值顺序作为准入证明。

## What Changes

- 在既有数组循环、SSA 与增强 `for` AST 上，识别唯一元素读取位于表达式子树中的窄形状；只在它之前没有可观察操作、类型／局部用途／异常边均闭合时，把该读取变成每轮元素绑定。
- 以局部 AST 替换保留读取之后的原表达式与调用；不能证明时保留现有可执行计数循环，不复制 JADX 的提前读取行为。
- 固定 `array[i] + tick()`、`tick() + array[i]`、写数组调用、调用实参、同轮两次读取、null 与异常路径的 Java 8 三方执行对照，并复核来源与预算。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 扩展已证明数组增强 `for` 的元素绑定位置，同时要求语义等价的求值顺序和保守拒绝。

## Impact

仅涉及 `jarde-java` 的现有循环候选、AST 来源与发射路径及定向测试；不增加 crate、全局表达式重写 pass 或依赖。`Iterable` 投影、通用副作用／别名分析和结构基底中仍为 explanation-only 的数组反例属于其它变更，不作为本变更前置完成项。实现可在 `project-proved-enhanced-for-loops` 的数组 1–3 已验收后独立开展；共享 `build.rs` 的并行编辑须错开验收。
