## Why

2026-10-05 的固定探针记录了循环体内具名 try/catch 的拒绝：DV.plainFor、whileLoop 为不可归约诊断，forEach 简化形另有 protected-local SSA 拒绝；无 try 循环和循环内 try/finally 为对照。历史 JADX 对其中 DF.loopCatch 曾把 throw 提出 try 之外，重编后异常逃逸。它们是待复核的具体差距，不能直接代表当前产品或所有同类形状。当前 CLI 完整类基线及固定四方法的同次 Q-i/Q-ii/Q-iii 已实际接受观察，见 [instrumentation.md](instrumentation.md)。普通catch的terminal分支所有权与foreach的try外引用写呈现为两个实际门；无限retry代表另出现region ownership overlap，仍需同次Region树定位。设计和产品实现尚未完成。

取证：[loop-body-catch-patrol](../../evidence/java-syntax-2026-10-05/loop-body-catch-patrol/README.md)（变体矩阵 + jadx 错码对照）。

## What Changes

- 优先扩展**既有** `proved_loop_catch_joins` 的循环-异常所有权证明，使其覆盖"循环体内单行具名 catch、保护区不跨循环测试块"的简单形。fragmented 认证服务于拆段异常范围与 induction-update，不能为普通单行 catch 放宽其特化条件；不新增平行机制。参考算法及原图约束见 [JADX 差异审阅](jadx-algorithm-review-root-2026-10-12.md)。
- 插桩先行（实现者在改动前回答，回答落盘进 change 目录）：
  - **Q-i**：`proved_loop_catch_joins` 的哪个合取项拒绝了本形（单后继 join？join 在循环内？header 支配？完整表范围/实际 throw-site 在循环体？行映射完备？普通路径全到 join？）——按物理 class 身份与 descriptor 逐项插桩 **DV.plainFor**；显式记录 terminal throw 的环成员及路径条件。
  - **Q-ii**：构成 "not reducible over [4,10,26,39]" 的具体边集是什么（哪条异常边使哪个自然循环多入口）；若把已证明的 catch 豁免边从归约检查中排除后图是否可归约。
  - **Q-iii**：DV.forEach 与 DF.loopCatch 的拒绝阶段分别实测；旧 protected-local SSA 拒绝不能归为普通 catch join 认证。若为独立门，拆分具体验证/实现任务，未满足三形验收前不关闭本片。
- MVP 边界：**每循环单具名 catch、单异常表行、保护区不含循环测试/递增块**先行；多行共享 handler、multi-catch、catch 内 continue/break 留待插桩结论后决定是否同片。
- 拒绝边界保持：保护区跨循环测试块、handler 有普通前驱、表行交叉——维持整方法响亮拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩充可证明的循环体内单行具名 catch 异常范围恢复，保持异常派发、效果顺序与拒绝边界。

## Impact

主要涉及 `jarde-java` 的 `region.rs`（`proved_loop_catch_joins` 与归约检查的交互）；独立 SSA/词法作用域改动须先由 Q-iii 确认落点。验收 fixture：[loop-body-catch-patrol](../../evidence/java-syntax-2026-10-05/loop-body-catch-patrol/) 的 DV.java（plainFor/whileLoop/forEach/loopNoTry）与 DF.java（loopCatch/loopFinally/default），以及已合入 fragmented-loop-catches 的既有 fixture。另保留本地 JADX `loops/TestTryCatchInLoop.java` 的无限重试与 break 代表，不能用有限 for 成功代替该边界。
