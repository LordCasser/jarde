## Why

[字符串域巡查](../../evidence/java-syntax-2026-10-02/string-ops-patrol/README.md)确认：引用相等 `a == b`（`if_acmpXX` + iconst_0/1 降低）的结果在**调用实参位**（`append(Z)`）无 int→boolean 证据 → 语句被引、重编缺语句（S1 固定复现）；同链其余字符串操作全部健康。布尔上下文的既有证明覆盖赋值/return/条件位，缺"比较结果直接作实参"。

## What Changes

- 实参位转换判定接受"分支选择 0/1 的引用相等值"：调用实参的 SSA 值由 `if_acmpXX` 双臂（iconst_1/iconst_0）合流产生且形参为 boolean 时，按 boolean 呈现（`a == b` 直译）。数值相等 `if_icmpXX` 同模式一并覆盖。
- S1 完整恢复且行为一致；既有赋值/return/条件位的布尔证明与负例零回退。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：引用/数值相等结果直接作 boolean 调用实参可呈现。

## Impact

仅 `crates/jarde-java` 私有实参转换判定及测试；复用既有布尔分支值 SSA 证明形态，无新机制。
