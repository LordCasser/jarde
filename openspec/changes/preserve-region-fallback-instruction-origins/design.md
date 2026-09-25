## Context

`Region::Fallback` 持有 canonical 块身份，`Builder::region` 通过 `covered_bcis` 把这些块转为拒绝来源，并追加 `FallbackReason::unaccounted()`。当前 `covered_bcis` 调用 `CanonicalBlock::blocks()`，但 canonical 层文档明确它是融合节点所代表的原始**块起点**，不是块内指令起点。合法 handler-boundary 样本中，块内 BCI 3–12 已存在于解码/SSA，却没有出现在 source map。

## Goals / Non-Goals

**Goals:** 拒绝的物理来源准确覆盖该 Region 已解码且实际归属的每条指令；保持稳定排序和去重，不凭 BCI 范围猜测不存在的指令；满足现有预算、取消和部分产物契约。

**Non-Goals:** 让跨异常表范围的候选被恢复为 Java `try`，修改 CFG/handler 根策略、给未解码字节虚构来源，或改动后置 `++` 的 SSA 规则。

## Decisions to Verify Before Implementation

1. 从 `CanonicalBlockId` 找到同一路径的 SSA 指令，核对其 BCI 是否完整覆盖 Region 所指的块；若 SSA 因不可达或部分分析不发布某块，先追溯解码 Code、原始 CFG 与 canonical 的覆盖关系，再决定可靠的回退来源。不能把 `start..end_bci` 的每个数字当指令。
2. 在 Region fallback 现有 `covered_bcis` 边界内修复；物理 BCI 相同的克隆来源可按来源模型去重，但必须确认不混淆不同 `jsr` 路径的拒绝覆盖。保持 `unaccounted()` 的补充语义。
3. 候选指令枚举和去重计入已有预算并轮询取消；停止时使用现有不交付半个成功来源的契约。默认/完整证据可改变来源段是否物化，不能改变正文或拒绝代码。

## Risks / Trade-offs

- SSA 只含可达路径时直接按 SSA 枚举会漏掉 Region 指名的不可达块；先用普通、不可达、融合和克隆样本核对覆盖，再确定组合数据源。
- 以连续 BCI 范围补全会把多字节操作数、未解码尾部误当作指令；只能使用 reader/IR 已验证的指令起点。
- 新的每指令来源会提高来源成本；需在真实样本上测预算维度和停止行为，不引入无界全方法扫描。
