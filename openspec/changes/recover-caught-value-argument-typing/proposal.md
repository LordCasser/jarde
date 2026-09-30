## Why

[参数槽复用巡查](../../evidence/java-syntax-2026-10-01/catch-param-slot-reuse/README.md)确认：catch 参数复用前序语句的局部槽（javac 对 `try (r) { … } catch (E e)` 的常规降低）时，handler 体内把参数传给形参收紧方法的实参呈现沿用**槽的类型决策**（资源类型 `S1`），与该读取的 SSA 值定义（catch 绑定 store，`Definition::Caught`，行类型 `IllegalStateException`）不符，触发 "no safe reference conversion evidence" 拒绝（S1.twrHelper 固定复现；无槽复用的普通 catch 对照完整恢复）。这是呈现归属错误，不是证明缺口：子句头本就以行类型正确拼写参数。

## What Changes

- handler 体内对 catch 参数槽的读取，实参呈现类型改为跟随值定义：读到的 SSA 值定义为绑定 store（Caught）时，呈现该值的行类型（与子句头拼写一致），不回退槽决策。
- 落点在实参/读取呈现的类型来源（build.rs），复用 `Definition::Caught` 既有的类型事实通道；无槽复用场景（对照）输出零变化。
- 以 S1 固定类、多种子句类型（具名/多 catch 各型）与既有 17b 家族回归验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：catch 参数槽复用时 handler 体内的参数消费按行类型呈现，传参调用完整恢复。

## Impact

仅 `crates/jarde-java` 私有 build.rs 呈现类型归属及测试；不新增机制；17b/typing/Catches 既有路径零回退。
