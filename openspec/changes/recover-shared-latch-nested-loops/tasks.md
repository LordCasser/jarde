## 1. 取证与基线

> **基线出处与重验义务（root 2026-10-04 补记）**：本 spec 的固定转录与期望输出（`13`/`9`/`20`/`13`、`[px:3]`）是在主线 **`30e54613`** 上测得的（见 [shared-latch-patrol](../../evidence/java-syntax-2026-10-04/shared-latch-patrol/README.md)），此后主线已合入 9 个切片。root 已用 git 核实两点降低风险：`crates/jarde-java/src/region.rs` 自 `30e54613` 起**零改动**，且 `latches.len() != 1` 判据未被触碰（`git log -S` 无命中），故**落点与循环形状判据不变**。但环 1 改过 `emit.rs`（匿名类声明位重拼）、环 3 改过 `facade.rs`，理论上不触及循环体发射——**开工时仍须实测确认**：先跑 S5/S3/S4/Svc 的**原 class** 与**当前主线渲染**，核对上述期望输出是否仍成立；若某个数字变了，**以实测为准并记录差异**，不要为对上 spec 里的旧数字而调实现。

- [ ] 1.1 重放固定 S5/S3/S4/Svc（SHA 核对）：读 `region.rs` 循环形状证明的 latch 归属判定与 dj 片边语义归类的复用面；记录 S5 两形与 Svc.lookup 基线（`jre_region_loop_shape` header@4 + 未覆盖块）。**落点已由 root 核实**：`fn latch_tested_loop`（region.rs:10470），其 `if latches.len() != 1 { return Ok(None) }`（10483）即"latch 归属单一所有者"的现约束；相关件 `latch_test_chain`(10133)、`first_latch_test_suffix`(10389)、`latch_test_suffix_is_effect_free`(10424)、另一 LoopShape 产生点(8244)。行号在 region.rs 未变的前提下准确，但仍以锚点名为准。
- [ ] 1.2 冻结至少四个变体/负例：三层共享 latch（登记现状）、内层退出目标≠外层 latch（拒绝）、外层 continue + 内层 break + 内层 continue 交叠形、带标签 `continue outer`（正例）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. latch 共享判据与归属

- [ ] 2.1 判据三条准入 + 归属按边语义分派（design 决策 1–2）；S5 两形恢复、整类重编运行与 o5.out 一致（`13`/`9`/`20`/`13`）；S3/S4 Map.Entry 双层与 Svc.lookup 体恢复（Svc 行为 `[px:3]`）。
- [ ] 2.2 负例保持拒绝；单层 continue/无跳转双层/既有循环与 dj 全部测试 diff 逐字不变；预算/取消原子性不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 loop-body-double-jumps、labeled-loop-tail-coverage、switch-loop-join、循环系全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.2 S5/S3/S4/Svc 与变体三方对照：原 class/固定 JADX（dev）/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核准入判据、归属分派与三方行为，更新 CF-09/CF-11 账本与巡查记录。
