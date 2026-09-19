
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
