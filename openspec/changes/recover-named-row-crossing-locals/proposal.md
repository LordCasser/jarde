## Why

[CF-15 扩验巡查](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/README.md)确认两个高频形状整方法回退：`Acceptor a = new Acceptor(); try { a.foo(); } catch (NamedE e) { … } continue-using-a`（C1.five/C4.constructNamed 家族）——(1) `guard.rs::resources` 门槛把"具名行 + 完成 store 前置"当 TWR 资源头做完整证明，`jre_guard_handler` 拒绝后阻断 Catches 呈现；(2) 即便交回 Catches（实验证实诊断前进），`build.rs::all_reads_reach_presented_writes` 的写值白名单只收 int 内联树，构造 store 直接 `DeclarationPlacement::Incomplete`。结构判别已实证：真 TWR 的具名行覆盖资源初始化（起点在构造前，如 `[0,36)→39`）且 TWR 自身保护行恒为 catch-all，**起点紧跟完成 store 语句的具名行不可能是 TWR 自己的保护行**。

## What Changes

- 门槛补第三种回答：`before` 为 store、行起点在语句边界（`statement_boundary`）、行为**具名类型**（`catch_type_index.is_some()`）时不当资源头，交回 `catches`。N1/P3StorePrefix 的"store 前置降级"负例按此结构论证翻转为正例；catch-all 行的降级、劈开/吞初始化/交叠呈现等全部既有负边界保持不变。
- `all_reads_reach_presented_writes` 的写值接受集扩展：同块、单用途、已验证构造点（`new`+init 恰在 store 前完成）作为可呈现写值——声明以构造为初始化器（复用既有 new-site 呈现），不为 finally 家族再增豁免旗标。
- 以 C1/C4.constructNamed 固定类、翻转后的 N1/P3StorePrefix、C3 对照组与 TWR 家族回归验收；`twrNamed`（真 TWR + 外层具名，CF-17 另案）不得有行为变化（保持当前拒绝）。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：具名 catch 行前的完成构造/调用 store 不再按 TWR 降级；跨该 protected region 的构造定义局部可按声明-初始化器呈现。

## Impact

仅 `crates/jarde-java` 私有 `guard.rs`（resources 门槛）与 `build.rs`（写值接受集）及测试；N1/P3StorePrefix 期望翻转需同步其测试与 patrol/preceded 证据说明。不新增 Shape/公开 IR/CLI/依赖；catch-all 降级语义一字不动。
