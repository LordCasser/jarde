## 1. Baseline and stopping point

- [x] 1.1 独立验收原形及普通 while 对照基线；核原形31命令/112文件、控制31命令/111文件闭合，原2/JADX4成功、旧Jarde4整类编译失败且零runtime，并准确记录 noPrefix 缺 goto@14 来源；以 root 实跑独立 verifier 的不可变接受文件验证。
- [ ] 1.2 资源守卫满足后执行带 method 身份的临时诊断，核实际 arm 的 prefix、header、next、Frame boundary 与 join 停止点；保存原始 stdout/stderr，撤销 probe 后核生产 diff。若链与 design 假设不同，先修订 design，不据静态猜测实施。

## 2. Local one-arm continuation

- [ ] 2.1 在 1.2 定位后由 Luna 提交 private patch，root 读审并应用；复用统一 region_at/Frame/自然循环证明，完成独占 Straight prefix、一个普通 Loop 及可选 Straight tail 的准确 join 组合；用 prefixWhile/takenArm/loopAndTail 与 iterator 原形永久正例核完整结构、唯一 owner 和尾部位置，无新 pass/Frame 字段/公共 IR。
- [ ] 2.2 验证 prefix/header/tail 外部入口、不同出口、已认领块、外层 loop 回边与父 scope 不闭合反例继续拒绝；预算/取消/递归 Stop 传播且不发表部分正文或来源。root 实跑相关永久测试和相邻单臂/双臂/loop-tail 回归，以真实结果验证。

## 3. Whole-class acceptance and delivery

- [ ] 3.1 冻结撤销 probe 后的 fresh CLI 与源码 pins；双 JDK 对照原形及 PlainOneArmLoops 的 default/all 八份完整生成类，原样重编运行并逐字比原 class 的 exit/stdout/stderr，核全部物理成员/owner/BCI 与 default/all 正文和 source map 恒同；noPrefix latch 来源独立修复完成后才可接受完整 BCI，不豁免已知 goto@14。
- [ ] 3.2 root 实跑 fmt、CI 同范围 Clippy、相邻回归与 OpenSpec strict；提交推送后捕获本产品精确 headSha 的 CI/API/日志并独立验收，不借旧提交 CI 或文档提交结果完成此门禁。
- [ ] 3.3 更新 71 单元账本与根 handoff.md，记录局部接受范围、独立来源债务状态及精确证据路径，不冒增 EM23 整单元完成数；提交推送全部授权修改并仅清理本仓编译残留，核主线 clean、无待合入工作树和分支占用。
