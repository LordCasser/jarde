## Why

[sif 取证](../../evidence/java-syntax-2026-10-03/single-interface-fold-patrol/sif/README.md)确认嵌套声明里程碑的**当前最高阻塞**：根类文本已含 lambda 伴生投影（lambda-inline-bodies 产物）时折叠整体拒绝——折叠 context 把 lambda/数组/枚举/初始化器投影输入全部置 `None`，重投影得到物理文本 ≠ 已投影 `root.text`，根重投影门保守拒绝（跳过则丢失 lambda 内联呈现——`Y1M` 实证退回 `lambda$…` + 合成成员重现）。Y1（lambda 根 + 单成员子）为代表的"投影根 + 折叠"族全部 no-fold；随 lambda 内联合入，corpus 中该族将持续扩大。

## What Changes

- **投影输入随报告保留**（或折叠内以同 context 重建——按取证二择一）：`ClassSourceReport` 携带折叠所需的既有投影输入（lambda 内联决策、数组/枚举/初始化器投影），折叠重投影与首轮一致——消除"重投影 ≠ 首轮"结构性差异，而非绕过门。
- 不变量：**重跑不得降级**（instance-folding 片的关键安全网）——折叠后的文本若再跑不得出现比首轮更差的呈现；lambda 内联/数组/枚举投影在折叠产物中原样保留。
- Y1 家族折叠（`interface StrFn` 嵌套 + lambda 内联呈现同时成立）、重编行为一致（`hi!`/`45`/`[b, aa]`/`8`）；无 lambda 根的既有全部家族 diff 逐字不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：带 lambda 伴生投影的根类可折叠，折叠产物保留首轮投影呈现。

## Impact

**共享契约级**：`ClassSourceReport`（或装配 context）新增折叠所需投影输入的保留（serde 兼容：新字段可选）；`src/facade.rs` 装配序；消费方为折叠通道。预算/计费沿用（输入保留不新增读取）。这是用户评估中"交接压力"（装配多路侧车）的真实冲突点——按其建议以真实冲突驱动、不新增平行机制。
