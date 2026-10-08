## 1. 冻结与门控

- [x] 1.1 将generic-holder五族双腿证据纳入可复现CI fixture/test；补null、安全与不安全混写、同名不同binder、数组变量、raw参数化字段和延期method参数正例，保存主线整类编译/诊断/Signature结果。
- [x] 1.2 门控验证method候选先结清后的字段写位类型判据；证明来源为同轮SSA参数槽/null及实际发布Signature，记录unknown类型安全拒绝和预算停止，并让root审查决定最小实现位置。

## 2. 实现与类型边界

- [x] 2.1 现有使用清单补写入值事实和源码适配证明；所有写位均满足才原子投影，TypedSetter/null与原有合法参数化字段测试通过，Hold/ObjectSetter/ObjectHold/CrossSetter安全回退且完整类编译通过。
- [x] 2.2 反例测试覆盖T/U/不同binder、改写参数、不完整scan、未知RHS和预算/取消；验证字段身份/descriptor/BCI、停止传播和无未证明cast，各项失败均可靠拒绝。

## 3. 验收

- [x] 3.1 双javac腿全类重编译/-Xverify:all行为；成功投影的泛型reflection与原类/JADX一致，拒绝族检查保守marker与行为（不要求恢复Signature）；root独立验收。
- [x] 3.2 fmt、CI同口径clippy、两固定seed workspace、strict OpenSpec与附近泛型字段回归通过，提交推送并更新handoff；构造器泛型恢复保持独立未完成状态。
