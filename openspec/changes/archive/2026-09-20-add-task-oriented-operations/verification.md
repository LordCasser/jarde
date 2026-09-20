# 验证记录

实现提交见仓库历史（紧邻本文件归档提交之前）。本机门禁如下，CI 结果随后回填。

## 契约实现

全部落在 `src/facade.rs`（随既有 `pub use facade::*` 导出，`src/lib.rs` 无需改动）：

| 入口 | 不证明什么 |
| --- | --- |
| `ClassRef::{Name, Definition}`、`MethodRef::{Name, Method}`、`BodyRef::{Name, Method}` | 绑定的定义/成员合法、可加载或可解析；`BodyRef::Name` 不证明 Body 存在（那由 `NotDeclared` 说） |
| `TargetCandidates`、`OperationOutcome<T>{Performed, Ambiguous}` | `Ambiguous` 证明什么都没跑；`Performed` 只证明规则绑定到的那个物理身份 |
| `OVERRIDABLE_BUDGET_DIMENSIONS`（5 项）、`BudgetOverride`、`task_limits`/`task_budget` | 运行是否落在配置内（那是 `report.usage`）；elapsed 是协作式的，不是硬期限 |
| `EnvironmentPolicy::{SingleClass, PlainJar, ExplicitClasspath, Layout}`、`EnvironmentRequest::build` | roots 可读或任何东西已解析——那由既有 validator 与搜索状态说 |
| `ClassViewRequest`/`ClassViewReport`/`ClassViewBody::{Read, NotDeclared, Refused}`、`BodyStageResult` | Body 是合法 Java、可重编译或等价；body 阶段是 reader 的两个解码阶段，**不是** IR stage |
| `MethodOperation::{Analysis, Recovery}`、`MethodOperationRequest/Report`、`MethodRecoveryReport` | verification（恒为 `NotPerformed`）与恢复完整性 |
| `ReferenceFinding::{ConstantPoolCandidate, StructuralReference, ResolvedDeclaration}`、`ReferenceGrouping` | 结构引用解析到了声明；它绝不补全未决候选 |
| `RecoveryPresentation`/`RecoveryPresentationPart` | 完整、可编译或等价——它只给报告既有字段排序 |
| `Engine::class_view` / `analyze_target` / `recover_target` | — |

`find_targets` 重构到共享的 `search_named_classes` 上，使导航入口与类视图不可能对「哪个定义」给出不同答案。

## 关键证据（`tests/task_operations.rs`，29 项）

- **1.1 目标选择**：两种写法与直接身份绑定同一 `PhysicalDefinitionId`；两个 origin 上的 `p/S`（上层 `WEB-INF/classes/p/S.class` 声明 `upper`、嵌套 `p/S.class` 声明 `nested`）返回 `Ambiguous{2 candidates}`，且 `class_headers` 仍为 1、`method_bodies` 与 IR 维度全 0（**歧义不执行分析**）；把嵌套候选的身份回传只渲染 `nested`；重载 `run()V`/`run(I)V` 返回两个方法候选且不运行；外来身份 → `operation_target_snapshot_mismatch`；无匹配 → `operation_target_not_found`；篡改 digest → reader 的 `definition_class_bytes_mismatch`。
- **1.2 stage 自选与发布**：`recover_target` 发布 `stages == AnalysisStage::ALL`（6）与 `limits == task_limits(&[])`；同列表的显式 `MethodAnalysisRequest` 得到相同 `stages`/`requested_stages`；显式 `[RawFacts]` 恰好调度 1 个阶段，`[Ssa]`/`[]` 保持 `analysis_no_stages`。
- **1.3 预算**：默认在每个计费维度有界、`elapsed_millis = 30_000`；一次覆盖只替换一个维度并发布完整生效 `Limits`；`BudgetOverride::new("input_bytes",10)`/`("nonsense",10)` → `budget_override_dimension_unknown`，`("output_bytes",0)` → `budget_override_invalid`（**不静默取默认**）；紧 `class_headers: 1` 对两候选 → `Partial{BudgetExceeded{ClassHeaders}}` + `budget_exceeded_class_headers` 且保留已确认前缀；`method_bodies: 1` 对两个 body → 1 个 `Read` + `Partial`；`output_bytes: 8` → `Partial{BudgetExceeded{OutputBytes}}` 且 `body = NotInspected`。
- **2.1 环境策略**：`SingleClass` 与手写环境 `==`、无 problem，且拒绝 ZIP（`environment_policy_snapshot_kind_mismatch`）；`PlainJar` 只声明一个 root（snapshot 的 root container、空前缀、`parent_first`/`class_path`、无 provider），**不激活**扫描到的嵌套库（`p/Nested` → `Missing`，`reads` 为空）；显式 classpath `[A,B]` 解析到 A、`[B,A]` 解析到 B，从不 `Ambiguous`。
- **2.2 不推断边界**：一个含 `Class-Path: lib/one.jar`、`lib/one.jar`、`WEB-INF/classes/p/W.class`、`BOOT-INF/classes/p/B.class` 的归档证明：`PlainJar` 只声明一个 root、树遍历显示这些条目确实存在、而 `p/W`/`p/B`/`p/Nested`/`p/Main` 在该策略下**全部 `Missing`**；只有显式声明带 `WEB-INF/classes/` 前缀的 root 才能解析 `p/W`；`Layout{War|SpringBoot|Generic}` → `Unsupported` 的 `environment_policy_layout_not_provided`。被 validator 拒绝的策略保持引擎的不可用状态且 `class_headers = method_bodies = 0`。
- **3.1 类视图**：见下读取计数；`abstract`/`native` → `NotDeclared{Abstract|Native}` 且不尝试 Body；成员记录损坏时在该记录处停止（类与停止前的成员仍发布、`member_table` 置位、execution `Partial`），停止之后的 body → `Refused{reference, method: None}`（**不为遍历未到达的成员编造身份**）。
- **3.2 轻量与一致**：打开 ZIP 只计 `input_bytes`（`class_headers`/`method_bodies`/`code_bytes`/`output_bytes`/`archive_entries` 全 0），四个构造维度为 0，同字节重开得到同一 `SnapshotId`；`class_view` 与 `recover_target` 在同一 snapshot 上两次运行在剔除 `elapsed_millis` 后逐字节相同，挂上 `FactsCache` 后同样成立（`consultations > 0`）。
- **3.3 引用组织**：一次 `MentionsSymbol` 报告把 `new p/T` 归到 `p/Caller.run` 的 BCI 0（`Type`/`New`），类级 `SuperClass` 命中留在 `class_level`、Manifest 命中留在 `resources`；未使用的 `Methodref` 是 `class_level` 里的单个 `ConstantPoolCandidate`（无消费者、无 BCI、无方法分组）；`p/Sub.foo()V` 是 `StructuralReference`（`resolution = NotRequested`，BCI 5），在显式环境下同一 use site 变为 `ResolvedDeclaration`（`state = Resolved`，owner `p/Base`）；缺少 `p/Base` 时 `items = 0`、`unresolved_candidates = 1`，分组如实带计数与诊断且零分组。
- **3.4 呈现顺序**：`nestedPlain` → `parts = [Content(ContainsStatements), Quality(Structured)]` 且 `stop = None`；`fieldCast` → `ExplanationOnly` 且交付文本非空；紧 `output_bytes` → `[Content, Quality, Stop]` 且 `NotProduced`、空文本/段表、非 Complete。**文本对照**：把真实报告里的 `the instruction at BCI 3` 改成 `the return instruction at BCI 3` 后 `content`/`quality`/`stop`/`parts()` 保持不变（若实现靠扫文本就会翻转成 `ContainsStatements`）。

## 读取计数（3.1/3.2，逐字）

`p/Bodies`（1 字段 + 2 具体 + 1 abstract + 1 native）在 2 类 JAR 中、按名字路径、请求一个具体 body：

```text
WITH BODY:    archive_entries: 3, entry_bytes: 219, read_bytes: 219, class_bytes: 219, attribute_bytes: 57, code_bytes: 1, result_items: 19, class_headers: 1, method_bodies: 1, ir_*: 0
WITHOUT BODY: archive_entries: 3, entry_bytes: 219, read_bytes: 219, class_bytes: 219, attribute_bytes: 38, code_bytes: 0, result_items: 16, class_headers: 1, method_bodies: 0, ir_*: 0
```

请求一个 body 只增加 `method_bodies +1`、`attribute_bytes +19`、`code_bytes +1`、`result_items +3`；`class_headers`/`class_bytes`/`read_bytes`/`entry_bytes` **完全相同**。

任务点名的两个变异（施加、运行、还原，失败输出逐字）：逐方法重读 Header → `class_headers` 断言 `left: 4 right: 1`；逐方法重复列举 → `class_bytes` 断言 `left: 876 right: 219`。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --test task_operations --locked` | 29 passed / 0 failed |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1240 passed / 0 failed / 6 ignored**（基线 1211，+29）；coder 另以两个固定 seed（`…569`/`…570`）各跑一次，均 1240/0 |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（18.2 s） |
| `cargo run --example resolve_and_analyze --locked -- …/HistoricalControlFlow.class` | exit 0 |
| `openspec validate --all --strict --no-interactive` | 21 passed / 0 failed（归档前） |

未新增 fixture 文件：新归档全部由测试内 `rawzip` 构造，复用的真实样本是既有的 `p3-nested-eval`、`p3-refused-cast` 与 ECJ v52，均已登记在 `tests/fixtures/README.md`，因此 corpus fingerprint 未改动。

## 设计里由实现者决定的取舍（已披露）

- 固定默认值（64 MiB input/class、30 s elapsed、4096 headers/bodies 等）与 5 个可覆盖维度名；覆盖值 `0` 视为输入错误；歧义装在 `OperationOutcome` 里并携带生效 `limits`。
- **类视图的 body 走 reader 解码**（`class_member_facts` + `method_code_facts`，复用同一份已物化字节）并自带两阶段 `BodyStageResult` 词汇，而不是逐 body 跑 P2 pipeline——后者会为每个方法各计一次 class header，使「类视图只读一次 Header」这条要求不可达。
- `Refused` 携带调用方的 `BodyRef` 与 `Option<PhysicalMethodId>`：遍历未到达的成员不会被编造身份。引用分组以 `ReferenceSource` 回显来源扫描，而不是重新推导其平面。
- 跨请求复用只使用 `bound-container-lookup` 与 P5 facts cache 已提供的东西，本 change 未新增缓存层，因此也没有声称任何新的复用路径。
- WAR/Boot **layout 策略**按设计缺省（`Layout` → unsupported）；`PlainJar` 与显式 classpath 是仅有的两个「自动」策略，且都不生成 roots。

## 边界

- 顺带记录、本次未改的既有问题：body 解码（`method_code_facts`）会经 noak 重新解析整个类，因此在成员记录损坏的类里每个请求的 body 都返回 `Refused`（类、停止前的成员与其它逐成员结果仍存活，报告状态为 `Failed`）；`list_members` 发布成员表停止诊断时不计费而 `find_targets` 计费（各自沿用局部约定）；`member_coverage` 用成员索引范围而范围扫描用条目索引范围，类视图在同一 `artifact_structural` 平面下按各自标签合并。
