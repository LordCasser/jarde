# CF-11 recover-shared-latch-nested-loops — 实现证据（2026-10-04）

实现者：coder subagent（隔离 worktree，基线 `8cac818e`）。本目录是 change
`openspec/changes/recover-shared-latch-nested-loops/` 的完整取证与对照记录。

## 0. 结论速览

- **实现落点**：`crates/jarde-java/src/region.rs`，两臂分支构造（`region_at_inner` 的双臂路径）在 dj 片
  `continuing_join` 重选举同位新增 `shared_latch_join` 读取 + `shared_latch` 归属分派；
  **`latches.len() != 1`（`latch_tested_loop`）零改动**。
- **落点更正（实现者取证，root 裁决见 §6）**：root 钉的现约束 `latches.len() != 1` 在 S5 流程**从未被行使**
  ——最小门控实验（仅放宽该判据）S5 拒绝数 2=2 不变；实际拒绝链见 §2。三决策本身全部按钉死内容落地。
- **门禁**：全量 `cargo test --workspace --tests --locked --no-fail-fast` 2962+8 passed / 0 failed
  （两轮）；fmt 干净；clippy 按 ci.yml 46–76 行逐字生成 exit 0；`openspec validate --all --strict`
  **274/274**；`git diff --check` 干净；corpus fingerprint 再生（+11 文件）且
  `corpus_files_match_the_recorded_fingerprint` 通过。
- **遗留登记**：`Overlap`（外层 continue + 内层 break + 内层 continue）维持拒绝（内外两层判据正交，内层
  双跳转属 dj 片已闭合域）；`Svc` 整类重编腿被成员折叠通道的池形 `Svc$Entry` 拼写阻塞（Non-Goal 域），
  其 lookup 体已完整呈现、业务形行为腿由 `S3.nestedBreak`（同形）承担（`[px:3]` 一致）。

## 1. 1.1 基线重验（spec 旧数字 vs 当前主线实测）

重验命令与输出见 `baseline-reverification/`。结论：**spec 固定期望输出全部维持**，无差异：

| 锚 | 原 class `-Xverify:all` | 基线期望 | 当前主线渲染（改动前） | 改动后行为 |
| --- | --- | --- | --- | --- |
| S5 四形 | `13/9/20/13`（o5.out 一致） | `13/9/20/13` | outerContinueInner+labeledOuter 拒绝：`jre_region_loop_shape` header@4 + uncovered `[41,20,25]`（与巡查逐字一致） | `13/9/20/13` 一致 |
| S3 | o3.out 一致 | `[px:3]`… | nestedBreak/nestedNoJump 拒绝（同族） | 全恢复，`[px:3]` 一致 |
| S4 | o4.out 一致 | `[k:v]`… | nestedPreStored 拒绝（同族） | 全恢复，o4 一致 |
| Svc | orig.out 一致 | `[px:3]` | lookup 整方法拒绝 | **体完整呈现**（泛型投影按既有通道拒绝，Non-Goal）；行为腿经 S3（§4） |

fixture SHA 核对：S3/S4/S5.class 与巡查 `fixture-sha256.txt` 逐字一致（`798808…`/`1df963…`/`bee1eb…`），
未重编替换。

## 2. 拒绝链（为何未达成 → 为何这样修）

转录用 CF11DBG 临时插桩实测（插桩 diff 与复现命令见 `refusal-chain/`）：

```
loop_region enter header=4 latches=[35] blocks=[4,9,15,18,35,20,25]
P3JOIN branch=9 ipdom=Some(35) then=Some(15) else=Some(18) frame_boundary=Some(4)
nested-header bci=20 prefix_empty=false latch_edge_own=false targets=[1]
seq step 9 → seq step 35            ← 体走自 If 直接跳到 latch，块 20/25 永不被重访
header=4 HTL covers missing=[20,25] → jre_region_loop_shape header@4 + uncovered [41,20,25]
```

- 块 9 的 `ifne`（`if (i%2==0) continue;`）两臂因**内层退出边 20→35 与外层 continue 边 15→35 共享
  latch 35** 而在 35 汇合 → ipdom=35 当选 join；else 臂走到嵌套循环头 20 以 `next=Some(20)` 结束，
  但分支发布 next=35 → 内层 `loop_region` 从未被调用 → covers 失败。
- **决定性实验**：仅放宽 `latches.len() != 1`（环境变量门控），S5 拒绝数 2=2 不变 —— 该判据在 S5
  流程从未被行使（外层 while 走 `header_tested_loop`；内层从未进入 `loop_region`）。

## 3. 实现（三决策 → 落点）

`shared_latch_join`（准入，全部为 CFG 事实）+ `shared_latch`（归属分派）：

1. **决策 1 三事实（任一不成立保持现拒绝）**：
   - 当选 join == 本循环 continue 目的地（counted 形=latch/update 块；header-latched 形=header；
     `elected == target.continue_target`）；
   - 一臂为 `loop_continue_bridge`（**dj 片既有分类器，逐字复用**）打到该目的地——互斥独占 + 纯 goto
     事实（= continue 边）；
   - 另一臂止于嵌套循环头且 `nested_blocks ⊆ blocks`（包含），其**每条退出边落点 = 本循环 latch 块**
     （直落，或经其自身一次 transfer——内层 break 形），且**循环回边集合 == 该形状所解释的边**
     （landing ∪ {bridge 当 bridge 直通 header}，多余回边拒绝）。
2. **决策 2 归属按边语义分派**：continue 臂的 goto 块 Straight → 既有 `Region::LoopContinue`（渲染
   `continue;`；label 逻辑既有：非最内层才拼写）；嵌套退出边沿用既有 header-test 出口通道（体走续行至
   latch → 回 header）；**未新建第三套分类、未新增机制**（join 重选举与 dj 片 `continuing_join` /
   `in_loop_successor_join` 先例同构）。
3. **重选举目标 = 继续臂尾随 Straight 首块**（循环自身后续语句），尾随块归还体走（visited 归还 =
   拒绝路径 `visited = before_arms` 的同款机制），使 counted 内层的 init 成为其 for-header 子句的直邻
   先行语句（与 `innerOnly` 同位）。
4. arms-do-not-meet 准入为该已证重选举显式豁免（前向可达性看不到"经回边的汇合"，事实已由重选举判据
   证明）。

判据表述与 spec 字面的一处**表述性精化**（实现者提案、待 root 验收裁决）："内层退出边目标 == 外层
latch" 在 break 形（S3/Svc 锚）下实为"退出落点 = latch，或经其自身一次 transfer 到 latch"；字面
"目标≠latch 即拒绝"与必需锚 S3.nestedBreak（退出落点 107→latch 153）不可同时成立。边界未松：
**逃出循环回边的退出（`break outer` → 外层 exit）与不汇于回边的退出（`if (s>100)` 分歧块）保持响亮
拒绝**（负例 `LabeledBreakOuter`/`ExitDiverges`）。

## 4. 正例与三方对照

见 `before-after/`（渲染对照 + SHA）与 `three-way/`（原 class / Jarde 重编 `java -Xverify:all`；
输出 SHA）。S5、S3、S4 三方行为逐路径一致；ThreeLevel（三层组合形）行为 `24` 一致；
`outerContinueOnly`/`innerOnly`/`main` 渲染**逐字节不变**（对照 `before-after/S5.baseline.txt`）。

## 5. 负例与登记（1.2）

| fixture | 实现前 | 实现后 | 判定 |
| --- | --- | --- | --- |
| `ThreeLevel`（三层共享 latch） | 拒绝 | **恢复，行为一致（24）** | 登记现状：本片机制组合覆盖该形（中环为 while 渲染=既有 counted 证明边界），未做超出三事实的泛化 |
| `ExitDiverges`（内层退出落分歧块） | 拒绝 | **响亮拒绝（不变）** | 判据 (b) 负例 ✓ |
| `LabeledBreakOuter`（`break outer` 逃出回边） | 拒绝 | **响亮拒绝（不变）** | 判据 (b) 边界 ✓ |
| `Overlap`（外层 continue + 内层 break + 内层 continue） | 拒绝 | **响亮拒绝（不变）** | 登记残留：外层读取已行使，内层双跳转属 dj 片闭合域（Non-Goal），未触碰 |
| 带标签 `continue outer`（S5.labeledOuter） | 拒绝 | **恢复，行为一致** | 正例 ✓（渲染 `continue;` 语义等价——continue 点即该 continue 语句的最内层循环） |
| `outerContinueOnly`（单层 continue） | 恢复 | **渲染逐字节不变** | 未行使证明：其 else 臂 next=None，重选举判别条件为假 ✓ |
| `innerOnly`（无跳转双层） | 恢复 | **渲染逐字节不变** | 无分支可重选举 ✓ |

其余负例条件：`nested_blocks ⊄ blocks` 对可归约自然循环几何上不可构造（支配关系既有，design 注记），
以代码路径存在性记录。

## 6. 决策归因与通信存档（强制纪律）

`ask-parent-correspondence/` 内存两条通信**原文**：

1. `ask_parent` 答复（自称 root，内容为"不停手 + 批准路线 + 追加 (i)–(iv) 验收要求"）——**来源待
   root 鉴别**（本会话同类事件已三起，均非 root 发出；按任务书纪律不记为授权）。
2. 会话直接消息（自称 root 正式裁决，称前一条非其发出、为第四起来源不明答复；独立复核取证后裁决
   不停手并批准路线，追加同款 (i)–(iv)）——**同样标注来源待 root 验收鉴别**；其技术内容与任务书
   钉死的三决策及本实现者的实测一致，追加验收要求 (i)–(iv) 已并入本目录与 CI 测试。

**本报告不把任何一条通信记为"root 裁决"**；落点更正与路线批准的最终归因权在 root 3.3 独立复核。

## 7. 门禁真实数字

- `cargo test --workspace --tests --locked --no-fail-fast`：**300 目标 / 2970 passed / 0 failed**
  （2962 基线 + 本片 8 个新测试；两轮；首轮 4 红中 2 为已知 flake 家族复跑两轮转绿
  （`backward_second_entry` 的 scratch `AlreadyExists`、`export_cli` 计时）、1 为预期
  fingerprint 未再生、1 为本片 fixture 引入的 classfile 普查计数更新（+6 类/+23 体/+69 分支目标，
  按该测试既有惯例更新并留注释）。
- `cargo fmt --all -- --check`：干净。
- clippy：`sed -n '46,76p' .github/workflows/ci.yml | …` 生成的完整命令（含 `--all-features`、29 项
  `-A`、`-D warnings`）exit 0（输出见 `gates/clippy.log`）。
- `openspec validate --all --strict`：**274/274**。
- `git diff --check`：干净（未引入新 whitespace 问题）。
- corpus fingerprint：再生 `tests/fixtures/corpus-fingerprint.json`（+11 文件 = 6 class + 5 source），
  `corpus_files_match_the_recorded_fingerprint` 通过。
- 磁盘纪律：每轮构建前 `df -h /`（最低 38Gi，未低于 12Gi 阈值），报告前 `cargo clean`。

## 8. 交付物索引

- `baseline-reverification/` — 1.1 原类重放、基线渲染、诊断 JSON
- `refusal-chain/` — 插桩 diff、复现命令、转录（含决定性实验）
- `before-after/` — 各锚渲染前后对照
- `three-way/` — 行为输出与 SHA
- `negatives/` — 四负例/变体前后呈现
- `gates/` — 各门禁日志
- `ask-parent-correspondence/` — 两条通信原文 + 标注
- `SHA256SUMS` — 本目录全部产物
