## Why

[CF-12 固定三方对照](../../evidence/java-syntax-2026-09-27/cf12-integer-switch/report.md)中，整数 switch 的合并标签、fall-through、无 default 与运行结果都已正确，但原源码及固定 JADX 的 `case LOW` / `return HIGH` 被 Jarde 写成 `case 2748` / `return 3294`。当前类级输出已经从字段的 `ConstantValue` 写出 `static final int LOW/HIGH`，这份同次物理证据尚未用于对应的源码表达式。

## What Changes

- 仅在完整类源码恢复中，对本类唯一、源级可引用的 `static final int` `ConstantValue` 字段，投影同值整数 switch 标签及该 case 的直接整数返回为字段名；保留原始整数 key、BCI 和物理字段记录。
- 候选字段同值歧义、名称冲突、属性不完整、方法不是可安全投影的已恢复 AST 时保留现有数字拼写；不凭数值推断实际原源码一定引用字段。
- 以 CF-12 的完整 Java 8 原/JADX/Jarde 类重编、11 行验证运行、标签/返回源码及负例、预算/取消验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证同类整数常量字段可作为 switch case 和该 case 直接返回的源级别名。

## Impact

使用 `src/facade.rs` / `src/class_source.rs` 已有的类字段 `ConstantValue` 读取与 `crates/jarde-java` 的同次 AST 侧车、switch 标签及表达式输出；不增加 crate、全局常量搜索或新的字节码扫描机制。方法级恢复在没有完整类字段证据时继续输出数值。更广的整方法常量替换、跨类/继承字段、资源 ID、字符串/枚举 switch 与字段重命名不在本次范围。
