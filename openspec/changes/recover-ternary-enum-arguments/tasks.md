## 1. 条件实参证书

- [ ] 1.1 检查 `prove_initializer_prefix` 现有 int argument parser 与 method condition source facts；定义并测试完整 int diamond 的 branch/arm/join/constructor BCI 绑定，不读取无关 method body
- [ ] 1.2 为多前驱 join、重复 consumer、额外调用/写入、错 arm 顺序、错误 constructor 和分支目标控制添加 verifier-valid 拒绝样例；每例断言整组不 projection 且原 initializer origin 保留

## 2. 投影与语义验收

- [ ] 2.1 通过 `TernaryInit` 输出 int conditional constructor arguments；Java 8 编译并以 `-Xverify:all` 执行，交替 predicate 样例覆盖两 arm 且 invocation count/order 与原 class 相同
- [ ] 2.2 对照 `LiteralInit`、普通 enum 和 `StringTernaryInit` 的边界：literal 行为不变，String 形态仍由 DT-11 独立决定且不借 int ternary proposal 越门
- [ ] 2.3 重放 original/JADX/Jarde 完整源码；运行相关 enum source tests、budget/cancel 测试和 OpenSpec strict validate，确认所有物理字段/BCI 来源仍可查
