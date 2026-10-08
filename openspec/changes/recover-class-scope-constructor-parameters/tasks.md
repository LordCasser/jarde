## 1. 冻结与证明入口

- [x] 1.1 冻结至少六个正例及独立边界族的原源码/class/jar/javap、JADX、主线 Jarde；真实 JDK8/JDK23 × debug/no-debug，完整重编、-Xverify:all 行为和 GenericDeclaration 身份反射分开统计，记录全部失败而非空输出；root 核对三方结果。
- [x] 1.2 审查同次 Program/Code/SSA/InitRecord 候选和所有消费者，验证类作用域/参数槽/初始化 this/物理字段的最小事实闭合；确认方法先发布、字段后消费且无循环依赖，交付 root 架构审查记录。

## 2. 既有路径实施

- [x] 2.1 扩展构造器候选到完整 Object() 后原参数直接 this 字段写，分流并原子发布类作用域参数 Signature；以六族四腿、交叉 binder、物理字段身份、essential/all 和既有 generic constructor 测试验证，保持方法级字段写旧拒绝边界。
- [x] 2.2 加入参数重写/call/phi/EH/this delegate/非 Object 父类/未知字段/重复 BCI/预算取消边界与旧消费者保护；用针对性测试确认新增路径安全拒绝且不丢正文、误隐藏成员或把未发布 Signature 提供给字段。

## 3. Root 验收与主线交接

- [x] 3.1 root 独立重放全部四腿完整类编译、独立运行与反射，并复核 production diff 和源码 SHA；正例必须全部闭环，拒绝族单列，泛型字段写前片冻结族无新增编译/行为退化。
- [ ] 3.2 root 执行 fmt、CI 同口径 clippy、两固定 seed 工作区测试、显式 ignored P3/构造实参/绑定引用门禁与 strict OpenSpec；提交合入推送，更新 handoff/陈旧账本，核对最新 HEAD 的 CI，解除实现分支占用并清理共享 Cargo 残留。
