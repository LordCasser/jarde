## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/twr-clause-patrol/README.md)：P3.voidBodySoloFin 的 BCI 序（TWR：close 正常@13/异常@21+suppression@30+athrow@34；finally：正常副本@35-44 goto、异常副本@47-58 athrow；汇合@59 后续语句）。twr-inner-finally 切片已建：finally-copy 抢占的**延迟保留**（无 TWR 认领时才原样拒绝）、`inner_finally_parts`/`place_inner_finally` 子证书、融合继续块/trail 呈现。第一个取证义务：确认 direct 形（finally 在 TWR 语句**之后**而非体内）与 inner 形的判据差异——direct 的 finally 副本锚定在 TWR 全表（含 close 链）结束之后，inner 的锚定在体内资源 init 之间；读取 `place_inner_finally` 的锚定逻辑判断是复用同一函数加"锚点=TWR 表尾"分支，还是平行的尾随子证书。

## Goals / Non-Goals

**Goals:** direct 形恢复（P3/P1 双体形）；既有 inner 形与纯两域逐字不变。**Non-Goals:** TWR+finally+catch 三子句；finally 内抛错重排（忠实序）；多层 TWR 各自带 finally（验一形登记，不泛化）；B 形态（catch 体调用语句——下一片）。

## Decisions

1. **尾随子证书**：TWR 证书主体证明后，扫紧随的 finally 降低表（两副本中段同形、区间连续、与资源行不交叠、正常副本入口恰为 TWR 正常出口的 goto 目标）；按既有 `cleanup_sequence` 副本等价复用判定。锚点实现按取证 1.1 的结论选（复用 `place_inner_finally` 加表尾锚 vs 平行函数），原则：一份副本判据。
2. **呈现**：`try (…) { … } finally { … }`——finally 体走既有 finally 呈现；TWR 头不变。
3. **验收锚定**：P3（`t[c]f`）+ P1（`b[c]f`）+ 变体（finally 含 return、多层 TWR 外层带 finally 登记现状）；负例（副本断链/交叠）整方法拒绝。

## Risks / Trade-offs

- **与 inner 形判据混淆** → 锚点互斥（表尾 vs 体内），测试双向钉死。
- **17b 包围行（catch around TWR）与本子句共存** → 三子句形态 Non-Goal，遇则拒绝并登记。
