## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/boxed-number-widening-patrol/README.md)：C8 BCI 63 拒绝。既有：`platform_reference_argument_widens`（表+walk）、两闭集核对脚本先例。**第一个取证义务**：javadoc/反射核对直接边全集——六数值装箱（Byte/Short/Integer/Long/Float/Double）`extends Number implements Comparable<Self>`；Boolean/Character `implements Comparable<Self>`；String `implements CharSequence, Comparable<String>`；枚举 → Comparable 由泛型擦除目标位出现频率决定是否入 MVP（默认入 `Enum`→`Comparable` 形验一形）。

### 与既有 change 的关系（立项查重，2026-10-04）

widening 域已验收片经逐读确认与本片**互补而非重复**，区分点是"类型证据的来源通道"：

| 已验收片 | 证据通道 | 为何不覆盖本片 |
| --- | --- | --- |
| `recover-snapshot-hierarchy-widening`（7/7） | **本快照物理类定义**：要求 T 与 U 均在快照内有定义，沿 extends 链与 interfaces（含超接口，有界深度）walk | `Integer`/`Number`/`Comparable` 是 **java.lang 平台类型，不在快照内** → 该片 walk 无从起步；本片走的正是它明确留下的"两平台闭集"通道 |
| `preserve-array-invocation-widening`（8/8） | 数组形参的实参上转型 | 域不同（数组 vs 装箱标量） |
| `recover-platform-collection-widening`（7/7） | 平台集合类型闭集 | 闭集内容为集合家族，不含 java.lang 数值装箱家族边 |
| `prove-narrow-numeric-conditional-arguments`（5/5） | 三元/条件位置的窄数值实参 | 消费位置不同 |
| `preserve-list-iterable-invocation-widening`（4/4）、`preserve-invocation-argument-types` | List/Iterable 与参数类型保留 | 同上，域不同 |

故本片是对既有 `platform_reference_argument_widens` 闭集的**内容扩展**（补 java.lang 装箱家族边），不新建通道、不与 snapshot walk 交叠——两者的分界即"类型是否在快照内有物理定义"，实现时不得把平台边塞进 snapshot walk（那会让闭集失去可核对性）。

## Goals / Non-Goals

**Goals:** java.lang 闭集表+walk；C8 恢复行为一致；既有闭集零回退。**Non-Goals:** 泛型参数化目标（`Comparable<Integer>` 参数化拼写——擦除位只呈现 `Comparable`，MVP 按非参数化）；用户类；`java.io`；Number→Object（既有 Object 分支）。

## Decisions

1. **表+walk 同先例**；呈现 `cast_argument` 保留要求类型。
2. **验收锚定**：C8（`larger(3,7)` 与 Double/Long 变体）+ String→Comparable/CharSequence 变体；负例（Boolean→Number 拒绝、用户类）。

## Risks / Trade-offs

- 表错对 → JDK 反射机械核对；表外回退拒绝。
