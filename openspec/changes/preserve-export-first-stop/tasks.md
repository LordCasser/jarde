## 1. 修复与回归

- [x] 1.1 复用 report 的首次停止选择失败文档，保留真实 Delivery/OutputBytes 的数字及 Output I/O 优先级；确定性测试内部取消、外部取消、并发其他维度停止与真实输出预算。
- [x] 1.2 root 审查原因传播链，export_cli 全族与 ir_items 紧/宽预算连续回归通过；不放宽原始维度/无 final 断言。

## 2. 主线收尾

- [x] 2.1 fmt/clippy、两个固定 seed workspace、strict OpenSpec 与共同 ignored oracle 通过，记录 root 验收。
- [ ] 2.2 合入推送 main、核对 CI、释放实现分支并更新 handoff，清理 Rust 编译残留。
