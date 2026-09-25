## 1. 测试调用对齐

- [x] 1.1 为所有旧式调用补齐符合测试意图的 `prove_generic_return` 参数，并检查没有遗留两参数调用：全仓 `rg -n 'recover_for_class_source\\s*\\(' --glob '*.rs'`。
- [x] 1.2 使用私有 Cargo target 运行受影响测试、格式检查及 strict 检查并记录结果：详见 `verification.md`；测试目标编译成功，但当前 workspace 有 3 个不涉及本次调用补参的断言失败。
