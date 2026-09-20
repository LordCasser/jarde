## 1. 名称选择的停止边界

- [x] 1.1 将 completion-review 的有效/损坏候选前后顺序反例纳入永久回归，并覆盖零/一候选时预算、受控取消和成员表截断；确认旧行为会失败，完整唯一、完整缺失、真实歧义和显式物理身份对照仍成立（A07/A14/A18）。
- [x] 1.2 在共享搜索传播成员表停止，为 OperationOutcome 和内部绑定增加 Incomplete，先检查完整性再检查候选数；验证 class_view/analyze_target/recover_target 均保留候选、coverage、诊断及实际用量，未完成选择的 method_bodies/IR 构造为零（A14/A16）。

## 2. 类视图与 CLI 一致性

- [x] 2.1 在库内汇总方法体停止，区分局部损坏与共享预算终止；以一个 BCI 0 非法 opcode 和一个正常方法验证顶层非 Complete、正常 body 保留、成员表结构覆盖真实，并覆盖拒付/取消后不再启动后续 body（A13/A14/A16）。
- [x] 2.2 迁移 CLI、示例与所有 OperationOutcome 匹配；验证未完成选择退出 4、完整歧义退出 3、真实输入错误退出 2，JSON 与库逐字段一致，text 保留同一停止事实；类视图不再只靠 CLI 补算未完整状态（A13/A14）。

## 3. 完成确认

- [x] 3.1 在修正后的固定提交执行 fmt、workspace 全量测试、clippy -D warnings、两条 P3 显式编译/执行对照及适用 CI；记录反例/正向对照与完整报告确定性（仅归一 elapsed_millis），更新 review、路线、支持矩阵和 benchmark 候选，strict 验证后才同步主规格并归档。性能专项样本未完成时仍保持待办，不将本修正当作性能交付。
