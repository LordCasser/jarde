## Why

[循环双跳转巡查](../../evidence/java-syntax-2026-10-02/loop-double-jump-patrol/README.md)以单变量判别钉死：同一循环体内两条独立循环跳转（break+continue，目标相同与否、标签与否——`if(k==1) break; if(k==0) continue;` 为最小复现）整方法拒绝；单跳转任意目标深度、三重嵌套、break+return 全部恢复。两拒绝层（for 族 `arms_do_not_meet`+`loop_shape` 级联；do-while+labeled 族 `ownership_overlap`）同源于体内单出口假设。真实代码中"满足即跳出、否则跳过本元素"的双守卫循环为高频形态。

## What Changes

- 循环体跳转边按语义归类：分支臂 join 判定区分 break 边（离开循环→循环出口）与 continue 边（→header）两种目标，双臂各自归类后循环形状证明继续；区域归属按边语义分派（continue→header 所有者、break→出口所有者），消除同块多重归属。
- L5 两最小复现（`brkSelfContSelf`/`dblJumpDoWhile`）与 L1.deepLabels/L2.triplePlain 复合恢复且行为一致；单跳转全矩阵（含双层标签、三重各深度目标）、break+return、无跳转嵌套 diff 断言逐字不变；非循环跳转组合（break+return 等既有通道）零回退。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：循环体内两条循环跳转边（break/continue，任意目标组合）可证明并呈现。

## Impact

仅 `crates/jarde-java` 私有 region.rs（臂 join 归类、区域归属分派）与循环形状证明及测试；判据为跳转边目标语义（结构事实），无新机制。既有 labeled-loop-tail-coverage、loop 系全部切片零回退。
