# `reuse-selected-class-read` 的实现与验证记录

本文件是 change `reuse-selected-class-read` 的实现记录。契约、工作量、时间**三层分列**（§1–§4），
每条给可复跑的命令与实际读数。本变更**不作任何时间/吞吐主张**；唯一被准入的理由是工作量与契约
（`optimize-demand-workloads` §3.4 的决定与条件）。

工作树：`~/.grow/worktrees/projects-jarde/subagent-01a0c6fe…`（基线 `03a544b`，即主工作区 HEAD）。
所有读数都在该工作树的最终文件状态下取得；未提交（no git commit，按任务要求）。

## 1. 契约层：实现了什么

**一句话**：新增一个"定义读取"层——`FactsCache` 按**定义身份**（`PhysicalDefinitionId` + `DEFINITION_READ_SCHEMA`）
保留一次已验证读取的字节与它自己建立的 digest/length，并在同一 snapshot 的后续请求上作答；入口只有
`ArtifactSnapshot::{retained_definition_read, remember_definition_read}` 一对，消费点只有两处。

| spec delta | 落点 | 读法 |
| --- | --- | --- |
| `facts-cache` ADDED「A verified definition read is reusable within explicit bounds」 | `crates/jarde-reader/src/facts_cache.rs`：`DEFINITION_READ_SCHEMA`、`DefinitionReadKey`、`DefinitionRead`、`FactsCache::{definition_read, remember_definition_read}`、`FactsReport::{definition_reads, definition_read_bytes, definition_read_consultations, definition_read_hits, definition_read_misses, definition_read_stored}` | 只发布完整读取：`remember_*` 只在一处被调用，且都在 `read_entry_for_analysis` 返回 `Ok` 之后 |
| `facts-cache` MODIFIED「Complete cache identity and invalidation」（身份表增加定义读取层） | 同上；身份维度由 `DefinitionReadKey` 承载（snapshot / origin chain+ordinal+raw name / digest / length / variant / schema） | 任一维不同 = 另一个 key，不是"近似命中" |
| `facts-cache` MODIFIED「…does not grant a fresh budget」的 scenario 改述 | `crates/jarde-reader/src/artifact.rs` 的两个入口 + `crates/jarde-reader/src/inspect.rs::materialize_definition` + `crates/jarde-jvm/src/providers.rs::read_definition_class` | 命中仍执行：目录定位、变体核对、调用方声明身份的核对；只有"重读字节 + CRC/size + 摘要"被免 |
| `artifact-snapshots` MODIFIED「Bounded archive reading」的新入口 | `artifact.rs::{retained_definition_read, remember_definition_read}` | 命中返回该读取的字节与其建立的身份；不命中/未启用/容量拒绝读零字节、计零费用 |

**实现细节（都是可复跑的读数之外的结构事实）**

- **store 只持有"它自己就是该定义"的读取**：`remember_definition_read(definition, bytes, content_digest)`
  先比较 `definition.class_bytes` 与 `(content_digest, bytes.len())`，不相等就不保留（调用方紧接着的身份核对
  会拒绝这样的读取，所以这个状态没有任何 lookup 能命中）。因此"命中即返回该读取建立的 digest"是**结构
  保证**，不是调用方的承诺。
- **只发布完整读取**：`remember_*` 的两个调用点都在 `read_entry_for_analysis` 成功返回之后；取消、限额、
  损坏、校验未完成的读取在到达 store 之前就以 `Err` 结束（§2 R02）。
- **共用既有两层上限、满则拒绝**：`remember_definition_read` 与 `remember_container` 用同一对
  `capacity.entries` / `capacity.retained_bytes`，条目计数 = entries+containers+definition_reads；
  不淘汰、不驱逐；拒绝计入既有的 `refused_capacity` / `refused_capacity_bytes`。
- **命中不计费、不伪造证据**：命中路径不 charge 任何维度（`definition_read` 只 `budget.poll()`）；
  `read_bytes`/`entry_bytes`/`archive_entries` 因此不再发生；D0 端口只在**本次请求真的读了**时计
  `class_materializations`（`PreparedClassRead::retained()` + `materialize_definition` 的第三个返回值）。
- **入口只有两处**：`src/facade.rs` 的身份绑定读取（经 `inspect.rs::materialize_definition`，`class_view` /
  `class_source` / `list_members` 走它）与 `crates/jarde-jvm/src/providers.rs::read_definition_class`
  （方法驱动读取）。**名字搜索的候选读取不入此路径**：`facade.rs::read_class_declaration` 未改，
  `list_class_declarations` / `ClassRef::Name` 的读取直接走 `read_entry_for_analysis`（§2 R03 的 0 咨询读数）。
- **无开关、无并行路径**：产品里没有 env/feature 开关，也不存在"普通构建编译掉的旁路"；唯一读 env 的是
  测试自己的容量档参数（§3）。
- **回退**：无 store（`budget.facts_cache()` 为 None）→ 直接读；容量拒绝 → 直接读；身份不符 → 不同 key、
  直接读；取消/到期/预算终止 → `budget.poll()` 先拒绝（命中不恢复请求）；异声明条目（另一个
  format/registry 的 handle）→ discard + 计数 + 直接读。

**改动文件**：`crates/jarde-reader/src/{facts_cache,artifact,inspect,prepared}.rs`、
`crates/jarde-jvm/src/providers.rs`、`src/facade.rs`、`src/lib.rs`（再导出 `DEFINITION_READ_SCHEMA`）、
`tests/p5_definition_read_reuse.rs`（新）、`tests/p5_container_lookup.rs`、`tests/p5_bulk_corpus.rs`。
没有新依赖、没有 `unsafe`、没有新线程、没有全局状态、没有淘汰。

## 2. 工作量层：计数读数（这是本变更唯一的主张面）

新增验收目标：`tests/p5_definition_read_reuse.rs`（11 个用例，全绿）。完整日志见
[`definition-read-reuse-tests.log`](definition-read-reuse-tests.log)。

```text
cargo test --test p5_definition_read_reuse --all-features --locked -- --nocapture
# test result: ok. 11 passed; 0 failed; 0 ignored  (target 内所有用例共用一个 gate，串行读 D0 端口)
```

**R03 两臂等价（单因素）**：同一 snapshot 上六步序列（`class_view` ×2、`recover_method` ×2、`list_members`、
`class_view by name`），一臂不挂 store，一臂挂**只容得下定义读取、容不下容器产品**的 store
（`FactsCapacity::new(8, 1<<10)`：容器产品 2,694 B 被拒 → 两臂的容器层代价相同，唯一变量就是定义读取）。
逐步读数（log 中 `step N` 行）：

| step | direct 读取 | reuse 读取 | D0 `class_materializations` | 其余 12 个计数维度 |
| --- | --- | --- | --- | --- |
| 0 `class_view #1` | AE=5 RB=352 EB=532 | AE=5 RB=352 EB=532 | 1 → 1 | 全等 |
| 1 `class_view #2` | AE=5 RB=352 EB=532 | AE=4 RB=0 EB=0 | 1 → 0 | 全等 |
| 2 `recover_method #1` | AE=9 RB=352 EB=532 | AE=8 RB=0 EB=0 | 1 → 0 | 全等 |
| 3 `recover_method #2` | AE=9 RB=352 EB=532 | AE=8 RB=0 EB=0 | 1 → 0 | 全等 |
| 4 `list_members` | AE=5 RB=352 EB=532 | AE=4 RB=0 EB=0 | 0 → 0 | 全等 |
| 5 `class_view by name` | AE=5 RB=352 EB=532 | AE=5 RB=352 EB=532 | 0 → 0 | 全等 |

（AE=`archive_entries`、RB=`read_bytes`、EB=`entry_bytes`。step 1–4 的 AE 差 1 = 被省掉的 entry 定位扫描；
容器目录解析的 4 条记录在两臂都付，因为容器产品在**两臂**都被拒。）

- 结果等价：每一步 `normalized(document)`（去掉 `usage`/`elapsed_millis`/**`limits`**——请求自己的配置不是结果）
  逐字节相同；6/6 步相同，0 步不同。
- 工作不变：`class_bytes`、`attribute_bytes`、`class_headers`、`method_bodies`、`code_bytes`、`output_bytes`、
  `input_bytes`、`ir_items`、`ir_edges`、`analysis_steps`、`normalization_clones`、`result_items` 逐步相同；
  D0 端口的 `class_preparations`、`body_decodes`、`recovery_runs`、`owned_records`、`read_detail_records`
  逐步相同（用例断言 `Counts{ class_materializations: 0, .. }` 两臂相等）。
- 只有被停止重复的读取消失：`class_materializations` 4 → 1（step 0 的首读仍在）；被 store 回答的 4 步
  （1–4）恰好等于 `definition_read_hits`（用例断言两者相等）。

**R01 身份逐维**（`a_retained_read_answers_for_its_own_definition_and_for_no_other`）：
同一份 class 字节在同一 snapshot 的三个坐标上；先按真实定义 offer 一次读取，再逐维变异后 lookup——
`(同 raw name 异 ordinal, 同字节异 raw name, 同坐标异容器, 异 digest, 异 length, 异 variant)` 6 个变异 +
"同内容另一个真实坐标" 1 个，**7/7 未命中**；命中 1 次且返回的字节与身份分别是那次读取自己的
（`definition_read_stored=1, hits=1, misses=7`）。
跨 snapshot：两个只差一个无关 entry 的同坐标同 digest 归档，`hits 0 → 0 → 1`、`reads kept 2`
（第二个 snapshot 的读取**没有**被第一个的回答；回到第一个则命中）。

**R02 终止点**（`a_read_that_stopped_is_never_retained_and_a_read_that_finished_is`）：
`cancelled 0 / limit 0 / partial 0 / unverified 0`（四个各自的 store 都保留 0 条），对照组同几何的完整读取
保留 1 条、191 B（= fixture entry 大小）。错误码分别为 `cancelled`、`EntryBytes`、`entry_integrity`（流
中途损坏）、`entry_integrity`（CRC 不符）。

**R03 名字路径**（`the_name_search_reads_its_candidates_itself`）：两轮 listing + `ClassRef::Name` 视图下
`definition_read_consultations = 0`、`definition_read_stored = 0`，且候选读取仍照常计费（`read_bytes > 0`）；
同一 store 上的身份绑定读取 `hits 1/2`。名字路径的读取计数**未变**。

**R04 五条回退**（`every_fallback_reads_the_definition_itself`）：无 store / 容量拒绝（`none`：16 次拒绝、
0 命中、0 保留，读与直读逐维相同）/ 身份不符（另一个 snapshot 的定义：`hits` 不动、本请求自己读；
另外三个**变异**定义——异 digest、异 length、异 variant——请求层的错误码与无 store 臂**逐字相同**
（`definition_class_bytes_mismatch` / `definition_variant_mismatch`），且 `hits` 不动）/
取消（facade 拒绝 + **入口自身** `retained_definition_read` 也以 `cancelled` 拒绝，且不计命中）/
异声明条目（`over(FactsIdentity{format+1})`：discard 计数增加、`hits` 不动、结果与直读相同）。

**容量梯度**（`the_capacity_ladder_keeps_the_answer_and_states_what_it_refused`）：5 档 × 每个定义问两次，
5/5 档结果指纹与无 store 臂相同：

| 档 | 保留（定义读取 / 字节） | 命中 | 未命中 | 容量拒绝（entry/byte） |
| --- | --- | --- | --- | --- |
| `none` (0,0) | 0 / 0 | 0 | 8 | 16 / 0 |
| `e1` (1, 1 MiB) | 0 / 0（容器占掉唯一槽：2,694 B） | 0 | 8 | 8 / 0 |
| `tiny` (8, 1 KiB) | 2 / 723 | 2 | 6 | 0 / 12 |
| `medium` (8, 1 MiB) | 4 / 1813 | 4 | 4 | 0 / 0 |
| `roomy` (16 Ki, 128 MiB) | 4 / 1813 | 4 | 4 | 0 / 0 |

每档断言：`definition_read_bytes` 恰等于该档接受的读取之和、`retained_bytes ≥` 该值、不超过声明的
entry/byte 上限、`consultations = hits + misses`；被拒绝的请求以直读完成（`entry_bytes == 该定义大小`）。

**既有 bulk 语料门禁的对照读数**（`tests/p5_bulk_corpus.rs`，187 请求）：

```text
direct  requests=187 archive_entries=1915 entry_bytes=365912 read_bytes=346379 …
shared  requests=187 archive_entries=57   entry_bytes=18680  read_bytes=16476  …
store   10 entries / 10 containers / 19 definition reads (14771 bytes), 51706 retained bytes …
reuse   container 398/408 hits, directories parsed 10, nested materialized 4 (2473 bytes); definition reads 168/187 hits (19 stored)
```

`class_bytes / class_headers / method_bodies / ir_items / ir_edges / analysis_steps / code_bytes /
output_bytes / result_items` 两臂相同；只有读取维度下降（`archive_entries 1915→57`、`entry_bytes 365912→18680`）。

**反例（用例真的有牙）**：三个变异各自被真跑捕获，随后还原（还原后目标 11/11 绿，见 §5）：

| 变异 | 观测到的失败 |
| --- | --- |
| A：store 的 key 不再绑定 snapshot（key 里把 snapshot id 归一成常量） | `two_snapshots_of_one_class_do_not_answer_for_each_other` 红：`a definition read of another snapshot was answered from the first snapshot's retention: … hits 0 → 1` |
| B：命中时补计旧 usage（命中后再 charge `EntryBytes`+`ReadBytes`） | 5 个用例红；最直接两条：`a_tight_budget_stops_truthfully_with_and_without_a_hit` → `…fits its budget: BudgetExceeded { dimension: EntryBytes, limit: 190, consumed: 0, requested: 191 }`；`the_capacity_ladder_…` → `the tiny tier's request 1/0 was answered and still charged a read: ArchiveEntries=4 ReadBytes=532 EntryBytes=532` |
| C：命中路径不再 `budget.poll()` | `every_fallback_reads_the_definition_itself` 红：`a cancelled request is refused by the read entrance itself: Some(([…bytes…], ClassBytesId { … }))`——取消的请求被服务了 |

## 3. 紧预算、完成度与内存（R05 / 4.1 / 4.2）

**冷/热对照**（`a_tight_budget_stops_truthfully_with_and_without_a_hit`）：

```text
cold stop `EntryBytes` (entry_bytes 190 of 191); warm completion with the entry read waived [ArchiveEntries=0 ReadBytes=0 EntryBytes=0]
a hit under a spent item allowance: 0 item(s) published, stop `Partial { reason: BudgetExceeded { dimension: ResultItems }, … }`
```

- 冷臂（无 store、`entry_bytes = 定义大小-1`）：请求在读取处停止，**报出的维度就是导致停止的维度**（`EntryBytes`）。
- 热臂（同一紧预算 + 已存该读取的 store）：请求完成，且发布物与宽松预算臂逐字节相同；完成是**如实**的
  ——它这一次真的没有读 entry（三个读取维度全 0），其余每一步（header attempt 等）仍照常计费。
- 反伪造的一半：命中之后，被 `ResultItems` 限额停下的请求仍然报 `Partial{BudgetExceeded{ResultItems}}`
  且 0 条 item（限额取该请求自己"总 item 计费 − 已发布 item 数"，用例内自算，不写死数字）；
- 已耗尽的预算不被命中复活：同一 budget 的第二次 `class_view` 在 header attempt 处被 `BudgetExceeded{ClassHeaders}`
  拒绝，usage 停在 1。

**驻留与 RSS 分开报告**（`rss-residency-ladder.txt`，每档 3 个独立进程，`/usr/bin/time -l`）：

| 档 | 保留定义读取 / 字节（store 自报） | max RSS 三次（B） |
| --- | --- | --- |
| off（无 store） | — | 5,373,952 / 5,292,032 / 5,406,720 |
| none | 0 / 0 | 5,390,336 / 5,701,632 / 5,701,632 |
| e1 | 0 / 0（容器 2,694） | 5,308,416 / 5,292,032 / 5,406,720 |
| tiny | 2 / 723 | 5,357,568 / 5,357,568 / 5,357,568 |
| medium | 4 / 1813 | 5,308,416 / 5,292,032 / 5,324,800 |
| roomy | 4 / 1813 | 5,292,032 / 5,341,184 / 5,341,184 |

结论按"分开报告"读：`retained_bytes`/`definition_read_bytes` 是 **weight 代理**，不是 RSS；同一档的 3 次
RSS 离散度（最大 409,600 B）比最大的驻留（4,507 B）大两个数量级，且**不随驻留单调**（`none` 保留 0 却读到
最高值，`medium` 保留 4,507 B 却读到最低值）。本变更**不建立 retained_bytes→RSS 换算**，也不主张任何 RSS 差。
（`JARDE_REUSE_TIER` 只选择**测试进程**跑哪一档；库不读环境变量，没有产品开关。）

## 4. 时间层：未证实（本变更不作时间主张）

- 本轮**没有**做交错时间测量（本任务的验收只要求工作量与契约；任务同时规定"任何时间主张必须 ≥10 次交错
  并写在证据里，否则记未证实"）。因此时间层记 **未证实**。
- 准入文档给的上界仍然成立并照抄：窗口份额上界 **0.5%**（要 5% 需 13.8 ms），
  `optimize-demand-workloads` §3 的窗口读数（W1 275,427 → 275,186 µs）在离散度内。
- 本变更因此**只**主张工作量（§2）与契约（§1）。

## 5. 门禁（最终文件状态，真跑）

```text
cargo fmt --all                              # 无改动（cargo fmt --all --check 通过）
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings   # CLIPPY=0，零告警
cargo test --workspace --all-targets --all-features --locked                   # EXIT=0
                                              # 111 个 test binary / 1603 passed / 0 failed / 18 ignored
openspec validate --all --strict --no-interactive                              # 25 passed, 0 failed (25 items)
```

基线（本变更前，`03a544b`）为 110 binary / 1592 passed / 0 failed / 18 ignored；本变更新增 1 个目标、
11 个用例（1603 = 1592 + 11）。

## 6. 既有断言变化清单（有意改变 vs 回归）

**有意改变（3 处，全部可追溯到 MODIFIED delta；没有一处是为了让测试通过而放宽契约）**

1. `tests/p5_container_lookup.rs::the_four_paths_agree_on_the_semantic_fingerprint`
   旧：warm 臂 `read_bytes > 0`（"warm 什么都没读，就不可能是核对过的 class"）。
   新：warm 臂 `read_bytes == 0` **且** 发布的分析记录里最后一条读取的 definition == 请求所声明的那个
   **且** `store.report().definition_read_hits == 1`。理由：`facts-cache` MODIFIED 的 scenario 明示"读取
   本身可由该定义的已保留读取作答"。语义指纹断言（4 条路径相同）不变。
2. `tests/p5_bulk_corpus.rs::Billing::SHARED_ARM` 重新钉数：`archive_entries 411→57`、`entry_bytes 330368→18680`
   （`read_bytes` 346379→16476 在同一门禁的行内可读），旧值与被替换的理由写在常量文档里。该文件的
   "retention 只移动读取、不移动工作" 逐维断言全部保留并通过。
3. `tests/p5_bulk_corpus.rs` 的门禁打印多一行 `facts.reuse()`，使上述 168/187 命中数在运行输出里可核对。

**回归（0 处）**：其余所有既有测试未改一行断言而全绿（含 `p5_benchmark.rs` 的
"the_engine_has_one_disabled_facts_cache_and_no_index_or_scheduler" 源码守卫——本变更没有让任何新文件命名
`facts_cache`，也没有任何源码构造 store）。

## 7. 未完成 / 未测 / 边界

- 时间层未测（§4）：一次受信读取自身的耗时、命中路径的 `h`（查找/拷贝）分量、端到端窗口差——都未在本轮测量。
- `retained_bytes` → RSS 的换算未建立（§3）；本变更只报告 weight 与 RSS 两列。
- 紧预算完成度**确实会变**（§3 冷/热对照）：命中让一个紧预算请求走得更远，本轮把它作为读数如实报告，
  并断言不伪造 `Complete`、停止原因如实。这是准入文档的未确认项 ② 的显式复测。
- 相同内容的两个 snapshot 是否共享同一个 snapshot 身份仍未测（准入 §6 (e)）；本实现按身份逐维比较，
  不依赖该行为。
- 名字搜索的候选读取**不**入此路径（§2 R03）；bulk class task 的读取（`ArtifactSnapshot::prepared_class`）
  与 `resolve_symbol` 的候选读取同样不在本次两个入口之内——它们不是"被选中的定义读取"。
- 未使用额外测试专用观测口：读数来自既有公共面（`UsageSnapshot`、报告文档、D0 端口）与本次按契约新增的
  `FactsReport` 字段（它们在结果之外，不进 fingerprint）；`JARDE_REUSE_TIER` 只存在于测试内（§3）。
