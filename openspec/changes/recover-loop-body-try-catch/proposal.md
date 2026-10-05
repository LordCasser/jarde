## Why

固定探针证明：**循环体内的具名 try-catch 全形被拒**——纯 for、while、for-each 三形皆拒于同一诊断 `the graph is not reducible over N block(s): a loop is entered at more than its header or two loops cross`（for-each 简化形诊断另有引注）；对照组：同循环无 try 恢复、循环内 try-finally（continue 交互）以等价分布恢复。该形状（每迭代捕获继续——解析循环/重试循环/逐项容错）是业务代码最高频模式之一。**jadx 对此形产出行为错码**（把 throw 提出try 之外使其永不被捕获，重编后异常直接逃逸方法）——jarde 的整方法拒绝是正确行为，但该形状的结构事实足以支撑证明。

取证：[loop-body-catch-patrol](../../evidence/java-syntax-2026-10-05/loop-body-catch-patrol/README.md)（变体矩阵 + jadx 错码对照）。

## What Changes

- 扩展**既有**循环-异常所有权机制之一（`proved_loop_catch_joins`（region.rs:1639）或已合入的 fragmented-loop-catches 认证路径），使其覆盖"循环体内单行具名 catch、保护区不跨循环测试块"的简单形——**不新增平行机制**。
- 插桩先行（实现者在改动前回答，回答落盘进 change 目录）：
  - **Q-i**：`proved_loop_catch_joins` 的哪个合取项拒绝了本形（单后继 join？join 在循环内？header 支配？保护区全在循环体？行映射完备？）——逐项插桩 plainFor（DF.java）。
  - **Q-ii**：构成 "not reducible over [4,10,26,39]" 的具体边集是什么（哪条异常边使哪个自然循环多入口）；若把已证明的 catch 豁免边从归约检查中排除后图是否可归约。
  - **Q-iii**：for-each 形的诊断差异（简化形无 throw-if 仍拒）——是否同一道门的不同文本，还是独立门。
- MVP 边界：**每循环单具名 catch、单异常表行、保护区不含循环测试/递增块**先行；多行共享 handler、multi-catch、catch 内 continue/break 留待插桩结论后决定是否同片。
- 拒绝边界保持：保护区跨循环测试块、handler 有普通前驱、表行交叉——维持整方法响亮拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩充可证明的循环体内单行具名 catch 异常范围恢复，保持异常派发、效果顺序与拒绝边界。

## Impact

主要涉及 `jarde-java` 的 `region.rs`（`proved_loop_catch_joins` 与归约检查的交互）；验收 fixture：[loop-body-catch-patrol](../../evidence/java-syntax-2026-10-05/loop-body-catch-patrol/) 的 DF.java（plainFor/whileLoop/for-each/loopCatch 四锚 + loopFinally/default 对照）+ 已合入 fragmented-loop-catches 的既有 fixture 零回退。
