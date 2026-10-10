# One-arm loop baseline independent verifier v3

本版尚未执行。v1/v2 的失败记录、脚本和验收输出均保留。v2 的 `KeyError: home` 来自把冻结 JDK manifest 的 `legs` 项误读为包含 `home/tools`；实际字段是 `jdk_tools`，`home` 只在 baseline 的 `jdk_legs` 记录中。本版逐项按真实结构比对每个 java/javac/javap path 和 SHA，再由 pinned java 路径推导并校验该 `home`。CLI metadata copy policy 也按真实 schema 从 `frozen_jarde_cli` 读取，而不是从 `jadx` 对象读取。静态对照还确认 command matrix 为真实的 31 条命令，并将 per-profile method/source-map 汇总与每份 raw JSON 重算结果逐项比较。

本版还固定检查真实十个 case、两个 render-profile 组、JADX profile 与 case 的连接、所有 source/class 文件集合及运行摘要；逐个核对原始文档的 physical owner/flags、完整 evidence category、方法正文、BCI 来源和重复保存的 per-method 汇总，不把 manifest 成功标记当作原始证据。验收结果写入 `independent-acceptance-luna-v3.json`，已存在时拒绝覆盖。

noPrefix 的物理 `goto` BCI 14 仍必须作为唯一缺失映射明确报告；另外三条 explanation-only fallback、四个 Jarde 全类 missing-return 编译失败及其无 runtime 边界全部保留。成功只接受 baseline 证据和这些观察事实，不表示产品功能通过。
