# P0 验证记录

## 记录身份

- 记录日期：**2026-09-17**。
- 本记录由**包含本文件的 commit**绑定；实现父基线为已推送的 `824971f`。
- 下列记录把已推送的 3.3 基线证据与当前 3.4 candidate 的重新验证分开；旧结果不冒充新文件状态。Linux x86_64 的远端 CI 结果只有包含本文件的 commit 推送后才能补录。

## 实测环境

- OS：Fedora-like Linux，aarch64。
- Kernel：`7.1.0-rc3-gaokun3+`。
- Rust：`rustc 1.96.1`、`cargo 1.96.1`。
- OpenSpec：`1.11.0`。
- JDK oracle：OpenJDK `25.0.4+7`（本地发行版未记录为 Temurin；CI 配置使用 Temurin）。
- P0 支持目标限 64-bit；Linux x86_64 已由固定 `ubuntu-24.04` remote CI 验证，Linux aarch64 有本地证据，32-bit 未建立支持证据。

## 最终 3.3 candidate 的已实测证据

此前在父基线候选上实际执行并得到：

- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`：通过。
- `cargo test --workspace --all-targets`：一轮通过；**72 lib + 9 engine + 1 historical + 8 CLI = 90 passed**，另有 **1 个 oracle ignored**。
- 显式 JDK oracle：**1 passed**；OpenJDK runtime `25.0.4+7`，固定动态 fixture 为 classfile 52.0，SHA-256 `04ea6ad5115a0c17d4bd604ef262f8110e417efa8725f39c9de9641565c2b345`，scope `instruction_boundary_only_not_verification`。
- `openspec validate --all --strict --no-interactive`：**6 passed / 0 failed**。
- 生产依赖 tree 检查与源码边界检查：仅 P0 artifact、budget、classfile、engine、model、error 与薄 CLI；无 query、resolver、XRef、CFG、SSA、AST、IR、Decompiler、JVM、网络、async 或数据库运行时。
- 历史 corpus：ECJ 4.6.1 生成的 8 个 45.3–52.0 fixture 全部通过；精确 SHA-256、大小和 provenance 见 [`tests/fixtures/historical/README.md`](../../../../tests/fixtures/historical/README.md)。
- 构建目录 `target/`：约 **2.0 GiB**。
- classfile 性质测试明确设置 `failure_persistence: None`，避免在工作区写入持久失败数据库；MUTF-8 256 cases，任意 0–64 byte 指令向量 512 cases。

这些测试覆盖当前声明的结构和边界，但**不证明**完整 classfile dialect validation、JVM verification、X1、resolution、runtime selection、nested archive 递归、CFG/SSA/IR、Java recovery、反编译质量或生产 JVM 依赖。oracle 只对 `tableswitch`、`lookupswitch` 和 CP-bearing fixed-width 指令的 instruction boundary 做受控交叉检查；它保持 test-only、默认 ignored。

## 3.4 candidate 的本地实测证据

父 agent 在当前未提交 candidate 上设置 `CARGO_BUILD_JOBS=1`、`CARGO_INCREMENTAL=0`、`RUST_TEST_THREADS=1`，同一时间只运行一个 Cargo 命令，实际得到：

- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`：通过。
- `PROPTEST_RNG_SEED=5350648285461741569 cargo test --workspace --all-targets --all-features --locked` 与 seed `5350648285461741570`：两轮均通过；每轮 **72 lib + 9 engine + 1 historical + 8 CLI = 90 passed**，另有 **1 个 oracle ignored**；example target 亦完成编译测试。
- `cargo test --test jvm_bytecode_oracle --locked -- --ignored --exact jdk25_instruction_boundaries_match_public_bytecode_inspection --nocapture`：**1 passed**；PATH 上 OpenJDK runtime `25.0.4+7`，fixture SHA-256 和 scope 与 3.3 记录一致。
- `cargo run --example inspect_class_header --locked -- tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class`：通过，输出 `HistoricalControlFlow`、`52.0`、3 个方法、`NotPerformed`；Header 路径 `code_bytes=0`。
- README 中的完整 JSON CLI 请求实际运行成功：`status="ok"`、bytecode execution `complete`，文件长度与 `transport.response_bytes` 均为 **2801**。
- `cargo +1.88.0 check --workspace --all-targets --locked`：通过，验证 MSRV 1.88.0；全程单作业。
- feature tree 同时包含 `blake3 feature "pure"` 与 `flate2 feature "rust_backend"`；normal production tree 未出现 CI 门禁列出的 async、图 IR、JVM、网络或数据库 runtime 依赖。
- `.github/workflows/ci.yml` 经官方 `actionlint 1.7.12`（已校验 release SHA-256）检查通过；workflow 内 Cargo 命令串行且设置低内存环境。修正首次 run 暴露的 JDK 版本语法后，第二次 remote run 的 stable、MSRV 与 supply-chain 三个 job 全部通过，建立 Linux x86_64 证据。
- 使用官方预编译 `cargo-deny 0.20.2`（与 `EmbarkStudios/cargo-deny-action@v2.1.1` 一致）执行 `--offline --locked --all-features check`：`advisories ok, bans ok, licenses ok, sources ok`。直接 git clone RustSec 数据库两次因外部 GitHub TLS/连接失败，故本地门禁改用 GitHub API 固定 RustSec commit `e2e640471715167f73e22eaf761f2e547adafeec`；下载 tarball SHA-256 为 `153f4ac096ad6981380884715cfa63acafe4189f48fe2f8c4ed5a64343850167`。四个未遇到的可接受许可证仅产生 warning，不影响门禁结果。
- `openspec validate --all --strict --no-interactive`：**6 passed / 0 failed**；`git diff --check`：通过。
- 本轮验证结束时 `target/` 的 `du` 大小为 `2,401,866,261` bytes，未超过 10 GiB 上限；remote CI 通过后执行 `cargo clean`，Cargo 报告移除 **7,737 files / 2.6 GiB**，工作区 `target/` 已清空。

## 首次 remote run

- GitHub Actions run ID：[`35168515529`](https://github.com/LordCasser/jarde/actions/runs/35168515529)。
- `MSRV 1.88.0`：success；`supply chain`：success。
- `stable / test and specification` 在任何 Cargo 步骤前，于 `actions/setup-java` 解析 Temurin 版本时失败：`25.0.4+7` 不被解析器接受。
- 这是 CI 配置失败，不是代码测试失败；该 run 本身没有提供 Linux x86_64 stable 测试通过证据，后续通过的 run 见下节。
- 精确修正：所有 checkout 更新为 `actions/checkout@v7.0.1`；setup-java 更新为 `actions/setup-java@v6.0.1`，并将 `java-version` 改为 `"25.0.4+7.0.LTS"`（runtime 仍为 `25.0.4+7`）；setup-node 更新为 `actions/setup-node@v7.0.0`，继续安装 Node 22，并设置 `package-manager-cache: false`，避免无 `package.json` 时自动缓存。

## 通过的 remote run

- GitHub Actions run ID：[`35168810088`](https://github.com/LordCasser/jarde/actions/runs/35168810088)，对应 commit `b49271776caed081329af2a144134c2f7a762cce`，结论 **success**。
- `stable / test and specification`：全部步骤 success，包括 stable fmt、clippy、两个固定 proptest seed 的全 workspace tests、显式 JDK 25 oracle、公共示例、feature/normal dependency tree 门禁、OpenSpec strict validation 和 tracked diff 检查。
- `MSRV 1.88.0`：success；`supply chain`：success，后者实际构建并运行 `EmbarkStudios/cargo-deny-action@v2.1.1`。
- runner 为 Linux x86_64、固定 `ubuntu-24.04`。该 run 与本地 Linux aarch64 证据共同满足 3.4 的平台、CI、示例与实际测试记录门槛。
- 3.4 完成记录 commit `c659ee218370f22fe99e64675202abdba29f0c2c` 触发的 [run `35169222351`](https://github.com/LordCasser/jarde/actions/runs/35169222351) 亦为 **success**；stable、MSRV 1.88.0 与 supply-chain 三个 job 全部通过。

## 3.5 最终本地门禁

在 `cargo clean` 后的 commit `c659ee218370f22fe99e64675202abdba29f0c2c` 上，以 `CARGO_BUILD_JOBS=1`、`CARGO_INCREMENTAL=0`、`RUST_TEST_THREADS=1` 串行执行：

- `cargo fmt --all -- --check` 与 `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`：通过。
- 两个固定 proptest seed 的 `cargo test --workspace --all-targets --all-features --locked`：每轮 **90 passed / 0 failed / 1 ignored**。
- 显式 JDK 25 oracle：**1 passed**，runtime `25.0.4+7`、fixture hash 与 scope 保持不变。
- 公共 Header example：通过，输出 `HistoricalControlFlow`、classfile `52.0`、3 个方法、`NotPerformed`，且 `code_bytes=0`。
- `cargo +1.88.0 check --workspace --all-targets --locked`：通过。
- feature tree、normal production dependency boundary、官方 `cargo-deny 0.20.2 --offline --locked --all-features check`、OpenSpec 1.11.0 strict validation、actionlint 1.7.12 与 `git diff --check`：全部通过；cargo-deny 仍只有四个未遇到 allowlist license warning，四类门禁均为 `ok`。
- 清洁构建后的 `target/` 为约 **671 MiB**；验证期间最低记录仍有约 **7.0 GiB available memory**，未并行运行 Cargo，也未触发 OOM。归档完成后再次执行 `cargo clean`，Cargo 报告移除 **1,866 files / 819.2 MiB**（`du` 前值 `697,925,932` bytes），`target/` 已清空。

## 归档结果

- `openspec archive establish-p0-foundation --yes`：成功；13/13 tasks complete。
- 已创建主规格 `openspec/specs/analysis-contracts/spec.md`、`artifact-snapshots/spec.md`、`classfile-inspection/spec.md`，共应用 **13 added requirements**。
- change 归档路径为 `openspec/changes/archive/2026-09-17-establish-p0-foundation/`；归档后 `openspec validate --all --strict --no-interactive` 覆盖 3 个主规格与 5 个后续 active changes。

低内存验证必须继续串行。独立 GitHub jobs 可使用不同 runner 并行，但每个 runner 不并行启动 Cargo。
