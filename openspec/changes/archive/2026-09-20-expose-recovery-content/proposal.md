## Why

benchmark 对 25,853 次请求的 token 启发式分类估计约 16.0% 只有理由注释，但 `RecoveryOutcome::Produced` 对这些结果仍成立；这还不是引擎提供的 AST 级统计。该字段表达“交付了产物”，调用方却缺少稳定的“产物是否有 Java 语句”信号，只能剥注释猜测，因而容易高估恢复覆盖。

## What Changes

- 明确 `Produced` 仅表示产物已提交，不表示语句、完整恢复、可编译或语义等价。
- 给 `RecoveryReport` 增加单一内容分类，区分未产出、仅解释、含语句；从最终提交的结构化产物导出，库和 CLI 同步返回。
- 将内容、quality、语法、execution 和行为验证继续分开；空分支与显式 `return;` 可以有语句，但不能因此变成 Structured 或完整语义。
- 为受控 fixture 和 benchmark 汇总固定分母、适用范围及 fallback 分类；历史正则结果标为启发式，不能静默改写。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：公开恢复产物的内容分类与 Produced 语义。
- `recovery-validation`：覆盖率报告区分有产物、有语句与已验证行为。

## Impact

前提为现有 AST/emitter、RecoveryReport 和公开 recover_method。影响 `jarde-java` 的报告/提交路径、CLI 序列化及恢复测试；不引入文本 parser 或依赖，不为兼容保留第二套质量体系。

不改变恢复算法、BCI coverage、fallback effect 保留或 source-map 契约，不把 corpus 百分比设为 CI 硬阈值，不新增通用评测平台。正确性修复与本 change 分别验收。关联分析见 [benchmark review](../../../benchmark-review.md)。
