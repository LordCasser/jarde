# Compound condition physical-origin test candidate v3

此增量基于 root 已格式化的当前测试文件。它只调整 default/all evidence 的断言边界：`essential + SourceMap` 不请求 `RegionDetails`，所以 default 检查产物、无 `@bytecode`、以及已观察 branch BCI 的实际 if-condition source-map span；`all()` 仍严格要求 RegionDetails 中存在结构化 `if` region，且没有 fallback region。两个 profile 的 method、ordinal、正文、source map 仍需相等；all profile 仍逐条验证完整物理字节码指令均有准确 `PhysicalMethodId` 来源。

已从 API 代码核实 `RecoveryEvidenceRequest::all()` 选择 `RecoveryEvidenceKind::SUPPORTED`，包含 `RegionDetails`；报告的 RegionDetails materializer 对内部 recovered regions 构造 `RegionRecord` 并保留 `region.rule()`。因此候选不会把 all regions 当成空，也不会因 default evidence 未选 regions 而放松 all 的结构门槛。all 中本 fixture 是否实际产出 `if` region 留给 root 按当前生产候选执行此严格断言确认。

Frozen class、javap BCI 集与 source-origin 规则沿用 v2，见 `base.json`。未编译、未运行，状态为 `not_run`。
