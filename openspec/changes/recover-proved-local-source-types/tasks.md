## 1. Frozen actual evidence

- [x] 1.1 root独立核CF12真实Java-input基线六JUnit/八class/16render/18生成源码腿及完整原/JADX/Jarde运行raw，核CLI/source/SDK/tool pins与失败分类，保存接受观察记录而不接受整CF12。
- [x] 1.2 root同次真实IR诊断确认两个局部的SlotUse/SSA stored producer、全部写与reuse身份；读完类型决策/声明/调用/selector周边，Luna私有补丁通过架构与对抗审查后才能应用，实际shape和失败raw保留。

## 2. Existing local type decision

- [ ] 2.1 在既有map决策中实现有限char producer种子与全写兼容证明，永久真实class测试核charAt→store→char append/switch和完整物理来源；一般int、范围外literal、混合写、未知合流和槽复用反例保持准确类型或拒绝，实际测试通过。
- [ ] 2.2 同一类型决策恢复null初始化且所有非null写同一准确Reference的局部；真实无default switch完整恢复并保留null-miss，混合/未知引用、全部null与未解copy/phi负例实际保持边界，不改调用转换门。
- [ ] 2.3 root实跑新增证明内预算Stop与取消，核实际Stop位置/维度、无部分正文/map；现有参数、boolean、guard、调用重载和声明/复用回归通过，raw保留。

## 3. Complete source and delivery

- [ ] 3.1 root在5GiB/target1GiB一秒守卫下完成fmt/CI同范围Clippy/相关测试，冻结新CLI/meta/source/test/class pins；真实CF12原/JADX/Jarde完整类default/all原样重编运行，两锚全部物理BCI覆盖、配置正文/map相同，其他fixture与基线正文/map及失败分类保持，保留check()/Inner/真实SDK。
- [ ] 3.2 OpenSpec strict实际通过，产品提交推送并独立接受精确自身CI的全部job/step/双seed/测试名/source pins；不得借前片或文档CI，失败raw不覆盖。
- [ ] 3.3 更新verification-root、71账本与handoff；提交推送全部授权修改并实核main/origin相同、全worktrees clean/无待合入或分支占用，cargo清理后保留新CLI与source/class/raw，不宣称整个CF12或长期目标完成。
