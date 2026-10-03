## 1. 取证与基线

- [ ] 1.1 重放 sif 配对（WCallI/WCallC，SHA 核对）：定位匹配器 CP 种类枚举位与 agent 最小补丁判据复核；记录 4 个新折叠 corpus 类形态。
- [ ] 1.2 冻结至少两个变体/负例：多接口方法调用族、覆盖段不含 CP 索引负例（保持拒绝）；`java -Xverify:all` 前后记录。

## 2. 判据位扩展

- [ ] 2.1 InterfaceMethodRef owner 入匹配（design 决策 1）；WCallI 折叠、`F1` 产物重编行为一致；既有家族 diff 逐字不变。
- [ ] 2.2 负例保持拒绝；预算/取消不变。

## 3. 回归与验收

- [ ] 3.1 全仓测试全绿、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [ ] 3.2 WCallI 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核判据位、4 类 corpus 归类与三方行为，更新账本与巡查记录。
