## Why

合法 class 的 `ireturn` 在方法返回描述符为 `Z` 时只保留整数最低位。当前 Jarde 对没有 boolean 类型证明的整数操作数保守拒绝，因此 `integerAsBoolean(I)Z` 等可表达的返回仍无法成为可重编译的 Java；把它写成 `value != 0` 又会在输入 2、-2 等处改变原 JVM 行为。

## What Changes

- 在真实 `ireturn` 且本方法返回 `Z` 的消费位置，对已完整呈现为 B/C/S/I 的值生成等价的最低位布尔表达式；普通、已有 switch 下推、同步与已证明字段自增返回共用该规则。
- 保留现有已证明 boolean 值与 0/1 字面量的直接拼写；未知、不可呈现或非整数值仍局部拒绝，并保留操作数与返回指令来源。
- 固定 Java 8 合法补丁类、原 JVM/JADX/Jarde 三方阶段与完整类执行对照，验证奇偶、负数、极值、调用次数、字段更新、同步及来源/预算契约。

前提是现有 Java 8 reader、SSA/Region/AST 返回路径以及已完成的窄整数返回和 boolean 字段写入规则。此次不恢复条件 stack phi、boolean 数组写入或一般局部 boolean 推断；不把 `(Z)B/C/S` 的非规范 raw 值转成 Java boolean 再数值化，也不承诺复原原源码写法。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加 `ireturn Z` 对可呈现整数值的最低位转换、结构化返回和保守边界的可观察要求。

## Impact

主要涉及 `crates/jarde-java/src/build.rs` 的现有返回消费与布尔表达式构造、Java 8 永久 fixture/Rust 回归和 OpenSpec 三方证据。复用既有 AST、来源、预算及停止规则，不改变公开 API、reader/JVM IR、依赖或 CLI 架构。
