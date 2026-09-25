# 阶段验收（2026-09-25）

`Region::Fallback` 现在从该 canonical 身份的 SSA 指令枚举真实 BCI；无 SSA 且仅一个原始块时从同一 Code 的已解码起点与精确块终点枚举；无 SSA 的融合块只列已知起点。逐候选计费和轮询取消，`UnaccountedInstructions` 的独立来源仍合并、排序、去重。其它调用 `covered_bcis` 的结构证明保持原口径。

主代理复跑 `p3_postfix_handler_boundary`，拒绝仍为 `jre_guard_resource_init` 与 `jre_region_uncovered_blocks`，`update` 的 BCI 0、3、6–13 都在 source map。额外入口原 Region 树的单块补上 BCI 26；与独立的[所有权拒绝](../refuse-overlapping-region-ownership/verification.md)合并后，whole-body quote 覆盖包括此前缺失的 8、26、29 在内全部 16 个已解码 BCI，默认/完整证据正文一致。额外入口低 `AnalysisSteps` 回归不发布伪完整结果。root 的 `jarde-java --lib` 163/163 与邻近 root 测试通过。

本 change 的任务 1.2/3.1 尚未把普通、融合、`jsr` 克隆、不可达和部分解码五类各自的精确指令集逐样本固定，故保持未勾；不能把当前两个真实样例的联合通过说成全范围已验收。`openspec validate preserve-region-fallback-instruction-origins --strict` 已通过，适用 Clippy 被工作树其它既有告警阻断。

后续只读审计把尚缺的端到端证据进一步缩小：普通 Java 8 额外入口样例已在所有权拒绝测试中逐一断言 16 个 Code 指令起点；融合块的 canonical 单测只证明原始块起点 `{7,12}` 与 Code 子程序指令 `{7,8,9,12}`，尚未进 Java fallback source map；ECJ v45 `HistoricalControlFlow.finallyPath` 已证明物理 BCI 17 有路径 `[5]` 与 `[12]` 两份 canonical 克隆，却未核对最终来源按物理 BCI 去重；不可达 CFG 单测仅证实 Code `{0,3,4,5}` 中 BCI 4 为死块；部分解码 Reader 单测仅证实 `[nop,0xcb,return]` 在 BCI 1 停止时可靠前缀为 `{0}`。四类均不能从下层单测推断最终 Java source map。后续测试要分别固定 canonical 身份、已解码归属与实际 quote 来源，核对两种证据正文、低预算/取消；融合节点无 SSA 时只可断言已知原始块起点，不可把整个 `[start,end_bci)` 视作它独占的指令。
