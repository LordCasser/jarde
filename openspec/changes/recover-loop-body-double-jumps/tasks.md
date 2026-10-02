## 1. 取证与基线

- [ ] 1.1 重放固定 L1–L5（SHA 核对）：javap 两最小复现的块/边图，读臂 join 判定与区域归属处理体内跳转边的位置，确定语义归类挂点；记录双诊断基线。
- [ ] 1.2 构造并冻结至少五个 verifier 有效变体/负例：双 continue 不同标签目标、双 break 不同目标、break+labeled continue 跨两层、三跳转（登记现状）、单跳转矩阵逐字不变断言集；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 边语义归类与归属分派

- [ ] 2.1 臂 join/循环形状证明按边语义归类（design 决策 1）；区域归属分派（决策 2）；L5 两最小复现与 L1/L2 复合完整恢复、整类重编运行与基线逐字一致。
- [ ] 2.2 单跳转全矩阵与 break+return 等既有通道 diff 断言逐字不变；负例边界正确；预算/取消原子性不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 labeled-loop-tail-coverage、switch-loop-join、enclosing-named-catch、loop 系全部既有测试）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [ ] 3.2 L1/L2/L5 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核边归类判据、归属分派与三方行为，更新 CF 账本与巡查记录。
