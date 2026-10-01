## Context

[巡查证据](../../evidence/java-syntax-2026-10-01/em19-bitops-patrol/README.md)：B5.s1 字节码 = 短路链（3–16）→ `istore_1`@17 → 拼接（18–37，`append:(Z)`@26）。既有 `recover-short-circuit-local-values`（见其 verification.md）建立"短路值存局部 + SSA 值证明 + 消费方呈现"，消费方目前覆盖 return；错误码 "reaches a shared value consumer … no SSA proof for that value" 即消费方未匹配时的拒绝路径。第一个取证义务：读该切片实现（`crates/jarde-java/src` 的 ShortCircuitValue 呈现与消费方枚举），确认消费方判定的确切位置与拼接链实参的消费形态（append(Z) vs 装箱 append(Object)——javac 8 对 `hasA + ":"` 的实际 lowering 以 javap 为准）。

## Goals / Non-Goals

**Goals:** 消费方集合纳入拼接 append 位（按取证的实际 descriptor）；B5/B4/B2 固定形态全恢复、行为一致；B3/s3 逐字不变。**Non-Goals:** 非拼接的复杂消费（算术/嵌套调用实参）——出现时登记；短路链共享消费者语义变更；布尔装箱的显式 cast 拼写新规则（按既有装箱通道）。

## Decisions

1. **消费方判定沿用切片既有结构**：在消费方枚举处增加拼接 append 匹配（descriptor 取证后精确写）；值的呈现走既有布尔拼写；装箱差异若拼接两形态 lowering 不同，分别匹配并测试钉死。
2. **验收锚定**：判别链三档（B3/s1/s3）diff 断言前两档不变、第三档翻正；B2.compound（复合位赋值前缀 + 双短路 + 拼接）与 B4 全恢复；负例（值在拼接前又被其它读消费）保持退化。

## Risks / Trade-offs

- **拼接实参位与其它 append 混淆** → 匹配以"该局部加载恰好是 append 的实参位"为准，与既有实参判定同一纪律。
- **装箱呈现** → 按既有装箱通道；若既有通道无布尔装箱先例，以 `(java.lang.Object) local` 显式拼写保守呈现并在测试钉死。
