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
- 最终本地交付检查记录约 **9.3 GiB available memory**、文件系统约 **103 GiB available**，`target/` 的 `du` 字节数为 `1,606,243,938`；远端 CI 通过后再执行 `cargo clean` 并补录释放量。
