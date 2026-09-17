# 当前五维支持矩阵

本页是当前用户可见能力状态的单一事实源。P0 的验收要求以[已生效主规格](../openspec/specs/)和[归档 tasks](../openspec/changes/archive/2026-09-17-establish-p0-foundation/tasks.md)为准；已完成的 P1 增量以 [`p1-query-xref` tasks](../openspec/changes/p1-query-xref/tasks.md) 和 [verification](../openspec/changes/p1-query-xref/verification.md) 为准。这里只描述实际实现，不是未来路线承诺。

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

## 输入与能力五维矩阵

| 输入/边界 | parse | X1 | resolution | decompile-quality | output-level | 说明 |
| --- | --- | --- | --- | --- | --- | --- |
| Standalone CLASS，45.x–51.x 与 52.0，Strict | Supported（version-only Header + 所选方法 instruction inspection） | NotImplemented | NotImplemented | NotImplemented | Header / raw instruction facts | 52 的非零 minor 不属于 Java 8 profile并被 Strict 拒绝；不检查完整 CP tag、flags、attribute 合法位置/基数、opcode dialect 或 JVM verification。 |
| JAR/WAR 顶层物理 entries | Supported | NotImplemented | NotImplemented | NotImplemented | Physical entries；选中 entry 的 Header/raw instructions | 保留 ordinal、raw name、压缩/布局/来源；entry inspection 必须携带枚举所得完整 `PhysicalEntry`。 |
| ZIP64 顶层目录 | Supported | NotImplemented | NotImplemented | NotImplemented | Physical entries | 已覆盖小型真实 ZIP64 结构；不暗示超预算巨型归档可完成。 |
| STORED / DEFLATED entry | Supported | NotImplemented | NotImplemented | NotImplemented | Bounded materialized bytes / inspection facts | 校验 size/CRC；压缩输入计 `read_bytes`，展开数据计 `entry_bytes`。 |
| Nested JAR/WAR candidate（普通 `enumerate`） | MetadataOnly | NotImplemented | NotImplemented | NotImplemented | `candidate_not_scanned` | 普通物理枚举只标记候选，不递归，也不激活 classpath。 |
| Nested JAR/WAR（显式 `enumerate_artifact_tree`） | Supported | NotImplemented | NotImplemented | NotImplemented | Physical containers / entries / origin / layout evidence | 迭代式有界遍历 STORED/DEFLATED child，逐层复核 nested entry；深度、entry、bytes、elapsed 与取消可返回可靠 Partial/Cancelled 前缀。 |
| Boot/WAR layout evidence | Supported（物理 evidence） | NotImplemented | NotImplemented | NotImplemented | `BOOT-INF` / `WEB-INF` classes/lib layout nodes | 匹配区分大小写，只记录物理 evidence；不执行 launcher，不推断 runtime classpath 或唯一选择。 |
| Multi-release physical variants | Supported（物理枚举） | NotImplemented | NotImplemented | NotImplemented | 每个物理 entry | 不做 manifest 条件或 runtime variant 选择；runtime selection/runtime view 属 P1。 |
| Classfile 45.x–51.x 与 52.0 | Supported（Strict version-only） | NotImplemented | NotImplemented | NotImplemented | Header / raw instruction facts | 45.x 历史 minor 规则保留；52 的非零 minor 不属于 Java 8 profile并被 Strict 拒绝。45.3–52.0 fixture 有证据；不是完整 dialect validation。 |
| Classfile 53–71 | StructuralProbeOnly（Forensic）/ Unsupported（Strict） | NotImplemented | NotImplemented | NotImplemented | Forensic Header structure | Forensic 仅读取边界可靠结构并标记 structural probe；方法 decoder 的 Java 8 cursor 不构成这些版本的 dialect 支持。 |
| Preview（modern minor 65535） | UnsupportedPreview（Forensic）/ Unsupported（Strict） | NotImplemented | NotImplemented | NotImplemented | Forensic Header structure + diagnostic | Forensic 可读取边界可靠结构但不支持 preview dialect；45.x 的历史 minor 65535 不等同 modern preview。 |
| Future major（>71） | FutureRelease（Forensic）/ Unsupported（Strict） | NotImplemented | NotImplemented | NotImplemented | Forensic Header structure + diagnostic | Forensic 可读取边界可靠结构但不把未知 future release 宣称为 dialect 支持。 |
| 合法长度的 unknown attribute | Supported（shell/span） | NotImplemented | NotImplemented | NotImplemented | 名称、完整 span/content span | 内容不解释；Header 成功不证明未知内容或方法 Body 合法。 |
| Illegal/truncated instruction | PartialOnError | NotImplemented | NotImplemented | NotImplemented | 可靠指令前缀、stop BCI/class offset、diagnostic | 首错停止，不在错误后猜测同步；CLI 仍可返回 `status: ok` 的 Partial report。 |
| JAR signature metadata | MetadataOnly | NotImplemented | NotImplemented | NotImplemented | metadata kind + `not_verified` | 不验证签名，不推断 trusted。 |

### 明确未实现的分析面

- **X1：全部 NotImplemented。** 不扫描常量池 consumer、metadata/resource/bootstrap 引用，也不构建 XRef。
- **resolution：全部 NotImplemented。** 不加载依赖、不绑定 loader、不做继承/派发或 runtime selection。
- **decompile-quality：全部 NotImplemented。** 不构建 CFG、SSA、IR/AST，不恢复 Java。
- **output-level：** 仅物理 container/entry/layout evidence、Header、所选方法原始 instruction/handler facts 及 coverage/execution/diagnostics；没有 Java、伪代码、MR 选择或 runtime view。

## 平台、adapter 与运行边界

| 维度 | 环境/入口 | 状态 | 边界 |
| --- | --- | --- | --- |
| 平台 | Linux x86_64 | Validated（CI） | `ubuntu-24.04` 上 stable、MSRV 1.88.0 与 supply-chain job 已通过；证据见 verification 中的 run `35168810088`。 |
| 平台 | Linux aarch64 | Supported（当前实际本地证据） | Fedora-like，kernel `7.1.0-rc3-gaokun3+`；证据版本见 verification。 |
| 平台 | 32-bit | NotValidated / Unsupported | noak `lookupswitch` 巨大 `npairs` 等 `usize` 风险未建立支持；P0 限 64-bit。 |
| Library adapter | `Engine` + `Budget` | Supported | 同步 API；含普通枚举和显式 `enumerate_artifact_tree`；`CancellationToken` 可由调用方注入，取消为协作式。 |
| JSON CLI | stdin / `--request FILE` | Supported | 单请求、1 MiB 控制面；接受十一项 limit schema，但 1.2 artifact-tree 仍只开放 library API，CLI 不暴露该 operation 或 cancellation token 注入。 |
| 未来宿主 adapter | reverse-engine/MCP/backend | NotImplemented | 应由独立 adapter 单向依赖 `jarde`；核心不依赖宿主协议。 |
| 生产运行 | 离线、无 JVM | Supported | 不执行目标代码，不启动 JVM/反编译器，不联网。 |
| 测试 oracle | JDK 25 Class-File API | Validated（test-only、显式 ignored） | 只交叉检查 instruction boundaries；固定 runtime 25 与 fixture hash/scope，不证明 verification 或生产 JVM 依赖。 |

请求暴露十一项 limit：input/archive-entry/entry/read/class/attribute/code/result/output、非累加的 nested-depth 高水位和 elapsed；它们不构成进程 RSS、硬 deadline、恶意并发写入下完整文件瞬时一致性或任意规模 ZIP 的保证。
