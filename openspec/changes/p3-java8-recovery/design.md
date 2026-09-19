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

### 1.1 的只读 IR 交接：所有权、生命周期与不建通用框架（2026-09-19 已落地）

`layer-jarde-crates` 把恢复侧输入的所有权留给 1.1 决定，本片按该决定落地，接缝放在 `jarde-jvm` 自己的公开面上（`jarde_jvm::method_ir` 与 `jarde_jvm::engine::analyze_method_ir`）；根门面**不**再导出它，因为 1.3 的 `jarde-java` 直接依赖 `jarde-jvm`，门面的再导出属于 1.3/5.1 的呈现决定，不是本片的必需面。

- **载荷与只读面。** `MethodIr` 由 `jarde-jvm` 拥有，按值持有该次请求发布的三个表（canonical CFG、frames、SSA/effects），每张表是一个 `Option`；`canonical()`/`frames()`/`ssa()` 只交出 `&`，且 `ssa()` 存在的前提是其下有 frames、frames 同理依赖 canonical。表类型（`CanonicalCfg`、`FrameTable`、`SsaTable` 及其记录/标识类型）从私有模块逐个再导出，字段保持 `pub(crate)`，读只经访问器：不建镜像类型，也不公开中端机器（raw CFG、call contexts、fact ledger、pass table、analysis run 与各 pass 入口）。
- **所有权与生命周期。** 一次请求 → 一次运行 → 一份载荷，由调用方按值持有；载荷不借用请求、快照字节或 reader 缓冲（类型无生命周期参数），把 `&MethodIr` 交给恢复层就是调用方作用域内的普通借用。**不需要 `Arc`、全局缓存或第二套生命周期**：同一进程只有一个消费者、没有跨请求身份可供缓存，而缓存会让上一次请求的产物活过为它付费的预算——正是本流水线要保持诚实的计费语义。若将来确有产物需活过作用域，那是带自己所有权与预算故事的新决定，不由本接缝预设。
- **入口选择。** 新增 `analyze_method_ir`，与 `analyze_method` 共享同一条 `run_request` 路径（同样校验、同样调度、同样计费、同样停止），返回 `MethodIrAnalysis`（同一次运行的 report + 载荷）。没有改 `analyze_method` 的签名或行为，也没有把载荷塞进 `MethodAnalysisRequest`/`MethodAnalysisReport` 的 schema：后者会无故改动 P2 的报告、serde 与 golden 契约。
- **不从摘要重建。** 载荷就是那些表本身（`Box` 移入 `MethodIr`），而 `MethodAnalysisReport` 没有任何字段能承载块、BCI、值或 phi（`origin` 仍按 P2 契约留空）。证据见下。
- **不建通用框架。** 一个生产者（`jarde-jvm`）、一个消费者（`jarde-java`，1.3）、一个具体载荷；没有 backend trait、动态 pass 注册、跨层 IR 抽象，也没有为此提前创建 `jarde-java` 骨架（1.3 才随首个真实闭环建包）。
- **计费与停止语义不变。** 交接只移动已发布的表，不读、不重跑、不重复计费；环境被拒的运行交出空载荷，停止/取消的运行交出它在停止前已发布的那部分表。

验证（本片实际执行，命令与数字见下节）：

- `tests/p3_method_ir.rs` 的主用例用 ECJ 4.6.1 v45 `HistoricalControlFlow.finallyPath(I)I`：载荷为 6 块 / 4 边（2 个 `jsr` call + 2 个 return）/ 2 clone / 3 个不可达节点 / 1 个 throw site、3 个 frame 条目、10 个 SSA 值、10 条 effect 记录；帧的 locals 数等于该 body 独立读出的 `max_locals`(5)，块起点是真实指令起点的子集，每条具名指令恰有一条 effect 记录；同一次运行的 report `origin` 为空——摘要面没有可重建 IR 的量。
- 同文件另有两条：只请求 `canonical_cfg` 的运行只交出图（frames/ssa 为 `None`），且两个入口对同一请求计费相同；已取消的运行交出空载荷并保持 `Cancelled`，两个入口一致。
- `crates/jarde-jvm/src/method_ir.rs` 的单元测试汇编一个分支 body（提交语料里没有需要 phi 的 body），载荷交出 1 个 entry phi、2 个操作数、位于合并块与 local slot——phi 面只有真实表能给。
- 源码守卫（同一集成测试）：`jarde-jvm` 公开面没有 `pub fn …(&mut self)` 或 `-> &mut`，接缝模块不公开任何字段；把可变访问器或公开字段加回去守则会红（该守卫是源码级检查，不是编译器证明——Rust 无法在测试里断言"没有 `&mut`"本身）。

仍未完成：1.1 还要定义 RecoveryProfile、模式前置条件、rule version、representation/quality/compile_status/semantic_validation/verification/fallback 类型，以及"未满足前置条件 fixture"的边界验证；本片只落地其中的 IR 交接、阶段有效性与计费/停止不变量。

### 1.1 的产物词汇：把 P3 的结果写成 P2 已有的平面（2026-09-19 已落地）

上节「仍未完成」里 profile、前置条件、rule version、fallback 四项仍成立；其中的 representation/quality/compile_status/semantic_validation/verification 类型就是本节的产物词汇，本片按 P3 规格句实际写下的取值扩展完成。恢复层的**内部声明**（`RecoveryProfile`、模式前置条件、rule version、失败 fallback）不在本片：它们随**第一个真实模式 pass** 在 1.3 落地（依据见下）。本片只做一件事——让 `recovery-validation` 的「每个 source result SHALL 独立返回 representation（…）、quality（…）、syntax_status（…）、compile_status（…）、semantic_validation（…）、verification（…）」在类型上可表达。判定逐句取自规格与架构输出表，不凭印象增删：

| 平面 | P2 基线（本片不改产出） | 本片新增 | 依据句 |
| --- | --- | --- | --- |
| `representation` | `Bytecode` | `Java`、`Mixed` | `recovery-validation`「representation（Java/Bytecode/Mixed）」；`java8-recovery`「representation=Mixed/Bytecode」 |
| `quality` | `Conservative`/`Fallback` | `Structured` | 同上「quality（Structured/Conservative/Fallback）」；`conservative-output`「Java/Mixed 表示与 Structured **质量**留给后续能力」；架构 §13.1 输出表把 `Structured` 列在 quality 行 |
| `syntax_status` | `NotJava` | `Checked`、`Unchecked` | 同上「syntax_status（Checked/Unchecked/NotJava）」；`java8-recovery` 的「非法 Java 名称」降级句与设计风险段都要求区分「生成了 Java」与「检查过语法」 |
| `compile_status` | `NotAttempted` | `Compiles`、`Failed` | 同上「compile_status（NotAttempted/Compiles/Failed）」与「只有实际执行编译且失败时才使用 compile_status=Failed」 |
| `semantic_validation` | 三值已齐 | 无 | 同上；`FixtureDifferential` 正是 P2 为受控对照预留的值，P3 3.3 的受控重编译/行为对照属同一证据类 |
| `verification` | `NotPerformed` | `Performed`、`Failed` | 同上「verification（Performed/NotPerformed/Failed）」 |

- **`Structured` 属 quality，不属 representation。**「成功 Structured 标志」是 `quality=Structured`：架构 §13.1 的输出表把它列在 quality 行，`conservative-output` 的措辞是「Java/Mixed 表示与 Structured **质量**」，`recovery-validation` 又明确「`Mixed` 只表示 representation，禁止把它当作 quality」。因此 representation 加的是 `Java`/`Mixed`，`Structured` 加在 quality；同理 `Java` 落在 representation，`syntax_status` 加的是 `Checked`/`Unchecked`（`NotJava` 已存在）。
- **P2 产出零变化。** 报告装配仍写基线值（`Bytecode`、`NotJava`、`NotAttempted`、`NotPerformed`，quality 仍按 3.5 的规则取 `Conservative`/`Fallback`）；既有变体的名字与 serde 形状未动，既有断言一行未改（817 → 818 只因新增了一条用例）。
- **没有生产者，也不伪造生产者。** 新增变体只被规格句与序列化用例要求；本片不新增任何能产出它们的路径（不为此造 pass、不写进 `AnalysisRun`），3.3 落地受控重编译/行为对照时才决定哪次运行写 `Compiles`/`Failed` 与 `Performed`/`Failed`。
- **一处跨 crate 增补。** `verification` 平面的类型来自 `jarde-reader`（P2 报告复用 `classfile::VerificationStatus`），故 `Performed`/`Failed` 加在该 enum 上而不是另立同名平面；P0–P2 的 header/bytecode/multi-release 报告仍只写 `NotPerformed`，「未执行验证」的语义不变。读者层词汇随阶段扩展有先例：P2 1.3 就向 `jarde-reader` 的 `CountedBudgetDimension` 追加过六个计费维度。
- **为什么不加 `ALL` 常量。** 穷尽性对照写在测试里（见下），因为生产代码没有任何地方遍历这些平面，公开 `ALL` 会为守卫而存在。这与 `EnvironmentProblemCode::ALL` 的分工一致：那里有消费者，所以才在类型上。

**1.3 的交付与依据。** `RecoveryProfile`、模式前置条件、rule version、失败 fallback 跟随模式 pass：本 change 决策 1 要求「每个**模式 pass** 声明所需 IR/effect/metadata、输出节点、rule version 和失败 fallback」，而模式 pass 在 `jarde-java`；`layer-jarde-crates` design 又写明「P3 `1.3` 随第一个真实 Region→AST→文本闭环创建 `jarde-java`」。声明随 pass 走、pass 随首个真实闭环走，因此本片不建空壳 crate、不预置跨层抽象，也不把这份分工解释成漏做。

验证：

- `tests/p3_product_vocabulary.rs`（本片新增，1 条用例，经 `jarde::*` 公共面命名这些类型）：对六个平面逐个断言 (a) 每个取值发布的 JSON 名等于由变体名派生的 snake_case 名、且可反序列化回自身；(b) 该平面的取值集合与声明它的源文件里的变体完全一致且同序——即「规格句写下的取值」与「类型能表达的取值」互为闭包。将来任何加变体的人都会在这里被拦下，直到把对应句子写进这张对照表。
- 证伪两组（`/tmp` 副本 + `sha256sum -c` 还原，独立 `CARGO_TARGET_DIR`，用完删除）：给 `Representation` 临时加 `Pseudocode` 而不改对照表 → 用例红（`left: ["Bytecode","Java","Mixed"]` / `right: […,"Pseudocode"]`）；把该 enum 的 `rename_all` 临时改成 `SCREAMING_SNAKE_CASE` → 用例红（`"BYTECODE"` vs `"bytecode"`）。
- 「P2 从不产出这些取值」由既有断言复证，本片未改它们：`tests/p2_contracts.rs`、`tests/p2_properties.rs`、`tests/p2_frame.rs`、`tests/p2_ssa.rs`、`tests/p2_cfg.rs`、`crates/jarde-cli/tests/json_cli.rs` 都在真实请求上断言基线值。

仍未完成（1.1 的剩余项）：任务文本要求的「未满足前置条件 fixture」边界与不误识别验证依赖前置条件类型，故随 1.3 的模式 pass 一并验证；「真实 P2 结果消费」由 1.1a 的 `tests/p3_method_ir.rs` 覆盖。

## Risks / Trade-offs

- [Risk] 模式误识别造成“漂亮但错误”的 Java → 所有 pass 要求完整前置条件，失败即 fallback；保留原始 evidence。
- [Risk] 变量/区域恢复在缺少 debug 或不可约 CFG 时不稳定 → 确定性命名、Partial/Mixed 和 source map。
- [Risk] 受控行为测试被误读为通用语义证明 → 支持矩阵按 fixture/compiler/profile 发布，不扩张承诺。
- [Risk] formatter 对非法 JVM 名称无直接 Java 表达 → 安全别名只影响呈现，strict output 标记 NotJava/Unchecked。
- [Risk] 基础闭环被所有语法糖、通用接口或共享库抽取拖住 → 先验收单方法可读输出和可解释 fallback；各模式独立增量，不提前增加生产实体。

## Migration Plan

以 P2 的结果契约为输入，再增量加入 recovery profile、Java AST/source map 和独立状态字段；若契约需要改变，必须通过 OpenSpec 明确修订，不作旧接口持续有效的承诺。完成 A09/A10/A12/A13/A16 与 Java 8 语料门槛后，P4 才消费稳定的 recovery result contract。

实施顺序为 **1.1 → 1.2 → 1.3（包含 3.1/3.2 的最小命名与映射子集）→ 2.x 各模式 → 3.x 完整覆盖与发布**。1.3 的退出条件是简单方法从真实库入口到 Java 文本/source map 可用，语法糖规则未命中仍能输出通用表达，复杂异常或资源停止能返回正确 fallback/状态；库/薄 CLI 消费相同结果。该闭环只是 P3 首片交付，不代表 P3 完成或全体 45–52 方法都可表达为 Java。3.1/3.2 在同一实现上扩展作用域、派生跨方法 origin 等覆盖，不另起第二套命名和映射系统。
