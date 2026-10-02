## Why

[非静态折叠巡查](../../evidence/java-syntax-2026-10-03/inner-class-folding-patrol/README.md)确认混合族缺口：非静态成员（Inner）混入后整族走 `prepared` 分离路径——上片 `StaticMembers` 多子化只对**纯静态族**生效，N1 的静态 Stat 也不折叠。真实代码的辅助/嵌套类族常见静非混合（Builder 静态 + Listener 非静态），第一层 MVP 先把族内静态子集折叠（机械全复用上片，零新证明），非静态层（this$0 消参/限定 new/消桥）留第二片。

## What Changes

- 家族扫描：直接静态成员子集独立收集折叠（`StaticMembers` 判定扩为族内子集），非静态候选仍走既有分离路径不受影响；投影装配序（枚举/注解/静态折叠/其余）保持。
- N1 的 `static class Stat` 折叠呈现（域内引用源码拼写）；`class Inner` 保持分离（本片边界，登记第二片）；纯静态族（M1/M2）与全部既有通道逐字不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：混合族中的直接静态成员类按嵌套声明折叠呈现。

## Impact

仅 `src/member_inner.rs`（扫描子集化）与投影装配及测试；复用上片全部机械。既有家族/投影通道零回退。
