# One-arm loop baseline independent verifier v5

本版正式验收入口尚未执行。v1–v4 的失败记录、脚本和验收输出均保留。v4 暴露出物理成员汇总的类型边界：javap flags 在 parser 中先按完整 ACC token 严格校验，本版再映射为 classfile access_flags 数值，以匹配 CLI JSON 的 `physical_methods[].flags`。BCI 的 canonical string-key 转整数修正保持不变。只读 `verify_candidate_baseline()` preflight 首次发现 profile fact 的 `declaration` 不带 javap 声明末尾分号；本版按原始 fact schema 去掉该分号。修复后 preflight 返回 31 commands、111 closed files 及既有 noPrefix BCI 14 gap。该 preflight 不写正式验收结果；root 仍需审查并执行正式入口。除这些类型/序列化边界外，其余验收逻辑不变。

本版还固定检查真实十个 case、两个 render-profile 组、JADX profile 与 case 的连接、所有 source/class 文件集合及运行摘要；逐个核对原始文档的 physical owner/flags、完整 evidence category、方法正文、BCI 来源和重复保存的 per-method 汇总，不把 manifest 成功标记当作原始证据。验收结果写入 `independent-acceptance-luna-v5.json`，已存在时拒绝覆盖。

noPrefix 的物理 `goto` BCI 14 仍必须作为唯一缺失映射明确报告；另外三条 explanation-only fallback、四个 Jarde 全类 missing-return 编译失败及其无 runtime 边界全部保留。成功只接受 baseline 证据和这些观察事实，不表示产品功能通过。
