## Why

CF-16 登记的剩余债务 [`FinallyOnce.main`](../../evidence/java-syntax-2026-09-27/cf16-finally/full-original-main-debt.md) 已拆分取证（[证据](../../evidence/java-syntax-2026-09-30/finallyonce-main-catches/README.md)）：`main` 的具名 catch 之前是同块 void 调用语句时，`guard.rs::resources` 把该行当 TWR 候选做完整证明，`initialisation` 在前置 BCI 拒绝并阻断普通 Catches 呈现，整方法回退——这与拼接无关，`helper(); try{…}catch(E e){…}` 一类的常见形态（M5）独立复现。其次，`concat.rs::verify` 的 split 检查用块迭代序首个同 owner `toString` 充当链尾，handler 块内的拼接链（头 86，自身 `toString` 111）被误配到入口块 BCI 28 并报 `jre_concat_split`。两个根因都已用最小修改在主线复证：双修后原始 `FinallyOnce.class`（SHA `3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2`）全类零 not-recovered，既有 23 组测试全绿，store 前置与真 split 链负例行为不变。

## What Changes

- 在 `resources` 候选循环的现有门槛族（无前置指令 / handler 绑定 / 具名行前字段赋值 / 普通赋值 store）中补上缺失的回答：前置指令**不是 `Store`** 时，该行不是本规则的资源头，交回 `catches` 呈现。所有 store 前置（含 `r = open(); …` 的 TWR 降级拒绝）行为不变。
- `concat.rs::verify` 的 `jre_concat_split` 归属改为沿候选链自己的 builder 值流（allocation/dup 起，经 append 返回值的有界回溯）找真正的消费 `toString`；找不到跨块消费点时不以此码拒绝，交回 walk 自身的拒绝理由。真 split 链（跨块 builder）仍拒绝且指向正确 BCI。
- 以原始 `FinallyOnce.class` 完整类、M5 家族正例、N1/N2b 边界负例与 verifier 有效近邻做三方（原 class / 固定 JADX Java-input / Jarde）Java 8 重编和运行对照验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：同块非 store 语句之后的具名 catch 可按普通 `try/catch` 恢复；拼接链的 split 拒绝归属到链自身的值流消费点。

## Impact

仅修改 `crates/jarde-java` 私有 `guard.rs`（resources 候选门槛）与 `concat.rs`（split 归属）及其测试；不新增字节码 IR、Region/AST 节点、CLI 开关或依赖。TWR 家族（多资源、可空资源、`r = open()` 降级）与既有 finally 证书的行为与拒绝均不放宽。Test2（`recover-void-loop-finally`）改动的 `SharedFinallyCompletion` 与本变更不相交；两变更都触达 `guard.rs` 的不同函数，集成时由 root 顺序合并并全量回归。
