## Why

[复合守护巡查](../../evidence/java-syntax-2026-10-02/compound-guard-patrol/README.md)确认：TWR 正文内显式 `finally`（`try (a) { try (b) { body } finally { mid } }`——资源清理与手动清理叠加的真实形态）整方法拒绝——内层 finally 的副本候选不满足 `prove_finally_copy` 的完整直体形（其清理已并入 TWR 降低、正文属 TWR 体），guard 家族证书两两独立无复合通道（T4.nested 固定复现；判别：纯 TWR、纯 finally 各自健康）。

## What Changes

- TWR 证书的正文证明接受"内层显式 finally"子形态：TWR 体内出现内层 finally 副本候选时，在内层（TWR 正文）语境下证明该副本（内层两行 catch-all + 中段清理在正文内自洽），呈现 `try (…) { try (…) { … } finally { … } }`。
- T4.nested 恢复（含 suppression 序 `body[b]mid[a]`）；纯 TWR、纯 finally（含各家族证书）逐字不变；内层副本不自洽（清理缺/跨界）整方法保持拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：TWR 正文内的显式 finally 块按嵌套 try/finally 呈现。

## Impact

仅 `crates/jarde-java` 私有 guard.rs（TWR 正文证明接受内层 finally 子证书）与呈现及测试；复用 finally_copy 判据在正文语境，无新机制。既有 TWR/finally 全家族零回退。
