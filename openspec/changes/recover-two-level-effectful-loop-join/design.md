## Context

见 `proposal.md` 和 [冻结回放](../../evidence/java-syntax-2026-09-27/cf08-endless-loops/two-level-if-baseline/report.md)。`TwoLevelIf.pick` 的外层判空起于 BCI 0，内层长度条件约在 9–21；循环头 23，带效果出口 28，数组读取/命中分支 37/46，回边 49，内层 join 55，外层 join/返回 61。原/JADX 完整类 Java 8 重编和验证运行四行一致；当前 Jarde 先报外层 `arms_do_not_meet`，未覆盖循环块 `[23,28,37,46,49]`，后续局部作用域拒绝是下游症状。

已受证 `effectful_dual_exit_loop` 只接受循环 join 的两个正常前驱和双输入 SSA φ；`Frame::arm` 清掉上一层 `if_arm`，新证书只在直接顶层分支臂启用。`continue_effectful_loop_arm` 要求内部 join 后还有一段独占直线尾段到父边界。本例循环出口直接到内层 join，兄弟空臂也到这里；三种结果值在同一物理 join 汇合。现有 `continue_inner_join_arm` 虽支持单出口 `Loop + Straight`，其 `loop_arm_join_source` 要求唯一循环出口，不能拿来绕过两个真实出口。

## Goals / Non-Goals

**Goals:** 在现有 Frame/Region 关系中只增加已证的二层条件臂 + 双出口循环 + 共享内层 join 组合；证明三来源 SSA 值、各路径一次效果、内层 join 只消费一次，外层续接与源码映射完整。

**Non-Goals:** `File` 构造型效果臂、循环体中的虚调用、内层任意深度分支、异常 handler、irreducible CFG、改变跨引用局部安全门，以及固定 `TestNotIndexedLoop` 整体恢复。若完整类仍因局部呈现失败，先记录物理阻碍，再决定是否需要最小 Builder 修正，不把无关债务混入。

## Decisions

1. **为内层直接臂传递明确的父分支证据。** 只从已证明的条件分支入口给循环臂一份新的 `if_arm` 上下文，保留外层边界检查；不得因为存在 `frame.boundary` 就一般性准许嵌套循环。内层空臂须独占直线赋值并仅到 join，循环头须只有来自该臂入口和唯一 latch 的入边，所有异常/多入口边沿用原证书拒绝。
2. **让原双出口证书认领两个出口，给父分支留下 join。** 在这一路径上循环 `Region::Loop.exit` 指向内层 join，却不认领 join 本身；join 的准确正常前驱是效果出口、直接 break 出口和空臂。旧顶层/单层形态的两前驱证明保持原状。新证明还要核内层 join 后只沿一段直线到外层边界，不能让任一臂越过或二次消费 join。这里扩展的是既有局部证书/续接，非通用 CFG 重写。
3. **单独证明三来源局部 φ。** 效果出口的 `cost(7)` 返回、命中出口的数组值、空臂的 -1 各有一个确定写入；内层 join 的结果 φ 恰取这三份值，后续 `result += calls` 只消费一次并留在外层非空臂。SSA 关系、调用次数与顺序不能由相同 slot 号替代证明；缺失或额外 consumer 必须拒绝。来源按既有 Region/Builder 锚点表达，不新设 AST。
4. **以冻结全类与可验证负例验收。** 用 `two-level-if-baseline/replay.py --require-jarde` 对原/JADX/Jarde 三方完整类 Java 8 重编、验证运行，核 null/空/命中/未命中四行。另编译同层级的额外入口、异向出口、绕过 join、第四入边、双调用与异常边负例，并用 Rust 检查引用、物理块和预算/取消原子性。现有 `NestedEffectful` 和顶层效果循环必须重放。固定 `TestNotIndexedLoop` 仍红，归后续对象构造/虚调用切片。

## Risks / Trade-offs

- [把内层 join 误归循环，导致空臂结果消失] → 三个准确前驱和单次 Region 归属一同证明。
- [用 slot 号误合并三份值] → SSA φ 的三个物理来源和唯一后继消费逐项核对。
- [嵌套许可外溢到任意分支] → 只在直接臂、准确父边界与原效果证书全部通过后发布；其余按旧门槛拒绝。
- [尾段或效果重复] → 原/JADX/Jarde 四组计数运行及完整 BCI/source-map 复核。
