## 1. 取证

- [ ] 1.1 冻结类型片验收态与direct-new完整family的原/JADX/Jarde输入、拒绝和双流，复核真实new/dup/init/aastore与唯一instance consumer；验证构造组合失败不被计作非法类型关系。
- [ ] 1.2 审查现有arrays→sites编排和所有constructor内array/varargs回归，记录闭合证明及费用；验证不新增扫描pass或层级服务。

## 2. 组合实现

- [ ] 2.1 明确已有child-array只读事实接口与Site单次移交点；在既有candidate内复用构造验证，准确匹配paired store BCI及stored ValueId，完整结构成功后共同提交ownership，后续allocation census保持一个verified/new记录；focused测试证明每个构造效果一次且来源完整，无全计划clone或额外扫描。
- [ ] 2.2 完成extra-reader、错误instance、后半元素结构/类型失败、旧数组、handler/block和乱序控制；分别验证零结构半提交与呈现失败的完整正文拒绝，保留全部生产者且无悬空Java或重复站点；保留五wrapper primitive-conversion独立控制。
- [ ] 2.3 双javac完整family覆盖平台接口/异常/集合和自有子类、可观察constructor参数效果；root冻结candidate重放全部源集，隔离编译和-Xverify:all双流匹配，不借原class或删成员。

## 3. Root验收

- [ ] 3.1 验证existing inline-array constructor arguments、nested/char[]/varargs和new/array模块零回退；预算/取消不发布半记录，实际费用进入P5账本。
- [ ] 3.2 root对抗审查并完成双seed workspace、MSRV、fmt、CI-exact clippy、ignored gates、strict specs、fingerprint及diff检查；所有真实exit/streams永久保留。
- [ ] 3.3 根据完整结果更新EM-18待扩验边界与handoff，提交推送并检查确切SHA实际CI；原main外围失败不得谎称整个单元已追平。
