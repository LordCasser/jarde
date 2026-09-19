当前未开始。P2 已完成至 5.2，但须先关闭 4.2b/4.3b 和 5.3/5.4 的整体出口。

## 1. Recovery contracts and first usable method

P2 整体出口仍是前置。顺序为 1.1 → 1.2 → 1.3（先带上 3.1/3.2 的最小命名/source map）→ 2.x → 3.x；基础输出不等待全部模式完成。沿用 `layer-jarde-crates` 的单向分层，首个实际恢复闭环才创建 `jarde-java`；不新增跨项目通用中端 crate，droidsaw 参考边界见 design。

- [ ] 1.1 定义 RecoveryProfile、模式前置条件、rule version、representation、quality、compile_status、semantic_validation、verification 和 fallback 类型；明确由 `jarde-jvm` 拥有、恢复侧消费的只读 method/CFG/value/effect/origin 输入，不公开全部可变中端或反向依赖门面；明确请求内 IR 载荷的所有权/生命周期与阶段有效性（当前 report 只是摘要，driver 会释放私有表），不重复运行 P2 或重建语义；用真实 P2 结果消费与未满足前置条件 fixture 验证边界和不误识别
  - 已落地（2026-09-19）：只读 IR 交接本身——`jarde-jvm` 自有的 `MethodIr`（按值持有 canonical/frames/ssa 三表，`canonical()/frames()/ssa()` 只给借用、通过访问器读）、共享同一次运行的 `engine::analyze_method_ir`（同时交出 report）、`method_ir` 的再导出面与源码守卫；决定与验证记录见 design 的“1.1 的只读 IR 交接”。仍未完成：本 bullet 的 profile/前置条件/rule version/表示与验证状态/fallback 类型，以及未满足前置条件 fixture 的边界验证。
  - 已落地（2026-09-19，续）：上一条里的**产物词汇**（表示与验证状态类型）——按 P3 规格句实际写下的取值把平面扩到可表达：`Representation` 加 `Java`/`Mixed`、`Quality` 加 `Structured`、`SyntaxStatus` 加 `Checked`/`Unchecked`、`CompileStatus` 加 `Compiles`/`Failed`、`verification`（类型是 `jarde-reader` 的 `VerificationStatus`，故加在那里）加 `Performed`/`Failed`；`SemanticValidation` 三值已够用（`FixtureDifferential` 即 P2 为受控对照预留的值）。每条新增变体的文档写明它属哪个阶段、P2 为何不产出它；取值判定逐句取自 `recovery-validation` 的平面句与架构 §13.1 输出表（`Structured` 是 **quality**，不是 representation；`Java` 是 representation，不是 syntax_status）。P2 的产出、既有变体名、serde 形状与既有断言零改动。证据：`tests/p3_product_vocabulary.rs`（命名/往返 + 六平面穷尽性对照）与 design 的两组证伪。
  - 1.3 交付（不在本片）：`RecoveryProfile`、模式前置条件、rule version、失败 fallback。依据：本 change 决策 1「每个**模式 pass** 声明所需 IR/effect/metadata、输出节点、rule version 和失败 fallback」，而 `layer-jarde-crates` design 写明 `jarde-java`「随第一个真实 Region→AST→文本闭环」在 1.3 创建——声明随模式 pass 走。因此本片不建空壳、不预置跨层抽象。
  - 1.1 剩余未满足项（如实记录）：「用真实 P2 结果消费」由 1.1a 的 `tests/p3_method_ir.rs` 覆盖（真实 fixture、真实入口、具体数字）；「未满足前置条件 fixture 的边界与不误识别」依赖前置条件类型本身，故随 1.3 的模式 pass 一并验证，本片未做这一半。
- [ ] 1.2 评估可用 Rust Java AST/formatter/文档组合库并选择通过 origin、转义和输出预算准入的最小实现；在内部划清正常流图视图、异常事实、Region 与 Java 输出的职责，沿用 petgraph，避免新增通用 backend trait；验证各输出质量/表示可独立表达
- [ ] 1.3 随实际实现创建 `jarde-java`，打通真实单方法入口的 P2 SSA → 普通 if/loop/switch 可证明子集 → Java AST → 文本/source map/诊断闭环，先实现 3.1/3.2 的最小稳定命名和 BCI 映射；模式未命中保留通用结构，不可约/交叉异常区域明确 bytecode fallback。以独立小图/已知 fixtures 核对分支极性、循环 header effect 次数、return 和异常优先级，并验证库/CLI 一致、jvm 不反向依赖 java、预算/取消不会变成空 body 或成功状态，覆盖 A09/A10/A13/A16

## 2. Java 8 patterns

- [ ] 2.1 实现已验证的 LambdaMetafactory/method reference 和 capture 恢复，保留 bootstrap/use-site evidence 并覆盖 A04
- [ ] 2.2 实现 StringBuilder/StringBuffer concat、bridge 和 synthetic accessor 派生呈现，验证 A12 的双 origin 边
- [ ] 2.3 实现 inner/local/anonymous、enum/enum switch、default/static interface 和构造器/字段初始化模式
- [ ] 2.4 实现 try-with-resources、synchronized/finally 的异常/effect 保真恢复，覆盖 A09

## 3. Names, maps and release gate

- [ ] 3.1 在 1.3 已接通的命名实现上补齐 slot/作用域、关键字/冲突别名和无 debug 覆盖，验证语法糖加入后仍保持稳定命名，完成 A10 验收
- [ ] 3.2 在 1.3 已接通的 source map 上补齐 CP/attribute、跨方法派生与 Mixed 区域映射及 representation/quality/coverage 诊断，确保完整扫描下的 Fallback 不被标记为 Partial，完成 A12/A13 验收
- [ ] 3.3 建立多代 javac/ECJ、缺失 debug、混淆、缺失依赖和非 Java compiler 语料，执行受控重编译/行为对照
- [ ] 3.4 发布五维 Java 8 支持矩阵并验证单方法闭包 A16，执行 cargo fmt、clippy、test 与 OpenSpec strict validation
