## Why

[CF-04 冻结对照](../../evidence/java-syntax-2026-09-27/cf04-ternary/report.md)显示 Jarde 已恢复普通 `?:` 与构造器 `this(...)` 条件实参，但把 `return (!a ? c : b) ? 1 : 2` 的共享整数返回整方法引用，完整类不能重编；原 class/JADX 的完整 Java 8 源码验证运行一致。区域已识别条件链和唯一消费者，现有 builder 的 1/0 布尔叶限制挡住了整数值。

## What Changes

- 在现有 `Region::ShortCircuitValue` 和 SSA 证明上，只为方法级唯一 `int ireturn`、两条直接整数常量生产路径及完整前驱/phi 使用闭包，准入嵌套条件值。
- 复用条件表达式生成，但整数返回保留 `?:` 的两个原值和求值顺序，不能套用布尔 1/0 简化；布尔返回、字段/数组/调用消费者的既有门不扩大。
- 无法证明生产者、消费者、异常边、预算或来源时继续整段原子回退，不输出少返回的表面完整类。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证嵌套条件链的整数常量尾返回恢复。

## Impact

主要在 `crates/jarde-java/src/build.rs` 的现有 `prove_short_circuit_value` 与条件值生成入口；`region.rs` 的候选结构、AST 类型及 class-source 装配无需新机制。JADX `TernaryMod` 的条件值/phi 合并和拒绝门可参考，但验证以原 class、固定 JADX 与 Jarde 完整 Java 8 源码重编、运行结果及物理来源闭合为准。任意表达式叶、异常保护、循环和 CF-05 数值转换另验。
