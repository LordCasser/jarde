# P1 实施验证记录

## 1.1 Query/view/identity 公共模型

验证基线为 P0 归档 commit `8977dcbc5c932d974abf2df77bf64801b32f7a73` 之上的 P1 1.1 候选工作树。实现范围只包含公共值类型和 P0 物理身份迁移，不包含 nested/Boot provider、MR 选择算法、consumer 扫描、query compiler、分页、CLI 查询、resolver、CFG、SSA、AST 或 IR。

### 已实现契约

- `PhysicalDefinitionId` 与 `ClassSource` 以显式 `StandaloneRoot` / `ArchiveEntry` location 表示 CLASS 来源；standalone 不再伪造 ZIP entry。
- `PhysicalEntryId` 以 snapshot、root container 和 outer→inner 的有向 steps 表示 origin；每一步保留父容器中的 entry ordinal/raw name 与 child container ID。Nested 来源与 base/MR variant 正交。
- 新增五类 `QueryRelation`、带显式版本且以有序集合规范化的 `ConsumerSchema`，以及只表达请求身份的 `PhysicalView` / `RuntimeView` / `RuntimeProfile` / `LoadDomain`。
- LoadDomain 保留 loader/parent identity、delegation、ordered roots、module mode、external override 与 runtime transformation uncertainty。当前不公开可任意构造的 resolved/selected definition 类型。

### 类型与回归测试

以 `CARGO_BUILD_JOBS=1`、`CARGO_INCREMENTAL=0`、`RUST_TEST_THREADS=1` 串行执行：

- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`：通过。
- `PROPTEST_RNG_SEED=5350648285461741569 cargo test --workspace --all-targets --all-features --locked`：通过；library 72、engine 9、historical corpus 1、P1 query model 6、CLI JSON 8，共 **96 passed / 0 failed / 1 ignored**。ignored 项仍是显式 JDK 25 oracle。
- P1 类型测试覆盖：standalone 无 synthetic entry、archive entry round-trip、两层 origin edge 与 duplicate parent ordinal、base/v11/v17 物理 variant、Java 8/11/17 runtime request identity、delegation/root order/uncertainty、MR/layout/module Unknown，以及 consumer schema 的 canonical JSON/Eq/Hash。
- `openspec validate --all --strict --no-interactive`：3 个主规格与 5 个 active changes，共 **8 passed / 0 failed**。
- `git diff --check`：通过；生产源码中不存在本阶段禁止的 CFG/SSA/Java AST/Decompiler 构造标记。

### 审查与资源边界

- 独立只读 review 首轮指出可任意构造的 `ResolvedDefinitionId` 会把请求视图误表述为已选择事实；该类型已从本阶段删除，并补强 standalone JSON 与所有 Unknown policy 的测试。
- 修正后独立复核未发现阻塞问题，确认可勾选 P1 1.1。
- 全程没有并行 Cargo。最终本地记录为约 **3.8 GiB available memory**、`target/` 的 `du` 字节数为 `682,057,299`。
- 实现 commit `8df41f74b9c931cee3d568d69ffd9fcbfb0b919e` 的 [GitHub Actions run `35172590694`](https://github.com/LordCasser/jarde/actions/runs/35172590694) 为 **success**；Linux x86_64 上 stable、MSRV 1.88.0 与 supply-chain 三个 job 全部通过。
- 远端 CI 通过后执行 `cargo clean`，Cargo 报告移除 **1,239 files / 738.0 MiB**；清理前 `target/` 为 `682,057,299` bytes，清理后已不存在。

## 1.2 Bounded artifact-tree / nested / Boot-WAR provider

实现范围严格限于显式物理 artifact-tree、nested entry replay、Boot/WAR layout evidence 和 A08；普通 `enumerate` 仍只枚举当前容器。本任务不实现 MR-JAR 选择、consumer/XRef、query compiler、resolver、CFG/SSA/AST/IR、launcher 或网络解析。

### 已实现契约

- `Engine::enumerate_artifact_tree` 以同步、无缓存、显式入口迭代遍历 `.jar`/`.war` candidates；root 与所有 child 共用一个 immutable snapshot，child identity 由已验证的 parent container、真实 ordinal 和 raw name 派生。
- `ArtifactTreeReport` 分别返回 container reports、Boot/WAR classes/lib 与普通 nested layout evidence、aggregate coverage/execution/diagnostics；layout node 只表示物理 evidence，不声称 classpath、MR 或 runtime selection。
- STORED/DEFLATED child 都经过 central/local header、size、CRC 和压缩方法验证；nested `read_entry` 从 root 逐层复核 candidate、origin、child container 和最终完整 metadata。中间物化计 `read_bytes`/`entry_bytes`，只有最终 caller output 计 `output_bytes`。
- `nested_depth` 作为 Limits/UsageSnapshot/termination 中的非累加高水位：root 为 0，直接 child 为 1。Depth、entry/bytes、elapsed、ResultItems 和取消中断都保留可靠前缀、真实 parent provenance，以及按 container/entry ordinal 区分的 scanned/skipped coverage。
- Child malformed、unsupported 或 central-directory 失败不会擦除可读 sibling；已建立但枚举失败的 child 保留 Failed container report，aggregate 为 Partial。取消和实际终止扫描的非深度预算优先于更早的局部错误。

### A08、预算和对抗回归

`tests/p1_artifact_tree.rs` 使用 `rawzip` writer 与 `flate2` 动态生成真实 DEFLATED nested fixture，共 **14 tests**，覆盖：

- Boot `BOOT-INF/classes` / `BOOT-INF/lib/*.jar`、WAR `WEB-INF/classes` / `WEB-INF/lib/*.jar` 和普通 nested evidence；大小写、直接路径与 `.jar` 后缀反例不冒充 layout library。
- DEFLATED child 扫描、inner entry replay、同名同 bytes 不同 ordinal 的 child identity 隔离，以及伪造 child ID、parent ordinal、普通 ZIP payload origin 和 entry metadata 的拒绝。
- `nested_depth` 1/2 高水位、EntryBytes/ArchiveEntries/ResultItems、预取消和中途取消；nested replay 不制造临时 ResultItems，结构化 BudgetExceeded/Cancelled 不被改写。
- Malformed/unsupported child、child central/local mismatch 与有效 sibling 并存；terminal reason priority、parent-entry provenance，以及 root/current/pending container 和每个 candidate 真实 ordinal 的 scanned/skipped coverage。

### 本地验证与审查

所有 Cargo 命令均设置 `CARGO_BUILD_JOBS=1`，交付检查另设置 `CARGO_INCREMENTAL=0`、`RUST_TEST_THREADS=1`，全程没有并行 Cargo：

- `cargo fmt --all -- --check`：通过。
- `cargo test --test p1_artifact_tree --locked`：**14 passed**。
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`：通过。
- `PROPTEST_RNG_SEED=5350648285461741569 cargo test --workspace --all-targets --all-features --locked`：通过。
- `PROPTEST_RNG_SEED=5350648285461741570 cargo test --workspace --all-targets --all-features --locked`：通过；最终候选合计 **113 passed / 0 failed / 1 ignored**，ignored 项仍为显式 JDK 25 oracle。
- `openspec validate --all --strict --no-interactive`：**8 passed / 0 failed**；`git diff --check`：通过。
- 独立只读 review 首轮发现 Boot/WAR 大小写、provider authorization、root/child failure、ResultItems 与 replay budget 等边界；修正后又针对 pending container 和 per-candidate coverage 做两轮反例复核。最终独立复核结论为 **Approve P1 1.2**。
- 最终本地交付检查记录约 **9.3 GiB available memory**、文件系统约 **103 GiB available**，清理前 `target/` 的 `du` 字节数为 `1,606,243,938`。
- 实现 commit `96e0aa6c6ee8b80a49effd2c09095f1cfa26cfde` 的 [GitHub Actions run `35179012712`](https://github.com/LordCasser/jarde/actions/runs/35179012712) 为 **success**；Linux x86_64 上 stable/test/specification、MSRV 1.88.0 与 supply-chain 三个 job 全部通过。
- 远端 CI 通过后执行 `cargo clean`，Cargo 报告移除 **4,491 files / 1.7 GiB**；清理后 `target/` 已不存在，文件系统仍约 **103 GiB available**。

## 1.3 标准 MR-JAR selection

实现范围严格限于按 container、Manifest evidence 与唯一 winning level 派生的标准 MR selection report、有界路径/class Header 合规诊断和公共 `Engine::select_multi_release` 入口；不包含 XRef/consumer 扫描、resolver、loader/module resolution、API 等价、verifier 或跨 container 消歧，也不增加 CLI operation。该任务从上一轮 WIP commit `ee0a3a1`（不编译）继续完成。

### 已实现契约

- `Engine::select_multi_release(&ArtifactSnapshot, &RuntimeView, &mut Budget) -> Result<MultiReleaseViewReport>`：入口校验 `multi_release_not_zip`、`multi_release_snapshot_mismatch`、`multi_release_root_container_mismatch`；`SnapshotAll` 只重新枚举 root，`ArtifactTree` 复用 bounded tree provider；物理 report 原样内嵌并作为唯一物理事实源，provider 只执行一次。
- 以 Manifest evidence 与唯一 winning level 派生 selection：`Active` 只在唯一可解析 root Manifest 主段的 `Multi-Release` 展开值 ASCII 大小写不敏感地等于 `true` 时成立；`Disabled` 为 base-only；`Custom`/`Unknown` 保留 candidates 并以 `multi_release_custom_policy`/`multi_release_unknown_policy` 返回 Failed，不产生 selected fact。
- 版本路径分类器（Directory → 大小写敏感 `META-INF/versions/` → Base）与六个 `MultiReleaseVersionPathIssue`；versioned `META-INF` resource 与非法路径不参与选择并产生闭合 code 的诊断；decision precedence、`Selected`/`Shadowed`/`Ambiguous`/`Inactive`/`NoSelection`/`NotApplicable` 与 compliance join（`NonConformant` 吸收一切、`Unknown{ProbeInterrupted}` 仅作未 probe 占位）按 spec 闭合表实现。
- 合规检查与选择分离：classfile major 上界 `N + 44`、public predecessor 真值表（含 module evidence → `ModuleExportsNotInspected`、重复 base → `PredecessorAmbiguous`、malformed base 的 base 侧诊断 + versioned 侧 mismatch）只产生诊断与 compliance，不改写 path selection；`verification` 恒为 `VerificationStatus::NotPerformed`。
- 计费与中断：容器 report + ManifestEvidence 原子预留 2 项，entry evidence/selection/domain diagnostic 逐项在 append 前计费，terminal diagnostic 属控制元数据不计费且必返；阻塞预算与取消阻止后续容器；`runtime_resolution` 使用真实 ordinal 单 entry range，物理未枚举区间原样透传；未处理容器（`produces_report == false`、预留失败、break 后剩余）统一声明已知 ordinal 为 skipped。
- 两条覆盖规则写入 change spec：Manifest ordinal「未被读取」记 scanned 而「读取被真正尝试但中断」记 skipped；compliance 维度只覆盖 probe 适用范围，probe 跑完记 scanned，被尝试但中断（含作为 public predecessor 的 Base）或适用未跑记 skipped。
- 保留的解释性决定：主段未以空行结束（含 0 字节 Manifest）判 `Malformed` 且不猜测激活状态——比 JDK 宽容行为更严格，方向保守（只把选择降为 Unknown，不伪造 selected fact），已写入 spec 措辞。

### 本地验证与审查

以 `CARGO_BUILD_JOBS=1`、`CARGO_INCREMENTAL=0`、`RUST_TEST_THREADS=1` 串行执行；交付树上由主 Agent 独立复跑：

- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`：通过，0 warning。
- `PROPTEST_RNG_SEED=5350648285461741569` 与 `…570` 两次 workspace 全量：**218 passed / 0 failed / 1 ignored**（ignored 仍为显式 JDK 25 oracle）。与本任务直接相关：`tests/p1_multi_release.rs` **29 passed**、`--lib` **93 passed**（含 MR 内部语义锁）。
- `openspec validate --all --strict --no-interactive`：**8 passed / 0 failed**；`git diff --check`：通过。
- 行为覆盖：A06 三视图（Java 8/11/17 → root/11/17）、移除 v17 与 v11 的回退、缺失 Manifest、Manifest Ambiguous/Malformed/named section/LF-CRLF-CR/continuation/非 canonical 大小写、六类非法版本路径、versioned META-INF resource（含 class 不 probe）、duplicate（winning level 与非 winner 层）、compliance 真值表各行、release 9 合法版本目录、预算/取消/结果条目耗尽、ArtifactTree 多容器与空容器、物理 report 等价、JSON 闭合与 `deny_unknown_fields` 边界。
- 独立只读 review 首轮发现阻塞问题 **B1**：ArtifactTree 下容器循环以预置的物理聚合 execution 判停，导致已完整枚举的 child 静默不产出 report、其已知 ordinal 在 runtime coverage 中无痕；另列 N1–N7。修正轮完成 F1（判停只看本次运行自身的中断）、F2（未处理容器统一声明 ordinal）、F3（base predecessor malformed 不再改 base compliance）、F4（policy code 不被同优先级 Unsupported 顶掉）与 11 项测试补齐后，独立复核结论为 **Approve task 1.3**，并确认 D1–D11 未被破坏；修正后以 4 次「还原旧实现 → 对应用例 FAILED」实验证明回归可证伪。
- 收口轮关闭 review 的非阻塞项：probe 改为 `Untouched`/`Interrupted`/`Completed` 三态，使被尝试的 base predecessor 在 compliance 维度一致地记 skipped；新增「物理截断但 Manifest evidence 已返回」「0 字节 Manifest」「Custom policy + 结果条目耗尽（公共入口证明阻塞预算优先于 policy）」三条用例；A06 的 `Shadowed{winners}` 由 shape 断言改为内容断言。
- 留待后续：MR 的 `priority` 与 `artifact.rs::tree_issue_priority` 对 aggregate 终止原因的排序不一致；provider 中从未 pop 的已建立 child 仅在物理聚合 coverage 中留痕（属 1.2/P0 契约，需要在 provider 暴露 pending container coverage 才能进 runtime 维度）。
- 远端 CI 尚未执行：本轮改动未提交、未推送，CI 结论需在推送后补充。

## 2.1–2.4、3.1–3.2 结构 XRef、查询产品与 CLI 暴露

实现范围：crate-private reader 事实层（常量池条目视图含 span 与符号展开、目标 attribute 内容与类型化事实、`method_code_facts`、`BootstrapMethods`）→ `src/xref/` 扫描编排（resource/code/metadata/bootstrap 四个 consumer 子模块）→ `src/query.rs` 的请求/结果/分页/coverage 公共 schema 与关系分派 → `Engine::query` → `jarde-cli` 的 `query` operation。设计与共享契约记录在 `design.md` 决策 15–24 与「公共 schema 骨架」节，模块写入所有权按 owner 表执行，四个子模块互不改写公共类型。

### 已实现契约

- **X0 与 X1 分离**：`constant_pool_contains` 返回原始池条目候选（`XrefDerivation::ConstantPoolCandidate`、`consumer: None`、`XrefOperation::ConstantPoolEntry`），`mentions_symbol` 只由真实 consumer 产生结构引用；未被 Code 使用的 `Methodref` 在 X1 无结果而在 X0 命中（A01 双侧对照）。
- **consumer 覆盖**：invocation/field/type/constant/exception（Code 指令、`ldc` 族、invokedynamic、异常表 catch 类型）、hierarchy/descriptor/Signature/annotation/inner-nest/Exceptions/ConstantValue/module uses-provides（metadata）、Manifest 属性与 `META-INF/services`（resource）、deferred bootstrap/condy 图（bootstrap，含共享子图、`via` 路径、环与由池规模派生的上限）；未实现的 `Verification`/`Debug` 只在被请求时进入 `coverage.unsupported_categories` 且不声明 complete-within-schema。
- **查询契约**：`QueryRequest`/`QueryReport`/`QueryAnalysis`/`QueryPage`/`QueryCursor`/`QueryCoverage`/`XrefItem` 按骨架逐字实现；稳定错误码 `query_snapshot_mismatch`、`query_artifact_tree_root_mismatch`、`query_target_relation_mismatch`、`query_cursor_mismatch`、`query_consumer_schema_version`；游标绑定 engine schema、snapshot、physical view、relation、consumer schema 与已发布边界（含 digest），续页不重复、不跳过；`page.has_more` 与 `execution` 独立。
- **2.4 证据保留**：`references_definition`/`may_dispatch_to` 以 `UnsupportedAnalysis` 返回，同时复用 X0 探针保留原始池条目候选（relation 改写回调用方关系、游标仍绑定调用方请求），不做 owner 扩展或任何解析；`coverage.artifact_structural` 反映该次 CP 扫描的真实覆盖。
- **计费与中断**：`xref::scan` 是唯一 `ResultItems` 计费点；terminal 诊断不计费且必返；预算/取消保留可靠前缀并标 Partial/Cancelled，不伪造 Complete；每个 unit 的读取按其维度计费。
- **CLI（3.2）**：新增 `query` operation（relation/target/physical scope/consumers/max_items/cursor），snapshot 由适配层 open 后回显，报告与库 `Engine::query` 逐字段一致，`OutputBytes` 收敛与错误契约沿用既有路径，未复制扫描或过滤逻辑。

### 本地验证与审查

以 `CARGO_BUILD_JOBS=1`、`CARGO_INCREMENTAL=0`、`RUST_TEST_THREADS=1` 串行执行；交付树上由主 Agent 独立复跑：

- `cargo fmt --all -- --check`：通过；`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`：通过（exit 0）。
- `PROPTEST_RNG_SEED=5350648285461741569` 与 `…570` 两次 workspace 全量：**250 passed / 0 failed / 1 ignored**（ignored 仍为显式 JDK 25 oracle）。分目标：lib 93、engine 9、historical 1、artifact-tree 14、multi-release 29、**p1_query_api 18**、query-model 6、**p1_reader_facts 3**、**p1_xref_bootstrap 23**、**p1_xref_code 19**、**p1_xref_metadata 17**、json_cli 8、**query_cli 10**。
- `openspec validate --all --strict --no-interactive`：**8 passed / 0 failed**；`git diff --check`：通过。
- 验收映射：A01/A02（未使用 Methodref 对照、invokevirtual 位置与 descriptor，含与公共 `inspect_method_bytecode` 的一致性）、A03（annotation/Signature/Exceptions/catch 中独有类型，含不经过 `CONSTANT_Class` 的样本）、A04/A05（真实 `metafactory` 形态 fixture：6 参描述符 + 恰好 3 个静态实参，实现 handle 在静态 `argument_index` 1，可链接性由属性字节断言；SAM 名由 code consumer 报告而 bootstrap-only 请求为 0 项；嵌套 condy 与共享子图、未使用表项不产出、环与超限诊断）、A14（预算/取消/`page.has_more` 与 execution 分离）、A17（无 CFG/SSA/Java AST，死代码中的引用仍返回）。
- 独立只读 review：**2.1 Accept**（A01 双侧对照、A02 坐标与对照、A17、类别过滤与局部性、确定性/续页成立；公共契约与 crate-private 边界无破坏）。**2.2 首轮 Reject → 修复 → Accept**：首轮发现阻塞缺陷——record component 的 `Signature` 属性长度按已消费字节记账错误（`checked_sub(4)`），使合法泛型 record 被判 malformed 并以 `Failed` 终止整个 unit；且正向用例不断言 `execution`/`diagnostics`，使"前缀已发布、中途停止"不可见。修复改为按已消费字节数计算 `attribute_body` 剩余量、错误码区分为 `query_record_malformed`（Record 结构）与 `query_signature_malformed`（签名语法），并把全部正向用例接入 `run_complete`（Complete + 空诊断，共 67 处完成性断言）；以「还原旧记账 → 用例失败」探针证明回归可证伪。**2.4 落地正确、B3 关闭**（保留的候选与 `constant_pool_contains` 逐字段相同、无 consumer/BCI/opcode/解析含义、不扩展 owner、未开计费与发布特例、游标仍绑定调用方关系）。**2.3 首轮不 Accept → 修复 → Accept**：首轮指出文档与 fixture 把"标准 LambdaMetafactory 形态"写错（JVM 在调用自行提供 `Lookup`、名字与描述符派生的 `Class`/`MethodType` 三个前置实参，`metafactory` 的静态实参为 `[samMethodType, implMethod, instantiatedMethodType]`，实现 handle 在静态 `argument_index` 1，SAM 名来自站点 `NameAndType`），旧 5 参 fixture 因此不可链接、A04 的正面证据不成立；修复改写为 JVMS 口径并新增真实形态 fixture（可链接性由 `BootstrapMethods` 属性字节断言、位置链四重钉死、`Invocation` 与 `Bootstrap` 在同一站点符号上分工对照），旧样本降级为 structural sample 并把"不可链接"标签钉成可证伪。**3.2 Accept**（薄委托、请求形状、库/CLI 逐字段一致、分页与错误语义、`OutputBytes` 不部分发布、fixture 可被真实引擎解析）。
- 已验证的中间态口径：`UnsupportedAnalysis` 与「performed 且无命中」在报告与 CLI 响应中可区分。

### 留待后续（不阻塞本阶段验收）

- **一致性**：`.class` 后缀谓词在 `code.rs`（大小写敏感、接受裸 `.class`）与 `metadata.rs`（大小写不敏感）不一致，统一前 `Foo.CLASS`/裸 `.class` 的扫描范围不同；建议统一到大小写敏感并把 helper 放 `mod.rs`。（2026-09-17：已作为 R4/2.1 修复并复核，见下方修复轮。）
- **覆盖缺口**：record component 上的注解属性从不读取（`Record` 目前只在请求 Signature 时打开），只出现在该位置的注解类型对 `Annotation` 类别是假阴性；`ldc` 的 `MethodType` 与 condy/`NameAndType` 描述符内的类型同样不产出（架构把它划给 descriptor/signature consumer）。（2026-09-17：已作为 R2/R3、任务 2.2 的修复范围，实施中。）
- **成本**：同一 unit 在 code 与 metadata 同时请求时被各物化一次，读/entry/class/attribute 计费翻倍（standalone CLASS 另有 `OutputBytes`）；准确结果是更早 Partial，不是错误。按 roadmap 属 P5「按实测决定的缓存/索引」范围。
- **文档**：`design.md` 决策 16 需补一句 `coverage.artifact_structural` 对未分析关系的含义（原始 CP 探针覆盖 ≠ 声明 consumer schema 全扫）；决策 21 的「entry 内位置」宜写成「单元内已发布序号」；`src/classfile.rs` 事实层残留 19 处现已冗余的 `#[allow(dead_code)]`；CLI 面向人的输出宜显式提示 `UnsupportedAnalysis`。
- **跨 API**：MR 的 `priority` 与 `artifact.rs::tree_issue_priority` 对 aggregate 终止原因的排序仍不一致。

## 2026-09-17 修复轮 R1–R4

按 design 的推进顺序修复复核发现的问题：每项保留最小复现，先确认旧实现失败，修复后再做独立只读复核。本节只覆盖 R1/R4；双固定种子、MSRV、supply-chain、ignored JDK 25 oracle、fuzz、`openspec validate` 与最终候选 CI 属 3.3/3.4 门禁，这里不声明。

### R1 / 3.1 游标绑定完整 QueryTarget（已复核 Approve）

- 实现：`QueryCursor` 新增 `target`，`QUERY_ENGINE_SCHEMA` 1 → 2（不保留旧游标兼容）；`cursor_digest` 以分层 ASCII tag + 原始字节/位模式编码 target（`Class{owner}` ≠ `Field{owner,"",""}`，String ≠ 同字节 Class，整数/浮点定宽大端位模式）；`validate_cursor` 在 relation 之后做 target 等值校验，任何失配仍返回 `query_cursor_mismatch` 并指名绑定字段；`src/xref/mod.rs::build_cursor` 传入调用方 request。
- 反例与证伪：同一测试文本在旧实现（临时回退，按校验和还原）下，跨目标续页返回 `items=0`、`execution=Complete`、`artifact_structural=CompleteWithinSchema`、空诊断；修复后为 `query_cursor_mismatch`。复核者另用公共 CLI 探针复现同一症状，并确认拒绝发生在任何枚举/读取/计费之前（`archive_entries=0`、`result_items=0`）。
- 覆盖：`tests/p1_query_api.rs` 的跨目标矩阵（symbol owner/name/descriptor/variant、literal kind/原始值/同字节不同 kind、float `0.0` vs `-0.0`、两个 NaN payload、double、float→double）、游标 JSON 缺 `target` 被拒、同身份分页等价（`max_items` 1/2/3/0 与逐页变化）、预算中断续页等价（`result_items` 中断得到 Partial + 游标 + 真实维度诊断，续页拼接等于不分页整跑、末页 Complete/CompleteWithinSchema）；`crates/jarde-cli/tests/query_cli.rs` 经真实 CLI stdin 拒绝跨目标续页并保留同身份续页成功。
- 复核：独立只读复核 **Approve**，无阻塞问题；唯一实质缺口（预算维度的续页等价未落成 committed 用例）已补齐，并给出 5 项可证伪实验（去掉预算中断 / 续页丢一项 / 放宽末页状态 / 重复发布 / 换预算维度，均 FAILED）。

### R4 / 2.1 共享 class candidate 规则与损坏候选诊断（已复核 Approve）

- 实现：`src/xref/mod.rs` 新增唯一候选规则 `is_class_candidate`（大小写敏感 `.class`、含裸 `.class`、目录名以 `/` 结尾自然排除）与 `class_content`：非候选 entry 不读取不计费；候选的 magic 缺失/截断/错误 → `query_class_candidate_malformed`（message 区分"短于 magic"与"四字节不对"，含转义后的 entry 名与观察长度）；standalone CLASS root 不套候选规则、不报损坏（P0 `classify` 保证其以 magic 开头）。code/metadata/bootstrap 删除各自的本地判定与 magic 检查，统一调用。fail-stop 编排、`coverage_parts`、`merge_issue`、`terminal_*` 未改，未处理 siblings 继续进入 skipped 区间。
- 反例与证伪：三个 xref 测试文件的 damaged 用例在旧实现下分别表现为"越过损坏候选继续扫描"（code/bootstrap 得到 2 条重复 item、metadata 得到 2 条 item 且无诊断、都声称 Complete）；修复后为 `Failed{reason: Error{code: query_class_candidate_malformed}}` + Error 诊断（provenance 指向真实 entry，含嵌套容器 origin）+ `artifact_structural=Partial` + sibling skipped 区间 + `has_more=true`，损坏前的前缀 item 保留。
- 覆盖：三个 scanner 各有 damaged 用例（截断 `CA FE BA`、0 字节、错误 magic、非 UTF-8 entry 名）与候选规则一致性用例（`Good.class`/裸 `.class` 正证，`Good.CLASS`/普通资源反证且不误报损坏，Phase 0 已把 `Good.CLASS` 改为合法 class 字节以对齐 spec Scenario 字面量），以及"只请求 Resource/Verification/Debug 时不读 class 字节、不报损坏"的类别范围对照；两处编码旧契约的既有用例（`zip_scan_reads_the_class_entry_and_never_other_entries`、`archive_entries_are_scanned_only_when_they_are_class_files`）改写后保留"非候选从不物化"与精确计费断言。
- 复核：独立只读复核 **Approve**，无阻塞问题；复核者用公共 CLI 重跑 design 的 R4 原始反例，并核对边界：损坏在容器末尾 / 同容器 sibling / 嵌套 sibling / 多容器的 skipped 算术、类别范围与计费（受损候选只计费一次）、standalone root 四种字节形态不可达性、逐字节 diff 的范围核对。非阻塞项按下面"后续项"处理：目录位约定写入 spec、`Location::Entry.span` 口径分歧与 `has_more/cursor=null` 记入 design 拆分处理表、`Good.CLASS` 用合法字节、`verification.md` 旧计数换成本轮证据。
- 反例可复现性：pre-fix 基线以 `.git` 中 unreachable blob 保存（`src/xref/mod.rs` `0c7c5ced…`、`code.rs` `153c5086…`、`metadata.rs` `48d82b50…`、`bootstrap.rs` `f4f8554d…`、`p1_xref_code.rs` `638571d8…`、`p1_xref_metadata.rs` `74f10041…`、`p1_xref_bootstrap.rs` `8d3f38a3…`），可用 `git cat-file -p <sha>` 复现 diff；这些对象被 prune 后失效。

### 本轮共同证据

单作业（`CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1`）在 R1+R4 交付树上由主 Agent 独立复跑：`cargo fmt --all -- --check` 通过；`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 通过；`cargo test --workspace --all-targets --all-features --locked` 为 **263 passed / 0 failed / 1 ignored**（lib 93、engine 9、historical 1、artifact-tree 14、multi-release 29、p1_query_api 22、query_cli 11、json_cli 8、query-model 6、p1_reader_facts 3、p1_xref_code 23、p1_xref_metadata 19、p1_xref_bootstrap 25；ignored 仍为显式 JDK 25 oracle）。本轮未跑 CI 要求的双固定种子两遍、MSRV 1.88.0、supply-chain、ignored JDK 25 oracle 与 fuzz，也未执行远端 CI；这些属 3.3/3.4 门禁。

### R2 / R3 与方法签名语法（2.2，已复核 Approve）

- 实现：reader 层（`src/classfile.rs`）接管 descriptor 事实（`DescriptorKind`/`descriptor_types` 从 metadata 迁入、新增 `entry_descriptor`：`MethodType`→method、`FieldRef`→field、`MethodRef`/`InterfaceMethodRef`→method、`Dynamic`→**field**、`InvokeDynamic`→method）、新增 `code_nested_attributes`（走 Code 结构到末尾的嵌套 attribute 列表，解指令为 0、不计 `CodeBytes`、每个 `Code` entry 一次 `AttributeBytes = 6 + content length`）与不计费的 `attribute_slice`。metadata consumer 读取 Record component 的可见/不可见注解与类型注解、以及 Code 内的可见/不可见类型注解；新增的嵌套位置用 `Location::Attribute{path, span}`（`Record.components[i].<attr>` / `methods[i].Code.<attr>`），`span` 是结构自身范围，`evidence.bci` 只在该结构确实命名一个 BCI 时写入。code consumer 对 `ldc` 的 `MethodType`/`MethodHandle` 成员/`Dynamic` 与 `invokedynamic` 站点按 `ConsumerKind::Type` 发布 descriptor 类型（operation 沿用 `Ldc`/`InvokeDynamic`，证据与同指令既有 item 同口径）；bootstrap consumer 对可达节点发布带 `via` 的 descriptor 类型（operation `BootstrapMethod`/`BootstrapArgument`，derivation `BootstrapEdge`）。决策 26 的门槛为类别含 `Bootstrap` 或 `Type` 之一且仍只在真实 use-site 时打开。
- 同轮修复第三类假失败：方法 `Signature` 的 `Result`/`ThrowsSignature` 曾按 descriptor 产生式解析，合法 javac 23.0.1 输出（`()Ljava/util/List<Ljava/lang/String;>;`、`(TT;)TT;`、`()V^TE;`）使请求以 `query_signature_malformed` 失败、0 item。现在按 JVMS 4.7.9.1 分离签名/descriptor 两条路径，`throws` 的 class 类型产生 `GenericSignature` item、type variable 不产生；class 签名的 superclass/interfaces 同步收紧为 `ClassTypeSignature`。
- 反例与证伪（均先在未修实现上跑出旧行为）：R2 record / R2 Code / R3 MethodType / R3 MethodHandle-condy-indy / R3 bootstrap-descriptor 五组新用例在旧实现下分别为 0 item + `Complete` + `CompleteWithinSchema` + 空诊断；签名四样本为 `Failed{query_signature_malformed}` + 0 item。修复后全部命中并断言精确来源。真实 javac 23.0.1 样本（`tests/fixtures/r2-annotation-positions/`、`method-signature-grammar/`、`b2-bootstrap-descriptor/`）按 README 命令重编后 SHA-256 逐字节一致。
- 首轮复核 **Reject** 的两个阻塞问题已按修正方向闭合并复验：
  - **B1 嵌套落位**：`parse_annotation` 只在结构读完后才落位，导致"结构内先命中、随后失败"的 item 以 `Location::ClassOffset` + CP 条目 span + `bci=None` 发布。现改为 `Scan::pending` 缓冲：嵌套位置在落位前不写 `out`，结构失败即丢弃（不发布、不计 `ResultItems`），成功则整体按顺序并入。复核者用自己的探针复验：旧行为 `items=1` + `ClassOffset{offset=102}` + CP span → 现为 `items=0` + `result_items=0` + `Failed{query_annotation_malformed}` + 非 Complete；合法对照仍给出 `Location::Attribute{path, span, bci}`；同一 attribute 内多个结构时，已完整结束的结构各自保留正确 span 与计费；flat 事实不被牵连（superclass、component 描述符等前缀项仍以 `ClassOffset` 发布）。
  - **B2 Type 门槛**：`Type`-only 曾不触达 bootstrap 可达 descriptor 类型（与 spec Scenario 和决策 26 的字面要求冲突）。现入口门槛改为 `Bootstrap || Type`，节点 symbol/literal 事实仍只属 `Bootstrap`、descriptor 类型只属 `Type`、无 use-site 时任何类别都不读 `BootstrapMethods`。复核者独立复验：`Type`-only 对真实 lambda 样本命中 `java/lang/invoke/MethodType`（`consumer=Type`、`operation=BootstrapMethod`、`via` 两跳、`bci=0`、`opcode=0xba`）；`Bootstrap`-only 0 条类型事实；无 use-site 但表项描述符命名了目标类型时三种类别均 0 项（未使用表项不产事实仍成立）。
- 记录在案的行为/成本变化（决策 26 的直接结果，非缺陷）：`Type`-only 现在对每个 unit 读 3 遍（code/metadata/bootstrap 各一次 class 读）、解 2 遍方法体、读 1 次 `BootstrapMethods` 内容（真实 lambda 样本实测 `attribute_bytes = 3×107 + 2×73 + 18 = 485`、`class_bytes = 3×731`、`code_bytes = 2×19`），紧预算下更早返回真实 Partial；`Type`-only 也会因"有 dynamic use-site 但表缺失/重复/畸形"而失败（仅非法 class 会触发）。两者分别记入 design 的拆分处理表与支持矩阵边界。
- 证据：单作业下 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 通过；`cargo test --workspace --all-targets --all-features --locked` 为 **282 passed / 0 failed / 1 ignored**（lib 93、engine 9、historical 1、artifact-tree 14、multi-release 29、p1_query_api 22、query_cli 11、json_cli 8、query-model 6、p1_reader_facts 3、p1_xref_code 26、p1_xref_metadata 32、p1_xref_bootstrap 28；ignored 仍为显式 JDK 25 oracle）；由主 Agent 独立复跑确认。
- 独立复核结论：首轮 **Reject → 修正 → Approve**；复核者另有本机 javac 23.0.1 重编样本比对摘要、22 种 `target_type` 的 BCI 判定全表、6 类 Code 结构坏样本边界、以及未使用表项/类别隔离的独立反例。非阻塞项处理：顺序与"未请求类别不会失败"的 doc 措辞已改精确、class 签名已收紧、record component 描述符的类别归属不一致与重复物化记入 design 的拆分处理表。

### 尚未关闭

- 3.3 已完成并复核 **Approve**（语料索引、golden、性质与有界 fuzz 见下两节）。
- 3.4 文档同步（README、支持矩阵、OpenSpec 入口/roadmap/acceptance）与完整 CI/归档：进行中；在最终候选 CI 通过前不勾选 3.4、不归档。

## 2026-09-17 P1 fuzz 门禁与高扇出边界（3.3 第二部分）

- 独立 test-only workspace `fuzz/`：自带 `[workspace]`（不是主 workspace 成员）、提交 `fuzz/Cargo.lock`、`fuzz/rust-toolchain.toml` 固定 `nightly-2026-07-20`（rustc 1.99.0-nightly）、`libfuzzer-sys =0.4.13`；工具 `cargo-fuzz 0.13.2`（`--locked` 安装）。`fuzz/src/lib.rs` 只把整段输入当 artifact 并只用公共入口与既有 `Limits`，两个 target（`query`、`artifact_tree`）分别覆盖查询与 MR/artifact-tree；契约检查（item 数 ≤ `limits.result_items`、`CompleteWithinSchema ⇒ execution Complete ∧ unsupported_categories 空 ∧ skipped 空`、非 Complete ⇒ `Partial`、`cursor ⇒ has_more`、usage 各维 ≤ limits、MR `verification == NotPerformed`）由两个 `#[should_panic]` 自检证明非空转。
- 种子：`python3 fuzz/corpus/generate_seeds.py`（固定时间戳、无随机、无网络、可重复生成并打印 SHA-256）产生 10 个最小种子（合法 class/jar、截断 class/jar、坏 magic 候选 jar、CRC 不符 jar、多版本/嵌套 jar），摘要表在 `fuzz/README.md`；`cargo test --manifest-path fuzz/Cargo.toml` 另断言"合法种子真产出 item、损坏种子不假 Complete"。
- 冒烟（单 worker、`-max_len=65536 -rss_limit_mb=512`）：本地 60 秒两次（`query` 1 646 123 / 1 650 577 次执行，peak RSS 173/181 MB；`artifact_tree` 1 278 312 次，peak 456 MB），并用 `-s none` 对照（同命令 61 秒 4 719 875 次、peak 平坦 30 MB）把 RSS 归因于 sanitizer 而非引擎；同一最小种子重复运行结果一致，`fuzz/artifacts` 为空。独立复核者另复跑 20 秒与 60 秒两种时长，均 exit 0、无 panic/OOM，种子 SHA-256 与本轮开始前一致。
- CI：`.github/workflows/ci.yml` 新增 `fuzz-smoke` job（`stable`/`msrv`/`supply-chain` 未改）：装 `nightly-2026-07-20` 与 `cargo-fuzz 0.13.2 --locked`、复用 runner 自带 clang、先跑 fuzz workspace 的语料自检，再对 `$RUNNER_TEMP` 的语料副本每 target 跑 20 秒（`-max_len=65536 -rss_limit_mb=512 -timeout=10 -workers=1`），末尾 `git diff --exit-code` 与干净树检查。**design 的"收口 smoke 至少单 worker 60 秒"由本地记录承担，CI 的 20 秒是额外有界门禁**（`fuzz/README.md` 与 job 注释都写明），本轮按此口径收口；CI 侧 RSS 余量（实测 peak 418–463 MB 对 512 MB）记入下节后续项。
- 主 workspace 未受影响：`cargo metadata` 仍只有 `jarde`/`jarde-cli`，`cargo tree --workspace -e normal` 无 `libfuzzer-sys`/`cc`/`arbitrary` 等；根 `Cargo.toml`/`Cargo.lock` 未改，MSRV 仍 1.88.0。
- 许可证/advisories：`cargo deny check` 在根与 `fuzz/` 均四段 ok。策略 owner 决定在单一 `deny.toml` 中允许 `NCSA` 并加注释（`libfuzzer-sys` 随包 vendored 的 LLVM libFuzzer 运行时，仅 test-only fuzz workspace 可达）；复核者用去掉该允许项的副本复验：fuzz 侧唯一违规者为 `libfuzzer-sys 0.4.13`，而根 workspace 仍 `licenses ok`，证明该允许项未掩盖生产依赖。
- 高扇出/结果预算边界（`tests/p1_query_bounds.rs`，4 用例，单 entry 内 2048 条命中同一 target 的 `invokevirtual`）：`result_items=16` 时只发布 15 项而 `code_bytes == 6145`，即 design 点名的"先累积 `unit_items` 再计 `ResultItems`"确实使**结果预算不能约束构造**；但构造被输入维度预算约束——`code_bytes = 3×1024` 时恰停在 1024 项且 `items×3 ≤ 已付 code_bytes`，每条 item 都对应已计费字节（`invokevirtual` 3 B、`ldc` 2 B、注解/描述符类型 ≥3 B、CP 候选 ≥1 B；`size_of::<XrefItem>() = 392 B`），因此不存在与预算无关的放大构造。结论：**不构成 I12 正确性修复**，作为已记录的复杂度特征登记（见 design 拆分处理表）；前缀等值、分页等价与取消边界（standalone 预取消、单 entry jar 取消）由同用例覆盖。
- 证据：`cargo fmt --all -- --check` 与 `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 通过；`cargo test --workspace --all-targets --all-features --locked` 在 `PROPTEST_RNG_SEED` 两个固定种子下各 **299 passed / 0 failed / 1 ignored**（93 lib、9 engine、1 historical、14 artifact-tree、29 multi-release、23 p1_query_api、4 p1_query_bounds、6 query-model、3 reader-facts、28 p1_xref_bootstrap、28 p1_xref_code、5 p1_xref_golden、33 p1_xref_metadata、4 p1_xref_properties、8 json_cli、11 query_cli；ignored 仍为显式 JDK 25 oracle），由主 Agent 独立复跑确认。
- 独立复核结论：**Approve**（golden 与实现逐字段等价、逐 replay 11–14 类语义字段扰动（去重 17 类）全部被检出、性质在真实样本上未找到反例、fuzz 门槛与语料自检非空转、CI job 与版本链一致、NCSA 允许项只服务 test-only 依赖）。非阻塞项处理：golden 增加 replay 名单/数量断言、性质 P1 末页断言改为按 execution 分支、索引两处措辞修正；属性生成器未覆盖 annotation/Bootstrap 形状与 CI RSS 余量记入 design 的拆分处理表。

## 2026-09-17 P1 验收语料索引（3.3 第一部分）

真实编译样本位于 `tests/fixtures/{historical,r2-annotation-positions,method-signature-grammar,b2-bootstrap-descriptor}/`，各自 README 记录编译器、命令、dialect、字节数与 SHA-256；编译器只是生成期输入，测试只读 checked-in 字节。手写样本由具名 generator 构建，测试直接断言 CP index/BCI/opcode/span 等坐标；其中五份 golden 输入另由 `tests/p1_xref_golden.rs::open_fixture` 校验 blake3 摘要。总览与重放方式见新增的 `tests/fixtures/README.md`；本索引与下文的 fuzz 门禁共同构成 3.3 的收口证据。

| 验收 | 覆盖测试 | 样本与来源 | 断言的关键不变量 |
| --- | --- | --- | --- |
| A01 | `p1_xref_code.rs::unused_constant_pool_entries_are_candidates_but_never_calls`、`a_value_relation_reports_consumers_and_never_pool_candidates`、golden `code.json` 的 X0/X1 对照 replay | 手写 Java 8 class（未使用的 `Methodref java/lang/Runtime.exec` 与 `Class p/UnusedType`）；golden 输入 196 B、blake3 `f77b8b60…22a7` | X0 恰一条 `ConstantPoolCandidate`（`consumer=None`、`derivation=ConstantPoolCandidate`、精确 index 与 tag 起始 span、无 BCI/opcode/via）；同一目标在 X1 五类别下 `items=[]`、`scanned_items=0`、Complete + CompleteWithinSchema + 空诊断 |
| A02 | `p1_xref_code.rs::invocation_item_keeps_the_complete_symbol_and_its_byte_coordinates`、golden `code.json` 的调用点与 literal replay | ECJ 4.6.1 v52 历史语料（SHA-256 `f9b6566f…b6ad`，来源见其 README）+ 手写 fixture | 完整 `SymbolRef::Method{owner,name,descriptor}`；`consumer=Invocation`、精确 operation/CP index/BCI/opcode、`attribute=Code`、span 起点为 opcode 且操作数与 CP 条目一致；同一 BCI 上与公共 `inspect_method_bytecode` 逐字段相等 |
| A03 | `p1_xref_metadata.rs::metadata_only_types_hit_only_their_requested_category`、record/Code 嵌套位置与编译样本用例、`requested_verification_and_debug_never_claim_a_complete_schema`、golden `metadata.json` | 手写 `meta_fixture`/`annotation_fixture`/`hierarchy_fixture`/`record_annotation_fixture`/`code_annotation_fixture`/`signature_fixture`/`module_fixture`；编译样本 `r2-annotation-positions`（v17 `11342150…f636e293`、v8 `8aabf4f4…e982ce29`）与 `method-signature-grammar`；golden 输入 232 B、blake3 `c1cea346…58c4` | 单类别请求只由本类别回答、其余类别为空；名字不存在于 `CONSTANT_Class`（描述符/签名类型无独立 `Utf8`）；正例 `Complete` + 空诊断 + 精确 consumer/operation/attribute/CP evidence；请求 `Verification`/`Debug` 时 `unsupported_categories` 精确列出且 `artifact_structural != CompleteWithinSchema`，只请求它们时零读取 |
| A04/A05 | `p1_xref_bootstrap.rs` 的 metafactory/结构样本/共享子图/未使用表项/环与深度预算用例、`a_type_only_request_reaches_a_type_a_bootstrap_descriptor_names`、golden `bootstrap.json` | 手写 `real_lambda_fixture`（6 参描述符 + 3 静态实参、实现 handle 在静态 `argument_index 1`）与 `descriptor_bootstrap_fixture`；编译样本 `b2-bootstrap-descriptor/v8/LambdaSample.class`（javac 23.0.1 `--release 8 -g:none`，731 B，SHA-256 `9588c94b…4083`，golden blake3 `92f2b3bc…e47`） | 实现 handle 沿 `via`（站点→handle 的静态实参位置）可追溯；每条静态实参保留自己的 `argument_index` 与值；bootstrap 边不是调用/加载/分配；未使用表项零事实；`Type` 类别按真实 use-site 触达可达描述符；SAM 名由 code consumer 报告；环/深度/边预算以诊断收口 |
| A06 | `p1_multi_release.rs::a06_selects_root_11_and_17_and_marks_the_other_variants` 等、golden `multi-release.json` | 手写 `a06_fixture`；golden 输入 1014 B、blake3 `413d7b4b…51b`（base/v11/v17 `p/Join.class` 各自调用 `p/All.run()V`，Manifest `Multi-Release: true`） | 三视图分别选中 `p/Join.class` / `versions/11/…` / `versions/17/…` 且 `verification=NotPerformed`；同一 physical query 不受 runtime 视图过滤，返回 3 条 item 且 `PhysicalVariant` 为 `Base`/`MultiRelease{11}`/`MultiRelease{17}`、entry 两两不同、不静默合并 |
| A07 | `p1_query_model.rs::directed_nested_origins_preserve_edges_and_distinguish_duplicate_parents`、`p1_artifact_tree.rs::duplicate_nested_entries_derive_distinct_child_and_inner_identities`、golden `nested.json`、性质 P4 | golden 输入 739 B、blake3 `5c011909…437`（同一 jar 字节分别 STORED 在 `WEB-INF/lib/a.jar`、DEFLATED 在 `WEB-INF/lib/b.jar`，各含同名同字节 `p/Dup.class`） | 两个 item 的 `class_bytes.digest/length` 相同但 `PhysicalEntryId`（origin steps、via_ordinal、child container）不同；tree 报告给出两个 `WarLibrary` layout node 且根 entry 压缩方法为 STORED+DEFLATED；loader/order 与歧义消解仍归 P2 |
| A08 | `p1_artifact_tree.rs` 的 DEFLATED/深度/预算/取消/伪造 origin 用例、golden `nested.json`、`p1_xref_code.rs::a_nested_truncated_candidate_bounds_pages_budgets_and_cancellation` | rawzip writer 生成的 STORED/DEFLATED child；新用例的 root=`lib/a.jar` STORED、`lib/b.jar` DEFLATED，内含调用、截断 `Broken.class`、可读 sibling | STORED/DEFLATED 均经 central/local header、size、CRC、压缩方法校验并从 root 逐层重放；中间物化只计 read/entry、最终 output 才计 output；嵌套截断候选给出带嵌套 origin 的诊断、非 Complete execution/coverage、前缀保留、未读 sibling 与第二个 child 进入 skipped `(2,3)`/`(0,1)`、续页不重复且仍非 Complete |
| A14 | `p1_query_api.rs` 的分页/预算/取消用例、`p1_xref_code.rs`/`p1_xref_bootstrap.rs` 的 stop 用例、`p1_multi_release.rs::budget_exhaustion_is_partial_and_never_fakes_completion` 等、`query_cli.rs::result_item_budget_keeps_the_reliable_prefix_and_names_the_dimension`、性质 P1 | 顶层与嵌套 fixture、性质生成器 | scanned/skipped 区间精确、终止维度点名、可靠前缀保留、绝不 Complete；`page.has_more` 与 execution 独立（失败场景可 `has_more=true` 且 skipped 非空，后续页仍 Failed，不把 `has_more` 当完成） |
| A17 | `p1_xref_code.rs::references_in_unreachable_code_are_still_reported`、`a_linear_scan_reads_trailing_dead_bytes_without_reachability_analysis`（`code.json` 只固定线性 body 的坐标，不作为死代码证据） | 手写 `dead_code_fixture()` 与 `trailing_dead_code_fixture()` | `goto` 跳过与 `return` 之后的字节中的引用仍作为字节事实返回（可达性分析会漏掉）；`code_bytes` 恰为全部指令宽度；无额外诊断/维度；生产依赖边界由 CI 的 normal-tree grep 守卫 |
| A18 | `engine.rs` 快照 seam、`p1_query_api.rs::cursor_mismatches_bind_snapshot_view_relation_and_schema`、R1 的跨目标/分页/预算用例、`query_cli.rs` 的跨目标拒绝、`a18_open_snapshot_stays_stable_and_a_reopened_one_rejects_old_cursors`、性质 P1 | 临时路径打开的快照 + `TempDir` helper | 快照自持字节；源路径内容改写后旧快照结果与续页不漂移；重新 open 得到不同 id，旧 snapshot 的请求返回 `query_snapshot_mismatch`、携带旧游标（schema=2、含 target）返回 `query_cursor_mismatch` 且点名 `cursor snapshot`；两次拒绝都发生在任何枚举与发布计费之前（断言 `archive_entries == 0`、`result_items == 0`） |

Golden（`tests/p1_xref_golden.rs`，5 项测试 + `tests/fixtures/p1-golden/*.json`，静态文件、缺失或变更即失败、无自动更新路径）：`code.json`（A01/A02，4 replay）、`metadata.json`（A03，5 replay，另断言单类别与组合请求 items 相等）、`bootstrap.json`（A04/A05，4 replay，真实 lambda）、`multi-release.json`（A06/A07，3 selection + 1 physical query）、`nested.json`（A07/A08，tree scope 两 origin）。归一化函数只删除 `elapsed_millis`（唯一依赖墙钟的字段）；snapshot id、digest、origin、coverage、diagnostics、via、完整 symbol、BCI/opcode/CP index/span 与 item 顺序都逐字段相等，且 golden 文件内含请求本身以便独立重放。

性质测试（`tests/p1_xref_properties.rs`，4 个 proptest、各 24 cases、两个固定种子）：P1 分页拼接等价（随机页大小与预算交替，逐页拼接 = 不分页整跑，游标绑定 schema/target；`Complete` 分支断言 `has_more == false`，契约允许的"停止且未发布新项"分支只断言 `has_more` 与已发布前缀等于整跑前缀——当前生成器/限值下该分支不可达，全 splice 等价仍每例执行，若未来形状恒停在停止页则由 `p1_query_bounds.rs`/`p1_query_api.rs` 承担该等价性）；P2 类别组合一致性（组合 items = 各单类别 items 的多重集并集，单类别 item 的 consumer 恒为该类别）；P3 插入未使用常量池条目不改变既有 X1 事实投影、且新 owner 在 X0 有候选而 X1 零命中；P4 同 bytes 三份 origin（根 STORED + 两嵌套）分为三组、组内事实序列一致、entry 两两不同。已知限制：除五份 golden 输入与编译样本外，其余手写 fixture 只在 `tests/fixtures/README.md` 记录 generator/test 名，未逐个钉字节摘要；性质生成器只构造 `Code` 体与未使用引用，annotation/Bootstrap/Record 形状的类别交互由确定性用例覆盖。
