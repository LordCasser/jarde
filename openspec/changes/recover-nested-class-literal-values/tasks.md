## 1. 取证与基线

- [ ] 1.1 重放固定 A10/A11/A12（SHA 核对）：定位类字面量准入判据与 nested-spelling 可拼性事实的复用接口；记录 A12 两形与 A11 三形基线（引注文本）。
- [ ] 1.2 冻结至少四个变体/负例：多段嵌套 `A$B$C` 字面量、折叠域内字面量（jar 口径）、本地类字面量（拒绝）、匿名类字面量（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 准入扩展

- [ ] 2.1 判据复用可拼性事实准入嵌套名（design 决策 1–2）；A12/A11/A9.main 恢复、整类重编运行与基线逐字一致；A10 五形 diff 逐字不变。
- [ ] 2.2 负例保持拒绝；呈现两口径（折叠/分离）按 nested-spelling 既有规则锚定；预算/取消原子性不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 nested-spelling、类字面量、反射相关全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.2 A10/A11/A12 与变体三方对照：原 class/固定 JADX（dev）/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核准入判据、负例边界与三方行为，更新 EM 账本（反射/类字面量域）与巡查记录。
