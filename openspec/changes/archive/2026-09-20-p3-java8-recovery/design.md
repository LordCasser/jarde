## Context

P0/P1、P2（29/29）和分层（7/7）均已归档，P2/分层归档提交为 `7a5f994`。当前进入 P3：`1.1/1.2/1.3/2.1/2.2` 已有交付记录（代码到 `492e31e`，收尾记录 `51a5cac`）；本轮新增基础值流修正 1.3d 后为 **5/12**。已有 Java 方法体、if/loop/switch、lambda 及部分拼接/bridge/accessor 呈现，仍受公开入口和语义边界约束；P4/P5 未实施。 当前状态见 [本轮复核](verification.md#review-2026-09-19-recovery)。本文下方各片“仍未完成”等描述保留历史时间点；本节与 Migration Plan 是继续实施的入口。

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

版本与源码取舍沿用 [P2 design §6.1](../2026-09-19-p2-jvm-ir/design.md)。`jarde-java` 已通过具体模块实现正常流视图、事实输入、Region、AST 与 emitter；没有动态 backend/pass 框架。`MethodIr` 已持有本次运行的 canonical/frame/ssa、解码、CP 与 bootstrap facts，`recover_method` 只调用一次分析，根门面只委托。剩余接缝是同次方法声明（flags/receiver/参数/debug）和按需 callee 证据的绑定，不再把任务写成“开放三张表”或“创建 jarde-java”。

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

P2 与分层已归档，1.1–1.3 与 2.1/2.2 已交付。先完成 **1.3d 值物化与 effect/fallback 修正 → 3.1 方法声明与作用域 → 3.2 跨方法 origin/按需成员交接**，再继续 2.3/2.4 的复杂模式与异常恢复，最后完成 3.3/3.4 的语料、重编译/行为对照和发布门禁。 2.2 已承认的底层 API 限制由 3.1/3.2 闭合；不把成员 Body 读取解释成仅补几个名字。

当前修正分为四个有独立验收的范围：

1. **1.3d：值与 effect。** `render_value` 必须表达 SSA 值被读取时的值，不能无条件把历史 load 变成当前 slot 名；`iinc`/覆盖后仍在栈上的旧值应物化为正确的已有值/必要临时量，证明不足则保留完整低级表示。普通调用是否省略语句，取决于最终消费者能否实际发射且保持次数/顺序，不能只按消费 opcode 名单推断；cast/field/分支等消费者 fallback 时必须连同其 effect 生产者保留。保留 `return tick()` 单次调用对照，不以重新引入重复调用来修漏失。
2. **3.1：声明与作用域。** 继续使用同次事实载荷，补方法 flags/receiver/参数槽/debug 范围；声明位置由定义/使用的作用域决定，不能用全方法一个 `declared` 集合决定 then/else 的局部可见性。普通 if/loop 的基本局部声明正确性先于 inner/enum/构造器复杂模式。方法体与完整 compilation unit 是不同交付范围，生产 compile_status 继续 NotAttempted。
3. **3.2：可定位的跨方法证据与公开入口。** 当前 `source_map::Origin` 只有 bci/cp/provenance，callee 的同数 BCI 不能冒充 caller 位置。复用既有物理方法/定义身份绑定每个 anchor，并保留 Direct/Derived 与 clone 来源。accessor 只读本次实际需要的 callee，由拥有 reader/预算的事实层交出并绑定物理定义/CP；普通方法无额外 Body，accessor 请求允许必要且有 reason 的 callee Body。不能要求调用方预装全类，更不能用同名 owner 代替内容身份。库/CLI 实际呈现后再断言 X1 两条原始边不变。
4. **3.3/3.4：独立验证。** 将本轮真实 javac 输入的返回值、调用次数、异常与作用域编译作为固定对照；只读文本形状的 oracle 不覆盖这些语义。确定性比较递归排除 elapsed_millis，其他字段不删。已有历史绿色与当前候选门禁分别记录。

这些修正扩展既有 build/names/source_map/MethodIr 接缝，不新建通用中端或并行 IR。2.3/2.4 的模式可以独立调查，但交付须消费已经满足这些条件的基础结构；P3 整体完成后 P4 再消费稳定恢复契约。

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

7. **遗留 1.3b**：循环/`switch` 与 header effect 次数；不可约/交叉异常区域的 fallback 细化；独立小图 oracle（分支极性/循环 header/return/异常优先级）；A09/A10/A13/A16 覆盖；库/CLI 一致性（CLI 入口本片未接）；上述事实缝的驱动侧派生；以及 1.2 记录的 `coverage=CompleteWithinSchema` 与 `CoverageState::CompleteWithinSchema` 的取值裁决。（**本节的 1.3c 段已接上**：CLI 入口与库/CLI 一致、A09/A10/A13/A16、模式声明类型均已落地；`coverage` 取值裁决仍未定。）

#### 需要的裁决与发现的冲突（不在本片擅自改）

1. **决策 5 的前提在本生态不成立。** 决策 5 说「通用语法能力优先复用成熟 Rust 库……避免重复实现通用语法基础设施」，但 6 条准入里真正决定性的两条（③ origin、⑤ 预算）**没有任何现存库满足**，③ 更是所有候选发射 API 的形状问题（一次调用回 `String`）。本片按下述读法落地：**能复用的复用**（`petgraph` 复用图算法；classfile/ZIP 继续用既有成熟库；将来若宽度重排或标准 source map 序列化成为需求，`pretty` 族与 `sourcemap` 是现成候选），**不能复用的不假装能复用**（Java 呈现面自研，范围收在决策 2 的可证明子集）。若父级认为决策 5 应解释为「必须引入某个库、可以接受 origin 粒度下降」，那是**契约变更**（决策 3 的「source map 是一等输出、不得事后反推」与准入 ③ 都要改），需要 OpenSpec 修订，不能由 1.3 自行降级。
2. **spec 里的 coverage 取值名与现有类型不一致。** 三份 P3 delta 写的是 `coverage=CompleteWithinSchema`（`recovery-validation/spec.md:19`、`java8-recovery/spec.md:57`），而仓库里唯一的 coverage 平面是 `jarde-reader::CoverageState::{NotRequested, CompleteWithinSchema, Partial, Unknown}`——`CompleteWithinSchema` 这个拼写在源码与 spec 里都**不存在**（`CompleteWithinSchema` 有约 30 处使用）。1.1b 的产物词汇表没有覆盖这一项。需要在 1.3/3.2 前裁决：是给「恢复范围内的完整」新立一个类型/取值（与 P1/P2 的 schema-scope 语义区分），还是把 spec 句子改成 `CompleteWithinSchema`（并说明为何 schema 与 scope 在这里同义）。本片只记录，不改 spec。

#### 1.3b 的实际落点：事实缝的闭合、循环与 switch、不可约/交叉异常、独立小图 oracle（2026-09-19）

本片的上级裁定只有一条，其余都是它的落地：**恢复层读的必须是同一次运行解出的解码事实，而不是调用方另建的一张表**。

**1. 事实缝怎么闭合的。** 1.3a 记录的那条缝（CP 引用的 owner/name/descriptor、常量值、`ldc`/`*const*`、`*load*`/`*store*` 的槽、分支极性都不在载荷里）按 1.1a 的既有做法闭合在**载荷**上：

- `MethodIr` 新增两个**按值**持有的字段：`code: Option<Box<MethodCodeFacts>>`（`raw_facts` 解出的指令、typed operands、声明的异常表）与 `constant_pool: Vec<CpEntryFacts>`（**同一次** header 读的常量池）。读面只有 `code()`/`constant_pool()` 两个只读借用，没有 `&mut`，也没有第二个构造路径（`new()` 是 `pub(crate)`）。
- 引擎在 run 末尾把它们 **move** 进载荷：`facts.map(Box::new)` 与 `declaration.pool`（同一次 header 读，frame/ssa 两个 pass 读的就是它）。**没有第二次解码、没有重读、没有计费变化**；`new()` 增加 `debug_assert!(canonical.is_none() || code.is_some())`——图是从解码建的，有图必有解码。
- `jarde-java` 新增 `decode::Operations::of(code, pool)`：这是 opcode → `Operation` 的**唯一**映射，按 **effective** opcode 分类（`wide` 是前缀不是指令），CP 引用由同一次读的池解析（`CpEntryKind::MethodRef`/`InterfaceMethodRef` 已带 owner/name/descriptor，`String`/`Integer`/`Long` 常量已带值）。未建模的 opcode → `Operation::Other`（被陈述的输入），解析不出来的引用同样 `Other`——不猜。
- `RecoveryFacts` **缩减**为载荷确实不携带的东西：方法身份（name/descriptor/parameters，请求点名的）与 debug 名（`LocalVariableTable`/`MethodParameters` 证据，驱动侧属性）。`operations` 字段与 `with_operation()`/`operation()`/`operations()` **全部删除**。
- **极性、槽号、常量值、switch keys 的唯一来源由签名证明**：`RecoveryRequest` 只有 `ir` 与收窄后的 `facts` 两个字段，调用方**没有任何参数**能再交出「另一张表」；一个 `ifne` 被标成 `ifeq` 这种事现在不可能由调用方造成，只可能来自解码本身（而那是被证伪覆盖的）。
- 同一解码顺带补上的事实面：`Operation::Comparison { op, target }`（`target` = BCI + `branch_offset`，是「哪一个是跳转目标、哪一个是 fall-through」的唯一来源）、`Operation::Switch { cases, default }`（`tableswitch` 的 low/high/offsets 与 `lookupswitch` 的 pairs，键配上绝对目标 BCI）、`Operation::Increment { slot, amount }`（`iinc`）。
- `CompareOp` 扩到 10 个变体，并按**操作数个数**分成三族：一个值对零的 `JumpIfZero/JumpIfNotZero`，一个引用对 null 的 `JumpIfNull/JumpIfNotNull`，**一个值对零的** `JumpIfNegative/JumpIfNotNegative/JumpIfPositive/JumpIfNotPositive`（`iflt/ifge/ifgt/ifle`），以及**两个值的** `JumpIfSame/JumpIfDifferent/JumpIfLess/JumpIfLessOrEqual/JumpIfGreater/JumpIfGreaterOrEqual`（`if_icmp*`/`if_acmp*`）。第三族与第四族的区分是**小图 oracle 在本片发现的生产 bug**：原来把 `iflt` 与 `if_icmplt` 合成一个变体，分支 arity 前置条件因此把一个合法循环判成 `UnrenderableOperand`。

**2. 循环与 switch 落在哪。** 正常流视图（①）新增 immediate dominators（入口为根）、`dominates()`、`natural_loops()`（回边 = target 支配 source；自然环 = header ∪ 能到达 latch 而不穿过 header 的块）与 `irreducible_blocks()`（**SCC 级**判据：一个含环的强连通分量可约，当且仅当它恰好有一个从分量外进入的节点且该节点支配整个分量——只有一条回边的环在不可约图里根本产生不了回边，所以这必须是分量级问题而不是回边级问题）。Region（③）新增 `Loop { header, test, test_bci, form: While|DoWhile, continuation: Taken|FallThrough, body, exit }` 与 `Switch { prefix, branch, branch_bci, groups: [{keys, default, arm}], join }`。可证明子集：

- **两种循环形状**：header 自己测（`while` 与 `for` 的 `goto test` 形状——test 块是回边目标，分支的 target 在环内、fall-through 在外面）与**唯一 latch 测**（`do … while`，含「整环就是一个自测块」的一元形状，此时块自己的语句就是循环体）。循环必须**单一入口**（环内非 header 块不得有环外前驱，否则 `Irreducible`/`LoopShape`）、**最多一次测试**（header 与 latch 各有一个分支即「两个测试」，不是本子集）。
- **`continuation` 是解码事实**：分支的 `target` 就是环内后继 → `Taken`（`goto test`/`do … while` 的形状，条件写分支自己的 sense）；否则 `FallThrough`（条件写 sense 的否定）。1.3a 的「更远的后继是 target」在这里恰好是反的，所以这条不变量不只是精度问题——没有它，`for` 形状的循环根本进不来。
- **switch 按终指令是否是解码出的 `switch` 分派**，不按后继个数：两个键共用一个 target 再加 default，在 graph 里只有**两个**后继。一个 target 一组（`case 0: case 1:` 的共享目标就是一组），target 是 join 的空臂 = `case K: break;`，default 指向 join 时不写 label（`switch` 自己落出去）；臂两两不得重叠（`SwitchArmsOverlap`——那是「一个 case 落进另一个 case 的代码」），键/目标与 graph 后继必须逐一对上（`SwitchShape`）。
- **effect 次数与次序不变量**：条件与 selector 写在 `while (…)`/`do { … } while (…)`/`switch (…)` 的**括号里**，它们的值表达式就是 test 块指令的文本 → 每次求值一次、次序与字节码一致；body 的语句留在花括号内，一条也不许提出来。两条守卫：(a) **循环的 test 块必须纯**（只有 `Push`/`Load`/`Arithmetic`）——`while (…)` 没有地方放 test 块的写，所以带写的 loop header → `TestBlockEffect` fallback 而不是搬走它（`a_loop_whose_header_writes_state_is_quoted_rather_than_hoisted`）〔**1.3c 起该取值改名为** `FallbackReason::UnmetPrecondition{pass: loop@1, requirement: StatementFree}`，码 `jre_region_unmet_precondition`，见 §1.3c.4〕；(b) 循环体在有 scope 的帧里走，任何从**分支**离开环的边（`break`、跳到环外的 arm）→ `LoopLeavesEarly` fallback，且环的每个块都必须被走到（`covers()`）否则 `LoopShape`。`if`/`switch` 的 test 块**效果**（例如条件里的赋值）则按次序写在结构**之前**——那里它只执行一次，位置正确。
- 循环内的 `Fallback` 形态不变：`Region::Fallback` + 逐条 `FallbackReason`（`LoopShape`/`LoopLeavesEarly`/`TestBlockEffect`/`Irreducible`）+ 诊断码，文本仍是 `// @bytecode …` 引用加原因，**不产空 body**。（1.3c 起 `TestBlockEffect` 更名为 `UnmetPrecondition`，其余三条取值未动。）

**3. 不可约与交叉异常的 fallback。** 两者都在走路之前判定，整具身体引用 bytecode（一个 `Region::Fallback` 覆盖全部 live 块，`blocks` 逐条列出）：

- **不可约 CFG**：`view.irreducible_blocks()` 非空 → `FallbackReason::Irreducible { blocks }`，诊断 `jre_region_irreducible`。理由写在消息里：环在 header 之外被进入，或两环交叉——没有 Java 结构能写这个形状。
- **交叉异常区域**：按**声明的异常表范围**判定「部分重叠、互不包含」（`[2,4)` 与 `[3,6)` 交叉；同一范围两条 catch 子句、嵌套范围都不算），并要求至少一条在 canonical 里有 handler row（否则没有块被保护，不因为一张表拒绝整具身体）→ `FallbackReason::CrossingExceptionRegions { record, other, blocks }`，诊断 `jre_region_crossing_exception_regions`，消息**按表序**点名两条记录（record 0 在前 = 抛点先到的那条 = primary）并列出它们保护的块——这正是「异常优先级在 fallback 里保留正确事实」的可核对形式。
- **平面**：两者都是 `representation=Mixed`、`quality=Fallback`、`execution=Complete`——**扫描完整**不是 `Partial`（`Partial` 只属扫描失败），`quality` 不被改写成 `Partial`；两者都**不**报 `Structured`、也不产空 body。

**4. 独立小图 oracle**（`crates/jarde-java/src/oracle.rs`，`#[cfg(test)] mod oracle`，不进生产构建）。

- **它是什么**：一个自写的**朴素字节码机**（自己的 opcode 解码器 + 局部变量/操作数栈机，未建模的 opcode 直接 panic）+ 一个自写的**文本模型**（自己的语句解析器与表达式求值器，只认识本子集写的形状）+ 一个比较器，要求两侧的 **calls 序列、`return` 值、test 求值次数**三项全等。
- **独立于什么**：两个 marker 之间的**模型段**不出现 `region::`/`Region`/`ast::`/`StmtKind`/`ExprKind`/`NormalFlowView`/`SourceMap`/`text_of_bci`/`build::`/`recover(`——由一个 `include_str!` 源码守卫断言，并**非空洞**地检查模型段确实含 `run_bytecode`/`run_text`/`compare`/`parse`。**不独立于**：fixture 的字节（共享 ground truth，正是对照的对象）与四个 opcode 的含义（oracle 自己解码，transcription 一错就红，不会静默）。
- **自造形状 ≥3**：`polarity_fixture(initial, sense)`（`ifeq` 与 `ifne` 两个 sense × 三个输入）、`while_fixture(initial)`（header 测、body 有调用与自减、`bipush` 返回）、`do_while_fixture(initial)`（latch 测、`ifgt` 回到 header）、`crossing_class()`（自造 class 字节，异常表 `[2,4)` 与 `[3,6)` 交叉、`athrow` 在 BCI 3）——都不用仓库既有 fixture。
- **有牙**：三个「削弱式」变异测试要求比较器**看见**问题——翻转条件的文本、把循环体 effect 提到循环外（或从循环里删掉）、改一个 `return` 字面量，各自必须返回 `Err`。比较器本身被削弱（丢掉 calls 比较）时，`the_oracle_rejects_a_loop_effect_moved_out_of_the_loop` 变红（证伪 ③），而生产用例仍绿——独立性因此不是声称。
- **异常优先级**：`primary_record(table, throw_bci)`（表序第一个覆盖者）由 oracle 从声明的表算出，再要求真实报告在 crossing fallback 里**先点名它**。
- **它抓到的生产 bug**：`iflt` 家族与 `if_icmp*` 家族的 arity 混淆（见上）——oracle 第一次跑就把它变成了红。

**5. 验证与证伪（本片实际执行）**：`cargo test --workspace --all-targets --all-features --locked --no-fail-fast` = **867 passed / 0 failed / 1 ignored**（1.3a 基线在 `git archive fc2d24b` 的独立副本上用**同一条命令**重测为 847，本片 **+20**：`jarde-java` 单元 20→34、集成 9→15，其余 crate 一字未动）；`cargo fmt --all -- --check` 与 `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`openspec validate --all --strict --no-interactive` = 12 passed；两个 CI example exit 0；分层门禁 `cargo tree -p jarde-jvm/jarde-reader/jarde-query` 均不含 `jarde-java`。四组证伪（`/tmp` 副本 + 独立 `CARGO_TARGET_DIR`，`shasum -a 256 -c` 逐文件确认只有目标文件变了，用完删除）：① 放行循环 test 块的 effect（region 去掉纯度前置 + build 把 test 块的语句写在循环**之前**）→ `a_loop_whose_header_writes_state_is_quoted_rather_than_hoisted` 红，失败输出正是「`local1 = local1 - 1;` 写在 `while (local1 != 0) {}` 之前」这一静默改次数；② `ifeq` 的极性反过来（`decode.rs` 一行）→ 极性/循环/集成 4 条红（含 oracle 的极性用例）；③ 削弱 oracle（比较器不再比 calls）→ `the_oracle_rejects_a_loop_effect_moved_out_of_the_loop` 红而生产用例全绿；④（附加）去掉不可约检查 → `a_graph_that_is_not_reducible_is_quoted_whole_with_its_own_reason` 红（整具身体的答案退化成 `ArmsDoNotMeet` + `UncoveredBlocks`）。
- **回归口径**：1.3a 的 29 条用例**逐条仍绿**，其中一条的行为按「读真事实」的后果**增强**而非放宽：`an_if_else_is_written_with_the_arm_the_branch_really_picks` 的 fixture 里，分支块在分支**之前**还有 `iconst_0; istore_1`（`local1 = 0`），旧实现把这条语句**静默丢掉**（分支块不被走），现在它按次序写在 `if` 之前——该用例原有的断言（`if (local1 != 0)`、fall-through 在 then、同一 BCI 两个 provenance）一条未改，另由 `a_loop_whose_header_writes_state_is_quoted_rather_than_hoisted` 固定「循环里不可搬走」的另一半。

**6. 留给 1.3c**：CLI 入口与「库/CLI 一致」、A09/A10/A13/A16 的验收覆盖、`RecoveryProfile`/模式前置条件/rule version/失败 fallback 类型（随模式 pass 走，见 1.3 交付项）、`ldc` 的 `float`/`double` 字面量（本片仍 `Other`）、`athrow`/`try`/`finally` 的呈现（2.4，本片只保证异常事实在 fallback 里正确）、字段/数组/转换类操作（2.x）、段表的 `cp` 面（3.2）。（**1.3c 段已交付前四项**：CLI/门面入口与逐字段一致、A09/A10/A13/A16、以及 profile/precondition/rule version/fallback 类型；仍留给后面的见 §9。）

#### 1.3c 的实际落点：门面与 CLI 入口、库/CLI 一致、A09/A10/A13/A16、以及 1.1 遗留的模式声明类型（2026-09-19）

本片把 1.3 的两半补完：**入口**（根门面 → CLI，一条请求一次分析）与**模式声明**（profile / rule version / 前置条件 / 失败 fallback），并给 A09/A10/A13/A16 逐条可指向的证据。

**1. 门面与 CLI 入口，以及「同一个 `MethodIr`」怎么拿到。** 分层链现为 `jarde → jarde-java → jarde-jvm + jarde-reader`：根 `Cargo.toml` 新增 `jarde-java` 依赖，root 是唯一可以（也应当）依赖恢复层的 crate；CI 的闭包门禁只管 `reader`/`query`/`jvm` 不反向依赖 `jarde-java`，本片未动该门禁，按 CI 口径复跑仍全绿。门面**收窄再导出**：跨过的只有恢复**请求**（`RecoveryRequest`/`RecoveryFacts`/`MethodFacts`/`recover`）、**报告**（`RecoveryReport`/`RecoveryOutcome`/`RegionRecord`/`SourceMap`）与报告里名得到的只读词汇（`RecoveryProfile`/`RuleVersion`/`Precondition`/`IrTable`/`StopReason`）；**没有**把 `jarde-java` 的模块整片 `pub use` —— `region`/`ast`/`build`/`emit`/`names`/`facts`/`decode`/`normal_flow` 一个都不在门面上，`Segment`/`Origin`/`OriginSet` 也留在下面（段表的读面已能回答「这段文本来自哪个 BCI」，3.2 需要命名它们时再定）。

入口是 `Engine::recover_method`（`src/facade.rs`）：它调用 `jarde_jvm::analyze_method_ir` **一次**，把该次运行的 report 与**同一份载荷**交给 `jarde_java::recover`，返回 `RecoveredMethod { analysis, recovery }`（两半都是同一次运行的）。因此 CLI 的 `recover_method` operation **只跑一次分析**：adapter 只绑定 content（`input_path` 打开的那个 snapshot），请求形状与既有 `analyze_method` 相同（`environment` + `method` + `stages`），响应是 `{"kind":"recover_method","analysis":…,"report":…}`（两个 `Box`，与 environment 同样的尺寸理由；`serde` 透明）。**恢复 profile 不是 operation 的独立字段**：门控读的就是 `environment.runtime.profile`，再加一个字段就是同一个事实的第二来源（也能让请求在它没声明的 profile 下呈现）。协议错误仍是 transport 级，**报告内停止仍是成功响应载荷**（用例 `a_recovery_request_that_asks_for_no_ssa_stops_inside_a_successful_response`：`status=ok`、`outcome.stopped.ir_table_missing.table="ssa"`、无文本无段表，且同一次运行的 analysis 半边如实写着 `execution=complete`——停的是呈现，不是这次运行）。

**2. `RecoveryProfile`：复用，不新立。** P3 规格句「在 Java 8 RuntimeProfile 下」点名的就是既有 `jarde_reader::view::RuntimeProfile`（经 `jarde_jvm::environment` 到门面），它已经带着门控要读的 `java_release`；再立 `RecoveryProfile { Java8, … }` 会是同一个事实的第二种拼写，两边第一次各自演进就会漂移。所以 `jarde_java::pass` 做的是 `pub use … as RecoveryProfile`（恢复侧自己的词汇），加**用法**：`Pass { rule, required_release, preconditions }`、`Pass::admits(&RecoveryProfile)`（`required_release: None` = 与 profile 无关；`Some(n)` = 需要 `java_release >= n` 的呈现），以及 `JAVA_8` 这个常量。**如实记录一处限制**：本片已注册的 4 条规则（`straight`/`if`/`loop`/`switch`）**全部** `required_release: None`（结构规则在哪个 release 下都是同样的 Java），所以 `admits` 的拒绝分支在 **2.x 的 Java 8 专属呈现规则**（lambda/拼接/TWR）接入前没有生产实例；本片用 `pass` 模块自己的合同用例覆盖谓词语义（release 8/9 接受、7 拒绝、结构规则恒接受），**不伪造一个 pass 去触发它**。

**3. rule version。** 每个模式一条 `RuleVersion { rule, version }`（`straight@1`/`if@1`/`loop@1`/`switch@1`），带 `Display`/`citation()`；`Region::rule()` 与 `Recovered::rules()` 把它变成报告里的数据：`RegionRecord.rule: Option<RuleVersion>`（谁产出了这个区域、或**哪条规则拒绝了它**）与 `RecoveryReport.rules`（本方法引用的规则，按首次出现去重）——「输出由哪条规则产生」由运行的记录回答，不用回读文本或源码。注册是**编译期常量表**（`PASSES`），没有 `trait Backend`、没有动态 pass 注册（P3 design 明确禁止）。

**4. 模式前置条件：类型 + 一条立刻用它门控的实例。** `Precondition` 按 P3 决策 1 的三族**类型化**：`IrTable(IrTable)`（IR）、`StatementFree`（effect）、`Metadata { attribute }`（metadata；本片**没有**任何规则要求 metadata，理由记在变体文档里——没有 debug 名是**命名决策**，不是被拒的理由，那是 A10）。**实例**是循环 pass：`LOOP` 声明 `StatementFree`，`region::test_is_pure` 在**打算认领循环**的那一刻检查它，失败即 `FallbackReason::unmet(&LOOP, StatementFree, block_bci, bci)`，运行因此**引用**（`// @bytecode …`）而不是把 header 里的写搬出循环（effect 次数与次序不变量）。`unmet` 里带 `debug_assert!(pass.requires(requirement))`：**声明与检查点漂移会在本 build 的测试里失败**，而不是静默地各走各路。**被取代的既有取值（逐条记录，原→新）**：`FallbackReason::TestBlockEffect { block_bci, bci }` → `FallbackReason::UnmetPrecondition { pass, requirement, block_bci, at }`，诊断码 `jre_region_test_block_effect` → `jre_region_unmet_precondition`；原因是 1.3b 的这条拒绝必须走决策 1 的**类型化前缀条件**路径（并带上 rule version），消息同时给出规则、要求与 BCI（`the loop@1 rule did not claim the block at BCI 2: it requires a test block whose every instruction is part of a value expression, and the instruction at BCI 5 is not part of one, …`）。**这是加强而非放宽**：既有用例除断言新码外，另断言 `RegionRecord.rule == Some(loop@1)`、消息含 `loop@1`/`value expression`/`BCI 5`、`report.rules` 含 `loop@1`、`execution == Complete`。其余 `FallbackReason` 取值（`ArmsDoNotMeet`、`UncoveredBlocks`、`LoopShape` 等）**本片未迁移**：它们是这条走查自己的形状陈述，迁移到声明式前置条件是 2.x/3.2 的事（`FallbackReason::pass()` 对它们返回 `None`，报告里因此不借任何规则的名）。

**5. 失败 fallback 走同一条可陈述路径。** 前置条件未满足**不停止**运行：它产出一个 `Region::Fallback` + 逐条 `FallbackReason` + 诊断（`Warning`，码 `jre_region_unmet_precondition`）+ `RegionRecord`（码/消息/规则）+ 平面（`representation=Mixed`、`quality=Fallback`、`execution=Complete`）。运行级缺表则**不是** fallback 而是停止（`StopReason::IrTableMissing` + `execution=Partial{Unsupported}` + `Error` 诊断 + 空产物）：没有任何图/名字/解码时连引用都写不出来，不能长得像「什么都没呈现的一次运行」。两者都是**值**（报告里的 reason + 诊断），都不抛异常、都不产空 body。

**6. 1.1 遗留的验证半条：未满足前置条件的 fixture。** 用例 `a_loop_whose_header_writes_state_is_quoted_rather_than_hoisted`（装配体：header 每轮写 `local1`，见 1.3b 的注释）断言输出是**被陈述的 fallback + 诊断**而不是「碰巧看起来对的结构」：文本里**没有** `while`、有 `// @bytecode`，记录与诊断都给出 `loop@1` + `StatementFree` + `BCI 5`，平面是 `Mixed`/`Fallback`/`Complete`。证伪 ②（见 §8）证明这条断言有牙齿。

**7. A09/A10/A13/A16 的逐条落点（含哪些半属后续阶段）。**

| 验收 | 用例（本片可指向的证据） | 断言要点 | 未覆盖的半 |
| --- | --- | --- | --- |
| A09 历史 `jsr`/`finally` 不误报成功 | `a_body_the_subset_cannot_prove_is_quoted_rather_than_emptied`（+ 既有 `crossing_exception_records_are_quoted_with_the_priority_the_table_states`） | 提交的 ECJ v45 `finallyPath` 走真实入口：`produced()` 但 `representation=Mixed`、`quality=Fallback`、`syntax_status=NotJava`、文本无 `try {`/`} finally`/`} catch` 而只有 `// @bytecode`（引用的 BCI 是该 body 自己的指令起点）、每个 fallback 的 `code` 是 `jre_region_*` 且消息非空、至少一条消息点出 BCI、`execution=Complete`（扫描跑完，不是 `Partial`） | **`finally`/`jsr` 的恢复本身属 2.4**（`try`/`finally` 呈现、primary/suppressed、资源关闭）。本片只覆盖「不误识别 + 正确降级」这一半，且如实如此声明 |
| A10 无 debug / 稳定命名 | `a_body_without_debug_names_is_named_deterministically_and_invents_no_source_scope`（+ 既有 `a_body_without_debug_metadata_gets_deterministic_names`、`a_keyword_name_takes_a_deterministic_alias_and_the_syntax_plane_says_so` 两例） | 无 LVT 的装配体：两次运行**整份报告逐字段相等**（含文本与段表）；名字是序号（`local1`/`local2`，无 `arg` 声明）；文本不含 `line`、不含 debug 侧的拼写；每个 `Segment` 的 `primary`/`derived` `cp` 均为 `None` 且至少一个 BCI；对照：同样字节**带** debug 名时文本用 `left`/`right` 且不再出现 `local1`，反之无证据的运行**不会**凭空出现 `left` | 语料级「多代 javac/ECJ × 无 debug × 混淆」矩阵与 slot/作用域完备属 **3.1/3.3** |
| A13 成员级失败隔离 + 平面分开 | `a_member_that_cannot_be_presented_leaves_the_member_that_can_alone`、`execution_quality_and_representation_each_state_their_own_thing`（+ `a_loop_whose_header_writes_state_is_quoted_rather_than_hoisted` 补 `execution=Complete`、A09 用例的 `Mixed`+`Complete` 组合） | 同一个类文件：`add(II)I` 是 `Java`/`Structured`/无 fallback，`finallyPath(I)I` 是 `Mixed`/`Fallback`/有 fallback；两者报告不互相携带对方的文本/诊断/BCI，且**再问一次**可呈现成员得到逐字段相同的报告（隔离，无跨请求状态）。平面：同一 body 在「足额预算」「差一字节」两种预算下，`representation`/`quality` 说的是产物（`Java`/`Structured` ↔ 停止时 `Bytecode`/`Fallback`），`execution` 说的是运行（`Complete` ↔ `Partial{BudgetExceeded}`），`quality` **从不**被改写成执行值（`Partial` 不是 quality），停止的诊断是停止那次自己的；反面同样断言：不可约 body 是 `Mixed`/`Fallback` 但 `execution=Complete`（「完整扫描下的 Fallback」组合），产出的报告里没有停止运行那条预算诊断 | 语料级成员隔离的完整矩阵与 coverage 诊断属 **3.2**（`completeWithinSchema` 的取值裁决也仍在 1.2 记录里待定） |
| A16 单方法闭包 | `one_recovery_request_reads_one_body_and_presents_that_member`（root，`tests/p3_recovery_entry.rs`）+ `recovery_matches_the_library_entry_field_by_field` 里的 wire 断言（CLI） | 前提先读：提交的 ECJ v52 fixture 经 `inspect_header` 断言「≥3 个成员带 `Code`」；随后 `Engine::recover_method` 一次请求：`class_headers == 1`、`method_bodies == 1`、`code_bytes > 0`、`reads` 恰一条且理由是 `DriverMethodBody`、`ir_items/ir_edges > 0`；呈现是那个成员自己的（`method == "add(II)I"`、文本是该 body、`rules == [straight@1]`、`outcome=Produced`）；CLI 侧同一请求的响应里 `analysis.execution.usage` 同样 `class_headers == 1`/`method_bodies == 1` | 「Region/AST 构造计数为零」类维度在 P2 侧仍不存在（5.2 已记录），本片只以「一次 header + 一次 body + 一条 read + 入口只调一次分析」作代理，不宣称有逐 pass 计数器 |

**8. 验证与证伪（本片实际执行）。** `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` = **877 passed / 0 failed / 1 ignored**（1.3b 基线 867，+10 = `jarde-java` 单元 34→37（`pass` 三条合同用例）+ 集成 15→18（A10、A13 ×2；A09 与循环前置条件用例为**加强**）+ root 新文件 `tests/p3_recovery_entry.rs` 2 + CLI json 11→13）；`cargo test -p jarde-cli --locked` = **24 passed**（基线 22，+2）；`cargo fmt --all -- --check` 与 `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`（1.98.1）干净；`openspec validate --all --strict --no-interactive` = 12 passed；两个 CI example exit 0；分层按 CI 口径：`cargo tree -p jarde-reader/jarde-query/jarde-jvm` 不含 `jarde-java`，门面含（本片新增）。`Cargo.lock` **只多一行**（`jarde-java` 的依赖边新增 `serde`；`serde` 本就在 workspace 与 lock 里，无新第三方包）。**证伪两组**（`/tmp` 副本 + 独立 `CARGO_TARGET_DIR`，用完删除）：① 让 CLI 的恢复响应改写一个字段（`report.quality` 改成 `Fallback`）→ `recovery_matches_the_library_entry_field_by_field` 必须红；② 让 `test_is_pure` 在不满足时仍返回 `Ok(())`（即前置条件未满足也照旧产出结构）→ 未满足前置条件用例必须红。

**9. 留给后面的（如实）。** 2.x：Java 8 专属规则（第一批带 `required_release: Some(8)` 的 pass，`admits` 的拒绝分支届时第一次有生产实例）、其余 `FallbackReason` 迁移到声明式前置条件；2.4：`try`/`finally`/TWR/`jsr` 的恢复本身（A09 的另一半）；3.1：参数槽/receiver 与 debug 名（入口现在**不猜** static 与否，`RecoveryFacts.parameters` 收在 0，故入口侧一律用 `localN` 序号名——库层接口允许调用方按自己的声明事实给出计数，`jarde-java` 的用例即如此）；3.1/3.3：无 debug/混淆语料矩阵；3.2：段表 `cp` 面与 `Mixed` 区域映射、coverage 取值裁决；`RecoveryReport` 的 `Deserialize` 面（码字段现为 `&'static str`，wire 文档是文本）留给 3.2/P4 决定——本片的库/CLI 一致以**整份序列化文档逐字段相等**为证据。

#### 2.1 的实际落点：已验证的 LambdaMetafactory 形态（2026-09-19）

本片的上级裁定只有一条：**验证所需的 bootstrap 表必须和 `code`/`constant_pool` 走同一道缝**（载荷、按值、同一次 header read），否则「这个 `invokedynamic` 到底是不是 lambda」只能由调用方喂进来的第二张表回答——而那正是 1.3b 删掉的东西，也正是 A04 最容易被写松的地方。

**1. 载荷补齐（跨层，事前已确认）。** `MethodIr` 新增 `bootstrap_methods: Vec<BootstrapMethodFacts>`（只读访问器，空表 = 该类没声明 `BootstrapMethods`）。读取发生在 `read_declaration` **同一次** header read 里：属性壳由这次枚举定位、字节是这次持有的、解析用的池就是这次请求留下的那份；**无该属性的类读零字节、计零费**（本片没有改动任何既有计费路径，也没有重解码/重跑）。用的是 `jarde-reader` 已公开、`jarde-query` xref 已在用的 `classfile::bootstrap_methods`，**reader 语义一字未改、无新依赖、`Cargo.lock` 未动**。一处**如实的语义后果**：那份函数会校验「句柄与每个实参都是可加载常量」（JVMS 4.7.23），读取失败因此按该 pass 的既有约定**上抛**——格式自相矛盾的类在这条 pass 上失败，而不是被静默当成「没有 bootstrap 表」。

**2. 已验证形态：逐条判定。** 新模块 `jarde-java/src/lambda.rs`（片内私有面 + 报告记录类型），判定链全部来自**同一次运行**的载荷：

| 步 | 判定 | 不满足时 |
| --- | --- | --- |
| 1 | 本次 profile 准入 `lambda@1`（`Pass::admits`，输出是 Java 8 构造） | `jre_lambda_rule_not_admitted` |
| 2 | 类自己的表里**存在**站点 `bootstrap_method_attr_index` 那一条 | `jre_lambda_no_bootstrap`（声明式要求 `IrTable(BootstrapMethods)`） |
| 3 | 条目句柄是方法句柄，且其成员是 `java/lang/invoke/LambdaMetafactory.{metafactory,altMetafactory}`，句柄 kind 为静态 | `jre_lambda_bootstrap` |
| 4 | 静态实参按**种类与个数**是 `[MethodType, MethodHandle, MethodType]`；`altMetafactory` 的第四参是**标志字 0** | `jre_lambda_bootstrap_arguments` |
| 5 | 站点描述符、`samMethodType`、`instantiatedMethodType` 都可读；后两者参数个数一致；站点参数个数 == 该指令真正读到的捕获数 | `jre_lambda_descriptor` / `jre_lambda_sam_descriptor` |
| 6 | 实现句柄是调用或构造器（kind 5/6/7/8/9），字段句柄不呈现 | `jre_lambda_implementation` |
| 7 | **arity 自洽**：`捕获数 + SAM 参数数 == 实现参数数 + receiver`，且逐位对齐的**描述符形状**相同（引用对引用，基本类型同字母） | `jre_lambda_sam_arity` / `jre_lambda_sam_types` |
| 8 | 每个捕获值的文本可以在新位置**重读**（字面量；或本方法只写一次的局部量），否则会把调用写成两次或写晚 | `jre_lambda_capture_not_replayable`（声明式要求 `Replayable`） |
| 9 | 站点产出的实例**有**读者，否则那条调用在产物里无处安放 | `jre_lambda_unconsumed` |

**没做、也没宣称**：适配（子类型/装箱/加宽）。实现参数只要求**可适配**到 `instantiatedMethodType`，本层只比较引用/基本类型的形状而不比较引用**名字**，需要更多适配的站点一律拒绝，不写一条没人写过的转换。

**3. 规则版本与前置条件。** `pass.rs` 新增 `LAMBDA`：`lambda@1`，`required_release: Some(8)`——这是本 build **第一条**带 `Some(8)` 的规则，因此 1.3c 记录的「`admits` 拒绝分支无生产实例」在 2.1 关闭（用例 `a_profile_that_does_not_present_java_8_is_refused_the_lambda_rule` 用 Java 7 profile 走真实门控）。前置条件三族各有实例：`IrTable{Ssa, Code, ConstantPool, BootstrapMethods}`、新增的**效果族** `Precondition::Replayable`（捕获值必须可重读）。`IrTable` 的文档相应改成分两种检查点：整趟没有它就什么都呈现不了的表在走查前检查（canonical/frames/ssa/code），**只有部分规则需要的表在「规则要认领形状」处检查**——一个不含 bootstrap 表的类是普通的类，不该整趟停摆；站点缺席因此是**被拒绝的站点**而不是被拒绝的运行。拒绝按既有 `Refusal::unmet(pass, requirement, …)` 构造，带同一个 `debug_assert!(pass.requires(...))` 防声明/检查漂移；形状性的拒绝（不是 lambda）不带 requirement，但记录里同样点名 `lambda@1`——「谁产出/谁拒绝」两种都读得回来。

**4. 呈现：lambda 与 method reference 分开写。** 两种写法语义相同，选哪一种由**实现句柄是不是这次站点自己的 body**决定，而这由事实决定、不由名字猜形状：

- 句柄名字带编译器为 lambda 生成的 body 标记（`lambda$…`，javac/ECJ 同一个约定）→ 写 **lambda**：`(params) -> Impl(captures…, params…)`（静态句柄带 `Owner.` 限定，实例句柄以第一个绑定值为接收者，构造器句柄写 `new Owner(...)`）；
- 否则当**捕获恰好等于句柄自己要的接收者**（实例 1 个、静态/构造器 0 个）→ 写 **method reference**：`receiver::name` / `Type::name` / `Type::new`（Java 的 `::` 没有「绑定实参」写法，这也正是它能被写的唯一情形）；
- 否则 → 仍写 lambda（具名成员带绑定实参时 `::` 表达不了）。

标记只决定**两种等价写法中的哪一种**，绝不决定「是不是 lambda」——后者只由第 2 节那条链决定；一个凑巧叫 `lambda$foo` 的普通成员得到的是「对它的一次调用的 lambda」，仍是同一个函数。参数**个数与顺序**取自 SAM 的 `instantiatedMethodType`，写出来的类型是它自己的（erased 的 `Object` 会丢掉类真正说过的话），参数名由 `names::free_name` 派生并保证不与任何局部量或别的 lambda 参数重名（JLS 6.4）。

**5. evidence 怎么读回来。** 两条一起：

- **报告记录** `RecoveryReport.lambdas: Vec<LambdaRecord>`（BCI 序，presented 与 refused **都**在）：`use_site`（该 `invokedynamic` 的 BCI）、`site_cp`（站点自己那条池项）、`bootstrap_index`、`bootstrap`（`owner.name (REF_kind)`）、`bootstrap_arguments`、`sam_name`/`sam_descriptor`、`sam_method_type`/`instantiated_method_type`、`implementation`（`owner.name(desc) (REF_kind)`）、`captures`（按站点读取顺序，每项带它来自的 BCI）、`form`、`refusal{code, rule, requirement, message}`；拒绝另有 `jre_lambda_*` 诊断，presented 有 `jre_lambda_sites` 汇总诊断；`RecoveryReport.rules` 在有站点时含 `lambda@1`（一条站点都没有的 body 不借这条规则的名）。
- **段表（既有 provenance）**：lambda 节点的 **primary** 是站点 BCI 且 `cp = Some(site_cp)`（站点自己的池项），每个捕获值作为 **derived** 锚点挂上去；捕获表达式自己的文本仍锚在它被产出处的 BCI（`direct`）。一个 lambda 表达式因此对应多个原始 BCI，且**不是**只留一个。节点内部不复制捕获值的节点（接收者类型名、参数）只锚在站点，避免「声称呈现了其实没呈现的锚点」。

**6. 独立对照与它的边界。** `oracle.rs` 扩成：自写 class 读取（池 + `BootstrapMethods`，未知 tag 直接 panic）→ 自写机器执行 fixture 字节（栈上带**值来源**，于是「站点按什么顺序、从哪些槽捕获」是它自己读出来的）→ 自写文本解析（lambda / `::`）→ 比较器。它要求：写法与「实现名字带不带 body 标记」一致、lambda 的参数个数与**类型拼写**等于它自己读出的 SAM 方法类型、body 调用的名字是实现名、**前 N 个实参就是它自己读出的那些槽名、顺序一致**，后接 lambda 自己的参数；槽名由「fixture 的 store 顺序」与「文本的赋值顺序」配对得出，**不依赖生产的命名规则**。它另与既有的 calls/return/`tests` 比较连通（fixture 结尾真调一次 `run(0L)`，于是「调用次数」两侧可对照）。**边界如实**：模型**不执行** lambda 体（那是别的方法的代码），它检查的是创建、调用次数与**形状**；`Src` 之外的捕获值一律报错而不是放过。

**7. 验证与证伪（本片实际执行）。** `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` = **894 passed / 0 failed / 1 ignored**（1.3c 基线 877，+17 = `jarde-java` 单元 37→43（`lambda` 4 + oracle 2）+ 集成 18→29（2.1 用例 11），其余 crate 一字未动）；`cargo fmt --all -- --check` 与 `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`（1.98.1）干净；`openspec validate --all --strict --no-interactive` = 12 passed；两个 CI example exit 0；分层 `cargo tree -p jarde-reader/jarde-query/jarde-jvm` 中 `jarde-java` 出现 **0** 次；`cargo metadata --manifest-path fuzz/Cargo.toml --locked` 通过、`cargo deny --manifest-path fuzz/Cargo.toml --workspace --locked --config deny.toml check`（在仓库根执行）四类全 ok（**未触及任何依赖边**，故预期锁文件不动，实测不动）。**三组证伪**（`/tmp` 副本 + 独立 `CARGO_TARGET_DIR`，`sha256sum -c` 逐个确认仓库侧未被改动，用完删除）：① 把形态判定的工厂检查改成恒真（`if false`）→ **只有** `an_arbitrary_bootstrap_is_never_presented_as_a_lambda` 红，输出里那条站点被写成了 `java.lang.Runnable local2 = () -> Test.lambda$method$0(local1);`（其余 27+43 全绿）；② 让 build 把捕获实参顺序倒过来 → 集成用例 `two_captures_are_written_in_the_order_the_site_reads_them` 红（产物正是 `Test.lambda$method$0(local2, local1)`）且 oracle 正例与其变异用例同时红；③ 削弱 oracle（顺序比较改成只比个数）→ `the_oracle_rejects_a_reordered_or_shortened_capture_list` 红而**生产 29/29 全绿**。

**8. 被修正的既有断言（逐条，原→新→原因，无放宽）。**

| 位置 | 原 | 新 | 原因 |
| --- | --- | --- | --- |
| `pass.rs::the_registered_table_states_each_rule_and_its_preconditions` | `PASSES.len() == 4`、规则表 `[…switch@1]` | `== 5`、含 `lambda@1` | 注册了第五条规则 |
| 同上 | `for pass in PASSES { assert_eq!(pass.required_release(), None) }`（注释写着「2.x 才会有 Some(8)」） | 逐规则 pin：`lambda` 为 `Some(8)`，其余为 `None` | 2.1 就是那条 Java 8 规则；断言由「谁都没有」**收紧**为「谁有、谁没有」 |
| 同上 | `IrTable` 前置条件只允许 `Canonical│Ssa│Code` | 允许本 build 存在的五个表，并另断言 `LAMBDA` **确实**声明 `BootstrapMethods`+`ConstantPool`、四条结构规则**都不**声明 `BootstrapMethods` | 表从 3 个增到 5 个，且新表按设计在站点处检查而非整趟；新增的两条断言是收紧 |
| 同上 | `assert_eq!(pass("lambda"), None)` | `Some(LAMBDA)` | 该规则现已注册 |
| `pass.rs` 的 `Precondition::IrTable` 文档 | 「一次性在走查前检查，缺表即停」 | 分两种检查点（整趟必需 vs 规则认领时） | 与实现一致；`BootstrapMethods` 缺席是**站点**被拒，不是运行停摆 |
| `ast.rs::Type` 文档 | 「`boolean` 故意缺席：字节码分不清」 | 四族（`boolean/byte/char/short`）由**描述符**读者产出、帧读者仍不产出 | 描述符确实写着 `Z`，那是事实不是猜测；帧侧不变 |
| `build.rs::value_type` | 对帧里的引用名只做 `/`→`.` | 剥掉描述符外框 `L…;` 再转名 | 帧把引用存成**描述符形式**（`Ljava/lang/Runnable;`，`frame.rs` 自己的 `parse_field_type`），此前产出的声明类型会是 `Ljava.lang.Runnable;`；本片首次真正走到这条路径。数组描述符仍按描述符拼写，属 3.x（已在代码注释里写明） |
| `oracle.rs::execute` 的 `Text::Return(None)` | 记为 `Some(0)` | 记为 `None` | 裸 `return` 不返回值；字节码侧模型的同一事实是 `None`，两侧必须一致。既有 fixture 全是 `ireturn`，该分支此前从未被走到 |

**9. 留给后面的（如实）。** 3.1/3.2：段表 `cp` 面的其余填充与 `LambdaRecord` 的 `Deserialize` 面（码字段现为 `&'static str`）；3.3：语料级 lambda 矩阵（多代 javac/ECJ、缺 debug、混淆）与受控重编译，以及「适配」类站点（子类型/装箱/加宽）——本片对它们一律拒绝并已写明；2.2/2.3：concat/accessor/bridge、inner/enum 等其余 Java 8 模式；`altMetafactory` 的非零标志位（markers/bridges/serializable）需要各自的呈现与 bridge 证据，属后续阶段。

#### 2.2 的实际落点：拼接链、bridge 与 synthetic accessor（2026-09-19）

本片把三条 Java 8 模式规则注册进同一张 `PASSES` 表（`straight@1`/`if@1`/`loop@1`/`switch@1`/`lambda@1` 之后新增 `concat@1`/`bridge@1`/`accessor@1`），并沿用 2.1 的 `Precondition`/`RuleVersion`/`Refusal` 机制 —— **没有第二套**：`Refusal` 从 `lambda.rs` 提到共享模块 `refusal.rs`，诊断码按 `(rule, requirement)` 查表（`requirement_code`），规则一侧只声明自己有哪些 requirement。

**1. `concat@1`：接收者类别、重载语义与求值顺序。** 已验证形态是 `new C ─ dup ─ invokespecial C.<init>()V ─ [操作数生产者] append(X)C … ─ toString()Ljava/lang/String;`，全部读自同一次运行的解码与值流：

- **接收者类别是判定的一部分**，且只接受两个名字（`CONCAT_CLASSES`）：`java/lang/StringBuilder`（javac 5 起）与 `java/lang/StringBuffer`（更早的拼写，源里仍可指名）。其它类上的同形调用链不被本规则认领。
- **`append` 逐重载检查转换语义**（`keeps_its_conversion`）：`int/long/float/double/boolean/String/Object` 这些重载写出的文本与 `+` 相同；`char`（`+` 会按数值相加）、`char[]`（`+` 写数组的 `toString`）、`CharSequence`（`append` 写字符序列而 `+` 走 `toString`）、`StringBuffer`（Java 5 重载）等一律**拒绝整条链**——转换不可丢。
- **求值顺序是三条可证的守卫**：(a) 链的整段 BCI（从 `new` 到 `toString`）由链**拥有**，`build` 对这些 BCI 不产任何语句，操作数的文本只在表达式里出现一次；(b) 每个操作数的产生 BCI 必须落在「上一条链指令」与「它自己的 `append`」之间，于是写出的表达式从左到右的求值与字节码同序；(c) 段内不得有产出语句的指令，否则按 `Precondition::StatementFree` 拒绝（把语句写进表达式会改变次数或次序）。
- **拒绝即引用**：链被分支切开（`jre_concat_split`，同类的 `toString` 在另一个 block）、实例被存入局部量（`jre_concat_interleaved_effect`）、单个 `append`（`+` 写不出「一个值就是拼接结果」）、重载转换不可证（`jre_concat_shape`）、`toString` 的结果无人读（`jre_concat_unconsumed`）。`new`/`dup`/`getfield`/`checkcast` 等指令在 `decode` 里建模，但**只有规则认领后才呈现**：未被认领的仍是按 BCI 逐条引用的 fallback。
- **段表**：一条链是一个节点、多份原始 BCI —— primary 是 `toString` 的 BCI，derived 保留链的**每一个** BCI（`new`/`dup`/`<init>`/每个 `append`/每个操作数生产者），两侧都留在表里。

**2. `bridge@1`：先读声明，再读消去证明。** bridge 身份来自**声明的 `access_flags`**（`MethodFacts::is_bridge`，本片为此给 `MethodFacts` 增加可选 flags，由调用方从同一个类的头读出），不由 body 形状推断。形状要求 body 的每一条指令都是「读数槽 / 一次转发调用 / 对被转发返回值的 `checkcast` / return」：

- **`checkcast` 是消去而不是检查**，其证明只依赖一处池事实：被 cast 的值正是转发调用返回的值，且 cast 的类型正是**该调用描述符声明的返回类型**（`invocation_returns_the_cast`）。Java 的声明类型保证值要么是 null 要么是该类型，因此该 cast 既不会失败也不会改变值，去掉它写的是同一件事。
- **参数上的 cast 一律拒绝**（`jre_bridge_cast_not_erasure`）：被转发调用的实参是擦除签名给的宽类型，把它 cast 到窄类型是**会失败**的检查。这也覆盖了「cast 出现在 loads 与 invoke 之间」的形态。
- 其它失败：body 不只转发（`jre_bridge_shape`，例如多一次 `astore`）、类没有把该成员声明为 bridge（`jre_bridge_not_declared`，于是 cast 照旧引用）、运行没有交出 flags（`jre_bridge_flags_missing`，走 `Precondition::Metadata{access_flags}` 的 `unmet` 路径）。**拒绝 bridge 形状不等于拒绝成员**：body 仍按通用结构呈现（多出来的那条语句照样写成语句）。presented 的 bridge 由通用路径写 `return x.m(args)`，本规则只拥有那个被证明是消去的 cast BCI（该 BCI 仍作为 derived 锚点留在段表里）。

**3. `accessor@1`：callee 自己的 body 是唯一证据（A12）。** `access$NNN` 这个名字只决定「这个调用点值不值得留一条 refusal 记录」，绝不决定它是不是访问器（与 `lambda$` 标记同一条纪律）。判定读的是成员表里那个成员的**声明与解码后的 body**：

- 声明必须 `static` 且 `synthetic`（`jre_accessor_declaration` 拒绝源码里能写出来的成员）；
- body 必须恰好是**一次实例字段访问加它前后的读取与 return**：读形态 `load 0; getfield C.f:D; return`（描述符 `(LC;)D`），写形态 `load 0; load 1; putfield C.f:D; return`（描述符 `(LC;D)V`），字段的类必须是本类（`jre_accessor_field`），字段类型必须与描述符一致；
- 呈现：读访问器 → `x.f`（`ExprKind::Field`），写访问器 → `x.f = v;`（`StmtKind::FieldAssign`）；调用点自身 BCI 是 primary，**访问器 body 里那条字段指令的 BCI 是 derived** —— 一次派生呈现带两个原始 BCI（`accessor@` 记录的 `field` 同时给出 owner/name/descriptor 证据）；
- 失败：body 做别的事（`jre_accessor_body`）、结果无人读（`jre_accessor_unconsumed`）、调用点实参渲染不出（`jre_accessor_arguments`）、运行没交成员表（`jre_accessor_members_missing`，`IrTable::Members` 的 `unmet`）。方法转发型 accessor（`invokestatic access$200(x)` 转发到别的方法）**不呈现**：本片只呈现字段访问，转发形态保留原来的调用（如实记录，见 §4）。

**4. 成员表这条缝（如实记录，供后续复核）。** synthetic accessor 是**同一个类的另一个成员**，一次方法的载荷装不下它。所以本片给 `RecoveryRequest` 加了一个可选的 `ClassMembers`（`owner` + 每个成员的声明与 `MethodCodeFacts`），由**读过同一个类头的那一方**交出；含义判断全在 `accessor.rs`，调用方交不出「这是个 getter」这种说法，只能交字节。**这与 1.3b 的载荷纪律不冲突**：body 的事实仍只来自同一次运行（`accessor.rs` 用呈现那次运行的池解引用成员 body）。**根门面本片没有接这条缝**：`Engine::recover_method` 的 `recovery_facts` 明说不做第二次类读，因此经门面的运行给出 `jre_accessor_members_missing` 这一条**被陈述**的拒绝（库层用例覆盖呈现，门面用例覆盖「拒绝不改 X1」）。

**5. A12 的双 origin 边在哪里断言、为什么在那里。** `jarde-java` **不依赖** `jarde-query`，所以 X1 的断言放在**根 crate 的用例** `tests/p3_accessor_edges.rs`：同一个装配 fixture，一次**不走恢复**、一次**走恢复**并逐字段比较两次 `Engine::query` 的输出（递归把 `elapsed_millis` 归零后比较整份序列化文档），再**分别**断言两条边各自存在 —— ①调用方 `method` 的 call site BCI → accessor 符号，②accessor 自己的 `getfield` BCI → 字段符号（两条 item 互不相等，因此不是「总数不变」的弱断言）。同一条用例另断言门面那次恢复**拒绝**呈现（缺成员表）且仍在文本里保留 `access$100(`：呈现是派生的，X1 一行未动。

**6. 独立 oracle 与它的边界。** 拼接与 accessor 的独立对照写在 `crates/jarde-java/tests/p3_patterns.rs`（与 `oracle.rs` 同一做法：自写模型 + `include_str!` 源码守卫），两个 marker 之间的**模型段**自带：自写常量池解析（未知 tag 直接 panic）、自写栈机（`new/dup/<init>/ldc/iload/invokestatic/invokevirtual/checkcast/return`，未建模 opcode 直接 panic）、自写文本解析与求值，比较器要求**调用序列、返回值、访问器调用次数**三项逐条相等（访问器在字节码侧是 1 次调用、在文本侧必须是 1 次字段访问且**0 次调用**）。模型自己声明 fixture 的事实（字段值、被调成员返回值），不读生产侧任何东西。**边界**：模型不知道 `this` 写不写限定、不比较静态调用的 owner 拼写、也不执行别的方法的 body。

**7. 验证与证伪（本片实际执行）。** `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` = **925 passed / 0 failed / 1 ignored**（2.1 基线 894，**+31** = `jarde-java` 单元 43→49（concat 2、bridge 1、accessor 1、refusal 2）+ 新集成文件 `crates/jarde-java/tests/p3_patterns.rs` 24 + 新根用例 `tests/p3_accessor_edges.rs` 1；其余 crate 与既有用例一字未改）；`cargo fmt --all -- --check` 与 `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`（1.98.1）干净；两个 CI example 按 CI 口径（带 `HistoricalControlFlow.class` 参数）exit 0；分层 `cargo tree -p jarde-reader/jarde-query/jarde-jvm` 中 `jarde-java` 出现 **0** 次；`cargo metadata --manifest-path fuzz/Cargo.toml --locked` 通过、`cargo deny --manifest-path fuzz/Cargo.toml --workspace --locked --config deny.toml check` 四类全 ok（**未触及任何依赖边与 workspace 成员，`Cargo.toml` 一字未改**）。**三组证伪**（`/tmp` 副本 + 独立 `CARGO_TARGET_DIR`，`sha256sum -c` 确认仓库侧 22 个源文件逐字节未变，用完删除）：① 让段内语句检查改为「一律接受」→ `a_chain_whose_instance_is_stored_in_a_local_is_refused` 红（实例被存进局部量的链被当成 `+` 写出来）；② 把 `append` 的操作数顺序倒过来 → 顺序用例红且 oracle 正例同时红（产物是 `return f() + 5 + "x";`）；③ 削弱 oracle（比较器不再比调用序列）→ `the_oracle_rejects_a_concatenation_whose_calls_are_written_in_another_order` 红，而同文件**其余 23 条（含全部生产用例）全绿**。

**8. 历史判断已被后续测量修正。** `bc283c0` 的普通 `return tick()` 确实会写两次调用，但 `492e31e` 已加入消费者守卫，本轮同一公开入口反例复测只调用一次。原“2.2 仍重复调用”的说法不适用于交付代码。新缺口是跨写入的旧值重读，以及消费者 fallback 时丢失生产者 effect；已按本轮 P3-R1/R2 列入 1.3d，不能以固定重复次数代替求值所有权。

**9. 留给后面的（如实）。** 3.1：参数槽/receiver 与 debug 名（本片 fixture 里 `this` 会被名字层别名成 `self_` 一类，因为 `this` 是关键字；门面仍不猜 receiver）；3.2：段表 `cp` 面、`ConcatRecord`/`AccessorRecord`/`BridgeRecord` 的 `Deserialize` 面；3.3：语料级 concat/accessor/bridge 矩阵（多代 javac/ECJ、缺 debug、混淆）与受控重编译（`altMetafactory` 的非零标志位、方法转发型 accessor 的呈现、内联 `checkcast` 之外的转换类站点都在那里重新评估）。门面接成员表（`ClassMembers`）是 3.1 的名字读与成员读一起做，而不是本片临时加第二次类读。

#### 2.3 的实际落点：构造、字段访问、dispatch 表与成员声明（2026-09-19）

本片有四件事：先判上一条的 §0，再做 inner/local/anonymous 的**使用点**、enum `switch`、interface 的 `default`/`static` 方法、构造器与字段初始化。新增四条规则（`new@1`/`field@1`/`enumswitch@1`/`init@1`）与一条只写进 envelope 的规则（`declaration@1`），`PASSES` 8→13；机制**沿用** 2.1/2.2：`Refusal::unmet` + 声明式 `Precondition` + `RuleVersion` + `presented/refused` 都记录的 `*Record`，**没有第二套**。

**1. §0 的判定：父亲的怀疑为真，已在片内修掉。** 旧实现的 `build::call_value_reaches_a_reader` 把 `CheckCast`/`Field` 也算作读者，于是「值被消费的调用」不写自己的语句，而那个读者若**被引用**（没有规则认领的 cast）就什么都不写：调用从文本与引用里同时消失。两个最小 fixture 的**修前实际文本**（`crates/jarde-java/tests/p3_patterns.rs`）：

- 被拒的 cast 消费者（`invokestatic Test.value()Ljava/lang/Object;` → `checkcast java/lang/String` → `astore_1`）——引用里只有 BCI 3 与 6，**BCI 0 一次都不出现**；
- 被拒的参数（`checkcast` 的值喂给 `invokestatic Test.take(Ljava/lang/String;)I`）——引用里只有 1 与 7，**BCI 4 不出现**。

修法两条，都在「一条会产出语句的指令不得从产物里消失」这条不变量上：(a) **读者集合按规则收窄**——`renders_the_value_it_reads(bci)` 只把「本 build 会把该值写进自己文本」的指令算作读者（`Store/Invoke/Return/Comparison/Switch/Arithmetic`；`CheckCast` 仅在 `bridge@1` 认领时；`Field`/`ArrayLoad` 仅在各自规则认领时），被引用的读者不再是理由；(b) **被引用的区域/生产者要报全**——`Region::If/Switch/Loop` 的测试读不出来时引用**该区域覆盖的每一个 BCI**（`region_bcis`），语句因值渲染失败而回退时引用**它没能写出的那些调用的 BCI**（`deferred` 记下「为某个读者跳过自己语句」的调用，`deferred_producers` 沿生产者链收集）。修后：探针 1 的文本是 `value();` 加 BCI 3、6 的引用（调用**恰好一次**），探针 2 的引用是 `// @bytecode 7 4`（4 就是那次调用）。这同时覆盖本轮复核列在 1.3d 的 **P3-R2（消费者 fallback 时丢失生产者 effect）**；**P3-R1（跨写入的旧值重读）不在本片**：本片没有 liveness 证明，`new@1` 对参数仍要求「生产者在本站点区间内」，宁拒不猜。

**2. A：inner/local/anonymous 的**使用点**，以及本片不声称的那一半。** 这类类在字节码里是普通类，类名由池原样给出（`p/Outer$1`）。`new@1` 认领的形状是 `new C ─ dup ─ [参数生产者]… ─ invokespecial C.<init>(…):V`，且要求：构造函数调用与 `dup` **同块**、接收者是本站点产出的实例、每个参数的生产者**落在 `dup` 与调用之间**（写进 `new` 表达式即与字节码同序）、区间内无 effect（`StatementFree`）、实例被**本 build 会写的**读者读到（`dup` 不算、被引用的指令不算：否则 `new` 表达式无处可写）。呈现 `new Type(args…)`，**参数按站点读取顺序、各自锚在自己的 BCI**。**不声称**：嵌套关系（`InnerClasses`/`EnclosingMethod`）与「这个参数是外层实例」——`MethodIr` 不携带任何类级事实（见 §4），外层实例就被写成站点真正读到的那个值（`return new p.Outer$1(self);`），不写 `Outer.this`、不写嵌套 `new Inner()`。拒绝即引用：参数顺序不可证（本片只在同块内认领）、实例只被引用的指令读（例如构造完又 `dup` 一次）。

**3. B：enum `switch` 呈现到哪一层为止。** javac 把 `switch (e)` 降成 `getstatic S.$SwitchMap$T [I ─ e.ordinal() ─ iaload ─ tableswitch`。`enumswitch@1` 认领的**是这次读**：静态 `[I` 字段 + 描述符 `()I` 的实例调用作下标；呈现就是这次读（`switch (p.Outer$1.$SwitchMap$p$Order[order.ordinal()])`，字段与调用的 BCI 都作为 derived 锚点进段表）。**不写 `switch (e)` 与 `case T.CONST:`**，理由是证据而不是工作量：常量名是**枚举类自己**的类级事实、表中「第 k 项是哪条常量」写在**合成类的静态初始化**里，两者都不在本次运行（也不在成员表里——那是另一个类）；字段名 `$SwitchMap$…` 只是编译器约定，与 `lambda$`/`access$` 同一条纪律：名字从不决定形状。记录与 `jre_enumswitch` 诊断把这条边界写成产物的一部分。拒绝即引用：下标不是调用（fixture 用局部量）时整只 `switch` 被引用，且引用**列出该区域覆盖的全部 BCI**。

**4. C：interface 的 `default`/`static` 方法——靠一条 caller-stated 的类级事实，而不是猜。** `default` 不是独立标志位：它是「interface 里既非 `static` 也非 `abstract` 的方法」（JVMS 4.6）。载荷里既没有成员的 flags 也没有类的 `ACC_INTERFACE`（`MethodIr` 的字段逐条查过：canonical/frames/ssa/code/constant_pool/bootstrap_methods，**没有任何类级事实**）。P3 2.2 已经为 bridge 开了「调用方从同一个类头读出成员 flags」的先例，本片沿用并补上同一族里的第二个：`MethodFacts::with_declaring_class(DeclaringClass { name, access_flags })`（**可选**，没交就是 `Refusal::unmet(Metadata{attribute:"declaring_class"})`，`jre_declaration_class_not_in_run`）。呈现落点是 **envelope**：`// @declaration an interface's default method of \`p.Shape\`, member flags 0x0001`（`a_constructor`/`a_static_initializer`/`instance method`/`static method`/interface 的三种各一句）。为什么只在注释里：重建方法签名要把描述符解析成类型（泛型、`throws`、注解），那是 3.x 的呈现决定，本层不越界。**载荷一字未动**（`jarde-jvm` 未改）；门面仍不交这条事实，所以经门面的运行给出**被陈述**的拒绝且不写声明行、`declaration@1` 也不进 `rules`（拒绝由记录与诊断点名）。

**5. D：构造器与字段初始化，顺序即语义。** 两条规则：`field@1` 把 `getfield/getstatic/putfield/putstatic` 呈现为 `receiver.f` / `Type.f` / `receiver.f = v` / `Type.f = v`——实例访问要求**帧给出的接收者静态类型恰好等于池写的 owner**（否则 `receiver.f` 可能指到子类自己的同名字段，`B extends A` 的双 `f` 就是反例）；静态访问没有这个问题。`init@1` 把实例初始化器的**序言**写成 `super(…)`/`this(…)`，判定链是：接收者是帧的 `UninitializedThis`（只有构造器自己的 `this` 在它的构造调用前是这个 token）→ JVMS 4.9.2 只允许它调用「声明类自己的」或「其直接父类的」 `<init>`（**帧 pass 自己在转换该 token 时就执行这条规则**，所以「不是声明类」⇒「是父类」是可证的）→ 声明类名由 `DeclaringClass` 交给本片。**不可证就拒不写**：没交类名时 `jre_init_class_not_in_run`，该 BCI 被引用（旧行为会写成 `self = self.<init>();`——`<init>` 不是合法 Java 标识符，且那是把 SSA 对 token 的转换误当成赋值）。**不重排**：语句按 BCI 次序产出，字段写就写在它自己的 BCI 处；内层类的合成引用写**在**序言之前（JVMS 4.10.1.9 允许的唯一 pre-init `putfield`，判定用 `UninitializedThis` + 声明类名 == `Fieldref` 的 owner），JLS 12.5 要求的「实例初始化在 `super(…)` 之后」因此不是被搬过去的，而是本来就那样。`<clinit>` 侧：`putstatic` 写成 `Test.g = 7;`，成员名 `<clinit>` 由 `declaration@1` 声明为「a static initializer」。

**6. 证据怎么读回来。** 报告新增 `news`/`fields`/`enum_switches`/`init`/`declaration`：`NewRecord`（head/dup/constructor/class/**参数的 BCI 序列**）、`FieldRecord`（access/is_static/owner/name/descriptor/presented/refusal）、`EnumSwitchRecord`（`TableRead` + `IndexCall`，各自带 BCI）、`InitRecord`（bci/target/class/declared）、`DeclarationRecord`（member_flags/declaring_class/interface/form）。presented 与 refused **都在**，每条都带 `RuleVersion`；`rules` 收录它们（`declaration@1` 只在真写了 envelope 行时收录，因为拒绝是对「本次运行没交事实」的陈述，不是对这段字节的结论）。诊断：每个拒绝一条 Warning；每一族一条 Info 汇总（`jre_new_sites`/`jre_field_accesses`/`jre_enumswitch`/`jre_constructor_prologue`/`jre_declaration`）。

**7. 独立 oracle 与它的边界。** `crates/jarde-java/tests/p3_patterns.rs` 的模型段（marker 守卫内）扩成：`Cell::Instance{class,text}`（构造的类名是模型自己的 key）、`new/astore/iconst/bipush/putfield/putstatic/iaload/return` 的解码、以及一条**有序 effect 日志**（`new C(args)`、`write 名字 = 值`、`prologue`、普通调用）。文本侧新增 `run_text_effects`（语句级读者，不解析表达式，与既有 `run_text` 并存），比较器新增 `compare_effects` = 既有三项 **+ effect 序列逐条相等**。**边界如实**：模型只覆盖它能执行的那两种 fixture（本片的构造器与实例化），不执行别的方法、不解析 `switch` 语句（B 的对照靠记录与 decode 的独立断言，不靠模型执行）；`compare_effects` 与 `compare` 分开，是因为一个被认领为 `+` 的链在文本里**没有**构造，而那是 `compare` 已经覆盖的形状。

**8. 验证与证伪（本片实际执行）。** `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` = **947 passed / 0 failed / 1 ignored**（2.2 基线 925，**+22** = `jarde-java` 单元 49→55（field/init/enumswitch 各 1、declaration 3）+ `p3_patterns.rs` 24→40；其余 crate 一字未动）；`cargo fmt --all -- --check` 与 `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`（1.98.1）干净；`openspec validate --all --strict --no-interactive`（`npx @fission-ai/openspec@1.11.0`，与 CI 同版本）= **12 passed**；两个 CI example 按 CI 口径 exit 0；分层 `cargo tree -p jarde-reader/jarde-query/jarde-jvm` 中 `jarde-java` 出现 **0** 次；`cargo metadata --manifest-path fuzz/Cargo.toml --locked` 通过、`cargo deny --manifest-path fuzz/Cargo.toml --workspace --locked --config deny.toml check` 四类全 ok（**未触及任何依赖边与 workspace 成员，`Cargo.toml`/`Cargo.lock` 一字未改**）。**三组证伪**（`/tmp` 副本 + 独立 `CARGO_TARGET_DIR`，`shasum -a 256 -c` 确认仓库侧逐字节未变，用完删除）：① 把字段写**提到语句最前**（跨过 `super(…)`）→ `a_constructors_field_initializers_are_written_after_its_constructor_call_and_in_order` 红，输出正是被重排的构造器体；② **放松 dispatch 表形态判定**（下标不是调用也认领）→ `a_table_read_indexed_by_a_local_is_refused_and_the_whole_switch_stays_quoted` 红，记录里出现被伪造的 `IndexCall { name: "", descriptor: "" }` 且 `presented: true`；③ **削弱 oracle**（`compare_effects` 不再比 effect 序列）→ 两条 `the_oracle_rejects_…` 红而同一文件其余 38 条（含全部生产用例）全绿。

**9. 被修正的既有断言（逐条，原→新→原因，无放宽）。**

| 位置 | 原 | 新 | 原因 |
| --- | --- | --- | --- |
| `p3_patterns.rs::a_chain_whose_instance_is_stored_in_a_local_is_refused` | `assert!(text.contains("// @bytecode"))` | `!contains(" + ")` + `contains("new java.lang.StringBuilder()")` + `matches("append(") == 2` + `Java/Structured` + `news[0].presented()` | 链仍被拒（该用例的其余断言一字未改），但被拒的链**不再意味产物有引用**：`new@1` 把分配写成 `new`，`append`/`toString` 按调用写出来。改后的断言比原断言更强（钉住两次调用各一次、顺序与平面），不是放宽 |
| 同上，`an_append_overload_that_plus_would_not_reproduce_is_refused` | 同上 | `!contains(" + ")` + `matches("append(") == 2` + `Java` | 同上：`CharSequence` 重载的转换仍不被写掉（没有 `+`），而链的指令被完整呈现 |
| `p3_patterns.rs` 既有 oracle 调用点（4 处） | `run_bytecode(&code, &pool, None)` / `Some("self")` | `&[]` / `&["self"]` | 模型的入参从「可选 receiver」变成「入口局部量列表」，本片需要 slot 1（构造器参数）**机械改动，断言本身未改** |

**10. 留给后面的（如实）。** §0 的 **P3-R1（跨写入的旧值重读）**仍是缺口，1.3d 处理；`field@1` 对「接收者静态类型 = owner」的要求会让**阴影字段**与**子类视图**的访问保持引用（宁可引用不猜）；`new@1` 对参数只认「生产者在本站点区间内」，join 出来的值（phi）、被引用指令消费的实例、跨块构造一律拒绝；方法转发型 accessor 与 `switch` 语句级的常量映射仍不做；3.2：新记录的 `Deserialize` 面与段表 `cp`；3.3：语料级 inner/enum/interface/构造器矩阵（多代 javac/ECJ、缺 debug、混淆、`this$0` 之外的合成字段）与受控重编译。

