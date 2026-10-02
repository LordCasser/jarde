# do-while(false) 体内 return 恢复 + 引注失败闭合（recover-return-in-do-while-false，2026-10-02 实现取证）

承接[巡查 README](../README.md)。本目录（`-dw` 专名）是实现切片的取证与验收记录：块/边取证（`javap/`）、变体前后（`results-dw/*.before/after`）、恢复输出与三方对照 SHA（`results-dw/`）。基线主线 `cc272d0f`（含 loop-body-double-jumps）。

## 1. 双面块/边取证与闭合落点（1.1）

SHA 核对通过（L5 `baaf9a55…`、D1 `5726a17b…`，见 `results-dw/sha256.txt` 与巡查冻结记录一致）。块/边全图见 `javap/`。两面的块计费与引注落点：

| 面 | 关键边 | 主线区域分解（class-source 实测） | 引注 |
| --- | --- | --- | --- |
| L5.dblJumpDoWhilePlain | 块 20 `if_icmple 2`：taken→2（header）、fall→26（`ldc "early"; areturn` 叶） | straight[0] + **loop[2,8,17,20]** + straight[29] + fallback[26]（`jre_region_uncovered_blocks`） | 块 26 落入方法末尾 uncovered 引注；剥引注后类可编译，运行 `i=10` ≠ 原类 `early` ——**静默错编** |
| D1.retInDoWhile | 同上但 taken→48（`hits++`）→56 `goto 2`；叶 26 为 10 指令 StringBuilder 链 `…areturn` | straight[0] + **loop[2,8,17,20,48]** + straight[59] + fallback[26] | dj 切片合入后与 L5 同面：部分体 + 叶引注，剥后可编译、`i=10` ≠ `early:5` ——**巡查记录的"安全拒绝"已随 dj 合入变为静默错编**（巡查时 `83ed9a2e` 的 `hits++` 计费不平拒绝不再发生：dj 的边归类让 loop 拥有 48） |

**根因（读引注触发条件）**：do-while(false) 编译后无回边、不构成 CFG 环——体只是外层循环体的一段直线加它的出边。条件 return 的叶（`areturn` 终结、无后继）不在循环自然块集内，也不满足 CF-07 末端叶证明的 `[iload N; ireturn]` 形状 → 不进 body scope → 分支臂走查在叶上被 `stops_at`（scope 外）截空 → 叶留给 uncovered 引注。`loop_body` 的 `terminal_returns` 通道（region.rs，CF-07 建立）就是"体拥有 return 叶"的既有机制——本片泛化它，失败闭合则落在新全局处（见 §2）。

## 2. 恢复归类与闭合不变量的实现位置

**恢复（design 决策 1）**：`region.rs::loop_terminal_returns`——体分支的独占前驱无后继块、终结指令为任一已解码 `return` 操作码（七种之一，值形状不限），即归类为方法退出边并入 body scope/expected（与 `loop_break_transfer`/`loop_continue_bridge` 的 goto 语义归类正交：return 不是循环边）。CF-07 的 `[iload; ireturn]` 形状与"恰一片叶"限制移除；`sharedLeaf`（双前驱）、`exceptionalLeaf`（异常出边）等既有排除不变。呈现按 dj 切片确立的等价形式：do-while(false) 体呈为其语句集（无回边故无 `do … while (false)` 字面包装），break 边呈空臂、return 边呈 return 臂——与 `dblJumpDoWhile`（break+continue labeled）根接受的呈现一致。

**失败闭合（design 决策 2，全局）**：分类器 `region.rs::quoted_control_flow_exit`（pub(crate)）——在完成的区域树上，引注区（fallback 区）块含 (a) `return`/`athrow` 终结，或 (b) 单 `goto` 且目标不属于任何区域（break/continue 外逃）时命名该边；两条护栏：无结构区的整方法引注不分类（已是拒绝）；引注须被呈现结构经**普通边**到达（仅异常边可达的 handler 引注不动——partial 体不声明异常路径）。**判定落点在 `build.rs` 引注落点全局处**（与 local-crossing/early-return-tail 同族的方法级升级并列）：region 层判不了"剥引注后是否仍可编译"，而提案原文的不变量正是"含控制流改变语句**且剥离引注后仍可编译**"——`build.rs::completes_normally` 按 JLS 14.22 语义（跳过尾部引注、try/catch 任一可完成、finally 不可完成则整体不可完成）读已建语句树，void/构造器恒可编译；两者相与才升级为整方法拒绝（`multiCatch` 因此豁免：其空 catch 落尾，剥引注后缺 return 不可编译——保留既有部分体，见守卫测试）。

## 3. 变体判别矩阵（1.2，DWVariants，javac 23 `--release 8 -g:none`，`java -Xverify:all` 通过）

固定 `DWVariants.class`（`999db75f…`）。主线（before，`git stash` 双腿实测）与本片后（after）：

| 变体 | before（主线） | after | 行为差判定 |
| --- | --- | --- | --- |
| `throwInDoWhile`（throw 叶读局部 `i`） | 整方法拒绝（local-crossing 升级） | 同左（class-source 语境）；方法级 harness 语境（facts 无 LVT）由闭合拒绝 | 两态皆拒绝，无静默错编 ✓ |
| `throwConstDoWhile`（throw 叶读常量——**throw 静默错编面**） | 部分体 + 空臂 + 叶引注，剥后可编译、异常被静默丢弃（`i=10`、无异常 ≠ 原类 `early` 异常） | **闭合触发**：整方法拒绝，理由点名 `quoted block at BCI 26 ends in a control-flow exit at BCI 35`（athrow） | 消除静默错编 ✓ |
| `doubleReturnDoWhile`（同体双条件 return） | 整方法拒绝（local-crossing） | **完整恢复**：`return "a" + i;` / `return "b" + i;` 双臂（`a8` ✓，单独重编运行一致） | 恢复 ✓ |
| `sharedLeafDoWhile`（`\|\|` 拼写的 return 叶双普通入口——不可恢复负例） | 整方法拒绝（`arms_do_not_meet`@26） | 同左（逐字） | 登记拒绝保持 ✓ |

## 4. 既有边界翻转（如实记录，供 root 3.3 复核）

恢复归类为全局语义（root 决策 1"与 break/continue 归类正交"的自然结果），既有登记边界随证明增强而移动，各测试已按本片有意行为更新：

| 既有登记 | before | after | 依据 |
| --- | --- | --- | --- |
| cf07 `otherValue`（`return array[i]`，登记为"非 `[load;ireturn]` 形状保持引注"） | local-crossing 整方法拒绝 | **完整恢复** `return arg0[local4];` | 叶形状泛化；`p3_loop_terminal_return` 负例表移出、新增恢复钉死 |
| cf08 `differentTarget`（效果双出口证书负例表） | 循环引注部分体 | **完整恢复**（return 叶独占前驱被体拥有；`return v + calls` 臂 + 尾部双出口，与字节码逐路径一致） | 同上；`p3_effectful_exits` 拆分断言 |
| cf07 `extraEntry`（补丁类：叶双入口） | 部分体 + 尾引注 | 同左（闭合被完成性门槛豁免：剥后缺 return 不可编译） | 未变 ✓（守卫测试钉死） |
| p5 corpus 计费 | analysis_steps 10533/17318 | 10539/17324（闭合分类器对引注体的边扫描记账 +6） | 钉值更新 |
| L5.dblJumpDoWhilePlain（dj 片零回退锚） | 部分体 + BCI 26/28 引注 | **完整恢复**（本片验收目标） | `p3_loop_body_double_jumps` 锚更新 |

主线上 D1.retInIf 同为静默错编面（`end` ≠ `f2`）；本片后恢复（`f2` ✓）。巡查 README 中"D1=安全拒绝"的记述以本目录 before 实测修正。

## 5. 三方对照（3.2）

腿：原 class / 固定 JADX dev（`--no-debug-info`）/ Jarde 重编（`class-source` 输出 `javac --release 8 -g:none`）。全部 `java -Xverify:all`。SHA256 见 `results-dw/sha256.txt`。

| 类 | 原类 | JADX | Jarde 重编 |
| --- | --- | --- | --- |
| L5 | `000 010 100 110 200 210 / i=8 / early` | 逐字一致 | **逐字一致**（整类重编通过，`early` ✓；其余已恢复方法逐字不变） |
| D1 | `early:5 / d3 / f2` | 逐字一致 | **逐字一致**（整类重编通过——retInDoWhile/retInPlainDo/retInIf 三方法全部恢复） |
| DWVariants | `early8 / early / a8 / early8` | 逐字一致 | 整类不编译（三拒绝方法无语句=安全失败）；恢复方法 `doubleReturnDoWhile` 单独重编运行 `a8` 一致；`throwConstDoWhile` 闭合拒绝（无"可编译且行为不同"输出） |

## 6. 门禁（3.1）

- `cargo test --workspace --tests --locked --no-fail-fast`：**全绿**（2843+ 含本片新增 `p3_do_while_return` 6 测试；本轮无 flake）。
- `cargo fmt --all -- --check`：干净。
- clippy（CI 实有 29 项 `-A` + `-D warnings`）：0 警告。
- `openspec validate --all --strict --no-interactive`：248/248。
- 磁盘纪律：每轮构建/测试前 `df -h /` 检查（最低 42Gi 可用）；报告前 clean。

## 7. 遗留缺口

- **字面 `do { … } while (false);` 包装**：呈现按 dj 切片确立的等价扁平形式（体语句集）；无回边的 do-while(false) 在 CFG 中不构成环，字面包装需要合成包装器，属呈现层后续。行为已三方逐字一致。
- **语句层引注（build 级 Fallback stmt）不进闭合分类**：本片分类器读区域树；build 层因声明/渲染失败产生的语句级引注保持既有登记行为（全量测试零回退，未见该族静默错编面）。
- `jsr/ret`、continue 目标外逃的更广闭合：按 Non-Goal 不做；闭合的 goto-escape 子句已按"目标不属于任何区域"实现自然外延。
- D1.retInDoWhile 的 return 臂值为原始 StringBuilder 链呈现（`new StringBuilder().append("early:").append(D1.hits).toString()`），未被 concat 规则折叠为 `"early:" + hits`——行为等价（`early:5` ✓），呈现与 concat 通道的差异留待该规则扩展。
