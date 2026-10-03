## 1. 基线与负例

- [ ] 1.1 重放固定 Y1（SHA 核对）：读单静态行选择点与改道面；corpus 双案等价性预扫；记录 Y1 基线。
- [ ] 1.2 构造并冻结至少三个变体/负例：单静态抽象方法类、非静态单子（不折叠）、M1/M2/FV 逐字不变断言集；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 通道统一

- [ ] 2.1 单静态行改道（design 决策 1，或准入案）；Y1 家族 jar 折叠呈现、重编运行一致；全部既有家族 diff 逐字不变。
- [ ] 2.2 负例边界正确；预算/取消不变。

## 3. 回归与验收

- [ ] 3.1 全仓测试全绿、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [ ] 3.2 Y1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核通道选择、corpus 等价性与三方行为，更新账本与巡查记录。
