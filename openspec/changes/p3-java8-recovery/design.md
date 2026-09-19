## Context

以 P2 完成为进入条件。本 change 规划消费 Demand Resolver、CanonicalCFG、Frame/SSA、effects 和 Conservative output；当前尚未实现。恢复是可选呈现层，必须保持 P1 X1 的原始引用与 P2 IR origin；Java 8 是第一套高质量承诺，不代表所有 JVM 字节码都能表达为 Java。 2026-09-19 的 P2 已交付到 5.2，但异常固定点与资源计费修正 4.2b/4.3b 和整体出口尚未关闭，因此本阶段继续保持未开始。

## Goals / Non-Goals

**Goals:**

- 在 Java 8 RuntimeProfile 中以 evidence-gated passes 恢复高频 javac/ECJ 模式和稳定变量命名。
- 先交付一个无需编译器语法糖规则也能运行的单方法闭环：P2 SSA → 普通控制流 Region → Java AST → 文本/source map/明确 fallback，再逐项增加高频模式。
- 评估可用的 Rust Java AST、formatter 和文档组合库，提供 source map，并独立记录 representation、quality 和 validation status；不预设一定存在符合纯 Rust 与语义要求的完整 Java AST 库。
- 以真实历史/Java 8 语料和受控重编译/行为样本建立支持矩阵。

**Non-Goals:**

- 不执行目标 bootstrap、静态初始化或用户 artifact；动态测试只使用隔离、已知 fixtures。
- 不引入 JVM 运行时或外部反编译器执行依赖；JVM 语义恢复由项目自己的 IR 和 recovery 层负责。
- 不承诺恢复 Kotlin coroutine、Scala/Clojure 宏、JSP 或混淆器的原始语言语法。
- 不把 synthetic recovery 当作 XRef 合并，不覆盖原始 BCI/edge，也不实现 P4 现代 record/sealed/condy 深度。

## Decisions

1. **Recovery pass 使用编译期注册和显式前置条件。** 每个模式 pass 声明所需 IR/effect/metadata、输出节点、rule version 和失败 fallback；相比按名字猜模式，能在编译器差异下保持 generic semantics。
2. **先有通用恢复闭环，再增加局部模式。** 首片覆盖简单表达式、调用、return、if/loop/switch 的可证明子集，并提供最小稳定命名、source map 与 bytecode fallback；不等待 lambda、concat、accessor、enum、TWR 等规则齐全后再整合输出。后续模式消费所需 ValueIR/Region/metadata，在自己的固定阶段内恢复；每次 rewrite 保留 OriginSet 和 effect order。少量可证明的 IR 局部重写仍可在 Region 前执行，这里调整的是交付顺序，不把所有 rewrite 强制放到 AST 后。
3. **source map 是一等输出。** AST node 到 BCI/CP/attribute 的映射与 Java 文本同时生成；derived accessor/lambda 明确多段来源。相比只输出文本，审查者可以回到原始事实。
4. **验证状态与 representation 分离。** `Structured` 只描述呈现，syntax/recompile/behavior 各自有状态。重编译和行为对照只给样本与 profile 打标签，不能泛化为全输入证明。
5. **通用语法能力优先复用成熟 Rust 库。** 对 Java 语法树、格式化和文档组合能力评估活跃且质量符合要求的 Rust 库，避免重复实现通用语法基础设施；JVM 语义恢复和 IR 适配由项目负责，不引入 JVM 运行时依赖，底层 classfile/ZIP 继续复用既有成熟库。

### droidsaw 中端设计的吸收边界

版本与源码取舍沿用 [P2 design §6.1](../p2-jvm-ir/design.md)。本次吸收以下边界，在 `jarde-java` 内部用具体模块和函数表达；不为每个算法再拆 crate，实际需要多个实现之前不建通用 backend trait 或动态 pass 注册层。workspace 归属见 [layer-jarde-crates](../layer-jarde-crates/design.md)：P3 1.1 先明确由 `jarde-jvm` 拥有的只读 method/CFG/value/effect/origin 输入契约，1.3 随首个真实闭环创建 `jarde-java`，根 `jarde` 只做入口委托；当前摘要型 report 不被假定为已经具备恢复所需全部输入。 `eac3759` 的 driver 在返回前释放 canonical/frame/ssa 表；1.1 要决定请求内真实载荷的所有权和生命周期，1.3 用同一份产物接通恢复，避免从 report 拼回 IR 或无条件重复 P2。接缝只暴露所需只读访问及阶段有效性，预算、停止和 origin 随请求传递，不为此先造公共可变中端、缓存或通用 backend。

- **图的视图与语义事实分开。** 借鉴 DEX 的 `NormalFlow`，普通 if/loop/switch 的支配关系使用过滤后的正常控制流视图，尽量借用现有图。异常恢复使用完整的 throw-site/context、保护区间、catch 类型与 handler 顺序；处理 handler 自身的普通控制流时明确其入口。不能全局删除异常边，也不能用正常流支配关系推出异常语义。
- **Region 决定结构，Java 输出负责语法。** 结构恢复消费 CFG/SSA 的条件、终结指令和 effect，构造已有规划中的 RegionIR；AST/formatter 消费 Region 和 origin。边发现、循环识别、异常范围推导不放进 formatter，AST 也不再解码 bytecode。droidsaw 的 `StmtBackend` 展示了这种分工，但 Jarde 首期只有 Java 消费方，使用具体私有函数即可。
- **正常循环不能破坏 effect。** 条件和 header 内的调用、读取、潜在异常按原执行次数和次序保留；不能因输出 `while` 就把每轮执行的 header 操作移到循环外。异常区域先支持有证据的嵌套形状，交叉/不可约情形明确 fallback；首片可以对整个方法保留 bytecode，不为了首版 Mixed 输出增设复杂片段拼接系统。
- **可选规则失败保留基础结果。** 先检查匹配前提和成本，再应用局部变换；普通 no-match 使用已有通用结构，内部不变量失败给诊断并保留最后可靠表示，预算耗尽/取消仍按真实 execution 停止。无需新增事务框架。若未来调库，选可报告失败的入口；不能把深度超限后的空 Region 当合法空 body。
- **独立对照有明确边界。** 参考 common 的小图 oracle 思路：普通可约 CFG 可用独立控制流/路径模型对照，核对分支极性、循环内 effect 次数和可达 return；不要求唯一的 Region 树形状。common 自带 Region oracle 明确不覆盖 switch、try/catch 与不可约 SCC，不能拿它的成功替代 JVM handler 顺序、finally、monitor、TWR 测试。这些继续用已知 fixtures 的预期事件与受控对照验收。

当前 common Region 的单 handler 接口不能直接表达完整 JVM 异常区域；ASC 的 DEX 后端也使用自己的 `structure.rs`，未直接调用这个 Region builder。因此 P3 当前按上述边界实施自己的 JVM 区域恢复，图算法继续复用 petgraph，formatter 按 1.2 准入。未来若有可薄适配的库，按无异常/多 handler/effect/资源停止四类证据重评；不预设拆分 droidsaw 才能继续 P3，也不宣称自研实现已优于现成实现。

## Risks / Trade-offs

- [Risk] 模式误识别造成“漂亮但错误”的 Java → 所有 pass 要求完整前置条件，失败即 fallback；保留原始 evidence。
- [Risk] 变量/区域恢复在缺少 debug 或不可约 CFG 时不稳定 → 确定性命名、Partial/Mixed 和 source map。
- [Risk] 受控行为测试被误读为通用语义证明 → 支持矩阵按 fixture/compiler/profile 发布，不扩张承诺。
- [Risk] formatter 对非法 JVM 名称无直接 Java 表达 → 安全别名只影响呈现，strict output 标记 NotJava/Unchecked。
- [Risk] 基础闭环被所有语法糖、通用接口或共享库抽取拖住 → 先验收单方法可读输出和可解释 fallback；各模式独立增量，不提前增加生产实体。

## Migration Plan

以 P2 的结果契约为输入，再增量加入 recovery profile、Java AST/source map 和独立状态字段；若契约需要改变，必须通过 OpenSpec 明确修订，不作旧接口持续有效的承诺。完成 A09/A10/A12/A13/A16 与 Java 8 语料门槛后，P4 才消费稳定的 recovery result contract。

实施顺序为 **1.1 → 1.2 → 1.3（包含 3.1/3.2 的最小命名与映射子集）→ 2.x 各模式 → 3.x 完整覆盖与发布**。1.3 的退出条件是简单方法从真实库入口到 Java 文本/source map 可用，语法糖规则未命中仍能输出通用表达，复杂异常或资源停止能返回正确 fallback/状态；库/薄 CLI 消费相同结果。该闭环只是 P3 首片交付，不代表 P3 完成或全体 45–52 方法都可表达为 Java。3.1/3.2 在同一实现上扩展作用域、派生跨方法 origin 等覆盖，不另起第二套命名和映射系统。
