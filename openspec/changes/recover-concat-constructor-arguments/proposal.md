## Why

Java 8 的 `new ArithmeticException("arm-" + label)` 在抛出和返回位置都可由原 class、JADX 重编源码正常执行，但 Jarde 将已证明的内层字符串拼接分配当作外层构造的独立插入效果，导致 `new` 与后续消费者一同回退。冻结的[最小矩阵](../../evidence/java-syntax-2026-09-26/exception-constructor-concat/analysis.md)排除了单独拼接、普通构造实参和 `athrow` 本身，指向两个已有证明计划之间的组合缺口。

## What Changes

- 允许一个外层构造实参依赖中完整已证的字符串拼接链嵌入 `new` 表达式，并保持外层分配、拼接求值、构造器调用和最终消费的 JVM 顺序。
- 对链的真实参数身份、唯一消费、物理范围、异常处理范围和所有权进行闭合核对；不接受只因某 BCI 被其它规则标记为 reserved 的任意分配或效果。
- 用构造对象的返回与抛出两个消费位置进行完整类 Java 8 重编和三方执行对照；独立 void 效果、别名/多消费和边界跨越继续拒绝并保留来源。

本项以前置的 `concat@1` 链证明与 `new@1` 构造证明为基础，不创建通用嵌套分配求解器，也不修改 `throw` 的规则。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证明拼接链可作为普通对象构造器实参，且构造结果可由已有消费者呈现。

## Impact

主要在 `jarde-java` 的 `concat::Plan` 到 `init::sites` 私有证明交接、构造实参依赖核对与定向测试；现有 AST、发射器、报告来源和 `throw`/`return` 消费位置复用。无公开 API、持久格式、reader/SSA、外部生产依赖变更。
