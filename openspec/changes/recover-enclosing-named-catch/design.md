## Context

固定 T3（[巡查证据](../../evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/README.md)）：具名行 `[0,36)→38` 覆盖 init@0 到 cleanup 尾；handler 38 为 `astore; ldc "caught"; areturn`（或 voidNamedRecover 的调用体）。主线 `jre_guard_unexplained_row`@0 拒绝。实验（放宽排除）后剩 `jre_region_uncovered_blocks [38]`——证明层通过、呈现层缺 catch 子句。`Unexplained` 的保守语义（不把 TWR 伪装成 catch）由本片的**完整性证明**替代：claim 自身行 + 清理语义完整呈现于 try 头，包围行由 catch 子句呈现，两者共同解释全部行与块。

## Goals / Non-Goals

**Goals:** 单具名包围子句：行覆盖 claim 全跨度（起点可早至 claim 语句边界，含 init 前置指令属于 catch 保护范围）、handler 在 claim 外、类型具名；Plan 携带子句，Builder 发射 `} catch (E e) { … }`，handler 块经既有 region 呈现并入覆盖；两个出口（正常续行/子句内 return）都闭合。

**Non-Goals:** 多 catch 子句与 multi-catch（`E1 | E2`）；catch-all 包围行（那是 finally/TWR 自身语义，另有证书域）；catch 包普通 try（Catches 既有路径不动）；嵌套双层 catch；17a 的调用语句体（先行切片）。

## Decisions

1. **容忍判据替换排除子句**：`enclosing_clauses` 对"覆盖全跨度"的具名行走新增分支——要求 `row.start_bci <= shape_start` 且 `row.end_bci >= handler_end`（原排除条件的**取反即为接受条件**，加 handler 块在 claim 外与既有多子句约束），注释改写为"javac 的 catch 包 TWR 行覆盖整个降低，由子句呈现解释"。原"部分覆盖"分支（起点在 shape 内）语义不变。
2. **子句呈现放 Builder 的 TWR 发射处**：Plan 增 `enclosing: Option<(row_ordinal, catch 参数槽)>`；Builder 在 try-with-resources 头之后发射 catch 子句，handler 体块集交 region walk（复用 Catches 的 handler 体呈现纪律：绑定 store 命名参数、体块走普通语句呈现）；覆盖账本把 handler 块记入本 statement。
3. **验收锚定叠加形状**：T3 两形、T1.twrVoidNamed、C4.twrNamed（与 17a 叠加）全部恢复；`正常路径 done/caught 注入路径`三方对照；无 catch 的 TWR 家族逐字不变。

## Risks / Trade-offs

- **误把 finally 包装行当 catch** → 只接受具名类型行；catch-all 包围行继续 Unexplained（负例钉死）。
- **子句内控制流复杂化覆盖** → 本片 handler 体限于 region 可呈现语句（return/调用/局部语句）；复杂体（分支/循环）负例保持拒绝，随后按 Catches 扩展路径另片。
- **17a 未合入时的顺序依赖** → 本片排在 17a 之后；若 17a 延迟，C4.twrNamed 以 T3（void 体）为验收锚。
