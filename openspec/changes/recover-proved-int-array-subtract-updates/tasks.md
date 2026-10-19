## 1. Evidence and narrow implementation

- [x] 1.1 root核原始continue完整基线、真实公开失败/既有Add-only证明及全部Array/returned调用者，明确这是Builder窄依赖而非循环新机制；不把未运行脚本当成功。
- [x] 1.2 Luna私有实现现有Array计划op和非返回值isub窄准入，附debug/no-debug真实IR正例、等长未批准operator负例与预算取消；root逐差异审架构/消费/求值顺序/旧返回值边界并实际定向验证。

## 2. Shared complete acceptance

- [x] 2.1 root以两片共同最终CLI完成父片完整八API/JDK8和23整类编译/256观察/错根/身份/maps对照，原件/JADX精确复用，所有失败和独立审查逐字归档。
- [ ] 2.2 两片共享一次fresh workspace及当前产品自身CI全部步骤/双seed/继承门，绑定Git/CLI/源码，分别同步main spec/strict/账本/handoff，提交推送clean-main后关闭；保留5GiB/target1GiB/defaultstack/guard/finally clean。
