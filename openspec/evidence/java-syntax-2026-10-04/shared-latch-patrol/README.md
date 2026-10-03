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
