## 1. 所有权与依赖事实

- [x] 1.1 审计 Region::blocks 在 loop、switch fallthrough、try/catch、`jsr` 克隆中的合法共享语义；用 canonical 身份精确统计额外入口样例的重复 owner 与其首次触发位置。验证：只重复相同 block 身份才触发，单纯同 BCI 异路径不触发。
- [x] 1.2 先验收 `preserve-region-fallback-instruction-origins` 对额外入口的 BCI 8/26/29 与 PostfixHandlerBoundary BCI 3–12；若依赖尚未闭合，不声称本 change 已满足完整来源要求。

## 2. 原子拒绝

- [x] 2.1 在方法 Region walk 后、任何 Java 发射前，以预算化检查发现重复 owner，改用已有 whole-body fallback 并明确诊断所有权冲突；不新增语法 AST 或条件合并 pass。验证：冻结 ChainExtraBoundary 不再重复 BCI 25 或报告四个 `jre_region_loop`，无结构化 `putstatic`。
- [x] 2.2 保留 P3-R7 无 canonical 指令的独立引用及既有预算/取消停止合同。验证：quote/source map 覆盖所有真实已解码 live BCI 与未入图指令；低预算/取消不发布部分结果。

## 3. 独立验收

- [x] 3.1 原 class、JADX、Jarde 完整类分别 Java 8 重编并在 JVM 验证下对照 32 条路径；明确 JADX 的 16 条差异与 Jarde fallback 的非等价语义，不把可编译或引用文本当作正向恢复。复跑已证明的两/三测试纯链、loop/switch/try/jsr 相邻测试。
- [ ] 3.2 运行格式、相关测试、适用 Clippy、`openspec validate refuse-overlapping-region-ownership --strict`，记录真实通过/失败、字段 `presented` 报告债及磁盘清理；只有证据完备时勾选。
