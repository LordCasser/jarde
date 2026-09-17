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
