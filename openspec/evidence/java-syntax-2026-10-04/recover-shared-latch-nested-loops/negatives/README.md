# 负例与变体：实现前后对照（1.2 与追加验收 (ii)(iii)）

每个 fixture 的原 class 均以真 javac 8（Corretto 1.8.0_432）编译、`java -Xverify:all` 重放；"before"
= 巡查基线判据下的主线渲染（worktree HEAD `8cac818e`，git stash 后构建的基线 CLI 所出，全文在
`*.baseline.txt`）；"after" = 本片修复后（`*.fixed.txt`）。类字节与渲染 SHA 见 `../SHA256SUMS`。

| fixture | before（拒绝数/签名） | after | 行为 |
| --- | --- | --- | --- |
| `ThreeLevel`（三层共享 latch，登记现状） | 1（LoopShape） | **0（恢复）** | 原 `24` == 重编 `24`，SHA `68ca3fba…` 两侧一致 |
| `ExitDiverges`（内层退出落 `if (s>100)` 分歧块，不汇于回边） | 1（LoopShape） | **1（响亮拒绝，不变）** | 原 `13`（拒绝形为引用呈现，无重编腿） |
| `LabeledBreakOuter`（`break outer` 逃出回边） | 1（LoopShape） | **1（响亮拒绝，不变）** | 原 `1` |
| `Overlap`（外层 continue + 内层 break + 内层 continue） | 1（LoopShape） | **1（响亮拒绝，不变）** | 原 `300`；登记残留：外层读取已行使，内层双跳转属 dj 片闭合域（Non-Goal 不触碰） |
| S5.labeledOuter（带标签 `continue outer`，正例） | 1 | **0（恢复）** | 见 `../three-way/`（S5 四形 `13/9/20/13` 两侧一致） |

追加验收 (ii) 的判别条件-负例对应：

- 「continue 臂打到非外层 latch/非 continue 目的地」→ `outerContinueOnly`（单层 continue）：其当选
  join==22==continue_target 但 else 臂 `next=None`（无嵌套循环头），重选举判别条件为假——渲染
  **逐字节不变**（`../before-after/S5.baseline.txt` vs `S5.fixed.txt` 的该成员 diff 为空，CI 测试
  `untouched_single_continue_and_no_jump_shapes_stay_byte_identical` 钉死）。
- 「内层退出边集合 ≠ latch 路由」→ `ExitDiverges`（分歧块非 latch 且非单一后继到 latch）：保持拒绝 ✓；
  `LabeledBreakOuter`（第二退出目标逃往外层 exit）：保持拒绝 ✓。
- 「`nested_blocks ⊄ blocks`」：可归约自然循环下几何不可构造（子环块集必含于包含它的环——支配关系
  既有，design 查重表注记）；以 `is_subset` 判据代码路径存在性记录，不造非法字节码。
- 「dj 全域未行使证明」：dj 片全部测试在全量套件中渲染逐字节不变（`recover-loop-body-double-jumps`
  的测试与其 golden 全绿）；dj 形的 break 臂目标为出口链（非 continue 目的地），`elected ==
  continue_target` 判别条件为假。
