# jarde

jarde 是纯 Rust、library-first 的 JVM artifact 分析引擎。P0/P1 已完成并归档：有界不可变快照、CLASS/JAR/WAR 读取、嵌套物理视图、MR 选择、Header/bytecode inspection 和 X0/X1 查询已交付。

**当前完成判断（2026-09-20，`bafdcec` / 行为 `85828c4`）**：常规门禁通过，但独立 review 复现了名称选择丢失搜索停止、类视图顶层忽略 body 停止两项问题，暂不能确认任务链收尾完成。反例、验证结果与独立修正计划见 [完成复核](openspec/completion-review.md)；性能专项仍为 0/22。

**已关闭的恢复问题（`fd0aae8`）**：P0–P5 与分层均已按各阶段范围归档，`cd6f2f0` 复核出的两条 P1 已由本轮关闭（`close-recovery-correctness-gaps`，4/4）：`(x + 1) + ++x` 不再输出对输入 7 返回 17 的 Java，改为保留被拒读取的可靠降级；被拒 cast 前的字段读取重新出现在产物与 source map 中，含字段链。两条反例、正向对照与执行基线已进入永久语料（`tests/fixtures/p3-nested-eval/`、`tests/fixtures/p3-refused-cast/`）与库/CLI 验收。实施、变异与门禁证据见 [收尾验证](openspec/changes/archive/2026-09-20-close-recovery-correctness-gaps/verification.md)。

**P3 恢复（已归档，12/12，`250fe1f`）**：方法体、结构/模式、声明/作用域、按需 accessor 与物理方法映射已交付；原 P3-R1–R7 的具体反例均已关闭，其中直接 `return x++` 通过可靠降级关闭。受控编译/执行对照已经运行；这不等于所有表达式已证明等价，也不等于任意 JVM 方法都能还原为完整 Java 编译单元——R8/R9 的关闭见上方历史收尾记录。

**P4 现代语义（已归档，10/10，`88416ab`）**：1.1–1.3（release registry，以及 record/sealed/condy/concat 事实与非法 fixture、golden diagnostics）、2.1–2.3（`Engine::runtime_matrix` 的多 profile 物理保留、X2 三态与缺失依赖、有界 X3 反射/ServiceLoader 推断）、3.1–3.2（versioned plugin descriptor 与 `META-INF/services` 只读 fixture）已落地，入口与边界见 [五维支持矩阵](docs/support-matrix.md#现代5371能力与-p4-新增入口2026-09-20)；3.4 的门禁数字与「结构支持 vs 源码恢复独立状态」的结论已随归档写进 [P4 验证记录](openspec/changes/archive/2026-09-20-p4-modern-semantics/verification.md)。P4 未触碰恢复层：`OutputLevel` 仍只有 `Java8`；`MethodParameters` 与嵌套等类级 metadata 仍不在载荷（声明类自身的 `this_class`/class flags 由后续 `carry-declaring-class-evidence` 补齐，见下）。

**P5 实测优化（已归档，10/10，`cd6f2f0`）**：已交付固定语料、direct 基线、CP/Header facts cache 与差分门禁。cache 存在但默认关闭；index、parallel、merged、single-flight 未实现。测得范围仍是小样本，性能阈值未定，不承诺普遍加速。**A15/A18 在已实现路径的适用范围通过**；实际 usage 节省单独记录，不存在的路径为不适用，不能用其缺席制造未完成项。实测数字、开关与边界见 [支持矩阵](docs/support-matrix.md#p5-实测边界与启用开关2026-09-20)，历史记录见 [P5 verification](openspec/changes/archive/2026-09-20-p5-measured-optimization/verification.md)。

本次收尾已归档（4/4，`fd0aae8`）；`MethodParameters`、handler 根策略和现代源码输出等仍分别保留为覆盖边界（类级事实里**声明类自身**的 `this_class`/class flags 已由 `carry-declaring-class-evidence` 补齐，嵌套与其它类级 metadata 仍是边界），需要时各自开 change，不混入已关闭的正确性修复。轻量调用方仍可直接依赖 `jarde-reader`/`jarde-query`。

`Engine::query` 保持 physical X0/X1，`references_definition`/`may_dispatch_to` 仍返回 UnsupportedAnalysis；`resolve_symbol`/`declaration_references` 是显式运行环境下的独立入口，回答类/字段/方法的**声明**解析与声明引用，dispatch 报告已知候选与 open-world 证据，不声称完整 JVMS 实现或 runtime selection（契约见主规格 `demand-resolver`）。`Engine::analyze_method` 可调度到 SSA；CLI 的 `analyze_method` 接收同形的 `environment`、`method`、`stages`，返回 `method_analysis`。适配层只提供 `input_path` 打开的单一 snapshot，不重写请求中的身份；方法报告含阶段与结果平面，P2 报告保持 Bytecode 契约；`jarde_jvm::analyze_method_ir` 另以只读 `MethodIr` 交付同次运行的实际表。`Engine::recover_method` 消费该载荷，CLI 同名 operation 返回方法分析与恢复报告，分析不重复执行。

恢复产物目前是**方法体**，包含 `text`、source map、rules/profile、诊断和独立结果平面。报告还带闭合的 `content`（`not_produced`/`explanation_only`/`contains_statements`），从最终提交的结构判断产物是否含实际发射的 Java 语句；`Produced` 仅表示产物已交付，`content` 不证明完整恢复，也不证明可编译或语义等价。完整结构可标记 Java/Structured，有低级引用时为 Mixed/Fallback；生产请求保持 `compile_status=NotAttempted`、`semantic_validation=Unproven`、`verification=NotPerformed`。门面已提供参数/receiver/debug 与按需成员证据（P3 3.1/3.2），accessor 的字段访问可从公开入口直接呈现；`carry-declaring-class-evidence` 又补上 driver Header **同一次**读取里的**声明类自身**两项（`this_class` 与 class `access_flags`，只读、不新增扫描），门面级 `declaration@1` 因此能判定普通实例/static、interface `default`/`static`、构造器与 `<clinit>`，同一事实也让构造器的 `super()`/`this()` 与对未初始化 `this` 的字段写入可达；这些 class flags 是 **parse 事实**，不是 dialect 合法性、runtime 解析成功、JVM verification 或质量结论，方法体自身的 quality/compile/verification 不因此升级。**嵌套**（`InnerClasses`/`NestMembers`）、`MethodParameters` 与其它类级 metadata 仍不在载荷，故不声称嵌套。当前支持范围见 [支持矩阵](docs/support-matrix.md)。

`Strict` 的 45.x–51.x 与 52.0 支持只表示结构读取和 version-only gate，不能解释为完整 dialect validation 或 JVM verifier。现代版本、preview、future 与缺失输入分别报告能力限制。实际边界以 [五维支持矩阵](docs/support-matrix.md) 为准。

## 支持范围

- Rust edition 2024；MSRV **1.88.0**。
- P0 支持目标限 **64-bit**。Linux x86_64 已由 `ubuntu-24.04` CI 验证；Linux aarch64 有历史本地证据，本轮复核环境为 macOS arm64。32-bit 未验证且不受支持。
- 生产库离线运行，不启动 JVM、外部反编译器或网络访问。JDK 仅用于显式 ignored 的指令 oracle 与受控编译/执行对照。
- 所有结果分别表达 coverage、execution、diagnostics 和 usage；`VerificationStatus::NotPerformed` 不得解释为 JVM verification 成功。

## Library quickstart

仓库中的 [`examples/inspect_class_header.rs`](examples/inspect_class_header.rs) 是可编译检查的公共 API 示例，只接受 standalone CLASS：

```sh
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo run --example inspect_class_header -- \
  tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class
```

示例使用 `Engine`、`ArtifactInput::Path`、`ClassTarget::Root`、`InspectionMode::Strict` 和有限 `Limits`，并打印版本、类名、成员数量、verification 与最终 usage。ZIP/JAR/WAR 需要先枚举并把**完整枚举所得的 `PhysicalEntry`**传给 `ClassTarget::Entry`；不能只用 ordinal 或 path 重新定位。

### 列出类与成员，并交回物理身份

[`examples/navigate_artifact.rs`](examples/navigate_artifact.rs) 是可编译检查的导航示例，接受 standalone CLASS 或 ZIP/JAR/WAR：

```sh
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo run --example navigate_artifact -- fuzz/corpus/query/minimal-jar
```

它走完整条链，且**不需要调用方拼装任何身份**：

1. `Engine::list_class_candidates(snapshot, scope, budget)` —— 只按 raw name 划分 scope，**零 Header 读取**；每个 entry 恰好出现一次（类候选或普通 resource），报告里的 `class_headers` 为 0。
2. `Engine::list_class_declarations(snapshot, scope, budget)` —— 真实读取每个候选，交回 `ClassDeclarationItem`：`definition`（location + class bytes digest/length + variant）、`this_class`、class access flags、super/interfaces、raw path 与声明名的一致性，以及成员表停止位置；未确认的候选列在 `unconfirmed` 里。
3. `Engine::list_members(snapshot, &definition, budget)` —— 用上一步交回的身份读取同一份物理定义，一次有界读取给出类声明、字段与方法；方法项带 raw name、descriptor、access flags 和**可直接用于方法请求**的 `PhysicalMethodId`。
4. `Engine::find_targets(snapshot, scope, &query, budget)` —— 点分隔名（`com.demo.A`）与内部名（`com/demo/A`）指向同一物理身份；同名多定义与同名多 descriptor 返回**全部**候选而不静默取第一个；无匹配返回空候选与已扫描范围。

同名不同 origin（上层目录 entry 与显式展开的嵌套库、重复 ordinal、相同字节）**全部**返回：不合并、不 first-wins，选择依据是每条候选自带的物理身份而不是遍历顺序。把返回的身份回传给 `list_members` 或方法请求，读到的就是那份物理定义。

**列举只读 Header**：它不解析声明、不加载、不构建 CFG/SSA/Region/Java AST，不推断 WAR/Boot 布局或 classpath 前缀，不给 MR 选择结论，也不证明类可加载、可链接或已验证。要版本与 dialect 平面用 `Engine::inspect_header`；要方法体用方法请求或 `Engine::recover_method`。摘要见 [五维支持矩阵](docs/support-matrix.md#导航列举与身份交接add-artifact-navigation2026-09-20)。

### 任务导向操作：一个目标、一个有界预算、一个显式环境

[`examples/task_operations.rs`](examples/task_operations.rs) 是可编译检查的任务级示例，接受 standalone CLASS：

```sh
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo run --example task_operations -- \
  tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class
```

它不再要求宿主拼装 P2 请求，而是把「给 artifact 与一个目标 → 得到结果」收进库内：

1. **一个目标选择**：`ClassRef`/`MethodRef`/`BodyRef` 接受 friendly 名称（点分隔/内部类名、成员名与可选 descriptor）或调用方已有的物理身份，两者收敛到同一 `PhysicalDefinitionId`/`PhysicalMethodId`；名称路径复用导航匹配与歧义规则。歧义不是失败：操作返回 `OperationOutcome::Ambiguous`（候选 + 生效 `limits` + `coverage`/`execution`/`diagnostics`）且**不执行任何分析**，调用方回传候选自带身份后才继续；身份不属于本次 artifact 是 `operation_target_snapshot_mismatch`，不会被同名定义替换。选择先看搜索自身是否完成，再看候选数：搜索因损坏候选、预算耗尽、取消或成员表中断而未完成时返回 **`OperationOutcome::Incomplete`**（同样带已确认候选、`coverage`、`execution`、`diagnostics` 与实际用量，且同样不执行分析）——零或一个已确认候选既不能当「未找到」也不能当「唯一」，未完成的搜索不会退回 `operation_target_not_found`；完整搜索真的没有匹配才是输入错误，完整搜索确认多个候选才是 `Ambiguous`。
2. **操作自选 stage**：`MethodOperation::{Analysis, Recovery}` 各有固定 stage 表（`MethodOperation::stages()`，今天是整条 P2 前缀），`Engine::analyze_target`/`Engine::recover_target` 不接受调用方的 stage 列表，并在报告里发布实际集合（`stages`）；把同一列表显式交给 `Engine::analyze_method` 得到同一调度与同一 stage 结果，底层入口的 `analysis_no_stages` 与 `ir_pass_*` 校验保持原语义。
3. **有界默认预算 + 少量覆盖**：`task_limits(&overrides)`/`task_budget(&overrides)` 是默认集的唯一来源，`OVERRIDABLE_BUDGET_DIMENSIONS` 只列 5 个高频维度（`output_bytes`、`elapsed_millis`、`result_items`、`class_headers`、`method_bodies`）；未知维度是 `budget_override_dimension_unknown`，0 是 `budget_override_invalid`，都不静默取默认。报告发布生效的完整 `Limits`（`report.limits`）与 `UsageSnapshot`（`report.usage`）；覆盖真的截断工作时得到真实 `Partial`/`Cancelled` 并带终止维度。
4. **三种显式环境策略**：`EnvironmentRequest::build` 按 `EnvironmentPolicy::{SingleClass, PlainJar, ExplicitClasspath}` 组装既有 validator 能校验的声明（roots + `parent_first` delegation + `class_path` module mode，一个 loader、一个 domain、无 provider）。它**不推断**：Manifest `Class-Path`、WAR/Boot 布局检出、嵌套库都不生成 roots（后两者需要调用方显式声明的 root）；`EnvironmentPolicy::Layout` 在本阶段是明确的 `environment_policy_layout_not_provided`，不返回一组声称等价于容器加载的 roots。
5. **类视图**：`Engine::class_view(snapshot, scope, request, budget)` 一次类 Header 读取 + 一次成员列举 + 按需方法 Body，全部共享一个总预算；每请求的 `class_headers` 恰好 1（不会逐方法重读），`method_bodies` 只在真的请求了带 `Code` 的成员时 +1。`abstract`/`native` 以 `ClassViewBody::NotDeclared` 如实陈述（不伪造空 body），损坏成员记录只停它自己的记录：类、前缀成员与其它 body 仍然发布（A13），未走到或读取失败的成员以 `ClassViewBody::Refused` 隔离。顶层 `execution` 在库内汇总搜索、类读取与每个被请求 body 的停止：任一 body 未完成，直接库调用读到的顶层就不是 `Complete`（宿主不必再遍历 body 补算）；每个 body 保留自己的结果、coverage 与诊断，类/成员的结构 coverage 按自身证据报告（成员表确实读完就仍是 `complete_within_schema`），而取消或共享维度耗尽会停止启动后续 body。
6. **引用按 owning method 组织**：`ReferenceGrouping::{from_query, from_declaration}` 只重组 item 列表，把方法体内命中按物理方法身份分组并保留 BCI/evidence，类级与 resource 命中留在 `class_level`/`resources`（不分配给任何方法）；`ReferenceFinding::class()` 区分常量池候选、结构 use-site 与 `ResolvedDeclaration` 三类，扫描自身的 `coverage`/`execution`/`diagnostics`/未决候选计数原样保留（不补全候选、不改变解析状态）。
7. **恢复呈现先给交付内容**：`RecoveryPresentation::of(&RecoveryReport)` 只读报告既有字段（`content`、`quality`、`outcome`、`execution`），`parts()` 按 content → quality → 停止原因排序；它不读 `text`、不剥注释、不数 token，因此说明文本里出现 `return` 之类字面量不改变呈现。

**明文边界**：本层不新增 crate、依赖、持久状态或后台服务，不发明 `Session`/`Workspace`/`Project`，不做自动 classpath 推断、依赖下载、批量/整 artifact 恢复，也不承诺性能。WAR 布局策略依赖 `bind-prefixed-load-roots`，跨请求复用依赖 `bound-container-lookup`：两者落地前不实现、不验收重叠部分。摘要见 [五维支持矩阵](docs/support-matrix.md#任务导向操作add-task-oriented-operations2026-09-20)。

## JSON CLI quickstart

CLI 从 stdin（或 `--request FILE`）读取一个不超过 1 MiB 的 JSON 请求。下面检查已提交的 ECJ 4.6.1 / classfile 52.0 fixture 中 `finallyPath(I)I`；方法名和 descriptor 按 JVM 原始字节数组传递：

```sh
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo run -p jarde-cli -- <<'JSON'
{
  "input_path": "tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class",
  "limits": {
    "input_bytes": 1048576,
    "archive_entries": 1024,
    "entry_bytes": 1048576,
    "read_bytes": 2097152,
    "class_bytes": 1048576,
    "attribute_bytes": 1048576,
    "code_bytes": 262144,
    "result_items": 10000,
    "output_bytes": 4194304,
    "class_headers": 1024,
    "method_bodies": 1024,
    "ir_items": 1048576,
    "ir_edges": 1048576,
    "analysis_steps": 1048576,
    "normalization_clones": 4096,
    "nested_depth": 8,
    "dependency_depth": 64,
    "elapsed_millis": 30000
  },
  "operation": {
    "kind": "inspect_method_bytecode",
    "target": { "kind": "root" },
    "selector": {
      "name": [102, 105, 110, 97, 108, 108, 121, 80, 97, 116, 104],
      "descriptor": [40, 73, 41, 73]
    }
  }
}
JSON
```

十八项 limit 在 CLI 中逐项必填，缺失即协议错误。P2 的 `class_headers`/`method_bodies` 计读取尝试，`ir_items` 计派生存储项，`ir_edges` 计图/def-use 边，`analysis_steps` 计分析工作，`normalization_clones` 计子程序克隆；`nested_depth` 与 `dependency_depth` 为独立高水位。这些维度已接入真实管线。契约要求增长前计费；复核发现的 Frame throw-site × locals 临时存储与部分 Frame/SSA 装配漏计已由 4.3b 关闭（frame 槽存储按槽在分配前计费，SSA 发布改为 clone 前计费），但计费仍是**预算上界**而非逐 pass 构造计数器。限制不承诺进程 RSS 或硬实时中断。

成功响应 exit code 为 0、`status` 为 `"ok"`，并包含精确的 `transport.response_bytes`（含结尾换行）。调用方仍须检查报告里的 `execution`、coverage 和 diagnostics：方法分析内部的 Partial/Cancelled 不被提升为 transport 错误，也不表示分析完成。协议错误及无法建立报告的打开/定位失败保持 `status: "error"`；具体边界见对应入口测试。

方法分析保持 `representation=Bytecode`、`syntax_status=NotJava`、`compile_status=NotAttempted`、`verification=NotPerformed`。产出 CanonicalCFG 后为 Conservative，否则 Fallback；只有本次 Ssa 阶段 Completed 才报告 LocalInvariants。仅 Frame 完成、阶段停止或 abstract/native 无 Body 都是 Unproven。库/CLI 逐字段对照见 [`json_cli.rs`](crates/jarde-cli/tests/json_cli.rs)，实际公开调用见 [`resolve_and_analyze.rs`](examples/resolve_and_analyze.rs)。

Library 可用 `Budget::with_cancellation_token` / `CancellationToken` 注入协作取消；P0 CLI 不暴露取消 token 注入，只支持 `elapsed_millis`。取消和 elapsed 都在协作检查点生效，不是强制抢占或硬超时。

### P1 查询（structural XRef）

`query` operation 接受 relation、target、`physical` scope、consumer schema、`max_items` 和可选 `cursor`；snapshot 由适配层打开并在报告里回显。下面的请求在已提交的 ECJ 4.6.1 fixture 上查找 `java/lang/Object.<init>()V` 的调用点（owner/name/descriptor 是 JVM 原始字节数组）：

```sh
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo run -p jarde-cli -- <<'JSON'
{
  "input_path": "tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class",
  "limits": {
    "input_bytes": 1048576,
    "archive_entries": 1024,
    "entry_bytes": 1048576,
    "read_bytes": 2097152,
    "class_bytes": 1048576,
    "attribute_bytes": 1048576,
    "code_bytes": 262144,
    "result_items": 10000,
    "output_bytes": 4194304,
    "class_headers": 1024,
    "method_bodies": 1024,
    "ir_items": 1048576,
    "ir_edges": 1048576,
    "analysis_steps": 1048576,
    "normalization_clones": 4096,
    "nested_depth": 8,
    "dependency_depth": 64,
    "elapsed_millis": 30000
  },
  "operation": {
    "kind": "query",
    "relation": "mentions_symbol",
    "target": {
      "kind": "symbol",
      "value": {
        "kind": "method",
        "owner": [106,97,118,97,47,108,97,110,103,47,79,98,106,101,99,116],
        "name": [60,105,110,105,116,62],
        "descriptor": [40,41,86]
      }
    },
    "physical": { "kind": "snapshot_all" },
    "consumers": { "version": 1, "kinds": ["invocation"] },
    "max_items": 0,
    "cursor": null
  }
}
JSON
```

响应 `status: "ok"`，`report.items` 含一条 `consumer: "invocation"`、`operation: "invoke_special"`、CP index 8、BCI 1、opcode 183 的 item，`execution.status` 为 `complete`，`coverage.dimensions.artifact_structural.state` 为 `complete_within_schema`。`max_items` 只限制每页条目、不改变查询含义；`page.cursor` 是为同一查询身份签发的续页 token（`QUERY_ENGINE_SCHEMA = 2`，绑定 snapshot、view、relation、完整 target 与 consumer schema），跨目标或跨快照重放会被拒为 `query_cursor_mismatch`。请求未实现类别（`verification`/`debug`）时不会返回 `complete_within_schema`；`references_definition`/`may_dispatch_to` 返回 `analysis: unsupported_analysis` 并保留原始常量池候选。预算、取消或损坏输入只会产生 `partial`/`cancelled`/`failed` 与定位诊断，不会伪装成完整无命中。

### 物理树枚举与显式前缀加载位置（`bind-prefixed-load-roots`）

`enumerate_artifact_tree` 是薄的 JSON operation：它打开 `input_path`，把该 snapshot 交给库的既有入口，返回库自己的 report（`containers`/`entries`/`layout_nodes`/`coverage`/`execution`/`diagnostics`），limit 仍逐项必填。它不声明 root、不推导 loader 委派、顺序或 classpath，也不改写 snapshot；普通 `enumerate` 仍只报告顶层容器。

```sh
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo run -p jarde-cli -- --request app-tree.json
# app-tree.json 的 operation 只有一项：
#   "operation": { "kind": "enumerate_artifact_tree" }
```

拿到真实的 container origin 与 entry 身份后，**调用方自己声明**加载位置（`LoadRoot`，JSON 拼写即其 `kind`）：

| 形状 | JSON | 含义 |
| --- | --- | --- |
| `StandaloneClass { snapshot }` | `{"kind":"standalone_class","snapshot":"…"}` | 整个 CLASS 快照本身就是一个定义，内部名由它自己的 `this_class` 给出 |
| `Container { origin, prefix }` | `{"kind":"container","origin":{"snapshot":"…","root_container":"root","steps":[]},"prefix":[87,69,66,45,73,78,70,47,99,108,97,115,115,101,115,47]}` | 快照的一个 container（完整 origin 链，可指向 nested container）加**归档内 raw 字节前缀** |
| `External { id }` | `{"kind":"external","id":"host-jdk"}` | 未提供的声明；不解析，报告 `unreadable_root` |

查名按 `prefix + internal_name + b".class"` 逐字节拼接：不 trim、不 URL 解码、不大小写折叠、不折叠 `.`/`..`/反斜杠，也不要求 ZIP 里存在对应的目录 entry。前缀为空或以 `/` 结尾；非空却不以 `/` 结尾是环境问题 `invalid_root_prefix`（该 root 上的诊断，不补分隔符）。选中候选的 Header 内部名必须与请求名一致，否则返回该候选自身的 `resolution_definition_name_mismatch`，不尝试下一个 root。

这个形状**替换**了旧的 `snapshot`/`artifact_tree` 两种 root（**BREAKING**，无兼容层、无 schema 适配）：旧 JSON 变体不再能反序列化。前缀只影响 container 目录内的查找 key（`ArchiveNameBytes` 参与环境身份，不进入 container facts cache 的 key），跨 root 优先级仍由声明顺序与 loader 委派决定。**支持边界**：前缀是通用机制，等于 Servlet/Boot 专用加载规则**未实现**——不读 `classpath.idx`、不自动排序 `BOOT-INF/lib`、不执行 launcher，`BOOT-INF/classes/` 只是同一种前缀的受控样本。

### 任务导向命令（`add-task-oriented-cli`）

命令行现在有两个入口：`--request`/stdin 的 JSON 控制面**保持原样**（18 项 limit 逐项必填、既有 6 个 operation、`status`/`result`/`transport` envelope、0/1 退出语义都不变），以及新增的任务链子命令。子命令只做三件事：把 friendly 参数翻成库请求、把库返回的那一份报告渲染成文本或 JSON、按报告自己的平面给出退出状态。

| 命令 | 调用的库入口 | 参数做什么 |
| --- | --- | --- |
| `list-classes` | `Engine::list_class_candidates` / `Engine::list_class_declarations`（`--evidence candidates\|declarations`，默认 `declarations`） | 传递 scope 与证据等级 |
| `list-members` | `Engine::list_members` | 把 `--definition` 的物理身份原样传入（不重拼、不按名字重找） |
| `references` | `Engine::query` + `ReferenceGrouping::from_query` | 把类名/成员/relation/consumer 交给库的扫描与分组 |
| `class-view` | `Engine::class_view` | 把类名或定义身份、以及要打开的 `--body` 传入 |
| `recover` | `Engine::recover_target` | 把 `--method`/`--class-name`、环境策略、profile 与 loader 传入 |

公共参数：`--input PATH`、`--scope JSON`（默认 `{"kind":"snapshot_all"}`，可声明 `artifact_tree` 的真实 container 身份）、`--budget DIMENSION=LIMIT`（可重复；五个可覆盖维度 `output_bytes`/`elapsed_millis`/`result_items`/`class_headers`/`method_bodies`，其余保持有界默认，未知维度或 0 是用法错误）、`--format text|json`（默认 `text`）、`--output FILE`。需要身份或文档的参数（`--definition`、`--method`、`--body-method`、`--scope`、`--root`、`--profile`）接受库自己的 JSON 文档，或 `@FILE`。

**JSON 与文本同源**：`--format json` 写的就是库报告自身的序列化（`OperationOutcome` 连 `outcome` 标签一起），没有 envelope、没有改名、没有 CLI 专有字段，因此可以对同一请求做逐字段对照；`--format text` 是同一份文档的投影，每行都是 `JSON 字段路径 = 值`，可逐项回到 JSON 字段。唯一的正文例外是恢复的 Java 文本：它按 `recovered.recovery.text` 的字段值逐字节写出，所以 `--format text --output A.java` 得到的就是报告里的那段代码。

**诊断与正文分离**：文本模式下正文（或内容字段行）走标准输出/`--output`，`limits`/`usage`/`coverage`/`execution`/`diagnostics` 五个记账平面走标准错误；JSON 模式下它们都是文档里的字段。失败（用法/输入错误、交付失败）不写标准输出，而是把库自己的 `error` + `usage` 文档写到标准错误。`--output` 与标准输出走同一次序列化与同一个 `output_bytes` 检查：收费被拒时**不创建文件**，执行停止时写出的是报告自己的 `partial`/`cancelled` 平面而不是伪装成功的文档。

**退出状态**：

| 状态 | 含义 | 验证用的 fixture |
| --- | --- | --- |
| `0` | 报告自己发布的每个执行平面都是 `complete` | `list-classes --input app.jar --format json` |
| `1` | 文档无法交付（写文件或标准输出失败） | `... --output <不存在的目录>/x.json` |
| `2` | 用法/输入错误：参数、身份文档、库的请求级拒绝、`output_bytes` 不足以交付文档 | 缺 `--consumer` 的 `references`；`recover --class-name p/Absent` |
| `3` | 名称歧义，需要调用方选择（库未执行任何操作，候选就是报告） | 两个 origin 的 `class-view --class-name p/Base` |
| `4` | 执行未完整：`partial`/`cancelled`/`failed`，**即使已交付可靠前缀也不为 0** | `list-classes --evidence declarations --budget class_headers=1` |

**任务链**（每个后续命令吃前一条命令打印的物理身份，不需要手工重拼）：

```sh
J=app.jar
# 1. 列类：`items[i].definition` 就是要交给下一条命令的身份
cargo run -q -p jarde-cli -- list-classes --input $J --evidence declarations --format json
# 2. 列方法：身份原样传回，`items[i].identity` 是方法身份
cargo run -q -p jarde-cli -- list-members --input $J --definition @definition.json --format json
# 3. 看引用：符号（类名 + 成员名 + descriptor）与声明的 consumer 类别
cargo run -q -p jarde-cli -- references --input $J --class-name p/Base --member method \
  --member-name foo --descriptor '()V' --consumer invocation --format text
# 4. 打开代码：方法身份原样传入，环境由调用方声明
cargo run -q -p jarde-cli -- recover --input $J --method @method.json --policy plain-jar \
  --format text --output Base.java
```

`class-view` 是另一条“打开代码”的路径：`--body 'foo()V'` 只为被请求的那个方法计一次 `method_bodies`，同一个类的其他方法不会被读取（`native`/`abstract` 这类无 Body 的成员不计尝试，返回 `not_declared`）；`--body-method` 接受方法身份。

**发现面**：导航命令只用库的列举与显式 `enumerate_artifact_tree`；普通枚举仍不递归（嵌套库只是一个普通 entry），`--scope {"kind":"artifact_tree","root_container":…}` 才进入嵌套 container 的 entries。CLI 不自行解包、不按路径推断 container、不生成 root：加载位置只能由 `--root`（`LoadRoot` 文档）或 `--policy` 显式声明。**不包含**：批处理/整 artifact 恢复、GUI/MCP、交互式 TUI、自动 classpath 推断、配置持久化、颜色/分页、性能承诺；不新增 crate 或依赖。验收见 [`task_cli.rs`](crates/jarde-cli/tests/task_cli.rs)。

## 规格与验证

- [五维支持矩阵](docs/support-matrix.md)
- [P0 实际验证记录](openspec/changes/archive/2026-09-17-establish-p0-foundation/verification.md)
- [P1 归档（proposal / design / tasks / verification）](openspec/changes/archive/2026-09-17-p1-query-xref/)
- [已生效主规格](openspec/specs/)（P0–P5 共 18 份；阶段归档不等于所有输入已验收）
- [P2 验证记录（含验收映射与 5.4 门禁）](openspec/changes/archive/2026-09-19-p2-jvm-ir/verification.md)
- [P3 恢复验证记录](openspec/changes/archive/2026-09-20-p3-java8-recovery/verification.md)
- [OpenSpec 入口](openspec/README.md)
- [P0 归档 proposal / design / tasks](openspec/changes/archive/2026-09-17-establish-p0-foundation/)
- [已生效的 P0 主规格](openspec/specs/)
- [架构验收映射](openspec/acceptance.md)
- [技术栈与依赖选型](openspec/dependencies.md)
- [架构基线](JVM_Rust_Engine_Final_Architecture.md)

低内存本地验证默认设置 `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1`。OpenSpec 静态验证：

```sh
npx --yes @fission-ai/openspec@1.11.0 validate --all --strict --no-interactive
```
