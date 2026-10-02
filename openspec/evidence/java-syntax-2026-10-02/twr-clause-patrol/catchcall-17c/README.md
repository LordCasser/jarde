# TWR 包围 catch 子句体调用语句恢复（`recover-enclosing-catch-call-bodies`，CF-17c）证据

[巡查 README](../README.md) 切片 B 的实现侧证据：TWR+包围 catch 且子句体为**调用语句**（`log.append("E")`
——非 void invoke + 紧随 pop 丢弃）整方法拒绝的根因是 17b 的 `enclosing_clause` 体语句白名单未接
17a 的丢弃调用判据。本片在白名单枚举处接入该判据，单变量翻转；判据复用，无第二份实现。

## 1. 枚举位取证与接入形态（tasks 1.1）

- **枚举位**：`crates/jarde-java/src/guard.rs::enclosing_clause`（17b 全跨度子句证明）的体语句循环
  ——绑定 store 之后逐指令判 `statement_carried(bci) || Return`。`log.append("E")` 的 javac 下降
  `astore(绑定); getstatic; ldc; invokevirtual append; pop` 中，`getstatic/ldc/invoke` 均在
  `statement_carried` 白名单（`Operation::Field/Push/Invoke`），唯独 `pop` 不在——这正是巡查判别
  钉死的单变量。
- **17a 判据**：同文件 `Facts::discarded_call_pop`（17a 为 TWR 体检查点建立的判据：pop 单读值、
  同块紧邻 invoke 写值 SSA 同、invoke 不落局部槽、pop 为该值唯一读者）。**接入形态匹配**：
  白名单循环直加一个 `|| facts.discarded_call_pop(step.bci())` 谓词调用即可，无需提取共享 helper
  （判据本体与 17a/17b 的 `statement_free_with_discarded_calls` 用例同一份，未复制）。
- **呈现链路零改动取证**：clause 体经 build.rs 的普通语句通道 `body_range` 呈现；`pop` 的呈现侧
  由 17a/P3 2c.31 机制（`Builder::discarded_evaluations` 的 `accounted`/`discards` 表 + 语句写
  `Operation::Other` 分支）承载，与 TWR 体侧同一套——本片未改 build.rs。
- **Catches 通道取证（root 决策 2）**：普通（非 TWR 包围）`try { … } catch (E e) { log.append("E"); }`
  的 catch 体丢弃调用**主线已恢复**（探针 `OCprobe.plainCatchDiscard` 于实现前构建重放，子句体
  `OCprobe.log.append("E");` 完整呈现；clause 体走 Region 树普通语句通道，不经本片白名单）——
  按决策**不动**。
- **基线重放**：主线（ee35101b）重放巡查冻结 P2/P3——`plainCatch`/`voidBodyCatch` 整方法拒绝
  `jre_guard_unexplained_row`，与巡查 `results/P2-plainCatch.base.json`/`P3-voidBodyCatch.base.json`
  一致；17b 冻结 fixture（T3/T1/W17b）恢复文本复核一致（实现前后逐字节比对见下）。

## 2. 冻结变体与负例（tasks 1.2，逐一 `java -Xverify:all` 通过）

fixture：[W17c.java](W17c.java) → [original/W17c.class](original/W17c.class)（javac 23.0.1
`--release 8 -g:none`，main 末位置入负例使恢复腿截断点可读）；[N17cGen.java](N17cGen.java)
（JDK 23 Class-File API）→ [original/N17c.class](original/N17c.class)。SHA 见
[results/fixture-sha256.txt](results/fixture-sha256.txt)（巡查冻结 P2/P3 的 SHA 复核与
[results/fixture-sha256.txt](../results/fixture-sha256.txt) 一致）。

| fixture 成员 | 形态 | 原类运行 | 实现前 | 实现后 |
| --- | --- | --- | --- | --- |
| `W17c.normalReturn` | catch 体 `return`（17b 形对照） | done | 恢复 | **逐字不变** |
| `W17c.callCatch` | catch 体 `log.append("E")`（P3.voidBodyCatch 同形） | done | 拒绝 | **恢复** |
| `W17c.bodyThrows` | 体抛 ISE 真实走子句（注入 E 路径） | done | 拒绝 | **恢复** |
| `W17c.callThenReturn` | 多语句子句体：丢弃调用 + `return "caught"` | caught | 拒绝 | **恢复** |
| `W17c.twoCalls` | 子句体两个丢弃调用 | done | 拒绝 | **恢复** |
| `W17c.chainedConsume` | `log.append(e.getMessage())`（链式消费，外层丢弃） | done | 拒绝 | **恢复** |
| `W17c.closeThrows` | close 的 ISE 直落具名子句（closeBoom=true） | done | 拒绝 | **恢复** |
| `W17c.suppressedBoth` | 体与 close 双抛，suppression 链 + 拼接消费 | done | 拒绝 | **恢复** |
| `W17c.branchBody` | 子句体含 `if/else` 分支（javac 合法形态） | done | 拒绝 | **仍拒绝**（本片负例） |
| `P3.voidBodyCatch` / `P2.plainCatch` | 巡查冻结锚点 | t[c] 等 | 拒绝 | **恢复** |
| `N17c.popWrongValue` | 子句体 `invoke; dup; pop; pop`（pop 读副本，非调用结果） | done | 拒绝 | **仍拒绝** |
| `N17c.popSecondReader` | 子句体 `invoke; dup; astore_1; pop`（结果被消费形：副本入库 + 丢弃并存） | done | 拒绝 | **仍拒绝** |
| `N17c.popAfterCast` | 子句体 `invoke; checkcast; pop`（调用与 pop 间插指令） | done | 拒绝 | **仍拒绝** |

前后整类文本：[results/before-\*.jarde.java](results/) vs [results/after-\*.jarde.java](results/)。
逐字节核对：实现前后 **T3/T1/W17b（17b 冻结家族）与 N17c 全类文本不变**；P2 仅 `plainCatch`、
P3 仅 `voidBodyCatch`、W17c 仅 8 个调用形态成员翻转（`branchBody`/`branchNoCatch` 等既有拒绝
逐字不动；P3.voidBodyBranch/voidBodySoloFin 等 direct-finally 锚点逐字不动）。

负例守卫映射：三分支手降负例各落空判据一条（pop 前一指令非 invoke / invoke 写值另有读者 /
pop 与 invoke 不同邻），子句证明回到 `None`，全跨度行维持 `jre_guard_unexplained_row` 拒绝
（与 N17b 六负例同一诊断；分支体 javac 负例经 transfer 指令落空白名单，同诊断）。N17b 全家族
负例在实现后重放，拒绝逐字不变。

## 3. 三方对照（tasks 3.2，[results/three-way/](results/three-way/)）

原 class / 固定 JADX（dev，`--no-debug-info`，`jadx-cli/build/install/jadx/bin/jadx`）/ Jarde
`class-source` 恢复文本重编，全部 `java -Xverify:all`，逐路径输出与 SHA 见
[run-sha256.txt](results/three-way/run-sha256.txt) / [leg-source-sha256.txt](results/three-way/leg-source-sha256.txt)：

| 类 | 原 class | JADX | Jarde |
| --- | --- | --- | --- |
| P3（main 三路径含 voidBodyCatch 锚点） | `t[c]`/`t[c]t[c]`/`t[c]t[c]t[c]f` | 逐字一致 | **逐字一致**（整类恢复） |
| P2（plainCatch 锚点） | `B[c]`/`B[c]b[c]` | 逐字一致 | plainCatch 经 Driver2 双腿 `b[c]` 一致；`branchNoCatch` 为 TWR 体分支的**既有拒绝**（非本片），stub 后整类可编 |
| W17c（九路径：正常/直落子句/注入 E/丢弃+return/双丢弃/链式/close 抛/suppression/分支） | 全部路径 | 逐字一致（catch 体保留，TWR 显式内联 close——参照列既有形态） | 前 8 路径逐字一致；`branchBody`（本片负例）stub 截断 |
| N17c（三负例） | done×3 干净运行 | — | 恢复拒绝（运行干净，被拒的是恢复） |

JADX 列对 TWR 仍显式内联 close/suppression（与 direct 片记录同类，无新增偏差）；具名 catch 及
其丢弃调用体两列均保留。P2/W17c 的 Jarde 腿 stub 惯例同 direct 片：未恢复成员以抛
`UnsupportedOperationException` 的 stub 替换，运行截断于首个 stub。

## 4. 已知边界（如实记录）

- 巡查冻结 P2/P3 本体的注入 E 路径不可触发（其体 `touch/append` 无抛出点；单字段字节补丁无法
  制造抛出而保持 verifier 有效）——注入路径由同 handler 形态（`astore; getstatic; ldc; append;
  pop`）的 `W17c.bodyThrows/callThenReturn/closeThrows/suppressedBoth` 真实承载，`java
  -Xverify:all` 双腿逐行一致。
- 子句体仍限单直行块语句子集 + 丢弃调用（本片验收域）；分支/循环体、结果被消费/多用途丢弃形
  按守卫保持拒绝（N17c 三负例 + W17c.branchBody 钉死）。17b 已知边界（子句参数传入形参收窄的
  静态方法仍拒）不受本片影响。

## 5. 测试与门禁（tasks 3.1）

- `tests/p3_twr_enclosing_catch_call.rs`：巡查锚点命中（P2/P3 整构造文本）、BCI 38 handler 入口
  锚、九形态变体族命中 + return 形对照逐字、N17c 三负例 + branchBody 拒绝（quality=fallback +
  `jre_guard_unexplained_row` + 无 catch 呈现）、N17c 本体运行、三腿编译运行（P3 整类 /
  P2 Driver2 / W17c stub 截断）、预算/取消零发布。
- 门禁（实现提交上实跑）：`cargo test --workspace --tests --locked --no-fail-fast` 全绿
  **2825 通过 / 0 失败**（282 套件，主线 2818 + 本片 7；本轮无 flake 复跑需求）；
  `cargo fmt --all -- --check` 干净；clippy（CI 完整 30 项 `-A` 清单 + `-D warnings`）
  零告警；`openspec validate --all --strict` **244 项全过**。磁盘：全程 ≥71Gi 余量，
  报告前 `cargo clean`。
