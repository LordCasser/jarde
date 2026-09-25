## Why

[新三方探针](../../evidence/java-syntax-2026-09-25/instanceof-boolean-merge/analysis.md)证明 Jarde 已正确恢复 `instanceof` 的 `0/1` 汇合，完整类八行与原 class/JADX 一致；但它把 `return !(value(x) instanceof String)` 写成 `return (value(x) instanceof String ? 0 : 1) % 2 != 0`。这对一般整数返回布尔值是必要的保真策略，对已证明两臂恰为 0/1 的候选则降低了源码可读性。

## What Changes

- 仅在已有条件值证明与真实 `ireturn Z` 消费处，精确证明两臂分别为常量 0/1、分支极性、Boolean 测试、唯一消费及完整来源时，输出原测试或 `!` 测试。
- 其它整数，包括 verifier 有效的 2/3、非恒定臂、额外消费者或不闭合控制流，继续使用现有低位适配或保守引用；不改变 JVM 行为、预算和停止契约。
- 冻结完整 Java 8 类的八行值/调用次数及来源，对比原 class、JADX 和 Jarde。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明的 0/1 条件值在布尔返回处可呈现为对应的 Boolean 测试表达式，同时保留一般整数低位语义。

## Impact

只扩展 `jarde-java` 现有条件值构造与返回消费接缝及定向测试，不新增公开 AST/IR/pass，不改变基础 `instanceof` 操作、JVM 解码或非布尔消费者。此项是源码质量改进，与在途实例字段和命名 catch 行为修复独立。
