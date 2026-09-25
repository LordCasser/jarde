## 1. 固定现状与最小传播边界

- [x] 1.1 用 `LambdaAdaptationProbe.strings()` 区分分析、IR、输出和 source-map 阶段用量，找到能在新增 adapter 工作内触发的预算阈值；确认前置 0 费用调用、`render_value` 所有 `lambda_expr` 路径及 fallback 边界。实测见 [预算验证](../preserve-lambda-descriptor-adaptation/verification-budget.md)。
- [x] 1.2 构造一个带真实捕获 producer、同时要求动态 cast 的 Java 8 工厂；保留原/JADX/Jarde 完整类的重编、执行、BCI/CP 与 hash，对 bound-null 创建阶段失败单列拒绝对照。证据见 [捕获夹具](fixture-evidence.md)。

## 2. 实施停止传播与计费

- [x] 2.1 在值渲染的必要调用链中显式保留 refusal 与 `StopReason`，普通不支持形状的原 fallback 和报告语义不变；停止到达 `build()` 后不交付半正文。
- [x] 2.2 在 lambda descriptor 输入、适配参数计划及新 Local/Cast AST 生成前，用共享预算按明确口径收费和轮询；删除无效的零额 `.ok()`，无私有预算、无 replay 重复决策。

## 3. 独立验收

- [x] 3.1 正反两类 site 和内嵌值调用点均用**内部**预算阈值证明准确停止类型/维度/site；取消进入构建后为 Cancelled，unsupported 仍为原 refusal，任何停止不发布半个适配正文。见 [根代理验收](verification-root.md)。
- [x] 3.2 带捕获正例的原/JADX/Jarde 完整类执行、essential/all 来源、相邻 lambda/调用实参/来源回归、语料 fingerprint、fmt、受影响 Clippy 与 OpenSpec strict 由 root 独立复核；预取消 IR 表分类另案，不混入修复。Clippy 现存/连带阻断与 reader census 债务见 [根代理验收](verification-root.md)。
