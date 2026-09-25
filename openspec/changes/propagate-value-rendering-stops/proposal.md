## Why

`preserve-lambda-descriptor-adaptation` 已让真实 Java 8 lambda 工厂的三段参数类型恢复出正确 Java，但新增的描述符遍历、适配向量及 cast AST 没有逐项预算计费。`lambda_expr` 中 `charge(IrItems, 0).ok()` 既不计费又吞停止。更根本的是 `lambda_expr`/`render_value` 使用 `Result<Expr, String>`：在里面给适配器加预算后，超限会被当成形状拒绝，经 fallback 输出，而不是本次构建的 `StopReason`。因此语义对照通过仍不能宣称资源与取消合同完成。

## What Changes

- 在现有值表达式构建链中保留**形状拒绝**与**预算/取消停止**的区别，让后者原样到达已有 `build`/报告停止平面；不为 lambda 建独立预算。
- 在 lambda 计划的新增 descriptor/适配工作、Builder 新增参数与 cast 节点构造前，按共享预算计费并轮询；停止时不发布正常或半适配正文。
- 以真实 `LambdaAdaptationProbe.strings()` 的内部阈值、带捕获适配对照及嵌套值调用点测试来源、正文和停止类别；保留预取消入口被误报 `jre_ir_table_missing` 的独立债务，不借本项修复整个 IR 前置阶段。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 新增的 lambda 值适配工作遵守共享工作/IR 预算，取消与超限沿现有停止平面报告；预算停止不被降级成普通 Java fallback。

## Impact

主要涉及 `jarde-java::lambda`、`build` 值表达式错误传播与对应测试。保持现有 AST、bootstrap 证明、SourceMap、公开 API 和语法准入边界；无需泛型/继承解析器。上游真实预取消的 IR 表分类、其它语法点的预算口径变化分别处理。
