## Why

[复合守护巡查](../../evidence/java-syntax-2026-10-02/compound-guard-patrol/README.md)确认：嵌套 `synchronized`（外层锁对象外、内层锁另一对象——计数器/双锁守护的真实形态）被 `monitor` 证书拒绝——单 monitor 对假设（"enter 一次 + 每路径 exit 一次"）把体内合法的**第二对** monitorenter/exit 当作外层不配对出口（T4.sync 固定复现，判别：单层 synchronized 既有恢复健康）。

## What Changes

- monitor 证书接受体内配对的第二对：外层扫描遇 monitorenter（位于外层 enter 之后、体内）且其 exit 与之配对（同锁槽、均在体内路径）时，将内层对按嵌套 `synchronized` 呈现并从外层配对计数排除；外层自身 enter/exit 配对证明不变。
- T4.sync 恢复（嵌套双锁 + 循环 + 返回）且行为一致（`10`）；单层 monitor、monitor 与 try 复合等既有形态逐字不变；不配对/跨出/三对以上（按需）保持拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：体内配对的第二对 monitor 按嵌套 synchronized 呈现。

## Impact

仅 `crates/jarde-java` 私有 guard.rs monitor 证书及呈现及测试；无新机制（嵌套判据复用配对证明）。既有 monitor/split-monitor-slot 切片零回退。
