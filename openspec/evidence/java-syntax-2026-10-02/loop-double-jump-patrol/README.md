# 循环体内双跳转巡查（2026-10-02）

labeled break/continue 深嵌套域定向取证（主线 `04fee28d`）。固定转录 [fixture](fixture/)（L1 复合 / L2–L5 判别链；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）。

## 判别矩阵（单变量钉死）

| 场景 | 主线 Jarde |
| --- | --- |
| 双层 labeled continue/break（任目标）、三重嵌套无跳转、三重各自单跳转（break/continue 自身/中层/最外层，带标签与否）、do-while(false) 单 break、break+return | 全部恢复 |
| **同一循环体内两条独立循环跳转**（`if(k==1) break; if(k==0) continue;`——两目标同为最内层也败；`break`+`continue mid`；do-while 体内 `continue w`+`break w`） | 整方法拒绝 |

最小复现：`L5.brkSelfContSelf`（`jre_region_arms_do_not_meet`@31 + 三个循环 `jre_region_loop_shape`——单体内两个出口边使分支臂不汇合、循环形状不可证）；`L5.dblJumpDoWhile`（`jre_region_ownership_overlap`@2——两跳转使区域树对同一块多重归属）。L1/L2 的复合失败同因。

## 根因

循环形状证明（loop header/test/exit/latch 子集）与区域树归属均按**每体单出口**假设处理体内跳转：同一体内出现第二条循环跳转边时，(a) for 族——分支臂出现两个不同 join 目标（break 出口与 continue 回边），`arms_do_not_meet` 级联三层 loop_shape 拒绝；(b) do-while(false)+labeled 族——两条边各自驱动区域归并，同块多重归属。既有 labeled-loop-tail-coverage 的 next-guard/标签通道处理的是**单跳转的尾覆盖**，未建模双出口体。

## 处置方向

`recover-loop-body-double-jumps`：循环体证明接受体内两条循环跳转边——(a) for/while 族：分支臂的 join 判定区分 break 边（离开循环）与 continue 边（回 header）两种语义目标，两臂各自按目标归类后形状证明继续（不要求单 join）；(b) 区域归属：双边驱动的归并按边语义分派归属（continue 边归 header 所有者、break 边归循环出口所有者）。判据为值/结构事实（跳转边目标语义），无新机制。L5 两最小复现 + L1/L2 复合恢复；单跳转全矩阵与 break+return diff 逐字不变。

原 class 为行为基准。
