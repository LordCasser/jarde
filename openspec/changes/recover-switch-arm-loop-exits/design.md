## Context

[巡查证据](../../evidence/java-syntax-2026-10-01/cf13-switch-continue-patrol/README.md)：两形字节码共通骨架（头 4–6、selector 9–12、arm goto join、default 内 `goto <latch>`）。`noCont` 恢复证明 join-统一形态已通；continue 边（57→63/66）是唯一变量。labeled-loop 切片已让普通块内的 continue 呈现（`Region::LoopContinue`），本片把它接到 switch arm 出口分类。插桩证明现状退化为三个重叠 Fallback（`[4,9]`/`[9]`/`[69,…]`）→ `ownership_overlap` 误报。

## Goals / Non-Goals

**Goals:** (1) W1 形态（join 存在 + 单 arm continue 绕过）与 W2 形态（全部 arm 出口即 latch，join==latch）都恢复 `while { switch { case…: …; case…: if (…) continue; … } [join 语句] }`；(2) 退化 Fallback 互斥划分，诊断指向真实首个失败。**Non-Goals:** labeled continue（外层非直接 loop，需要 label 的双层嵌套首片只验不加 label 的形态，label 形态随后按需另片）；`break` 出 switch 到 loop 外（LoopBreak 已有通道，作回归不扩语义）；irreducible；string switch 内 continue（变体仅回归现状如实记录，若同因顺带修并在报告区分）。

## Decisions

1. **出口分类在 switch 构造的 arm 走查处。** 对每个 arm 的终边分类：到其余 arm 公共汇合块（join）、到外层 loop 的 latch/test 块（continue）、到 loop 外（break/return，既有通道）。continue 边把该 arm 的收尾呈现为 `continue;`（复用 `LoopContinue` region/呈现）；join 候选 = 排除 continue 边后其余 arm 的公共汇合；无剩余 arm 时 join 即 latch（arm 自然落出）。selector 块唯一属 Switch region（修双 claim 的主因）。
2. **外层 loop 判定用既有 loop 事实**（`loop_of`/latch 集，region.rs 已有），不新造 loop 分析；arm 出边必须恰为 latch 或 test 出口块才归类 continue，其它落点保持现有拒绝。
3. **Fallback 互斥划分**：退化路径按 walk 顺序对已 claim 块去重（后到 gap 剔除已属块；空集不生成 region），保证最终树无重叠；诊断取该 walk 真实首个 FallbackReason。这是账本真实性修复，不改变"构造失败→整方法 quote"的保守语义。
4. **验收锚定**：W1 `70`/`36`、W2 `noCont 70`（逐字不变）/`contNoJoin` 行为一致；变体族（arm 内 break、default 即 continue、两 arm 都 continue）编译运行对照。

## Risks / Trade-offs

- **误把到 latch 的普通计算边当 continue** → javac 的 continue lowering 恰为 goto latch；非 continue 的 goto latch 在源码上不存在（编译器不生成），负例以手工字节码或改写变体钉住边界（落点非 latch/test → 拒绝）。
- **join 排除后空** → 决策 1 的 join==latch 分支正是 W2 形态；测试钉死。
- **与既有 switch 切片回归交叠** → `noCont`、named-catch 内 switch、string switch 回归全绿为门禁。
