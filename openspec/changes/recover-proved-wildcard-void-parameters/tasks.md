## 1. 同轮空正文与参数证明

- [ ] 1.1 复用 `GenericReturnCandidate` 的空 `void` 形状，完整记录唯一物理参数槽和同轮名称；核对真实 Code、AST、SSA、效果和无 handler，参数无读写
- [ ] 1.2 对参数读取/写入、隐藏调用、handler、额外指令和不完整 Code 的真实负例验证候选拒绝

## 2. 类源码声明投影

- [ ] 2.1 在 reader Signature 擦除及现有 `ordinary_parameterized_declaration` 门下准入一个 `List` 通配符参数的静态空 `void` 方法，复用既有通配符/数组拼写；raw `List` 不推测实参
- [ ] 2.2 对错误擦除、复杂界、额外参数、同名调用、type-use 注解及 budget/cancel 核对完整拒绝、物理来源和单方法报告

## 3. 三方验收

- [ ] 3.1 运行 `dt17-wildcard-void/replay.py fixed`，使原/JADX/Jarde 完整源码以 Java 8 重编，`-Xverify:all` 的五个通配符与一个 raw 控制的反射类型逐字一致
- [ ] 3.2 运行相关 jarde-java、reader、class-source 测试、Rust fmt/check 与 OpenSpec strict；清理临时 Cargo target，并记录未覆盖的 DT-17 组合
