# 结构化 List 合并控制：旧产品真实基线

本次 root 在产品基底 `5f5fb2c605d61e18f392bb3abe6bdae6e10bf41e` 的冻结 CLI 上实际执行了 25 条命令。Java 8/23 原始完整 `UpstreamNestedLoops` 与 Runner 均编译、运行 exit 0，17 条异常/集合状态/副作用记录逐字一致。编译使用 `-g`，原类、javap、原始流、完整生成源码和所有命令见 `capture/`；输入 SHA 见 `inputs/manifest.json`。该目录是观察基线，不是新产品验收。

JADX 1.5.6 default/rename-none 的完整生成类在双 JDK 共 4 次编译均 exit 1。合法源码以 raw 别名向 `List<String>` 写入 Integer，随后遍历触发 ClassCastException；JADX 消去别名后生成 `target.add(value)`，同时保留 `List<String>` 参数，导致 Integer 无法赋给 String。原始编译 stderr 保留，未手改源码，也没有运行失败产物。

旧 Jarde default/all 在双 JDK 的完整类与条件隔离编译共 8 次均 exit 1；单一目标类下的“隔离”仍是同一完整类，不代表方法隔离成功。`rawAliasEscape` 的两个互不相交局部生命期都复用 slot 5，物理 checkcast 分别为 BCI 27 Integer、69 String，LVT 分别 `[32,41)` 和 `[74,82)`；正文合并成一个 Integer 声明后又赋 String。`handled` 则保留 ExplanationOnly 的 `jarde_refused_body()`，不是可编译源码。两项独立债务登记在 `openspec/completion-review.md`，不混入当前泛型投影实现。generic refusal 不能代替正文质量证明。

原 v5 checker 实际失败：它错误要求所有方法的物理异常表为空，失败 invocation 原样留在 `failed-v5-checker-invocation/`。Luna 私有 v6 只在确切原始 source SHA、owner、`handled(List,List)V` 三者匹配时要求唯一 `[0,47)->50 RuntimeException` 行；每个方法仍解析 Code 全部异常表并逐条核 javap 和 parser count，其他方法要求零行。root 审查 diff 后复制同字节到独立目录，再实际运行 exit 0。成功 invocation、checker 和 `observation.json` 均保留。

观察状态为 `verified_observation_capture_with_quality_gaps`，`behavioral_acceptance=not_claimed`。四个 Jarde `handled` method map 有已报告的物理 BCI 缺口；失败编译均为 no-runtime，不计行为通过。checker 对 class BLAKE3 只核形状、owner/location 与 default/all 一致性；物理 class bytes 另以 SHA-256 固定，未声称独立重算 BLAKE3。不同类名的独立 String/Integer 正向完整类另行实测，不能删掉这里的负向控制来取得整类成功。
