## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/single-interface-fold-patrol/README.md)：member_family 诊断三键。**第一个取证义务**：读 `scan_family_root` 单静态行选择点（`Candidate` 分支条件）与 `StaticMembers` 返回序——确定"单静态行统一改道 StaticMembers"的改动面与 `Candidate` 通道其它消费方（非静态候选/枚举投影对）的隔离；corpus diff 双案（改道 vs 窄通道加准入）等价性。

## Goals / Non-Goals

**Goals:** 单静态接口子折叠；Y1 家族重编行为一致；M2 类形态与全部既有家族零回退。**Non-Goals:** 非静态窄通道；枚举投影对；匿名行；孙代。

## Decisions

1. **改道优先**：单静态行 → `StaticMembers`（一行列表），`Candidate` 仅保留非静态候选与投影对路径——除非取证发现改道面更宽（此时窄通道加 0x0608 准入）。
2. **验收锚定**：Y1（`interface StrFn` 折叠、`StrFn` 拼写、`hi!`/`45`/`[b, aa]`/`8`）+ 变体（单静态抽象方法类、单静态注解形登记）；负例（非静态单子不折叠、M1/M2/FV 逐字不变）。

## Risks / Trade-offs

- **改道改变 M2 输出** → corpus diff 必须逐字（窄通道回退进折叠本应同构；不同则如实报告并回退到准入案）。
