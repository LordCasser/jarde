# 验证记录

固定提交见仓库历史（实现提交紧邻本文件归档提交之前），门禁由本机执行、随后由该提交的 CI 复核。

## 契约实现

**定向 container 访问**（`crates/jarde-reader/src/artifact.rs`，新公开入口两个，其余 `pub(crate)`）：

- `ArtifactSnapshot::container_candidates(&ContainerOrigin, raw_name, budget)`：调用方声明「哪个 container + 要查哪个 raw name」。内部 `check_container_origin` 校验 snapshot、root container 与每一步 derivation（重算 `derive_child_container` 并比较）；`build_container_facts` 先建 ancestor，施加 `check_nested_depth`，要求父 entry 是候选，再经既有校验读取物化父容器并解析目录；`parse_container_directory` 现在是 `enumerate`/`enumerate_artifact_tree` 共用的唯一目录解析路径；`verify_local_against_record` 是 root 读取与保留目录读取共用的唯一本地头校验；`materialize_verified` 是唯一读取/CRC/size/输出计费路径。目录不完整时返回 `container_refusal`（预算/取消/损坏），**不返回前缀**，因此不会把未完成目录判成 Missing。
- `ArtifactSnapshot::container_record(&PhysicalEntryId, budget)`：按身份读取，供方法体读取使用，替代原先的整树枚举。
- providers：`zip_candidates` 构造 root origin 调用定向入口；`tree_candidates` 传声明 origin；`candidates_at` 给拒绝加位置标签；`listed_entry` 改用 `container_record`。**显式整树枚举未改**，仍报告全部（含损坏 sibling）。
- 定位表：`ContainerFacts { origin, backing, entries, wayfinders, names, weight }`，`names` 是 raw name → `entries` 位置（按中央目录顺序压入，重复项不合并），命中只 clone 被匹配记录，读取走 wayfinder，且仍重新校验本地头、span、descriptor、CRC 与 size。

**有界复用**（扩展现有 `FactsCache`，无第二个 store、无全局、无新注入路径，默认仍 off）：

- 产品 = identity + `Arc<ContainerFacts>` + weight；key = `{snapshot, ContainerOrigin, schema}`，`RuntimeProfile`/loader 顺序/prefix **不入**该 key（仍由 resolver 每次施加）。
- 容量 = `FactsCapacity { entries, retained_bytes }`，一个权重核算覆盖 backing、目录、名称表与既有 CP/Header；共享 backing 只计一次；满则拒绝插入（无 LRU、无驱逐、无新依赖），被拒绝的插入**不会**让当前请求重扫（零容量 store 实测解析 2 次目录而非 4 次）。
- 命中先 poll、按**当前** `NestedDepth` 与取消/时间/输出限制复核；命中不重放计费、不重置预算，且仍对实际发生的 class 读取计费（DEFLATE 场景：暖路径 `archive_entries 0 / read_bytes 55 / entry_bytes 55`，`class_headers 1`）。
- 复用与容量作为独立事实报告：`FactsReport` 增加 container 命中/未命中/咨询、`refused_capacity`、`refused_capacity_bytes`、`retained_bytes`；`FactsCache::clear` 释放持有引用（实测 entries/containers/retained_bytes 归零，随后请求重新解析）。

## 确定性计数（提交在 `tests/p5_container_lookup.rs`，fixture 按 digest 固定）

| fixture | 整树参照（同提交显式枚举） | 定向查找（零容量探针） | 冷+暖（保留 store） |
| --- | --- | --- | --- |
| `war_with_target(STORE)`：26 container、24 sibling | 26 次目录解析、**25** 次 nested 物化、7862 B | 2 / 2、720 B | 冷 2/1；同名第二次 **+0 / +0**，命中 3；同方法第二次 **+0 / +0**，命中 5 |
| 两层嵌套链 | — | 3 / 4 | 冷 3/2；暖 +0/+0 |
| flat JAR 4 sibling / 96 sibling | — | 1 次解析、**0** 次物化 | — |
| 重复 `p/S.class` ×2 | — | 2 候选、ordinal [0,1]、Ambiguous | 暖复用 +0/+0 |
| `war_with_target(DEFLATE)` | off → 冷 `archive_entries 28 / read_bytes 213 / entry_bytes 415` | — | 暖 0 / 55 / 55（class 读取仍计费） |

两种 fingerprint 都在 `tests/p5_container_lookup.rs` 中具名并同时保留：**semantic**（剔除 usage 与 elapsed，供 direct/cold/warm/容量不足四条路径对比）与 **full-report**（只剔除 elapsed，含 usage；同初始状态的两个 store 用同一冷请求预热后逐字段相等，并断言冷 ≠ 暖以免检查空转）。确定性验收四条全部断言：未搜索 sibling 物化为零、保留期间第二次查找解析与物化均为零、暖路径不再扫目录（零上限下 `archive_entries == 0`）、语义 fingerprint 相等。**未发布任何加速结论**。

## 负向证据（逐条实测）

伪造 origin → `child_container_mismatch`；不完整目录 → `budget_exceeded_archive_entries`（**不是** Missing），解析次数 1；未被搜索的损坏 sibling → 局部 Resolved，显式整树 `partial` + `entry_integrity`（`expected 0xb32ac0f0, got 0x911d94d0`，provenance 指向 ordinal 1 的 `WEB-INF/lib/L000.jar`）；暖路径传入被篡改 metadata → `entry_metadata_mismatch`，诚实 entry 仍读且 digest 相符；暖链上更低的 `nested_depth` → `budget_exceeded_nested_depth` 且 state ≠ Resolved；到期时钟 → `budget_exceeded_elapsed_millis`；预取消 → `Cancelled`，构建不写任何 container；已终止 Budget → 第二次调用既非 Resolved 也非 Complete；暖读输出预算 → 含 `OutputBytes`，`output_bytes` 用量 0；容量不足 → `refused_capacity_bytes ≥ 1` / `refused_capacity ≥ 1`。

## 计费口径变化（已披露，且是本次的目的之一）

`tests/p2_dispatch.rs::the_range_enumerates_each_name_once` 记录的 bill 由 **34 → 22**。原因：旧路径每次「定位一个名字」都产出一份没人要的整树 listing（每次计 5 entries + 2 containers + 1 nested 候选），现在按实际读到的 container 记录计费；该用例的后半段（少一个单位即在该候选处停止）保留，说明 pin 仍然卡在边界上。这不是放宽断言：改变的是「为没做的工作付费」这件事本身。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --test p5_container_lookup --locked` | 29 passed / 1 ignored |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1164 passed / 0 failed / 6 ignored** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（18.9 s） |
| `openspec validate --all --strict --no-interactive` | 24 passed / 0 failed（归档前） |
| 实现提交的 CI（`b22ea04`） | [run 35496369587](https://github.com/LordCasser/jarde/actions/runs/35496369587) **四 job success**：stable（fmt、clippy `-D warnings`、两轮固定 seed 全量测试、JDK 25 oracle、P3 编译执行对照、依赖边界、OpenSpec strict、`git diff --exit-code`）、MSRV 1.88.0、supply chain、fuzz smoke |

## 边界与偏差

- **STORED 未按「引用已验证 backing 的区间」实现**，而是每个 container 持有一份不可变 owned backing：校验性读取无论如何都要触碰字节，借用区间会把子 container 的事实与父 `Arc` 生命周期及「共享 backing 只计一次」的核算纠缠在一起。这是有意的取舍，不是遗漏。
- **目录/校验 schema 是编译期常量**，所以「schema 变化导致不命中」只能靠重新构建触发；测试改的是同类身份维度（registry）。
- **前缀语义不在本 change**：`container_candidates` 的入口形状是「container origin + 请求的 raw name」，`bind-prefixed-load-roots` 落地时由调用方在返回候选上施加 prefix 规则；迁移点已写在入口文档与支持矩阵里。
- **未测量进程 RSS**：retained weight 是驻留代理，没有任何 RSS 声明；触发复评的条件是「某行的资源报告由 container backing 主导」。
- 顺带记录、本次未改：resolver 的 *dispatch range* 仍枚举整树（`resolver.rs::search_coverage_with_artifact`）；`FactsCache` 的 CP/Header 权重仍是字节长度代理；`FactsReport` 计数是 per-store 而非 per-request（测试取增量）。
- 公开 API 增量：`FactsCapacity`、`CONTAINER_FACTS_SCHEMA`、`FactsCache::clear`、扩展的 `FactsReport`、`FactsCache::new/current/capacity` 签名迁移（一次迁移仓库调用点，无平行旧接口）、reader 两个方法，以及 `SnapshotId`/`ContainerId`/`ArchiveNameBytes`/`ContainerOriginStep`/`ContainerOrigin` 的附加 `Ord/PartialOrd` 派生。未新增 crate 或依赖，`Cargo.toml`/`Cargo.lock` 未改。
- fixture 全部由 rawzip writer 在测试内生成或内嵌既有 ECJ v52 样本，未新增二进制 fixture，因此未触发语料 fingerprint 重生成。
