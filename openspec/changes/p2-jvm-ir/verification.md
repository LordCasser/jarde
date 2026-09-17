# P2 实施验证记录

日期：2026-09-17。规划与契约基线 `35fdf6d`。本轮只实现 **1.1**；1.2/1.3 与 2.x–5.x 均未开始，解析、闭包、CFG、SSA 与预算维度扩展都没有实现。所有命令按单作业执行（`CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1`）。

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

## P2 验收映射现状（滚动更新）

按 `openspec/acceptance.md` 与 tasks 的对应关系逐条对照，避免"局部通过"被当成"整体正确"。状态只在有验证记录时前进。

| 验收 | 承担任务 | 现状 | 还缺什么（退出 P2 前必须补） |
| --- | --- | --- | --- |
| A11 Base.foo / Sub CP owner | 2.3、2.4 | **部分**：2.3 已实现并复核成员解析（含 `Resolved`/`Missing`/`Ambiguous`/`Inaccessible`/ICCE 与"缺失依赖不变否定"，有 46 条用例与探针）；2.4 的声明引用查询实现中 | 2.4 的 `Base.foo` 在 `Sub` 调用的端到端对照（`mentions_symbol` 仍按原始符号）、未使用 CP 不算引用、未决候选不当已排除；2.5 的 dispatch/open-world |
| A14 全范围中断/缺失依赖 | 1.3、2.1、2.2、2.3、2.5、5.1 | **部分**：18 项预算维度与两个高水位就位；2.1/2.2/2.3 的停止语义（`Partial`/`Cancelled`/`BudgetExceeded` + 前缀）有实证 | 2.5 的 scope 枚举预算；5.1 的库/CLI 一致性与终止语义逐字段一致 |
| A16 单方法按需边界 | 2.2、2.3、5.2 | **部分**：`reads` 记录 (definition, loader) 与理由；成员搜索不读 Body（`code_bytes == 0` 有真实对照） | 5.2 的实际入口读取/构造计数（不加载无关 Body、不建全局 XRef） |
| A17 X1 零 CFG/SSA/AST | 1.1、5.2 | **部分**：源码级守卫（`query`/`xref` 不得引用 P2 模块与类型，含推导的类型名单与注入自检） | 5.2 的构造计数（resolver/CFG/SSA/Region/AST 次数为零）；petgraph 引入后守卫需覆盖新依赖位置 |
| A09 历史 jsr/finally | 3.3–3.5 | **未开始** | raw CFG/returnAddress/有界规范化 + 真实历史 finally 语料 |
| A10 缺失 StackMap/debug | 4.1–4.3 | **未开始** | Frame 推导、版本合法性诊断、`NotPerformed` 语义 |
| A13 成员级失败 | 5.1 | **未开始** | 同类正常与失败方法并存、五平面分开报告 |
| A18 输入变化 | P0/P1 已覆盖 | **保持** | 每个缓存/并行阶段引入时回归（P5） |

结论：2.x 完成前不宣称任何 P2 验收通过；上表在每片收口时更新。

## 第一片（1.1–1.3）状态与闸口

- 1.1、1.2、1.3 均已完成、独立复核 **Approve** 并有各自 CI 记录；第一片的退出条件（reader 类型化操作数、预算维度、结果/请求契约可用）已满足。第一片整体以提交 `0cba0d6`（实现）+ `6344508`（文档）推送，CI run [`35253446169`](https://github.com/LordCasser/jarde/actions/runs/35253446169) 四个 job 全部 success（`stable` 含 ignored JDK 25 oracle、`MSRV 1.88.0`、双 workspace `supply chain`、`fuzz smoke`）。
- 2.5 起未开始（2.1–2.4 已完成并复核 Approve）。按 `tasks.md`，2.x 各片逐项实现、验证并只读复核后再交接。
- 债务池（登记，不阻塞）：1.2 的操作数存储放大与未完整解码前缀语义（3.x 消费前收紧）、`fuzz/README.md` 措辞、P2 维度真实膨胀由 3.5/4.3 验收、`query-api` 与 `analysis-contracts` 的 spec delta 在 P2 归档时同步主规格。
