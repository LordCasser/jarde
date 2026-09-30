## Why

[标注循环巡查](../../evidence/java-syntax-2026-10-01/labeled-loop-patrol/README.md)确认两个呈现质量缺口（正确性均保持、重编行为一致）：(1) `continue label` 循环的尾随语句段，其链式调用尾 `pop` 不被区域走查拥有，输出残留 `@bytecode` 引用标记（L2 固定复现；普通循环/直线体同形态已覆盖，L3 对照零残留）；(2) 标签用合成名 `jarde_loop_10`，暴露内部痕迹。17a 的 `discarded_call_pop` 判据已证明"非 void 调用 + 紧随 pop"的语句形态，只在 TWR 体检查点启用——本切片把它泛化到标注循环尾段，并改进标签拼写。

## What Changes

- 标注循环（含 `continue label`）尾段走查接受 discarded-call 语句（复用 `discarded_call_pop` 判据与计费），尾段 BCI 全覆盖；L2 输出零 `@bytecode` 残留。
- 标签拼写改为源码式简名（不带 `jarde_` 前缀与 BCI 数字；命名规则简单确定，如 `loop`、`outer` 递增去重），测试钉死。
- 以 L1/L2/L3 固定类与变体（break-label 尾段、多层标注、尾段混合 void/链式调用）验收；行为逐字一致。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：标注循环尾段的调用语句完整拥有（无引用残留），标签以源码式拼写呈现。

## Impact

仅 `crates/jarde-java` 私有区域走查与标签呈现及测试；串行排在 array-slot-retype 之后。17a 判据复用不放宽其负例；不新增证明机制。
