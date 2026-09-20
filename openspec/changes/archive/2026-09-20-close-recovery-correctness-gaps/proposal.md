## Why

P0–P5 已按各阶段范围归档，但 `cd6f2f0` 的公开恢复入口仍存在两个可复现的正确性缺口：`(x + 1) + ++x` 被写成语义不同的 Java/Structured；被拒 cast 前的 `getstatic` 从文本与 source map 消失，遗漏可能触发的类初始化。当前不能确认整体完成，需在已有恢复层关闭这两个缺口；历史测试绿色与归档记录保留。

## What Changes

- 嵌套表达式按实际生成代码的求值点验证旧 SSA 值；不能把 producer BCI 当成当前求值点。无法安全重建时保留完整低级表示，禁止输出已知语义错误的 Structured 结果。
- fallback 保留其依赖的可观察生产者及物理 origin，包括字段读取、类初始化与可能抛异常的操作；不能只登记延迟调用。
- 将两个独立 javac 反例纳入永久回归和库/CLI 验收，并明确既有编译/执行对照只覆盖其列出的样本。
- 更新当前完成判断：本 change 的实现、反例与候选提交门禁全部通过后，才可确认这些正确性阻塞已关闭。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：明确实际求值上下文贯穿嵌套表达式，以及 fallback 的可观察生产者闭包。
- `recovery-validation`：补充能否认错误 Structured 输出、核对 fallback effect/origin 完整性的独立完成门槛。

## Impact

前提是现有 MethodIr、SSA/effect、声明、source map 和公开 recover_method 已交付。实现范围为 `jarde-java` 的 build/相关证据检查及根恢复回归；必要时调整既有私有 AST 的最小物化表示，不重建 SSA、不增加通用中端或新 crate，不改第三方依赖。

本 change 不扩展现代源码输出、MethodParameters/类级声明、cache/index/并行，也不重做 P0–P5。它只关闭本轮发现的两类错误，不承诺任意 JVM 方法都能还原为完整 Java 编译单元。复核反例与基线验证见 [verification](verification.md)。
