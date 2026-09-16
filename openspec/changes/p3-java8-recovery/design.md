## Context

以 P2 完成为进入条件。本 change 规划消费 Demand Resolver、CanonicalCFG、Frame/SSA、effects 和 Conservative output；当前尚未实现。恢复是可选呈现层，必须保持 P1 X1 的原始引用与 P2 IR origin；Java 8 是第一套高质量承诺，不代表所有 JVM 字节码都能表达为 Java。

## Goals / Non-Goals

**Goals:**

- 在 Java 8 RuntimeProfile 中以 evidence-gated passes 恢复高频 javac/ECJ 模式和稳定变量命名。
- 评估可用的 Rust Java AST、formatter 和文档组合库，提供 source map，并独立记录 representation、quality 和 validation status；不预设一定存在符合纯 Rust 与语义要求的完整 Java AST 库。
- 以真实历史/Java 8 语料和受控重编译/行为样本建立支持矩阵。

**Non-Goals:**

- 不执行目标 bootstrap、静态初始化或用户 artifact；动态测试只使用隔离、已知 fixtures。
- 不引入 JVM 运行时或外部反编译器执行依赖；JVM 语义恢复由项目自己的 IR 和 recovery 层负责。
- 不承诺恢复 Kotlin coroutine、Scala/Clojure 宏、JSP 或混淆器的原始语言语法。
- 不把 synthetic recovery 当作 XRef 合并，不覆盖原始 BCI/edge，也不实现 P4 现代 record/sealed/condy 深度。

## Decisions

1. **Recovery pass 使用编译期注册和显式前置条件。** 每个模式 pass 声明所需 IR/effect/metadata、输出节点、rule version 和失败 fallback；相比按名字猜模式，能在编译器差异下保持 generic semantics。
2. **优先恢复可证明的局部模式。** 先处理 lambda、concat、accessor、inner/capture、bridge、enum、TWR、interface methods，再进行 region/命名整合；每次 rewrite 保留 OriginSet 和 effect order。
3. **source map 是一等输出。** AST node 到 BCI/CP/attribute 的映射与 Java 文本同时生成；derived accessor/lambda 明确多段来源。相比只输出文本，审查者可以回到原始事实。
4. **验证状态与 representation 分离。** `Structured` 只描述呈现，syntax/recompile/behavior 各自有状态。重编译和行为对照只给样本与 profile 打标签，不能泛化为全输入证明。
5. **通用语法能力优先复用成熟 Rust 库。** 对 Java 语法树、格式化和文档组合能力评估活跃且质量符合要求的 Rust 库，避免重复实现通用语法基础设施；JVM 语义恢复和 IR 适配由项目负责，不引入 JVM 运行时依赖，底层 classfile/ZIP 继续复用既有成熟库。

## Risks / Trade-offs

- [Risk] 模式误识别造成“漂亮但错误”的 Java → 所有 pass 要求完整前置条件，失败即 fallback；保留原始 evidence。
- [Risk] 变量/区域恢复在缺少 debug 或不可约 CFG 时不稳定 → 确定性命名、Partial/Mixed 和 source map。
- [Risk] 受控行为测试被误读为通用语义证明 → 支持矩阵按 fixture/compiler/profile 发布，不扩张承诺。
- [Risk] formatter 对非法 JVM 名称无直接 Java 表达 → 安全别名只影响呈现，strict output 标记 NotJava/Unchecked。

## Migration Plan

以 P2 的结果契约为输入，再增量加入 recovery profile、Java AST/source map 和独立状态字段；若契约需要改变，必须通过 OpenSpec 明确修订，不作旧接口持续有效的承诺。完成 A09/A10/A12/A13/A16 与 Java 8 语料门槛后，P4 才消费稳定的 recovery result contract。
