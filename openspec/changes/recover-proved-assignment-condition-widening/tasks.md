## 1. Baseline and architecture

- [x] 1.1 root接受完整固定上游原始/JADX/Jarde基线及独立六场景控制的原始双编译器/双VM、CodeSource负控制、全部物理方法/BCI/raw map；分列常零helper与动态控制、真实整类编译失败和旧JDK框架边界，保存实际来源及原始流。
- [x] 1.2 root完成本Spec/设计的对抗审查及strict校验，按本change实际派Luna私有实现，保存范围/派发收据；确认只复用既有widens和赋值消费者，不新增机制。

## 2. Implementation and regression

- [x] 2.1 root先实际接受完整控制生成源码的JADX/Jarde基线，再审查集成Luna最小类型门补丁；通过真实Builder消费者及公开完整上游/控制测试验证扩大转换、原exact路径、unknown/boolean/引用/窄化/错复制用途拒绝、一次求值和来源闭合；冻结真实fixture并独立核算reader库存和指纹。
- [x] 2.2 root运行相关局部复制/条件/循环/短路/类型转换和预算/取消/期限回归、fmt/CI Clippy/MSRV适用广门，全过且源不变，构建最终CLI并exactclean；不把已有通过的片段拼为fresh workspace。

## 3. Complete comparison and delivery

- [x] 3.1 root用最终CLI完成固定完整上游及六场景动态控制的源码/JADX/Jarde两编译器/双VM对照，验证完整类/真实方法库存、值65535、非零位置恢复、调用次数/事件和自然越界，default/all/essential及raw来源一致；编译失败不运行、不裁剪源码或借原target冒充生成类。
- [ ] 3.2 root完成fresh workspace、产品提交推送与其自身CI全部jobs/官方step/两seed及新增精确门，绑定源/CLI/Git；同步主规范、固定账本与handoff，strict/apply通过，target清空且main干净/远端一致后关闭本叶子，持续goal保持active。
