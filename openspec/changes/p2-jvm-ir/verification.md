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

### 远端 CI

本切片以两个提交推送 `main`：`0406178`（实现与测试）与 `6fc1674`（本 change 的契约与验证记录）。CI run [`35247034235`](https://github.com/LordCasser/jarde/actions/runs/35247034235) 在 `6fc1674` 上四个 job 全部 success：`stable / test and specification`（含 ignored JDK 25 指令边界 oracle 与两条公共示例）、`MSRV 1.88.0`、`supply chain`（根与 fuzz 两个依赖图）、`fuzz smoke`。1.2 的提交与 CI 在其小节内记录。

## 尚未关闭

- 1.2（reader 类型化操作数与目标校验）、1.3（预算维度扩展）未开始；2.x–5.x 全部未开始。本 change 的 20 项任务中只有 1.1 具备勾选条件。
