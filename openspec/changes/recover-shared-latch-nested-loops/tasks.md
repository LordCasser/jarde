## 1. 取证与基线

- [ ] 1.1 重放固定 S5/S3/S4/Svc（SHA 核对）：读 `region.rs` 循环形状证明的 latch 归属判定与 dj 片边语义归类的复用面；记录 S5 两形与 Svc.lookup 基线（`jre_region_loop_shape` header@4 + 未覆盖块）。
- [ ] 1.2 冻结至少四个变体/负例：三层共享 latch（登记现状）、内层退出目标≠外层 latch（拒绝）、外层 continue + 内层 break + 内层 continue 交叠形、带标签 `continue outer`（正例）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. latch 共享判据与归属

- [ ] 2.1 判据三条准入 + 归属按边语义分派（design 决策 1–2）；S5 两形恢复、整类重编运行与 o5.out 一致（`13`/`9`/`20`/`13`）；S3/S4 Map.Entry 双层与 Svc.lookup 体恢复（Svc 行为 `[px:3]`）。
- [ ] 2.2 负例保持拒绝；单层 continue/无跳转双层/既有循环与 dj 全部测试 diff 逐字不变；预算/取消原子性不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 loop-body-double-jumps、labeled-loop-tail-coverage、switch-loop-join、循环系全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.2 S5/S3/S4/Svc 与变体三方对照：原 class/固定 JADX（dev）/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核准入判据、归属分派与三方行为，更新 CF-09/CF-11 账本与巡查记录。
