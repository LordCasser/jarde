
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
