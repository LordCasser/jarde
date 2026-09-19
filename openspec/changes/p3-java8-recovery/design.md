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

### 1.2 的库评估与 1.3 的最小实现路线（2026-09-19）

**结论先行。** 在 2026-09-19 当日对 crates.io 的全量检索里，**没有任何 Rust 库同时满足 6 条准入**；决定性的一条是准入 3（origin 可映射）：所有候选的发射 API 都是「一次调用 → 一个 `String`」，而 P3 决策 3 要求 AST node 到 BCI/CP 的映射与文本**同时**产生、不得事后从成品文本反推，节点级位置只能在渲染过程中记录。因此 1.3 的最小实现是**自有的最小 Java 语句/表达式 AST + 记录偏移的 emitter**；复用面是仓库已有的 `petgraph`（图算法）与既有 classfile/ZIP 依赖（决策 5 早先那条基线）。**本片不新增任何依赖**（`Cargo.toml` 一字未改），也不建通用 backend trait。

#### 检索面（本次实际执行的工具与查询）

- **crates.io API**：`/api/v1/crates?q=…` 与 `/api/v1/crates/{name}`、`/{name}/{version}/dependencies`，取版本、许可、下载量、最近下载、最后更新、依赖清单与 repo。查询词：`java parser`、`java ast`、`java formatter`、`java decompiler`、`java codegen`、`java emitter`、`java writer`、`java source generator`、`java pretty print`、`javadoc`、`documentation comment`、`jvm bytecode decompiler`、`java bytecode`、`decompiler java class`、`pretty printer`、`wadler`、`tree-sitter-java`；另按 `keyword=java&sort=downloads` 取全量前 40 做兜底（共 274 个带该关键词的 crate）。
- **GitHub API**：`/repos/{r}` 与 `/repos/{r}/commits`（星数、最后 push、最近提交）用于准入 6；核对 `Eatgrapes/JSyntax`、`mzdk100/java-lang`、`ejfkdev/jdc-core`、`ejfkdev/jcdc`、`Marwes/pretty.rs`、`tree-sitter/tree-sitter-java`。
- **exa 网页检索** 3 组：Rust 解析 Java 源码的 AST 库、Rust 的 Java 格式化/美化库、"generates Java source code from an AST with source position mapping"。
- **读候选源码**（不是读 README）：把 `jsyntax 0.1.0`、`jdc-core 0.1.9`、`java-lang 0.3.2`、`jarust-ast 0.1.0`、`pretty 0.12.5` 的 `.crate` 解到 `/tmp` 逐文件核对 API 形状与词法实现。

**查了但没有（如实记录）**：没有满足全部准入的完整库；**没有任何 Rust Java emitter 暴露「渲染中回调/位置报告」接口**；没有任何面向「字节码 → Java 文本 + source map」的库（现存 Java emitter 全是 IDL/SDK/协议 codegen：`zerodds-idl-java`、`rdc`、`baml sdkgen_java`、`brec_java_gen`……发的是各自 schema 的固定骨架，没有 origin，也没有预算接口）；没有任何 Java **注释文本生成**库（现存 `javadoc` 命中的全是解析器）。

#### 判定图例

准入 ①：纯 Rust、无 JVM 运行时依赖（不得间接引入 JVM 或需要外部反编译器进程）；②：许可证在 `deny.toml` 允许集合内且**不需要新增全局例外**；③：origin 可映射（AST node 能携带/关联我们给的 BCI/CP/attribute，或 API 能把 node 与我们生成的位置绑定）；④：转义由库负责或可被我们控制；⑤：输出预算可约束（分步/可度量，或先算规模再输出，不能是黑盒一次吐全）；⑥：活跃度与质量（最近发布/提交、下载量/被采用、测试与文档、维护者响应迹象）。`—` 表示该条的判定对象不存在（类型不适用）。

#### 表 1 解析器族：**方向不符**，整族不适用（①③④⑤⑥ 只在「将来要解析 Java 源码」时才相关）

| 候选 | 版本 / 许可 / 最后活跃 | ① | ② | ③ | ④ | ⑤ | ⑥ | 判定 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `java-lang` | 0.3.2 / MIT OR Apache-2.0 / 2026-06-11（2★，1004 dl，依赖 `thiserror`+`unicode-xid`） | ✅ | ✅ | ❌ 只有 `Span`（源码字节偏移），**无 emitter** | — | — | ⚠ 单一作者、无 CI 迹象 | 不满足（且方向相反） |
| `jarust-ast` | 0.1.0 / MIT OR Apache-2.0 / 2026-09-18（13 dl，**零依赖**） | ✅ | ✅ | ❌ | — | — | ❌ 发布 1 天、2 个提交 | 不是 AST：全库 33 行，`AstNode::Method { body: String }` |
| `java-ast-parser` | 1.0.0 / MIT / 2026-04-06（226 dl） | ✅ | ✅ | ❌ | — | — | ⚠ | 自述 "without initializers and function bodies"，连方法体都不表示 |
| `oak-java` | 0.0.11 / **MPL-2.0** / 2026-03-30（473 dl，12 版） | ✅ | ❌ **不在 allow 集合** | ❌ 红绿树无 origin 槽位 | — | — | ⚠ 0.0.x | 不满足（见「若要走例外」） |
| `tree-sitter-java`（+ `tree-sitter`） | 0.23.5 / MIT / 2024-12-21（11.3M dl，275★） | ❌ 核心是 C（`cc` 编译生成 C 解析器），非纯 Rust | ✅ | ❌ | — | — | ✅ 业界标准 | 解析源码用，P3 不解析源码 |
| `rezel-lang-java` | 0.0.0 / MIT OR Apache-2.0 / 2026-09-06（12 dl） | ✅ | ✅ | ❌ | — | — | ❌ | 不满足 |

**为什么整族不适用（结构性理由，不是偏好）**：P3 的方向是**生成**——我们有 Region、SSA、origin，要产出 Java 文本；解析器族解决的是「源码 → AST」，而 class file 里**没有 Java 源码**可解析。同理，class file 里也**没有 doc comment**：`tree-sitter-javadoc 0.3.1`（MIT，2026-03-22，31.8k dl）、`doctor 0.3.4`（MIT，2021-01-12，**5 年未更新**）、`oak-javadoc`（MPL-2.0）这些 javadoc 解析器**没有可解析对象**（唯一可用的 debug 证据是 LVT/LineNumberTable/MethodParameters/Signature）。因此注释能力的判定只在**生成侧**，而那侧没有任何库。

#### 表 2 发射 / 反编译族：形状对得上，但准入 3、5、6 全数不满足

| 候选 | 版本 / 许可 / 最后活跃 | ① | ② | ③ | ④ | ⑤ | ⑥ | 判定 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `jsyntax` 0.1.0 | MIT / 2026-09-18（10 dl；仓库同日创建、**2 个提交、0★、无 README、无 tests 目录**） | ✅ 纯 Rust，`forbid(unsafe_code)`；依赖 `unicode-general-category`（Apache-2.0）+ 可选 `ferro-jtype` | ✅ 闭包全在 allow 集合 | ❌ **AST 无 id/span 字段，`to_java_with(&PrintOptions) -> Result<String, EmitError>` 只回文本**；只能「并行表 + 逐节点发射」关联，节点级位置拿不到 | ✅ 见下 | ❌ 内部 `String` 累加，无 sink/回调/上限，只能按节点分步 | ❌ 发布 1 天、0 采用、无测试 | 不满足 |
| `jdc-core` 0.1.9 | MIT / 2026-09-18（112 dl；仓库 2026-09-15 创建、1★、包 2026-09-16 起 2 天内 8 个版本；带 `tests/`） | ✅ 依赖 `bitflags` + 可选 `serde` | ✅ | ❌ **全库无 origin→IR 映射**：`ir::Expr/Stmt` 不带 pc，`Cfg::Block` 的 `u32` offset 不进入语句树；发射器只认 `VarTable` | ✅ `escape_string`/`escape_char` 到 `\uXXXX` | ❌ `Printer { out: String, .. }` 私有累加 | ❌ 2 天、无第三方依赖 | 不满足；**且它是整条 structuring(7.7k 行)+convert(1.7k)+emit(3.6k)**，采用它会替换决策 2 划给项目的 Region/语义恢复，并要求实现其 `Ctx` 前端 trait——超出决策 5 的复用边界 |
| `jcdc` / `jcdc-decompiler` | 0.1.1 / MIT / 2026-09-14（10–12 dl） | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | 同上，且是 `jdc-core` 的 CLI 侧 |
| `rusty-javac` | 0.2.3 / MIT / 2026-05-30（150 dl） | ✅ | ✅ | ❌ | — | — | ⚠ | 方向相反（源码→字节码），其 AST 只服务编译 |
| `coffea` | 0.1.0 / MIT / 2020-06-22（近 30 天 9 dl） | ✅ | ✅ | ❌ | — | — | ❌ 自称 WIP、5 年未动 | 不满足 |
| `ferro-jtype` | 0.2.5 / MIT / 2026-09-18（365 dl） | ✅ | ✅ | — | — | — | ⚠ | 字节码类型推断，非 AST/发射；只有在 1.3 之后要做类型呈现时才相关 |

`jsyntax` 值得单独记两笔：它的词法实现是这批候选里**唯一**认真处理准入 4 的（`escaped()` 按 UTF-16 单元转义、控制字符走八进制/`\uXXXX`、`NaN`/`Infinity` 展开成 `(0.0F / 0.0F)`；`identifier()` 拒绝保留字与非法字符；`comments()` 显式拒绝注释里的 `\u` 与 `*/`——注释里的 Unicode 转义是 JLS 的真实陷阱），且带 Java 语言级别门控（`PrintOptions::language`）——只是**没有任何测试随包发布**，这些规则没有被它的作者用可复核的方式固定下来。它是「将来若有带位置回调的成熟版本，可以重评」的对象，不是今天可以承重的对象（1 天的 0.1.0 不能承担「支持矩阵按多代 javac/ECJ 语料发布」的稳定面）。

#### 表 3 布局/文档组合（Wadler Doc）族：允许，但只解决排版，不解决 origin/转义

| 候选 | 版本 / 许可 / 最后活跃 | ① | ② | ③ | ④ | ⑤ | ⑥ | 判定 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `pretty` | 0.12.5 / MIT / 2025-09-26（21.8M dl，181★；依赖 `arrayvec`/`typed-arena`/`unicode-width`） | ✅ | ✅ | ⚠ 只有 `annotate(A)` + `RenderAnnotated::{push,pop}_annotation`：**自定义 sink 能在渲染时记录标注开合点**（`A` 可以是我们的 OriginSet），但引擎本身不报位置，未标注文本无从对应 | — 语言无关 | ✅ `render_fmt(width, &mut W: fmt::Write)`，预算 sink 返回 `Err` 即中途中断 | ✅ | 引擎可用，但准入 3 只能经「标注 sink + 我们自己的字典」间接满足，准入 4 仍归我们：**不是能承重的完整实现**；见下 |
| `prettyless` | 0.3.0 / MIT / 2025-07-17（81k dl） | ✅ | ✅ | ⚠ 同上族 | — | ✅ | ✅ | 同族替代品 |
| `tiny_pretty` | 0.4.3 / MIT / 2026-08-05（912k dl） | ✅ | ✅ | ⚠ | — | ✅ | ✅ | 同族替代品 |
| `pretty-lang` | 1.0.0 / Apache-2.0 OR MIT / 2026-07-07（22 dl，`no_std`+`forbid(unsafe_code)`，`render_into`/`render_writer`） | ✅ | ✅ | ⚠ | — | ✅ | ❌ 发布 2 个多月、无采用 | 不满足 |
| `oak-pretty-print` | 0.0.11 / **MPL-2.0** / 2026-03-29（9928 dl） | ✅ | ❌ | ⚠ 需 oak 红绿树 | — | ✅ | ⚠ | 不满足（许可） |
| `sourcemap` | 9.3.2 / BSD-3-Clause / 2026-01-20（36.1M dl） | ✅ | ✅ | — 这是**序列化格式**库，不是发射器 | — | — | ✅ | 记在 3.2 名下：若将来要把我们的映射导成标准 source map 文件，它是现成的成熟选择；1.3 不需要 |

**为什么 1.3 不引入 Doc 引擎。** 布局引擎能用 `annotate` + 自定义 sink 把准入 3 重新变成可达（这是它唯一真正的加分项），代价是：① 我们仍要写全部文本拼装、转义与命名；② 每个节点都必须被标注，并额外维护「标注 → Origin」字典，映射多了一层可脱节的机制；③ 换来的是**宽度重排**——P3 三份 delta 里没有任何一句要求它。而我们自己的 emitter 里，节点 range 是同一次写入的副产物（一次机制，不可能失配），插入点正好就是预算检查必须存在的位置。因此 Doc 引擎记为**将来若出现宽度重排需求时的首选重评对象**（届时准入 3 仍可满足），不是今天的最小实现。

#### 决定：1.3 的最小实现路线（Route A）

代价一句话：**我们承担决策 2 那个可证明子集的词法与版式**（不含完整 Java），换来 origin、转义和预算三条准入由同一层同时满足，且 0 新依赖。

1. **AST 与 formatter：自己写最小 Java 语句/表达式 AST + 自己的 emitter**，放 `jarde-java` 私有模块。理由不只是「没有库满足准入」，还有结构性的两条：(a) 解析器族方向相反；(b) 发射族要么不带 origin、要么连 Region 一起替换掉（越界）。**边界（明确不覆盖）**：只做决策 2 的「简单表达式、调用、return、if/loop/switch 的可证明子集」；不覆盖 lambda/method reference、`StringBuilder` 拼接、TWR/synchronized/finally、内部类/枚举、注解与泛型签名的完整语法面（2.x 逐项加），也不做宽度重排（固定版式，确定性优先）。这不是「重复实现通用语法基础设施」，而是把决策 2 要求的最小呈现面连同它的 origin/预算义务一起放在一层里。
2. **source map 载体：AST node 持有 `OriginSet`，emitter 同步产段表。** `OriginSet { primary: Origin, derived: Vec<Origin> }`，`Origin { bci: u32, cp: Option<u16>, provenance: Direct | Derived }`；段表 `Segment { start, end, origin: OriginSet }` 用**生成文本的字节区间**做键（不是行/列，因为我们不重排）。多段来源（派生 accessor/lambda）落在 `derived` 上并标 `Derived`，正是 `source-maps` 规格里「同时包含调用 accessor 的原始 BCI 和 accessor 访问字段的 BCI，并标记为 derived」的形状。**这就是 1.3 的接口前提**：3.1 的命名与 3.2 的 CP/attribute、跨方法派生都往同一张段表上长，不另起第二套。
3. **文档组合：不需要库；注释文本组装是我们自己的小函数。** class file 里没有 doc comment 可解析（见上）；能写进注释的只有我们已有的证据（原始名、descriptor、解析到的类型、fallback 原因），组装就是把文本折行、加 `*` 对齐，并拒绝 `*/` 与 `\u`（JLS 危险点，`jsyntax` 的实现值得照抄这条规则）。
4. **转义与输出预算在哪一层：都在 emitter 一层，且都在写入口。** 转义：字符串/字符字面量按 **UTF-16 单元**转义（补充字符 → 代理对，`\uXXXX`），控制字符、`"`、`\`、`\n` 等显式处理；标识符走「关键字 + 非法字符」自检，不可写就出**确定性安全别名**（原拼写只作为 evidence 保留，不进文本）。预算：**每个 `put`/`line` 前检查**（可复用既有 `CountedBudgetDimension` 的 `OutputBytes`，节点/边记 `IrItems`、算法步记 `AnalysisSteps`，具体清单由 1.3 定），超限返回带 `written`/`limit`/停顿 BCI 的理由并**丢弃部分文本**——调用方不会拿到半成品当结果、更不会拿到「成功」状态。这与 `jvm/cfg.rs` 已记录的已知边界一致：petgraph 的支配点算法内部没有中断钩子，所以**必须在进入算法前**用块数上限计费（沿用该模块 `MAX_BLOCKS_DEFAULT` 的做法），不能指望算法中途停下。
5. **探针证据**（一次性，只在 `/tmp`，不入库；`sha256` 见下）。`/tmp/p3probe/probe.rs`（std-only，`rustc --edition 2021 -O`）实跑 **PROBE OK**，27 项断言全绿（每条打印 `ok`，失败即退非零），其中直接支撑准入 3/4/5 的是：
   - 一次 `if (count + 1) { other.count + 1; } else { return; } return 0;` 的发射得 **71 字节 / 12 段**；字段节点自身的区间**恰为 `other.count`**，其 origin 为 `primary bci 30 / cp 9（Direct）` + `derived bci 55（Derived）`；**共享同一 BCI 30 的表达式段与语句段各自保留不同区间**（`other.count + 1` 与 `    other.count + 1;\n`）——即「node → 文本区间」是映射表说了算，不需要回扫文本。
   - 预算：同一发射在 `limit=65` 时于写出 **61 字节**后停在 **bci 20**，返回 `Budget { written: 61, limit: 65, at: bci 20 }`，缓冲里只剩这 61 字节且调用方不构造输出——**中断发生在节点内部**，不是语句边界。
   - 转义 9 条向量 + 2 条性质：`"` → `\"`、`\` → `\\`、`\n` → `\n`（转义）、`\u0000`/`\u0007`/`\u007f`/`\u2028` 全部走 `\uXXXX`、`😀` → `\ud83d\ude00`（UTF-16 语义）；产物无裸控制字符、`"` 只由 `\"` 产生；`int` 不是合法标识符且获得确定性别名 `int_`。
   - 平面：`Mixed + Fallback + CompleteWithinSchema` 与 `Java + NotJava` 两组组合在产物上直接构造成功（见下节）。
   `probe.rs` `sha256 43db4815…7598`；二进制 `sha256 517d26e8…2195`。**注意它的定位**：它证明的是**路线可行**（三类准入由一层同时满足），**不是**证明自研实现优于现成实现——后者本片没有证据，也不宣称。
6. **若要走许可例外（`oak-java`/`oak-pretty-print` 的 MPL-2.0）**：需要往 `deny.toml` 的 `[[licenses.exceptions]]` 加一条 crate/版本收窄的例外（P1 对 `libfuzzer-sys@0.4.13` 的 NCSA 已有先例）。代价明显更高：NCSA 那条是 **test-only** 依赖，而 MPL-2.0 是弱 copyleft，会覆盖一个生产依赖；且这两个 crate 仍不满足准入 3。**1.2 不建议走**，理由与代价一并记在此处供 1.3 复核。

#### 四者职责与数据流（正常流图视图 / 异常事实 / Region / Java 输出）

| 层 | 职责 | 输入 | 输出 | 明确不做 | 归属 |
| --- | --- | --- | --- | --- | --- |
| **正常流图视图** | 用**过滤后的正常控制流**判定支配关系、循环与可约性 | canonical CFG 的块/普通边（`CanonicalCfg::canonical()/blocks()/edges()`） | 只读的派生视图（块身份 → `petgraph::NodeIndex` 的映射 + 各算法结果） | 不删边、不改写 canonical、不从它推异常语义、不判 handler 归属 | `jarde-java`（消费 `jarde-jvm` 的只读载荷） |
| **异常事实** | 提供 throw site、保护区间、catch 类型与声明顺序、effect 次序 | canonical 的 `throw_sites`/`handler_rows` + SSA effects | 区域边界与 handler 入口判定的**事实输入** | 不全局删异常边、不用正常流支配关系推异常语义、不改 effect 次数与次序 | 事实由 `jarde-jvm` 拥有；消费在 `jarde-java` |
| **Region** | 决定结构：循环/条件/switch/异常区域的形状与嵌套 | 正常流图视图（结构）+ 异常事实（范围）+ SSA 条件与终结指令 + effect | Region 树（AST 的唯一输入） | 不发文本、不解码字节码、不决定语法；证据不足**绝不**产出空 body | `jarde-java` |
| **Java 输出** | 决定语法：AST（带 `OriginSet`）+ emitter（文本、段表、预算、转义、命名自检、诊断） | Region + origin + 命名决策 | Java/Mixed 文本 + source map + 诊断 + fallback 片段 | 不做边发现、循环识别、异常范围推导；不解析 bytecode | `jarde-java` |

数据流方向**单向**（与归档的 `layer-jarde-crates` 一致）：

```
jarde-reader（facts：method/code/exception table/attribute/CP）
        │ 既有
        ▼
jarde-jvm   raw CFG（petgraph DiGraph，BCI 为节点）→ canonical CFG（blocks/edges/throw sites/handler rows）+ frames/SSA/effects/origin
        │ MethodIr 的**只读**借用（1.1 已交付，载荷即那三张表）
        ▼
jarde-java  ①正常流图视图 → ②异常事实 → ③Region → ④AST + emitter → 文本/段表/诊断
        │ （Region、AST、命名、source map 全部属 jarde-java，见 layer-jarde-crates 的表）
        ▼
根门面 jarde / jarde-cli（只做入口委托与呈现）
```

- **`jarde` 底座不得反向依赖 `jarde-java`**：`jarde-jvm`/`jarde-reader`/`jarde-query` 不认识 Region、AST、命名或段表；`jarde-java` 由 1.3 随首个真实闭环创建（本片不建骨架）。恢复侧也**不**改写原始 X1 facts 或反向调用统一门面。
- **沿用 `petgraph`（本片不新增依赖）**：既有 `jarde-jvm` 的 raw CFG 已经用 `petgraph::graph::DiGraph`（`crates/jarde-jvm/src/cfg.rs`）。1.3 只在**①正常流图视图**这一层用它：以 canonical 块标识为键建/复用一份过滤后的有向图，调用 `dominators::simple_fast`、`tarjan_scc`、`toposort` 等既有算法，结果只读。**边界**：petgraph 的支配点算法没有中断钩子（`cfg.rs` 已记录），所以上限与计费必须在**进入算法前**收口（块数上限 + `IrItems`/`AnalysisSteps`），不能把「算法跑到一半停下」当作预算机制。
- **不新增通用 backend trait**：生产者一个（`jarde-jvm`）、消费者一个（`jarde-java`）、被呈现的语言一套（Java）。四者用 `jarde-java` 内的**私有模块 + 具体函数**表达即可；现在加 `trait Backend`、动态 pass 注册或跨层 IR 抽象，正是 P3 design 明确推迟到「实际需要多个实现之前」的东西，也会让「Region 决定结构、Java 输出负责语法」这条分工被抽象层冲淡。将来若真出现第二个消费者（P4 的现代语义插件），再按那时的证据决定，而不是提前造。

#### 产出平面可独立表达

六个平面各自的取值来源互相独立，且类型之间**没有任何 `From`/构造耦合**（`crates/jarde-jvm/src/ir.rs` 里 `Representation`/`Quality`/`SyntaxStatus`/`CompileStatus`/`SemanticValidation` 之间无 `impl From`，本片核对为空）：

| 平面 | 1.3 由什么写 | 独立输入 |
| --- | --- | --- |
| `representation` | 装配：有任一区域回退到 bytecode → `Mixed`；全部区域是 Java → `Java` | 区域回退集合（与结构强度无关） |
| `quality` | 按区域的结构恢复强度，整体取最弱处；`Structured` 只描述呈现 | 每个 region 的前提满足情况 |
| `syntax_status` | 名字/结构能否写成 Java（别名只影响呈现）；`Checked`/`Unchecked` 由是否真跑过语法检查决定 | 标识符自检 + 是否跑了检查 |
| `compile_status` | **只有 3.3 真跑了编译**才写 `Compiles`/`Failed` | 受控重编译的执行 |
| `semantic_validation` | P2 的 SSA 不变量（`LocalInvariants`）/3.3 的 fixture 对照（`FixtureDifferential`） | 不变量检查或对照执行 |
| `verification` | 只有真跑了验证才写 `Performed`/`Failed` | 独立执行 |

规格明写的两种组合，本路线都能产出（探针上直接构造成功）：

- **`representation=Mixed` + `quality=Fallback` + `coverage=CompleteWithinSchema`**：扫描**完整**跑完，但某个区域只保留了可靠低级结构（`BytecodeFallback` 节点，文本形如 `// @bytecode 44 45 46 …`）。三件事分别来自「区域呈现混合」「区域结构强度」「扫描是否完成」，没有任何一步把它们绑在一起——`quality` 也不会被改写成 `Partial`（`Partial` 属 coverage，不属 quality）。
- **`representation=Java` + `syntax_status=NotJava`**：原始名是 Java 关键字/非法标识符/混淆名时，文本里出现的是**确定性别名**（例：`int` → `int_`），呈现仍是 Java（`representation=Java`），但该结果不被声称为合法 Java 语法（`syntax_status=NotJava`），并同时是 `compile_status=NotAttempted`、`semantic_validation=Unproven`、`verification=NotPerformed`——与 `recovery-validation` 的 `Structured output cannot compile` 场景逐字对应。

做不到的（明说）：本片**没有**任何生产者能写出 `Checked`/`Compiles`/`Performed` 这些强状态（1.1b 已如此记录），1.3 也只写 `Unchecked`/`NotAttempted`/`NotPerformed` 一侧；谁能写 `Compiles`/`Failed` 与 `Performed`/`Failed` 仍由 3.3 决定。

#### 1.3a 的实际落点与事实缝（2026-09-19）

1.2 定下的 Route A 已按原样落地，**没有**新增第三方包（`Cargo.lock` 只多出 `jarde-java` 这一个 workspace 成员），四者职责各有一个模块，数据流单向：

| 层 | 模块 | 实际提供的面 | 明确不做 |
| --- | --- | --- | --- |
| ① 正常流图视图 | `normal_flow::NormalFlowView` | 只保留 `Normal` + `Return` 边的投影（`petgraph::DiGraph`）、`successors/predecessors`、`immediate_post_dominator`（虚出点 + 反向支配）、`cyclic_blocks`（`tarjan_scc`）、`excluded()` 计数 | 不删 canonical 的边、不写回、不从投影推异常语义；异常/`jsr` 边只在 `excluded()` 里计数 |
| ② 异常与解码事实 | `facts::{RecoveryFacts, Operation}` | 方法身份、每槽 debug 名、每条 BCI 的解码操作（Push/Load/Store/Arithmetic/Comparison/Invoke/Return/Transfer/Other） | 不产语句、不决定语法；`Other` 与「未解码」是**被陈述**的输入，不是猜测 |
| ③ Region | `region::{recover, Region, FallbackReason}` | 直线 run、`If{prefix, branch, then_arm, else_arm, join}`、`Fallback{blocks, reason}`；六条前置条件各带一种 `FallbackReason`（异常边/`jsr` 入口/≥3 后继/可重入/分支极性未解码/操作数不可渲染/两臂不相交/未覆盖块/块自身证据缺） | 不发文本、不解码字节码、不决定语法；证据不足**不产空 body** |
| ④ AST + emitter | `ast` / `build` / `emit` | 带 `OriginSet` 的语句与表达式、语句化规则（`Store`/`Invoke`/`Return` 成句，`Push`/`Load`/`Arithmetic`/`Comparison`/`Transfer` 不成句）、文本与段表**同一批写入**产生、每个写入口先查 `OutputBytes` 再写、超限丢弃缓冲 | 不做边发现/循环识别/异常范围推导 |

2. **段表载体（3.2 长在它上面）**：`source_map::{Origin, OriginSet, Segment, SourceMap}`。`Origin { bci, cp: Option<u16>, provenance: Direct|Derived }`；`Segment { start, end, origin }` 用**生成文本字节区间**做键；表按**完成顺序**记录（嵌套节点在内层先完成），因此 `covering(byte)` 给出最具体节点、`of_bci(bci)` 给出该 BCI 触到的全部节点（`direct_of_bci`/`derived_of_bci` 分开）。`Origin::cp` 本片恒为 `None`：1.1 载荷不发布 CP 索引，字段留出以免 3.2 改表形状。
3. **命名/转义/预算各在一层**：命名在 `names`（关键字/非法拼写→`alias_for` 纯函数别名，无 debug→`argN`/`localN`，冲突按槽序加后缀；别名只在**呈现**上生效，原始拼写留在证据里）；转义与预算都在 `emit`（字符串按 **UTF-16 单元**转义，含代理对 `😀`→`\ud83d\ude00`；注释走 `comment_text` 去掉可起 `\u` 转义的 `\` 与换行）。
4. **平面写入方式**（各自独立输入，互不派生）：任一区域或任一语句回退 → `Mixed`/`Fallback`；有别名 → `syntax_status=NotJava`（其余 `Unchecked`，本片无检查器，从不写 `Checked`）；`NotAttempted`/`Unproven`/`NotPerformed` 恒为本片取值。停止（预算/取消/缺表）不写任何平面为成功：`text`/`source_map` 为空 + `execution` 为 `Partial{BudgetExceeded}`/`Cancelled` + `RecoveryOutcome::Stopped`。

5. **本片发现的契约与代码缝（需要 1.3b 接上，本片不擅自扩契约）**：1.1 的只读载荷发布**结构**（块/边/BCI/frames/SSA/effects），但**不发布符号与操作数词汇**——常量池引用的 owner/name/descriptor、`ldc`/`*const*` 推的常量值、`*load*`/`*store*` 命名的是哪个槽、分支跳转的**极性**（`ifeq` 还是 `ifne`：两者图同构、极性的唯一来源是解码事实）都不在里面。要让呈现有内容，这些事实经 `RecoveryFacts` **由调用方交入**（按 BCI 键控），与 debug 名同一条缝；**把这些事实从 class 字节/CP 与真实解码派生出来，是驱动侧（1.3b）的工作**，本片的测试用一张只覆盖它能核对的 opcode 的小表来填（`operations_of`，未知 opcode → `Other` 而不是错分类）。`Origin::cp` 同理。**不**新增 backend trait、**不**给恢复侧可变访问，1.2 的边界未动。

6. **验证与证伪（本片实际执行）**：`cargo test --workspace --all-targets --all-features --locked --no-fail-fast` = **846 passed / 0 failed / 1 ignored**（基线 818 + 新 28：`jarde-java` 19 单元 + 9 集成）；`cargo fmt --all -- --check` 与 `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`openspec validate --all --strict --no-interactive` = **12 passed**；两个 CI example exit 0；分层门禁按 CI 口径本机复跑 12 个配置全绿（`jarde-reader`/`jarde-query`/`jarde-jvm` 的 normal/all × 有无 `--all-features` 都不含 `jarde-java`）；`fuzz/Cargo.lock` 无需更新（`cargo metadata --locked` 通过）。三组证伪（`/tmp` 副本 + 独立 `CARGO_TARGET_DIR`，用完删除）：① 段表区间整体 +1 字节 → 映射用例红；② 超限时改为「报成功」→ 预算用例红；③ 超限时保留半成品缓冲但仍返回停止理由 → 「停止不交半成品」用例红。

7. **遗留 1.3b**：循环/`switch` 与 header effect 次数；不可约/交叉异常区域的 fallback 细化；独立小图 oracle（分支极性/循环 header/return/异常优先级）；A09/A10/A13/A16 覆盖；库/CLI 一致性（CLI 入口本片未接）；上述事实缝的驱动侧派生；以及 1.2 记录的 `coverage=CompleteWithinSchema` 与 `CoverageState::CompleteWithinSchema` 的取值裁决。

#### 需要的裁决与发现的冲突（不在本片擅自改）

1. **决策 5 的前提在本生态不成立。** 决策 5 说「通用语法能力优先复用成熟 Rust 库……避免重复实现通用语法基础设施」，但 6 条准入里真正决定性的两条（③ origin、⑤ 预算）**没有任何现存库满足**，③ 更是所有候选发射 API 的形状问题（一次调用回 `String`）。本片按下述读法落地：**能复用的复用**（`petgraph` 复用图算法；classfile/ZIP 继续用既有成熟库；将来若宽度重排或标准 source map 序列化成为需求，`pretty` 族与 `sourcemap` 是现成候选），**不能复用的不假装能复用**（Java 呈现面自研，范围收在决策 2 的可证明子集）。若父级认为决策 5 应解释为「必须引入某个库、可以接受 origin 粒度下降」，那是**契约变更**（决策 3 的「source map 是一等输出、不得事后反推」与准入 ③ 都要改），需要 OpenSpec 修订，不能由 1.3 自行降级。
2. **spec 里的 coverage 取值名与现有类型不一致。** 三份 P3 delta 写的是 `coverage=CompleteWithinSchema`（`recovery-validation/spec.md:19`、`java8-recovery/spec.md:57`），而仓库里唯一的 coverage 平面是 `jarde-reader::CoverageState::{NotRequested, CompleteWithinSchema, Partial, Unknown}`——`CompleteWithinSchema` 这个拼写在源码与 spec 里都**不存在**（`CompleteWithinSchema` 有约 30 处使用）。1.1b 的产物词汇表没有覆盖这一项。需要在 1.3/3.2 前裁决：是给「恢复范围内的完整」新立一个类型/取值（与 P1/P2 的 schema-scope 语义区分），还是把 spec 句子改成 `CompleteWithinSchema`（并说明为何 schema 与 scope 在这里同义）。本片只记录，不改 spec。
