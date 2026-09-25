## Why

Java 8 的引用类型和数组类字面量由 `ldc Class` 产生；当前解码把该常量归为 `Other`，五个真实方法缺少返回或调用语句，使完整类不可编译。池项已经陈述类型，现有表达式链只缺一个受限的字面量形状。

## What Changes

- 在已验证的 `ldc`/`ldc_w` Class 常量位置恢复引用类、数组类和本类的 `T.class`，保留返回与调用参数位置的一次求值、静态类型及真实来源。
- 用现有类型拼写与名称避让规则拒绝非法/有歧义的池名；不放行 MethodType、MethodHandle 或未知常量。
- 保持已有基本类型/void 的 `TYPE` 字段输出及独立浮点常量任务，本项不合并两种字节码机制。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：真实 Class 常量可恢复为语义相同且可编译的 Java 类字面量。

## Impact

主要影响 jarde-java 的常量解码、现有表达式类型/发射和来源测试，无 reader/SSA/依赖变更。七项独立运行证据在 `../../evidence/java-syntax-2026-09-22/class-literals/`。这把父变更 `present-proved-java-structure` 2c.5 的 Class 部分拆成最小闭环；浮点部分不在本项。
