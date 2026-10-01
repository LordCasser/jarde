## Why

[用户层级巡查](../../evidence/java-syntax-2026-10-02/user-hierarchy-widening-patrol/README.md)触发此前两闭集切片（throwable-wrap、collection-widening）登记的升级路径：用户实现类传给用户接口/父类形参（`viaInterface(new En(), …)`）无证据通道，行为级失败（输出缺语句）。同快照内 T、U 均为物理类时，`T.class` header 自带 extends/interfaces 链——层级事实来自被分析工件自身字节，无需 classpath 猜测。这是"策略/回调/实现注入"类反编译最高频形态之一。

## What Changes

- 新增按 BCI 的 widening 证明集合（与 `reference_overload_calls` 同款通道形态）：实参呈现类型 T、要求类型 U **均在本快照有物理类定义**时，沿 T 的 extends 链与 interfaces（含超接口，有界深度/步数）walk 达 U 即证；证明由有环境访问的 pass 收集，build.rs 分派处消费，`cast_argument` 保留要求类型拼写。
- I1 家族（`viaInterface(new En(), …)` 等）完整恢复且行为一致；两平台闭集、overload 证明、全部既有负例零回退。
- 单边快照外（T 或 U 不在快照——含用户类→平台目标如 Throwable/List）不在 MVP，保持拒绝并登记（触发条件：下一真实案例）。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：同快照双物理类定义的用户层级实参上转型可呈现，用户实现传接口/父类形参完整恢复。

## Impact

`crates/jarde-java`（证明收集 pass 与 build.rs 分派消费）与 `src`（如 pass 装配点在根 crate）及测试；复用既有 per-BCI 证明通道形态与类 header 读取（member-family/enum 家族先例），无新跨类机制（只读本快照）。A16 单类读取预算沿用（按需 header 读计费）。
