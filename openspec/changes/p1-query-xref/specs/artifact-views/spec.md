## Purpose

为嵌套归档、MR-JAR 与 Boot 布局建立可复核的物理和运行时选择模型，避免把文件路径启发式或单一 classpath 视为 JVM 的完整加载规则。

## ADDED Requirements

### Requirement: Physical and runtime views are separate

系统 SHALL 先保留选定范围内全部物理定义和 entry，再根据显式 RuntimeProfile、LoadDomain、版本和布局规则派生 RuntimeView。物理视图不得因运行时选择而丢失证据。

#### Scenario: Multi-release variants

- **WHEN** 同一归档含 root、`META-INF/versions/11` 和 `META-INF/versions/17` 变体，并由 root Manifest 主段的 `Multi-Release: true` 激活标准 MR 规则
- **THEN** 物理结果保留三个真实 entry；独立的 selection report 在 Java 8、11、17 请求下分别以 root、11、17 为确定选择事实，并为其余物理变体给出未选原因（验收 A06）

#### Scenario: Java 8, 11 and 17 A06 fixture

- **WHEN** 一个真实 ZIP fixture 含 public `p/A.class` root（major 52）、versions/11（major 55）、versions/17（major 61）和 active Manifest，三个 class 保持同一 `this_class`
- **THEN** Physical evidence 同时保留三个 ordinal，Java 8/11/17 的 Enabled reports 分别 Selected root/11/17；移除 v17 后 Java 17 选择 v11，移除 Manifest 后 Java 17 选择唯一 base 且版本 entries 为 Inactive

#### Scenario: RuntimeView is a request, not a selection fact

- **WHEN** 调用方只构造包含 Java release、MR policy、layout 和 LoadDomain 的 RuntimeView
- **THEN** 该值只标识请求条件；在 provider 读取 Manifest、完整枚举对应 container 并生成 selection report 之前，任何 entry 都不得被声称已选中

#### Scenario: Unsupported multi-release policy preserves evidence

- **WHEN** RuntimeProfile 的 multi-release policy 为 Custom 或 Unknown
- **THEN** provider 仍返回已取得的物理 candidates，但 selection 为 Unknown，execution 以对应稳定 Unsupported code Failed；不得以空结果、base fallback 或任意 custom guess 冒充支持

#### Scenario: Physical scope controls nested MR inspection

- **WHEN** root 含 nested JAR 且调用方分别请求 SnapshotAll 与 ArtifactTree scope
- **THEN** SnapshotAll 只产生 root container 的 MR report，nested entry 仍只是物理 resource/candidate；ArtifactTree 只对 bounded provider 已成功建立的 child 分别解析其自身 Manifest，不为失败或未扫描 child 伪造 report

### Requirement: Standard multi-release selection is evidence-backed

系统 SHALL 以 container 为边界生成 MR selection report，并同时返回请求 RuntimeView、Manifest evidence、全部物理候选、每个候选的 logical path/variant/selection decision/有限合规状态、coverage、execution 和 diagnostics。`SnapshotAll` scope 只检查 root container；`ArtifactTree` scope 才调用显式 bounded tree provider 并分别检查已建立 child。选择只覆盖标准 JAR entry overlay，不跨 container、LoadRoot 或 loader 做唯一 definition resolution。

标准 `Enabled` policy 表示请求标准 MR 处理，不覆盖物理 Manifest：只有一个可可靠解析的 root Manifest，其主段含 `Multi-Release` 属性且展开后的值以 ASCII 大小写不敏感方式等于 `true` 时才激活版本选择；named section 中的同名属性不激活 MR。唯一、可解析但未激活的归档按普通 JAR 选择 base；`Disabled` 总是 base-only。`Custom` 和 `Unknown` 在本阶段 MUST 保留物理候选、将所有 selection decision 标为 Unknown，并分别以 `multi_release_custom_policy` / `multi_release_unknown_policy` 的 Unsupported reason 返回 Failed execution，不得产生 selected fact。

Manifest candidate 的 raw path 按 JAR/JDK 行为与 `META-INF/MANIFEST.MF` 做 ASCII 大小写不敏感比较，原始大小写仍保留；非 canonical case 产生 `multi_release_manifest_noncanonical_path` diagnostic，但单一、可解析的 case variant 仍可作为激活 evidence。多个大小写等价 candidate 为 `Ambiguous`；单一 candidate 中重复 `Multi-Release` 主属性或语法错误为 `Malformed`；entry 读取未完成为 `Unknown`；没有 candidate 为 `Missing`；单一可靠 candidate 的值为 true/其他或属性缺失时分别为 `Active`/`Inactive`。Manifest parser SHALL 在 entry/read/elapsed/cancel 预算内线性解析主段的 CRLF/LF/CR 行、single-space continuation 与 `name: value`，额外内存不得超过已物化 Manifest 大小且不使用 class `attribute_bytes` 预算；属性名 ASCII 大小写不敏感，展开后的值除 continuation 语法外不 trim，因此 `TRUE` 激活而 `true ` 不激活。`Multi-Release` 主属性名重复（含不同大小写）、orphan continuation、NUL、缺少分隔符或未终止 header 均为 Malformed，selection 为 Unknown；实现不得采用 ZIP 顺序或 last-wins。未终止 header 包含主段未以空行结束（含 0 字节 Manifest）的情形，此时不得按已读属性猜测激活状态。未知属性和 individual sections 不需要解释，签名验证、完整 UTF-8/72-byte line 合规不属于本任务。

合法 versioned candidate 只允许精确 raw prefix `META-INF/versions/<N>/`，其中 `<N>` 满足 `[1-9][0-9]*`、可无损表示且 `N >= 9`，logical path 是去掉该前缀后的非空 raw bytes；不得进行 `.`、`..`、重复分隔符或反斜杠规范化。Java release 小于 9 或 MR 未激活时只选择 base；激活后对每个 logical path 选择唯一的最高 `N <= target` candidate，否则回退唯一 base。只有 versioned candidate 而无 base 时，标准 resource 或 non-public class 仍可被选择；`META-INF` 下 resource 不得通过 versioned path 选择。

同一 container、同一 logical path、同一 base/version 出现多个物理 entry 时 SHALL 保留所有 ordinal 并把实际 winning level 标为 Ambiguous，不得 first/last-wins；较高唯一 winning candidate 不因被其遮蔽的低版本重复而失去选择事实。合规诊断和 entry selection SHALL 分离：classfile 版本过高、可检测的 public class predecessor 问题或其他归档违规不得删除物理 candidate，也不得把“被 JAR entry overlay 选中”误写成“该类可加载、可链接或 API 兼容”。完整扫描后得到的 Inactive、Ambiguous、NonConformant 或 Manifest-Unknown 是可信 domain result，本身不使 execution/coverage 变成 Partial；只有工作因预算、取消、unsupported I/O/compression 或未知物理 suffix 未完成时才报告非 Complete execution。

公共 library API SHALL 为 `Engine::select_multi_release(snapshot, runtime_view, budget) -> Result<MultiReleaseViewReport>`。入口先验证 RuntimeView 的 snapshot 与 scope；`SnapshotAll` 只重新枚举 root，`ArtifactTree` 只接受该 snapshot 的真实 root container 并复用显式 tree provider。非 ZIP、view snapshot 不匹配和 tree root 不匹配分别返回 `multi_release_not_zip`、`multi_release_snapshot_mismatch`、`multi_release_root_container_mismatch` 的 InvalidInput；root ZIP open 的既有结构错误 code 原样保留。P1 1.3 不增加 CLI operation，CLI 暴露留给 3.2。

公共 Rust/JSON schema 固定如下；本节新增的 MR-owned struct 使用 `deny_unknown_fields`，带 payload 的 MR-owned enum 使用内部 `kind` tag 与 snake_case variant，unit enum 使用 snake_case string；复用的 P0/P1 类型保持其既有 serde 契约。后续增加 variant 需要新的 OpenSpec 修订，当前实现不得返回开放字符串 reason：

- `MultiReleaseViewReport { view: RuntimeView, physical: MultiReleasePhysicalEvidence, containers: Vec<MultiReleaseContainerReport>, coverage: Coverage, execution: ExecutionReport, diagnostics: Vec<MultiReleaseReportDiagnostic>, verification: VerificationStatus }`；`verification` MUST 为既有 `VerificationStatus::NotPerformed`；
- `MultiReleaseReportDiagnostic = Domain { diagnostic: MultiReleaseDiagnostic } | Terminal { diagnostic: Diagnostic }`；`MultiReleaseDiagnostic { code: MultiReleaseDiagnosticCode, severity: DiagnosticSeverity, message: String, provenance: Option<Provenance> }`。Domain code 使用下文闭合 enum 并在反序列化时拒绝未知值；Terminal 只包装本次 probe 从既有 `Error` 生成的 P0 `Diagnostic`，有意保留 P0 开放 String code 兼容性，生产端不得把 MR domain violation 塞进 Terminal；
- `MultiReleasePhysicalEvidence = Snapshot { report: EnumerationReport } | ArtifactTree { report: ArtifactTreeReport }`，原样返回本次 API 内部实际执行一次的 provider 结果，作为所有 entry 引用的唯一物理事实源；不得接受调用方提供的旧 report；
- `MultiReleaseContainerReport { origin: ContainerOrigin, manifest: ManifestEvidence, entries: Vec<MultiReleaseEntryEvidence>, selections: Vec<MultiReleaseSelection>, coverage: Coverage, execution: ExecutionReport }`；origin MUST 等于其 physical report 中真实 container origin，root/child 不得重建或缩写；
- `ManifestEvidence { entries: Vec<PhysicalEntryId>, state: ManifestState, attribute_name: Option<ArchiveNameBytes>, attribute_value: Option<ArchiveNameBytes> }`；entries 按 central ordinal 包含该 container 中全部 ASCII case-equivalent Manifest candidates。`ManifestState = Missing | Active | Inactive | Ambiguous | Malformed | Unknown`。只有单一、语法可靠的主属性可填 name/value；Missing 无 entries，duplicate/Malformed/Unknown 的 attribute fields 为 None；
- `MultiReleaseEntryEvidence { entry: PhysicalEntryId, logical_path: Option<ArchiveNameBytes>, variant: MultiReleaseEntryVariant, decision: MultiReleaseSelectionDecision, compliance: MultiReleaseCompliance, class_evidence: Option<MultiReleaseClassEvidence> }`；Base 的 logical_path 是完整 raw name，valid Versioned 是剥离 prefix/release 后的 raw tail，Directory 与 InvalidVersioned 固定为 None。`MultiReleaseClassEvidence { major_version: u16, minor_version: u16, access_flags: u16, this_class: JvmString }` 只在有限 class probe 成功时返回；
- `MultiReleaseEntryVariant = Base | Versioned { release: u64 } | InvalidVersioned { issue: MultiReleaseVersionPathIssue }`；`MultiReleaseVersionPathIssue = EmptyRelease | NonDecimalRelease | LeadingZeroRelease | ReleaseBelowNine | ReleaseOverflow | EmptyLogicalPath`。Directory 唯一判据是 raw name 非空且以 byte `/` 结尾；忽略 ZIP external attributes/DOS bits，零字节但不以 `/` 结尾仍是 resource。Classifier precedence 为 Directory → exact、大小写敏感的 `META-INF/versions/` → Base。命中 prefix 且非 Directory 后，按首个 `/` 拆 `<N>`/logical path：无第二个 `/` 时把 remainder 当 `<N>`、logical path 为空；依次判 EmptyRelease、NonDecimal、LeadingZero、u64 Overflow、BelowNine、EmptyLogicalPath，首个 issue 胜出；其他 case variant 是普通 Base raw path；
- `MultiReleaseSelection { logical_path: ArchiveNameBytes, outcome: MultiReleaseSelectionOutcome }`；每个合法、非目录、非 versioned-META-INF logical group 恰有一个 selection，按 group 最小 ordinal 排序；`MultiReleaseSelectionOutcome = Selected { entry: PhysicalEntryId } | Ambiguous { entries: Vec<PhysicalEntryId> } | NoSelection { reason: MultiReleaseNoSelectionReason } | Unknown { reason: MultiReleaseUnknownReason }`；
- `MultiReleaseSelectionDecision = Selected | Shadowed { winners: Vec<PhysicalEntryId> } | Inactive { reason: MultiReleaseInactiveReason } | Ambiguous | Unknown { reason: MultiReleaseUnknownReason } | NotApplicable { reason: MultiReleaseNotApplicableReason }`；
- `MultiReleaseInactiveReason = PolicyDisabled | ManifestMissing | ManifestInactive | TargetBelowNine | ReleaseAboveTarget`；
- `MultiReleaseNoSelectionReason = NoBaseWhileInactive | NoApplicableRelease`；
- `MultiReleaseUnknownReason = PhysicalEvidenceIncomplete | ManifestEvidenceAmbiguous | ManifestEvidenceMalformed | ManifestEvidenceUnreadable | CustomPolicy | UnknownPolicy | ProcessingInterrupted`；
- `MultiReleaseNotApplicableReason = InvalidVersionPath | VersionedMetaInfResource | DirectoryEntry`；
- `MultiReleaseCompliance = NotApplicable | ConformantWithinChecks | NonConformant | Unknown { reason: MultiReleaseComplianceUnknownReason }`；`MultiReleaseComplianceUnknownReason = PhysicalEvidenceIncomplete | ProbeInterrupted | PredecessorAmbiguous | ModuleExportsNotInspected | PolicyUnsupported`。

Version-path issue 每个 entry 只产生一条对应 Domain diagnostic：

| `MultiReleaseVersionPathIssue` | `MultiReleaseDiagnosticCode` |
| --- | --- |
| EmptyRelease | `multi_release_version_path_invalid` |
| NonDecimalRelease | `multi_release_version_path_invalid` |
| LeadingZeroRelease | `multi_release_version_path_invalid` |
| ReleaseBelowNine | `multi_release_version_below_nine` |
| ReleaseOverflow | `multi_release_version_overflow` |
| EmptyLogicalPath | `multi_release_version_path_invalid` |

因此 `META-INF/versions/9/` 是 Directory，`META-INF/versions/9` 是 InvalidVersioned/EmptyLogicalPath，`META-INF/versions//A.class` 是 EmptyRelease，`META-INF/versions/x/A.class` 是 NonDecimalRelease；specific below-nine/overflow 不再额外产生 generic invalid diagnostic。

在 MR item 预算允许时，physical report 中每个已返回 entry（包括 Directory、Manifest、InvalidVersioned 和普通 resource）恰有一个 `MultiReleaseEntryEvidence`；Directory 仍计 entry evidence item并进入 scanned ordinal coverage，但 decision 为 `NotApplicable(DirectoryEntry)`、logical_path 为 None，不创建 selection。所有 entry/winner 引用 MUST 使用完整 `PhysicalEntryId`，并且必须能在内嵌 physical report 中按完整 origin + ordinal + raw name 找到，不能只用 path/version。`Selected` outcome 与 candidate decision 必须一致；winning level 重复时 outcome 为 Ambiguous、重复 winners 为 Ambiguous、低层 candidates 以这些 winners 标为 Shadowed，不得回退。`NoSelection` 只允许完整 evidence 下确实没有 base 且 MR 未激活/没有适用 candidate；directory、invalid-version 和 versioned-META-INF entries 不形成 logical selection。不完整 evidence 必须用 Unknown。

Decision precedence 固定为：Directory/InvalidVersioned/versioned-META-INF → NotApplicable；Custom/Unknown policy → Unknown 对应 policy reason；physical/Manifest 工作中断 → Unknown；完整 Manifest Ambiguous/Malformed → Unknown；Disabled → versioned PolicyDisabled；Enabled + Missing/Inactive → versioned ManifestMissing/ManifestInactive；Active + target `< 9` → versioned TargetBelowNine；Active + version `> target` → ReleaseAboveTarget；其余按最高可用 level 产生 Selected/Shadowed/Ambiguous。Base candidates 仅在 winning level duplicate 时 Ambiguous，否则在无适用 versioned winner 时 Selected、存在 winner 时 Shadowed。仅有 inactive versioned candidates时 outcome 为 `NoSelection(NoBaseWhileInactive)`；Active 且只有 target 以上版本、没有 Base 时为 `NoSelection(NoApplicableRelease)`。`java_release` 的所有 `0..=8` 值均按 target-below-nine 处理，不另报 invalid profile。

Custom/Unknown policy 在 physical provider 后停止标准 Manifest/class probe：Manifest candidates 只记录 entries，state 为 Unknown、attribute fields 为 None；合法 entries 的 decision 为对应 policy Unknown、compliance 为 `Unknown(PolicyUnsupported)`，已由 raw path 直接证明的 InvalidVersioned/versioned-META-INF/duplicate 仍按闭合规则诊断。Aggregate 为 Failed/Unsupported，但 physical evidence 可保持 Complete。

Runtime-resolution coverage 使用真实 ordinal 的单 entry ranges：`container:<id>:multi_release_selection_entries` 记录 path/Manifest decision evidence，`container:<id>:multi_release_compliance_entries` 记录 bounded Header checks；不存在用 path hash 或全局计数代替 ordinal 的范围。Artifact structural coverage 原样聚合。Result/resource accounting 固定为：

| 对象/动作 | `result_items` | archive/read/entry/class | `output_bytes` |
| --- | --- | --- | --- |
| 内嵌 physical report 的 entry/container/layout/physical diagnostic | 只使用 provider 已发生的原计费，MR 层不重复 | provider 原语义 | provider 原语义 |
| `MultiReleaseViewReport` envelope、coverage/execution/verification | 0，属于控制 envelope | 0 | 0 |
| 每个实际返回的 MR container report | 1 | 0 | 0 |
| 每个实际返回的 ManifestEvidence | 1；与 physical Manifest entry 分开 | Manifest replay 逐层计 archive/read/entry，parser 按行 poll elapsed/cancel | 0 |
| 每个实际返回的 MultiReleaseEntryEvidence | 1 | 分类本身 0 | 0 |
| 每个实际返回的 MultiReleaseSelection | 1 | 0 | 0 |
| 每个 Domain diagnostic | 1；P0 `duplicate_raw_name` 保留在 physical，MR `multi_release_duplicate_candidate` 是不同语义并单独计费 | 0 | 0 |
| class compliance probe | 不创建临时 Header report item；成功 facts 嵌入已计费的 entry evidence，失败只为相应 diagnostic 计费 | root/nested replay 计 archive/read/entry；crate-private minimal probe 计 class bytes并 poll，不遍历/计未请求 attribute/code facts | 0 |
| Terminal diagnostic（Budget/Cancelled/I/O/unsupported/structural） | 0，延续控制元数据例外 | 已发生用量保留 | 0 |

MR API 内部只执行一次所选 physical provider，不接受外部 report。Containers 按 physical provider 顺序，entry evidence 按 central ordinal，logical selections 按 group 最小 ordinal返回。每个 container 在写入前原子预留 container + ManifestEvidence 两项；不足则不返回半个 container，并把其已知 ordinal 范围标 skipped。后续 entry/selection/ordinary diagnostic 分别在 append 前计费，耗尽时保留已返回可靠前缀和 physical evidence，不构造未计费 item。

Root ZIP 或 RuntimeView identity 无法建立时入口返回 `Err`；root 已建立后返回 report。Aggregate 终止优先级固定为 Cancelled、实际阻止继续处理的 budget/physical provider error、unsupported policy、局部 compliance error；path selection 已证明后发生的 compliance budget/cancel 保留 selection，只将 compliance 与 aggregate execution 标为 Unknown/non-Complete。

#### Scenario: Manifest does not activate MR processing

- **WHEN** versioned paths 存在，但 root Manifest 缺失、`Multi-Release` 值不是 `true`，或 policy 为 Disabled
- **THEN** 完整 container 的唯一 base entry 仍可确定 selected，所有 versioned entries 保留为物理 evidence 并标明 inactive；不得仅因 `META-INF/versions` 存在而激活 MR

#### Scenario: Manifest evidence is ambiguous or malformed

- **WHEN** 同一 container 有多个大小写等价的 Manifest entry、主段重复 `Multi-Release`，或 Manifest 语法/continuation 无法可靠解析
- **THEN** report 保留 Manifest 和版本候选，selection decision 为 Unknown 并给出 provenance diagnostic；不得按 entry 顺序或最后属性值猜测激活状态

#### Scenario: Invalid versioned paths remain physical evidence

- **WHEN** entry 使用低于 9、带前导零、非十进制、溢出或空 logical path 的 `META-INF/versions/<N>/...`
- **THEN** entry 保留真实 raw name 与 ordinal 并产生结构化 diagnostic，但不参与标准 MR candidate group；未受该歧义影响且已完整枚举的 logical path 仍可形成确定选择

#### Scenario: Duplicate winning candidates are ambiguous

- **WHEN** 某 logical path 在目标 Java 实际 winning 的 base 或同一版本层存在多个物理 entry
- **THEN** 所有重复项保留 origin/ordinal，selection 为 Ambiguous 且无 selected fact；不同版本层之间的同 logical path 是正常 variant，不作为 duplicate

#### Scenario: Partial physical evidence cannot prove fallback

- **WHEN** container 枚举或 Manifest evidence 因预算、取消或结构失败而未完成
- **THEN** 该 container 中所有已见 logical paths 的 selection 为 Unknown，物理前缀保留，runtime-resolution coverage 明确 scanned/skipped ordinal，整体不得标为 Complete；“尚未看到更高版本或 duplicate”不得被当作 base fallback 或 NoDefinition

Selection proof SHALL 使用以下确定规则，而非猜测某个 path 是否已“看完”：

| Container / Manifest evidence | 已见 logical group | 未见 logical group | selection coverage / execution |
| --- | --- | --- | --- |
| central directory 未完整枚举，无论失败 ordinal 在哪里 | 每个已见 group 恰有一个 `Unknown(PhysicalEvidenceIncomplete)`；已有 candidates 的 decision 同为 Unknown | 不创建 selection，不推断 NoSelection | selection scanned 为已分类 ordinals，physical skipped ranges 原样进入 runtime skipped；container/aggregate 非 Complete |
| physical complete，但 Manifest candidate 读取因 budget/cancel/I/O/unsupported 未完成 | 每个 group 为 `Unknown(ManifestEvidenceUnreadable)` | 不存在未见 group | Manifest ordinal 记 runtime skipped；container/aggregate 非 Complete |
| physical complete，Manifest 为完整的 Ambiguous 或 Malformed domain evidence | 每个 group 分别为 `Unknown(ManifestEvidenceAmbiguous|ManifestEvidenceMalformed)` | 不适用 | runtime selection coverage CompleteWithinSchema，execution 可 Complete，因为不是工作中断 |
| physical + Manifest 完整且 policy 可执行 | 对每个合法 group计算 Selected/Ambiguous/NoSelection | 不创建不存在的 group | selection coverage CompleteWithinSchema；compliance 可以独立 Partial |

Central-directory ordinal 没有按 logical path 分区，因此任何未知 suffix 都可能补入同 path 的更高 release 或 winning-level duplicate；即使失败发生在该 group 最后一个已见 entry 之后，也不得局部宣称 Selected。`NoSelection` 只可在 physical container 完整结束后产生。一个 container 内 path-selection proof 完整而后续 compliance probes 部分完成时，所有 selection 保持确定；已 probe candidates 可有确定 compliance，未 probe candidates 为 Unknown，container runtime-resolution coverage/aggregate execution 为 Partial 或 Cancelled。

`multi_release_compliance_entries` 只覆盖 probe 适用范围：probe 实际跑完（含确定为违规、确定为合规，或确定为 Unknown 如 `PredecessorAmbiguous`/`ModuleExportsNotInspected`）的 ordinal 记 scanned；probe 被尝试但被预算、取消或读取错误中断的 ordinal（包括作为 public predecessor 的 Base）与适用但未跑的 ordinal 记 skipped。`multi_release_selection_entries` 记录 path/Manifest decision 证据：该 ordinal 的 entry evidence 已返回即记 scanned（包括物理枚举不完整、Manifest 因此从未被读取的情形）；Manifest 读取被真正尝试但中断时，该 Manifest ordinal 记 skipped。

#### Scenario: Compliance probe failure does not rewrite path selection

- **WHEN** container 和 Manifest 已完整证明唯一 winning entry，但后续 class entry 物化/Header 合规 probe 因预算、取消或不可继续的读取错误未完成
- **THEN** 已证明的 entry selection 保留，candidate compliance 标为 Unknown，execution/coverage 为对应 Partial、Cancelled 或 Failed，并保留 terminal diagnostic；不得把合规未知改写成较低版本 fallback

#### Scenario: Malformed class is a completed nonconformance result

- **WHEN** class entry 字节已被完整、校验地物化，但 bounded Header parser 证明其结构无法形成所需 version/identity/access evidence
- **THEN** 该 candidate 为 NonConformant 并带 class diagnostic；其他 candidates 继续检查，单凭该输入违规不把完整 report 降为 Partial，也不改变 path selection

### Requirement: Multi-release compliance diagnostics are bounded and non-selecting

系统 SHALL 对本阶段可由物理路径、Manifest 和 bounded class Header 证明的 MR 违规给出 diagnostic：非法版本目录、versioned `META-INF` resource、versioned classfile major 高于以宽整数计算的 `N + 44`、可直接观察的 `ACC_PUBLIC` class 缺少 root 下同 logical path、同 `this_class` 且 `ACC_PUBLIC` 的 predecessor，以及重复 Manifest/candidate。每个 candidate 的有限合规状态 SHALL 明确为 NotApplicable、ConformantWithinChecks、NonConformant 或 Unknown。Selection 规则仍按 entry overlay 独立计算；diagnostic 不证明完整 JVM 合法性，也不自动令较低 candidate 取代规范会先查到的较高 entry。

`MultiReleaseDiagnosticCode` 是闭合 unit enum，其 JSON string 与下列字面值完全一致：Warning 只有 `multi_release_manifest_noncanonical_path`；Error 为 `multi_release_manifest_duplicate`、`multi_release_manifest_malformed`、`multi_release_version_path_invalid`、`multi_release_version_below_nine`、`multi_release_version_overflow`、`multi_release_meta_inf_resource`、`multi_release_duplicate_candidate`、`multi_release_class_malformed`、`multi_release_class_version_too_new`、`multi_release_public_predecessor_missing`、`multi_release_public_predecessor_mismatch`。Constructor MUST 从 code 派生并验证固定 severity，JSON 中 code/severity 不一致时拒绝反序列化。Budget/Cancelled/I/O/unsupported probe termination 只进入 `MultiReleaseReportDiagnostic::Terminal`，复用既有 P0 code/severity，不改名为 MR-specific code。

`ExecutionReport`/`TerminationReason` 为 P0 共享公共类型，其 String code 保持向前兼容且 JSON 反序列化不因 MR 收窄；MR producer 只允许新增两个 Unsupported 输出 `multi_release_custom_policy`、`multi_release_unknown_policy`，其他 terminal reason 必须直接来自本次 physical/probe 的既有 Error。Manifest inactive/absent、target release 以上 candidate 和普通 NoSelection 不产生 diagnostic；新增 MR-specific domain code 或 producer-owned Unsupported code 需要后续 OpenSpec 修订。

对每个完整返回的 entry evidence，decision 与 compliance 按以下闭合规则组合：

| Entry / evidence | compliance | diagnostic / predecessor action |
| --- | --- | --- |
| directory | NotApplicable | decision `NotApplicable(DirectoryEntry)`，无 diagnostic |
| `InvalidVersioned` | NonConformant | 按 issue 产生 version-path code；不 probe class |
| valid versioned logical path 位于 `META-INF/` | NonConformant | `multi_release_meta_inf_resource`；不参与 selection |
| 同 logical path + level 的 duplicate | NonConformant | 为每个 duplicate entry 产生带其自身 provenance 的 `multi_release_duplicate_candidate`；winning level 为 Ambiguous，shadowed level 不改变 winner |
| Base、Manifest 或普通非-versioned resource，且无已知 duplicate | NotApplicable | 仅作 selection/Manifest/predecessor evidence |
| valid versioned 非-class resource | ConformantWithinChecks | 无 class probe；是否高于 target 只影响 decision |
| valid versioned `.class` 无法完整读取，或 probe 被 budget/cancel/unsupported/I/O 中断 | Unknown | 复用 terminal diagnostic，aggregate 非 Complete；selection 不变 |
| valid versioned `.class` 已完整读取但 minimal Header 结构错误 | NonConformant | `multi_release_class_malformed`；继续 siblings，完整 report 仍可 Complete |
| valid versioned `.class` Header 完整且 major > `u128::from(N) + 44` | NonConformant | `multi_release_class_version_too_new`；仍继续 predecessor probe 以保留可得 evidence |
| valid versioned non-public `.class`，Header/major checks 通过 | ConformantWithinChecks | 不要求 root predecessor |
| valid versioned public `.class` | 见下表 | major 与 predecessor violations 任一已证明即 NonConformant；没有已证明违规但证据中断则 Unknown |

Public predecessor truth table 只使用同 container、同 logical raw path 的 Base level，不搜索低 version、其他 container 或 LoadDomain：

| Base predecessor state | versioned public candidate compliance / diagnostic |
| --- | --- |
| Base missing/malformed/access/name mismatch，但存在 root 或 valid-versioned `module-info.class` evidence | `Unknown(ModuleExportsNotInspected)` / 不发 predecessor violation，因为非导出 package 可适用 modular MR 例外；malformed Base 自身仍发 `multi_release_class_malformed` |
| physical complete、没有 module evidence 且没有 Base candidate | NonConformant / `multi_release_public_predecessor_missing` |
| Base winning level duplicate | `Unknown(PredecessorAmbiguous)`（duplicate Base entries 自身为 NonConformant）/ 不猜测某个 predecessor |
| 唯一 Base 无法读取或 probe 中断 | `Unknown(ProbeInterrupted)` / 原始 terminal diagnostic，aggregate 非 Complete |
| 唯一 Base 已读取但 Header malformed，且没有 module evidence | NonConformant / Base 的 `multi_release_class_malformed` + versioned 的 `multi_release_public_predecessor_mismatch` |
| 唯一 Base Header 非 `ACC_PUBLIC`，且没有 module evidence | NonConformant / `multi_release_public_predecessor_mismatch` |
| 唯一 Base `this_class` 与 versioned 不同，且没有 module evidence | NonConformant / `multi_release_public_predecessor_mismatch` |
| 唯一 Base 为 public、同 `this_class` | 若 versioned major check 通过则 ConformantWithinChecks，否则保持已知 NonConformant；module evidence 不影响已证明的正匹配 |

检查所有 valid versioned `.class`，不因目标 release 较低或 candidate 被 shadow 而跳过；Base 只在充当 public predecessor 时 probe。`.class` 后缀和 `META-INF/versions/` prefix 都按 raw bytes 大小写敏感。Root major/dialect、class logical path 与 `this_class` 的单独一致性不在本有限检查内；只要 minimal Header 能安全取得 access/name facts，future root major 仍可作为 predecessor evidence，不代表其可加载。

ClassFile `access_flags` 没有 `ACC_PROTECTED`；protected nested-class visibility 需要 `InnerClasses` metadata。Module descriptor 只作“可能适用 non-exported package 例外”的物理 gate，本阶段不解析 exports，也不把 missing/mismatch 误报为 definite violation。完整 public API 等价、module descriptor 合规、签名验证、classfile dialect/verifier 与 loader/linkage 成功均不属于本任务。后续 metadata/module consumer 或 resolver 必须保留这些 Unknown，而不能把本阶段 `ConformantWithinChecks` 或 diagnostic absence 当作完整合规证明。

#### Scenario: Selected entry is not a loadability claim

- **WHEN** 目标 Java 选中的 versioned class path 唯一且 Manifest/版本规则成立，但 classfile major 超过该目录 release 或 public class 缺少 root predecessor
- **THEN** report 仍记录该 entry 被标准 path overlay 选中，同时给出不合规 diagnostic 和 `verification: not_performed` 边界；不得把它宣称为可加载、可链接或 API compatible

#### Scenario: Versioned META-INF resource

- **WHEN** 合法版本目录中的 logical path 位于 `META-INF/`
- **THEN** entry 保留为物理 evidence并产生不可版本化 diagnostic，不生成标准 selected fact

#### Scenario: Duplicate definitions in WAR

- **WHEN** WAR 或嵌套依赖中存在多个同名类
- **THEN** 系统保留每个物理 origin，并在 RuntimeView 中保留显式 loader/order 条件；P1 不做唯一解析或静默消歧（验收 A07）

### Requirement: Bounded nested and Boot layout traversal

系统 SHALL 仅在调用方显式请求 artifact-tree scope 时递归 nested JAR；普通 physical snapshot 枚举只保留 candidate，不把标准 JAR 内任意 archive 自动激活为运行时 classpath。Provider SHALL 将 nested JAR、`BOOT-INF/classes`、`BOOT-INF/lib`、WAR 的 `WEB-INF/classes` 和 `WEB-INF/lib` 表示为带 evidence 的物理布局节点，并 SHALL 允许通过 origin chain 从 root snapshot 逐层复核并重读 nested entry。递归展开 MUST 受 `nested_depth` 高水位、archive entry、read/entry bytes、elapsed 和取消预算约束，并且不得自动联网、执行 launcher 或推断唯一运行时选择。

`nested_depth` SHALL 以 root container 为 0、直接 child 为 1；它记录请求达到的最大已接受深度，不作为累加 charge。Root container 建立后的 child malformed、unsupported、budget 或 cancellation MUST 返回可靠前缀、定位到父 entry 的 diagnostic、已扫描/未扫描 coverage 与非 Complete execution；不得返回空 child 冒充成功。

#### Scenario: Nested candidate without explicit tree scope

- **WHEN** 标准 JAR 含任意 nested JAR，但调用方只请求普通 physical snapshot 枚举
- **THEN** nested entry 保持 candidate-not-scanned，系统不递归、不把它加入运行时 classpath

#### Scenario: Nested DEFLATED archive

- **WHEN** 调用方显式请求 artifact-tree，嵌套 JAR 使用 DEFLATED 且展开后不超过预算
- **THEN** 系统物化并扫描其内容，layout node 记录真实父 entry 与 child container，内层 entry 保留完整 origin chain，且后续可从 root snapshot 重读（验收 A08）

#### Scenario: Nested traversal exceeds budget

- **WHEN** 递归展开超过任一 `nested_depth`、archive entry、read/entry bytes、elapsed 或取消预算
- **THEN** 结果声明具体预算维度或 cancellation、已扫描范围和未扫描范围，保留已完成 container/entry 的可靠前缀，且不标为 Complete（验收 A14）

#### Scenario: Malformed child does not erase siblings

- **WHEN** root 已建立且一个 nested child 在物化、ZIP open 或 central-directory 枚举阶段损坏，或使用不支持的压缩，而其他 sibling 可读
- **THEN** 结果保留可读 sibling，整体为 Partial，并对失败父 entry 给出 Error/Unsupported diagnostic；已建立但枚举失败的 child 保留非 Complete container report，失败 child 不以空列表表示成功

#### Scenario: Terminal budget after a local child failure

- **WHEN** 一个 child 已产生可继续的 Error/Unsupported，后续 sibling 又耗尽非深度预算或收到取消
- **THEN** traversal 立即停止，aggregate execution 报告实际阻止继续扫描的预算维度或 Cancelled；此前 child diagnostic 和可靠前缀仍保留

#### Scenario: Nested entry replay is bounded and revalidated

- **WHEN** 调用方用 artifact-tree 返回的 nested entry 从 root snapshot 重读
- **THEN** 系统逐层验证真实父 entry、candidate 状态、派生 child container 和最终完整 metadata，并对每层执行 `nested_depth`、archive entry、read/entry bytes、elapsed 与取消检查；预算或取消错误保持其原始结构，不改写成普通 invalid-input

#### Scenario: Ordinary ZIP bytes cannot forge a child container

- **WHEN** 非 `.jar`/`.war` candidate 的普通 entry 恰好包含合法 ZIP bytes，调用方手工构造 origin chain
- **THEN** nested entry replay 拒绝该 chain，普通物理 entry 不因内容可解析为 ZIP 而成为 artifact-tree child
