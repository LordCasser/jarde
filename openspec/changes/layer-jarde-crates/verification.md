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

**未做**：文件搬迁（1.2 进行中）。cross-check 待办：`ci.yml:81` 的 `jvm` 边界正则需在真实 `cargo tree` 输出上实测不误命中 `jarde-jvm`。
