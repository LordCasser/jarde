# jarde

`jarde` 是纯 Rust、同步、library-first 的 JVM artifact 有界静态检查底座。**P0 已完成并归档**：不可变 CLASS/JAR/WAR 快照、顶层物理 ZIP entry 枚举与读取、classfile Header inspection、按方法的原始指令边界 inspection、预算/协作取消、公共 `Engine`、单请求 JSON CLI、支持矩阵、低内存 CI、公共示例与验证记录均已落地。**P1 的 1.1–3.3 已完成并有证据**：query/view/identity 模型、显式 `enumerate_artifact_tree`（有界 nested 遍历、Boot/WAR 物理布局 evidence、origin chain 复核）、标准 MR-JAR 选择与合规诊断、结构 XRef（code/metadata/bootstrap/resource consumer，含 Record component 与 Code 内注解、实际使用点 descriptor 类型）、target-bound 游标与分页、CLI `query` operation，以及 P1 验收语料索引、结构 XRef golden、proptest 性质与有界 fuzz 门禁；3.4（文档、完整 CI 与归档）是收口步骤。IR 阶段仍为 planned；**P2 进行中**，其中预算维度（1.3）与显式运行环境下的类名查找（2.1，`Engine::resolve_symbol` 对 `SymbolRef::Class` 给出 `Resolved`/`Missing`/`Ambiguous`）已落地并有证据，IR 阶段尚未实现；成员解析（2.3）已落地并有证据：`SymbolRef::Field`/`Method` 按 JVMS 5.4.3 解析**声明**（字段、class method、interface method 三条搜索路径与接口的 maximally-specific 集合，`target` 保留原始引用），施加调用种类（静态/实例、class/interface owner、`<init>` 仅 `invoke_special`、`invoke_special` 不解析抽象方法）与访问规则（5.4.4，调用方类未知时给 `resolution_access_not_checked`），并对 default conflict、signature-polymorphic 与数组 owner 给出明确分支；声明引用查询（2.4）已落地：`Engine::declaration_references` 按**成员形状**（name/descriptor，owner 不参与）从结构 consumer 发现候选，逐条解析后只发布解析到请求声明的 use-site（`Sub.foo` 的调用点能为 `Base.foo` 的声明作证），未使用的常量池条目不算引用，未决候选（缺失/歧义/链接与访问失败/数组 owner/预算/取消/损坏/证据不含指令级种类）保留 use-site 而不当作已排除，签名多态声明另按 owner+name 匹配；按需 Header 闭包（2.2）已落地：请求目标、`super_class` 链与父类/接口闭包经同一请求内 `(loader, internal_name)` 去重的闭包解析，三个解析报告新增 additive 的 `reads` 字段记录实际读取的 `(definition, loader)` 绑定与读取理由；已知范围 dispatch（2.5）已落地：`ResolutionRequest.dispatch = Some(DispatchScope)` 时在显式物理范围内枚举该声明的已知实现/覆盖（CHA-lite，只读 Header，`reads` 的 `DispatchScope` 理由），每个候选附 open-world 证据，`open_world` 在有证据、范围存在不可判定位置或预算/取消停止时为 true，报告不含“唯一运行目标”字段；范围内的停止按发生位置分开：查找或 listing 被截断时 `coverage` 保留未检查的后续位置，发布阶段的停止（`ResultItems` 被拒）只把覆盖标为 Partial、不伪造 skipped，而进入报告的每条诊断各计一次 `ResultItems`（环境平面、停止解释与「范围未运行」说明不收费）。

这不是反编译器的完成版本。X1 只在声明的 consumer schema 与物理视图内提供结构引用：`references_definition`/`may_dispatch_to` 返回 `UnsupportedAnalysis`（P2 resolver），`Verification`/`Debug` 类别不实现，不构建 CFG/SSA/Java AST；resolution 只在显式 `ResolutionEnvironment`（domain/provider 绑定与入口提供的不可变 snapshot）下实现类名查找、按需 Header 闭包（含 `reads` 证据）、成员声明解析（字段/方法、访问规则、调用种类、default conflict、signature-polymorphic 与数组 owner 分支）与声明引用查询（`Engine::declaration_references`：按成员形状匹配候选、逐条解析并与声明比对，未决候选保留 use-site 而不当作已排除）；已知范围 dispatch（`dispatch = Some(DispatchScope)`）已落地：在显式物理范围内枚举该声明的已知实现/覆盖（CHA-lite，只读 Header），每个候选附 open-world 证据，`open_world` 在有证据、范围不可判定或预算/取消停止时为 true，报告不含任何“唯一运行目标”字段；Java recovery 与 runtime view 均未实现。`Strict` 支持 45.x–51.x 与 52.0，且只表示 **version-only gate 下的结构读取和方法指令 inspection**，不是完整 dialect validation 或 JVM verifier；52 的非零 minor 不属于 Java 8 profile，`Strict` 拒绝。53–71、preview 与 future release 可由 `Forensic` 读取边界可靠的 Header 结构，但能力分别标为 `StructuralProbeOnly`、`UnsupportedPreview`、`FutureRelease`；`Strict` 均拒绝。完整、逐输入类型与版本的边界见[五维支持矩阵](docs/support-matrix.md)。

## 支持范围

- Rust edition 2024；MSRV **1.88.0**。
- P0 支持目标限 **64-bit**。Linux x86_64 已由 `ubuntu-24.04` CI 验证；当前本地证据为 Linux aarch64。32-bit 未验证且不受支持。
- 生产库离线运行，不启动 JVM、外部反编译器或网络访问。JDK 仅用于一个显式 ignored 的测试 oracle。
- 所有结果分别表达 coverage、execution、diagnostics 和 usage；`VerificationStatus::NotPerformed` 不得解释为 JVM verification 成功。

## Library quickstart

仓库中的 [`examples/inspect_class_header.rs`](examples/inspect_class_header.rs) 是可编译检查的公共 API 示例，只接受 standalone CLASS：

```sh
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo run --example inspect_class_header -- \
  tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class
```

示例使用 `Engine`、`ArtifactInput::Path`、`ClassTarget::Root`、`InspectionMode::Strict` 和有限 `Limits`，并打印版本、类名、成员数量、verification 与最终 usage。ZIP/JAR/WAR 需要先枚举并把**完整枚举所得的 `PhysicalEntry`**传给 `ClassTarget::Entry`；不能只用 ordinal 或 path 重新定位。

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

十八项 limit 都是请求级上限，CLI 请求对象里逐项必填（`deny_unknown_fields`，缺一项即协议错误，不静默取默认值）：P0/P1 的 `input_bytes`、`archive_entries`、`entry_bytes`、`read_bytes`、`class_bytes`、`attribute_bytes`、`code_bytes`、`result_items`、`output_bytes`，P2 新增的 `class_headers`、`method_bodies`、`ir_items`、`ir_edges`、`analysis_steps`、`normalization_clones`，以及两个非累加高水位 `nested_depth`（已接受嵌套容器深度）与 `dependency_depth`（依赖闭包深度），另加 `elapsed_millis`。P0/P1 入口目前只使用前九项与两个高水位中的 `nested_depth`；P2 的六个计费维度由 2.x–5.x 的闭包、CFG、Frame/SSA 与规范化阶段逐步开始计费（1.3 只加入维度与计费入口）：`class_headers` 自 2.1 的单次查找起计费，2.2 的闭包与层级展开、2.3 的成员搜索（每处理一个类各一次）另计 `analysis_steps`（每次工作列表迭代一次）并逐层观察 `dependency_depth` 高水位——因此成员解析请求必须给出非零的 `analysis_steps`（否则第一步即按该维度返回 `BudgetExceeded`），而 `dependency_depth = 0` 仍允许读取成员 owner 自身（深度 0 不观察深度），只在需要沿超类链或超接口上升一步时才按该维度停止；`method_bodies`/`ir_items`/`ir_edges`/`normalization_clones` 仍保持零值，直到 3.x/4.x 接通。计数的含义：`class_headers`/`method_bodies` 计读取**尝试**（同一 `(definition, loader)` 绑定在同一请求内由调用方去重，失败尝试仍计一次），`ir_items` 计 frame/local 槽、SSA 值、phi 输入与 origin 成员，`ir_edges` 计 CFG 边（含异常边）与 SSA def-use 边，`analysis_steps` 计工作列表 pop/处理（重复访问计数），`normalization_clones` 计 `jsr`/`ret` 克隆节点，全部在分配/入队/加边/克隆**之前**计费。除两个 depth 高水位外，其余维度约束累计工作量和结果缓冲；这些限制不承诺进程 RSS 或硬实时中断。

成功响应 exit code 为 0、`status` 为 `"ok"`，并包含精确的 `transport.response_bytes`（含结尾换行）。只有在 ZIP 枚举报告已经建立，或目标 `Code` 已定位并已开始 exception-handler / instruction 扫描之后，局部预算耗尽、协作取消或局部输入失败才会作为 `status: "ok"` 内的 `Partial` / `Cancelled` report 返回；调用方必须读取内部 `execution`、coverage 和 diagnostics。协议错误，以及 artifact open、class Header、class 物化或方法/`Code` 定位完成前发生的错误，仍返回 exit code 1 和 `status: "error"`。

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

## 规格与验证

- [五维支持矩阵](docs/support-matrix.md)
- [P0 实际验证记录](openspec/changes/archive/2026-09-17-establish-p0-foundation/verification.md)
- [P1 归档（proposal / design / tasks / verification）](openspec/changes/archive/2026-09-17-p1-query-xref/)
- [已生效主规格](openspec/specs/)（P0 的 analysis-contracts / artifact-snapshots / classfile-inspection，P1 的 artifact-views / query-api / structural-xref）
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
