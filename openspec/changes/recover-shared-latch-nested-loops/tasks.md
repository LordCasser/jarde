## 1. 取证与基线

> **基线出处与重验义务（root 2026-10-04 补记）**：本 spec 的固定转录与期望输出（`13`/`9`/`20`/`13`、`[px:3]`）是在主线 **`30e54613`** 上测得的（见 [shared-latch-patrol](../../evidence/java-syntax-2026-10-04/shared-latch-patrol/README.md)），此后主线已合入 9 个切片。root 已用 git 核实两点降低风险：`crates/jarde-java/src/region.rs` 自 `30e54613` 起**零改动**，且 `latches.len() != 1` 判据未被触碰（`git log -S` 无命中），故**落点与循环形状判据不变**。但环 1 改过 `emit.rs`（匿名类声明位重拼）、环 3 改过 `facade.rs`，理论上不触及循环体发射——**开工时仍须实测确认**：先跑 S5/S3/S4/Svc 的**原 class** 与**当前主线渲染**，核对上述期望输出是否仍成立；若某个数字变了，**以实测为准并记录差异**，不要为对上 spec 里的旧数字而调实现。

- [x] 1.1 重放固定 S5/S3/S4/Svc（SHA 核对）：读 `region.rs` 循环形状证明的 latch 归属判定与 dj 片边语义归类的复用面；记录 S5 两形与 Svc.lookup 基线（`jre_region_loop_shape` header@4 + 未覆盖块）。**落点已由 root 核实**：`fn latch_tested_loop`（region.rs:10470），其 `if latches.len() != 1 { return Ok(None) }`（10483）即"latch 归属单一所有者"的现约束；相关件 `latch_test_chain`(10133)、`first_latch_test_suffix`(10389)、`latch_test_suffix_is_effect_free`(10424)、另一 LoopShape 产生点(8244)。行号在 region.rs 未变的前提下准确，但仍以锚点名为准。（**实现者 2026-10-04**：六锚点核实属实；但插桩门控实验证伪"该判据即拒绝点"前提——`latches.len()!=1` 在 S5 流程从未被行使，实际拒绝链为分支 join 选举吞并嵌套循环头，见 `../../evidence/java-syntax-2026-10-04/recover-shared-latch-nested-loops/refusal-chain/` 与 baseline-reverification/；spec 期望数字无差异。）
- [x] 1.2 冻结至少四个变体/负例：三层共享 latch（登记现状）、内层退出目标≠外层 latch（拒绝）、外层 continue + 内层 break + 内层 continue 交叠形、带标签 `continue outer`（正例）；各自 `java -Xverify:all` 通过并记录实现前后行为。（**实现者 2026-10-04**：`tests/fixtures/shared-latch-nested-loops/` 五类 + S5/S3 锚复制；三层形实现后组合恢复且行为一致、ExitDiverges/LabeledBreakOuter/Overlap 保持响亮拒绝、labeledOuter 恢复；对照与判定见 evidence negatives/README。）

## 2. latch 共享判据与归属

- [x] 2.1 判据三条准入 + 归属按边语义分派（design 决策 1–2）；S5 两形恢复、整类重编运行与 o5.out 一致（`13`/`9`/`20`/`13`）；S3/S4 Map.Entry 双层与 Svc.lookup 体恢复（Svc 行为 `[px:3]`）。（**实现者 2026-10-04**：判据落地为 `region.rs` 双臂路径的 `shared_latch_join` 准入 + `shared_latch` 归属分派，复用 dj 片 `loop_continue_bridge` 与 `continuing_join` 重选举同位；S5/S3/S4 三方行为一致；Svc lookup 体完整呈现、`[px:3]` 行为腿由同形 S3.nestedBreak 承担——Svc 整类重编被成员折叠通道池形 `Svc$Entry` 拼写阻塞（Non-Goal 域），已登记。）
- [x] 2.2 负例保持拒绝；单层 continue/无跳转双层/既有循环与 dj 全部测试 diff 逐字不变；预算/取消原子性不变。（**实现者 2026-10-04**：`outerContinueOnly`/`innerOnly`/`main` 渲染逐字节不变并有 CI 钉死；dj 全域与既有循环系测试全绿；全量 2970/0；预算/取消测试未触碰、全绿。）

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 loop-body-double-jumps、labeled-loop-tail-coverage、switch-loop-join、循环系全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。（**实现者 2026-10-04**：2970 passed / 0 failed（2962 基线 + 8 新测试，终轮单次全绿）；首轮 4 红均判定：2 已知 flake 复跑两轮转绿、1 fingerprint 待再生（已再生）、1 classfile 普查计数按其既有惯例更新；fmt 干净；clippy 按 ci.yml 46–76 生成（`--all-features` + 29 `-A` + `-D warnings`）exit 0；openspec 274/274；diff check 干净；最低磁盘 23Gi。）
- [x] 3.2 S5/S3/S4/Svc 与变体三方对照：原 class/固定 JADX（dev）/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。（**实现者 2026-10-04**：S5/S3/S4 三方输出 SHA 两两一致（`4685b821…`/`472b7fe2…`/`7ef56628…`，JADX leg 以 testzone dev 构建 `defpackage` 渲染重编）；变体 ThreeLevel 原/重编一致 `68ca3fba…`，三负例为拒绝形无重编腿；Svc 见 2.1 注——体呈现 + 同形 S3 行为腿，整类重编阻塞已登记。产物在 `../../evidence/java-syntax-2026-10-04/recover-shared-latch-nested-loops/three-way/`。）
- [ ] 3.3 root 独立复核准入判据、归属分派与三方行为，更新 CF-09/CF-11 账本与巡查记录。
