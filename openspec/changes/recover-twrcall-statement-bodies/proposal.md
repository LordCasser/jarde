## Why

[CF-17 巡查](../../evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/README.md)确认：TWR 正文含**结果被丢弃的调用语句**（Java statement-expression，如 `r.toString();`，字节码为非 void `invoke` + 紧随 `pop`）时整方法回退——`guard.rs` TWR 体证明的语句子集只收 void 调用与 return。该形态是真实代码里最常见的 TWR 体写法之一（T2.popBody/popBodyVoidTouch 固定复现；void 体对照组恢复正常）。

## What Changes

- TWR 正文语句子集新增"调用语句"形态：同块内非 void `invoke` 后紧随 `pop`（pop 的读值恰为该调用结果），呈现为该调用的语句形式（复用现有调用语句呈现，结果丢弃与源码一致）。
- pop 之后语句边界、多语句体（调用语句与 void 调用/return 混排）一并覆盖；调用结果被消费（无 pop 或 pop 读者非该调用）不属本片，保持既有拒绝。
- 以 T2 固定类与变体族（纯调用语句体、混排体、try 内 return 后调用语句）验收；void 体对照组逐字不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：TWR 正文中的非 void 调用语句（结果丢弃）可随资源证书一起恢复。

## Impact

仅 `crates/jarde-java` 私有 Guard 正文语句子集与对应呈现及测试；多资源/可空资源/TWR 家族证书与 17b（包围具名 catch，随后另片）不相涉。不新增 Shape/公开 IR/CLI/依赖。
