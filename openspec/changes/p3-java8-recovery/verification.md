
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
