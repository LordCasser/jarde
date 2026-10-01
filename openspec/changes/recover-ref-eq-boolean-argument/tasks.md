## 1. 基线与负例

- [ ] 1.1 重放固定 S1（SHA 核对）：定位实参位类型判定点与既有布尔证明接入位；记录 BCI 148–156 基线。
- [ ] 1.2 构造并冻结至少三个 verifier 有效变体/负例：`!=` 拼写、数值相等入双 boolean 形参、比较结果存后传参（正变体）；非比较来源的 0/1 int → boolean 位（保持拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 实参位布尔呈现

- [ ] 2.1 实参位接入比较双臂布尔证明（design 决策 1）；S1 完整恢复、整类重编运行与基线逐字一致（含 `falsetrue` 尾段）；变体逐项恢复；负例保持拒绝。
- [ ] 2.2 既有布尔位（赋值/return/条件）测试与负例零回退。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 3.2 S1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核判据、呈现与三方行为，更新 EM-05 账本与巡查记录。
