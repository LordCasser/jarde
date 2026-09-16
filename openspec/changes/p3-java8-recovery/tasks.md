## 1. Recovery pass contracts

- [ ] 1.1 定义 RecoveryProfile、模式前置条件、rule version、representation、quality、compile_status、semantic_validation、verification 和 fallback 类型，并用未满足前置条件 fixture 验证不误识别
- [ ] 1.2 评估并选用可用、活跃且质量符合要求的 Rust 语法树、格式化和文档组合库，实现 origin/effect 接口，验证 Structured 与 Bytecode/Mixed 可以并存
- [ ] 1.3 实现 generic JVM fallback 和不可约/交叉异常区域处理，覆盖 A13

## 2. Java 8 patterns

- [ ] 2.1 实现已验证的 LambdaMetafactory/method reference 和 capture 恢复，保留 bootstrap/use-site evidence 并覆盖 A04
- [ ] 2.2 实现 StringBuilder/StringBuffer concat、bridge 和 synthetic accessor 派生呈现，验证 A12 的双 origin 边
- [ ] 2.3 实现 inner/local/anonymous、enum/enum switch、default/static interface 和构造器/字段初始化模式
- [ ] 2.4 实现 try-with-resources、synchronized/finally 的异常/effect 保真恢复，覆盖 A09

## 3. Names, maps and release gate

- [ ] 3.1 实现 slot/作用域感知的确定性命名、关键字/冲突别名和无 debug fallback，覆盖 A10
- [ ] 3.2 实现 AST/source map 到 BCI/CP/attribute 的映射及 representation/quality/coverage 诊断，确保完整扫描下的 Fallback 不被标记为 Partial，覆盖 A13
- [ ] 3.3 建立多代 javac/ECJ、缺失 debug、混淆、缺失依赖和非 Java compiler 语料，执行受控重编译/行为对照
- [ ] 3.4 发布五维 Java 8 支持矩阵并验证单方法闭包 A16，执行 cargo fmt、clippy、test 与 OpenSpec strict validation
