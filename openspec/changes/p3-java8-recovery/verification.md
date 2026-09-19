# P3 实施验证记录

当前状态见文末 [2026-09-19 恢复复核](#review-2026-09-19-recovery)。下列各交付片段的待实施、任务归属与数字是历史时点记录；继续实施以本轮复核和 tasks 当前路线为准；已完成 1.1/1.2/1.3/2.1/2.2，新修正单列 1.3d，不把历史完成解释为新反例已通过。

## 2026-09-19 1.1：只读 IR 交接与产物词汇（提交 `b422f80`、`bbebe93`）

分两片交付。**1.1 仍未勾选**：任务文本的最后一项（「未满足前置条件 fixture 的边界与不误识别」）**依赖前置条件类型**，而那些类型随**模式 pass**落地（见下），故该项随 1.3 验证——如实记录，不当作已完成。

### 1.1a（`b422f80`）：只读 IR 交接

- **`MethodIr`**：`jarde-jvm` 自有，**按值持有**该次运行发布的三个表（`Option<Box<CanonicalCfg>>` / `Box<FrameTable>` / `Box<SsaTable>`）；字段全私有、无外部构造路径；`canonical()/frames()/ssa()` 只给 `Option<&…>`，且 `ssa ⟹ frames ⟹ canonical`（阶段有效性，`new()` 内 `debug_assert!` 固定）。
- **只读面**：把中端需要的值类型逐个改为 `pub` 并在 `method_ir` 统一再导出（字段仍 `pub(crate)`，读经访问器）；**机器保持私有**（raw CFG/effects、call contexts、fact ledger、pass table、`AnalysisRun`、三个 pass 入口及其输入视图）。唯一新增读取入口 `SsaTable::values_with_ids()`（否则 `ValueId` 不可枚举）。
- **入口**：新增 `engine::analyze_method_ir` 返回 `MethodIrAnalysis { report, ir }`，与 `analyze_method` **共享同一条运行**（不重复跑 P2）；`analyze_method` 的签名/行为/schema **一字未改**。
- **所有权/生命周期**：一次请求 → 一次运行 → 一份载荷，调用方按值持有、在其作用域内借给恢复层。**不需要 `Arc`/缓存**的理由已写入模块文档：同步引擎里没有第二个消费者、没有跨请求身份可缓存，且缓存会让上次请求的产物**活过为它付费的预算**。
- **为什么不是从摘要重建**（父级独立核对）：`MethodAnalysisReport` 的字段里**没有块、没有 BCI、没有值、没有 phi**——重建只能得到一张编造的图。
- **证据**（真实 ECJ v45 `finallyPath(I)I`）：6 个 canonical 块、4 条边、2 个 clone、3 个不可达节点、1 个 throw site、3 个 frame 条目、帧的 locals 数 == 该 body 独立解码的 `max_locals`、10 个 SSA 值、10 条 effect 记录、块起点集合是真实指令起点的子集。
- **证伪两组**：① 载荷改为由报告字段推导 → 3 条红（含「取消的运行不交表」）；② 加 `pub fn canonical_mut(&mut self) -> …` 或把某个表字段改成 `pub` → 源码守卫红（Rust 无法断言「不存在 `&mut`」，故沿用仓库既有的源码守卫风格，并配非空洞性检查）。

### 1.1b（`bbebe93`）：产物词汇

**父级在本片派单时**给了两条判据，实现者**按规格原文纠正了其中两处，父级已复核并确认实现者正确**：

| 父级派单的说法 | 规格实际写的（`specs/recovery-validation/spec.md:9`） | 结果 |
| --- | --- | --- |
| `Java` 加到 **syntax_status** | `representation（Java/Bytecode/Mixed）` | **父级错**：`Java` 归 representation |
| `Structured` 归 representation | `quality（Structured/Conservative/Fallback）`，且「`Mixed` **只表示 representation，禁止把它当作 quality**」 | **父级错**：`Structured` 归 quality |
| 「spec 没明写就不要加」`compile_status`/`verification` 变体 | 该句**明写**了 `compile_status（NotAttempted/Compiles/Failed）` 与 `verification（Performed/NotPerformed/Failed）` | 应加 |

**根因（父级记录）**：父级只引用了 `specs/java8-recovery/spec.md`，而该 change 有**三份** delta（`java8-recovery`、`recovery-validation`、`source-maps`），逐值清单在**未被引用的 `recovery-validation`** 里。教训：派单前应先 `ls` 该 change 的全部 spec 文件，而不是凭记忆引用一份。

**实际交付**：新增 `Representation{Java, Mixed}`、`Quality{Structured}`、`SyntaxStatus{Checked, Unchecked}`、`CompileStatus{Compiles, Failed}`、`VerificationStatus{Performed, Failed}`（最后一个定义在 `jarde-reader/src/classfile.rs`——P2 报告复用的就是该类型，故新值只能落在类型所在处；**读者层有先例**：P2 曾向 `CountedBudgetDimension` 追加六维）。**既有变体的名字与 serde 形状一字未改**，既有断言零改动。

**文档**：每个新变体都写明**归属阶段**、**P2 从不产出它**、以及语义边界——其中两处值得记：`Mixed` 是 representation **不是** quality；`NotJava` 有**第二种**情形（可读结果含 Java 表达不了的名称/结构，此时 `representation=Java` 与 `syntax_status=NotJava` **并存**，见该 spec 的 `Structured output cannot compile` 场景）。

**非死代码的证据**（`tests/p3_product_vocabulary.rs`）：一条用例两段——每个取值的 JSON 名 == 由变体名派生的 snake_case 且可往返；取值集合 == **从声明源文件读出的变体**（同序）。**父级独立证伪**：给 `Representation` 临时加 `Pseudocode` 不改表 → 用例如期转红（`left: ["Bytecode","Java","Mixed"] / right: […,"Pseudocode"]`，`sha256sum -c` 还原）。

### 分工记录（避免 1.1 看起来漏做）

`RecoveryProfile`、模式前置条件、rule version、失败 fallback **不在本片**：P3 design 决策 1 说这些由**每个模式 pass 声明**，而模式 pass 在 `jarde-java`；`layer-jarde-crates` design 明确「P3 `1.3` 随第一个真实 Region→AST→文本闭环创建 `jarde-java`」。故它们随 1.3 落地，1.1 的「未满足前置条件 fixture」验证随之一并做。

### 证据

全量 **818 passed / 0 failed / 1 ignored**（812 → 817（1.1a 的 5 条）→ 818（1.1b 的 1 条），**既有断言零改动**——父级用 `git diff -U0 | grep '^-' | grep -cE 'assert|expect'` 核为 0）；`p3_method_ir` 4、`p3_product_vocabulary` 1；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0。
**CI**：`b422f80`（1.1a）→ run 35433686800、`bbebe93`（1.1b）→ run 35434220030，均四 job success。

## 2026-09-19 1.3a：`jarde-java` 与最小闭环（提交 `fc2d24b`）

创建恢复层并打通**直线 + if** 的闭环：真实单方法入口 → `MethodIr` → Region → Java AST → **文本 + source map + 诊断**。

### crate 与分层

- `crates/jarde-java/`（已入 workspace members）。依赖：`jarde-jvm`、`jarde-reader`、`petgraph =0.8.3`（与 jvm 同 feature）；dev 另加 `jarde-reader`(test-support)、`blake3`。**0 个新第三方包**——`cargo tree -p jarde-java` 的闭包只有三个 workspace crate 加 petgraph 及其既有传递依赖。
- **分层门禁仍绿**：父级按 CI 口径本机复跑 12 个配置（`jarde-reader`/`jarde-query`/`jarde-jvm` × {normal,all} × {默认,--all-features}）全部 PASS，无配置触到 `jarde-java`；父级另跑 `cargo tree -p jarde-jvm` 确认闭包中 `jarde-java` 出现 **0** 次。`fuzz/Cargo.lock` **无需更新**（`cargo metadata --manifest-path fuzz/Cargo.toml --locked` 通过）——这一条是父级特别要求的，因为历史上正是独立锁文件缺包导致过 CI 红。
- 模块：公开 `source_map`/`names`/`facts`/`normal_flow`/`region`/`ast`/`report`/`stop`；私有 `build`/`emit`。入口 `recover(&RecoveryRequest, &mut Budget) -> RecoveryReport`。

### 四者职责的落点（数据流单向）

| 层 | 落点 | 边界 |
| --- | --- | --- |
| ① 正常流图视图 | `normal_flow::NormalFlowView` | 只保留 `Normal`+`Return` 边的 `petgraph` 投影；**不删边、不写回、不推异常语义**（异常/`jsr` 边只在计数里） |
| ② 事实 | `facts::{RecoveryFacts, Operation}` | 不产语句、不决定语法 |
| ③ Region | `region::{recover, Region, FallbackReason}` | `Straight`/`If`/`Fallback`；**不发文本**、证据不足**不产空 body** |
| ④ AST+emitter | `ast`/`build`/`emit` | 文本与段表**同一批写入**；逐写入口预算检查；转义 |

petgraph 只出现在 ①。

### source map 载体（3.2 的地基）

`Origin { bci, cp: Option<u16>, provenance: Direct|Derived }`；`Segment { start, end, origin }` 以**生成文本字节区间**为键；表按**完成顺序**记录（嵌套节点内层先完成）→ `covering(byte)` 给最具体节点、`of_bci(bci)` 给该 BCI 触到的全部、`direct_of_bci`/`derived_of_bci` 分开。`Origin::cp` 本片恒 `None`（1.1 载荷不发布 CP 索引），**字段留出以免 3.2 改表形状**。

### 用例与数字（29 个：20 单元 + 9 集成）

- 直线含调用：`Object local1 = null;` / `local1.run(0L);` / `return;`，`text_of_bci(4)` 同时给出**表达式**与**语句**两种粒度。
- **分支极性**：`ifeq` → fall-through 臂在 `if`、跳转目标在 `else`；BCI 3 同时有 `if` 语句（`Direct`）与条件节点（`Derived`）。
- 无 debug → 确定性 `local1`；关键字 `int` → `int_` 且 `representation=Java` + `syntax_status=NotJava`（**这两个并存的组合**正是 1.2 记录里点名的规格形态）。
- 转义 9 向量（`"`、`\`、`\n`、`\u0000`、`\u0007`、`\u007f`、`\u2028`、`😀`→`\ud83d\ude00`、源码 `\u0041`）。
- **真实语料**：ECJ v45 `add(II)I` → `return arg1 + arg2;`（Java/Structured/Unchecked）；`finallyPath(I)I`（jsr/finally）→ **Mixed/Fallback** + `// @bytecode …` 引用的 BCI **全是该 body 的真实指令起点**。
- **预算/取消/缺表**：恰好成、少 1 字节 → `text==""`、`Bytecode/Fallback`、`execution=Partial{BudgetExceeded}`、`jre_output_budget`；取消 → `text==""` + `Cancelled`；缺 canonical 表 → `IrTableMissing`。**不产空 body、不报成功**。
- 单元 `a_stop_inside_a_node_leaves_nothing_behind` 遍历所有 bound，证明**存在在节点内部停止的边界**（不是只在语句边界停）。

### 探针与开发中抓到的两个真 bug（父级记录）

1. **反向支配的虚出点边方向错**：原来每个 `immediate_post_dominator` 都是 `None` → **if 臂不汇合会被误判为循环**。由用例抓出并修。
2. **命名表按 `max(参数, debug)` 建表**：无 debug 时**整表无名字**。改为按 frames 的槽数建表。

### 证伪三组

| 变异 | 结果 |
| --- | --- |
| A 段表区间整体 +1 字节 | 红：段表顺序用例 + 2 条集成（`19 passed; 1 failed` / `7 passed; 2 failed`） |
| B 超限时报成功（`return Ok(())`） | 红：3 条单元 + 1 条集成 |
| B2 超限时**保留半成品缓冲**（仍返回停止理由） | **首轮全绿** → 暴露 `discard()` 当时**不可观测**；补 `a_refused_write_empties_the_buffer_it_had_already_filled` 后转红 |

B2 值得记：**「返回正确的停止理由」并不足以证明「没有留下半成品」**——需要一条专门盯缓冲的断言。

### 证据

全量 **847 passed / 0 failed / 1 ignored**（818 + 29：`jarde-java` 20 单元 + 9 集成，其余 crate 数字未变）；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0。
**CI**：`fc2d24b` → run 35438965338 四 job success（**含分层闭包门禁**）。
**既有断言零改动**（开发中修正的 7 处都是本次**新增**用例的期望值或实现，逐条见 tasks 记录）。

### 遗留的架构问题（已路由 1.3b）

1.3a 指出：`MethodIr` 发布**结构**，但**不发布符号与操作数词汇**——CP 引用的 owner/name/descriptor、常量值、槽命名、以及**分支极性**（`ifeq` 与 `ifne` 图同构，极性唯一来源是解码事实）都不在其中；这些经 `RecoveryFacts`**由调用方交入**。缺事实会落成被陈述的 fallback（安全），但**错的事实会让极性静默反向**。**父级已裁定**：恢复层必须读**同一次运行**的解码事实——而 `engine` 里 `facts: Option<MethodCodeFacts>` 本就是运行内局部量（约 250/329 行）却没交给载荷（约 774 行）。该缝在 **1.3b** 闭合。


## 2026-09-19 1.3b：事实缝闭合、循环/switch、不可约与交叉异常、独立小图 oracle

**命令与数字**（全部在本机实跑）：

| 检查 | 命令 | 结果 |
| --- | --- | --- |
| 全量测试 | `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **867 passed / 0 failed / 1 ignored**（1.3a 基线 847 —— 在 `git archive fc2d24b` 的独立副本上用同一条命令重测；+20 = `jarde-java` 单元 20→34、集成 9→15） |
| 格式 | `cargo fmt --all -- --check` | 干净 |
| 静态检查 | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 干净 |
| 规格 | `openspec validate --all --strict --no-interactive` | 12 passed / 0 failed |
| CI example | `cargo run --example inspect_class_header/resolve_and_analyze --locked -- tests/fixtures/.../v52/HistoricalControlFlow.class` | 两个 exit 0 |
| 分层 | `cargo tree -p jarde-jvm/jarde-reader/jarde-query | grep -c jarde-java` | 0 / 0 / 0 |

**事实缝的证明形式**：`RecoveryRequest { ir, facts }` 两个字段，`facts` 只承载方法身份与 debug 名——**签名上已无参数**可以交出极性、槽号或常量值；解码事实按值随 `MethodIr`（`code()`/`constant_pool()` 只读）交出，`new()` 断言「有图必有解码」。

**四组证伪**（`/tmp` 副本 + 独立 `CARGO_TARGET_DIR`；`shasum -a 256 -c` 逐文件确认只有目标文件变化；副本用完删除）：

1. **放行循环 test 块的 effect**（`region.rs` 去掉循环的纯度前置 + `build.rs` 把 test 块语句写在循环之前）→ `a_loop_whose_header_writes_state_is_quoted_rather_than_hoisted` **红**，失败输出正是被禁止的形状：`local1 = local1 - 1;` 写在 `while (local1 != 0) {}` 之前。
2. **`ifeq` 极性反转**（`decode.rs` 一行：`0x99 => CompareOp::JumpIfNotZero`）→ **4 条红**：`decode::…polarity…`、`oracle::…agree_on_polarity`、`an_if_else_is_written_with_the_arm_the_branch_really_picks`、`a_while_loop_keeps_its_test_inside_the_statement_and_its_body_in_order`；`shasum -c` 显示只有 `decode.rs` 变了。
3. **削弱 oracle**（比较器不再比较 calls 序列）→ `the_oracle_rejects_a_loop_effect_moved_out_of_the_loop` **红**，而 `p3_java_recovery.rs` 的 15 条生产用例**全绿**；`shasum -c` 显示只有 `oracle.rs` 变了——独立性不是声称。
4. **（附加）去掉不可约检查** → `a_graph_that_is_not_reducible_is_quoted_whole_with_its_own_reason` **红**：答案退化为 `jre_region_arms_do_not_meet` + `jre_region_uncovered_blocks`，而不是整具身体的 `jre_region_irreducible`。

**oracle 在本片发现的生产 bug**：`iflt/ifge/ifgt/ifle`（一个值对零）与 `if_icmp*`（两个值）被合成同一个 `CompareOp` 变体，分支 arity 前置条件因此把一个合法循环判成 `UnrenderableOperand`；现已按操作数个数拆成两组变体，并由 oracle 的两个 sense × 三个输入的极性用例固定。

**被修正的既有断言**：**零条被放宽**。1.3a 的 29 条用例逐条仍绿；唯一行为变化是**增强**——`an_if_else_is_written_with_the_arm_the_branch_really_picks` 的 fixture 在分支块里还有 `iconst_0; istore_1`，旧实现静默丢掉这条语句，现在它按次序写在 `if` 之前，而该用例原有的四条断言（`if (local1 != 0)`、fall-through 在 then、`} else {` 的位置、同一 BCI 的 direct+derived 两个 provenance）一条未改。

## 2026-09-19 1.3b：事实缝闭合、循环与 switch、独立 oracle（提交 `a0954ed`）

### 事实缝闭合（本片的架构重点）

**动机（父级已裁定）**：1.3a 时 `RecoveryFacts` 的**操作表由调用方构造**。缺事实 → 已落成被陈述的 fallback（安全）；但**错的事实**（把 `ifeq` 标成 `ifne`）→ **静默产出极性相反的 Java**，下游无从发现。

**落地**：
- `MethodIr` 新增两个**按值**字段：`code: Option<Box<MethodCodeFacts>>`（`raw_facts` 解出的指令、typed operands、声明的异常表）与 `constant_pool: Vec<CpEntryFacts>`（**同一次** header 读的池）；读面只有 `code()`/`constant_pool()` 两个只读借用；引擎在 run 末尾 **move**（不重解码、不重读、计费与停止语义未变，`Cargo.lock` 一字未动）。
- 新增 `jarde-java/src/decode.rs`：`Operations::of(code, pool)` 是 opcode → `Operation` 的**唯一**映射；未建模 opcode 与解析不出的引用 → `Other`（被陈述）。
- `RecoveryFacts` **缩减**为 `{ method, debug_locals }`；`operations` 字段与 `with_operation()`/`operation()`/`operations()` **全部删除**。
- **唯一来源由签名证明**（父级独立核对）：`RecoveryRequest` 现在只有 `ir` 与 `facts`，**没有任何参数**能交进极性/槽号/常量值——调用方**给不出**第二个意见。这比任何断言都强。
- **父级修正的一处陈旧文档**：`report.rs` 里 `facts` 的注释仍写「...and the decoded operations of its body」——操作已不在 `facts` 里，已改为如实表述（`facts.rs` 的模块文档本来就把这条写对了）。

**oracle 抓到的生产 bug**：`iflt/ifge/ifgt/ifle`（一个值与零比）与 `if_icmp*`（两个值比）被合成同一变体，分支 arity 前置条件因此**把合法循环判成 `UnrenderableOperand`**。已按操作数个数拆成两组（`CompareOp` 10 变体）。

### 循环与 switch

- `normal_flow`：immediate dominators、`dominates()`、`natural_loops()`（回边 = target 支配 source）、**SCC 级** `irreducible_blocks()`（分量内唯一入口且支配全分量，否则不可约——**回边级判据会漏**掉只有一条回边的不可约图）。
- `region`：`Loop{header, test, test_bci, form: While|DoWhile, continuation, body, exit}`、`Switch{prefix, branch, groups, join}`。两种可证明形状：header 自测（`while`/`for`）与唯一 latch 测（`do…while`）；switch 按**终指令是否为解码出的 switch** 分派（不按后继数）。
- **effect 次数如何保证（P3 design 明写的关键不变量）**：条件/selector 写在 `while (…)`/`do {…} while (…)`/`switch (…)` 的**括号里**，其值表达式就是 test 块指令的文本 → **每次求值一次、次序同字节码**；body 语句留在花括号内。**守卫**：循环 test 块**必须纯**，否则 `TestBlockEffect` fallback（**不搬走**）；从分支离开环的边 → `LoopLeavesEarly`；环内块必须全被走到否则 `LoopShape`。

### 不可约 / 交叉异常

两者都在走路**之前**判定并**整具身体**参考 bytecode：`Irreducible{blocks}` → `jre_region_irreducible`；`CrossingExceptionRegions{record, other, blocks}` → `jre_region_crossing_exception_regions`（按**声明范围**判定「部分重叠、互不包含」）。平面为 `representation=Mixed`、`quality=Fallback`、`execution=Complete`（**扫描完整不写 `Partial`**），不报 `Structured`、不产空 body。

### 独立 oracle（`src/oracle.rs`，`#[cfg(test)]`）

自写**朴素字节码机**（自有解码器 + 栈/局部变量机）+ 自写**文本模型**（自有解析器/求值器）+ 比较器（**calls 序列、return 值、test 求值次数**三项全等）。
- **独立于**：生产 Region/AST/段表全部 helper——模型段（两 marker 之间）不含 `region::`/`Region`/`ast::`/`StmtKind`/`ExprKind`/`NormalFlowView`/`SourceMap`/`text_of_bci`/`build::`/`recover(`，由 `include_str!` 守卫断言，并**要求含** `run_bytecode`/`run_text`/`compare`/`parse`（非空洞性）。
- **不独立于**：fixture 字节（共享 ground truth）与四个 opcode 的含义（自己解码，抄错即红）——**如实记录**。
- **自造 ≥3 形状**：`polarity_fixture`（`ifeq` 与 `ifne` × 3 输入）、`while_fixture`、`do_while_fixture`、`crossing_class()`（自造 class 字节，异常表 `[2,4)`+`[3,6)`）。

### 证伪四组

| 变异 | 结果 |
| --- | --- |
| 放行循环 test 块 effect（test 块语句写在循环前） | 红，输出正是 `local1 = local1 - 1;` 排在 `while` **之前** |
| `ifeq` 极性反转（`decode.rs` 一行） | **4 条红**（decode 单测、oracle 极性、`an_if_else_…`、`a_while_loop_…`） |
| **削弱 oracle**（比较器不再比 calls） | oracle 用例红，而**生产 15 条全绿** —— 证明独立对照有牙、且它捕捉的是生产不会自曝的东西 |
| 去掉不可约检查 | 红，退化为 `jre_region_arms_do_not_meet` + `jre_region_uncovered_blocks` |

### 既有断言：零条放宽

1.3a 的 29 条逐条仍绿。行为上的差异只有两处，**都是增强**：`invokeinterface` 的目标从测试硬编码改为**从类自己的池解析**（取值相同）；`an_if_else_…` 的分支块里原本被**静默丢掉**的 `local1 = 0` 现在按次序写在 `if` 之前（该用例原有四条断言一条未改）。

### 证据

全量 **867 passed / 0 failed / 1 ignored**（847 + 20：`jarde-java` 单元 20→34、集成 9→15，其余 crate 未动）；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0；`cargo tree` 对 reader/query/jvm 三个包中 `jarde-java` 出现 **0/0/0**。
**CI**：`a0954ed` → run 35441081367，四 job success。

### 本片明说的边界（未做，留给后续）

带写的循环 header 与 `break`/`continue` 形状走 `TestBlockEffect`/`LoopLeavesEarly` **fallback**（不搬走、不丢弃）；`do {} while (c)` 的空体走 `LoopShape`；`ldc` 的 `float`/`double` 字面量仍 `Other`；`athrow`/`try`/`finally` 的**呈现**属 2.4（本片只保证异常事实在 fallback 里正确）；字段/数组/转换类操作属 2.x；段表 `cp` 面属 3.2。

## 2026-09-19 1.3c：门面与 CLI 入口、库/CLI 一致、A09/A10/A13/A16、模式声明类型

落点与逐条裁决见 design 的「1.3c 的实际落点」，任务记录见 tasks 的 1.3 第三条。本节只记证据与限制。

### 入口与分层

`Engine::recover_method`（`src/facade.rs`）= `jarde_jvm::analyze_method_ir` **一次** + 把该次运行的载荷交给 `jarde_java::recover`，返回 `RecoveredMethod{analysis,recovery}`（两半同一次运行）。CLI 新增 `recover_method` operation（`environment` + `method` + `stages`；**无独立 profile 字段**，门控读 `environment.runtime.profile`），响应 `{"kind":"recover_method","analysis":…,"report":…}`；报告内停止仍是成功载荷。**只跑一次分析**：adapter 里对库的调用只有 `engine.recover_method(...)` 一次。

门面收窄：跨 `jarde` 的只有 `RecoveryRequest`/`RecoveryFacts`/`MethodFacts`/`recover`（请求）、`RecoveryReport`/`RecoveryOutcome`/`RegionRecord`/`SourceMap`（报告）、`RecoveryProfile`/`RuleVersion`/`Precondition`/`IrTable`/`StopReason`（报告里名得到的只读词汇）；`region`/`ast`/`build`/`emit`/`names`/`facts`/`decode`/`normal_flow` 与 `Segment`/`Origin`/`OriginSet` **不在**门面上。

### 用例与数字

全量 **877 passed / 0 failed / 1 ignored**（867 + 10：`jarde-java` 单元 34→37（`pass` 合同用例 3）、集成 15→18（A10 1 + A13 2）、root `tests/p3_recovery_entry.rs` 2、CLI json_cli 11→13）；`cargo test -p jarde-cli --locked` = **24**（22 + 2）；`cargo fmt --all -- --check` 与 `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`（1.98.1）干净；`openspec validate --all --strict --no-interactive` = 12 passed；两个 CI example exit 0；分层门禁按 CI 口径 12 配置全 PASS（`jarde-reader`/`jarde-query`/`jarde-jvm` 的 normal/all × 有无 `--all-features`），`cargo tree -p jarde` 含 `jarde-java` 1 次。`Cargo.lock` diff 只有依赖边：root 的 `jarde-java`/`serde` 与 `jarde-java` 的 `serde`，**无新第三方包**。

### A09/A10/A13/A16（用例名 → 断言要点 → 未覆盖的半）

见 design 的 1.3c §7 表（四行，逐条列出用例名、断言与「哪半属 2.4/3.1/3.2」）。摘要：A09 = `a_body_the_subset_cannot_prove_is_quoted_rather_than_emptied`（`Mixed`/`Fallback`/`Complete`、无 `try {`/`} finally`/`} catch`、BCI 引用真实、诊断可陈述；**恢复本身属 2.4**）；A10 = `a_body_without_debug_names_is_named_deterministically_and_invents_no_source_scope`（两次运行整份报告相等、序号名、`cp=None`、带 debug 名时用 evidence、无 evidence 时不出现拼写；**语料矩阵属 3.1/3.3**）；A13 = `a_member_that_cannot_be_presented_leaves_the_member_that_can_alone` + `execution_quality_and_representation_each_state_their_own_thing`（同类两成员互不影响、两种顺序都逐字段稳定；平面三分：`Mixed`/`Fallback` 与 `execution=Complete` 并存，停止运行只由 `execution` 表达且诊断不串台；**语料矩阵属 3.2**）；A16 = `one_recovery_request_reads_one_body_and_presents_that_member`（真实 v52 fixture，前提经 `inspect_header` 断言 ≥3 个带 `Code` 成员，`class_headers=1`/`method_bodies=1`/`reads` 一条 `DriverMethodBody`）与 CLI 侧同请求的 wire 断言。

### 被取代的既有取值（原→新→原因，无放宽）

`FallbackReason::TestBlockEffect { block_bci, bci }` → `FallbackReason::UnmetPrecondition { pass, requirement, block_bci, at }`；诊断码 `jre_region_test_block_effect` → `jre_region_unmet_precondition`。原因：P3 决策 1 要求模式 pass 的类型化前置条件，这条拒绝必须带上**规则版本**与**要求类型**才能进报告/诊断。用例 `a_loop_whose_header_writes_state_is_quoted_rather_than_hoisted` 的断言**加强**：新码之外另断 `RegionRecord.rule == Some(loop@1)`、消息含 `loop@1`/`value expression`/`BCI 5`、`report.rules` 含 `loop@1`、`execution == Complete`。`region.rs` 其余取值一字未动（`FallbackReason::pass()` 对它们返回 `None`）。**既有的入口/报告断言无一条被放开**：`RecoveryRequest::new` 多一个必填 profile 参数（3 个调用点改为 `pass::JAVA_8`），`RegionRecord` 多一个 `rule` 字段、`RecoveryReport` 多 `profile`/`rules` 两个字段（值随运行而定，不是放开的谓词）。

### 证伪两组（`/tmp` 副本 + 独立 `CARGO_TARGET_DIR`，`sha256sum -c` 逐文件核对后删除）

① CLI 的恢复响应改写一个字段（`crates/jarde-cli/src/main.rs` 里把 `report.quality` 改成 `Fallback`）→ `recovery_matches_the_library_entry_field_by_field` **红**，失败输出显示两份文档**只差 `quality`（`"fallback"` vs `"structured"`）**，其余字段逐字相同——即整份文档比较确实在看每一个字段。
② 前置条件未满足时仍产出结构（`crates/jarde-java/src/region.rs` 的 `test_is_pure` 改为 `Ok(())`）→ `a_loop_whose_header_writes_state_is_quoted_rather_than_hoisted` **红**，失败输出正是那条静默改次数的形态：`int local1 = 2; while (local1 != 0) { } return;`（header 每轮的 `local1 = local1 - 1` 被丢掉）。两组副本与独立 target 目录用完已删除。

### 本片明说的限制（不虚报）

`Pass::admits` 的**拒绝分支在 2.x 之前没有生产实例**：本片 4 条已注册规则全部 `required_release: None`（结构规则与 release 无关），故只用 `pass` 模块的合同用例覆盖谓词语义（8/9 接受、7 拒绝），不伪造 pass 去触发它。入口侧的 `RecoveryFacts.parameters` **不猜 static 与否**（载荷不发布 access flags），故入口呈现一律用 `localN` 序号名，参数槽/receiver 命名属 3.1。`RecoveryReport` 的 `Deserialize` 面未做（码字段是 `&'static str`），库/CLI 一致以整份序列化文档逐字段相等为证据。`cargo doc` 对 `jarde-java` 的既有私有 intra-doc 链接告警（`build`/`emit`/`decode`/`charge`）为**既有**、非 CI 门禁（本片新增的 `facade.rs` 链接已修）。

## 2026-09-19 1.3c：门面与 CLI 入口、验收覆盖、恢复侧声明类型（提交 `2d156d8` + `a681e4c`）

**1.3 至此完成**（1.3a/1.3b/1.3c），1.1 遗留的四项类型也随本片落地并勾选。

### 门面与 CLI

- 根门面 `jarde` 依赖 `jarde-java`（分层链 `jarde → jarde-java → jarde-jvm + jarde-reader`；闭包门禁只禁 reader/query/jvm，不禁门面）。**只导出**恢复请求/报告半边与报告里名得到的只读词汇；**不导出** `jarde-java` 的任何内部模块路径（父级核对：`src/lib.rs` 里对 `region`/`ast`/`build`/`emit`/`names`/`facts`/`decode` 的引用计数 **0**）。
- CLI 新 operation（wire tag `recover_method`）= `environment` + `method` + `stages`，与 `analyze_method` 同形；响应含 `analysis` 与 `report`。
  - **不另设 recovery profile 字段**：门控读的就是 `environment.runtime.profile`——再加一个字段是同一事实的第二来源。
  - **只跑一次分析**：`Engine::recover_method` 调 `analyze_method_ir` **一次**，把该次运行的 report 与该次运行的载荷一起交给恢复层（`RecoveredMethod{analysis, recovery}`）。
  - 协议错误仍 transport 级；**报告内停止仍是成功载荷**（呈现停 ≠ 运行停，用例断言 `outcome.stopped` 与 `analysis.execution == complete` 并存）。

### 恢复侧声明类型（1.1 遗留）

- **`RecoveryProfile`：复用** `jarde_reader::view::RuntimeProfile`（规格句点名的就是它；新立枚举即同一事实的第二种拼写），`pass.rs` 做别名 + `JAVA_8` + `Pass::admits`。
- **`RuleVersion`**：`straight@1`/`if@1`/`loop@1`/`switch@1`，落进报告——`RegionRecord.rule` 记**谁产出、或谁拒绝**，`RecoveryReport.rules` 记本方法引用到的规则（首现去重），另记 `profile`。
- **`Precondition{IrTable, StatementFree, Metadata}`** 类型化 + 编译期常量表 `PASSES`（**无 trait、无动态注册**，符合 P3 design）。
- **失败 fallback 收敛**：`FallbackReason::TestBlockEffect` → **`UnmetPrecondition{pass, requirement, block_bci, at}`**，码 `jre_region_test_block_effect` → `jre_region_unmet_precondition`；构造带 `debug_assert!(pass.requires(requirement))`，使**声明与检查点漂移在本 build 的测试里失败**。缺表仍走**停止**、前置条件未满足走 **fallback**——两条都是报告里的值 + 诊断。

### 1.1 遗留的验证（父级独立证伪）

`a_loop_whose_header_writes_state_is_quoted_rather_than_hoisted`：文本**无 `while`**、只有 `// @bytecode`，记录与诊断给出 `loop@1` + `StatementFree` + `BCI 5`，`Mixed`/`Fallback`/`execution=Complete`。
**父级把前置条件检查改为恒 `Ok(())`** → 该用例转红，且失败输出正是要防住的形态：
```
int local1 = 2;
while (local1 != 0) {
}
return;
```
——header 每轮的 `local1 = local1 - 1` **被丢掉而结构看起来对**。即「不误识别」这条**确有承重**。

### A09/A10/A13/A16 逐条

| 验收 | 用例 | 关键断言 | **属后续阶段** |
| --- | --- | --- | --- |
| A09 | `a_body_the_subset_cannot_prove_is_quoted_rather_than_emptied` + 既有交叉异常用例 | ECJ v45 `finallyPath`：`produced()` 但 `Mixed`/`Fallback`/`NotJava`，**无** `try {`/`} finally`/`} catch`，引用 BCI 均为该 body 指令起点，诊断码 `jre_region_*` 且至少一条点名 BCI，`execution=Complete` | **finally/jsr 恢复本身属 2.4** |
| A10 | `a_body_without_debug_names_is_named_deterministically_and_invents_no_source_scope` | 无 LVT：两次运行**整份报告逐字段相等**；序号 `localN`、无 `arg`、无 `line`；segment 的 `cp=None`；有 debug 名时用真名、无证据时**不出现**该名 | 语料矩阵属 **3.1/3.3** |
| A13 | `a_member_that_cannot_be_presented_leaves_the_member_that_can_alone`；`execution_quality_and_representation_each_state_their_own_thing` | 同类两成员报告互不携带对方内容，两种顺序再问逐字段相同；平面三分：`Mixed`+`Fallback` 与 `execution=Complete` 并存，差一字节预算下停止**只由 `execution=Partial` 表达**（`quality` 仍是 `Fallback` 而非 `Partial`） | 语料矩阵与 coverage 诊断属 **3.2** |
| A16 | root `one_recovery_request_reads_one_body_and_presents_that_member` + CLI wire 断言 | 前提经 `inspect_header` 断言有 ≥3 个带 `Code` 成员；一次请求 `class_headers=1`/`method_bodies=1`/`code_bytes>0`/`reads` 恰一条 `DriverMethodBody` | 无逐 pass 计数器（5.2 已记），以「一次 header + 一次 body + 一条 read + 入口只调一次分析」作**代理**，不宣称有 |

### 被修正的既有断言（4 处，均加强或等价，无放宽）

`a_loop_whose_header_writes_state_…` 的 fallback 码改为 `jre_region_unmet_precondition` 并**新增**四点断言（规则版本可追溯）；A09 用例**仅新增**窄限定断言（首版的 `!text.contains("finally")` 会匹配方法名 `finallyPath`，已换成结构性三串比较——**这不是放宽，是把假断言换真**）；`RecoveryRequest::new(ir, facts)` → `+ profile`（3 个调用点补 `JAVA_8`）；CLI 测试的 fixture helper 委托化（既有调用方字节不变）。

### 第二次锁文件事故（**父级记录，含教训**）

`2d156d8` 的 CI **两 job 红**：`fuzz smoke` 的「Check the committed corpus against the targets」与 `supply chain` 的 fuzz 检查——**都是 `--locked` 失败**。根因：门面在本片新增了对 `jarde-java` 的依赖，而 **`fuzz` 是独立 workspace、有自己的 `Cargo.lock`**，该锁未同步。

**这与 P2 期间那次是同一类错误**（当时是 reader 抽出后 `fuzz/Cargo.lock` 缺包）。**为什么本地门禁没发现**：fmt / clippy / 全量测试 / 依赖闭包**都不读那个 workspace 的锁**——本机跑 `cargo metadata --manifest-path fuzz/Cargo.toml --locked` 只需一秒就能发现，但没人跑它。

**修复（`a681e4c`）**：只补 `jarde-java` 一条包记录，**无任何第三方版本/source/checksum 变动**（父级用 `diff` 逐类核对）。**教训写在此处**：凡**新增 workspace 成员**或**给根门面加依赖边**，必须跑一次
```
cargo metadata --manifest-path fuzz/Cargo.toml --locked
```
并把 `cd fuzz && cargo deny --manifest-path fuzz/Cargo.toml --workspace --locked --config deny.toml check` 一并跑过——这两条是唯一会读该 workspace 锁的门禁。

### 证据

全量 **877 passed / 0 failed / 1 ignored**；`-p jarde-cli` 24（基线 22）；`jarde-java` 单元 37 + 集成 18；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0；分层 12 配置全 PASS 且 `cargo tree -p jarde` 含 `jarde-java`；**`Cargo.lock` 无新第三方包**（父级核对：`git diff Cargo.lock | grep -c '^+name ='` = 0，只有依赖边）。
**CI**：`2d156d8` → 35442183621（**两 job 红**，见上）；`a681e4c` → **35442441114 四 job success**。

## 2026-09-19 2.1：lambda / method reference / capture 恢复（提交 `02d5492`）

### 形态判定（九步，全部只读**同一次运行**的载荷）

`invokedynamic` 站点原本落 `Other`；现解成 `Operation::InvokeDynamic(DynamicSite { cp, bootstrap_index, name, descriptor })`（只陈述**站点身份**，不陈述形状）。`MethodIr` 增加 `bootstrap_methods: Vec<BootstrapMethodFacts>`——在 `read_declaration` **同一次 header read** 里读并 move 进载荷（无该属性的类**读零字节、计零费**；无重解码/重跑）；复用 `jarde-reader` 已有的 `classfile::bootstrap_methods`（xref 也在用），**未触及任何依赖边**。

判定链（`lambda.rs`，任一步不满足即**拒绝**并给码）：

| 步 | 判定 | 不满足时的码 |
| --- | --- | --- |
| 1 | profile 准入 `lambda@1` | `jre_lambda_rule_not_admitted` |
| 2 | 类自己的表里**存在**该站点 | `jre_lambda_no_bootstrap`〔`IrTable(BootstrapMethods)`〕 |
| 3 | 句柄是方法句柄、成员是 **`LambdaMetafactory.{metafactory, altMetafactory}`**、kind 为静态 | `jre_lambda_bootstrap` |
| 4 | 静态实参按**种类与个数**为 `[MethodType, MethodHandle, MethodType]`；`altMetafactory` 第四参 = **标志字 0** | `jre_lambda_bootstrap_arguments` |
| 5 | 站点/SAM/实例化方法类型可读、参数个数一致、站点参数个数 == 该指令读到的捕获数 | `jre_lambda_descriptor` / `_sam_descriptor` |
| 6 | 实现句柄是调用或构造器（kind 5–9）；字段句柄不呈现 | `jre_lambda_implementation` |
| 7 | **arity 自洽**：`捕获数 + SAM 参数数 == 实现参数数 + receiver`，且**逐位形状相同** | `jre_lambda_sam_arity` / `_sam_types` |
| 8 | 每个捕获值**可重读**（字面量，或本方法只写一次的局部量） | `jre_lambda_capture_not_replayable`〔`Precondition::Replayable`〕 |
| 9 | 站点实例**有读者**（否则那条调用在产物里无处安放） | `jre_lambda_unconsumed` |

**`LAMBDA = lambda@1, required_release = Some(8)`** —— 本 build **第一条 Java 8 专属规则**，1.3c 记录的「`admits` 拒绝分支无生产实例」由此**关闭**（Java 7 profile 用例走真实门控）。新增效果族 `Precondition::Replayable`；`Precondition::IrTable` 的文档改为**两种检查点**（整趟必需 vs 规则认领时）——理由是**不含 bootstrap 表的类是普通的类**：站点缺席是被拒绝的**站点**，不是被拒绝的运行。

**明说没做**：适配（子类型/装箱/加宽）不比较引用名字——需要适配的站点一律**拒绝**，不写没人写过的转换。

### 呈现与 evidence

- **写法由事实决定**：实现句柄是本站点自己的 body（`lambda$…` 标记）→ **lambda**；否则当**捕获恰好等于句柄自己要的接收者** → **method reference**（`receiver::name` / `Type::name` / `Type::new`）；其余仍写 lambda。**标记只选写法，绝不决定「是不是 lambda」**——后者只由上面那条链决定。
- **capture 顺序/个数**取自站点指令的**栈操作数顺序**（SSA 值流），参数取自 SAM 的 `instantiatedMethodType`；参数名经 `names::free_name` 派生并避开局部量与别的 lambda 参数（JLS 6.4）。
- **evidence 读回**：新增 `RecoveryReport.lambdas: Vec<LambdaRecord>`——**presented 与 refused 都在**：`use_site`、`site_cp`、`bootstrap_index`、`bootstrap`（`owner.name (REF_kind)`）、`bootstrap_arguments`、`sam_name`/`sam_descriptor`、`sam_method_type`/`instantiated_method_type`、`implementation`、`captures`（按站点顺序，**各带来源 BCI**）、`form`、`refusal{code,rule,requirement,message}`。
- **段表**：lambda 节点 primary = 站点 BCI 且 `cp = Some(site_cp)`，**每个捕获值作为 `Derived` 锚点挂上**，捕获表达式自己仍锚在其产出 BCI——**一个表达式对应多个原始 BCI**；节点内部（接收者类型名、参数）只锚在站点，**不冒领锚点**。

### A04 的核对（父级独立证伪）

**反例形状**：自造 bootstrap `Test.myBootstrap` + **与 metafactory 完全相同**的三元静态实参 + 同样的站点描述符与捕获——**唯一差别是工厂**。
**父级**把工厂检查改为恒假 → **只有** `an_arbitrary_bootstrap_is_never_presented_as_a_lambda` 转红（其余 28+43 全绿），与实现者自报**一致**。该用例断言：文本**不含** `->`/`::`、含 `// @bytecode`、`Mixed`/`Fallback`；记录 `form == None`、`bootstrap == Some("Test.myBootstrap (REF_invokeStatic)")`、`refusal.code == "jre_lambda_bootstrap"`、`rule == lambda@1`、消息点名 `Test.myBootstrap` 与 `LambdaMetafactory`。
另有：SAM 不自洽 → `jre_lambda_sam_arity`；`altMetafactory` 标志字 1 → `jre_lambda_bootstrap_arguments`；未消费站点 → `jre_lambda_unconsumed`；Java 7 profile → `jre_lambda_rule_not_admitted`。

### oracle

扩成三段自写：**class 读取**（池 + `BootstrapMethods`，未知 tag 直接 panic，**不调用 reader**）→ **机器执行 fixture 字节**（栈上带**值来源**，故「站点按什么顺序从哪些槽捕获」是它自己读出来的；模型**不执行** lambda 体，只检查创建、调用次数与形状——已写明）→ **文本解析 + 比较器**。要求：写法与「实现名字带不带 body 标记」一致、lambda 参数个数与**类型拼写**等于它自读的 SAM 方法类型、body 调用名 == 实现名、**前 N 个实参就是它自读的那些槽名且顺序一致**；**槽名由 fixture 的 store 顺序与文本赋值顺序配对得出**，不依赖生产的命名规则。

### 证伪三组

| 变异 | 结果 |
| --- | --- |
| 放松形态判定（工厂检查恒真） | **只有** A04 反例红，产物正是 `java.lang.Runnable local2 = () -> Test.lambda$method$0(local1);` |
| 捕获顺序颠倒 | 集成顺序用例红（`() -> Test.lambda$method$0(local2, local1)`）；oracle 正例与其变异同时红 |
| **削弱 oracle**（顺序比较改成只比个数） | oracle 变异用例红，**生产 29/29 全绿** |

### 被修正的既有断言（8 处，均加强或纠正，无放宽）

`pass.rs` 注册表用例的 `PASSES.len()` 4→5、`required_release` 由「谁都没有」**收紧**为**逐规则 pin**（`lambda = Some(8)`、其余 `None`）、`IrTable` 允许表由 3 增至 5 并**新增**两条断言（`LAMBDA` 确实声明 `BootstrapMethods`+`ConstantPool`、四条结构规则**都不**声明）；`Precondition::IrTable` 文档改为两种检查点；`ast::Type` 文档澄清四族由**描述符**读者产出而帧读者仍不产出（描述符确实写着 `Z`，是事实不是猜测）；**`build::value_type` 的一处真 bug**——引用名只做 `/`→`.` 而未先剥 `L…;` 外框，会产出 `Ljava.lang.Runnable;`（本片首次走到该路径）；`oracle.rs` 的 `Text::Return(None)` 由 `Some(0)` 纠正为 `None`（裸 `return` 不返回值）。

### 证据

全量 **894 passed / 0 failed / 1 ignored**（877 + 17：`jarde-java` 单元 37→43、集成 18→29）；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0；分层三包中 `jarde-java` 出现 **0** 次；**未触及依赖边**，锁文件两条命令均 exit 0（`cargo metadata --locked`、`cargo deny … licenses ok`）。
**CI**：`02d5492` → run 35444591661，四 job success。

### 一处修正父级指令（如实记录）

父级在纪律里写的锁检查命令 **`cd fuzz && cargo deny --manifest-path fuzz/Cargo.toml --config deny.toml check` 字面形式会失败**（`--manifest-path` 必须指向 Cargo.toml，且 `cd fuzz` 后找不到 `deny.toml`——它在**仓库根**）。实现者按 CI 口径（仓库根、`--manifest-path fuzz/Cargo.toml`、`--config deny.toml`）执行并说明。**父级据此更正纪律**：两条命令**都在仓库根**跑，`fuzz` 只出现在 `--manifest-path` 里。

### 留待判断（实现者提出，父级记为可接受）

`lambda$` 标记**只**用于在两种语义等价写法之间选一个，被写成 `lambda.rs` 的**文档化决定**而非前置条件（标记缺失时仍产出 lambda，只是用 `::` 或带接收者的调用写）。若评审认为应升级为声明式前置条件，改动面是 `pass.rs` 的 `LAMBDA` 声明 + 一处检查点，不影响其余结论。

## 2026-09-19 2.2：concat / bridge / accessor（提交 `492e31e`）

三条规则注册进同一张 `PASSES`（5→8），沿用 2.1 的机制：`Refusal` 从 `lambda.rs` **提到共享模块 `refusal.rs`**（诊断码按 `(rule, requirement)` 查表），`Precondition`/`RuleVersion`/`Pass::admits` 未另立。三者 `required_release = None`——它们写出的构造（`a + b`、`x.f = v`、`return x.m()`）在每个 release 都是同一段 Java，**变化的是输入**，由规则自己声明。

### 形态判定

- **`concat@1`**（`concat.rs`，前置 `IrTable{Ssa,Code,ConstantPool}` + `StatementFree`）：接收者只接受 `{StringBuilder, StringBuffer}` 且类名读自同一次解码；形状必须是 `new C; dup; <init>; …; toString()`，接收者由本链产生；逐 `append` 检查描述符（参数类型须属于 `+` 语义相同的重载集合 `int/long/float/double/boolean/String/Object`、返回类型须是同一拼接类）；每个操作数的产生 BCI 必须落在「上一条链指令」与「它的 append」之间；**段内不得有产出语句的指令**；`toString` 结果必须被渲染型读者消费。拒绝码 `jre_concat_split`/`_interleaved_effect`/`_shape`/`_unconsumed`/`_overlap`。
- **`bridge@1`**（`bridge.rs`，前置 `IrTable{Ssa,Code,ConstantPool}` + `Metadata{access_flags}`）：身份只来自**声明**；body 每条指令必须是「读数槽 → 一次转发调用 → 可选 `checkcast` → return」；**唯一被接受的 cast** 是「被 cast 的正是转发调用的返回值，且 cast 类型正是该调用描述符声明的返回类型」（消去证明只依赖这一处池事实）；**参数上的 cast 一律拒绝**（那是会失败的检查）。拒绝码 `jre_bridge_shape`/`_cast_not_erasure`/`_not_declared`/`_flags_missing`。
- **`accessor@1`**（`accessor.rs`，前置 `IrTable{Ssa,Code,ConstantPool,Members}`）：调用点须是 `invokestatic`；callee 须在调用方交出的成员表里；声明须同时有 `ACC_STATIC|ACC_SYNTHETIC`；body 恰为「一次实例字段访问 + 读取 + return」（读形态 / 写形态），字段的类须是本类、类型须与描述符一致。**`access$` 标记只决定值不值得留 refusal 记录**，与 `lambda$` 同纪律。

### 拼接的求值顺序（三重守卫）

**(a)** 链拥有从 `new` 到 `toString` 的**整段 BCI（含操作数生产者）**，这些 BCI 一律不产语句 → 操作数只写一次；**(b)** 每个操作数的产生 BCI 必须落在「上一条链指令」与「它自己的 append」之间 → 写出的 `+` 从左到右与字节码同序；**(c)** 段内任何产出语句的指令（store / void 调用 / iinc / 字段写 / 未建模）按 `StatementFree` **拒绝整条链**。
断言：`"x" + 5 + f()` 逐字面 + `text.matches("f()").count() == 1` + 操作数顺序 `["\"x\"", "5", "f()"]` + 段表锚点含链的**全部** BCI；`StringBuffer` 与「抛异常调用在中间」各一条。

### A12 双 origin 边（本片验收核心）

读访问器 → `ExprKind::Field`（primary = **调用点** BCI，derived = **访问器 body 里 `getfield`** 的 BCI）；写访问器 → `StmtKind::FieldAssign`，同样双锚点。
根 crate 的 `tests/p3_accessor_edges.rs`：同一 fixture（字段 `f:I` + `access$100` + 调用方 `method()I`），**一份不走恢复、一份走恢复**，两次都跑 `Engine::query`，把整份序列化文档递归归零 `elapsed_millis` 后逐字段相等；**再各自断言两条边**——① `method` 的 BCI 3 → accessor 符号，② accessor 的 BCI 1 → 字段符号，并断言两条 item **互不相等**（**不是**「总数不变」）。**该断言放在根 crate 而非 `crates/jarde-java/tests/`，因为恢复层不得依赖 query 层**（`cargo tree` 复验 0 次）。

### bridge 的边界

**呈现**：`ACC_BRIDGE` + body 恰为「参数槽按序载入 → 一次转发 → 对被转发返回值的 `checkcast`（类型 = 该调用描述符声明的返回类型）→ return」。**不呈现**：参数上的 cast（`_cast_not_erasure`）；body 不只转发（`_shape`，**但成员本身照旧按通用结构呈现**，多出来的语句照样写成语句）；类未声明 bridge flag（`_not_declared`——**形状像转发不等于 bridge**）；运行未交 flags（`_flags_missing`）。

### 证伪三组

| 变异 | 结果 |
| --- | --- |
| 段内语句检查改成「一律接受并拥有」 | `a_chain_whose_instance_is_stored_in_a_local_is_refused` 红——实例被存进局部量的链被当成 `+` 写出来 |
| `concat_expr` 改成逆序取操作数 | 顺序用例红（产物 `f() + 5 + "x"`）**且** oracle 对照红 |
| **削弱 oracle**（删掉调用序列比较） | oracle 变异用例红，**其余 23 条（含全部生产用例）全绿** |

### 被修正的既有断言（6 处，均收紧）

`pass.rs` 的 `PASSES.len()` 5→8 与规则表；release pin 的野生 `_ => None` 一支改为**具名一支**（把「为什么与 release 无关」写进断言）；`IrTable` 允许集合增加 `Members` 并新增三条 pin（`ACCESSOR` 确实声明 `Members`、`BRIDGE` 确实声明 `Metadata`、`IrTable::Members.name()`）；`for pass in [...]` 断言扩到新规则；`LOOP/CONCAT` 的 `StatementFree` 与 `Replayable` 声明被逐个 pin；`lambda.rs` 的 `Refusal` 提为共享 `refusal.rs`（机制**复用而非复制**）；`build::value_is_consumed` 的读者集合追加 `Field`/`CheckCast`。

### 父级核查：2.2 报告里的一条「既有边界」不成立

2.2 的实现者在汇报 §10 声称：「普通调用臂在值被消费时仍会**同时**写调用语句与消费处表达式（例：被拒绝的访问器写成 `access$200(self);` **与** `return access$200(self);`）」，并说 2.1 的一条用例依赖该行为。

**父级用三个探针实测**（逐个打印恢复文本并数出现次数）：
| 探针 | 文本要点 | 该调用出现次数 |
| --- | --- | --- |
| 被拒的 accessor（`access$100`，门面未交成员表） | `return access$100(local0);` | **1** |
| body 不止转发的 accessor（`access$200`） | `return access$200(self);` | **1** |
| 被拒的 capture（`produce()`） | `produce();` + 引用的站点 + `return;` | **1** |

**结论：该说法不成立于交付代码**——`call_value_reaches_a_reader` 的读者集合与调用臂的 `if write.is_none() && …` 守卫正是为此而设；2.1 的用例断言的是「拒绝站点不丢 effect」（`produce();` 仍在），**不是**依赖重复写。**未据该说法修改任何代码**（只查不改）。
**由它引出的另一个方向**（父级怀疑、已交 2.3 核查）：若消费该值的指令**本身被引用**（例如**非 `bridge@1` 所有**的 `checkcast` 走单 BCI fallback），那么调用臂**没写**语句、读者又**没渲染**值——**effect 可能从呈现里静默消失**（方向相反，更坏）。已在 2.3 的派单里作为**先做的核查项**。

### 证据

全量 **925 passed / 0 failed / 1 ignored**（894 + 31：`jarde-java` 单元 43→49、新 `p3_patterns` 24、新根测试 `p3_accessor_edges` 1；**既有用例一条未增未删**）；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；**未触及依赖边**，锁文件两条均通过。
**CI**：`492e31e` → run 35446581536，四 job success。

### 已知边界（留给后续）

**门面未接成员表**：`Engine::recover_method` 明说不做第二次类读，故经门面对访问器调用给 `jre_accessor_members_missing`（被陈述的拒绝，文本保留调用）。呈现这一半在库层（`ClassMembers` + `RecoveryRequest::with_members`）已可用；把成员读接到门面与 3.1 的 receiver/参数命名是**同一件事**，属 **3.1**。其余：`StringBuilder` 之外的拼接类、嵌套链、方法转发型 accessor 的呈现、`altMetafactory` 非零标志位、段表 `cp` 面与新 `*Record` 的 `Deserialize` 面（3.2）。


<a id="review-2026-09-19-recovery"></a>
## 2026-09-19 恢复实现与路线复核

代码基线 `bc283c0`，收尾纳入 `492e31e` 的 2.2 实现及 `51a5cac` 完成记录。两版均在 git archive 隔离副本核对；主工作区另有 agent 的 build/模式测试改动，不作为已关闭证据，本轮未修改生产代码或仓库测试。P2 29/29 与分层 7/7 已由 `7a5f994` 归档，原 R9/R10 修正及 P2 门禁不再是待办。P3 历史 5/11 完成，新增 1.3d 后 **5/12**。

### P3-R1 · P1：跨 local 写入的旧值被重读 → 1.3d

用 OpenJDK 23.0.1 的 `javac --release 8 -g:none` 编译 `public static int post(int x) { return x++; }`，真实字节为 `1a 84 00 01 ac`（iload_0; iinc 0,1; ireturn）。公开 `Engine::recover_method` 在 `bc283c0` 和 `492e31e` 都输出：

```java
{
    local0 = local0 + 1;
    return local0;
}
```

结果为 Java/Structured/Unchecked，生产语义平面为 Unproven。将生成的 body 原样包入 `static int post(int local0)` 后编译/执行：原方法 `post(7)=7`，生成方法 `post(7)=8`。原始读取已留在 operand stack，`build::render_value` 却把其 `Operation::Load` 重新写成当前 slot 名；这是值语义错误，不是缺少 debug 或未执行编译。应保存该 SSA 值并保持必要的物化与顺序；只在 lambda capture 上使用 replayable 守卫不足以保护通用路径。

### P3-R2 · P1：消费者 fallback 时调用生产者消失 → 1.3d

真实 Java 输入：`static Object make(){ calls++; return "ok"; }`、`public static String cast(){ return (String)make(); }`。`492e31e` 的公开恢复结果为 Mixed/Fallback，文本只引用 checkcast 的 BCI 3 和 areturn 的 BCI 6；没有 `make()`，也没有生产者 BCI 0 的低级引用。`call_value_reaches_a_reader` 把 CheckCast 归为可渲染消费者，调用臂因此不发射语句；非 bridge 所属 cast 又只发射自己的 fallback，消费处没有兑现生产者值。

降级可以不生成 Java，但必须保留完整 effect/来源，不能以 Mixed 掩盖生产者丢失。修正需要决定“哪些值最终由哪个实际产物消费”，或将依赖生产者合并进 fallback，不能只用 opcode 名单删语句。该反例与 `51a5cac` 中怀疑的方向一致，本轮已从公开入口复现；后续工作区修正未计为关闭。

**已关闭对照**：`bc283c0` 对 `return tick()` 输出 `tick(); return tick();`，受控执行原方法返回 1/调用 1 次，生成方法返回 2/调用 2 次。`492e31e` 同一反例只输出 `return tick();`，本轮已复测关闭；不把这个旧缺陷继续列为当前重复调用。保留单次调用回归，避免修 R2 时重新引入它。

### P3-R3 · P2：分支声明作用域不成立 → 3.1

真实 `public static int scope(boolean b){ int x; if(b) x=1; else x=2; return x; }`，字节 `1a 99 00 08 04 3c a7 00 05 05 3c 1b ac`。公开输出为 Java/Structured/Unchecked：

```java
{
    if (local0 != 0) {
        int local1 = 1;
    } else {
        local1 = 2;
    }
    return local1;
}
```

即便给测试包装提供 `int local0` 以单独排除条件类型问题，`javac --release 8` 仍在 else 和 return 两处报 local1 不可见。`build::declare` 的全方法已声明集合不代表 Java 词法作用域。将已有 3.1 提前，按定义/使用与 Region 选择声明位置；同时补同次方法 flags、receiver/参数槽与 boolean/category-2 类型边界，不把普通分支的正确性留到复杂模式之后。

### P3-R4 · P2：确定性比较包含 elapsed，测试存在假红 → 3.4 先修

`bc283c0` 原始全量首次在 `p3_java_recovery::a_body_without_debug_names_is_named_deterministically_and_invents_no_source_scope`（约 1312 行）失败：两份完整 RecoveryReport 唯一区别为 `usage.elapsed_millis=1/0`。随后重跑通过。恢复层须复用已有递归排除 elapsed 的比较口径，保留全部确定字段/顺序与计费断言，并以字段/顺序变异证伪；不增加可跳过真实差异的宽松比较器。

### 交接与 source map 的现状

- MethodIr 三表、同次 code/CP/bootstrap、jarde-java、门面/CLI recover_method 都已存在，不重复列为待实现。
- 门面仍以 `parameters=0` 交 RecoveryFacts，未交访问 flags/receiver/debug；bridge 所需声明与 accessor 所需 ClassMembers 主要由低层 API 的调用方补充。把这部分记为底层实现，不声称公开入口已能呈现所有 2.2 模式。
- 当前 source-map Origin 只有 bci/cp/provenance，没有物理方法身份；accessor callee 的字段 BCI 与 caller BCI 无法由 anchor 自身区分。3.2 复用既有物理身份并绑定 CP，不能新增同义身份体系。
- “读当前方法声明”与“读 accessor 的另一个 Body”是两类需求。后者按实际候选请求并计费、绑定定义/CP；普通恢复不读无关 Body，不能为接线预装全类成员。公开成功呈现与 X1 两边保真应在同一验收中发生。

### 本轮验证与调整

- `bc283c0` 首次原始测试：因上述 elapsed 用例失败，默认 fail-fast 未完成全量；后续重跑 894 条既有回归通过、1 ignored（同次另有 1 条隔离探针，单独计数）。不以重跑绿色抹掉假红。
- `492e31e` 原始固定副本（排除审计探针）：`cargo test --workspace --all-targets --all-features --locked --no-fail-fast` = **925 passed / 0 failed / 1 ignored**。
- 四类公开入口探针均经真实 javac CLASS 和 reader/IR/恢复管线；返回值、调用次数与作用域的受控对照使用 OpenJDK 23.0.1，只证明这些 fixture，不替代 JDK 25 reader oracle 或完整 P3 语料。
- 新路线：**1.3d → 3.1 → 3.2 → 2.3/2.4 → 3.3/3.4**；elapsed 比较先修，现有 2.2 勾选保留。P4/P5 不扩大，P2/分层档案不改写。完成记录、规格/入口文档、能力与公开边界已按同一状态修订。
- 文档校验：`openspec validate --all --strict --no-interactive` **12 passed / 0 failed**；本轮修改文档的本地路径与锚点检查通过，`git diff --check` 干净。补正当前入口和 fuzz 文档的 P2/分层归档链接，保留归档文件原记录。最终核对 HEAD 仍为 `51a5cac`；其他 agent 未提交的 2.3 源码继续保留，不纳入本次验收计数。

## 2026-09-19 1.3d：跨写入的旧值重读修复（提交 `1e6521e`）

修复复核的 **P3-R1**（P1）并为 **P3-R2** 落下真实语料回归。**R4 已在 `b8ba342` 先行修复**（见下）。

### 根因（父级用 SSA 探针定位，实现者复核一致）

`post(int x) { return x++; }`（`1a 84 00 01 ac`）的 SSA **本来就是对的**：

```
bci 0 iload_0  reads [(Local(0), V0)] writes [(Stack(0), V1)]
bci 1 iinc 0,1 reads [(Local(0), V0)] writes [(Local(0), V2)]
bci 4 ireturn  reads [(Stack(0), V1)] writes []
V1 uses = [bci 4]（返回的正是加载的旧值）；V2 无人使用
```

**缺陷纯在渲染**：`render_value` 对 `Definition::Instruction` + `Operation::Load{slot}` **无条件**把槽名当表达式。而槽名在**使用点**表示的是该槽**当时**的内容（此处 `V2`），不是该 load 所表示的 `V1`。

### 修法

新增 `Builder::slot_name_denotes_the_same_value(slot, denotes, at) -> bool`：在**含使用点 `at` 的 block** 里，取 `entry()` 中该槽的值，再按 `instructions()` 顺序把 `bci < at` 的每次对 `Local(slot)` 的写入覆盖上去（最后写者胜），**判等 `denotes`**。
- `denotes` 的三种来源：`Load{slot}` = **该 load 自己 `reads()` 里的 Local 值**；`Entry{slot}`/`Phi{slot}` = 该 value id 自身。
- `at` 语义未改（仍是**使用点** BCI）；`at` 不在任何 block 时返回 `false`（无证据不写名）。
- 不成立时返回 `Err` 走既有 `fallback`，**绝不退回写槽名**；新增 `deferred_producers(value, reader, …)` 走查，使拒绝**同时**陈述 load 的 BCI 与使用点（R2 的原则：降级可以不出 Java，但 effect/来源必须完整）。

### 实现者发现的两处父级诊断之外的情形（**均有执行对照**）

| 形态 | 修前产物 | 执行对照 |
| --- | --- | --- |
| `int y = x++`（store 消费点） | `int local1 = local0;` | 原 7 / 修前 **8** |
| `if (x++ > 0)`（分支消费点） | `if (local0 > 0)` | `conditional(0)` 原 0 / 修前 **1**（**改变走哪条臂**） |
| `post`（return 消费点，即用户反例） | `local0 = local0 + 1; return local0;` | 原 7 / 修前 **8** |

**修后**（`post` 的公开产物）：
```java
{
    local0 = local0 + 1;
    // @bytecode 4 0
    // the value at BCI 4 is the value local 0 held at BCI 0, and the slot does not hold it at BCI 4: the slot's name would read the value the body wrote in between
}
```
即**不再声称有 return**、平面降为 `Mixed/Fallback`——不完整但诚实（把旧值物化成临时变量从而**呈现**而非降级不在本片）。

### 必须保持的反面（对照组）

`bump`/`doubleIt`（真写后重读 → 仍写 `return local0;`）、`loopAcross`（**跨块**：循环体改了槽，循环后仍写 `return local2;`）——修后**完整呈现、零引用**；执行对照 `doubleIt(7)=14`、`loopAcross(7,3)=7` 与原方法同值。

### 父级独立证伪（两个方向，副本 + `sha256sum -c`）

| 变异 | 结果 |
| --- | --- |
| 判据**恒真**（恢复修前行为） | **恰好 3 红**：`a_value_the_slot_no_longer_holds_is_not_returned_through_the_slot_name`、`a_store_of_a_superseded_load_is_refused_with_the_read_named`、`a_branch_on_a_superseded_load_is_refused_with_the_read_named` |
| 判据**恒假**（一律拒绝） | **恰好 3 红**：两条对照组 + store 的呈现面 |

**真值在中间**，两个方向各有测试承重——这正是「不是一律拒绝」的证明。

### fixture 与可复现性（父级独立复核）

`tests/fixtures/p3-local-rewrite/`：真实 `javac 23.0.1 --release 8 -g:none` 输出，8 个成员（`post`/`saved`/`conditional` 为缺陷面；`bump`/`doubleIt`/`loopAcross` 为对照面；`cast`/`make` 为 R2）。README 记录编译器、命令、逐成员字节码。
**父级用提交的源文件自行重编译**：输出与提交的 `.class` **逐字节相同**（均为 `f755f062bc9779d941e93bf1ef4889b3ce6efb0dbd670e5ccbd07152126158a6`，650 字节，major version **52**）——fixture 确实由所提交源码产生，非手工拼装。

### 证据

全量 **953 passed / 0 failed / 1 ignored**（947 + 6，全部来自新测试目标 `p3_local_rewrite`；既有目标无状态改变）；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；**未触及依赖边**（故未跑 fuzz 锁检查）。
**CI**：`1e6521e` → 见下。
**唯一被修正的既有断言**（收紧而非放宽）：`classfile.rs::repository_class_fixtures_validate_without_false_target_rejections` 的计数 `(15, 42, 8, 8, 8)` → `(16, 51, 8, 11, 8)`——新增第 16 个 fixture class（9 个有体成员、3 个新分支目标），断言仍是同一个严格相等。

### 尚未闭环的边界（如实）

- `Entry`/`Phi` 的 `Local` 分支在本切片**没有可达路径**（`render_value` 只被喂 stack 值）——判据在这两处是同一不变量的**补全**，但**未经实测触发**。
- 本片只做到「不再产出错误的 Java」；把旧值**呈现**为 `x++` 或临时变量属后续模式工作。

## 2026-09-19 3.1：声明事实与作用域（提交 `f7f90b7`）

关闭复核的 **P3-R3**（P1/P2）并接线**方法声明事实**。

### 第 1 部分：声明事实从**同一次** header 读取进载荷

- `jarde-jvm/src/method_ir.rs` 新增 `MethodDeclaration { access_flags, name, descriptor, parameter_slots, identity }`，作为 `MethodIr` 的**私有字段** `declaration: Option<Box<MethodDeclaration>>`，只给 `Option<&…>`（与 `code()/constant_pool()/bootstrap_methods()` 同形）。
- 取值点：`engine.rs::read_driver_method` 里 `read_own_definition(…, HeaderDemand::DriverMethodBody)` 的**唯一一次** header 读取所定位到的 `member`；`parameter_slots` 由描述符 + `ACC_STATIC` 推出（**receiver 计 1、`long`/`double` 计 2**）。
- **门面 `recovery_facts` 不再固定 `parameters=0`**：改从载荷取 `parameter_slots`/`access_flags`，并把 debug 名一并交给 `RecoveryFacts`。仅当运行没读到成员头时才退回请求身份 + `parameters=0`（唯一保留的诚实「无声明」分支）。
- **没有第二次解码 Body**：LVT 是在 `raw_facts` **已经读过并计费的那个 `Code` 条目内部**用 `code.attributes()` 走嵌套属性表读出的（按名字跳过 `LineNumberTable`/`StackMapTable`/类型注解），**不调用** `code_nested_attributes`（那会再计一次 `AttributeBytes`）、不新增 charge、不新增 poll。
- **父级独立核对**：同一 fixture 改前/改后 usage 逐字段相同（`class_headers=1`、`method_bodies=1`、attribute/code/class bytes、`ir_items`、`analysis_steps` 全等，只有墙钟 `elapsed_millis` 变）——「一次运行，一个来源」成立。

### 第 2 部分：R3（分支作用域）

**判据**（`build.rs::declarations`）：从 SSA 收集该槽**全部使用**（`reads()`/`writes()` 里的 `Local(slot)`）→ 求**包含全部使用的最内层 Region** `R`（`region_paths()` 把 Region 树编成路径，嵌套区域先走、`or_insert` 后写，故 block 归属**最内层**）→ 若**首次写入所在 Region 就是 `R`** 则**保持现有行为**（`int x = <value>;`，`declared` 集合照旧）→ 否则在 `R` **开头**写**无初值**的 `int x;`，所有写入改为普通赋值。
**安全出口**：任一次使用落在 `Region::Fallback`（引用字节码）或 block 未被树认领 → 该槽**不参与**（保持旧行为，不产生正文里没人用的声明）。

**产物**（`tests/fixtures/p3-scope/v8-debug/Scope.class`，真实 javac 23.0.1 `--release 8`）：
```java
{
    int x;
    if (b != 0) {
        x = 1;
    } else {
        x = 2;
    }
    return x;
}
```

**父级独立验收**（**不用实现者的 harness**）：
- 用提交的 fixture 经公开入口打印产物（上面那段，含真实 debug 名 `x`/`b`）；
- 自行包成 `static int scope(int b)` 并用 `javac --release 8 -g:none` 编译 → **exit 0**（仅「源值 8 已过时」警告）；
- 自行执行对照：`b=1 generated=1 original=1`、`b=0 generated=2 original=2` —— **同值**。
- **修前**（实现者用 HEAD 产物复现）报 `找不到符号: 变量 local1` 于 **else 与 return 两处**，与复核记录的位置一致。

**代价说明（如实）**：`Scope.scope` 的参数是 `boolean`，而产物写 `b != 0`——该比较只对 int 型原语成立。**复核自己就排除了这一项**（其 R3 原文写「即便给测试包装提供 `int local0` 以单独排除条件类型问题」），故本片按同一口径验收；根因是 LVT 的 `descriptor` 未用于类型、参数类型来自 frames（四个 int 型原语同形），属**既有边界**，已在 verification 与本片记录中点名。

### 第 3 部分：稳定命名与三类覆盖

- **slot 复用**：选「命名与声明都成立」——本层命名的是**槽**，SSA 给出该槽唯一的定义/使用链，故一个存储位置呈现为一个变量：声明提升到两臂之外的共同区域、两臂各自赋值。**带 debug 时该槽被 LVT 命名两次**（`c` 与 `d`）→ 如实给出**无名字**（落回序号名），不伪造。
- **category-2**：`after(JI)I` → `{ return arg2; }`（`long` 占两槽，`arg2` 是 `int` 参数）、`receiver(J)J` → `{ return arg1; }`；并断言**参数绝不作为局部声明**（`!text.contains("int ")`）。
- **无 debug**：确定性序号名，不伪造源码作用域；有 debug 时用真实名。
- **A16** 未退化：改前/改后 usage 逐字段相同（一次 header + 一次 body）。

### 父级独立证伪（两个方向）

| 变异 | 结果 |
| --- | --- |
| `declarations()` 返回空（**无提升** = 修前行为） | **4 红**：`a_slot_written_in_both_arms_is_declared_where_both_can_see_it`（R3）、`an_arms_own_local_is_still_declared_inside_that_arm`、`two_variables_sharing_one_slot_are_declared_once_where_both_are_visible`、`a_table_that_names_a_slot_twice_states_no_name_for_it` |
| 「就地」分支改为恒假（**一律提升**） | **2 红**：`a_slot_filled_and_read_in_one_region_keeps_its_initial_value`、`an_arms_own_local_is_still_declared_inside_that_arm` |

真值在中间——两个方向各有测试承重。

### 被修正的既有断言（6 处，无一条放宽）

`p3_java_recovery.rs` 的 `an_if_else_…` 与 `a_tableswitch_…`：`find("int local2 = 1;")` → `find("local2 = 1;")` 并**新增**「声明在 `if`/`switch` 之前」的断言（这两个 fixture 的槽在两臂都写 → 按新规则提升，**原本在臂内声明，正是 R3 缺陷**）；`tests/p3_local_rewrite.rs` 5 处 `local0` → `arg0`（参数槽现在由载荷声明命名，**禁止的内容一字未减**）；`tests/p3_recovery_entry.rs` 的 `return local1 + local2;` → `return arg1 + arg2;`（`add(II)I` 是实例方法，slot 0 是 receiver）；fixture 普查计数 `(16, 51, 8, 11, 8)` → `(18, 67, 8, 23, 8)`。另两处是**测试代码**改动（`oracle.rs` 的文本模型新增 `Text::Declare`，读到未赋值局部**仍然 panic**，是加强；`from_parts` 增参数，6 处机械补）。

### 证据

全量 **960 passed / 0 failed / 1 ignored**（953 + 7，全部来自新测试目标 `p3_scope`；**既有目标无状态改变**）；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；**未触及依赖边**（另跑 fuzz 两条均 OK）。
**fixture 可复现性（父级独立复核）**：用提交的 `Scope.java` 自行 `javac --release 8 -g:none` 重编译 → 与提交的 `.class` **逐字节相同**。
**CI**：`f7f90b7` → 见下。

### 未做（如实，属 3.2 或更后）

`MethodParameters` 未读（方法级属性，不在 `Code` 条目内，读它需为一类新输入新增 `AttributeBytes` 计费——按派单条件不做「顺手改口径」）；LVT 的 `descriptor` 未用于类型；LVT range 未作锚点；`LocalDebugTable::Unstated`（声明了但内容解不出）无 fixture；**`access_flags` 已进载荷并交给 `bridge@1`，但仓库无声明 `ACC_BRIDGE` 的样例**，故「门面 → bridge 规则」无端到端 fixture。
**另发现一处既有缺口（非本片引入）**：`if` 的 else 臂为空、跳转目标就是 join 的形状会被 `region.rs` 判成 `arms_do_not_meet` + `uncovered_blocks` → 整段引用；fixture 因此改成两臂都非空。记录在此，不在本片范围。

## 2026-09-19 附加发现（父级独立验收 3.1 时测得，**不在复核的 R1–R4 之列**）

**P3-R5 · 候选：`boolean` 参数被写成整数比较，产物按方法自己的签名无法编译。**

- **复现**（父级用提交的 fixture 自行执行）：`tests/fixtures/p3-scope/v8-debug/Scope.class` 的 `scope(Z)I` 经公开入口产出
  ```java
  int x;
  if (b != 0) { x = 1; } else { x = 2; }
  return x;
  ```
  把这段原样包成 `public static int scope(boolean b)` 后 `javac --release 8 -g:none` **报错**：
  ```
  Gen.java:4: 错误: 不可比较的类型: boolean和int
          if (b != 0) {
  ```
  包成 `int b` 则编译通过且执行同值（`b=1→1`、`b=0→2`）——**即当前产物只对 int 型原语成立**。
- **为什么不算 R3 的失败**：复核自己就排除过该项（其 R3 原文：「即便给测试包装提供 `int local0` 以**单独排除条件类型问题**」），本片正是按同一口径验收的；R3 的作用域缺陷**确实已修**。
- **但它是什么**：产物在方法**自己的签名**下无法编译，而报告此时仍写 `representation=Java` / `syntax_status=Checked`（若 `Checked` 由本层断言）——即**声称了比证据更强的可编译性**。`boolean`/`byte`/`char`/`short` 四个 int 型原语在 frames 里同形，故类型信息只能来自**描述符或 LVT 的 descriptor**，而两者当前都未用于参数类型（3.1 的如实边界）。
- **建议归属**：**3.2 之后、3.3 之前**（3.3 要求「把 P3-R1/R2/R3 纳入可重放的实际 Java 8 编译/执行对照」，本项天然属于同一对照组）。两条可行方向：①用**同次运行的声明事实**（3.1 已把 `descriptor` 放进载荷）给参数定型，`boolean` 参数写 `if (b)`；②无法定型时**拒绝**（`syntax_status` 不得声称 `Checked`）。
- **未擅自实施**：用户的 tasks 未列本项，且 3.1 的边界是用户复核时明确排除的；父级在此**只记录证据**，不改 tasks，等复核裁决。

## 2026-09-19 3.2：物理身份锚点与按需 callee 证据（提交 `4488170`）

用户要求的四条逐条落地：**锚点带物理身份**、**JVM 事实层按需提供 callee 证据**、**接通门面/CLI 的成功呈现与拒绝**、**A12 用同一次公开入口验收 + Mixed 映射/诊断**。

### 锚点的形状（第 1 步）

`source_map::Origin` 扩为 `{ bci, method: Option<Box<PhysicalMethodId>>, cp, provenance }`——**复用既有身份类型**（`Origin::member()` 直接给出 reader 的 `OriginMember::MethodPoint{method, bci}`），**未新立同义体系**；`Direct`/`Derived` 的语义与取值**一字未改**。
身份由两处写入：build 在 accessor 的 derived 锚点上写 **callee** 的身份；emitter 在记录 segment 的那一次写入里给未声明成员的锚点补上**被呈现方法**的（来自 payload 的 `declaration().identity()`）。
实现中发现 `clippy::large_enum_variant`（按值内联让每个 AST 节点涨约 200B），按 lint 建议改为 `Box` 并写明理由——节点尺寸回到原状。

**四类区分用例**：caller/callee **同号 BCI** 与「两个 callee 同字段 BCI、不同成员」（`access$200`→`g`、`access$300`→`h`，两者 `getfield` 都在 BCI 1，而调用点在 `both` 里是 BCI 1 与 5）；**canonical clone**（自造 v45 `jsr`/`ret` 体，`normalization_clones >= 1`，引文把 BCI 14 按两个入口写两次而锚点只持一个坐标——**锚点集合 == 引文坐标集合**）；**CP/attribute**（lambda 站点锚点的 `cp()` == 站点 CP 项，`method()` 指明该索引所属成员体）。

### callee 证据（第 2、3 步）

新 `crates/jarde-jvm/src/callee.rs`（`pub mod callee`，入口 `read_callees`）产出 **jvm 自己的**只读类型（`CalleeCandidate`/`CalleeMember`/`CalleeBody`/`CalleeRefusal`/`CalleeReadReport`），字段全私有、只给只读访问器。
- **分层**：`ClassMembers` 定义在 `jarde-java`，**jvm 造不出来**——由**门面**（同时依赖两者）用 `member_table()` 逐成员适配；语义判定仍只在 `jarde-java::accessor`。`cargo tree` 的 12 个配置全 PASS，`jarde-reader`/`jarde-query`/`jarde-jvm` 闭包中 `jarde-java` 出现 **0** 次（父级复跑确认）。
- **候选来源**：被呈现体**自己的 decode** 里的 `invokestatic` 调用点，过滤谓词**就是** `accessor@1` 的 `names_an_accessor`（`verify` 与 `accessor::candidates` **共用同一函数**）。
- **计费与 reason**：1 次 `ClassHeaders`（同一物理定义的第二次头读——payload 只带该体的表、不带字节）+ 每个**真正声明且带体**的候选 1 次**先扣费** `MethodBodies` + 该体自身的 attribute/code 字节；无体成员**不扣费**。新 `HeaderDemand::CalleeMemberBody` → `ReadReason::CalleeMemberBody`。
- **绑定同一物理定义/CP**：定义取自 payload 的 `declaration().identity().owner`，按 identity（digest+length 校验、`this_class` 在声明 loader 下绑定）读取；调用点 owner 与 `this_class` 不同即 `callee_not_this_class`；每个成员的 `identity` 都写该定义。
- **接线位置**：`Engine::recover_method` 内、**分析与呈现之间**；`analyze_method_ir` 仍**只跑一次**，无候选时连头都不读；恢复层**没有**反向依赖 jvm 的读取入口。

### A12 同次公开入口验收（父级独立核对）

同一次 `recover_method` 的产物：
```java
{
    return arg0.g + arg0.h;
}
```
- 文本**含直接字段表达式**、**不含** `access$200(`/`access$300(`；
- 锚点：`arg0.g` = (BCI 1, `both`)+(BCI 1, `access$200`)；`arg0.h` = (BCI 5, `both`)+(BCI 1, `access$300`)——**同号 BCI 由成员身份区分**；
- **按需（A16）**：`method_bodies == 3`（1 个被呈现体 + 2 个被调用点点名的成员），而 fixture **声明 6 个带体成员**——`access$400` 一个体都没读；`analysis.reads.len() == 1` 不变；
- **X1 两条边逐项不变**：`method`@3→`access$100`、`access$100`@1→`f`，`elapsed_millis` 归零后逐字段相等，且两 item **互不相等**；
- **拒绝半边**：`foreign_call_class`（`Test` 自己也有同名同描述符 `access$100`，调用点却指向 `Other.access$100`）→ `callee_not_this_class`、`members` 空、`method_bodies=1`，规则以 `jre_accessor_not_a_member` 拒绝，文本保留 `access$100(arg0)` 且无 `.f`；
- **CLI**：`callees` 字段与库序列化逐字段相等（文本 + 成员 + reason + usage）。

### Mixed 映射与诊断（第 4 步）

`Builder::fallback` 现在把引文写出的**每一个** BCI 都作为锚点（primary = 区域自身，其余为 presented）。
用例断言：`Mixed`+`Fallback` 同时成立；`analysis.coverage.artifact_structural.state == CompleteWithinSchema` 且 `skipped` 为空（**扫描完整，Fallback 未被改写成 Partial**）；每个被引用的 BCI 都有锚点且**引文锚点集合 == 引文 BCI 集合**；每个 fallback 区域都有 `code`+`message` 且 diagnostics 里存在同 code（A12/A13/A16）。

### 父级独立证伪

| 变异 | 结果 |
| --- | --- |
| 放松**物理定义绑定**（`owner != class` 恒假） | **恰好 1 红**：`a_call_to_another_classs_same_named_member_is_not_read_from_this_class`——放松后 `Test` 自己的 `access$100` 被读成 `Other.access$100` 的 callee，**正是复核点名的风险**，证明绑定承重 |

（实现者另跑两组：预装全类成员 → 3 红且 `method_bodies` 涨到 8；锚点丢掉身份 → 3 红，其失败输出正是两个 `method: None` 的锚点相等——即 3.2 要修掉的「无法自证」形态。）

### 被修正的既有断言（4 处，无放宽）

`tests/p3_accessor_edges.rs`：由「site 被拒（`jre_accessor_members_missing`）+ 文本含 `access$100(`」改为「site **被呈现** + 文本含 `return arg0.f;` + `callees.members()==[access$100]`」——门面现在会读调用点点名的成员，**成功呈现正是本片目标**；**X1 的断言一条未动**。另三处是机械适配（`MemberBody::new` 增身份参数、`emit(...)` 增参、CLI 测试助手拆分）。

### 证据

全量 **968 passed / 0 failed / 1 ignored**（960 + 8：`p3_accessor_edges` 1→4、`p3_patterns` 40→42、`p3_local_rewrite` 6→7、`json_cli` 13→14、`jarde-java` 单测 55→56；其余每个 binary 与基线**逐项相同**）；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0；**未触及依赖边**。
**CI**：`4488170` → 见下。

### 未完成项（如实）

`CalleeBody` 的线上摘要是**新 schema**（`instructions`/`code_bytes`），尚无文档行；新公开面（CLI `callees` 字段、`ReadReason::CalleeMemberBody`、`AccessorRecord.callee`、`MemberBody` 的身份/无体、`Origin::method`/`member()`、`into_parts` 三元组）需要文档同步（属 3.4）。本片**未做独立 review**。

## 2026-09-20 2.4：TWR / synchronized / finally，并关闭 P3-R5（提交 `d4c3901`）

### TWR（`twr@1`，`crates/jarde-java/src/guard.rs`）

walker 遇到**异常边**时先于既有 fallback 检查。逐条证明（全部读**同一次运行**的载荷）：

| 事实 | 证明方式 |
| --- | --- |
| 资源初始化在 `try` 之内 | 每个资源 = 一段**指令区间**，从 `L_j.start_bci` 反向生长到「恰好一条语句」（除 store 外每条指令的写入都须被区间内读；store 的读值须由区间产生），且不越过上一层起点 |
| 正常路径 close | 指令级匹配 `aload slot; ifnull L; aload slot; invokevirtual close()V; goto L` + CFG 核（ifnull 块后继恰为 close 块与 `L`；close 块唯一后继为 `L`）；**close 的接收者必须是那次 `aload` 产出的同一 SSA 值** |
| **close 顺序** | 从最内层区间末尾起，把 close 链与 `resources.iter().rev()` **逐个匹配**；不符或漏一即 `jre_guard_close_order` |
| handler 覆盖与 catch 类型 | `row.start_bci == 该层首指令`、`row.end_bci ==` 内层 handler 跨度末期；`catch_type_index` **不作放宽**，靠 handler 形状判定 |
| **`addSuppressed` 接收者是 primary** | handler 必须是 `Store{e}; Load{primary}; Load{e}; Invoke(addSuppressed)`——**两个 load 的顺序就是证明**；反向即 `jre_guard_suppressed` |
| `athrow` 重抛 primary | handler 尾必须 `Load{primary}; Throw`，否则 `jre_guard_primary` |
| 不可解释的 row | 语句跨度内**每条**被保护的指令，其 row 必须在 `twr@1` 自己的 row 集合内 → 带 `catch` 的 TWR 拒绝（`jre_guard_unexplained_row`） |

呈现：`try (Res local0 = open("r")) { body(); }`；**证明读过的 BCI（两个 close、`addSuppressed`、`athrow`、primary 的 store）全部进 derived origin**。

### synchronized（`monitor@1`）

全方法**恰好一个** `monitorenter`；header 必须是「锁值表达式 + 一个 store」，且 store 填入的值与 enter 读的值同一（`dup; astore; monitorenter` 的惯用法用「同一指令产出的两个输出」证明）；全方法**恰好两个** `monitorexit`（正常 + handler）；handler 必须是 `Store{p}; Load{lock}; monitorexit; Load{p}; Throw` 且其 monitorexit 被它**自己那条 row** 保护。**异常路径缺 exit / 第二入口 / 只在部分路径 exit 一律拒绝**——不输出会漏掉异常路径退出的 `synchronized`。

### finally：**如实拒绝**（本片的判断，附理由）

`fin()`/`catchFinally()` 落在新码 **`jre_guard_finally_copy`，`rule: None`**（没有任何规则声称它）。理由：javac 为 `finally` 生成的是**复制**（正常路径一份 + 异常路径一份）；把两份并成一份 `finally` 只在两份**可证等价**时才忠实，而证明它需要比较本层未建模的操作数并证明**每条 exit 恰好跑一份复制**——**执行次数正是风险所在**。故引用并把 BCI 报全，诊断说明原因。**`jsr`/`ret`（v45）既有行为未变**（仍 Mixed/Fallback）。

### P3-R5 关闭（父级此前记录的候选缺陷）

参数类型改由**方法自己的 descriptor**（同次运行的声明事实）解析：零测试条件写 `if (b)`（`ifeq`→`!b`、`ifne`→`b`；其它零测试对 boolean 无 Java 拼法 → **拒绝**），从 boolean 参数填充的局部声明为 `boolean`。
**父级独立验收**：取 `scope(Z)I` 的产物自行包成 `public static int scope(boolean b)` → `javac --release 8` **exit 0**（此前同一包装**编译不过**，报「不可比较的类型: boolean和int」）；执行 `b=true → generated=1 / original=1`、`b=false → generated=2 / original=2`。
**顺带澄清**：`SyntaxStatus` 本层**从不声称 `Checked`**（只在有别名时 `NotJava`，否则 `Unchecked`），故 R5 记录里那条「可能仍称 Checked」的担忧原本不成立——用例钉住 `Unchecked` + `compile_status=NotAttempted`。

### 父级独立证伪（两组，均与实现者自报一致）

| 变异 | 结果 |
| --- | --- |
| close 链改为**声明序**匹配（不再逆序） | **2 红**：`three_resources_are_declared_in_the_order_whose_closes_run_backwards`、`a_resource_initialised_after_an_earlier_one_is_inside_the_region_that_closes_it` |
| **suppressed 关系不检查** | **恰好 1 红**：`a_suppression_that_is_the_other_way_round_is_refused`，失败输出 `left: Java / right: Mixed`——即反向抑制的类被**错误呈现**成 Java |

### 证据

全量 **982 passed / 0 failed / 1 ignored**（968 + 14：`p3_guard` 13 + `p3_scope` 1；**既有目标逐项不变**）；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；**未触及依赖边**（故 fuzz 锁两条不适用）。
**执行对照**（实现者）：TWR/monitor 的 7 个成员产物包成 `Gen extends Guarded` 编译后与调用原成员的 runner 逐行比对 **IDENTICAL**，含 `two`/`three` 的**逆序 close**（`close s` → `close r`）与 `suppressed` 的 `caught boom` + `suppressed close-r`。
**CI**：`d4c3901` → 见下。

### 未完成（如实）

- **3.3**：「产物 → 包装 → javac → 执行 → 事件流比对」目前是 shell 里的对照（与 3.1 先例一致，仓库约定测试不依赖 javac）；做成**可重放门禁**属 3.3。
- **3.4**：本片新公开面需文档行——`jarde_java::guard`（`GuardPlan`/`GuardShape`/`GuardResource`）、`FallbackReason::Guard`、`Operation::Monitor`/`Throw`、`StmtKind::Try`/`Synchronized`、`ExprKind::Not`、`Ast::ResourceDecl`、`MethodFacts::parameter_types`、`jre_guard_*` 15 个诊断码。
- **既有平面缺口（非本片引入，已点名）**：builder **内部**的引用（如以类字面量为锁值的 `syncBody`）使产物 `Mixed/Fallback`，但其原因只写在**产物文本**里、不进 `report.diagnostics`（诊断平面目前只收 region 级 fallback）。该形状被保留为边界用例。
- **已知边界**：守卫语句必须**起始于其所在节点首指令**（否则更早的语句会被 claim 后丢弃，由 `explained()` 拒绝）；带 `catch` 的 TWR、分支体、`return` 值跨 close 的形状均拒绝，不做部分呈现。
- 本片**未做独立 review**（父级已做两组独立证伪）。

## 2026-09-20 父级发现：slot 复用与既有 spec 的冲突（**待裁决，代码未动**）

**P3-R6 · 候选：`specs/source-maps/spec.md` 的 `Slot reuse across ranges` 场景与 3.1 的实现相反。**

**规格原文**（`specs/source-maps/spec.md`，**该场景是该 delta 的既有内容，不在本轮用户新增之列**——父级用 `git diff` 核过，用户新增的是 `A local is defined in both arms and used after the join` 与 `BCI-to-source evidence` 的物理身份句）：

> **WHEN** 同一个 local slot 在不同 BCI 区间承载不同变量
> **THEN** source map 和 Java AST **建立不同作用域/名称**，**不把整个 slot 合并成一个变量**

**实现的实际行为**（3.1 `f7f90b7`，由 `tests/p3_scope.rs::a_table_that_names_a_slot_twice_states_no_name_for_it` 的注释与断言固定）：

> 「a slot the table names **twice** … takes **no** name, because **one storage location has one name** and neither record's name is the truth about the whole of it」

即：**把整个 slot 当作一个变量**，带 debug 时因 LVT 给出两个名字而**放弃命名**（落回 `local3`）。

**证据（父级本机实测）**：`tests/fixtures/p3-scope/Scope.java:57` 的 `reuse(ZI)I`，其 `-g` 样本的 LVT 为

```
Start  Length  Slot  Name   Signature
    8       2     3     c   I      <- then 臂
   10       3     2     a   I
   17       2     3     d   I      <- else 臂
    0      21     0     b   Z
```

**slot 3 在两个不相交区间 [8,10) 与 [17,19) 上分别叫 `c` 与 `d`**——这正是规格场景说的「不同 BCI 区间承载不同变量」，证据**就在同一个 `Code` 属性里**（3.1 已从它读出名字，只是没把 range 用于作用域）。

**影响**：规格句是 **MUST 级**（「建立不同作用域/名称」「不把整个 slot 合并成一个变量」），当前实现**两处都不满足**；而 3.1 已按此行为勾选。按用户「只有实际完成并验证的任务才能勾选」的规则，**3.1 的勾选在该点上不成立**。

**根因（父级自己的派单失误）**：父级给 3.1 的派单写的是「**slot 复用**：…**判断并说明**你选哪种」——**在该点上授权了自由选择，而规格早已明确要求**。实现者的推理只权衡了「一个变量」与「拒绝整具身体」两种，**没有考虑规格的第三种（按 range 分成两个作用域变量）**，因为派单没让它去读那条场景。

**可能的解法（两点都有证据支撑）**：
- ① 有 LVT 且 range **不相交**、名字**不同**时：按 range 切分该槽的使用，**在各自 range 的合法作用域内**声明两个变量（`int c = …;` / `int d = …;`），source map 的位置仍绑同一物理方法的 BCI；
- ② 无该证据（`-g:none`、range 相交、名字相同或只有一条记录）：**保持现状**（一个槽一个变量）——这与 `Deterministic names and scopes` 的「缺失证据时 MUST 使用稳定 `argN`/`localN`」一致。

**归属**：属 **3.1 的 `slot 复用` 覆盖项**（任务文本本就点名「覆盖 slot 复用」），应在 **3.4 的「P3 所有出口通过」之前**关闭。

**未擅自实施**：改动会变更既有断言（`two_variables_sharing_one_slot_…` 与 `a_table_that_names_a_slot_twice_…` 的期望值）与 `NameTable` 的「一槽一名」合同，属规格级取舍；父级在此**只记录证据**，等复核裁决（或由父级按规格直接实施，两者都已备好证据）。

## 2026-09-20 父级发现：ECJ v52 `finallyPath` 的 handler 未被交代（**待裁决，代码未动**）

**P3-R7 · 候选：类文件声明的异常 handler 的指令既不属于任何块、也不在 `unreachable` 里，而产物报 `Java/Structured`。**

**证据（父级本机实测，探针已删）**：`tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class` 的 `finallyPath(I)I`。

类文件（`javap`）：
```
0: iload_1   1: iconst_1   2: iadd   3: istore_3     // local3 = arg1 + 1
4: iinc 1,2                                             // 正常路径的 finally tail
7: iload_3   8: ireturn
9: astore_2  10: iinc 1,2  13: aload_2  14: athrow      // 异常路径：tail + 重抛
Exception table: from 0 to 4 target 9 type any
```

`MethodIr` 的事实（同一探针）：
```
decoded_bcis = [0,1,2,3,4,7,8,9,10,13,14]        <- 解码**读到了** handler 的四条指令
handlers     = [{start_bci: 0, end_bci: 4, handler_bci: 9, catch_type_index: None}]
canonical    = 单个块 start=0 end=9 blocks=[0]   <- handler 的指令不在任何块里
unreachable  = []                                <- 也不列为死代码
```

产物与报告：
```
representation=Java quality=Structured
regions=[{bci:0, structured:true, blocks:[0], rule: straight@1}]
text:
    int local3 = arg1 + 1;
    arg1 = arg1 + 2;
    return local3;
BCI 9/10/13/14：既不锚定、也不出现在文本里
```

**为什么这是问题（不论 handler 是否真会跑）**：
- `[0,4)` 覆盖的 `iload_1/iconst_1/iadd/istore_3` **都不能同步抛异常**，故按「可达性剪枝」这个 handler 在实践中不会执行——**语义上未必错**；
- **但**：`any` handler 按 JVMS 也承接**异步异常**，且这个剪枝依赖 P2 的 `may_throw` 模型「不漏」——**一旦某条能抛的指令被模型漏标，同一个机制会静默丢掉一个真会跑的 handler**，而产物仍报 `Java/Structured`；
- **可核实的事实**是：类文件声明了 handler、解码读到了它的四条指令，而**产物与报告对它们一字不提**——读者无法区分「判为死代码而丢弃」与「根本没看见」。这正是复核在 R2 立下的原则（「降级可以不生成 Java，但**必须保留完整 effect/来源**，不能以 Mixed 掩盖丢失」）在**块级**的同一形态。
- 对照：**javac** 的同类形状（`fin`/`catchFinally`）已被 2.4 **如实拒绝**（`jre_guard_finally_copy`）。**ECJ 与 javac 两种 `finally` codegen 的口径不一致**，且 ECJ 这一边更宽松。

**归属与两种解法**（父级未擅自实施）：
- **① 最小、P3 范围内**：恢复层在呈现前核对「**类文件声明的 handler 的指令是否被某个块认领**」；未被认领即**引用 + 诊断**（不产更短的 body、不报 `Java/Structured`）。不动 P2 的 canonical 模型。
- **② 更彻底、动 P2 模型**：让 canonical **为每条已解码指令记账**——要么在某个块里，要么在 `unreachable` 里（P2 对 v45/`jsr` 的样本正是这么做的：六节点里三个是死的）。代价是改已归档 P2 交付的剪枝语义，需明确这是一次**取代**而非改写档案。

**父级建议 ①**（P3 自己的不变量，不动归档阶段的行为；且与 R2 的既有纪律一致）。**待用户裁决。**

**注**：3.3 的可重放对照 harness **已检测到**该现象（打印 `note: exception handler entry BCI(s) [9] are named by no anchor`），但把它归为 note 而非失败——本片的对照口径在这一点上偏松，裁决后应改为硬断言。

## 2026-09-20 3.3：可重放的实际编译/执行对照与多代语料（提交 `04cf83a`）

### A：R1/R2/R3(+R5) 现在是**可重放**的

**机制**：`tests/p3_execution_comparison.rs`（**两个 `#[ignore]` 用例**），而非 shell 脚本。理由：对照必须走**公开入口** `Engine::recover_method` 取产物；脚本要么在 shell 里重实现入口、要么自己拼请求 JSON。`#[ignore]` 正是仓库既有的「需要 JDK 的检查」口径（既有 `jvm_bytecode_oracle` 的 JDK 25 oracle 同形）。**不带 `--ignored` 的 `cargo test` 永不执行它**，故没有 JDK 的机器仍绿；`javac` 只编译**本文件现场生成的包装**，CI 的 JDK 版本不会改变任何样本字节。
**CI**：`.github/workflows/ci.yml` **add-only 一处**（在既有 ignored JDK 25 oracle 步骤之后加一步）。这是本片对「改动限于 `crates/**`/`src/**`/`tests/**`」的唯一例外。

**包装由事实派生**（本片新增唯一公开面 `RecoveredMethod::facts()`，`#[serde(skip)]`，CLI 文档逐字段不变）：
```
let facts = recovered.facts();                       // 同一次运行的事实
let stated = facts.method().parameter_types();       // P3-R5 读的那个事实
for (slot, spelling, primitive) in &described {      // 事实与描述符必须一致
    if !primitive { continue; }
    assert_eq!(stated.get(&slot).map(|ty| ty.spell()).as_deref(), Some(spelling.as_str()));
}
```
实跑打印的真实包装：`scope(Z)I` → `public static int scope(boolean b)`（debug 样本）/ `(boolean arg0)`（`-g:none`）；`receiver(J)J` → `public long receiver(long a)`（实例方法无 `static`）；`open(String)LRes;` → `static Res open(java.lang.String arg0)`；`add(II)I`（ECJ）→ `public int add(int arg1, int arg2)`。**修饰符取自 `access_flags()`、类型取自描述符与参数类型事实，无手写类型。**

**对照维度**（一起比，不只比文本形状）：返回值、样本自身打印的事件**顺序**、自身计数器**次数**、异常（含 suppressed）。
实跑：`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture` = **`2 passed; 0 failed`**（13–18s）。
摘要（全表在 `tests/fixtures/p3-corpus/README.md`）：

| 成员 | 产物 | 对照 |
| --- | --- | --- |
| `bump`/`doubleIt`/`loopAcross`（R1 反面） | Java/Structured | **executed，轨迹一致**（必须仍呈现，已钉） |
| `post`/`conditional`（R1）、`saved` | Mixed | 不执行（javac 拒绝）；**次数对照**同 |
| `cast`（R2） | Mixed | 不执行；**次数对照 `calls 1->2` 两侧一致**（R2 实质） |
| `scope`×2（R3+R5）、`after`/`reassign`/`receiver` | Java/Structured | **executed，轨迹一致** |
| TWR/monitor 8 个成员 | Java/Structured | **executed，70 行轨迹逐行相同**（含 `close s→close r` 逆序与 `boom | suppressed close-r`） |
| `fin`/`catchFinally`/`withCatch`/… 7 个 | Mixed（纯引用） | 边界；**每个被拒区域的 BCI 全被引用、每个被引 BCI 都有锚点** |
| `add`/`finallyPath`（ECJ 4.6.1 v52） | Java/Structured | executed，轨迹一致（`finallyPath` 见 **P3-R7**） |
| `Flags` 4 组 flag | Java/Structured | executed，轨迹一致 |
| `MissingDependency.viaAbsentLibrary` | Java/Structured | **javac 拒绝**（缺失类型）；`plain` executed |

### B：多代语料（来源/命令/SHA 在 `tests/fixtures/p3-corpus/README.md`）

- **多代 javac**：本机只有 javac 23.0.1，故覆盖「同编译器不同合法 flag」：`--release 8 -g:none`/`-g`/`-g:lines,source`/`-parameters`/`-source 8 -target 8`（`Flags.class` 325/597/441/395/325 字节，SHA 逐条记录）。
  **记录到的事实**：`--release 8 -g:none` 与 `-source 8 -target 8 -g:none` 产出**逐字节相同**（`212fde26…`，测试内断言）；同一比对在既有 `p3-local-rewrite` 上也得**逐字节相同**（`f755f062…`）。
- **ECJ**：纳入既有 `historical/ecj-4.6.1/v52/HistoricalControlFlow.class`（52.0、303 字节、SHA `f9b6566f…b6ad`）。**如实记录**：本机**不能运行 ECJ**（无 jar、无网络），只读其产物。
- **缺失 debug**：`-g:none` 与 `-g:lines,source` 两组。
- **缺失依赖**：`MissingDependency.java` 对着未提交的 stub 编译，只提交主类；`absent.Library` 不可解析。
- **如实记录不可得**：真混淆（无 ProGuard/R8、无网络）、非 Java 编译器（无 `kotlinc`/`scalac`/`groovyc`）、旧版 javac 自身的 codegen（本机只有 23.0.1）。

### C：不能独立成 compilation unit 的边界（逐条）

1. **本层引用了它证不出的读**：`post`/`conditional`/`cast`（javac: missing return statement）、`saved`（cannot find symbol）——这是答案本身（引用报全 BCI + 锚点）。
2. **只含引用的 Mixed 体**：能编译但**不执行**（否则是拿空体比真体）。
3. **发现 (i)**：按成员自己的签名编译不过——`open`/`openFailing`。`new@1` 写 `new Res(arg0, 0)`，而 `Res.<init>` 是 `(Ljava/lang/String;Z)V` → `javac: int cannot be converted to boolean`。这是 **P3-R5 的实参侧对偶**（2.4 定型的是被呈现方法的形参），**本片只记录不修**。
4. **缺失类型**：`MissingDependency.viaAbsentLibrary`（原方法也跑不了）。
5. **`<init>`/`<clinit>`**：包装类无法重声明样本自己的类名，`super()`/`this` 未建模；**逐样本打印为跳过**，不是静默遗漏。
6. **R2 的次数对照声明**是本文件给的 `void` 声明（文本不写返回值），只用于**次数**维度。

### 父级独立核对

- 全量 **982 passed / 0 failed / 3 ignored**（982 与 2.4 相同；ignored 1→3 = 两个新 `#[ignore]`）——父级直接跑 `--ignored` 得 **`2 passed`**、独立核出 `ignored` 合计 **3**；
- fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；
- **CI 的 diff 为 add-only**（父级逐行核过：仅新增一个 step 与注释）。

### 被修正的既有断言（1 条，精确相等，无放宽）

`crates/jarde-reader/src/classfile.rs::repository_class_fixtures_validate_without_false_target_rejections`：`(20, 93, 44, 81, 8)` → **`(26, 116, 44, 86, 8)`**（新增 6 个类文件 = 5×`Flags` + `MissingDependency`、23 个体、`choose` 的 5 个分支目标；异常表与 `jsr` 计数未变）。同时把该测试上方**已过时**的文档数字改写成实测口径。

### 证据

**CI**：`04cf83a` → 见下。**未触及依赖边**（无 `Cargo.toml`/`Cargo.lock` 改动）。

### 未完成（如实）

- **两处发现待裁决**：**(i)** `new@1` 把 `boolean` 实参写成 `0/1`（R5 的实参侧对偶）；**(ii) P3-R7**——ECJ v52 `finallyPath` 的 handler 未被交代（见下节）。
- **3.4**：`RecoveredMethod::facts()` 需文档行；`tests/fixtures/p3-corpus/README.md` 的矩阵应并入 `docs/support-matrix.md`（本片未改用户文件）。
- **CI-only 未验证**：`--release 8` 在 CI 的 Temurin 25 上仍受支持（本机只有 23）。
- 本片**未做独立 review**（父级已核 ignored 计数、CI diff 与全量门禁）。

## 2026-09-20 P3-R6 已关闭：slot 复用按 spec 拆分（提交 `58f0df9`）

**规格句**（`specs/source-maps/spec.md`，MUST 级、该 delta 的既有内容）：`Slot reuse across ranges`——同一 slot 在不同 BCI 区间承载不同变量时 SHALL 建立**不同作用域/名称**，**不得**把整个 slot 合并成一个变量。

### 分组规则（本片的实际难点）

**不是**「range 包含该 BCI」那么简单：**LVT range 描述的是变量可见范围，不是存储范围**——`c` 的 range 从 BCI 8 开始，而存它的 `istore_3` 在 **BCI 7**；`d` 同理（store 16 / range 17 起）。新增 `src/reuse.rs` 的判据：

- **read** → 归「range 包含**该 read 自己** BCI」的记录；
- **write** → 归「**其值被某条 range 内 read 消费**」的记录（反向用 `store_producing()` 定位产生该值的 store，与 1.3d 的 `slot_name_denotes_the_same_value` 同一套块内扫描）——**这正是 store 落在 range 之外的解药**；
- 仅在**赋值完整且无歧义**时拆（每条 read 恰落在一条 record、每个 write 的值恰被一条 record 的 in-range read 消费、同 BCI 的 read/write 归属一致）；否则退回「一个槽一个变量 + 序号名」；
- **range 不完整的 `a`**（slot 2，两条**同名**记录）→ 名字相同即 `Whole("a")`，**不**拆分，仍按 R3 规则提升到两臂与合流都可见处；
- 另两条保守规则：range 重叠/空/无 range 不拆；**guard 头部声明的资源槽永不拆**（2.4 的 header 自己写声明）。

### `NameTable` 的合同变更

从「一槽一名」变为「一槽一条（未拆）或多条（拆）」：键由 `u16` 变为新公开类型 **`LocalVariable { slot, index }`**；证据类型由「每槽一个名字」变为带 range 的 **`DebugLocal`** 与 **`SlotEvidence { Unnamed | Whole(String) | Split(Vec<String>) }`**；冲突/别名/`free_name` 改跑在**变量**上（槽序 → 槽内变量序，确定性）；新增 `whole(slot)`（**只**在该槽是一条变量时给名字，资源槽走它，防止误用第一条变量名）。声明提升（R3 的规则）也按**变量**各自计算。

### 产物（父级独立经公开入口复现）

`reuse(ZI)I`，`-g` 样本——**与原源码同形**：
```java
{
    int a;
    if (b) {
        int c = seed + 1;
        a = c;
    } else {
        int d = seed + 2;
        a = d;
    }
    return a;
}
```
`-g:none` 样本——**保持 3.1 的保守形**（无证据不拆，序号名）：`int local2; int local3; if (arg0) { local3 = arg1 + 1; local2 = local3; } …`，符合「缺失证据时 MUST 使用稳定 `argN`/`localN`」。

### 父级独立验收（**不用实现者的 harness**）

把 `-g` 产物自行包成 `public static int reuse(boolean b, int seed)` → `javac --release 8` **exit 0**；自行执行对照：
```
b=true  seed=5  generated=6  original=6
b=false seed=5  generated=7  original=7
```

### 父级独立证伪（三组，与实现者自报一致）

| 变异 | 结果 |
| --- | --- |
| `split()` 恒 `None`（退回合并） | 用例红，失败输出正是 3.1 旧形（`int local3;` 提升 + 两臂赋值 `local3`） |
| 拆分的名字**逆序**（c/d 对调） | 同一用例红，输出 `int d = seed + 1;` / `int c = seed + 2;`——证明用例钉的是**名字对位**而不只是「拆开了」 |
| （实现者）R3 的声明退回首次写入处 | R3 用例红，打印 `int local1 = 1; … local1 = 2;`（证明修 R6 没打断 R3） |

### 既有两个断言的处理（**编码的正是错误行为**）

- `two_variables_sharing_one_slot_…` **改名**为 `without_debug_evidence_a_reused_slot_stays_one_variable`——它只跑 `NO_DEBUG` 样本，**断言值一字未动**；改的是它原先写下的**通则**（「本层命名槽而非源变量，所以诚实的呈现是一个变量」）被限定为「**无 LVT 证据时**一槽一变量」。
- `a_table_that_names_a_slot_twice_states_no_name_for_it` 拆成两条：`scope(Z)I` 的 3 条断言**逐字保留**；`reuse` 的 3 条断言**编码的正是错误行为**（`int local3;` + `local3 = c;` 不出现），替换为对整块产物的精确断言 + 「不得出现 `local3`/`local2`」+ c/d 各在各自臂内 + `int a;` 在 `if` 之前。**弃用的 3 条没有以更弱形式保留。**

### 证据

全量 **986 passed / 0 failed / 3 ignored**（982 + 4 新用例）；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；**未触及依赖边**；`cargo test --test p3_execution_comparison -- --ignored` = **2 passed**（`reuse` 的 `Expect::Executed` 未回归）。
**CI**：`58f0df9` → 见下。

### 附：3.1 的勾选据此成立

3.1 任务文本的「覆盖 slot 复用」一项**在 3.1 交付时与 spec 相反**（父级派单失误：把已由 spec 定下的事当成可自由选择，见 P3-R6 段）。本片交付后该覆盖项**才真正成立**，故 3.1 的勾选由本片补齐依据。

### 仍待处理（不做，如实记录）

- **`docs/support-matrix.md` 第 51 行仍写「门面只有 ordinal local 名称」**——P3-R5/R6 之前的口径，属用户正在编辑的文件，本片只报告不改（3.4 应同步）。
- **P3-R7（ECJ v52 handler 未被交代）与发现 (i)（`new@1` 把 boolean 实参写成 `0/1`）仍未修**，两者都是「已记录、证据齐备、待排期」。

## 2026-09-20 P3-R7 与发现 (i) 已关闭（提交 `0b5eeb0`）

### P3-R7：每条已解码指令都必须被交代

**落点**：`region.rs`（恢复层内、**呈现前**），判据 `unaccounted_instructions(code, canonical)`——对每条**已解码指令起点**，覆盖集 = 所有 canonical 块的 `[blocks()[0], end_bci)` ∪ `unreachable` 里每个 id 的 bci 及其节点跨度；不在其中即「未被交代」。用的是 `ir.code()` 的 `instructions` + `ir.canonical()` 的 `blocks()/unreachable()`，**不依赖 `Operation` 模型**。

**违反时两档，都引用全部未被交代的 BCI + `Mixed`/`Fallback` + 诊断 `jre_region_unaccounted_instruction`**：走查原本会整具结构化 → **整具引用**；走查本来就已拒绝某些区域 → 保留那些**更具体**的理由并**追加**一个只引用未被交代 BCI 的区域。第二档必要，否则会把 `jre_guard_handler` 这类更具体的理由顶掉（2.4 的 patch 样本正是这种情形）。

**父级独立复现**（公开入口，`tests/fixtures/historical/ecj-4.6.1/v52/` 的 `finallyPath(I)I`）：

修前 `Java/Structured`，文本只有 `int local3 = arg1 + 1; arg1 = arg1 + 2; return local3;`，BCI 9/10/13/14 **既无文本也无锚点**。修后：
```java
{
    // @bytecode 0 9 10 13 14
    // the decoded body states 4 instruction(s) at BCI [9, 10, 13, 14] that no canonical block covers and that the graph does not list as unreachable: the graph is not an account of these bytes, so no region of it may be presented as the body
}
```
未被交代的 BCI = **[9, 10, 13, 14]**。`add(II)I` 不受影响（文本逐字不变）。

**canonical 侧的根因（实现者核实，父级记录，只报不改）**：`cfg.rs` 的异常边**只从 `may_throw` 表承认的指令发出**，且只在走查已建的块内；**声明的 `Exception table` 本身不是走查根**。`finallyPath` 的受保护区间 `[0,4)` 是 `iload_1/iconst_1/iadd/istore_3`——`may_throw` 全为 false → 无 throw site → **`throw_sites()` 与 `handler_rows()` 都为空**、handler 入口 9 落在任何块之外。v45 同形身体之所以有六节点（三死），是 `jsr` 的 call context 使每个上下文入口都成为根。
**若改在 canonical**（让每条已声明 handler 入口成为走查根、未达者进 `unreachable`）：牵动 frames/SSA 公布面、计费（`IrItems`/`normalization_clones`）、块数计数与已归档的 `p2_canonical`/`p2_golden`/`p2_contracts` 期望值——**属 P2 归档交付的取舍，需另行裁决**。恢复层的核对已消除「静默丢失」，且**不预设 `may_throw` 模型永不漏标**。

**3.3 harness 已硬化**：note 改为两条硬断言（新增 `Ledger`/`ledger_of`，用同一 request 独立再分析一次按同一判据算账）——① 未被交代的 BCI 存在即失败，**除非全被引用**（即已如实拒绝）；② handler 入口若既无锚点又无引用且产物为 `Java` 即失败。

### 发现 (i)：`boolean` 实参

**落点**：共享实参渲染路径（新增 `typed_arguments(descriptor, arguments)` + `parameter_descriptors`），在 `call_expr`（invokevirtual/special/static/interface 全走它）、`new_expr`、`constructor_call`（`super()`/`this()`）三处调用。参数为 `Z` 且实参是字面量 `0/1` → `ExprKind::Boolean`（新增变体，发射 `true`/`false`）；`int`/`long`/`float`/`double`/`String`/引用不动；`byte`/`char`/`short` 收 int 常量在 Java 里合法故不动；描述符无法解析或**参数个数与实参不符则不猜**（保持既有行为）。

**父级独立复现**：`open(Ljava/lang/String;)LRes;` → `return new Res(arg0, false);`；`openFailing(…)` → `return new Res(arg0, true);`。
**父级独立验收（不用实现者的 harness）**：自行包装 + 提交的 `Res.class` → `javac --release 8` **exit 0**（修前同包装报 `int cannot be converted to boolean`）；自行执行：
```
generated open        -> no throw
generated openFailing -> java.lang.IllegalStateException: close-r
```
即 `true` 那个布尔**确实生效**（`close()` 抛出），`false` 那个不抛。

### 被修正的既有断言（5 处，均加强，无放宽）

① harness 的 ECJ `finallyPath`：`Expect::Executed` → **`Expect::Quoted(Some("jre_region_unaccounted_instruction"))`**——旧期望断言的正是**一段静默丢掉 4 条指令的「完整呈现」**；② harness 的 `open`/`openFailing`：`NotACompilationUnit` → **`Executed`**（现在要执行并逐行比轨迹，比「javac 拒绝」更强）；③ `p3_guard` 的 `a_close_the_exception_path_lacks_is_refused` 保留原有 `jre_guard_handler` 断言并**新增**两条（新码存在 + BCI 32/33/34/35 被引用）；④ harness 的两条 note → 硬断言；⑤ 轨迹标签经 `trace_label()` 转义（此前无已执行成员的实参含引号，故无既有行变化）。

### 证据

全量 **988 passed / 0 failed / 3 ignored**（986 + 2 新用例）；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 12 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；锁文件两条 均 exit 0；`p3_execution_comparison -- --ignored` = **2 passed**。
**父级独立证伪**（与实现者自报一致）：判据恒空 → ECJ 用例与 harness 红；判据恒真（一律拒绝）→ 4 条 `presented` 用例与 harness 红；`typed_arguments` 原样返回 → `open` 的编译断言红。

### 残留（如实，非本片范围）

TWR 已被 `twr@1` 证明的 handler 入口**并非每条都有 segment 锚点**（`one()V` 的 BCI 20 有、BCI 32 无，但两者都被该区域的 block 记录认领）——这是 2.4 guard origin 的 **provenance 缺口**，不是「未被交代」；harness 的两条断言按「锚点/引用/被块覆盖」三选一判定，已在 `tests/fixtures/p3-corpus/README.md` 写明它**不属于** P3-R7 那一类。
