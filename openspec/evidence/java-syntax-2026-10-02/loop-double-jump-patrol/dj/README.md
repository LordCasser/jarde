# 循环体内双跳转恢复（recover-loop-body-double-jumps，2026-10-02 实现取证）

承接[巡查 README](../README.md)。本目录（`-dj` 专名）是实现切片的取证与验收记录：边图取证（`javap/`）、变体前后（`results-dj/*.before/after`）、恢复输出与三方对照 SHA（`results-dj/`）。

## 1. 边语义归类挂点（1.1 取证）

两最小复现的块/边图见 `javap/L5.javap.txt`。判据均为 CFG 事实（`goto` 目标与 header/出口的关系），非语法猜测：

| 复现 | 边 | 目标语义 | 挂点（region.rs） |
| --- | --- | --- | --- |
| `L5.brkSelfContSelf` 块 31/40 | 37→74（`break k`）/ 45→68（`continue k`） | 出口链 vs 回边（update 68） | 臂 join 判定 `region_at_inner`：`loop_bridge_join` 证明分类后 **join 改判 continue 目标**（68）而非 boundary；两臂验收按各自边语义接受 |
| `L5.dblJumpDoWhile` 块 8/20 | 17→2（`continue w`）/ 26→29（`break w`） | header vs 出口 | 到达自家 header 的边按回边停止（`loop_targets.last().header == node` 时不再把 header 当新循环进入），消除同块二次归属（`ownership_overlap`） |

派生挂点（复合形态所需，均由同一判据驱动）：

- **`loop_continue_bridge`**（`loop_exit_bridge` 的镜像）：体内单指令 `goto` 独占前驱地落在**本循环 continue 目标**（for 的 update 块或 while 的 header）＝ continue 边；兄弟臂的全部路由须终于本帧循环的目标集合（`loop_side_routes`，拒绝 `return` 叶）。
- **`loop_break_transfer`**：体内单指令 `goto` 独占前驱地落在**任意外层循环的 break 目标**＝ break 边（传输块可在内层自然环之外——带标签的外层 break 不会回到内层 latch）。
- **`nested_loop_ahead` + 臂内 join 改判（`continuing_join`）**：当选 join 是自家 header/continue 目标而跳跃臂不返回、兄弟臂的嵌套 `if` 已止于其自身 join 时，改选该 join，让体走查继续（嵌套循环与其后的 update 不再丢失）。
- **lbj 细化**：join 改判 continue 目标仅在「continue 目标 ≠ header 的独立 update 块」时发生；endless/while（continue 目标＝header）保持 boundary（`p3_effectful_exits` 钉死）。

诊断基线（主线）：`brkSelfContSelf`＝`jre_region_arms_do_not_meet`@31 + 三层 `jre_region_loop_shape` + `jre_region_uncovered_blocks [86,80,74]`；`dblJumpDoWhile`＝`jre_region_ownership_overlap`@2（whole-method quote）。

## 2. 判别修正（巡查矩阵的精确化）

固定 `DJLoops`（`variants/`，javac 23 `--release 8 -g:none`，原类 `java -Xverify:all` 通过）后，主线基线（`git stash` 双腿实测，`results-dj/*.before`）表明：

| 同体两跳转组合 | 主线基线 | 本切片后 |
| --- | --- | --- |
| 双 break 不同目标（`dblBreakTargets`：`break k` + `break outer`） | **已恢复**（break 边族，`loop_bridge_join` 既有通道） | 逐字不变 |
| break + labeled continue 跨两层（`brkContTwoLevels`：`break k` + `continue outer`） | **已恢复**（同上，`continue outer` 呈现为 j 层 `break loop`，行为等价） | 逐字不变 |
| 双 continue 不同目标（`dblContLabels`：`continue k` + `continue outer`） | **整方法拒绝**（`arms_do_not_meet`@31） | 完整恢复 |
| break+continue 同体（`brkSelfContSelf`/`brkSelfContMid`）、do-while+标签（`dblJumpDoWhile`/`labelWhile`）、三重嵌套复合（`triplePlain`/`deepLabels`） | 整方法拒绝（巡查 README 判别） | 完整恢复 |

即真正缺失的族是「同体任一组合包含 continue 边」（continue 边把 post-dominator 拉过外层 continue 目标而被过滤，或使区域树对同块多重归属）；纯 break 边对既有通道已覆盖。单跳转矩阵（`contSelfOnly`/`contMidLabelOnly`/`brkMidLabelOnly` + 固定 fixture L3/L4）逐字不变（`results-dj/L3/L4.jarde.after.java` 与 `../results/L3/L4.jarde.java` 零 diff；DJLoops 五个无关方法 before/after 逐字节相同）。

## 3. 三方对照（3.2）

腿：原 class / 固定 JADX dev（`jadx-cli` build install，`--no-debug-info`）/ Jarde 重编（`class-source` 输出 `javac --release 8 -g:none` 后）。全部 `java -Xverify:all`。SHA256 见 `results-dj/sha256.txt`。

| 类 | 原类 | JADX | Jarde 重编 |
| --- | --- | --- | --- |
| L1（deepLabels+labelWhile） | `05abc189…` | 一致 | 一致（`orig.out` 逐行相同） |
| L2（triplePlain 复合） | `3ec307c2…` | 一致 | 一致（`o2.out`） |
| L5（两最小复现+brkSelfContMid） | — | 一致 | 前三行一致（含两最小复现路径）；第 4 行（`dblJumpDoWhilePlain`，break+return 既有通道）与本变更前逐字相同地保留 BCI 26/28 引用——编译后为 `i=10` ≠ 原类 `early`，为**变更前既有**的行为偏差，本片不碰（spec：break 与非循环控制组合逐字不变） |
| DJLoops（六方法） | `63874159…` | 一致 | 一致（六行全同） |
| DJTripleJump（负例） | — | 一致 | 按边界拒绝（三跳转登记现状），Jarde 腿不适用 |

## 4. 门禁（3.1）

- `cargo test --workspace --tests --locked --no-fail-fast`：**2839 通过 / 0 失败**（284 套件；主线 2832 + 本片 7 测试；本轮无 flake）。期间两处真实回归（`p3_loop_transfers`、`p3_labeled_loop_tail`）由 lbj 细化的 endless-loop 误判与 `LoopContinue` 边臂未纳入验收引起，修正后全绿——前者以 `continue_target != header` 收紧细化、后者把「臂止于已证 `continue` 转移」与 break 边臂同权。
- `cargo fmt --all -- --check`：干净。
- clippy：CI 完整 29 项 `-A` 清单（`.github/workflows/ci.yml` 实有）+ `-D warnings`：0 警告。
- `openspec validate --all --strict --no-interactive`：246/246。
- 新增回归 `crates/jarde-java/tests/p3_loop_body_double_jumps.rs`（7 测试）：双跳恢复、复合恢复、传输块单归属、单跳转矩阵钉死文本（两方法全文逐字）、break+return 引用边界、三跳转拒绝、预算/取消原子性。

## 5. 遗留缺口

- **三条及以上体内循环跳转**：登记现状拒绝（`DJTripleJump.tripleJump`，`jre_region_ownership_overlap`），前后输出逐字相同。
- **跳转边与守护（finally/TWR）边交叠**：不在本片；守护通道优先顺序未动。
- **`dblJumpDoWhilePlain`（break+return）**：既有通道保留 BCI 26/28 引用（行为偏差为变更前既有），按 spec 不碰。
- **emit 的 `else if` 缩进怪癖**（嵌套体保持外层绝对缩进）：变更前既有呈现，非本片范围。
