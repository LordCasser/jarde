## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/loop-double-jump-patrol/README.md)：两最小复现的诊断（`brkSelfContSelf`：块 31 分支臂不汇合 → 三层 loop_shape 级联；`dblJumpDoWhile`：块 2 多重归属）。既有通道：labeled-loop-tail-coverage 的 next-guard（单跳转尾覆盖）、`arms_do_not_meet` 的臂归并（enclosing-named-catch、switch-loop-join 等切片曾扩展）、循环形状证明（header/test/exit/latch）。**第一个取证义务**：javap 两最小复现的块/边图（break 边与 continue 边的目标差异——goto 循环出口 vs goto header），读臂 join 判定与区域归属各自处理体内跳转边的位置，确定语义归类的挂点（很可能两处在同一 pass 的相邻判定）。

## Goals / Non-Goals

**Goals:** 恰两条体内循环跳转边（MVP：break+continue 任意标签/目标组合）恢复；单跳转矩阵与既有通道逐字不变。**Non-Goals:** 三条及以上（登记）；跳转边与非循环控制（return/throw）的任意组合已由既有通道覆盖不碰；switch 内双跳转（switch-loop-join 既有）；跨方法。

## Decisions

1. **边语义归类**：臂 join/循环形状证明按跳转边目标语义分派——目标在循环外（出口链）= break 边、目标为 header/latch = continue 边；双臂各自归类后各自证明，不再要求单 join。归类依据是 CFG 事实（goto 目标与 header/出口关系），非语法猜测。
2. **区域归属分派**：两条边驱动的归并按同一语义分派归属（continue→header 所有者、break→出口所有者），消除 overlap；保持先既有单边通道、双边才走新分派的顺序。
3. **验收锚定**：L5 两最小复现 + L1/L2 复合（行为基线见 fixture orig/o 文件）+ 变体（双 continue 不同标签目标、双 break 不同目标、break+labeled continue 跨两层）；负例（三跳转登记现状、单跳转矩阵逐字不变断言）。

## Risks / Trade-offs

- **break 边目标非直接出口（经中间块）** → 归类按"目标是否最终离开循环"判定（可达性到出口不经过 header）；不能判定的边保持现拒绝。
- **与 finally/TWR 边交叠** → 跳转边语义判定遇到守护边时先走守护通道（既有顺序不变），本片不覆盖守护内双跳转（登记）。
