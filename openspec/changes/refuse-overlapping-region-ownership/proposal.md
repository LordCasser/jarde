## Why

[合法 Java 8 三元值嵌入 OR 的样例](../../evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/analysis.md)让 BCI 8 `goto` 从短路链外进入共享真值生产者 BCI 25。当前链式 proof 正确地不把它当纯同极性链；普通 Region walk 却四次把 BCI 25 当“循环”重访，并把唯一字段消费点留给失去栈来源的后续 Straight region。可编译的 Jarde fallback 文本与原 class 在 32 条路径中有 30 条不同，还漏了多个来源 BCI。JADX 的完整类亦有 16 条不同，说明其条件合并不应成为真实性依据。

## What Changes

- 以 canonical block 身份检查一次 recovery 的 Region 树是否重复认领同一物理块；重复时丢弃部分结构，改为已有 whole-body fallback，说明是所有权重叠而非真实循环。
- 在预算和取消约束下保留整个已解码 body 的引用与 source map；本 change 依赖 [Region fallback 逐指令来源](../preserve-region-fallback-instruction-origins/proposal.md)先闭合，不在此重复枚举来源。
- 用额外入口样例以及既有 loop、switch、try、两/三测试短路控制，确认不会把有效单 owner 结构误拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 若 Region 树不能给同一 canonical block 唯一所有者，恢复结果 SHALL 原子拒绝结构化方法正文，完整报告来源，而非发布重复或断裂的 `if`/消费点。

## Impact

主要在 `jarde-java::region` 的最终所有权校验及其 fallback 诊断；复用已有 whole-body quote，不增新语法 AST 或 CFG pass。字段 `presented=true` 与实际发射不一致是单独的报告层债务，本 change 只保证 Region/正文拒绝正确；混合极性与三元条件的正向恢复也另案分析。
