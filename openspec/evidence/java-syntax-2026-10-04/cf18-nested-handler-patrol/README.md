# CF-18 嵌套 handler 巡查（2026-10-04）——两形定性：硬前沿，非 MVP 片

CF-18 区域相交域定向取证（主线 `30e54613`，复用 spn worktree 的 HEAD 版 jarde-cli，零额外构建）。固定转录 [fixture](fixture/)（R1：嵌套 handler × if/continue/回边两形；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 orig.out（`6`/`11`）。

## 结果矩阵

| 方法 | 源码形态 | Jarde 拒绝码 |
| --- | --- | --- |
| `run` | `for { if (i<0) { try {…} catch {continue;} } else { try {…} catch {…} } if(…) break; }` | **`jre_region_irreducible`**：6 块 `[4,9,13,27,41,50]` 图不可规约（"循环在非 header 处进入或两循环交叉"）→ 整方法不发布（完整类不可编） |
| `nestedTry` | `try { for { try { if(i==2) continue; …} catch {…} } } catch (RuntimeException) {…}` | **`jre_region_uncovered_blocks`**：7 块仅经异常边可达（normal-flow view 漏）→ 整方法不发布 |

## 架构定性（Goal 要求的“是否一定要新增机制”判断）

**两形均为硬前沿，不符合 MVP 纪律，本轮不立实现 spec**：

1. **`run`（irreducible）**：javac 把 loop-in-try-with-continue 降低为**物理块级不可规约图**（源码本身规约，但异常边 + continue 回边 + break 使标准 region decomposition 无法单入口归约）。恢复它需要**新机制**——不可规约图的结构化处理（节点分裂/多入口循环建模），触及 region 层核心，非窄切片。
2. **`nestedTry`（exception-edge reachability）**：内层 try 的 handler 使部分块仅经异常边可达，normal-flow 视图遗漏。这是 CF-18 账本登记的"区域层既有拒绝"，与 `recover-crossing-array-read-values` 追踪时记录的"真嵌套 try 两形态为区域/图构建层既有拒绝"同族——需区域层异常边归属扩展，同为中等以上机制变更。

**关键安全判断**：[CF-18 账本](../../jadx-feature-inventory-2026-09-27/cf18-exception-regions/report.md) 已记录**固定 JADX 在 `run` 同形上把外层 handler 错放到负值分支、改变异常传播语义**（运行在输入 2 处失败）——即 JADX 输出是**错的**。Jarde 保守整方法拒绝（诚实标注不可编、不发布错误源码）**比 JADX 更安全**。因此这不是"Jarde 落后于 JADX"的追赶缺口，而是"两者都不完美、Jarde 选择诚实失败"的权衡点。

## 处置

- **不立实现 spec**（避免为凑覆盖率引入不可规约图新机制，违反 MVP 与"不为文件庞大增设实体"的用户指导）。
- 登记为**区域层硬前沿**：`jre_region_irreducible`（不可规约图）与嵌套 try 异常边归属，待真实 corpus 频率证据或用户优先级驱动时再评估——若届时要碰，应先做**频率普查**（corpus 中 irreducible 拒绝占比）再决定是否值得新机制。
- 若后续要做 `nestedTry` 形（exception-edge reachability，比 irreducible 更tractable），应作为独立中等片，验收锚定"不发布错误 handler 归属"（JADX 反例为对照）。

原 class 为行为基准（`6`/`11`）。
