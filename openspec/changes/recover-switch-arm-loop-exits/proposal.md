## Why

[CF-13 巡查](../../evidence/java-syntax-2026-10-01/cf13-switch-continue-patrol/README.md)确认账本登记的已证差距并定位根因：loop 体内整数 switch 的 arm 存在直跳外层 loop latch 的边（`continue` lowering）时，主 walk 无法构造 `Loop { Switch { arm 内 continue } }` 而整方法 quote（W1.mix/W2.contNoJoin 固定复现；同形状去掉 continue 即恢复，判别变量精确）。仓库已有 `Region::LoopContinue` 呈现（labeled-loop 切片已验收），缺的是 switch arm 走查对"arm 出边 = 外层 loop latch"的识别与构造。次生缺陷：退化 Fallback 生成重叠块集（`[4,9]`/`[9]`/`[69,…]`），最终误报 `jre_region_ownership_overlap` 而非真实首个失败。

## What Changes

- switch 构造处：识别 arm 的出边落在外层 loop 的 latch/test 集内且该 loop 已被（或将本 walk）构造时，该 arm 以 `continue;`（直接外层无需 label）呈现，并从 switch join 候选中排除；join 取其余 arm 的公共汇合；全部 arm 均 continue/落出到 latch 时 join 为 latch 本身（W2 形态）。
- 退化路径的 Fallback 块集改为互斥划分（或合并为单一 quote region），使最终诊断指向真实首个构造失败而非 ownership_overlap。
- 以 W1/W2 固定类与变体族（break-from-switch 同构、多层嵌套、string switch 内 continue）验收；`noCont` 对照逐字不变；行为三方一致。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：switch arm 到外层 loop latch 的出边恢复为 arm 内 `continue`；switch 与 loop 的出口所有权按边归属判定。

## Impact

仅 `crates/jarde-java` 私有 region 走查（switch arm 出口分类）与 fallback 划分及测试；复用既有 `LoopContinue` 呈现。`recover-switch-inside-named-catch`/`recover-switch-local-join-before-loop-continue` 等已验收 switch 切片零回退。不新增公开 IR/CLI/依赖。
