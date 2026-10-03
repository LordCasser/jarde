## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/boxed-number-widening-patrol/README.md)：C8 BCI 63 拒绝。既有：`platform_reference_argument_widens`（表+walk）、两闭集核对脚本先例。**第一个取证义务**：javadoc/反射核对直接边全集——六数值装箱（Byte/Short/Integer/Long/Float/Double）`extends Number implements Comparable<Self>`；Boolean/Character `implements Comparable<Self>`；String `implements CharSequence, Comparable<String>`；枚举 → Comparable 由泛型擦除目标位出现频率决定是否入 MVP（默认入 `Enum`→`Comparable` 形验一形）。

## Goals / Non-Goals

**Goals:** java.lang 闭集表+walk；C8 恢复行为一致；既有闭集零回退。**Non-Goals:** 泛型参数化目标（`Comparable<Integer>` 参数化拼写——擦除位只呈现 `Comparable`，MVP 按非参数化）；用户类；`java.io`；Number→Object（既有 Object 分支）。

## Decisions

1. **表+walk 同先例**；呈现 `cast_argument` 保留要求类型。
2. **验收锚定**：C8（`larger(3,7)` 与 Double/Long 变体）+ String→Comparable/CharSequence 变体；负例（Boolean→Number 拒绝、用户类）。

## Risks / Trade-offs

- 表错对 → JDK 反射机械核对；表外回退拒绝。
