# P2 实施验证记录

以下各片记录保留实施时点。当前状态和本轮发现以文末「2026-09-18 当前工作区复核」为准；本轮仅审查、修订文档，未修复实现。

1.1 历史记录日期：2026-09-17。规划与契约基线 `35fdf6d`。本轮只实现 **1.1**；1.2/1.3 与 2.x–5.x 均未开始，解析、闭包、CFG、SSA 与预算维度扩展都没有实现。所有命令按单作业执行（`CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1`）。

## 1.1 公共契约与最小 API

### 交付

- 契约：`design.md` 的「公共 schema 骨架（1.1，P2 新公共类型的唯一契约来源）」——实现前先做只读复核，8 组问题全部采纳后定稿（环境可读内容与搜索顺序归属、状态平面拆分、dispatch 平面、专用 `ReferenceUse`、声明/方法报告承载面、结构化环境定位与负例语义、serde 约定、1.3 预算维度与 churn）。
- 新模块：`src/environment.rs`（环境绑定、provider 声明、环境问题码与校验）、`src/resolver.rs`（解析与声明引用查询请求/报告）、`src/ir.rs`（方法分析请求/报告、阶段与产物状态）。
- additive 改动：`src/model.rs` 的 `OriginSet`/`OriginMember` 与 `Coverage::not_requested`；`src/budget.rs` 的 `CountedBudgetDimension::ALL`、`Limits::counted_limit`、`UsageSnapshot::counted_usage`（为 1.3 铺路，**未新增维度、未改既有字段**）。
- 入口：`Engine::{resolve_symbol, declaration_references, analyze_method}`；示例 `examples/resolve_and_analyze.rs`（CI 的 example 步骤现在同时运行它与 `inspect_class_header`）。
- 测试：`tests/p2_contracts.rs`（28 条）。

### 行为（本切片的诚实状态）

- 合法请求：`analysis = NotPerformed`、`state = None`（解析/声明查询）或 `stages` 全 `NotPerformed`（方法分析）、`execution = Failed { Unsupported { code: resolution_not_implemented | method_analysis_not_implemented } }` + 同 code 诊断、三维 coverage `NotRequested`、counted usage 全 0（`elapsed_millis` 除外）、`body = NotInspected`、`dispatch = None`。
- 三个入口**不读取 content 字节、不计费、不轮询 Budget**：把 `Limits` 全部置 0（含 elapsed）仍返回 `Ok`，而同一预算下 P1 的 `query`/`enumerate_artifact_tree`/`inspect_header` 都会停止（对照实验见复核记录）；预先取消的 token 也返回 `Failed{Unsupported}` 而不是 `Cancelled`（契约已写明）。
- 请求级失配 → `Err(InvalidInput)`：`resolution_snapshot_mismatch`、`resolution_target_use_mismatch`、`analysis_no_stages`。
- 环境问题（8 个闭集码：`duplicate_loader`、`caller_domain_mismatch`、`missing_parent`、`parent_cycle`、`unsupported_policy`、`unreadable_root`、`content_not_provided`、`provider_root_unbound`）进三份报告的 `environment_problems`，同 code 诊断按问题顺序排在能力码诊断之前且 severity 为 `Error`，subject 结构化（loader/provider/root 序号/symbol）。

### 两处 serde 例外（已写进契约）

- `MethodBodyState::DeclaredWithoutBody` 的内部 tag 与载荷字段不得同名 → 线格式 `{"kind":"declared_without_body","no_body_kind":"abstract|native"}`；旧的双 `kind` 形状反序列化失败（有测试钉住）。
- `EnvironmentSubject` 用外部标记（`{"loader":…}` / `{"provider":…}` / `{"root":{…}}` / `{"symbol":{…}}`），因为 `tag="kind"` 无法序列化字符串载荷的 newtype variant。

### 复核与关闭项

- **契约先行复核**（实现前）：结论"需先改契约"；8 组问题全部采纳（含把 `operation: XrefOperation` 换成专用 `ReferenceUse`、`state` 与 `analysis` 平面拆分、`dispatch` 独立报告字段、环境问题结构化定位、预算维度命名与 1.3 churn）。
- **实现复核**：首轮 **Approve** + F1–F4（诊断断言太弱、两个入口的负路径无测试、A17 守卫有绕过路径、依赖表与代码不符）→ 关闭 → 复审 **Approve**（N1–N3、D1–D5）→ N1–N3 与 D1/D2 已关闭，遗留项登记为债务。
- 关闭后守卫的形态：类型名单从 `src/environment.rs`/`resolver.rs`/`ir.rs` 的公开类型**推导并自证完整**（漏一个即失败），`ALL` 列表与源码变体逐项比对，A17 全覆盖 `src/query.rs` 与 `src/xref/**`（目录枚举，含新增文件）。
- **变异证伪证据**（实现者执行、复核者抽样独立复现）：清空或改码环境诊断 → 8 个测试失败（原 1 个）；逐入口丢弃 `environment_problems` → 各 8 个失败；把环境诊断反转或删条目 → 18 个失败；A17 注入分组别名 `use crate::{resolver as r}`、`use crate::*`、`ir as p2`、裸类型名、新增 `src/xref` 文件、路径空白与分组写法 → 均被精确标记；`AnalysisStage` 加变体不改 `ALL` → 源码比对断言失败（穷尽 `match` 另使漏臂编译失败）；`A17_MIN_SOURCE_LEN` 的旧规则对小文件误报 → 新规则的回归用例捕获。
- **剩余债务（登记）**：A17 守卫是 token 级近似（注释中复述也会命中；`#[path]`、宏拼接、字符串内引用不覆盖），构造级证据由 5.2 的构造计数补强；`declared_variants` 是行式解析（枚举写法变化会响亮失败）；1.3 加维必须同步 `ALL`/`try_from`/CLI 请求 schema/goldens/fuzz 的 usage 断言。

### 证据

- `cargo fmt --all -- --check` 干净；`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净。
- `cargo test --workspace --all-targets --all-features --locked` = **328 passed / 0 failed / 1 ignored**（`ignored` 仍为显式 JDK 25 oracle；基线对照：P1 归档时 299 + 28 条契约测试 + 1 条 budget 单测 = 328）；`tests/p2_contracts.rs` 28 条。
- `cargo run --example resolve_and_analyze --locked -- tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class` exit 0，输出诚实状态与缺 parent 的 `EnvironmentProblem`。
- 以上命令由主 Agent 独立复跑确认；生产代码在复核修正轮中零改动（`src/**` 与首轮交付逐字节一致）。

### 1.2 reader 层类型化操作数与目标校验

- 交付：`src/classfile.rs` 新增 crate-private 的 `InstructionOperands`/`ImmediateValue`/`LocalOperand`/`SwitchOperands`，`MethodCodeFacts.operands` 与 `instructions` 同序同长；`MethodCodeFacts::control_flow_targets` 产出 `ControlFlowTarget`/`ControlFlowTargetKind`（branch/default/case/handler）。公共 `InstructionFact`/`BytecodeInspection`/`inspect_method_bytecode` 与计费口径**逐字段未变**（复核者在改动前后对 16 个 fixture 做公共 JSON 对照：剥 `elapsed_millis` 后逐字节相同；新增代码零 `charge`/`poll`）。
- 校验：目标必须是指令起点、不得越出 `code_length`（含负向换算落到 BCI 0 之前）、switch default 与每个 case 同规则、`handler_pc` 是指令起点；保护区间按 JVMS 4.7.3 与 `specs/jvm-ir` 收紧为 `start_pc` 是指令起点、`end_pc` 是指令起点或恰为 `code_length`、`start_pc < end_pc`（空区间非法）。错误码分工：区间关系用 `classfile_exception_range_invalid`，端点/入口不在起点用 `classfile_instruction_invalid_target`，越界用 `classfile_instruction_target_out_of_bounds`。
- 反例与证伪：F1（区间端点对齐/空区间）由主 Agent 裁定为**收紧**（JVMS 与 spec 字面都要求；复核者量化现有 15 个 fixture、42 个 body、8 条异常表记录的未对齐与空区间各为 0，故零回归）。9 个变异全部被新老测试捕获：switch default 不校验、`jsr`/`jsr_w` 缺失、符号扩展丢失、`code_length` 差一、未覆盖的 `wide` 族、四条 F1 子规则。首轮复核另列 5 组"实现正确但测试不敏感"的盲点（switch default、`jsr`、负数操作数、`wide astore/fload`、`target == code_length` 错误码），已由 7 条新测试补齐。
- `repository_class_fixtures_validate_without_false_target_rejections`：递归扫描 `tests/fixtures/**` 的 15 个 class，全部 `class_facts` + `method_code_facts` + `control_flow_targets` 通过，并核对种群与 8 条 `jsr/jsr_w` 目标——真实历史 `finally` 子程序走同一套校验，收紧规则不拒绝既有语料。
- 证据：单作业下 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`cargo test --workspace --all-targets --all-features --locked` = **348 passed / 0 failed / 1 ignored**，`cargo test --lib` = **114 passed**；由主 Agent 独立复跑确认。
- 已知债务（登记，不阻塞）：`control_flow_targets` 对未完整解码的 body 是"可靠但不完备"的视图（不会放过非法目标，但会把未读后缀里的合法目标报成非法），调用方 MUST 先查 `execution`/`stopped_at`，3.x 消费前若需类型级保护再收紧；`newarray` atype、`multianewarray` dimensions、`invokeinterface` count 有意不保留（4.1/4.2 需要时先改契约）；`immediate` 只表示 CP 条目字面量类型，不承担 ldc 变体与 tag 配对合法性；操作数 facts 约 80 B/指令的派生放大只按 `CodeBytes` 1:1 计费，1.3 的 `IrItems` 记账或写明上界。
- 远端 CI：实现与文档提交 `9f618e6`、`e4f6bdb` 推送 `main` 后，CI run [`35251066394`](https://github.com/LordCasser/jarde/actions/runs/35251066394) 四个 job 全部 success（`stable` 含 ignored JDK 25 oracle、`MSRV 1.88.0`、双 workspace `supply chain`、`fuzz smoke`）。
- 事故记录：本轮实现过程中，coder 在变异实验里误用 `git checkout -- src/classfile.rs` 回退了未提交实现，随后从会话快照恢复并重放本轮改动。主 Agent 独立核实：HEAD 仍为 `6f820ab`（未提交任何东西）、工作树两处改动完好、文件 sha256 `3ba8b359…` 与 coder 声称一致、`git show HEAD:src/classfile.rs` 仍是 P2 前基线、20 个 1.2 测试函数全部在位、全量测试与仓库语料测试通过。结论：无内容丢失；后续变异实验一律用文件副本还原，不得触碰 git。

### 1.3 预算维度扩展（含 churn 同步）

- 维度面：`CountedBudgetDimension` 9 → 15（`ClassHeaders`/`MethodBodies`/`IrItems`/`IrEdges`/`AnalysisSteps`/`NormalizationClones`），`BudgetDimension` 11 → 18（新增非累加高水位 `DependencyDepth`，插在 `NestedDepth` 与 `ElapsedMillis` 之间）；`ALL`/`counted_limit`/`counted_usage`/`add`/`get`/`From`/`TryFrom` 全部同步，`TryFrom` 拒绝集恰为 `{NestedDepth, DependencyDepth, ElapsedMillis}`。新增 `Budget::observe_dependency_depth`（与 `check_nested_depth` 同形：超限报 `DependencyDepth` 且 `consumed = depth-1`，否则取 `max` 高水位），两者相互独立。新增 `impl Default for Limits`（**全零、fail-closed**，文档写明是测试/工具基底而非隐式生产限额）。
- 计数单位按契约表：`ClassHeaders`/`MethodBodies` 计读取**尝试**（同 (definition, loader) 去重由调用方负责）；`IrItems`/`IrEdges`/`AnalysisSteps`/`NormalizationClones` 由 3.x/4.x 在分配/入队/加边/克隆**之前**计费；1.3 只交付维度与计费入口，生产代码暂无调用方（契约明示）。
- churn 同步（契约表逐项）：42 处需改字段的 `Limits` 字面量（41 个 `Limits {` + CLI `From` 的 1 个 `Self { }`）全部更新——26 处加 `..Limits::default()`、其余经既有 helper 继承，既有维度取值逐字段未变；CLI `RequestLimits` 11 → 18 个**必填**字段（`deny_unknown_fields` 保留、无 `serde(default)`），`From` 穷尽 18 字段；5 个 P1 golden 的 24 个 usage 对象**纯加法**（7 个零值键按字母序插入，既有键值/顺序不变）；`fuzz/src/lib.rs::assert_usage` 改为 `CountedBudgetDimension::ALL` 遍历 + 两个 depth 单独断言；`docs/support-matrix.md`/`README.md` 更新为十八项并写明计数含义与"P2 维度尚未真正计费（归 3.5/4.3）"；另同步 `src/artifact.rs::budget_dimension_code`、`src/xref/bootstrap.rs::dimension_counts`、`src/classfile.rs` 的 usage 键集合断言（18 键，仍精确）、`examples/resolve_and_analyze.rs`。
- **CLI JSON 契约的有意变更**（须记录）：请求对象从 11 项变为 18 项必填；旧请求现在返回协议错误（实测 11 字段请求 → `cli_request_json: missing field \`class_headers\``，17 字段缺 `dependency_depth` 同样被拒，未知键被 `deny_unknown_fields` 拒绝）。库侧 `Limits`/`UsageSnapshot` 同样逐字段必填，新键不能省略。README 两个示例已同步并实跑通过。
- 反例与证伪（复核者独立复现 11 组变异，全部被捕获并用副本还原核对 sha256）：两个 depth 共用槽/共用 limit、`AnalysisSteps` 写入 `IrItems` 槽、`IrEdges` 写入 `IrItems` 槽、`Default` 非零、高水位改用赋值而非 `max`、`consumed` 去掉 `-1`、`ensure_within` 用 `<` 而非 `<=`、跳过取消检查、`ALL` 截断或换序（后者同时被两处锚点抓到）。
- 主 Agent 的补充收口：`tests/p1_query_bounds.rs::assert_within` 改为 `ALL` 驱动（并新增自检 `the_within_bound_rejects_every_dimension_over_its_limit`，逐维验证"恰好通过 / 超一单位被拒"，同时与 `UsageSnapshot` 的序列化 schema 对表）；`budget_dimension_code` 的 18 个诊断码加穷尽 `match` + serde 名对照断言（此前新增码改错会静默出厂）。两处均以变异证伪（截断遍历、改回手写、错误码改名）。
- 证据：单作业下 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`cargo test --workspace --all-targets --all-features --locked` = **361 passed / 0 failed / 1 ignored**，`cargo test --lib` = **126**、`cargo test -p jarde-cli` 全绿、fuzz workspace `cargo test` = **9**；由主 Agent 独立复跑确认。
- 复核结论：**Approve**（实现满足契约；唯一阻塞项是本节尚未登记，已由本次写入关闭）。复核者另核对：golden 纯加法、既有断言未被削弱（`src/**`/`tests/**` 除契约相关外全为纯新增行）、`observe_dependency_depth`/`check_nested_depth` 归一化后同形且既有 `charge`/`check`/`poll`/`usage` 函数体与 HEAD 逐字节相同、`Cargo.*`/`deny.toml`/`.github/**`/`fuzz/Cargo.*` 未动、无新增依赖。
- 债务（登记，不阻塞）：`fuzz/README.md` 关于"every `usage` field stays inside the limit"的措辞略宽（`elapsed_millis` 有意不查），下次文档同步顺手收紧；P2 六个计费维度与 `dependency_depth` 的真实派生膨胀停止行为由 3.5/4.3 验收（支持矩阵已声明）。



本切片以两个提交推送 `main`：`0406178`（实现与测试）与 `6fc1674`（本 change 的契约与验证记录）。CI run [`35247034235`](https://github.com/LordCasser/jarde/actions/runs/35247034235) 在 `6fc1674` 上四个 job 全部 success：`stable / test and specification`（含 ignored JDK 25 指令边界 oracle 与两条公共示例）、`MSRV 1.88.0`、`supply chain`（根与 fuzz 两个依赖图）、`fuzz smoke`。1.2 的提交与 CI 在其小节内记录。

## 2.x 切片

### 2.1 Header providers 与搜索顺序

- 交付：新增 crate-private `src/providers.rs`（有效序列、位置展开、候选匹配、读取与计费；`resolver → providers → {view, environment, artifact, classfile, budget, model, error}`，无反向边、不新增公共类型）；`src/resolver.rs` 的类符号查找接入（成员符号仍 `NotPerformed`）；`src/environment.rs` 新增 `CallerLoaderMismatch`（闭集 9 项）；`PhysicalVariant` 路径派生从 `src/xref/mod.rs` **原样搬到** `src/model.rs`（`pub(crate) physical_variant_for_path`，P1 与 P2 共享同一身份规则，行为不变、P1 golden 全绿）。
- 搜索模型（契约）：`ChildFirst` 的 loader 序为 `roots(l) ++ R(parent)`、`ParentFirst` 为 `R(parent) ++ roots(l)`，每个 loader 内部按 `roots` 声明顺序；混合链按各自 delegation 递归展开。root 内候选是 `raw_name == internal_name + b".class"`（字节精确、大小写敏感、目录名以 `/` 结尾天然不匹配）；standalone CLASS root 按 `this_class` 比对；ArtifactTree 只在 origin 相等的 container 内搜索。**首个匹配胜出且不回退**（解码失败、读取失败、listing 停摆、预算/取消都不回退）；`Ambiguous` 只出现在同一选择位置（同名不同字节，以及同名同字节的重复 entry，不按 ordinal 猜）。
- 平面映射（本轮修正，按 `specs/demand-resolver` 的"另行记录输入损坏、取消和实际 execution"）：`state = Some` 当且仅当到达语义判定；取消 = `Performed` + `None` + `Cancelled`；输入损坏/listing 停摆/读取失败 = `Performed` + `None` + `Failed{Error{code}}` + 带 origin 诊断；预算停止 = `Some(BudgetExceeded)`；能力未运行（成员符号、环境被拒）= `NotPerformed` + `None`。`Inaccessible`/`IncompatibleClassChange` 在解析路径上不再被占用（保留给 2.3/2.5 的访问与链接规则）。
- 计费与覆盖：每次 Header **读取尝试**计一次 `ClassHeaders`（含失败尝试、Ambiguous 的每个候选、standalone 不匹配）；枚举/读取沿用既有维度，不新建索引；performed 查找按"首个命中即停"记 `runtime_resolution` 为 `CompleteWithinSchema` + `scanned=[0,examined)` 且无 skipped，仅预算/取消/损坏才 `Partial` + `skipped=[examined,positions)`。任一 1.1 环境问题都拒绝整次查找（零读取）。
- 能力码：类查找执行时若请求带 `dispatch`，追加 `dispatch_not_implemented`（Warning，2.5 消失）；成员符号仍是 `resolution_not_implemented`（2.3 落地后消失）。
- 反例与证伪：实现者 3 组变异（ParentFirst/ChildFirst 写反、损坏候选静默回退、Ambiguous 取首个）与主 Agent 契约修正后的 3 组变异（损坏映射改回 `Inaccessible`、取消改回 `NotPerformed`、去掉 `CallerLoaderMismatch`）全部被捕获；复核者另做 9 组变异 + 10 条独立探针（混合链 6 种 delegation 组合、字节精确候选与 MR 路径变体、3 候选歧义与 container 作用域、EOCD 结构损坏、读取层 CRC 损坏、无关 domain 拒绝、member/dispatch 平面、MR/uncertainty 未应用、caller 双检查组合），其中 8 组被交付物测试捕获；**M8（把计费移到读取成功后）只被探针捕获**，已按复核要求补一条"读取层失败"用例（失败尝试计一次 `ClassHeaders`、不回退、`scanned=[]`/`skipped=[0,positions)`）。
- 证据：单作业下 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`cargo test --workspace --all-targets --all-features --locked` = **385 passed / 0 failed / 1 ignored**（`p2_resolution` 16、`p2_contracts` 29、`src/providers.rs` lib 单测 7），示例 exit 0 并打印 `state=Resolved`/`class_headers=1`；由主 Agent 独立复跑确认。
- 远端 CI：实现与文档提交 `2d25ce0`、`07b771b` 推送 `main` 后，CI run [`35258105942`](https://github.com/LordCasser/jarde/actions/runs/35258105942) 四个 job 全部 success。
- 独立复核结论：**Approve**（"必须改"两项：本条记录与读取层失败用例，均已处理）。登记的债务：`output_bytes` 在 standalone（`root_bytes`）与 ZIP/tree（`read_entry_internal`）之间口径不对称，2.2/5.3 按维度断言前须先钉死；skipped 的语义是"未达判定的 position"而非"未触及"；带目录属性但名字不以 `/` 结尾的 entry 会被当候选并报解码失败（artifact 层无目录位）；ArtifactTree position 每次查找都枚举整棵树（2.2 多次查找会重复计费，P5 索引前）；`MethodAnalysisRequest` 侧没有"无 caller 时不报"的显式用例；2.1 不应用 `RuntimeProfile` 的 release/multi-release 与 uncertainty 判定（MR 由 `select_multi_release` 提供，uncertain-runtime 诊断归 2.5）。

### 2.2 按需 Header 闭包、读取 reason 与去重

- 交付：`src/providers.rs` 的 `HeaderClosure`（键 `(loader, internal_name)` 记忆已判定需求、`HeaderDemand::{RequestedDefinition, ParentChain, HierarchyClosure}`、`parent_chain`/`hierarchy_closure` 展开、`WalkGaps{missing, ambiguous, cycles}` 与停止粘滞）；`src/resolver.rs` 的 `ReadReason`（6 项 snake_case）与 `HeaderRead{loader, definition, reason}`，三个报告各加 `reads`（**类型面 additive，wire 面是 `deny_unknown_fields` 下的必填新字段**，按 1.3 先例记为有意的 schema 变更）；类符号路径改经闭包并发布 `reads`。
- 语义：同 (definition, loader) 只记一次（reason 取首次读取的需求）、只有成功读取且取得定义身份才入记录、Ambiguous 每个候选各一条、standalone 声明别名也记；读取尝试记 `ClassHeaders`、层级展开每层记 `AnalysisSteps` 并在**扩展前**观察 `dependency_depth`；闭包不读任何 Body（公开证据是 `code_bytes == 0`，配 `inspect_method_bytecode` 的非零对照）；停止时 `reads` 与 `coverage` 一起发布可信前缀，停止的需求不被记忆（重问会重新搜索）。
- 反例与证伪：实现者 6 组变异（记忆键去掉 loader 分量、记录用起点 loader、去掉记录去重、去掉深度观察、"扩展后停止"、停止报告成 Missing）中 5 组被捕获；复核者 20 组变异 + 5 条独立探针（深链 limit∈{0,1,2,3}、真实 class 的 body 计费对照、Ambiguous 超类、损坏候选、停止后重问等）中 10 组被交付测试捕获，其余 6 类语义（`parent_chain` 不跟接口、损坏/读取失败不入记录、Ambiguous 每候选一条、停止的需求不被记忆、`gaps.ambiguous`、自环诊断）由**复核指出缺测**后补齐（6 组新变异逐一证伪）。复核另指出公开用例曾用恒真的 `method_bodies == 0` 充当"不读 Body"证据（该维度在 3.x 前无计费点），已改为 `code_bytes == 0` + 真实 body 路径对照。
- 证据：单作业下 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`cargo test --workspace --all-targets --all-features --locked` = **407 passed / 0 failed / 1 ignored**，`cargo test --lib` = **145**、`p2_closure` = **10**；示例输出 `resolve_symbol.class: reads=[("app", RequestedDefinition)]`；由主 Agent 独立复跑确认。
- 远端 CI：实现与文档提交 `da47836`、`6f5f0b2` 推送 `main` 后，CI run [`35263295922`](https://github.com/LordCasser/jarde/actions/runs/35263295922) 四个 job 全部 success。
- 独立复核结论：**Approve**（三项"必须改"：`ir.rs` 公开 doc 与 `ReadReason::DriverMethodBody` 自相矛盾、恒真的 Body 证据、4 处语义无测试；均已关闭）。登记债务：闭包键 loader 分量在当前公开路径不可证伪（触发条件是"出现第一个以非调用方 loader 发起需求的调用方"，可能晚于 2.5）；`remembered`/`record_read`/`visited` 的线性扫描在 2.5 全 scope 枚举前需索引（P5）；`analysis_steps` 耗尽路径无用例；`providers.rs` 3 处冗余 `#[allow(dead_code)]`；`method_bodies` 在 3.x 接通计费前没有计费点，任何以它证明"不读 Body"的断言都不成立。

### 3.1 petgraph 准入证据

准入对象：`petgraph 0.8.3`（2025-09-30 发布，当前最新且未 yanked，仓库 `petgraph/petgraph` 活跃，包内 `.cargo_vcs_info.json` SHA1 `162903562ce5b00cdba390a0d9c1bb80f1c75bf5`）。**结论：准入通过，按四条硬约束引入**（约束已写入 design 的「3.1 准入证据与使用约束」与 `openspec/dependencies.md`）。

| 准入项 | 实证结果 |
| --- | --- |
| MSRV 1.88 | `rustup run 1.88.0 cargo check --locked`（默认与最小 feature 两种、含 `--all-targets`）与行为测试在 1.88.0 下全部通过；petgraph 声明 `rust-version = 1.64`，实际下限由传递依赖决定（indexmap/hashbrown 1.85），有效下限 1.85 ≤ 1.88 |
| 许可与纯 Rust | `MIT OR Apache-2.0`；无 `build.rs`、无 C/C++、无 `cc`/`bindgen`；`cargo-deny`（仓库原样 policy）在两个 feature 集合下四段 ok，唯一告警是 hashbrown 双版本（`multiple-versions = "warn"`）；RustSec 唯一相关条目 RUSTSEC-2024-0402 只影响 `=0.15.0`，实际使用 0.15.5 |
| feature 集合 | 推荐最小集 `default-features = false, features = ["std"]`（normal 依赖 7 个）；默认 feature 会写入 serde 家族、`rayon` 会引入后台线程池、`all` 引入 dot-parser/quickcheck |
| 平行边 | 同对节点 3 条有向边（1 普通 + 2 异常）计数与枚举完整保留；`edges_directed` 按加入逆序、`find_edge` 返回最近加入者（实现不得依赖该顺序，另见下条） |
| 不可达节点 / 自环 / 孤立点 | `kosaraju_scc` 与 `tarjan_scc` 都覆盖全部节点，自环与孤立点各成单元素分量；`toposort` 对环与自环显式返回 `Err(Cycle)` |
| 多出口 | 后支配需合成 super-exit（`simple_fast(Reversed(&g), super_exit)`）；入口 `immediate_dominator` 为 `None`、`dominators` 为自身；不可达节点的 dominators 三 API 全 `None` |
| **确定性（关键）** | `kosaraju_scc`/`tarjan_scc`/`toposort`/图遍历不依赖 hash（跨进程逐字节可复现），但顺序是插入顺序/`NodeIndex` 的函数；**`Dominators::immediately_dominated_by` 用 hashbrown `HashMap`，同一二进制不同进程顺序不同**（实测 5 次运行出现 4 种顺序）。→ 必须自己按 (物理定义, BCI) 排序 |
| 预算/取消粒度 | 源码无任何中断/预算/超时钩子；四个目标 API 无取消参数。→ 记录为「阶段级不可取消：进入前检查、结束后再检查，靠规模上界保证单次调用有界」；实测规模（release、macOS aarch64）SCC/拓扑 ~13 ns/点·边、`Graph` 52 B/点 + 26 B/边、支配树 ≈195 B/block、65 535 点 `simple_fast` ≈7.7 ms / 12.6 MB |
| **进程 abort 风险（关键）** | `tarjan_scc` 递归：2 MiB 栈在 20 000 节点链状图即 stack overflow 且 **abort 进程**（不可捕获）；`kosaraju_scc` 迭代，512 KiB 栈 100 000 节点正常 → SCC 只用 `kosaraju_scc`。`simple_fast` 对不属于该图的 root 在 debug 与 release 都 panic → 调用前自校验 root 归属；构图后不得 `remove_node`（swap-remove 会改写索引） |

复现入口：`/tmp/p2-petgraph-probe/`（`ADMISSION-EVIDENCE.md` 汇总；`algoprobe` 含 10 个行为测试、规模/深度/顺序探针；`feature-matrix.txt`、`scale-*.txt`、`depth-thresholds.txt`、`order-run*.txt`）。该目录是临时证据，不入库；本节的表格与命令是长期记录。

**依赖引入（2026-09-18，与准入证据同片）**

- `Cargo.toml`：`petgraph = { version = "=0.8.3", default-features = false, features = ["std"] }`（精确版本 + 最小 feature；依赖声明处注释说明「准入已通过、首个消费者是 3.3」）。全仓库 `*.rs` 中 `petgraph` 出现 **0 次**：引入不等于可用，3.1/3.2 都不产出图形算法调用。
- 依赖树（`cargo tree --workspace --all-features --locked -e normal` 的唯一差异）：`petgraph v0.8.3 → fixedbitset v0.5.7`、`hashbrown v0.15.5 → foldhash v0.1.5`、`indexmap v2.14.2`（复用既有）。**没有** `rayon`/`serde-1`/`cc`/`bindgen` 进入生产树；`indexmap`/`hashbrown 0.17.1` 本就在树中（经 noak）。
- lockfile：`Cargo.lock` 与 `fuzz/Cargo.lock` 各 34 insertions / 1 deletion——新增 4 个包与 jarde 依赖表项，唯一删除行是 `indexmap` 依赖表的 `"hashbrown"` → `"hashbrown 0.17.1"` 消歧；**没有任何既有包被顺带升级**（复核者用 (name, version) 集合与依赖表逐条重算确认）。`fuzz/Cargo.lock` 必须同改：`fuzz` 是独立 workspace 且以 `path = ".."` 依赖根 crate，不改会让 CI 的 `--locked` 步骤直接失败（复核者在副本上还原该文件复现了 exit 101）。
- CI：normal-tree 禁令只删 `petgraph([^[:alnum:]_]|$)|` 一项（其余 **18** 个边界逐字节未动）；feature-tree 步骤加**双向**断言——正向 `grep -F 'petgraph feature "std"'`、负向禁止 `graphmap|stable_graph|matrix_graph|rayon|serde-1|serde|serde_derive|all|quickcheck|dot_parser|unstable|generate`。复核者端到端证实：把 `default-features = false` 删掉后该步骤 exit 1 而 normal-tree 步骤仍 exit 0（即这条负向断言不可替代），把 `features = ["std"]` 换成 `[]` 也能被抓到；18 个注入的假边界行 18/18 命中，`petgraph`/`jvmti-sys`/`rayon_core_extra`/`redisx` 等近似名不误报。
- 证据：fmt 干净；clippy `-D warnings` 0 警告；`cargo test --workspace --all-targets --all-features --locked` = **487 passed / 0 failed / 1 ignored**（含 CI 第二 seed）；`rustup run 1.88.0 cargo check --workspace --all-targets --locked` 通过；两个 workspace 的 `cargo deny` 四段 ok（唯一新增告警是 hashbrown 0.15.5/0.17.1 的 `duplicate` warn，即 `multiple-versions = "warn"` 的既有策略，未改 `deny.toml`）；`ci.yml` 两段脚本本地等价复跑 exit 0。
- 远端 CI：依赖与文档提交 `b30fe9e`、`c7c9c45` 推送 `main` 后，CI run [`35278388875`](https://github.com/LordCasser/jarde/actions/runs/35278388875) 四个 job 全部 success——这也是新增的双向 feature 断言与 petgraph 进入生产树在真实 CI（Linux x86_64）上的首次验证。
- 独立复核结论：**Approve**（依赖引入本身正确；关键声称均可独立复现，CI 负向断言的实际强度高于声称）。
- 登记债务：**A17 守卫缺口已由复核者证实**——引入依赖后，在受守卫文件（如 `src/query.rs`）里 `use petgraph::…` 并构图**能编译且守卫测试仍通过**（改动前该代码根本无法编译）。`tasks.md` 3.1 限定「仅调整 CI 禁令」，守卫扩展属 P2 退出前项，因此不阻塞本片勾选，但**必须在 3.3 首个消费者落地时同步补守卫**（把图算法 crate 的导入加入 `p2_tokens_in` 的 token 表，并用「注入 `use petgraph::…` → 测试转红」证伪）；另：normal-tree 禁令移除 petgraph 后，第二个 petgraph 版本只剩 `cargo-deny` 的 warn 可见；升级门槛（重推 feature 名清单）依赖人工，属升级前清单项。

未验证/遗留：`simple_fast` 最坏 O(|V|²) 未复现（block 上限是唯一保险）；递归/栈只测了链状深图；跨平台（CI 的 Linux x86_64）未复跑；依赖引入后需重跑 MSRV、两个图的 `cargo deny` 与 feature-tree 断言。

### 2.3 成员解析（JVMS 5.4.3 / 5.4.4）与调用种类规则

- 交付：新增 crate-private `src/members.rs`（字段/class method/interface method 三条搜索路径、maximally-specific 集合、访问与调用种类规则、sig-poly 与数组 owner 分支）；`src/resolver.rs` 把成员符号从 `NotPerformed` 接成真解析（`resolved` 只在 `Resolved` 发布，声明符号与请求符号都可见）；`src/providers.rs` 增加 `SupertypeEdge` 与 root reason 参数，使 `ReadReason` 按语义拆分（`super_class` 边 → `ParentChain`、`interfaces` 边 → `HierarchyClosure`、按身份命名 → `MemberOwner`，两个变体都有真实生产者、映射仍穷尽）。
- 语义：字段（自身 → 超接口递归 → 超类）、class method（类链 → 超接口 maximally-specific）、interface method（该接口 → 其超接口）；**maximally-specific 按 JVMS 5.4.3.3/5.4.3.4 排除 `ACC_STATIC`/`ACC_PRIVATE`**（集合为空即 `Missing`，不误报 conflict），owner 直接点名的 static 仍可解析（`invokestatic` → `Resolved`，`invokeinterface` → ICCE）；default conflict → `IncompatibleClassChange`；全抽象 → `Resolved` + Warning；访问规则按 JVMS 5.4.4 且"调用方未知/层级读不全"时报 `resolution_access_not_checked` 而不谎称已检查；sig-poly 按 name 匹配并给出 `target`/`resolved.member` 描述符不同的证据；数组 owner → `UnsupportedPolicy`。
- 反例与证伪：首轮复核 **Reject**——发现接口步未排除 `ACC_STATIC`/`ACC_PRIVATE`（合法 Java 8 的 static+default 组合被误报 `resolution_default_conflict`；类 owner 形态把应 `Missing` 的引用判成 ICCE），并指出"直接超接口声明序"零覆盖（变异 M3a 在 37 条全绿下存活）。修正后复核 **Approve**：上轮 2 条失败探针转绿，6 组新变异（顺序反转、只过滤 static、过滤泄漏进 `matching()`、完全不过滤、`InvokeDynamic` 误加规则、空集改报 conflict）与上轮存活变异 M3a 全部被捕获；owner 点名 static/private、类链未被误过滤等边界由探针独立复核。
- 证据：单作业下 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`cargo test --workspace --all-targets --all-features --locked` = **453 passed / 0 failed / 1 ignored**（`p2_members` 46、lib 145、`p2_closure` 10、`p2_contracts` 29）；示例 exit 0；由主 Agent 独立复跑确认。
- 远端 CI：实现与文档提交 `d8857bb`、`3b056f5` 推送 `main` 后，CI run [`35270265686`](https://github.com/LordCasser/jarde/actions/runs/35270265686) 四个 job 全部 success。
- 语义边界（已写入 `specs/demand-resolver` 的边界段，不得读作 JVMS 完全实现）：default conflict 在解析期报告（JVMS 8 放在 invocation selection）；interface owner 不隐式继承 `java/lang/Object` 的方法；只检查成员自身声明的可访问性（不查声明类，JVMS 5.4.3.1）；`InvokeDynamic` 的 owner 只是搜索起点；同一 owner 内同名同描述符重复声明只能表达为 `Ambiguous`。
- 登记的债务：调用方层级成环时 `subtype_of` 静默跳过 → 判 `Inaccessible` 且无环诊断（与声明侧不对称）；sig-poly 在调用点描述符恰好等于声明描述符时仍发"两者按规则不同"的文案；`read_definition` 的记录挂在请求声明的 `caller.loader` 上（`PhysicalDefinitionId` 不含 loader，API 内不可校验）；成员 coverage 的求和语义与"推导有效序之前停止则区间为空"已写入契约；`HierarchyWalk` 的逐层 reason 仍只由 `providers` 的 lib 单测固定（公开消费者是 2.5）。

### 2.4 声明引用查询（复用结构 consumer）

- 交付：`src/xref/mod.rs` 增 crate-private `CandidateFilter`（`Exact` / `MemberShape{name,descriptor}`（owner 不参与）/ `SignaturePolymorphic{owner,name}`）与 `scan_candidates`；三个 consumer 子模块改走同一 `candidate_matches`/`published_target` 决策点（`resource.rs` 按类型形状天然不参与）；`src/resolver.rs` 的 `Engine::declaration_references` 从诚实不可用变为真查询（成员形状候选 → 逐条 2.1+2.3 解析 → 只发布解析到请求声明的候选；未决候选计数 + 保留 use-site；`max_items` 截断按 P1 页限；scope 校验与 P1 同规则）。
- 语义：未使用的常量池条目不是引用；"解析到别的声明"既不是 item 也不是未决；损坏候选是**扫描级停止**（不进未决计数）；请求级环境问题与"被拒环境"分属两条路径；**每条进入报告的 item 与诊断各计一次 `ResultItems`**（环境平面与停止解释两类元数据不收费，与 P1 同纪律）；解析停止在 `execution` 上优先于装配停止。
- 反例与证伪：首轮复核 **Approve** + F1/F2/F3（签名多态站点被静默漏报、`ArtifactTree` root 未校验、报告条目不计费）→ 修正 → 复审 **Approve**（loader 轴不可证伪、诊断计费口径、停止归属、计数成对）→ 再修正 → 三轮 **Approve**。全过程 30+ 组变异：过滤器退回 owner 精确、把别的声明当命中、未决当已排除、`max_items` 不下传、身份比较丢 loader、sig-poly 忽略 name/owner、tree root 大小写、诊断不收费、停止不去重等全部被捕获；P1 的 23 条查询差分矩阵（2.4 前树 vs 当前树）除 `elapsed_millis` 外**逐字段相同**，且该矩阵本身有判别力（注入 owner 精确匹配或去掉 descriptor 类型判定即 DIFFERS）。
- A11 端到端：`Base.foo` 在 `Sub` 调用时 P1 的 `mentions_symbol(Base.foo)` 不展开 owner，而声明引用查询返回该 use-site（`referenced` 保留 `Sub` 符号、`resolved` 指向 `Base` 声明、origin 为调用点 BCI）；同一链路有 `Mid`/无关层级/异描述符/未消费 `Methodref` 的对照。
- 证据：单作业下 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`cargo test --workspace --all-targets --all-features --locked` = **487 passed / 0 failed / 1 ignored**（`p2_declaration_refs` 34、`p1_xref_golden` 5、`p2_members` 46）；示例 exit 0；由主 Agent 独立复跑确认。
- 远端 CI：实现与文档提交 `2da3abe`、`d94206e` 推送 `main` 后，CI run [`35277379198`](https://github.com/LordCasser/jarde/actions/runs/35277379198) 四个 job 全部 success。
- 独立复核结论：**Approve**（三轮）。登记债务：closure 自身诊断的计费循环当前无生产者（2.5 接上后生效，代码 fail-safe）；`DeclarationRefQuery.consumers.version` 不校验（`Engine::query` 会拒绝非 1）；"解析到别的声明"无独立报告字段（由 `reads`/usage 观察）；签名多态与 `MemberShape` 共享 2.3 的 name-only 近似。

### 2.5 已知范围 dispatch 与 open-world

- 交付：新增 crate-private `src/dispatch.rs`（范围枚举沿用 P1 的 scope 词汇、CHA-lite 候选发现、open-world 事实分类与固定优先级）；`src/resolver.rs` 的 `resolve_symbol` 接入 dispatch 平面（`state = Resolved` 且成员形状可表达时才运行）、`DispatchReport { scope, candidates, open_world }` 与 `DispatchCandidate { member, evidence: Option<..> }`；`read_definition`/`searched_extent` 等 2.2 机器的复用；旧 `dispatch_not_implemented` 全链路移除（改为真报告或 `resolution_dispatch_no_declaration` 说明性诊断）。
- 语义：候选规则**结构性**（自身声明同 kind/name/descriptor 且声明 owner 严格在其超类型路径之上；不筛 private/static/abstract、不排除 `<init>`）；每个候选附证据（缺失依赖 > 外部内容 > 链外 loader > 运行时不确定性 > 非首个 root），`OrderedRoot` 仅在 `index > 0` 时成立；`open_world` 与 coverage 是独立平面，**单一候选也不声称唯一**（报告无任何唯一目标字段，键集合由 serde 断言钉住）；只读 Header（证据是 `code_bytes == 0` + 真实 body 对照，不用无计费点的 `method_bodies`）。
- 覆盖与计费：查找停（`ClassHeaders`/`DependencyDepth`/listing 截断/取消/损坏）保留 `skipped = [examined, positions)`；**发布停**（候选 `ResultItems` 被拒）标 `Partial` 但**不伪造** skipped（由 `DispatchStop::ended_a_search()` 显式区分）；进入报告的诊断（含闭包自报的 `resolution_hierarchy_cycle`）与随扫描增长的列表条目各计一次 `ResultItems`；判定证据字段（`resolved`/`candidates`）不单独计费；停止⇒`open_world` 只有一处实现；发布阶段已停下的请求不再启动 dispatch 平面。
- 三轮复核的证伪链：首轮「有条件 Approve」→ 三项必须改（文档残留、覆盖平面丢 skipped 违反 spec MUST、闭包诊断未计费，前者主 Agent 修文档、后两者改实现）→ 复审 **Reject**（不是实现缺陷，而是三条契约化语义在 521 条测试下**变异存活**：结构性候选规则、规则诊断逐条计费、`Truncated` 保留 skipped）→ 补三条用例 → 有界复核 **Approve**（三组变异现在各被对应新用例捕获，且整仓只有该用例失败；另拆出只筛 private/static/abstract 三种单标志变异，均被捕获）。
- 证据：单作业下 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`cargo test --workspace --all-targets --all-features --locked` = **525 passed / 0 failed / 1 ignored**（`p2_dispatch` 32、`p1_xref_golden` 5、lib 151）；示例 exit 0；由主 Agent 独立复跑确认。
- 远端 CI：实现与文档提交 `12e735c`、`915bd6f` 推送 `main` 后，CI run [`35285853954`](https://github.com/LordCasser/jarde/actions/runs/35285853954) 四个 job 全部 success——**2.x 全片（2.1–2.5）至此完成、复核并全绿**。
- 独立复核结论：**Approve**（三轮）。登记债务：`skipped` 在多次查找共享请求时是「未检查位置」的保守上界（高报未决、绝不低报；5.3 收紧为逐查找集合或在 golden 固定）；`resolve_symbol` 的判定证据字段不单独计费（契约已明确口径，由绝对账单边界守护）；`DependencyDepth` 在 dispatch 级的停止现已有用例；`resolve_symbol` 报告里 `resolved`/`candidates` 无逐条计费；范围枚举每次 demand 重跑容器枚举（P5 索引前）；range 的中途取消公开不可构造（预取消已覆盖）；`SnapshotAll` 与 `ArtifactTree` 的差异已有对照用例。

### 3.2 Pass 契约与 invalidation 校验

- 交付：新增 crate-private `src/passes.rs`（`IrPhase`/`FactKind`/`PassBudgetClass`/`PassDescriptor`/`PASSES` 静态表/`FactLedger`/`validate_requested_stages`）；`src/engine.rs` 在 `ir::validate_request` 之后接入启动校验（错误为 `Error::InvalidInput{code}`）；`src/lib.rs` 加私有模块。**没有动态注册、插件、运行时图或 `dyn`**（复核者 grep 全文件零命中，唯一「图」是判定成环用的局部 Kahn 草稿，不参与排序）。
- 表（一 phase 一 pass，按 `IrPhase` 升序，执行顺序即表顺序）：`raw_facts`（产 `Instructions`/`ExceptionTable`，不计费——解码是 reader 的工作，字节已按 `ClassBytes`/`AttributeBytes`/`CodeBytes` 收过费）、`raw_cfg`（产 `RawCfg`/`ThrowSites`/`Effects`，计 `[Blocks, Steps]`）、`legacy_normalization`（产 `CallContexts`，计 `[Steps]`）、`canonical_cfg`（产 `CanonicalCfg`，失效 `Effects`/`Frames`/`Ssa`，计 `[Clones]`）、`frame`、`ssa`（重算 `Effects`）。
- 校验语义：phase 降序 → `ir_pass_order_invalid`（同 phase 多 pass 合法，为 3.3–4.x 拆 phase 留门）；整表的 producer→consumer 环 → `ir_pass_graph_cycle`；**被调度前缀**的缺前置 → `ir_pass_prerequisite_missing`；使用未重算的失效事实 → `ir_stale_fact`。顺序/成环是整表性质，缺前置只判前缀——复核者用自建非法表逐条最小触发验证，并确认固定表下四个码经 `analyze_method` **均不可达**（合法请求永远走不到）。
- 失败隔离：`apply` 先全量检查 `requires`、再记 `invalidates`、再 `produces`、最后单调推进 `last_completed`（`max`，重入不回退）；被拒时一个事实都不发布、`invalidates` 一项都不落地，ledger 恰等于「该 pass 之前的前缀」（复核者用 replay 对照证明）。
- 反例与证伪：实现者 5 组变异（前置校验跳过、表逆序、`invalidates` 被忽略、映射错位、先发布后检查）；复核者 10 组变异（含 `invalidates` 空实现、同 phase 也算降序、成环检查直接 Ok、前缀取 min、校验器拒绝一切请求）——除两项外全部被捕获；**M8「删掉 `engine.rs` 的校验调用」与 M10「交换两条校验调用顺序」存活**，即该接入在公共路径上行为不可观测（已登记）。
- 复核发现的契约表达力缺口（**3.3 开工前必修，已修**）：`budget` 原为单一类别，无法表达 raw CFG 同时计 `IrItems`+`IrEdges`+`AnalysisSteps`——按字面实现会**静默漏计 `AnalysisSteps`**，违反 1.3 的超限验收。契约改为维度集合（`Blocks = IrItems + IrEdges`、空集合 = 不计费），并新增金标断言 `every_pass_declares_exactly_the_dimensions_it_bills` 与重入单调性断言 `re_entering_an_earlier_phase_never_lowers_the_last_completed_phase`（两组变异各被对应新断言捕获）。
- 证据：单作业下 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`cargo test --workspace --all-targets --all-features --locked` = **540 passed / 0 failed / 1 ignored**（27 个 suite 全 ok；lib 160→162、`p2_passes` 4），`cargo test --test p1_xref_golden --locked` = 5；由主 Agent 独立复跑确认。
- 远端 CI：实现与文档提交 `2bc0ea6`、`8d74ce4` 推送 `main` 后，CI run [`35287796487`](https://github.com/LordCasser/jarde/actions/runs/35287796487) 四个 job 全部 success。
- 独立复核结论：**Approve**（无必修项；D1 契约缺口已在 3.3 前修正）。登记债务：**`engine.rs` 的接入不可观测**（删掉调用或换序都无测试变红，且前缀规则在 `ir::scheduled_stages` 与 `passes::validate_schedule` 各有一份实现——5.1 必须以校验器返回的表前缀作为唯一执行/阶段来源，并把「报告 `stages` == 校验器前缀」写成断言）；`Effects` 目前无消费者（其失效在运行时不被强制，故契约已写明「事实的消费者必须写进 `requires`」）；`progress()` 的 `no phase completed yet` 分支与空集合分支仓内无覆盖（探针证明可达且正确）；本片的计费语句只有声明，真实计费点从 3.3 起。

### 3.3 raw CFG、throw sites 与 effect facts

- 交付：新增 crate-private `src/cfg.rs`（`RawCfg{blocks, edges, throw_sites, handlers, unreachable, completeness, unresolved_returns}`、`EdgeKind{Normal, Exception{ordinal}, SubroutineReturn{call_site}}`、`EffectFacts`）；`Engine::analyze_method` 首次真跑方法分析（`passes::implemented` 决定哪些阶段有实现，每个 pass 完成后才 `FactLedger::apply`）；`providers::read_definition_content`、`classfile::method_code_coverage`（与 `inspect_method_bytecode` 共享一份规则）与 `MethodCodeFacts::exception_handler_count`。**petgraph 首次成为真实消费者**（`DiGraph` 承载多重图 + 邻接工作列表；不调用 `algo::`，四条准入约束自然成立）。
- 语义：指令级 throw site（逐条 throwing 指令、handler 列表按异常表声明顺序、空列表也记录）；**边的身份写死**（普通转移按 (块, 不同目标) 一条、异常边按 (块, 记录) 一条且共享入口的平行边保留、subroutine 返回边按 call site 一条）——写死是为了让「一次转移记成两条边」可被判为缺陷；`jsr` 在原始图不展开（`ret` 无后继、call site 进 `unresolved_returns`、可达性含 jsr 续点故 `unreachable` 是**欠报**而非谎称死块）；截断体分两种（目标不可校验 ⇒ `Partial` + `ir_raw_cfg_incomplete_body`；前缀自洽 ⇒ `Partial` + reader 的 stop）；catch 类型匹配留给 resolver。
- 计费：`raw_cfg` 的 `budget` 声明 `[Blocks, Steps]` 且与**实际计费维度逐项相等**（`IrItems` 块/handler/throw site/指令 effect、`IrEdges` 每条边、`AnalysisSteps` 每条入块指令与每次可达性迭代）；块上限 16 384 是 crate-private 常量（请求级控制是 `ir_items`，超限以 `BudgetExceeded{IrItems}` 停止并保留前缀）；`method_bodies` 只计目标方法一次；A17 守卫扩到 10 个 module token（含 `crate::cfg`/`crate::passes`）。
- 反例与证伪：实现者 6 组变异（throw site 只取块尾、handler 顺序反转、忽略 `stopped_at`、去掉显式边排序、A17 注入、`may_throw` 恒真）；复核者 10 组变异 + 14 条自建 fixture（跨进程 SHA-256 确定性、块中段 throw site、switch 去重、混合体、16 385 块的护栏、截断两分支）。首轮结论 **Reject**：发现条件分支目标等于自身 fall-through 时**发出两条相同普通边并计两次费**（合法字节码触发），以及异常边 (块, 记录) 去重**零用例**、A17 token 未覆盖 `crate::cfg`/`crate::passes`（`use crate::cfg::raw_cfg;` 注入守卫仍绿）。四项修正后各自变异被对应新用例捕获。
- 证据：单作业下 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净；`cargo test --workspace --all-targets --all-features --locked` = **565 passed / 0 failed / 1 ignored**（`p2_cfg` 9、`p2_contracts` 29、`p2_passes` 4、`p1_xref_golden` 5）；示例 exit 0 并输出 `stages=[RawFacts Completed, RawCfg Completed, LegacyNormalization Failed{ir_pass_not_implemented}, …]`、`usage method_bodies=1 ir_items=15 analysis_steps=12`；由主 Agent 独立复跑确认。
- 远端 CI：实现与文档提交 `10e5c0c`、`66ee2d8` 推送 `main` 后，CI run [`35291411285`](https://github.com/LordCasser/jarde/actions/runs/35291411285) 四个 job 全部 success。
- 独立复核结论：**Reject → 修正 → 待连续性确认**（首轮问题全部修正并有变异证据）。登记债务：D25（driver 读取按物理身份，5.1 决策）、D26（内部上限与请求上限只能靠消息文本区分）、D27（`wide` 包裹 opcode 属 1.2 边界）、D28（catch 类型不过滤，已入契约）。

## 债务登记（滚动，归档前逐条处置）

各片复核登记的边角与已知边界集中在此，避免归档时丢失。**每条都必须有一条处置**：已修、转为显式契约边界、指派到具体后续任务、或明确接受并写进文档。

| # | 债务 | 来源 | 处置 |
| --- | --- | --- | --- |
| D01 | `artifact_tree` 的 fuzz 峰值 RSS 470–500 MB 对 512 MB 限额（余量 2–8%） | P1 验证 | 保持观察：CI 未 OOM；**禁止用缩短运行掩盖**，一旦 OOM 先留证据（`/usr/bin/time -l` / CI 日志）再按 design 的处置顺序决策；5.3/5.4 需带证据复核 |
| D02 | `control_flow_targets` 在截断方法体上 sound-but-incomplete | 1.2 复核 | 已由 3.3 的 completeness/可靠前缀与 p2_cfg 回归处理；本轮该测试目标 9 passed，0.2 修改分块后须重跑 |
| D03 | `newarray` atype、`multianewarray` dimensions、`invokeinterface` count 未保留 | 1.2 补充 | 撤回“P2 不需要”：4.x 消费前必需；转 0.2 共享 reader 补齐与真实字节对照 |
| D04 | 操作数事实 ≈80 B/指令，只按 `CodeBytes` 计费 | 1.2 | 接受：1.3 的 churn 表已记录；如需更细粒度须改 1.3 口径 |
| D05 | `Location::Entry.span` 坐标与其它来源不一致；record component 的 descriptor 类别不一致 | P1 | 接受为既有边界；5.4 文档同步时在支持矩阵的已知边界里点名 |
| D06 | `has_more = true` 且 `cursor = null` 的组合 | P1 | 已在支持矩阵写明（停止且未发布新项时调用方需重试同一请求）；5.1/5.3 的 golden 覆盖该形态 |
| D07 | 重复物化 / 方法体解两遍（P1 `Type`-only 请求、2.x 的多次枚举） | P1/2.2 | 归 P5（索引）；2.2/2.5 已登记为「2.5 全 scope 枚举前需索引」 |
| D08 | `output_bytes` 在 standalone root（`root_bytes`）与 ZIP/tree entry 之间口径不对称 | 2.1 复核 | 接受并已写入支持矩阵；5.3 按维度断言前须引用该口径 |
| D09 | `skipped` 是整请求求和的**保守上界**（高报未决、绝不低报） | 2.2/2.5 复核 | 已写进 2.5 契约；**5.3** 要么收紧为逐查找集合，要么在 golden 里固定该上界语义 |
| D10 | 闭包键 `(loader, name)` 的 loader 分量在公开路径不可证伪 | 2.2 复核 | 撤回不可证伪边界：R1 已用公开成员查询证明错误，转 0.1；闭包后继必须由 defining loader 发起 |
| D11 | `providers` 的 `remembered`/`record_read`/`visited` 线性扫描 | 2.2 复核 | 归 P5（索引）；规模受 `class_headers`/`analysis_steps`/`elapsed_millis` 约束 |
| D12 | `method_bodies` 在 3.x 接通计费前没有计费点 | 2.2/2.3 复核 | 已由 3.3 接通；本轮 p2_cfg 回归通过。旧切片的零计费证据仍按当时 code_bytes 对照解释 |
| D13 | `subtype_of` 遇到调用方层级成环时静默跳过 → `Inaccessible` 且无环诊断 | 2.3 复核 | 接受为不对称边界；若 5.x 需要诊断须先给调用方层级加环码 |
| D14 | signature-polymorphic 警告在调用点描述符恰好等于声明描述符时文案仍称「两者按规则不同」 | 2.3 复核 | 低优先：改文案或在 5.4 文档里说明 |
| D15 | `read_definition` 的记录挂在请求声明的 `caller.loader` 上（`PhysicalDefinitionId` 不含 loader，API 内不可校验） | 2.3 复核 | 转 0.1：显式校验 caller/driver 的 loader 与物理定义绑定；不同 snapshot 可以是合法 provider，不能只校验 snapshot 相等 |
| D16 | `DeclarationRefQuery.consumers.version` 不校验（`Engine::query` 会拒绝非 1） | 2.4 复核 | 接受；若要统一，属小改动，5.4 前决定 |
| D17 | 「解析到别的声明」无独立报告桶（只能由 `reads`/usage 观察） | 2.4 复核 | 已写进契约（报告不设第三个桶）；5.3 的 golden 覆盖该形态 |
| D18 | `ResolutionReport.resolved`/`candidates` 不单独计费 | 2.5 复核 | 已写进契约（判定证据字段不重复收费）；由绝对账单边界守护 |
| D19 | petgraph A17 守卫缺口：受守卫文件里 `use petgraph::…` 不被捕获 | 3.1 复核 | 已由 3.3 增加 petgraph 三类 token 与注入自检；工作区另覆盖 call_context，5.2 仍需实际构造计数 |
| D20 | 第二个 petgraph 版本只剩 `cargo-deny` 的 warn 可见；升级门槛（重推 feature 名清单）依赖人工 | 3.1 复核 | 记入升级清单；升级时必须重跑 3.1 的行为证据 |
| D21 | `engine.rs` 的 pass 校验接入在公共路径不可观测；前缀规则在 `ir::scheduled_stages` 与 `passes::validate_schedule` 各有一份 | 3.2 复核 | 3.3 已使 Engine 消费 validate_requested_stages 返回的 pass 前缀；5.1 保留 stages 与执行前缀的一致性验收 |
| D22 | `Effects` 目前无消费者（其失效在运行时不被强制） | 3.2 复核 | 当前 3.4 已读取 raw.effects 但漏 Effects requires；转 0.3，加 stale/未产出反例，见本轮 R6 |
| D23 | `progress()` 的「尚无 phase 完成」分支与空集合分支仓内无覆盖 | 3.2 复核 | 探针证明可达且正确；5.1 装配真实 `stages` 时会走到 |
| D24 | `analysis-contracts` 的 Purpose 仍是 P1 口径；`query-api` 仍称 P2 会处理 `references_definition`；`jvm-ir` spec 写「budget class」单数而契约为维度集合 | P1/P2 记录 | jvm-ir budget 集合已在本轮 delta 修订；5.4 仍须直接修正主规格 Purpose 和 query-api 的过时阶段承诺，不接线 P1 query |
| D25 | `analyze_method` 的 driver 读取按**物理身份**，不受环境 domain/root 约束（构造「环境指向快照 B、请求 owner 在快照 A」可读到 A 的定义并把 loader 记成 app） | 3.3 复核 | 转 0.1，与 D15 一起固定 driver 的真实 loader/definition 绑定；不能给 content 中任意物理定义贴 app 身份 |
| D26 | 内部块上限（16 384）与请求级 `ir_items` 在报告层只能靠诊断文案里的 `limit=16384` 区分（`Error::BudgetExceeded` 的 limit/consumed/requested 在 `ir::terminal` 被丢弃） | 3.3 复核 | 接受为现状；5.1 若要发布计数需先决定是否给独立 code |
| D27 | `wide` 包裹的 opcode 在 1.2 未保留（`wide iload/istore/ret` 既不分类局部读写也不结束块；`wide iinc` 经 increment 仍分类） | 3.3 实现 | 不再接受为可推迟边界：转 0.2，需回归 1.2/3.3 与 3.4 的现代 wide 方法、51+ wide ret 分支 |
| D28 | catch 类型匹配不在 `cfg` 层做（throw site 的 handler 列表只按保护区间与声明顺序，不过滤类型） | 3.3 契约 | 已写进契约：类型层次属 resolver，`cfg` 不得依赖 |
| D29 | BCI→块有两种查法：`cfg::block_position` 用 `binary_search_by_key`（要求恰为块起始），`call_context::block_of` 用 `partition_point`（最后一个起始 ≤ bci）；`jsr_continuations` 用前者，故 `jsr` 非块首时静默丢掉续块关系 | 3.4 复核（D-1） | **转 0.4**：统一为「包含该 BCI 的块」并各补一条非块首 `jsr` 回归；修正后 ECJ 45–48 的 `unreachable` 应为 `[11,15]` |
| D30 | `Established` 的载荷可以含 `targets: []` 的 `SubroutineReturn`（死代码里的 `ret`）；契约未规定 3.5 如何消费「已发布但无归属」的返回点 | 3.4 复核 | 已写进 3.4 的载荷不变量：`targets` 为空时 3.5 MUST NOT 据其建边；3.5 验收须含该形态 |
| D31 | 截断体但上下文恰好完整时是 `Established` + stage `Partial`；原「截断体 ⇒ 触发①」的叙述只覆盖失败的那一半 | 3.4 复核 | 契约已改为逐触发声明作用域；3.4 验收须同时断言这一组合 |
| D32 | `ir_call_context_inconsistent` 不是「公共路径不可达」，而是「raw 图与 reader facts 自洽时不可达」；它经 `ir::terminal` 映射为 `Failed{Error}` + Error 诊断写进公共 diagnostics | 3.4 复核 | 措辞已按此修正；`provenance: None` 与 A09/A13 的 origin 期望差距仍归 5.1 |
| D33 | 装配期（`assemble`/`instruction_ranges`/`successors`/`plans`）没有 poll/charge，是唯一不按 pass 边界检查取消的窗口 | 3.4 复核 | **转 0.3**：建表与最终装配均需 poll，派生存储在增长前计 `IrItems` |
| D34 | `ir_pass_not_implemented` 的产物面组合：3.4 正常完成后 `stages` 为 C,C,C + 后续 `Failed{ir_pass_not_implemented}`，此时 `quality = Fallback` 是 `analysis_report` 的字面量（`ir.rs` 无分支），不构成「走了 fallback」的分类证据 | 3.4 复核 | 3.5 前不得用 `quality` 作断言依据（示例与文档已注明）；5.1 给出真实分类后补断言 |
## P2 验收映射现状（滚动更新）

按 `openspec/acceptance.md` 与 tasks 的对应关系逐条对照，避免"局部通过"被当成"整体正确"。状态只在有验证记录时前进。

| 验收 | 承担任务 | 现状 | 还缺什么（退出 P2 前必须补） |
| --- | --- | --- | --- |
| A11 Base.foo / Sub CP owner | 0.1、2.3、2.4、2.5 | **单 loader 基线已覆盖，跨 loader 修正未完成**：2.3 成员解析（46 条用例 + 探针，含 JVMS 5.4.3 三条搜索路径、访问与调用种类规则、default conflict）；2.4 声明引用查询（`Base.foo` 在 `Sub` 调用时 `mentions_symbol(Base.foo)`=0 而声明查询返回该 use-site、`resolved` 指向 `Base`；未使用 CP 不算引用；未决候选保留 use-site 不当作已排除）；2.5 已知范围 dispatch（候选 + open-world 证据，单一候选不声称唯一运行目标） | 0.1 关闭 R1，重跑声明查询/dispatch 的定义身份对照，再执行 5.4 总门禁 |
| A14 全范围中断/缺失依赖 | 1.3、2.1、2.2、2.3、2.4、2.5、5.1 | **部分**：18 项预算维度与两个高水位就位；2.1–2.5 的停止语义（`Partial`/`Cancelled`/`BudgetExceeded` + 前缀）各有实证，2.5 补上 scope 枚举预算与 `DependencyDepth`→listing 截断的停止路径 | 5.1 的库/CLI 一致性与终止语义逐字段一致；4.x 阶段的停止（Frame/SSA 预算） |
| A16 单方法按需边界 | 2.2、2.3、2.4、2.5、5.2 | **部分**：`reads` 记录 (definition, loader) 与理由（含 `DispatchScope`）；成员搜索与 dispatch 都不读 Body（`code_bytes == 0` 有真实对照，2.4 另有"与同 consumers 的 P1 扫描计费相等"口径） | 5.2 的实际入口读取/构造计数（不加载无关 Body、不建全局 XRef） |
| A17 X1 零 CFG/SSA/AST | 1.1、3.3、5.2 | **部分**：petgraph、cfg、passes 的 token 守卫已由 3.3 补齐；工作区另有 call_context token | 5.2 的实际构造计数；新增私有模块的守卫覆盖仍须审计，源码 token 不是行为证明 |
| A09 历史 jsr/finally | 0.2、0.3、3.3–3.5 | **部分**：raw CFG 已交付，3.4 候选被 R2/R3/R4 阻塞 | 修正值流/异常/预算、真实历史 finally 与 3.5 有界规范化 |
| A10 缺失 StackMap/debug | 4.1–4.3 | **未开始** | Frame 推导、版本合法性诊断、`NotPerformed` 语义 |
| A13 成员级失败 | 5.1 | **未开始** | 同类正常与失败方法并存、五平面分开报告 |
| A18 输入变化 | P0/P1 已覆盖 | **保持** | 每个缓存/并行阶段引入时回归（P5） |

当前结论：2.x/3.3 的已有证据保留；本轮跨 loader 反例使 A11 重新需要修正，3.4 尚未通过。A14/A16/A17 仍缺各自后续入口/资源证据，不能因某片测试全绿就宣布 P2 完成。

## 第一片（1.1–1.3）状态与闸口

- 1.1、1.2、1.3 均已完成、独立复核 **Approve** 并有各自 CI 记录；第一片的退出条件（reader 类型化操作数、预算维度、结果/请求契约可用）已满足。第一片整体以提交 `0cba0d6`（实现）+ `6344508`（文档）推送，CI run [`35253446169`](https://github.com/LordCasser/jarde/actions/runs/35253446169) 四个 job 全部 success（`stable` 含 ignored JDK 25 oracle、`MSRV 1.88.0`、双 workspace `supply chain`、`fuzz smoke`）。
- 2.5 已完成实现并经两轮独立只读复核（首轮有条件 Approve，三项必须改已关闭；复审要求补三条用例，收口记录见 2.5 节）。3.1–3.3 已有交付，3.4 为未提交候选且本轮 review 未通过；当前执行顺序改为 tasks 的 0.x 后再继续 3.4/3.5。
- 债务按上表区分已关闭、必须前置与继续延期；D03/D10/D15/D22/D25/D27 已提升为 0.x 的进入门槛。P5 物化/索引、P1 坐标口径和文案维护继续拆分，不混入这些正确性修正。

## 2026-09-18 当前工作区复核

### 基线与结论

**结论：需要修正后继续；3.4 不接受交接到 3.5。** 本轮不改实现、不归档、不勾选任务，保留其他 agent 已写的代码。原 11/20 项历史勾选保留，新增 0.1–0.3 后为 11/23；0.x 是明确未完成的修正义务。审查重点是 2.x 定义身份、3.3/3.4 边界及后续 Frame/SSA 设计，不是对全部 P2 代码的无缺陷证明。

- HEAD 与 origin/main 同为 `4beb6b9ce5322a94ff0a0c571ae532d687096523`。3.4 工作区包含 `src/call_context.rs`、`tests/p2_return_address.rs` 及 engine/cfg/passes/ir/API、示例和文档修改；均未提交。
- `src/call_context.rs` SHA-256 为 `dfcbe0ed9b81dd5acef3507cfb5b06c69bb5d26bb00825e4eb55d54988817d30`，`src/cfg.rs` 为 `bf7395c26fc520643c4b77ea6880ad23ad2a7765df898a8d7046770573ac6f00`，`src/engine.rs` 为 `ff37579dbbd87aa79407654aaf8c9d2c62cebaae32e4799873a71fba040e2242`，`src/passes.rs` 为 `dff9fa158d8fd8d5ece16ab5c0ce29938fd36b615394a3a62581c68990b0302b`。本地验证起止 hash 相同。
- 3.3 后续文档提交 `66ee2d87d8db3abeb63bf6718004e1ff308383d3` 的 [CI 35291411285](https://github.com/LordCasser/jarde/actions/runs/35291411285) 与 HEAD 的 [CI 35293844696](https://github.com/LordCasser/jarde/actions/runs/35293844696) 均四 job success。后者不含未提交 3.4，不能给该候选背书。
- 3.3 原复核写“待连续性确认”；本轮确认其后续提交/CI 与本地 CFG 回归，但未取得原复核者新增的 Approve，因此不改写原结论。0.2 改到 reader/分块后须重新确认受影响部分。

### 本轮门禁与反例

默认 `/opt/homebrew/bin/cargo` 为 1.98.1。本轮独立只读核验：fmt check、workspace/all-targets/all-features clippy `-D warnings` 均 exit 0；`cargo test --locked --test p2_return_address --test p2_cfg --test p2_passes --test p2_contracts` 分别 7、9、4、29 passed，合计 **49 passed / 0 failed**。修改规划前 OpenSpec strict **10/10**。未重跑完整测试、MSRV、oracle 或完整 fuzz，本轮不冒认这些为新增本地证据。

四个反例均在复制 tracked/untracked 源码的隔离副本运行，新增探针没有写回工作区：

| 编号 / 优先级 | 输入与实际结果 | 原因与影响 | 修正归属 |
| --- | --- | --- | --- |
| R1 / P1 | ChildFirst 的 child 中有 Base；parent 中有 Owner extends Base 和另一个 Base；两 Base 都声明 public f:I。公共 `resolve_symbol(Owner.f)` 实际返回 **child Base** 的物理定义/loader，并报 Resolved、Complete、无诊断；期望 parent Base | `HeaderClosure::demand` 总从 runtime loader 开始，HierarchyWalk 待展开项只有 name，没有定义 loader。错误声明会传入声明查询、dispatch 和后续类型分析 | 0.1，重新验证 2.x |
| R2 / P1 | v49：`0:jsr 4; 3:return; 4:astore_0; 5:ret 1`。实际 Established，返回目标 BCI 3；local 1 从未保存返回地址 | `Walk::visit` 只按 ret 所在 active context 加入 targets，不消费 ret local/token。后续克隆会凭空建立控制流；NotPerformed 不能授权伪造已证明的返回点 | 3.4 |
| R3 / P1 | v49：jsr 4；子程序 astore_0 后在 BCI 7 idiv，handler 写 local 1/2 再 goto ret 0。实际 Established 的 affected_locals 为 `[0]`，应包含 `[0,1,2]` | `Walk::step` 丢弃全部 Exception 边，只记录覆盖 ordinal；handler 在同一上下文继续执行时，其写入和 token 变化全部漏掉。原 design 的“handler locals 不进入遍历”也错误，已撤回 | 3.4 |
| R4 / P2 | 先以充足预算建立 raw CFG，再对合法 jsr/astore_0/ret 0 调 `call_contexts`，新 budget `ir_items=0`、steps 充足。实际成功创建非空 contexts/returns | plans、visited、每上下文集合和装配只消耗 Steps，越过独立 IR 存储限额；私有载荷同样可能按 context×block/handler 膨胀 | 0.3 |

探针命令：`cargo test --locked --lib call_context::tests::review_ -- --nocapture`（新增三项都应在旧实现失败）；`cargo test --locked --test p2_members review_parent_defined_owner -- --nocapture`（新增一项在旧实现失败）。首次用较宽 `--lib review_` 过滤时另外命中一项既有 classfile 测试且通过，不能计成新增探针通过。

最小复现资料（复用相应测试文件已有 helper，修复轮需转成永久回归）：

```text
R2 / R4：call_context.rs tests::body
instructions = [jsr(0,4), plain(3,0xb1), store(4,0x4b,0), ret(5,N)]
handlers = []; code_length = 7; major = 49
R2: N=1; assert Unresolved（或等价的明确失败，无 CallContexts）
R4: N=0; graph 先建立；call_contexts 的 Budget 仅 ir_items=0，其他相关维度充足；assert Err(IrItems)

R3：同一 body helper；code_length=17; major=49
0 jsr +4; 3 return; 4 astore_0; 5 iconst_1; 6 iconst_0;
7 idiv; 8 pop; 9 ret 0; 11 astore_1; 12 iconst_0; 13 istore_2;
14 goto -5（到9）
exception table: ordinal 0, start=5, end=8, handler=11, catch_all
assert contexts[0].affected_locals == [0,1,2]

R1：p2_members.rs 的 Class/open/zip_of/domain/environment/World helpers
child snapshot: p/Base { public int f; }
parent snapshot: p/Owner extends p/Base; p/Base { public int f; }; java/lang/Object
child domain: ChildFirst, parent_loader=parent, roots=[child snapshot]
parent domain: ParentFirst, roots=[parent snapshot]
content=[child,parent]; runtime domain=child; caller=abstract_caller(child)
resolve(field(p/Owner,f,I), FieldRead)
assert state=Resolved && resolved.loader=parent
并断言 resolved.definition 对应 parent snapshot 的 Base，而非仅 owner 名字相等
```

以上前三类语义反例依据 JVMS 的 defining loader、ret local 与子程序数据流规则，不要求本引擎实现完整 verifier。参考 [JVMS 5.3/5.4.3.1](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-5.html#jvms-5.4.3.1)、[ret 指令](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html#jvms-6.5.ret)；R4 来自本项目既有的分配前预算契约。

### 静态发现及规划修正

- **R5（前置 facts）**：wide effective opcode 缺失会使合法现代 wide 方法 unresolved，且 wide ret 无法进入 51+ 违规判定；atype/dimensions/count 也不能继续登记成“P2 不需要”。0.2 补齐共享 reader，并回归 raw CFG/effects。
- **R6（Pass 契约）**：当前 call_context 读取 `raw.effects`，`legacy_normalization.requires` 未声明 Effects。0.3 加入依赖与 stale 反例；后续阶段预算集合同时补齐，并保留真正可消费的 payload。
- **R7（未实施的 Frame/SSA 设计）**：locals 合流缺 Top、初始化只转换单槽、把 `<init>` 的返回值当转换来源、phi 只数聚合 raw 边，以及允许 fixture 证据升级 verification 的描述均已修订。参考 [JVMS 4.10.2](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.10.2)；该项是防止后续实现照错误设计推进，不宣称发现已有 Frame/SSA 实现错误。

不追加依赖；继续复用已准入 noak/petgraph。P5 索引/重复物化与 P1 坐标/类别等债务保持拆分。依次执行 0.1 → 0.2 → 0.3 → 3.4，通过定向反例与独立复核后再进入 3.5/4.x。本轮文档修订不等于这些修正任务完成。

### 文档修订后的核验

独立只读核验：`openspec validate --all --strict --no-interactive` 为 **10 passed / 0 failed**，`git diff --check` exit 0；tasks 为 **11 checked / 12 unchecked（11/23）**，0.1–0.3 均未勾选。12 个变更 Markdown 文件的 41 个本地文件链接均存在，当前状态文字与历史记录已区分。对照本轮开始时的 10 个相关源码/测试 hash，无一变化；另对隔离快照比对其余源码、manifest/lock 和 CI 文件，未发现实现变更。本轮新增内容仅为文档，四个探针及其运行产物留在隔离副本。
