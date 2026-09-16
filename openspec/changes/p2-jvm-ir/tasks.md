## 1. Resolver and demand closure

- [ ] 1.1 定义 PlatformUniverse、LoadDomain、provider 和 Resolved/Missing/Ambiguous 等状态，并用多 loader fixture 验证结果可追溯
- [ ] 1.2 实现按需 Header/body 闭包和类/方法/字节预算，验证 A14/A16 不读取无关 Body
- [ ] 1.3 实现 resolution 与 dispatch 分离、Base/Sub 候选扩展、default/interface/conflict 诊断，并以继承和缺失依赖 fixture 验证 A11 与 open-world 状态

## 2. JVM IR pipeline

- [ ] 2.1 定义 PassDescriptor、phase、required/produced facts 和 invalidation registry，并用错误顺序测试验证拒绝半初始化 IR
- [ ] 2.2 实现 raw CFG、throw sites、handler order、effect sequence 和 legacy returnAddress facts，覆盖 A09
- [ ] 2.3 实现有界 jsr/ret normalization、origin mapping 和膨胀 fallback，验证历史 finally 与非法高版本指令行为
- [ ] 2.4 实现 CanonicalCFG、Frame、stack/local SSA、category-2/uninitialized/array 合流和 invariant checks，覆盖 A10

## 3. Conservative output and acceptance

- [ ] 3.1 实现独立的 representation（Java/Bytecode/Mixed）、quality（Structured/Conservative/Fallback）、compile_status、semantic_validation、verification、coverage/diagnostics 和成员级失败隔离，覆盖 A13
- [ ] 3.2 保证 X1 查询路径不创建 CFG/SSA/Java AST，建立计数断言覆盖 A17
- [ ] 3.3 建立 P2 历史字节码、异常重叠、缺失 debug、缺失依赖和不可约 CFG 语料，验证 A09/A10/A13/A14/A16
- [ ] 3.4 更新支持矩阵并执行 cargo fmt、clippy、test 与 OpenSpec strict validation，确认未宣称 Java 源码恢复
