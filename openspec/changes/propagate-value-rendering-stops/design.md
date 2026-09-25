## Context

Root 的 [lambda 验收](../preserve-lambda-descriptor-adaptation/verification-root.md)与 [预算实测](../preserve-lambda-descriptor-adaptation/verification-budget.md)说明：低预算没有发布半个适配正文，但 `IrItems` 命中的其实是完整正文后的来源交付。`lambda::plan` 通过 `parse_method` 读 site/SAM/instantiated/implementation descriptor，再按 SAM 参数遍历并分配适配向量；这些步骤不接收 Budget。`Builder::lambda_expr` 为每个参数构造 Local、0–2 个 Cast 和 Call/New；这些嵌套表达式不经 `push` 的 statement 计费。Emitter 的 output-bytes 计费与 source-map replay 不能替代构造前预算。

`Builder::instruction` 返回 `StopReason`，但被消费的 `InvokeDynamic` 是 `render_value` 的递归分支；该链和 `lambda_expr` 返回 `String` refusal，普通调用者把它变成 fallback。若直接将 `StopReason` 格式化为 String，`AnalysisSteps` 耗尽还可能成功写入 fallback，造成错误的执行平面。零额 `charge(..., 0).ok()` 尤其会吞掉取消。

## Decision

1. 继续用 `build()` 的共享 `Budget`、`stop::poll/charge` 和现有 `StopReason`。只为值渲染的**必要链**引入显式的二分结果：普通不支持形状仍走原有 refusal/fallback，预算/取消/elapsed 停止必须逐层向外传播并终止构建。允许一个局部 typed failure；不引入跨模块通用结果框架、pending-stop 隐藏状态、adapter 私有预算或把停止写成 refusal 文本。
2. `lambda::plan` 的 bootstrap descriptor 解析前按可量化的 descriptor 输入收费（字节/组件口径须在实现中固定并测试），新适配参数遍历/向量构造前按参数数目收费，循环间轮询。不要对已经拒绝、不走适配器的 site 伪造 cast 费用。对 JVM/reader 已有限制的 descriptor 仍计其新增分析工作。
3. `Builder` 在参数 Local、动态 Cast、实现 Object 固定 Cast 等新节点构造前按实际数量收现有 `IrItems`；表达式从 statement 入口、递归 `render_value` 和 source-map replay 进入时都遵守同一已决定计划，不在 replay 重新收费或选择不同形状。预算拒绝发生于构造前，无可被提交的半 AST。
4. 报告保留原 `StopReason::{Budget,Cancelled,Interrupted}` 的维度、位置和预算用量；形状拒绝仍保留 `jre_lambda_*` 及调用点/捕获来源。不能通过让 fallback 再失败的偶然性来模拟停止；低限值必须命中真正的计划或节点构造收费点。
5. 将直接方法恢复预取消时返回 `IrTableMissing` 的上游入口问题记录为单独债务。本项只要求取消进入已选值渲染路径后分类正确；不修改 reader/IR 的全局入口顺序。

## Alternatives Rejected

- 在 `lambda_expr` 内把 `StopReason` 转成 String：会把资源停止当作形状拒绝并可能发布 fallback。
- 只依赖方法 statement、输出字节或 source-map replay 的费用：未覆盖新增计划与 AST 构建工作，实测阈值也落在后置证据交付。
- Builder 内设置隐式 pending-stop 再从 `String` 返回：每个 fallback/提交边界都需另查状态，遗漏一个就会输出假正常正文；显式 typed failure 更容易局部审计。
