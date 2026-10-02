## 1. 取证与基线

- [x] 1.1 重放固定 L1–L5（SHA 核对）：javap 两最小复现的块/边图，读臂 join 判定与区域归属处理体内跳转边的位置，确定语义归类挂点；记录双诊断基线。（fixture SHA 与 `results/fixture-sha256.txt` 一致；边图取证与挂点见 [dj/README §1](../../evidence/java-syntax-2026-10-02/loop-double-jump-patrol/dj/README.md)，javap 落 `dj/javap/`；挂点＝`region_at_inner` 臂 join 的 `loop_bridge_join` 细化＋新增 `loop_continue_bridge`/`loop_break_transfer` 分类，及到达自家 header 的回边停止条款；基线诊断 `arms_do_not_meet`@31 三层级联与 `ownership_overlap`@2 复现）
- [x] 1.2 构造并冻结至少五个 verifier 有效变体/负例：双 continue 不同标签目标、双 break 不同目标、break+labeled continue 跨两层、三跳转（登记现状）、单跳转矩阵逐字不变断言集；各自 `java -Xverify:all` 通过并记录实现前后行为。（`dj/variants/` DJLoops/DJTripleJump，javac 23 `--release 8 -g:none`；**判别修正**：双 break 不同目标与 break+labeled continue 跨两层主线已由 break 边族既有通道恢复（基线 stash 双腿实测，`dj/results-dj/*.before`），真正缺失族为「同体任一组合含 continue 边」；前后输出与 SHA 见 `dj/results-dj/`；单跳转矩阵 L3/L4 与 DJLoops 五个无关方法逐字不变）

## 2. 边语义归类与归属分派

- [x] 2.1 臂 join/循环形状证明按边语义归类（design 决策 1）；区域归属分派（决策 2）；L5 两最小复现与 L1/L2 复合完整恢复、整类重编运行与基线逐字一致。（L1/L2 重编 `-Xverify:all` 与 orig.out/o2.out 逐行一致；L5 两最小复现+brkSelfContMid 路径一致，dblJumpDoWhilePlain（break+return 既有通道）保持变更前逐字相同的 BCI 26/28 引用；到达自家 header 的回边停止消除 `ownership_overlap`）
- [x] 2.2 单跳转全矩阵与 break+return 等既有通道 diff 断言逐字不变；负例边界正确；预算/取消原子性不变。（L3/L4 与冻结基线零 diff；DJLoops 五方法 before/after 逐字节相同；三跳转拒绝前后逐字相同；回归 `tests/p3_loop_body_double_jumps.rs` 七测试含预算/取消腿）

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 labeled-loop-tail-coverage、switch-loop-join、enclosing-named-catch、loop 系全部既有测试）、fmt、CI 完整 29 项 allowlist clippy（`.github/workflows/ci.yml` 实有清单）、`openspec validate --all --strict`（246 项）、diff check；磁盘纪律同前。（结果见 [dj/README §4](../../evidence/java-syntax-2026-10-02/loop-double-jump-patrol/dj/README.md)）
- [x] 3.2 L1/L2/L5 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。（五类 JADX 腿全部与原类一致；Jarde 腿 L1/L2/DJLoops 全一致、L5 除既有 dblJumpDoWhilePlain 引用路径外一致；SHA 见 `dj/results-dj/sha256.txt`）
- [ ] 3.3 root 独立复核边归类判据、归属分派与三方行为，更新 CF 账本与巡查记录。
