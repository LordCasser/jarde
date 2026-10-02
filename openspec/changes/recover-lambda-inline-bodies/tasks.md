## 1. 取证与基线

- [ ] 1.1 重放固定 Y1（SHA 核对）：定位 lambda 呈现位点与伴生方法装配关系（伴生进入类文本的路径、MethodHandle 引用位点）；记录冲突编译基线。
- [ ] 1.2 构造并冻结至少五个 verifier 有效变体/负例：双参比较器、捕获局部、多 lambda 同方法、复杂体（分支）伴生、伴生多用途（手工字节码负例）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 内联与保守分支

- [ ] 2.1 直线体内联 + 伴生隐藏（design 决策 1）；Y1 整类重编通过、运行与 orig.out 逐字一致。
- [ ] 2.2 复杂体重命名保守（决策 2）可编；负例保持现呈现；无 lambda 类 diff 逐字不变；预算/取消原子性不变。

## 3. 回归与验收

- [ ] 3.1 全仓测试全绿（含 functional-receiver/lambda 相关全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [ ] 3.2 Y1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核内联判据、参数绑定与三方行为，更新 EM 账本（lambda 域）与巡查记录。
