## Why

[TWR 子句巡查](../../evidence/java-syntax-2026-10-02/twr-clause-patrol/README.md)确认：TWR+包围 catch 且 catch 体为**调用语句**（`catch (e) { log.append("E"); }`——append 返回值 pop）整方法拒绝——17b 的 `enclosing_clause` 语句白名单只收 return/void 调用/局部语句，17a 已建的丢弃调用判据（TWR 体侧）未接入子句白名单（P2/P3 判别钉死：return 形 catch 体恢复、丢弃调用形拒绝，单变量）。

## What Changes

- `enclosing_clause` 的 catch 体语句白名单接受丢弃调用语句（复用 17a 的 `discarded_call_pop` 判据：非 void invoke + 紧随 pop、SSA 同、不落局部）。
- P3.voidBodyCatch / P2.plainCatch 恢复且行为一致；17b 冻结 fixture（return 形）与全部既有子句形态逐字不变；非丢弃形（结果被消费/插指令）保持拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：TWR 的包围 catch 子句体可含调用语句（结果丢弃）。

## Impact

仅 `crates/jarde-java` 私有 guard.rs enclosing_clause 白名单及测试；判据复用，无新机制。17b/17a 既有零回退。
