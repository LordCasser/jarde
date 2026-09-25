# Lambda 适配的来源与停止实测

`tests/p3_lambda_adaptation.rs` 对真实 Java 8 `strings()` 调用点分别请求 essential/all。两者正文相同；all 中动态 `(String) p0` 只映射到 `invokedynamic` BCI 0 / CP #7，没有虚构 `checkcast`。该站点无捕获，来源表不应有派生捕获 BCI。新增的 `CapturedLambdaAdaptationProbe.captured()` 则有真实捕获：前缀值在 BCI 0 生产，BCI 4 装载捕获，`invokedynamic` 在 BCI 5 / CP #13。动态 cast 的主来源是 BCI 5，包围它的 lambda 保留 BCI 4 派生来源，不能把前缀生产 BCI 0 当成捕获装载。

`lambda::tests::lambda_adapter_parameter_work_stops_through_the_shared_analysis_budget` 用四个真实描述符和 `LambdaMetafactory` 事实，把 `AnalysisSteps` 精确设为描述符总字节数。四段解析付费完毕，第一项适配参数计划在 site BCI 42 以 `StopReason::Budget(AnalysisSteps)` 停止；同一路径预取消时得到 `StopReason::Cancelled { at: Some(42) }`，使用量仍为零。

root 对 `strings()` 单方法按 essential 运行测得完整用量为 33 个 `IrItems`、122 个 `AnalysisSteps`、2500 个 `OutputBytes`（包含输入 class 的 2205 bytes）。`IrItems=31` 在 Builder 预收费两节点 Local/Cast 时拒绝：已用 30，尚留 1 个名额，不足以创建整批节点；停止位置是 BCI 0。此时分析步骤已付足 122，输出仅有输入 class 的 2205 bytes，正文和来源表均为空。因此这次确实停在适配 AST 内，不能把后置 source-map 交付耗尽误认为内部验证。永久测试在完整用量以下搜索这一阈值，并同时核对两节点收费、位置、阶段用量与无半正文。

all 证据下的 `IrItems`、`AnalysisSteps`、`OutputBytes` 限额也各自验证了停止维度；若后置来源交付才用尽，完整正文可以保留，不能宣称每个限额都撤销已提交正文。直接从 `recover_method_with_evidence` 入口预取消仍会在更早的 IR 表阶段误报 `IrTableMissing { canonical }`，虽不交付正文，也不是准确的 `Cancelled`。这是另案入口分类债务，不代表上述 Builder 内的停止传播失败。
