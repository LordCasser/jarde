# Tasks

## 1. 范围事实与公开契约

- [ ] 1.1 在现有 raw block→Draft→fusion→assembly 路径填充真实成员范围 ordinal 并明确 accessor 文档；加入 canonical 单元回归：非 throwing/throwing 区别、半开端点、重叠/catch-all/同 handler 多 row、部分融合、前向 goto 跨 protected gap、shared/nested/handler/dead legacy clones。实际通过对应单测，验证 block 非空真值、声明顺序、origin/path、不变 handler-row/throw-site 与 unreachable。
- [ ] 1.2 补强公开 `p3_method_ir` 对物理成员范围的精确非空断言，修正“每个 block ordinal 必有 handler row”的过强断言；落实新增扫描/存储/副本增长前计费与可移动载荷，加入精确限额、少一项、处理中取消及无 handler 无费用增量控制。实际测试无 partial canonical 完整发布、后继 Frame/SSA 未执行，保留现有 legacy clone 预算/取消真值。

## 2. 集成与交付验证

- [ ] 2.1 串行实际运行 `cargo test -p jarde-jvm --lib --locked canonical::tests` 与 `cargo test --test p3_method_ir --locked`，复跑当前 legacy clone/frame/SSA 相关门；记录实际命令、源码摘要、日志、失败与修复，经独立 review 核 handler/effect/source 恢复语义未扩大后更新任务。
- [ ] 2.2 通过当前 fmt/clippy、OpenSpec strict、workspace 与实现提交自身 CI；独立提交本变更，交付 verification 文档绑定最终源码/版本与真实测试，保持私有候选和已验收状态可辨。

## Workflow follow-up

- 验收后按项目流程同步主规范并归档，核验归档事实和最终实现一致。
