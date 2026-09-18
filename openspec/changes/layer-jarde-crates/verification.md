# layer-jarde-crates 验证记录

约束：本 change 是**结构重组**，不兼带语义修复。每条记录携带**精确 commit** 与当时的全量数字；未绿不搬迁。

## 1.1 基线（2026-09-18）

**精确基线**：`0f3134b`（`docs: accept the call-context slice`）。前置条件已满足——P2 的 `0.3`/`0.3b`/`3.4` 及其依赖全部验收完毕（0.1–0.5 六项与 3.4 各有独立复核；3.4 经**四轮**复核后 Approve）。

**测试结果（该 commit 上实测）**：

| 项 | 结果 |
| --- | --- |
| `cargo test --workspace --all-targets --all-features --locked` | **628 passed / 0 failed / 1 ignored** |
| `call_context` 单测 | 36 |
| `cargo test --test p1_xref_golden --locked` | 5 passed |
| `cargo fmt --all -- --check` | 干净 |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 干净 |
| CI（`24ae87e`） | run [`35341736858`](https://github.com/LordCasser/jarde/actions/runs/35341736858) 四 job success |

**生产依赖树**（`cargo tree -p jarde --edges normal --depth 1`）：`blake3`、`flate2`、`noak 0.7.0`、`petgraph 0.8.3`、`rawzip`、`serde`、`thiserror`。按 design 的归属：`noak`/`rawzip`/`flate2`/`blake3` → reader；`petgraph` → jvm；`serde`/`thiserror` 按实际使用声明。**拆分不顺手升级任何第三方版本或 feature**。

**待拆分的 `src/` 模块**（19 项）：`artifact`、`budget`、`call_context`、`cfg`、`classfile`、`dispatch`、`engine`、`environment`、`error`、`ir`、`lib`、`members`、`model`、`multi_release`、`passes`、`providers`、`query`、`resolver`、`view`、`xref/`。

**本机环境限制（如实记录）**：验证期间本机链接器失效（Xcode 许可未接受，`xcrun --sdk macosx --show-sdk-path` 失败、链接报 `library 'System' not found`）。全部 cargo 命令均在 `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk` 且 `PATH` 以 `/Library/Developer/CommandLineTools/usr/bin` 开头的环境下执行。CI 侧（Linux runner）不受影响。

## 1.2 盘点（2026-09-18，1.1 的第二半）

全仓 `pub(crate)` 命名项 185 个，其中 140 个被至少一个其它文件引用。完整清单按「接缝 / jvm 内部 / 无需公开」三类归档，下面是**必须在搬迁前处理**的部分。

### 接缝归属（升公开面的最小集）

| 接缝 | 位置 | 目标 crate | 公开面 |
| --- | --- | --- | --- |
| `ArtifactSnapshot::read_entry_internal` | `artifact.rs:773` | reader | 升 `pub` 并改名 `read_entry_for_analysis`；保留快照/entry 校验与 `Intermediate` 记账 |
| `class_facts`/`ClassFacts` | `classfile.rs:1832`/`:1725` | reader | `pub`，**不提供构造器**（不开放伪造已验证状态） |
| `MethodCodeFacts` | `classfile.rs:2520` | reader | `pub struct`，但 **`operands` 字段保持私有**，改只读访问器/lockstep 迭代器（`classfile.rs:2717` 的 `debug_assert_eq!` 就是该不变量） |
| `InstructionOperands` 及同族 | `classfile.rs:2571+` | reader | 必须 `pub`（否则 `MethodCodeFacts` 公开不了）；无 reader 外构造路径即为最小面 |
| CP/descriptor/attribute 查询族 | `classfile.rs:3845+` / `:2190+` / `:1941+` | reader | `pub`（只读查询，不暴露 pool layout） |
| `xref::scan_candidates` + `CandidateScan` | `xref/mod.rs:338`/`:194` | query | `pub`（jvm 唯一接缝） |
| `CandidateFilter` | `xref/mod.rs:104` | query | **不整体导出**：只暴露成员形状与 signature-polymorphic 两种（见 design §3.5） |
| `query::execute` | `query.rs:380` | query | `pub`（门面唯一入口） |
| `xref::with_usage` | `xref/mod.rs:1249` | reader | 升 reader 的 `pub`，**与 `multi_release.rs:1814` 的私有副本合并为一份**，query/jvm/门面共用 |
| `budget_dimension_code` | `artifact.rs:1576` | reader | `pub`（纯映射，无状态） |

### jvm 内部（拆包后仍同 crate，**全部不公开**）

`providers`（Header 闭包/仲裁）、`members`（成员规则）、`dispatch`、`resolver`、`environment`、`ir`/`passes`、`cfg`/`call_context` 的全部 `pub(crate)` 项。**「不为维持门面原实现而公开可变内部」是硬约束**——`FactLedger`、`HeaderClosure`、`AnalysisRun`、`IrPhase`、`PassDescriptor` 在 jvm 之外必须不可见。

### 无需公开（只是恰好同 crate）

`JvmString::from_parts`（消费者都在 reader 内）、`probe_minimal_header`（consumer 是 reader 的 `multi_release`）、`query::validate_request`、`xref::scan`/`ScanResult`、`query.rs` 的三个 coverage 辅助——保持 crate 内。

### 会因拆包而编译失败、必须先处理的 7 项

| # | 位置 | 原因 | 处理 |
| --- | --- | --- | --- |
| 1 | `cfg.rs:1751`、`call_context.rs:1972/2009/2022` | 跨包访问 reader 的 `#[cfg(test)] classfile::test_class` | reader 加 `#[cfg(any(test, feature = "test-support"))]` 门禁 + jvm 用 dev-dependency 启用；**不得无条件 `pub`** |
| 2 | `tests/p2_contracts.rs:2571-2584`、`:2891-2909` | A17 守卫硬编码 6 条 `src/` 路径与文件数 | 改为按包/模式枚举，搬迁前先对当前布局跑绿 |
| 3 | `tests/p2_contracts.rs:2685` | `derived_p2_type_tokens` 读 `src/environment.rs`/`resolver.rs`/`ir.rs` | 同上 |
| 4 | `classfile.rs:4455`、`call_context.rs:1271-1279`、`cfg.rs:1618-1628` | `include_bytes!("../tests/fixtures/…")` 相对 `src/` | 改为 `CARGO_MANIFEST_DIR` 或 workspace 级 fixtures 常量 |
| 5 | `p1_xref_golden.rs:472`、`p2_contracts.rs:2179/2842/3155`、`jvm_bytecode_oracle.rs:137`、`classfile.rs:9127` | `CARGO_MANIFEST_DIR` 随 crate 变化 | 同上 |
| 6 | `providers.rs:2140` | jvm 侧 `rawzip` 仅测试使用 | jvm 的 `[dev-dependencies]` |
| 7 | 根 `tests/*.rs` 全部 | `rawzip`/`blake3`/`petgraph` 不再是根包 normal dep | 根包补 `[dev-dependencies]` |

### 门面耦合（**必须迁入 jvm**）

`ACC_ABSTRACT`/`ACC_NATIVE`、`IR_RAW_CFG_INCOMPLETE_BODY`、`report_unimplemented`、`run_method_analysis`、`FactLedger::new` 与 `HeaderClosure::new` 的直接构造、`DriverRead`/`read_driver_method`/`has_code_attribute`/`no_body_kind`/`reader_stop`/`stopped_at_code`/`stop_severity`、`raw_cfg_failure`/`stage_state`/`termination_code`/`with_usage`（后者的实现改为引用 reader 的共享版本）。门面保留 `Engine` 与各入口的一行委托 + 收窄的再导出白名单（`lib.rs:27-40`，移除 passes/cfg/call_context/providers/members/dispatch 面）。

**已核实的耦合面**：`crates/jarde-cli/src/main.rs:3`、`crates/jarde-cli/tests/{json_cli,query_cli}.rs`、`examples/*.rs`、`fuzz/src/lib.rs:20-25` 都只 `use jarde::{…}`，**不触及任何 `pub(crate)`**——因此只要门面再导出白名单不变，它们无需改动；CLI 与 fuzz 的耦合风险集中在白名单收窄那一步。

## 1.3 搬迁前的前置动作（2026-09-18）

盘点列出的 7 项里有 2 项属于「不先做、一搬就断」，已先做并各自验证。其余 5 项随对应搬迁步骤处理。

| # | 前置动作 | 状态 |
| --- | --- | --- |
| 1 | `classfile::test_class` 跨包可达 | **已完成** |
| 4 | fixtures 路径不再相对 `src/` | **已完成** |
| 2、3 | A17 守卫改造（`tests/p2_contracts.rs` 的硬编码路径与文件数） | 随 2.1/2.2 搬迁同步 |
| 5 | `providers.rs:2140` 的 `rawzip` 改 jvm dev-dependency | 随 2.2 |
| 6 | 根包补 `[dev-dependencies]` | 随 1.2 |
| 7 | CI `jvm` 边界正则实测 | 随 3.2 |

### 1.3.1 `classfile::test_class` 的门禁改为 feature

`cfg.rs` 与 `call_context.rs` 共 4 处单测直接调用 reader 的 `#[cfg(test)] mod test_class`；拆包后 4 处都在 jvm，而 jvm 的 `cfg(test)` 看不到 reader 的测试模块，**会直接编译失败**。

- 门禁改为 `#[cfg(any(test, feature = "test-support"))]`，根 `Cargo.toml` 新增 `test-support = []`；jvm 搬迁后用 `[dev-dependencies] jarde-reader = { path = "...", features = ["test-support"] }` 启用。
- **不进生产 API**：没有任何 normal 依赖启用它；模块内也没有 `pub`（包级可见性）项。已实测：`cargo check --lib --locked`（无 feature）与 `cargo test --workspace --all-targets --locked`（无 `--all-features`）都不含该模块，且全量仍 **628 passed / 0 failed / 1 ignored**。
- **注意**：CI 用 `--all-features`，因此该模块在 CI 的普通 lib 构建里也会被编译。已按此写 `#[allow(dead_code, reason = ...)]`，`clippy -D warnings` 在 `--all-features` 下干净。

### 1.3.2 fixtures 改为从 crate 根寻址

11 处 `include_bytes!("../tests/fixtures/…")`（`classfile.rs` 1 处、`call_context.rs` 5 处、`cfg.rs` 4 处，另有 `classfile.rs` 1 处运行期 `CARGO_MANIFEST_DIR`）都**相对所在文件**拼路径；文件一旦搬进 `crates/<name>/src/`，同一字面量会解析到新 crate 内部而编译失败。

- 新增 `src/test_fixtures.rs`：一个 `fixture!` 宏（编译期嵌入）与一个 `fixtures_root()`（运行期遍历），两者各自把**深度写在一处**；调用点只写 fixture 名。搬迁时改这一个模块，而不是逐个字面量。
- 唯一一份 fixtures 仍在仓库根 `tests/fixtures`，**未复制**。
- **证伪**：把宏里的路径段指向「搬迁后新 crate 会去找的位置」，编译立刻失败，11 个调用点**全部通过 `src/test_fixtures.rs:29` 这一个位置报错**（`couldn't read ...: No such file or directory`）——即路径错误是响亮的编译错误，且修复点唯一。还原后 `sha256sum -c` OK。

### 证据

`cargo fmt --all -- --check` 干净；`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`cargo test --workspace --all-targets --all-features --locked` = **628 passed / 0 failed / 1 ignored**（与基线一致，本片为纯结构改动）；无 feature 的 `cargo test --workspace --all-targets --locked` 同样 628/0/1；提交 `d845a2d`、`21ea5b5` 已推送；CI run [`35343834985`](https://github.com/LordCasser/jarde/actions/runs/35343834985) 四个 job 全部 success（含 `--all-features` 下新 feature 的门禁与 MSRV）。

**1.2 的可行性已核实**：reader 侧 7 个文件（`artifact`/`classfile`/`model`/`budget`/`error`/`view`/`multi_release`）的 `crate::` 引用**全部落在彼此之间**（`artifact`↔`budget`、`error`↔`budget` 互相引用类型，design 已允许同包），**没有任何一条指向 query/jvm/engine**——即设计所称的「源码依赖支持拆分」已由实测确认。`engine.rs` 882 行中，检查入口部分（`ClassTarget`/`ClassSource`/`materialize`/`inspect_header`/`inspect_method_bytecode`/`header_coverage`/`bytecode_coverage`）随 reader 走，driver 部分（`run_method_analysis` 起）随 jvm 走。

## 1.2 抽出 `jarde-reader`（2026-09-18，实现完成，待独立复核）

**新包** `crates/jarde-reader/`：`artifact`、`budget`、`classfile`、`error`、`model`、`multi_release`、`view` + 新增 `inspect.rs`（从 `engine.rs` 迁出的检查入口）+ `test_fixtures.rs`。

- `inspect.rs` 承载 `ClassTarget`/`ClassSource`/`EngineHeaderReport`/`EngineBytecodeReport` 与 `inspect_header`/`inspect_method_bytecode`；`materialize`/`header_coverage`/`bytecode_coverage` 一并迁入但**保持私有**（无外部消费者，不进公开面）。
- `engine.rs` 882 → **732** 行：只剩 `Engine` 与 driver，两个 inspect 入口改为一行委托。
- 根包 `src/lib.rs` 用 `pub use jarde_reader::{…}` 再导出，因此 `crates/jarde-cli/**`、`examples/**`、`fuzz/src/lib.rs`、根 `tests/**` 的 `use` 行**一行未改**。

### 可见性收敛

升 `pub` 的接缝按盘点清单执行：`read_entry_internal` → **`read_entry_for_analysis`**（文档写明保留快照/entry 校验、读取类别与计费）、`budget_dimension_code`、`class_facts`/`ClassFacts`、`MethodCodeFacts`（**`operands` 字段保持私有**，改由 `operands()` 只读访问器暴露）、`InstructionOperands` 及 `ImmediateValue`/`LocalOperand`/`SwitchOperands`、CP 与 descriptor 查询族、attribute/bootstrap 族、`method_code_facts`/`method_code_coverage`、`ControlFlowTarget`/`ControlFlowTargetKind`/`control_flow_targets`、`multi_release::select`。

**未升**（保持 crate 内）：`MinimalHeaderFacts`、`probe_minimal_header`、`JvmString::from_parts`、`test_fixtures::{fixture, fixtures_root}`。

**`with_usage` 已合并为一份**：由 `xref::with_usage` 迁入 `reader::model::with_usage`，并吸收 `multi_release.rs` 与 `engine.rs` 各自那份私有副本——4 处调用现在共用同一实现（盘点前置动作之一，已随本片完成）。

### 实现者如实报告的三处「超出纯可见性」改动（父级裁决）

| 改动 | 原因 | 裁决 |
| --- | --- | --- |
| 新增 `MethodCodeFacts::from_parts`（`test-support` 门禁） | `operands` 私有后，根包 `cfg.rs:1184`、`call_context.rs:1479/1496` 的**测试夹具无法再构造** `MethodCodeFacts`（`E0451`） | **接受**：它只在 `test-support` 下存在，入参是配对列表（锁步无法破坏），生产构建没有该构造路径 |
| `tests/p2_contracts.rs` 里一处路径重指向（读 `budget.rs` 取 `BudgetDimension` 变体） | `budget.rs` 已搬走，该测试立刻变红（盘点把 A17 守卫改造记为「随 2.1/2.2」，但本片已触发） | **接受**：是路径同步而非改期望值 |
| 额外升 `pub`：`physical_variant_for_path`、`CpIndexOf`、`InnerClassFacts`/`EnclosingMethodFacts`/`ProvidesFacts`/`ModuleFacts` | 前两者是跨包使用的纯映射；后四者是公开字段的类型，不公开则 `private_interfaces` 在 `-D warnings` 下报错 | **接受**：都是「不公开就编译不过」的必然公开面 |

### 依赖归属（实测）

| 包 | `[dependencies]` |
| --- | --- |
| `jarde-reader` | `blake3`、`flate2`、`noak`、`rawzip`、`serde`、`thiserror`；**无 `petgraph`** |
| 根 `jarde` | `blake3`（query/xref/providers 仍直接哈希，收敛属 2.1/2.2）、`jarde-reader`、`petgraph`、`serde`；**移除** `noak`/`thiserror`/`flate2`/`rawzip` |
| 根 `[dev-dependencies]` | `flate2`（`tests/p1_*` 直接构造 ZIP/deflate 夹具）、`jarde-reader`(`test-support`)、`proptest`、`rawzip`（`providers` 的测试写 ZIP）、`serde_json` |

两个 path 依赖都写了 `version = "=0.1.0"`（`deny.toml` 的 `wildcards = "deny"`）；`cargo deny check bans` 通过。

### 证据（父级独立复跑确认）

| 项 | 结果 |
| --- | --- |
| `cargo check -p jarde-reader --locked` / `cargo test -p jarde-reader --locked` | 干净 / **126 passed / 0 failed**（独立可执行，design 退出门槛之一） |
| `cargo tree -p jarde-reader --edges normal --locked` 中 `petgraph\|jarde-query\|jarde-jvm\|根 jarde` | **0 命中**（退出门槛之二） |
| `cargo test --workspace --all-targets --all-features --locked` | **628 passed / 0 failed / 1 ignored**（与基线逐项一致；单元测试 224 = 根 98 + reader 126） |
| `cargo test --test p1_xref_golden --locked` | 5 |
| `cargo fmt --all -- --check` / `clippy -D warnings` | 干净 |
| `cargo run --example resolve_and_analyze` | 与 `git archive` 出的真基线逐行 diff，只有 `elapsed_millis` 差异，归一化后 **18 行完全一致** |

**反例**：在 reader 里临时 `use jarde::Engine as _;` 与 `use petgraph::…` → `cargo check -p jarde-reader` **编译失败**（reader 确实不依赖上层与图算法），还原后逐字节校验。

### 一处被证伪的设计假设（重要，影响 2.2）

1.3.2 的文档原先写「依赖方通过 reader 的宏读夹具」。**实测不成立**：`env!("CARGO_MANIFEST_DIR")` 在 `macro_rules!` 体内是**按调用方 crate** 求值的——我另建一个最小 workspace 复现（宏定义在 `x`、从 `y` 调用 → 打印 `y` 的 manifest 目录），而 `const` 里的 `include_bytes!` 则在**定义方**求值（嵌入 `x` 的文件）。

因此：**共享夹具的正确形态是「定义方嵌入的具名常量」，不是导出宏**；根包在模块迁走前仍需自己那份 `test_fixtures.rs`（深度不同）。两处文档已按此更正，2.2 搬迁 `cfg`/`call_context` 时按此设计。

### 独立复核（Approve，无阻断项）与据其修正

复核者（第三方只读）逐项核对后 **Approve**，并确认：reader 独立性是编译器强制的（它自己在副本里加 `use petgraph` / `use jarde` → `E0432` 失败）；除已申报的三处外**无语义改动**（`budget.rs`/`error.rs`/`view.rs` 与旧文件逐字节等价；`artifact.rs` 仅改名；`classfile.rs` 解码路径零改动）；`engine.rs` 分割归属正确、无错位；`from_parts` 的锁步由类型保证（`unzip` 无法传两份不等长列表）。

它另指出两处并已修正：

| 复核发现 | 修正 |
| --- | --- |
| `artifact.rs::execution_with_usage` 是 `with_usage` 的**第 4 份**副本，「已合并为一份」实际是 3/4（与本记录措辞不符，功能无影响） | 删除该副本，调用点改用唯一实现 |
| 门面对 `model` 做 glob 再导出，使**`jarde::with_usage` 成为公开 API**——等于给每个消费者一条「把 reader 从未测过的账目写进任意报告」的构造路径，与 reader 自身「不导出它未曾建立的状态的构造路径」相矛盾 | 把该操作从数据模块 `model` 移入**门面不再导出的** `jarde_reader::accounting`（跨包调用方按包路径直呼）。**探针验证**：`tests/` 里引用 `jarde::with_usage` → `E0425: cannot find value with_usage in crate jarde`，Facade 不再可达 |

**fuzz lock（复核者与实现者都曾把它记为 3.1 的范围，实测不能延后）**：新包使 `fuzz/Cargo.lock` 过期，CI 的 `supply chain` 与 `fuzz smoke` 两个 job 直接红（`--locked` 拒绝更新 lock）。已就地修复：`cargo update -p jarde` 只**新增** `jarde-reader` 条目，**第三方版本零变动**（diff 无 `version` 行变化），`cargo metadata --locked` 通过、`cargo deny check bans` 通过、`cargo build --locked --bins`（nightly-2026-07-20）通过。**不延后**的理由：本仓库的纪律是每一步留一个绿色边界，红着的 CI 不是可以带着走的状态。

### 独立复核（Approve）与它要求带进 2.2 的一项

复核者 **Approve**，并逐条核实：依赖方向是编译期事实（含 `--edges all`）；`CandidateFilter` 的最小面是**「不可拼写」而非「导出后写文档禁止」**——`From` 的 `match` 只有两个臂且无通配臂，**编译器已证明**没有第三个变体，`#[non_exhaustive]` 全包 0 处；搬迁**无任何语义改动**（归一化 diff 后 `xref/{code,metadata,bootstrap,resource}.rs` 逐字等价，`query.rs` 仅 `execute` 升 `pub`）；`src/resolver.rs`/`src/engine.rs` 的调用形态与搬迁前完全等价；`CandidateScan` 的 5 个公开字段**恰好**等于 resolver 的实际读取面（不过宽也不过窄）。

**必须带进 2.2 的一项（复核者判定不阻塞 2.1）**：本片造成**门面净扩张**——搬迁前这四个项在根包是 `pub(crate)`，现在可从 `jarde::` 触达：

| 项 | 判断 |
| --- | --- |
| `scan_candidates` | **不该在门面上**（设计上只给分析层的 jvm 接缝，且绕过 `Engine::query` 的游标/分页语义） |
| `CandidateScan`（含 5 个字段） | **不该在门面上**，与上一条同生共死 |
| `CandidateFilter` | **不该在门面上**（门面消费者无从表达合法 filter） |
| `execute` | 低风险（语义与 `Engine::query` 相同），**随 2.2「门面只保留委托」一并决定去留** |

否则它们会成为永久公开 API。**2.2 的验收须包含门面白名单收窄**（不是可选）。

### 未完成 / 待办

- ~~独立复核未做~~ → 已完成并 Approve（见上）。
- ~~`fuzz/` 的 path/lock 同步~~ → 已随本片完成（见上）。
- `ci.yml` 的 `jvm` 边界正则实测 → 3.2；MSRV 1.88 与两套 supply-chain → 3.3。
- **`test-support` 打开时 `jarde::test_class` 可经门面 `pub use classfile::*` 触达**（此前是 `pub(crate)`）→ 2.2 收窄再导出白名单时一并处理。

## 1.5 A17 守卫已改造为布局无关（2026-09-18，复核 Approve）

1.4 的方案已实施并独立复核。**本片只改测试**，`src/**`、`crates/**` 未动。

**改造要点**：

- `A17_EXPECTED_FILES`（6 条 `src/` 字面量）→ `A17_EXPECTED_MODULES`（模块身份 + 候选路径，旧布局在前）；新增 `resolve_layout`，对候选做存在性过滤并断言**恰好命中一个**（0 个 = 布局变了没人更新；≥2 个 = 两套并存，不任选）。目录身份（`xref/`）、正对照（`engine.rs`）、类型派生源（`environment.rs`/`resolver.rs`/`ir.rs`）各有一份身份表。
- **token 表只增不删**：module token 12 → 15（+`jarde_jvm::`、`jarde_query::`、`jarde::`），import token 7 → 12（+`jarde_jvm as`、`jarde_query as`、`jarde as`、`extern crate jarde_jvm/query`）。两套形态**都要**保留：搬迁后 jvm 侧的 `crate::query` 不再指 query 包。
- **sandbox 用例表改为身份 + 双布局**：同一份 `cases` 数据（offenders/evidence/tiny_files 一字未动）在 `SANDBOX_LAYOUTS` 的两种布局下各跑一遍；新增两条跨 crate 用例（`jarde_jvm::…`、别名与门面形态）。
- `A17_GUARDED_FILES = 6`、`A17_MIN_SOURCE_LEN`、全部完整性与非空断言**未改**。

**父级独立验证（在真实树上，不只在副本里）**：

| 验证 | 结果 |
| --- | --- |
| 在 `src/query.rs` 注入 `use crate::resolver::ResolutionReport as _;` | **红**：`src/query.rs: crate::resolver, resolver::, ResolutionReport` |
| 把 `src/query.rs` 移走（改用 `#[path]` 垫片保住编译） | **红**：`found []`（不是静默通过） |
| 构造**半搬迁**树（`query.rs` 留 `src/`、`xref/` 移到 `crates/jarde-query/src/`） | **红**：`the guarded sources are split across layouts (src/query.rs and crates/jarde-query/src/xref)` |

**复核者（第三方）结论：Approve，无必须改项**。它逐项核对了「未放宽」：被守文件数与尺寸下限未动；A17 区内 `assert!` 8→8、`assert_eq!` 4→5、`panic!` 2→2、`.expect` 5→5（**只增不减**）；匹配器本体（`normalize_module_paths`/`contains_token`/`p2_tokens_in`/`declared_public_type_names`/`collect_rs_files`）**逐字节相同**；sandbox 用例表在路径→身份归一后**完全一致**。它另实测：把 `query.rs` 建成目录、把 `xref` 建成文件都不会误判为命中；两套并存与一个都没有都会响亮报错。

**复核者指出并由父级修正的一处**：唯一性是按**身份**判定的，所以半搬迁的混搭树**能**解析通过（每个身份各自命中），而 `GuardedModule` 的注释声称「两套并存必须响亮失败」——文档强于实现。已加 `assert_single_layout`（比较两个被守身份解析到的布局根），并把注释改为准确表述；半搬迁树现在**失败**（上表第三行）。

**范围外发现（转 2.2 处理）**：`tests/p2_contracts.rs` 的 `all_lists_are_complete_and_align_with_the_serde_names` 仍写死读 `src/environment.rs`/`src/ir.rs`，2.2 搬走后会直接 panic；它不在 A17 守卫范围内，但可直接复用本片的 `resolve_guarded_file`。

证据：`p2_contracts` = **29 passed**；全仓 **628 passed / 0 failed / 1 ignored**；`p1_xref_golden` = 5；`fmt`/`clippy -D warnings` 干净。提交 `5ada874`、`90fed85`。

## 2.1 抽出 `jarde-query`（2026-09-18，实现完成，待独立复核）

**新包** `crates/jarde-query/`：`query.rs` + `xref/{mod,code,metadata,bootstrap,resource}.rs`。根包保留 resolver/environment/ir/cfg/passes/call_context/providers/members/dispatch/engine，并再导出 query 面，因此 CLI/examples/fuzz/根 `tests/**` 的 `use` 行一行未改。

**搬迁差异**（对副本逐行比对）：`query.rs` 16 行、`xref/mod.rs` 118、`code.rs` 18、`metadata.rs` 14、`bootstrap.rs` 22、`resource.rs` 10——**全部是 `crate::` → `jarde_reader::`/`jarde_query::` 路径改写与可见性/接缝改动**，无语义变更。

### 接缝收敛（本片的重点）

| 项 | 处置 |
| --- | --- |
| `query::execute` | 升 `pub`（门面唯一入口） |
| `xref::scan_candidates` + `CandidateScan` | 升 `pub`，其字段（`items`/`has_more`/`coverage`/`execution`/`diagnostics`）按 resolver 实际读取面公开 |
| **`CandidateFilter`** | **不整体公开**：内部枚举改名 `CandidateRule`（`pub(crate)`，保留 `Exact`/`MemberShape`/`SignaturePolymorphic`），新增**只含两个变体**的公开 `CandidateFilter`，用 `impl From<CandidateFilter> for CandidateRule` 直接搬移（同名变体，非二次翻译）。**未用** `#[non_exhaustive]`——那等于把 scanner 语义面放开。`Exact` **没有任何公开拼法**，`CandidateRule` 不被再导出 |
| `query::validate_request`、`xref::scan`/`ScanResult`、coverage 辅助 | 保持 crate 内（消费者都在 query 包内） |
| `src/resolver.rs`（jvm 侧） | 改经 `jarde_query::{CandidateFilter, scan_candidates}` 调用，**语义不变**——这是 design 明确保留的真实依赖 |
| `src/engine.rs` | query 入口改为委托 `jarde_query::query::execute` |

### 依赖归属（实测）

- `jarde-query` normal：`blake3`、`jarde-reader`、`serde`；dev：`serde_json`。**无** `petgraph`/`jarde-jvm`/`jarde-java`/根 `jarde`。
- `blake3` 暂留 query 侧（`query.rs` 直接构造 cursor 绑定摘要），注释标明这是 design §3.5 的**后续收敛项**，本片按边界未做。
- 根包 normal 仍持 `blake3`（`providers.rs`，属 2.2 的 jvm 侧）、`petgraph`、`jarde-reader`、`jarde-query`、`serde`。

### 证据（父级独立复跑确认）

| 项 | 结果 |
| --- | --- |
| `cargo check/test -p jarde-query --locked` | 干净 / **3 passed**（独立可执行，design 退出门槛之一） |
| `cargo tree -p jarde-query --edges normal --locked` 中 `petgraph\|jarde-jvm\|jarde-java\|根 jarde` | **0 命中**（退出门槛之二）；含 dev 边同样 0 命中 |
| `cargo test --workspace --all-targets --all-features --locked` | **628 passed / 0 failed / 1 ignored**（与基线一致） |
| `cargo test --test p1_xref_golden --locked` / `--test p2_contracts --locked` | **5** / **29** |
| `cargo fmt --all -- --check`、`clippy -D warnings` | 干净 |
| `fuzz/Cargo.lock` | 只新增 `jarde-query` 条目；`version =` 行**仅**多一条本地 `0.1.0`，**第三方零变动**；`cargo metadata --locked`、`cargo deny check bans`、`cargo build --locked --bins` 均通过 |

**反例（实现者在副本里做，复核者复测并更正）**：加 `use petgraph::…` → `E0433`；加 `use jarde_jvm::…` → `E0432`（不是 `E0433`，本记录初稿口径有误，已更正）；`CandidateFilter::Exact` → `E0599`（无该变体）；`CandidateRule`/`xref::scan`/`ScanResult` → `E0603`（私有）；反向用例（两个形状可构造 + `execute`/`scan_candidates` 签名 + `CandidateScan` 五字段可读）通过。

**复核者指出的一处空真（重要）**：`jarde-jvm` **这个包目前还不存在**（workspace members 只有 `.`/`jarde-cli`/`jarde-query`/`jarde-reader`），所以「query 够不到 jvm」在今天有相当部分是**空真**。真正承重的是它补的那条反例：在 query 里 `use jarde::…` → `E0432`，即**查询包不反向依赖承载全部分析能力的根包/门面**。3.2 的依赖闭包门禁应以这条为主用例，而不是尚不存在的 `jarde-jvm`。

**A17 守卫已跟着走**（证明它不是空转）：在副本的 `crates/jarde-query/src/xref/resource.rs` 注入 `// probe: crate::resolver` → `p2_contracts` 失败并指名 `crates/jarde-query/src/xref/resource.rs: crate::resolver`——守卫确实在扫搬迁后的文件。

### 未完成 / 待办

- ~~独立复核待做~~ → 已完成并 Approve（见下节）。
- 归属后续片：3.2 的 Cargo 依赖闭包门禁与 `query→jvm` 反例；2.2 的门面白名单收窄（本片使 `jarde::query::execute`、`jarde::xref::scan_candidates`、`jarde::CandidateFilter`、`jarde::CandidateScan` 可从门面触达）；`blake3` 向 reader 收敛。

## 1.4 A17 守卫的改造方案（供 2.1/2.2 与 3.2 执行）

`tests/p2_contracts.rs` 的 A17 守卫今天**写死了布局**，搬迁后必然失败，而且必须**在搬迁前**先改造好——它是拆包「没有改变依赖方向」的可执行验证，不能等拆完再补。

当前写死之处：

| 项 | 位置 | 搬迁后为何失败 |
| --- | --- | --- |
| `A17_EXPECTED_FILES: [&str; 6]` | `tests/p2_contracts.rs:2571` | 六个 `src/…` 路径里 query/xref 会搬到 `crates/jarde-query/src/` |
| `A17_GUARDED_FILES: usize = 6` | `:2584` | 数量断言绑在旧布局 |
| 正对照读 `src/engine.rs` | `:2915` | driver 搬入 jvm 后，门面的 `engine.rs` 不再调用 P2 入口 |
| `derived_p2_type_tokens` 读 `src/environment.rs`/`resolver.rs`/`ir.rs` | `:2685` | 三个文件搬到 `crates/jarde-jvm/src/` |
| `A17_MODULE_TOKENS` 12 条 | `:2602` | 拆包后要补**跨 crate 形态**（`jarde_jvm::`）与 facade 再导出形态 |

**改造要求**：

1. **按模块身份寻址、按布局解析**：把硬编码路径换成 `(模块身份 → 候选路径列表)`，解析时断言**恰好命中一个**；候选同时包含旧布局与拆包后布局。这样同一个守卫在搬迁前后都能跑，且搬完不会因为「枚举不到」而假绿。
2. **token 表加入跨 crate 形态**：`jarde_jvm::`、`jarde_query::`（以及 facade 再导出形态），否则拆包后 query 调 jvm 就绕过了守卫。
3. **正对照迁移**：正对照要指向「按契约允许调用 P2 入口」的那个 crate 的文件（拆包后是 jvm 侧），且断言它**不在**被守集合内。
4. **反例必须仍然有效**：`the_a17_guard_detects_rewritten_references_and_added_files` 的 sandbox 用例要在**两种布局**下各跑一遍（旧布局一套 + 新布局一套），证明守卫不是只认其中一种。
5. **3.2 的依赖闭包门禁**（Cargo 层面的证据）与本守卫**并存**：本守卫看源码 token，门禁看 `cargo tree` 输出；design 明确「`src/` 变空不能让旧字符串守卫假绿」，所以两者都要，且 3.2 要用「临时加一条 query→jvm 依赖」的反例证明门禁会失败。

**1.2 已完成并勾选**。CI：`45328a0`（搬迁本体）→ run 35346239122 **红**（fuzz lock 过期，supply chain 与 fuzz smoke 两 job 失败）；`0aea34b`（lock 修复）→ run [`35346754502`](https://github.com/LordCasser/jarde/actions/runs/35346754502) 四 job 全绿；`d08f014`（accounting 边界修复）→ run [`35347094952`](https://github.com/LordCasser/jarde/actions/runs/35347094952) 四 job 全绿。

**未做**：文件搬迁（2.1 起——抽 `jarde-query`）。cross-check 待办：`ci.yml:81` 的 `jvm` 边界正则需在真实 `cargo tree` 输出上实测不误命中 `jarde-jvm`。
