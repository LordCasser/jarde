## Why

JADX 的 Java 集成测试覆盖空枚举、单常量及四常量普通枚举；Jarde 现有类级枚举证明固定为两个常量，且要求每个常量带源级 `int` 构造实参。已实测的空枚举 `EmptyMood` 在原始与 JADX 完整源码中按 Java 8 重编运行输出 `count=0`，Jarde 完整源码却在 `$VALUES` 字段处报 `enum constant expected here`。DT-10 因此仍未达到 JADX 的基本枚举语法覆盖。

## What Changes

- 将普通 Java 8 枚举的类级证明推广到有界的 0、1、4 常量及现有两常量正例：逐项核对字段顺序、构造点、隐式 name/ordinal、`$VALUES`、`values()`、`valueOf()` 和所有物理方法的额外使用。
- 接纳仅含 JVM 注入参数、无源级构造实参和无用户初始化效果的平凡枚举构造器；由已证事实写出合法的空/普通枚举常量列表，保留用户方法和物理成员报告。
- 任一成员、Code、来源、预算或使用关系证明不全时整组拒绝投影；已有带源整数参数、委托构造器及匿名常量体的证明不得退化。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在完整类级证明下恢复普通 Java 8 枚举的零个及多个常量和隐式无源参数构造器，并保持可编译、行为与物理来源。

## Impact

主要涉及 `src/enum_constants.rs` 的普通枚举证明、`src/facade.rs` 的同次候选采集以及 `src/class_source.rs` 的类源码装配；复用 reader、方法恢复、预算和现有类级投影，不引入 crate、依赖或公开来源 schema。`recover-proved-enum-constants` 的两常量证书与 `recover-proved-enum-constant-bodies` 的匿名常量体是回归边界，不扩大这两项的原定构造参数或常量体范围。
