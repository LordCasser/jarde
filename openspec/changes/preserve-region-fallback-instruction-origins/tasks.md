## 1. 确认可靠的指令归属

- [x] 1.1 用永久 `PostfixHandlerBoundary` 样本追踪 Region fallback、canonical 块、SSA、原始 CFG 和 Code 的 BCI 集合，明确 BCI 3–12 在哪一层丢失；核对 `covered_bcis` 的唯一/其他调用者。
- [ ] 1.2 固定普通、融合、`jsr` 克隆、不可达与部分解码样本；记录每个拒绝块真实指令起点及当前来源差值，先验证证据层可用范围。[额外入口样例](../../evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/analysis.md)须作为普通 Java 8 负例，核对 BCI 8/26/29；其所有权与字段报告债务不归本任务。

## 2. 最小修复

- [x] 2.1 仅在 Region fallback 来源采集路径上，以已证明同一 canonical 块的已解码指令替代块起点列表；保留 `unaccounted()`，稳定去重和路径归属，不改变拒绝理由或结构选择。
- [x] 2.2 计入已有预算并轮询取消；对无完整指令证据的块保守报告已知起点，不虚构字节码。

## 3. 独立验收

- [ ] 3.1 对 1.2 全部样本验证 source map 精确覆盖，默认/完整正文一致，来源预算及取消时无伪完整报告。
- [ ] 3.2 复跑 handler-boundary 的原 class 与 JADX 执行对照及相邻 Region/来源回归，核对 OpenSpec strict、受影响 Clippy、磁盘占用；将与本变更无关的图策略债务继续单列。
