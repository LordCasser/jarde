## ADDED Requirements

### Requirement: 外层 continue 与内层循环退出共享 latch 时可证明

系统 SHALL 在满足三条 CFG 结构事实时接受循环 latch 共享：内层循环块集完全包含于外层循环体、内层循环的**每个退出都路由到**外层 latch 块（直接落在 latch 上，**或经其自身的一次 transfer（`goto`）落到 latch**）、外层 continue 边目标等于同一 latch 块；归属按边语义分派（continue 与内层退出均回外层 header）。此外循环的后边 SHALL **恰好**是本形状所解释的路由（不多不少），且被重选举的 continue 臂 SHALL 是**恰一块**的 `Straight`。单层 continue、无跳转嵌套、同一循环体内双跳转（既有片）与全部既有循环形态 SHALL 逐字不变；内层退出**逃逸过** back edge（如带标签 `break outer`）或**分歧**（落在不经单一 transfer 到 latch 的块上）SHALL 保持既有拒绝。多层（含三层）共享 latch 若**逐层满足**上述三事实与后边闭合，SHALL 由同一机制组合恢复（不另立泛化判据）；不满足者保持既有拒绝。

> **root 验收更正（2026-10-04）**：本 Requirement 原文写"内层循环退出边目标**等于**外层 latch 块"与"三层以上共享 SHALL 保持既有拒绝或登记"，两处均经实测更正。(1) "等于"过窄：必收锚 `S3.nestedBreak`（`Svc.lookup` 业务形）的内层退出是 `84: ifeq 153` → `153: goto 20`，即**经一次自身 transfer** 落到 latch 20，故判据须为"路由到"（直接或经其自身一次 transfer）；其阻断力由 `ExitDiverges`（分歧）与 `LabeledBreakOuter`（逃逸）两个负例实证。(2) 三层形**并非一律拒绝**：root 实测 `ThreeLevel` 冻结 fixture 渲染 quotes=0、恢复成功，且两腿运行输出 SHA 一致（`68ca3fba…`）——它是同一机制逐层组合的结果，行为已证。

#### Scenario: 双层过滤循环恢复

- **WHEN** `for (i…) { if (cond) continue; for (j…) { s += j; } }`（纯 int，无标签与带标签两形）三方 Java 8 重编
- **THEN** 双层循环与 `continue` 完整呈现，`java -Xverify:all` 逐路径与原 class 一致（`13`/`13`）

#### Scenario: 真实业务形体恢复

- **WHEN** Map 缓存双层 for-each + 前缀过滤 `continue` + 内层 `break`（`Svc.lookup` 形）三方重编
- **THEN** 方法体完整呈现（泛型投影按既有通道如实处理），运行逐路径与原 class 一致

#### Scenario: 既有与负例不变

- **WHEN** 输入为单层 continue、无跳转嵌套循环、同体双跳转（既有片），或内层退出**不路由到**外层 latch（**逃逸**过 back edge，如带标签 `break outer`；或**分歧**——落在不经单一 transfer 到 latch 的块上）
- **THEN** 前三者与本变更前逐字一致；后两者保持既有拒绝（`jre_region_loop_shape` + `jre_region_uncovered_blocks`）。root 实测三个负例（`LabeledBreakOuter` 逃逸、`ExitDiverges` 分歧、`Overlap` 同体双跳转属 dj 域）的 **`run` 循环方法**均保持拒绝——即每条准入条件各自独立阻断，重选举面未过宽
