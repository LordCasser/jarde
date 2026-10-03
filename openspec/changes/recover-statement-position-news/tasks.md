## 1. 基线与负例

- [ ] 1.1 重放固定 B5/B6（SHA 核对）：读语句位判定处与实参分类数据面；记录 B6 四形基线。
- [ ] 1.2 构造并冻结至少三个变体/负例：多语句混合、静态嵌套类语句 new、CST 冻结反例（逐字不变断言）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 语句位呈现

- [ ] 2.1 判据呈现（design 决策 1）；B5.main/B6 四形恢复、整类重编运行与基线逐字一致；CST 反例不变。
- [ ] 2.2 消费位构造与既有 new@1 通道 diff 零回退；预算/取消不变。

## 3. 回归与验收

- [ ] 3.1 全仓测试全绿（含 refuse-unconsumed-construction-invokes 全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [ ] 3.2 B5/B6 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核判据边界、CST 保护与三方行为，更新账本与巡查记录。
