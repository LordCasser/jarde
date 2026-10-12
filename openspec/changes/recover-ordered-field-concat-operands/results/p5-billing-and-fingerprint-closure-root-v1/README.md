# P5 计费、指纹和最终局部门禁

2026-10-13，root 执行并独立重开 `capture.tar.gz`，逐成员核对 sidecar 清单中的字节数及 SHA-256。大 CLI 不重复归档，其准确构建、冻结字节和模式以原 receipt 为准；其它原始记录完整保留。

真实新旧 CLI 对照包含 90 个成功命令、43 个物理方法（Guarded 24、BooleanContexts 19）。逐请求身份相同，text/outcome/content/source_map 全等，15 个计费维度只在 AnalysisSteps 增加 44，分别为 syncThrowsCatching 13、secondInitFailsCatching 13、suppressedCatching 18；IR 阶段的计费单独保存并不变。新增 retry proof 的计费扫描可从 region.rs 核对，但 CLI 未序列化 canonical CFG，不能声称已从响应证明各扫描的图规模或失败合取项。

此前完整 workspace-v4 的 P5 失败原样保留在相邻归档。ignored recorder 实际 1 passed，六 case 仅 many-method-class 的 AnalysisSteps 增加 44；随后正常 P5 测试实际 5 passed/1 ignored，同时检验 many=12104、direct/shared=19955 和完整正文一致。不是单凭重记表就接受。

指纹中断的原 receipt 仍是 running/零命令状态，raw 只观察到 ignored regenerator 1 passed，未观察其进程 exit 或正常 gate。恢复时实际句柄缺失且进程不存在，独立 cargo clean 成功。当前索引只新增 29 项，原 2164 项及分类不变，2193 项的正常 fingerprint gate 在 final-v6 实际 5 passed/1 ignored。

final-v6 receipt SHA `71d00d2c5f890a6272ae57d3d5063d4783179e0776696e21d925a11b41e52c63`。fmt/当前 CI Clippy、lib390、TWR4、P5 bulk5/1 ignored、fingerprint5/1 ignored、field-copy3/1 ignored、Unicode4、p3 patterns92、CLI build/global strict/finally clean全部通过；4854 来源前后相同，CLI SHA仍 `f991644eb87cef29df1509b270ba48d7c5d26ba72f46a5fae3271fc4528e3622`，root独立接受。当前源快照保存 region.rs 与两份 P5 输入。

本归档不替代 fresh 双 seed workspace、自身产品 CI 或干净主线交付。archive prep-v1/v2未执行，root-v3只修正准备脚本错误的路径与实际 receipt/root-summary字段，未改变历史记录。collector README 的“未执行”是准备时状态；实际执行结果见归档 raw 和独立接受。
