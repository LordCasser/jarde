## 1. Evidence and architecture

- [x] 1.1 根独立审查两版 SDK、javac8/23 debug/none 原始控制与完整 Jarde/JADX 重编译收据，确认 Integer/primitive、重载及真实 CHECKCAST 边界；以原始流、Code facts 和运行 oracle 验证，分类所有旧失败。
- [x] 1.2 根审查 List body 与 opaque AST 事务的最小复用设计，并以 OpenSpec strict 和实现派发记录确认范围、预算及可选缺证保留核心契约。

## 2. Implementation and regression

- [x] 2.1 Luna 私有补丁复用现有元素局部证明，根审查并集成精确 equals/remove 可选包装清理及 staging/final 重验证；固定 String/Integer 公开回归通过且 remove 不影响 E 推断。
- [x] 2.2 覆盖可选断链保留泛型/typed、未知/重载目标、真实参数 CHECKCAST、重复/错误来源、default/all/raw/map 和 Stop；根运行相关 Rust 回归、fmt、CI Clippy 与适用集成门验证。

## 3. Comparison and delivery

- [x] 3.1 根实际比较原始完整 TestNestedLoops 与 Jarde/JADX 完整生成源码，javac8/23 重编译及双 VM oracle、CodeSource 错根控制、原始/投影事实差分均接受；对独立控制只统计成功完整类的真实运行，保留编译失败。
- [ ] 3.2 根完成所需 workspace 回归并提交推送产品，验收实际产品自身 CI 全部 jobs/steps，绑定最终 CLI 和来源；同步主规范、账本及 handoff，strict 通过、清理 target、main 干净且与远端一致。
