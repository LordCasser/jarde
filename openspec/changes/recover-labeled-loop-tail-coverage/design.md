## Context

[巡查证据](../../evidence/java-syntax-2026-10-01/labeled-loop-patrol/README.md)：L2 尾段 `append('T').append(i).append(';')` 的尾 `pop`（BCI 66）被引；L3 对照（无标注）零残留。17a 引入的 `discarded_call_pop`（`guard.rs`）判据：pop 单读值 == 同块紧邻非 void invoke 的唯一栈写值（SSA same）、不落局部槽——已带文档与负例；仅 TWR 体检查点（`statement_free_with_discarded_calls`）启用。标签名由呈现层生成 `jarde_loop_{bci}`。

## Goals / Non-Goals

**Goals:** 标注循环尾段（及同走查路径上的同类段）接受 discarded-call 语句；L2 零 `@bytecode` 残留、行为逐字一致；标签源码式拼写。**Non-Goals:** 17a 负例放宽；标签名与源码逐字相同（无名字事实，不虚构）；`pop2`/双槽消费；非走查路径的其它残留。

## Decisions

1. **泛化启用点而非复制判据**：找到标注循环尾段走查的语句核验处（region.rs 循环体/尾段核验），以与 17a 相同方式接入 `discarded_call_pop`（或把 `statement_free_with_discarded_calls` 的使用面扩到该路径）；判据与负例不复制第二份。
2. **标签拼写**：简单确定规则（`loop` 起始，冲突递增或 `outer`/`inner` 语义序——实现者按现有命名表惯例选一，测试钉死），去除 `jarde_` 前缀与 BCI 数字；仅呈现层。
3. **验收锚定**：L1/L2/L3 逐字（除标签名与残留移除）；变体（break-label 尾段、双层标注、尾段混合语句）零残留；17a 测试全绿。

## Risks / Trade-offs

- **泛化面过宽** → 只接标注循环尾段走查路径；其它路径残留另案（如实报告）。
- **标签改名破坏既有期望** → 全仓 grep `jarde_loop` 同步受影响测试；改名是本片有意行为，逐处更新并注明。
