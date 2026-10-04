# 共享 latch 的嵌套循环巡查（2026-10-04）——真实业务代码高频缺口

真实业务类巡查引出：`Svc.lookup`（Map 缓存 + 双层 for-each + `continue`/`break`）整方法拒绝。定向收窄（主线 `30e54613`；**巡查复用 spn worktree 的 HEAD 版 jarde-cli，零额外构建**）。固定转录 [fixture](fixture/)（S5 最小判别 / S3/S4 中间层 / Svc 真实业务类；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 o5/o3/o4/orig.out。

## 判别矩阵（单变量钉死，纯 int 无装箱/cast/拼接）

| 场景 | 主线 Jarde |
| --- | --- |
| 外层 `continue` 无内层循环（`outerContinueOnly`） | 恢复 |
| 内层循环无外层 `continue`（`innerOnly`） | 恢复 |
| 双层循环无跳转拼接（`nestedNoCast`）、单层循环含 cast 拼接（`singleCastConcat`）、`inLoop` | 恢复 |
| **外层 `continue` + 同体内嵌套循环**（`outerContinueInner`，纯 int） | 拒绝：`jre_region_loop_shape`（header@4）+ 3 未覆盖块 `[41,20,25]` |
| **同形带标签**（`labeledOuter`，`continue outer`） | 同拒 |
| 真实业务形（`Svc.lookup`、S3/S4 的 Map.Entry 双层） | 同拒（并因泛型返回叠加 Signature 投影拒绝） |

## 根因（字节码锚）

`outerContinueInner`：外层 header@4 / latch@35（`iinc 2,1; goto 4`）/ 退出@41；内层 header@20 / latch@29（`iinc 3,1; goto 20`）/ 退出 `if_icmpge 35`。**latch 块 35 被两条语义不同的边共享**：

1. 外层 `continue`（`ifne 12 → goto 15 → 35`）；
2. 内层循环退出（`if_icmpge 35`）。

既有循环形状证明要求 latch 归属单一所有者（"a test, an exit or a latch this subset does not prove"），共享 latch 使内层退出边与外层 continue 边无法各自归属 → 整循环不证 → 3 块仅经这些边可达（uncovered）。

## 处置方向

`recover-shared-latch-nested-loops`（区域层，中等片）：循环形状证明接受 latch 被"本循环的 continue 边 + 嵌套内层循环的退出边"共享——判据为 CFG 结构事实（内层循环完全包含于外层体、内层退出目标 == 外层 latch、continue 边目标 == 同一 latch），归属按边语义分派（continue→外层 latch 语义、内层退出→离开内层后落入外层 latch 语义），不新增机制。与 `recover-loop-body-double-jumps`（同体两跳转边归类）相邻但不同：那片处理**同一循环体内**的 break/continue 双边，本片处理**嵌套两循环共享 latch**。S5 两形 + S3/S4 + Svc.lookup 体恢复（Svc 的泛型投影为另一域，由 same-class/nested-headers 通道处理，本片只锚体恢复）；单层/无跳转形逐字不变。

原 class 为行为基准。


## root 落点更正（2026-10-04，实现片取证后；追加段，不改写上文）

**本巡查与立项 spec 把落点钉在 `region.rs` 的 `if latches.len() != 1 { return Ok(None) }`（`fn latch_tested_loop`，10470/10483），称其为"本片要放宽的现约束"。实现者在 tasks 1.1 取证时用最小门控实验证伪了该前提，root 独立复核后确认更正。**

实现者的决定性实验：**只**放宽 `latches.len() != 1`（环境变量门控），S5 的拒绝数 **2 → 2 不变**。即该判据在 S5 的流程中**从未被行使**：

- **外层** `while` 形走的是 `fn header_tested_loop`（`region.rs:9206`），不是 `fn latch_tested_loop`（10470）——故后者的 latch 单一所有者约束根本不在路径上；
- **内层**循环**从未进入** `loop_region`，故也没有机会撞该约束。

**真实拒绝链**（实现者以 CF11DBG 插桩实测，root 读码复核其引用的机制全部存在）：块 9 的 `ifne`（源码 `if (i % 2 == 0) continue;`）**两臂的 immediate post-dominator 都是 35**（共享 latch——因内层退出边 `20 → 35` 使两臂在 35 汇合）→ **join 当选 35** → else 臂以 `next = Some(20)`（嵌套循环头）结束、但分支发布 `next = 35` → 块 20/25 **永不重访** → 内层 `loop_region` 未被调用 → `covers` 失败 → `LoopShape@4` + uncovered `[41, 20, 25]`，与本巡查冻结的诊断**逐字一致**。

即：真正的落点是 **join 选举**，不是 latch 归属计数。共享 latch 是通过"让两臂的 ipdom 汇合到同一块"间接导致内层块被跳过的。

**root 复核的实现者路线（已批准）**：当选 join == 外层 latch ∧ latch == continue_target（for-update 形）∧ 一臂为 `loop_continue_bridge`（**dj 片既有分类器**，`region.rs:3199` 调用）打到该 latch（事实 c）∧ 另一臂 next 为嵌套循环头且 `nested_blocks ⊆ blocks`（事实 a）∧ 嵌套退出边集合 == `{latch}`（事实 b）→ **把 join 重选举为嵌套循环头**（实现者后续修正为 stay 后继 18，见下）；continue 臂的 `Straight` 归属为既有 `Region::LoopContinue`（决策 2 的边语义归类，非新分类）；内层退出边沿用既有 header-test 出口通道。**不触碰 `latches.len() != 1`、不触碰 `emit.rs`/`report.rs`、Non-Goals 全不碰。**

root 核实该路线有**既有同构先例**：`region.rs:3267` 的 `in_loop_successor_join`（其说明注释在 3258-3266，消费于 3311 `let join_node = in_loop_successor_join.or(join_node);`），注释原文即 "one successor stays in the loop and its continuation reaches a *nested* loop header before this loop's own header. The in-loop successor is then where the body's statements continue — the one-armed shape the existing join reading below writes when a successor *is* the join — and **electing it keeps the nested loop and the code around it inside the body walk**"。故 join 重选举是既有机制的同一形状，**不是新造机制**。

**实现者报告的实施卡点与修正方向（root 认可）**：重选举到嵌套头 20 后，`local3 = 0`（块 18）落在 else 臂内而非循环直接前驱，被 `build.rs:13905` 拒（"preceding local initialisation did not produce one Java header clause"）。修正为把重选举目标从 20 改为 **stay 后继 18**（分支自己的体侧后继），使初始化成为循环前直邻语句——与 `in_loop_successor_join` 先例同构，且**不改 `build.rs`**（停手条件 (d) 纪律）。

**基线重验（tasks 1.1 的另一半，实现者已做、root 认可其结论）**：spec 记载的期望输出在当前 HEAD **全部成立、零漂移**——S3/S4/S5.class 的 SHA256 与本巡查记录逐字一致，javap 锚点（header@4 / latch@35 / 退出@41 / 内层@20/@29）逐指令吻合，原 class 行为基线复现（o5 = `13`/`9`/`20`/`13`、orig = `[px:3]`），当前主线渲染的拒绝与本巡查逐字相同，`outerContinueOnly`/`innerOnly` 照常恢复。故本 README 上文的所有事实性记载**仍然有效**，只有"落点是 `latches.len() != 1`"这一条被更正。

**待 root 在验收时执行的文档更正**（不在本文件内改，以免与实现片 worktree 冲突）：`openspec/changes/recover-shared-latch-nested-loops/{tasks.md,design.md,proposal.md}` 中所有把 `latches.len() != 1` 记为"要放宽的现约束"的表述，须按本段更正为"落点是 join 选举；`latches.len() != 1` 在 S5 流程从未被行使（实现者门控实验证伪，2→2 不变）"。
>
> **✅ 已执行（root 2026-10-04 验收时完成）**：上述文档更正已在合并 `6a94285a` 后落地——`tasks.md` 追加 3.3 验收记录（含门控实验证伪、真实落点为 join 选举、8 道 fail-closed guard 合取的判据复核）、`design.md` 决策 1 追加更正段、`spec.md` 的 Requirement 与负例 Scenario 措辞已更正。**同时更正了 spec 的两处字面判据**（均由 root javap 实测证伪）：(1) "内层退出边目标 **==** 外层 latch 块"过窄——必收锚 `S3.nestedBreak` 的内层退出是 `84: ifeq 153` → `153: goto 20`，即**经一次自身 transfer** 落到 latch，故须为"路由到"；(2) "三层以上共享 SHALL 保持既有拒绝"与实测不符——`ThreeLevel` 冻结 fixture 渲染 quotes=0、恢复成功且行为一致（SHA `68ca3fba…`），系同一机制逐层组合。CF-11 账本状态由"部分已测、窄首片未见差距"改为"**部分已测、首片质量差距已修复**"（总数仍 71，计数句的 45 冻结/1 已证差距不受影响）。

**教训（与 getClass 片同源，root 自己第二次犯）**：spec 的"落点"不能只靠**读代码找到看起来相关的判据**来钉死，必须用**最小门控实验**（只放宽该判据、看目标锚的行为是否变化）证伪或证成。root 两片的 spec 都因跳过这一步而把落点写错：getClass 片漏了"检查落在实参窗口内"的位置差异（实际第一道门是 `init.rs:687` 的实例读者门），本片把"latch 归属计数"当成了落点（实际是 join 选举）。两次都由实现者的取证纠正。**已固化进 handoff：写 spec 钉落点前，须先做一次最小门控实验确认该判据确实在目标锚的流程上被行使。**
