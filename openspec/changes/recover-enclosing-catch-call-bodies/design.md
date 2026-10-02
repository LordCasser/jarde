## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/twr-clause-patrol/README.md)：17b 的 enclosing_clause（`src/facade.rs::resolve_enum_constant_body_relations` 同文件族的 guard.rs 通道）catch 体白名单当前形态见其切片记录；17a 的 `discarded_call_pop` 判据在 guard.rs（TWR 体检查点 `statement_free_with_discarded_calls`）与 build.rs `discarded_evaluations` 两处既有。第一个取证义务：读 enclosing_clause 的体语句枚举位，确认接入形态（大概率直加一个判据调用）。

## Goals / Non-Goals

**Goals:** 丢弃调用形 catch 体恢复；P2/P3 双形 + 变体（多语句 catch 体：丢弃调用 + return）。**Non-Goals:** catch 体含分支/循环（既有拒绝保持）；结果被消费形；direct finally 与 catch 共存（三子句）；非 TWR 的普通 catch 体（Catches 通道已有丢弃调用支持与否需取证——若已有则不动）。

## Decisions

1. **白名单加判据**：enclosing_clause 体语句枚举处接入 17a 判据（同块紧邻 invoke+pop、单读值 SSA 同）；失败保持现拒绝文本。
2. **验收锚定**：P3.voidBodyCatch（`t[c]` 正常路径 + catch 路径注入 `E`）、P2.plainCatch；17b 冻结 fixture 逐字不变。

## Risks / Trade-offs

- **白名单过宽** → 仅丢弃调用形；消费形/多用途负例钉死。
