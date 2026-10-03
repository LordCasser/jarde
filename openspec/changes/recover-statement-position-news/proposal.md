## Why

[语句位构造巡查](../../evidence/java-syntax-2026-10-03/statement-new-patrol/README.md)确认：语句位 `new X(args);`（结果不被消费）四形整方法拒绝——`refuse-unconsumed-construction-invokes` 为 CST 序保真收紧的边界在**无序敏感实参**形上过宽（无参/常量参/已证消费链参的呈现即逐字忠实）。B5.main 形的 ctor 副作用（println）在剥离引注后静默丢失——可编译但行为差的同一风险族。

## What Changes

- 语句位构造按表达式语句呈现：既有 new@1 构造证明（ctor 解析、实参既有通道）通过 ∧ 实参为常量/局部读/已证消费链（**无实参位真实调用**）→ `new X(args);`。
- 原切片保护的 CST 形（实参含真实调用的语句位 new，其冻结反例）保持拒绝逐字不变；B5.main/B6 四形恢复且行为一致；消费位构造零回退。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：无序敏感实参的语句位构造按表达式语句呈现，方法行为完整。

## Impact

`crates/jarde-java` init/build 语句位判定及测试；无新机制。既有 new@1/消费位通道零回退。
