## Context

[巡查证据](../../evidence/java-syntax-2026-10-04/shared-latch-patrol/README.md)：S5 `outerContinueInner` 字节码锚——外层 header@4/latch@35/退出@41，内层 header@20/latch@29，**latch 35 被内层退出边（`if_icmpge 35`）与外层 continue 边（`goto 15→35`）共享**。既有相邻片：`recover-loop-body-double-jumps`（同体两跳转边归类，`loop_break_transfer`/`loop_continue_bridge`/`loop_side_routes`）、`recover-labeled-loop-tail-coverage`（next-guard 与标签通道）。**第一个取证义务**：读 `region.rs` 循环形状证明的 latch 归属判定（"a test, an exit or a latch this subset does not prove" 的产生点），确认它如何枚举 latch 候选与归属检查；再确认 dj 片的边语义归类能否复用为"内层退出边 → 外层 latch 语义"的分派。

## Goals / Non-Goals

**Goals:** 外层 continue 与内层循环退出共享 latch 的双层循环恢复；带标签 `continue outer` 同形恢复；真实业务形（Map.Entry 双层 + 前缀过滤）体恢复。**Non-Goals:** 泛型 Signature 投影（`Svc.lookup` 的返回 `List<String>` 投影属 same-class/nested-headers 域，本片只锚体恢复，投影按既有通道如实呈现或拒绝）；三层以上共享 latch（验一形登记）；irreducible 图（CF-18 硬前沿，不碰）；同体双跳转（dj 片已闭合，不动）。

## Decisions

1. **latch 共享判据（CFG 结构事实）**：三条同时成立才准入——(a) 内层循环的块集完全包含于外层循环体（支配关系既有）；(b) 内层循环的退出边目标 == 外层 latch 块；(c) 外层 continue 边目标 == 同一 latch 块。任一不成立保持现拒绝（不放宽到任意共享）。
2. **归属按边语义分派**：continue 边 → 外层 latch 语义（回外层 header）；内层退出边 → "离开内层后落入外层 latch"语义（同样回外层 header）。复用 dj 片的边语义归类思路，不新建第三套分类。
3. **验收锚定**：S5 两形（`13`/`13`）+ S3/S4 的 Map.Entry 双层 + `Svc.lookup` 体（行为 `[px:3]`）；负例（内层退出目标≠外层 latch、三层共享登记现状、单层 continue 与无跳转双层逐字不变）。

## Risks / Trade-offs

- **归属误分派致 continue/break 语义错** → 判据三条全要 + 行为三方对照（`-Xverify:all` 逐路径）；带标签形单独钉死（`continue outer` 必须回外层而非内层）。
- **与 dj 片的边归类交叠**（同体既有 continue 又有内层循环）→ 两片判据正交：dj 管同一循环体内两跳转边，本片管跨层 latch 共享；测试双向覆盖交叠形（如外层 continue + 内层 break + 内层 continue）。
- **三层以上** → 验一形登记，不泛化（MVP）。
