# jarde

jarde 是纯 Rust、library-first 的 JVM artifact 分析引擎。P0/P1 已完成并归档：有界不可变快照、CLASS/JAR/WAR 读取、嵌套物理视图、MR 选择、Header/bytecode inspection 和 X0/X1 查询已交付。

**当前完成判断（2026-09-20，`fd0aae8`）**：P0–P5 与分层均已按各阶段范围归档，`cd6f2f0` 复核出的两条 P1 已由本轮关闭（`close-recovery-correctness-gaps`，4/4）：`(x + 1) + ++x` 不再输出对输入 7 返回 17 的 Java，改为保留被拒读取的可靠降级；被拒 cast 前的字段读取重新出现在产物与 source map 中，含字段链。两条反例、正向对照与执行基线已进入永久语料（`tests/fixtures/p3-nested-eval/`、`tests/fixtures/p3-refused-cast/`）与库/CLI 验收。实施、变异与门禁证据见 [收尾验证](openspec/changes/archive/2026-09-20-close-recovery-correctness-gaps/verification.md)。

**P3 恢复（已归档，12/12，`250fe1f`）**：方法体、结构/模式、声明/作用域、按需 accessor 与物理方法映射已交付；原 P3-R1–R7 的具体反例均已关闭，其中直接 `return x++` 通过可靠降级关闭。受控编译/执行对照已经运行；这不等于所有表达式已证明等价，也不等于任意 JVM 方法都能还原为完整 Java 编译单元——本轮 R8/R9 的关闭见上方当前完成判断。

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
