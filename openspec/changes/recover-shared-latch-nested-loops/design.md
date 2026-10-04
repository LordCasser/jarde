## Context

[巡查证据](../../evidence/java-syntax-2026-10-04/shared-latch-patrol/README.md)：S5 `outerContinueInner` 字节码锚——外层 header@4/latch@35/退出@41，内层 header@20/latch@29，**latch 35 被内层退出边（`if_icmpge 35`）与外层 continue 边（`goto 15→35`）共享**。既有相邻片：`recover-loop-body-double-jumps`（同体两跳转边归类，`loop_break_transfer`/`loop_continue_bridge`/`loop_side_routes`）、`recover-labeled-loop-tail-coverage`（next-guard 与标签通道）。**第一个取证义务**：读 `region.rs` 循环形状证明的 latch 归属判定（"a test, an exit or a latch this subset does not prove" 的产生点），确认它如何枚举 latch 候选与归属检查；再确认 dj 片的边语义归类能否复用为"内层退出边 → 外层 latch 语义"的分派。

### 与既有 change 的关系（立项查重，2026-10-04）

loop 域共 19 个 change，其中已验收的相邻片经逐读其 Why 段确认**均不覆盖跨层 latch 共享**：

| 已验收片 | 覆盖形态 | 与本片的关系 |
| --- | --- | --- |
| `recover-loop-body-double-jumps`（7/7） | **同一循环体内**两条独立循环跳转（break+continue，`if(k==1) break; if(k==0) continue;` 为最小复现） | 最近邻但不同层：本片处理**嵌套两循环共享 latch**，其判据（内层退出边目标 == 外层 latch）不在 dj 片的边归类范围 |
| `recover-proved-loop-exit-gateways`（5/5） | 同一循环的**双 break 出口**闭合（CF-08 `while(true)` 双 break） | 单循环多出口，无嵌套关系 |
| `recover-effectful-dual-loop-exits`（5/5） | 带效果的双出口（循环头越界执行 `cost(7)` 后跳共同后继） | 同上，单循环 |
| `recover-switch-local-join-before-loop-continue`（6/6） | 循环内 switch 两 case 汇合后共享语句、另一 case 直跳外层循环更新 | 汇合归属问题，非 latch 共享 |
| `recover-loop-arm-join-continuation`（6/6） | 外层 `if` 两臂汇合后需执行 `value += 1`（臂内一臂含单出口循环） | 汇合后续语句，非 latch 归属 |

即本片的判别（外层 `continue` ∧ 同体内嵌套循环 → latch 被两条语义边共享）是**上述五片均未触及的独立边界**，不重复立项。`recover-labeled-loop-tail-coverage`（next-guard 与标签通道）亦为不同机制。

## Goals / Non-Goals

**Goals:** 外层 continue 与内层循环退出共享 latch 的双层循环恢复；带标签 `continue outer` 同形恢复；真实业务形（Map.Entry 双层 + 前缀过滤）体恢复。**Non-Goals:** 泛型 Signature 投影（`Svc.lookup` 的返回 `List<String>` 投影属 same-class/nested-headers 域，本片只锚体恢复，投影按既有通道如实呈现或拒绝）；三层以上共享 latch（验一形登记）；irreducible 图（CF-18 硬前沿，不碰）；同体双跳转（dj 片已闭合，不动）。

### 账本归属与落点（root 2026-10-04 补记）

- **inventory 单元归属**：本片属 **CF-11**（"嵌套/顺序循环及循环体内条件、取值更新的嵌套区域恢复"，状态"部分已测、仍待扩验"）——CF-11 的边界原文即"只覆盖 CFG 支配/回边信息可形成嵌套区域的形态"，本片正是该边界内尚未覆盖的一个子形（外层 continue 与内层退出共享 latch）。落地后须回写 CF-11 行。**与 CF-09（循环出口 break/continue/标签/嵌套出口）相邻但不同**：CF-09 处理单循环的多出口归类，本片处理**跨两层**的 latch 归属。
- **落点（root 已定位，行号会漂移、以锚点名为准）**：`crates/jarde-java/src/region.rs` 的 `fn latch_tested_loop`（ncl 合并后约 10470），其 `if latches.len() != 1 { return Ok(None) }`（约 10484）即"latch 归属单一所有者"的现约束——本片要放宽的正是它。相关既有件：`fn latch_test_chain`（约 10133）、`fn first_latch_test_suffix`（约 10389）、`fn latch_test_suffix_is_effect_free`（约 10424）；另一处 LoopShape 产生点在约 8244（`self.latch_test_chain(...)` 之后 `loop_of.latches().iter().find_map(...)`）。
- **与既有五片的正交性已核实**（见 Context 的查重表）：`recover-loop-body-double-jumps`(7/7)、`recover-proved-loop-exit-gateways`(5/5)、`recover-effectful-dual-loop-exits`(5/5)、`recover-switch-local-join-before-loop-continue`(6/6)、`recover-loop-arm-join-continuation`(6/6) 均不覆盖跨层 latch 共享。

## Decisions

1. **latch 共享判据（CFG 结构事实）**：三条同时成立才准入——(a) 内层循环的块集完全包含于外层循环体（支配关系既有）；(b) 内层循环的**每个退出都路由到外层 latch**（直接落在 latch 上，**或经其自身的一次 transfer（`goto`）落到 latch**）；(c) 外层 continue 边目标 == 同一 latch 块。任一不成立保持现拒绝（不放宽到任意共享）。
   > **root 验收更正（2026-10-04，采纳实现者升级的措辞冲突）**：本条 (b) 原写"内层循环的退出边目标 **==** 外层 latch 块"（字面直接相等）。实现者指出该字面判据与 break 形锚冲突，**root 用 javap 独立复核确认冲突真实**：`S3.nestedBreak`（tasks 2.1 钉死的必收锚，即 `Svc.lookup` 业务形）的内层退出是 `84: ifeq 153` → `153: goto 20`，即**经一次自身 transfer 落到 latch 20**，而非直接等于 latch。故字面 (b) **过窄**，会使必收锚被拒。正确判据是"**每个退出都路由到后边**（直接或经其自身的一次 transfer）；**逃逸与分歧退出拒绝**"。该泛化的阻断力已由两个负例实证：`ExitDiverges`（内层退出落在**不**经单一 transfer 到 back edge 的块上 → 拒）、`LabeledBreakOuter`（带标签 `break outer` **逃逸过** back edge → 拒）。故 (b) 的准确表述为上文的"路由到"而非"等于"。
2. **归属按边语义分派**：continue 边 → 外层 latch 语义（回外层 header）；内层退出边 → "离开内层后落入外层 latch"语义（同样回外层 header）。复用 dj 片的边语义归类思路，不新建第三套分类。
3. **验收锚定**：S5 两形（`13`/`13`）+ S3/S4 的 Map.Entry 双层 + `Svc.lookup` 体（行为 `[px:3]`）；负例（内层退出目标≠外层 latch、三层共享登记现状、单层 continue 与无跳转双层逐字不变）。

## Risks / Trade-offs

- **归属误分派致 continue/break 语义错** → 判据三条全要 + 行为三方对照（`-Xverify:all` 逐路径）；带标签形单独钉死（`continue outer` 必须回外层而非内层）。
- **与 dj 片的边归类交叠**（同体既有 continue 又有内层循环）→ 两片判据正交：dj 管同一循环体内两跳转边，本片管跨层 latch 共享；测试双向覆盖交叠形（如外层 continue + 内层 break + 内层 continue）。
- **三层以上** → 验一形登记，不泛化（MVP）。
