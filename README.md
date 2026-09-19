# jarde

jarde 是纯 Rust、library-first 的 JVM artifact 分析引擎。P0/P1 已完成并归档：有界不可变快照、CLASS/JAR/WAR 读取、嵌套物理视图、MR 选择、Header/bytecode inspection 和 X0/X1 查询已交付。

**当前状态（2026-09-20）**：P2（29/29）与分层（7/7）均已归档；`jarde-java` 已交付 1.1–1.3、1.3d、2.1–2.4 与 3.1–3.3，**P3 12/12**（本片 3.4 是最后一项）。复核出的 P1/P2 缺陷——`return x++` 的旧值重读、被拒 cast 的 producer effect 丢在降级里、branch-local 声明作用域、确定性比较的 elapsed 假红——**均已关闭**；另关闭 P3-R5（`boolean` 参数按描述符定型）、R6（slot 复用按 spec 拆分）与 R7（每条已解码指令必须被块或 `unreachable` 交代）。**新增可重放的编译/执行对照**：`cargo test --test p3_execution_comparison -- --ignored` 把产物包成编译单位后用 `javac --release 8` 编译并执行，CI 的 JDK job 已挂。仍未承诺完整源码或语义等价，也未实现 verifier。现状与证据见 [P3 验证记录](openspec/changes/p3-java8-recovery/verification.md)。

**P4 现代语义（已归档，10/10，`88416ab`）**：1.1–1.3（release registry，以及 record/sealed/condy/concat 事实与非法 fixture、golden diagnostics）、2.1–2.3（`Engine::runtime_matrix` 的多 profile 物理保留、X2 三态与缺失依赖、有界 X3 反射/ServiceLoader 推断）、3.1–3.2（versioned plugin descriptor 与 `META-INF/services` 只读 fixture）已落地，入口与边界见 [五维支持矩阵](docs/support-matrix.md#现代5371能力与-p4-新增入口2026-09-20)；3.4 尚待把门禁数字与「结构支持 vs 源码恢复独立状态」的结论写进验证记录（本轮 3.3 已实跑 1075 passed / 0 failed / 3 ignored，fmt/clippy 1.98.1 与 `openspec validate --all --strict` 14 passed 干净）。P4 未触碰恢复层：`OutputLevel` 仍只有 `Java8`，类级事实与 `MethodParameters` 仍不在载荷。

接下来按 [阶段路线](openspec/roadmap.md) 归档已通过出口的 P3，并完成 P4 的 3.4 与 P5；P3 的 12 项已全部落地，仍未做的三处（`MethodParameters`、类级事实、canonical 对「handler 入口即根」的裁决）记录在 [P3 验证记录](openspec/changes/p3-java8-recovery/verification.md)。轻量调用方仍可直接依赖 `jarde-reader`/`jarde-query`。

`Engine::query` 保持 physical X0/X1，`references_definition`/`may_dispatch_to` 仍返回 UnsupportedAnalysis；`resolve_symbol`/`declaration_references` 是显式运行环境下的独立入口，回答类/字段/方法的**声明**解析与声明引用，dispatch 报告已知候选与 open-world 证据，不声称完整 JVMS 实现或 runtime selection（契约见主规格 `demand-resolver`）。`Engine::analyze_method` 可调度到 SSA；CLI 的 `analyze_method` 接收同形的 `environment`、`method`、`stages`，返回 `method_analysis`。适配层只提供 `input_path` 打开的单一 snapshot，不重写请求中的身份；方法报告含阶段与结果平面，P2 报告保持 Bytecode 契约；`jarde_jvm::analyze_method_ir` 另以只读 `MethodIr` 交付同次运行的实际表。`Engine::recover_method` 消费该载荷，CLI 同名 operation 返回方法分析与恢复报告，分析不重复执行。

恢复产物目前是**方法体**，包含 `text`、source map、rules/profile、诊断和独立结果平面。完整结构可标记 Java/Structured，有低级引用时为 Mixed/Fallback；生产请求保持 `compile_status=NotAttempted`、`semantic_validation=Unproven`、`verification=NotPerformed`。门面已提供参数/receiver/debug 与按需成员证据（P3 3.1/3.2），accessor 的字段访问可从公开入口直接呈现；仍不在载荷里的是类级事实（`InnerClasses`/`ACC_INTERFACE`），故不声称嵌套。当前支持范围见 [支持矩阵](docs/support-matrix.md)。

`Strict` 的 45.x–51.x 与 52.0 支持只表示结构读取和 version-only gate，不能解释为完整 dialect validation 或 JVM verifier。现代版本、preview、future 与缺失输入分别报告能力限制。实际边界以 [五维支持矩阵](docs/support-matrix.md) 为准。

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

## 规格与验证

- [五维支持矩阵](docs/support-matrix.md)
- [P0 实际验证记录](openspec/changes/archive/2026-09-17-establish-p0-foundation/verification.md)
- [P1 归档（proposal / design / tasks / verification）](openspec/changes/archive/2026-09-17-p1-query-xref/)
- [已生效主规格](openspec/specs/)（P0 的 analysis-contracts / artifact-snapshots / classfile-inspection，P1 的 artifact-views / query-api / structural-xref，P2 的 demand-resolver / jvm-ir / conservative-output）
- [P2 验证记录（含验收映射与 5.4 门禁）](openspec/changes/archive/2026-09-19-p2-jvm-ir/verification.md)
- [P3 恢复验证记录](openspec/changes/p3-java8-recovery/verification.md)
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
