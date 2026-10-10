# One-arm loop baseline independent verifier v4

本版尚未执行。v1/v2/v3 的失败记录、脚本和验收输出均保留。v3 在 javac23 constructor 的 source-map 比较中暴露了 JSON 序列化边界：物理 BCI 字典键写入 JSON 后成为字符串，而报告中的 BCI 仍是整数。本版仅在消费物理方法表前将 canonical 十进制键还原为整数，并拒绝非 canonical 键；其余验收逻辑不变。冻结 JDK、CLI、原始证据及 source-map 汇总的 schema 预检结论沿用 v3。

本版还固定检查真实十个 case、两个 render-profile 组、JADX profile 与 case 的连接、所有 source/class 文件集合及运行摘要；逐个核对原始文档的 physical owner/flags、完整 evidence category、方法正文、BCI 来源和重复保存的 per-method 汇总，不把 manifest 成功标记当作原始证据。验收结果写入 `independent-acceptance-luna-v4.json`，已存在时拒绝覆盖。

noPrefix 的物理 `goto` BCI 14 仍必须作为唯一缺失映射明确报告；另外三条 explanation-only fallback、四个 Jarde 全类 missing-return 编译失败及其无 runtime 边界全部保留。成功只接受 baseline 证据和这些观察事实，不表示产品功能通过。
