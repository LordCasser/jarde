# Tasks

## 1. 范围事实与公开契约

- [x] 1.1 在现有 raw block→Draft→fusion→assembly 路径填充真实成员范围 ordinal 并明确 accessor 文档；加入 canonical 单元回归：非 throwing/throwing 区别、半开端点、重叠/catch-all/同 handler 多 row、部分融合、前向 goto 跨 protected gap、shared/nested/handler/dead legacy clones。实际结果与计费校准见 [verification-root.md](verification-root.md)：canonical 单元与公开 P3、clone/frame/SSA 门均通过；v1 的 P5 旧 pin 失败被如实保留，实测后仅校准六个计费常量及说明。
- [x] 1.2 补强公开 `p3_method_ir` 对物理成员范围的精确非空断言，修正“每个 block ordinal 必有 handler row”的过强断言；落实新增扫描/存储/副本增长前计费与可移动载荷，加入精确限额、少一项、处理中取消及无 handler 无费用增量控制。实际测试覆盖新增预算、取消及无 handler 控制；结果和失败边界见 [verification-root.md](verification-root.md)。

## 2. 集成与交付验证

- [x] 2.1 串行实际运行 `cargo test -p jarde-jvm --lib --locked canonical::tests` 与 `cargo test --test p3_method_ir --locked`，复跑当前 legacy clone/frame/SSA 相关门；命令、源码摘要、v1 失败、计费修正与独立本地验收均记录于 [verification-root.md](verification-root.md)；不将该验收表述为 workspace 或在线 CI 通过。
- [x] 2.2 通过当前 fmt/clippy、OpenSpec strict、workspace 与实现提交自身 CI；独立提交本变更，交付 verification 文档绑定最终源码/版本与真实测试，保持私有候选和已验收状态可辨。

## Workflow follow-up

- 验收后按项目流程同步主规范并归档，核验归档事实和最终实现一致。

2026-10-15 最终联合验收：以上早期“待验”描述保留为历史。产品d3e2b2457338b61660b842f5705b225bba08a73a自身CI38673548915成功，workspace-v3两seed各3587/0/98，focused-v3为462/0/2；独立审计接受，4/4关闭。见verification-root.md新增联合验收节。
