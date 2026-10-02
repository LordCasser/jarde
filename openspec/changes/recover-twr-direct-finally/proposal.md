## Why

[TWR 子句巡查](../../evidence/java-syntax-2026-10-02/twr-clause-patrol/README.md)确认：`try (r) { … } finally { … }`（TWR 语句直接携带 finally 子句）整方法拒绝——TWR 降低（close 正常/异常副本 + suppression）与 finally 降低（正常/异常副本）**串联**在同一方法，任一现有证书单独不拥有全表，finally 副本候选先抢占失败（`jre_guard_finally_copy`）。P3（void 体）/P1（复合体）双形同败；TWR 体与调用分支、17b catch 形态各自健康（判别钉死）。

## What Changes

- TWR 证书接受**尾随 finally 子证书**：TWR 降低表完整后，若其后紧跟 finally 降低表（正常副本在 close 链后 goto 汇合/异常副本 astore+athrow、两副本中段语句逐指令同形、区间连续不与资源 suppression 交叠），则 TWR 证书一并拥有；呈现 `try (…) { … } finally { … }`（finally 体复用既有呈现）。
- P3.voidBodySoloFin / P1.soloFinally 恢复且行为一致；纯 TWR、纯 finally、twr-inner-finally（嵌套形）逐字不变；串联不自洽（副本不同形/交叠/断链）整方法保持拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：TWR 语句直接携带的 finally 子句按 `try (…) { … } finally { … }` 呈现。

## Impact

仅 `crates/jarde-java` 私有 guard.rs TWR 证书（尾随子证书）与呈现及测试；复用 finally_copy 副本判据与 twr-inner-finally 的延迟保留机制，无新机制。既有 TWR/finally 全家族零回退。
