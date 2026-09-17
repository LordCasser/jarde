# 当前五维支持矩阵

本页是当前用户可见能力状态的单一事实源。P0 的验收要求以[已生效主规格](../openspec/specs/)和[归档 tasks](../openspec/changes/archive/2026-09-17-establish-p0-foundation/tasks.md)为准；P1 的增量以[归档 tasks](../openspec/changes/archive/2026-09-17-p1-query-xref/tasks.md)和[verification](../openspec/changes/archive/2026-09-17-p1-query-xref/verification.md)为准。这里只描述实际实现，不是未来路线承诺。

## 状态词

- **Supported**：当前公共入口实现且有任务内回归证据，仍受请求 budgets 和表中边界约束。
- **StructuralProbeOnly**：只返回边界可靠的已读结构；不承诺该 dialect、语义或 JVM 合法性。
- **PartialOnError**：遇到局部错误后停止并保留可靠前缀，内部 execution 为 `Partial`；不是静默成功。
- **MetadataOnly**：只识别并保留 metadata/位置，不执行其声明的安全或语义操作。
- **NotImplemented**：对应分析面没有实现。
- **Configured**：CI 或入口已配置，但尚无首次实际成功运行证据。
- **Validated**：限定范围已有实际执行证据；括号内限定（如 test-only）是状态的一部分。
- **NotValidated**：实现或依赖可能在该环境工作，但当前阶段没有建立支持证据；按不支持处理。
- **Unsupported**：入口明确拒绝，或超出当前支持范围。

所有 classfile Header 与方法指令结果的 `verification` 都是 **NotPerformed**。`parse: Supported` 只表示表中声明范围的结构读取，不表示完整 JVMS dialect validation、字节码 verifier、链接或运行成功。

X1 列的 `Supported` 一律限定在**请求声明的 consumer schema 与物理视图**内：P1 只分析 `constant_pool_contains`、`mentions_symbol`、`literal_value` 三个关系（`references_definition`/`may_dispatch_to` 返回 `UnsupportedAnalysis` 并保留原始候选），覆盖 Invocation、Field、Type、Constant、Exception、Signature、Annotation、InnerNest、Module、Bootstrap 与 Resource consumer；请求 `Verification`/`Debug` 时进入 `coverage.unsupported_categories` 且不声明 complete-within-schema。query 不按 runtime 视图过滤，`resolution` 恒为 `not_requested`。

## 输入与能力五维矩阵

| 输入/边界 | parse | X1 | resolution | decompile-quality | output-level | 说明 |
| --- | --- | --- | --- | --- | --- | --- |
| Standalone CLASS，45.x–51.x 与 52.0，Strict | Supported（version-only Header + 所选方法 instruction inspection） | Supported（声明 consumer schema 内） | NotImplemented | NotImplemented | Header / raw instruction facts；P1 query 报告（items/page+cursor/coverage/execution/diagnostics/analysis） | 52 的非零 minor 不属于 Java 8 profile并被 Strict 拒绝；不检查完整 CP tag、flags、attribute 合法位置/基数、opcode dialect 或 JVM verification。X1 不构建 CFG/SSA/AST：死代码中的引用仍返回，损坏/截断输入返回非 Complete 与定位诊断。 |
| JAR/WAR 顶层物理 entries | Supported | Supported（同上；同一快照扫全部 entry） | NotImplemented | NotImplemented | Physical entries；选中 entry 的 Header/raw instructions；P1 query 报告 | 保留 ordinal、raw name、压缩/布局/来源；entry inspection 必须携带枚举所得完整 `PhysicalEntry`。nested archive 在该 scope 下只作为普通 entry 的字节，不进入其内部。 |
| ZIP64 顶层目录 | Supported | Supported（与顶层 entries 同一路径；无独立 ZIP64 query 证据） | NotImplemented | NotImplemented | Physical entries | 已覆盖小型真实 ZIP64 结构；不暗示超预算巨型归档可完成。 |
| STORED / DEFLATED entry | Supported | Supported（同上） | NotImplemented | NotImplemented | Bounded materialized bytes / inspection facts | 校验 size/CRC；压缩输入计 `read_bytes`，展开数据计 `entry_bytes`；query 的读取沿同一计费。 |
| Nested JAR/WAR candidate（普通 `enumerate`） | MetadataOnly | Supported（当前容器 entries） | NotImplemented | NotImplemented | `candidate_not_scanned`；P1 query 报告 | 普通物理枚举只标记候选，不递归，也不激活 classpath；query 在该 scope 下不进入 nested 内容。 |
| Nested JAR/WAR（显式 `enumerate_artifact_tree`） | Supported | Supported（`PhysicalScope::ArtifactTree` 内 nested entries） | NotImplemented | NotImplemented | Physical containers / entries / origin / layout evidence | 迭代式有界遍历 STORED/DEFLATED child，逐层复核 nested entry；深度、entry、bytes、elapsed 与取消可返回可靠 Partial/Cancelled 前缀；query 复用同一 provider，不据此声称 runtime 选择。 |
| Boot/WAR layout evidence | Supported（物理 evidence） | Supported（布局 nodes 随 provider 返回） | NotImplemented | NotImplemented | `BOOT-INF` / `WEB-INF` classes/lib layout nodes | 匹配区分大小写，只记录物理 evidence；不执行 launcher，不推断 runtime classpath 或唯一选择。 |
| Multi-release physical variants | Supported（物理枚举） | Supported（各物理变体分别扫描） | NotImplemented | NotImplemented | 每个物理 entry | 标准 MR 选择、合规诊断与 `verification=NotPerformed` 由 `Engine::select_multi_release` 单独提供（P1 1.3）；query 不按 runtime 视图过滤，也不跨 container 消歧。 |
| Classfile 45.x–51.x 与 52.0 | Supported（Strict version-only） | Supported（52.0 与 `--release 8` 有 query 证据；45.x–51.x 走同一 reader 路径，未单独建立 query 证据） | NotImplemented | NotImplemented | Header / raw instruction facts | 45.x 历史 minor 规则保留；52 的非零 minor 不属于 Java 8 profile并被 Strict 拒绝。45.3–52.0 fixture 有证据；不是完整 dialect validation。 |
| Classfile 53–71 | StructuralProbeOnly（Forensic）/ Unsupported（Strict） | NotValidated（按不支持处理） | NotImplemented | NotImplemented | Forensic Header structure | Forensic 仅读取边界可靠结构并标记 structural probe；查询路径没有跨版本证据，不把 Forensic 的 Header 读取解释为 X1 支持。 |
| Preview（modern minor 65535） | UnsupportedPreview（Forensic）/ Unsupported（Strict） | NotValidated | NotImplemented | NotImplemented | Forensic Header structure + diagnostic | Forensic 可读取边界可靠结构但不支持 preview dialect；45.x 的历史 minor 65535 不等同 modern preview。 |
| Future major（>71） | FutureRelease（Forensic）/ Unsupported（Strict） | NotValidated | NotImplemented | NotImplemented | Forensic Header structure + diagnostic | Forensic 可读取边界可靠结构但不把未知 future release 宣称为 dialect 支持。 |
| 合法长度的 unknown attribute | Supported（shell/span） | Supported（声明 schema 内；未知 attribute 不解释） | NotImplemented | NotImplemented | 名称、完整 span/content span | 内容不解释；Header 成功不证明未知内容或方法 Body 合法。已知标准注解位置（含 Record component 与 Code 内 type annotation）已覆盖；未知自定义 attribute 不产事实且不影响 complete-within-schema。 |
| Illegal/truncated instruction | PartialOnError | Supported（PartialOnError：可靠前缀 + stop 诊断 + 非 Complete） | NotImplemented | NotImplemented | 可靠指令前缀、stop BCI/class offset、diagnostic | 首错停止，不在错误后猜测同步；CLI 仍可返回 `status: ok` 的 Partial report。class 候选 magic 缺失/截断/错误返回带 entry origin 的 `query_class_candidate_malformed` 与非 Complete；嵌套注解结构失败不发布该结构的事实。 |
| JAR signature metadata | MetadataOnly | NotImplemented（签名不作为 X1 consumer） | NotImplemented | NotImplemented | metadata kind + `not_verified` | 不验证签名，不推断 trusted。 |

### 已实现与未实现的分析面（P1 收口时点）

- **X1（结构 XRef）已实现声明范围**：`Engine::query` 提供 `constant_pool_contains`（原始 CP 候选）、`mentions_symbol`、`literal_value`，覆盖 code（invoke/field/type/constant/`ldc`/`invokedynamic`/异常表）、metadata（hierarchy/descriptor/Signature/注解含 Record component 与 Code 内 type annotation/Exceptions/ConstantValue/InnerNest/Module）、bootstrap（deferred 图、共享子图、环与预算）与 resource（Manifest 属性、`META-INF/services`），以及实际使用点（`ldc`/`invokedynamic`/bootstrap 可达）的 descriptor 类型。范围外：`references_definition`/`may_dispatch_to` 返回 `UnsupportedAnalysis` 并保留原始候选（P2 resolver）；`Verification`/`Debug` 类别不实现；未知自定义 attribute 不解释；不构建 CFG、SSA 或 Java AST。
- **resolution：全部 NotImplemented。** 不加载依赖、不绑定 loader、不做继承/派发或 runtime selection；`QueryResolution` 恒为 `not_requested`；MR 只按 Manifest 与唯一 winning level 给出选择事实和合规诊断，不据此过滤 query 结果。
- **decompile-quality：全部 NotImplemented。** 不构建 CFG、SSA、IR/AST，不恢复 Java。
- **output-level：** 物理 container/entry/layout evidence、Header、所选方法原始 instruction/handler facts、P1 query 报告（items/page+cursor/coverage/execution/diagnostics/analysis）、MR selection report 与 CLI `query` operation；没有 Java、伪代码，也没有按 runtime 视图过滤的引用集合。
- **已知边界**：cursor 绑定完整查询身份（`QUERY_ENGINE_SCHEMA = 2`，含完整 target 与 digest），跨目标/跨快照续页被拒；预算、取消与损坏输入永不伪造 Complete；`page.has_more` 与 `execution` 独立，停止且未发布新项时可能 `has_more=true` 且无 cursor（调用方需重试同一请求）；`Type`-only 请求会读 `BootstrapMethods` 并对每个 unit 解两遍方法体（属 P5 债务）。

## 平台、adapter 与运行边界

| 维度 | 环境/入口 | 状态 | 边界 |
| --- | --- | --- | --- |
| 平台 | Linux x86_64 | Validated（CI） | `ubuntu-24.04` 上 stable、MSRV 1.88.0 与 supply-chain job 已通过；证据见 verification 中的 run `35168810088`。 |
| 平台 | Linux aarch64 | Supported（当前实际本地证据） | Fedora-like，kernel `7.1.0-rc3-gaokun3+`；证据版本见 verification。 |
| 平台 | 32-bit | NotValidated / Unsupported | noak `lookupswitch` 巨大 `npairs` 等 `usize` 风险未建立支持；P0 限 64-bit。 |
| Library adapter | `Engine` + `Budget` | Supported | 同步 API；含普通枚举、显式 `enumerate_artifact_tree`、标准 MR 选择与 P1 `query`（`PhysicalScope::SnapshotAll`/`ArtifactTree`）；`CancellationToken` 可由调用方注入，取消为协作式。 |
| JSON CLI | stdin / `--request FILE` | Supported | 单请求、1 MiB 控制面；接受十八项 limit schema（P0/P1 十一项 + 六个 P2 计数维度 + `dependency_depth`，全部必填、无静默默认值）；暴露 `query`（relation、target、`PhysicalScope` 含 `artifact_tree`、consumers、`max_items`、cursor）并回显它解析出的 snapshot，错误码与库一致；`enumerate` 仍只枚举顶层容器；不暴露 cancellation token 注入。 |
| 测试门禁 | `fuzz/` 独立 workspace + CI `fuzz-smoke` | Validated（test-only、有界） | cargo-fuzz 0.13.2、libfuzzer-sys `=0.4.13`、nightly-2026-07-20；两个 target（查询与 MR/artifact-tree）单 worker、`-max_len=65536`、`-rss_limit_mb=512`，本地收口 60 秒、CI 20 秒。只证明有界 smoke 不 panic、不越公开预算、损坏输入不假 Complete，不是安全或覆盖率证明；该 workspace 的依赖不进入生产树。 |
| 未来宿主 adapter | reverse-engine/MCP/backend | NotImplemented | 应由独立 adapter 单向依赖 `jarde`；核心不依赖宿主协议。 |
| 生产运行 | 离线、无 JVM | Supported | 不执行目标代码，不启动 JVM/反编译器，不联网。 |
| 测试 oracle | JDK 25 Class-File API | Validated（test-only、显式 ignored） | 只交叉检查 instruction boundaries；固定 runtime 25 与 fixture hash/scope，不证明 verification 或生产 JVM 依赖。 |

请求暴露十八项 limit：input/archive-entry/entry/read/class/attribute/code/result/output、P2 的 class-header/method-body/IR item/IR edge/analysis-step/normalization-clone、非累加高水位 nested-depth 与 dependency-depth、以及 elapsed。其中 P0/P1 入口只使用前九项和 `nested_depth`；P2 的六个计费维度与 `dependency_depth` 由 1.3 加入 limits/usage 与计费入口，在 2.x–5.x 的闭包、CFG、Frame/SSA 与规范化阶段接通后开始真正计费，因此现在它们在任何 P1 请求里保持零值，且 **P2 尚未验证实际派生膨胀的停止行为**（归 3.5/4.3）。这些限制不构成进程 RSS、硬 deadline、恶意并发写入下完整文件瞬时一致性或任意规模 ZIP 的保证。
