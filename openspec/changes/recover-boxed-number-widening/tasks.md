## 1. 基线与负例

- [ ] 1.1 重放固定 C7/C8（SHA 核对）：反射核对直接边全集（design 取证义务）；记录 BCI 63 基线。
- [ ] 1.2 冻结至少三个变体/负例：Double/Long 装箱变体、String→CharSequence、Boolean→Number（拒绝）；各自 `java -Xverify:all` 前后记录。

## 2. 闭集扩展

- [ ] 2.1 java.lang 边入表+walk；C8 恢复、重编行为逐字一致（`x`/`1:2`/`7`/`eoeoeoe`）；既有闭集与用户类负例 diff 零回退。
- [ ] 2.2 预算/取消不变。

## 3. 回归与验收

- [ ] 3.1 全仓测试全绿（含 throwable/collection widening 全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [ ] 3.2 C8 与变体三方对照；记录输出 SHA。
- [ ] 3.3 root 复核闭集逐对与三方行为，更新 EM 账本与巡查记录。
