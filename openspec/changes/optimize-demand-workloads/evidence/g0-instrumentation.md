# G0 1.3：工作计数、外部互斥阶段、三档消融、锁/有序等待与仪表对照

本文件报告的是**读数**，不是加速结论：所有表格都是中位数（括号内为样本组）
[`baseline-fixture-raw.jsonl`](baseline-fixture-raw.jsonl)（200 行 / 290 样本）与
[`baseline-bcprov-raw.jsonl`](baseline-bcprov-raw.jsonl)（180 行 / 270 样本）的原始样本算出，
每格 **n=10**、无事后挑选；口径与统计方法见 [g0-metrics.md](g0-metrics.md)。

## 1. 三个分开报告的平面（不混口径）

| 平面 | 来源 | 在本轮样本里的字段 |
| --- | --- | --- |
| **usage（本次请求的工作量额度账）** | 请求自己的 `Budget`/报告（`UsageSnapshot`）；bulk 用操作自己的总账 `BulkRecoveryReport::{usage, entry_usage, discovery_usage, method_usage, delivery_usage}` | `usage`、`bulk` |
| **cache 驻留与复用** | store 自己的 `FactsReport`（entries/containers/retained_bytes/hits/misses/refused/directory_parses/nested_materializations） | `cache` |
| **进程 RSS** | campaign 的外部包裹（`/usr/bin/time -l` 的 maximum resident set size），与进程 wall 一起写在行上 | 行级 `max_rss_bytes` |

三者**没有**被换算成彼此：`retained_bytes` 是驻留代理而不是 RSS（`FactsCapacity` 自己的文档如此），
RSS 只按进程报告，usage 不解释时间。示例：bcprov W6a discard 的 RSS 中位数 129→126→140→162 MiB
（1/2/4/6 worker），而同一批的 `usage.archive_entries` 恒为 2,569、`methods` 恒为 15,003。

## 2. 三档 sink 消融（同结果的丢弃 / JSON 编码到计数 sink / 编码写文件）

> §2/§3/§4 的中位数取自第一次运行（`baseline-*-raw-first-run.jsonl`）；第二次运行（`baseline-*-raw.jsonl`，
> 最终树构建）的契约读数与之完全相同，时间读数落在同机噪声内（见 [g0-metrics.md](g0-metrics.md) §6）。

`request` 段与 `window` 是同一进程内的中位数；`sink_*` 是**嵌套**读数（父阶段 `request`），
单位 µs：`sink_visit` 只计数并回答，`sink_encode` 是 `serde_json` 序列化，`sink_write` 是文件追加。

### bcprov（15,003 方法 / 19,865 记录 / 391.2 MB 编码量）

| sink | workers | request ms | window ms | first ms | encoded MB | written MB | sink_visit µs | sink_encode µs | sink_write µs | RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| discard | 1 | 3844.0 | 4113.7 | 2.54 | 0 | 0 | 310 | 0 | 0 | 129.2 |
| discard | 2 | 2922.1 | 3193.0 | 2.61 | 0 | 0 | 326 | 0 | 0 | 125.9 |
| discard | 4 | 2470.5 | 2739.6 | 2.67 | 0 | 0 | 338 | 0 | 0 | 140.0 |
| discard | 6 | 2143.7 | 2413.9 | 2.73 | 0 | 0 | 337 | 0 | 0 | 161.9 |
| encode | 1 | 4156.9 | 4434.7 | 2.58 | 391.18 | 0 | 313 | 307,480 | 0 | 131.8 |
| encode | 2 | 2929.8 | 3200.2 | 2.57 | 391.18 | 0 | 327 | 315,261 | 0 | 132.2 |
| encode | 4 | 2516.5 | 2786.3 | 2.63 | 391.18 | 0 | 334 | 327,252 | 0 | 136.8 |
| encode | 6 | 2228.1 | 2496.5 | 2.68 | 391.18 | 0 | 338 | 332,737 | 0 | 163.3 |
| write | 1 | 4311.4 | 4579.6 | 2.68 | 391.18 | 391.20 | 308 | 306,862 | 144,951 | 129.5 |
| write | 4 | 2616.8 | 2887.0 | 2.77 | 391.18 | 391.20 | 337 | 333,768 | 180,243 | 135.6 |

### fixture（183 方法 / 233 记录 / 2.84 MB 编码量）

| sink | workers | request ms | window ms | first ms | encoded MB | written MB | sink_visit µs | sink_encode µs | sink_write µs | RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| discard | 1 | 15.37 | 16.65 | 0.21 | 0 | 0 | 3 | 0 | 0 | 7.4 |
| discard | 6 | 11.41 | 12.70 | 0.56 | 0 | 0 | 4 | 0 | 0 | 8.9 |
| encode | 1 | 17.60 | 18.90 | 0.21 | 2.837 | 0 | 3 | 2,364 | 0 | 7.5 |
| encode | 6 | 12.27 | 13.56 | 0.56 | 2.837 | 0 | 4 | 2,768 | 0 | 9.0 |
| write | 1 | 19.26 | 20.61 | 0.33 | 2.837 | 2.837 | 3 | 2,369 | 1,452 | 7.5 |
| write | 6 | 13.48 | 14.81 | 0.62 | 2.837 | 2.837 | 3 | 2,757 | 1,651 | 9.0 |

**这三档同结果**，由 `the_three_sink_modes_publish_the_same_result` 断言：三档的 stream 指纹
（顺序 + 身份 + disposition + 正文 + 每记录语义指纹）逐字节相同，`records`/`methods` 相同，
并且 `written_bytes == scratch_file_bytes`（写出的行数就是文件内容）；`discard` 的
`encoded_bytes == 0` 且两个嵌套读数为 0，`encode` 只付编码、不付写盘。写盘落在 cargo 自己的
`CARGO_TARGET_TMPDIR`（`target/` 内），断言明确拒绝 `/tmp`。

**读法**：丢弃档给出操作自身（含它的窗口与交付）的成本上界，编码档加上序列化，写盘档再加上文件
I/O。在 bcprov 上编码占 391 MB → 8.4%（w=1）到 3.9%（w=6）的 request 段；写盘再加 ~4.2% / ~6.8%。
`first_result` 与 worker 数同向增长（2.54 → 2.73 ms，fixture 0.21 → 0.56 ms）：**首结果不是并行段**。

## 3. 锁等待与有序等待（观测口自己的读数）

bcprov W6a discard / 4 worker 的 `probe`（`--features test-support`，样本里的 `probe` 字段）：

| 观测 | 读数 | 说明 |
| --- | --- | --- |
| 总账锁等待（ledger 六站点 `waited_calls`/`waited_nanos`） | **0 / 0**（charge_discovery、charge_methods、charge_delivery、checkpoint、probe、depth 全部） | 按维度原子准入后，计量路径不再出现"等锁"；`charge_methods` 调用 12,450,327 次、临界区合计（`held`）17.7 ms、`checkpoint` 10.0 ms |
| 有序等待（window 站点） | `take_front` 12,837 次等待 / 合计 **2.216 s**（同期 request 段 2.470 s → ≈90%）；`take_task` 40,796 次等待 / 合计 5.029 s（线程时间） | 协调器等"最早活动类"的下一记录，是当前最大的可观测等待 |
| worker 生命周期 | 4 个 worker 合计存活 9.92 s、在类任务内 4.88 s | 约 51% 的 worker 时间在等窗口/等任务 |
| 交付本身 | `deliver_nanos` 95.2 ms、`sink_nanos` 90.9 ms（19,865 条） | discard 档下"交付 + 计数"约占 request 段 3.7% |

fixture W6a discard / 4 worker 同形（量级小）：ledger 等待同样为 0；`take_front` 141 次 / 7.97 ms，
`take_task` 389 次 / 15.73 ms，`sink_visit` 合计 3 µs。

**口径**：`ledger` 的 `waited_nanos`/`held_nanos` 是**采样**（每 64 条一条），`calls`/`refusals` 是精确的；
`window` 的等待是窗口自己信号上的 `wait_timeout`，不是"整次调用变慢"。所以本表只支持"哪一段在等"，
不支持把某个数字当总时长。W1–W4（单请求路径）**没有**共享总账，ledger 六个站点在这四个工作负载里
根本不存在——这不是"没测到"，而是它们没有共享总额度可等；这一点由 §5 的源码事实支持
（只有 `src/bulk.rs` 构造 `OperationLedger`）。

## 4. 仪表开/关对照（同一结果，三种构建/开关）

同一 bcprov W6a discard / 4 worker（中位数，方括号为 min/max，n=10）：

| 配置 | request ms | window ms | 相对 window | domain 指纹 |
| --- | --- | --- | --- | --- |
| `--features test-support` + `instrument=on`（挂 probe） | 2,470.5 [2,424.8, 2,510.5] | 2,739.6 [2,691.1, 2,788.4] | **+5.9%** | 1 个 |
| `--features test-support` + `instrument=off`（计数口在，probe 不挂） | 2,318.5 [2,278.2, 2,375.8] | 2,587.9 [2,544.6, 2,649.8] | **+1.4%** | 同 |
| 不给 `test-support`（计数口与 probe 均编译掉） | 2,285.9 [2,271.8, 2,384.0] | 2,552.7 [—] | 基线 | 同 |

挂 probe 与不挂 probe 的区间**不重叠**（2,424.8–2,510.5 对 2,278.2–2,375.8），而"计数口在但不挂"
与"编译掉"的区间互相覆盖（+1.4% 落在噪声里）。fixture 同形（4 worker discard）：
request 11.55 / 10.64 / 10.70 ms、window 12.79 / 11.95 / 11.96 ms。W2 导航（完全没有 probe 参与）
在三种配置下为 bcprov 288.6 / 290.6 / 285.6 ms、fixture 2.87 / 2.83 / 2.84 ms——差异都在各自 spread 内。

**所有配置的 `domain` 指纹完全相同**，`the_instrumentation_changes_no_domain_report` 断言这一点，
并同时断言：`instrument=off` 的样本 `probe` 为 `null`、`usage`/`counts` 文档里**不出现** harness 自己的
`phase`/`phases`/`ledger`/`startup`/`stage`/`stages` 键；feature-off 构建的样本 `counts` 与 `probe`
都为 `null`（计数口真的被编译掉了，不是被分支绕过）。

## 5. 嵌套阶段不重复归因，且阶段时间不进领域报告

**不重复归因**是靠结构而不是承诺：

- `Stages::phase` 在被测路径之外开始计时，并在下一个阶段开始前**断言**上一个已经结束
  （阶段重叠会 panic），因此四个顶层阶段是同一时钟上互不相交的区间；
  `window_micros` = 首阶段起点到末阶段终点，`unattributed_micros` = 窗口 − 各阶段之和，样本里两者都在。
- bulk 的"sink 自己的存活 + 编码 + 写盘"是**嵌套**读数（`nested[]`，`parent: "request"`），
  从不进入阶段总和；`the_stage_ledger_keeps_phases_disjoint_and_nested_stages_out_of_the_total` 直接测这两条规则
  （包括"嵌套段必须指名一个真实存在的父阶段"，否则该调用 panic），
  `the_probe_counts_every_delivered_record_and_window_call` 另外断言 `sink_visit ≤ request`。
- 上层读数不重复计入下层：编码在请求段内被测，`output` 段只测调用方把返回文档渲染成字节的部分；
  W2/W3/W4/W5 的 `output` 段因此只做序列化，不重放请求。

**阶段时间不进领域报告指纹**，两条独立证据：

1. **行为**：§4 的三个构建/开关组合给出**同一个** `domain` 指纹（报告去掉每个 `elapsed_millis` 之后的
   blake3），而阶段时间只出现在 harness 打到 stdout 的 sidecar JSON 行（`phases`/`nested`/`window_micros`/
   `unattributed_micros`）里；样本的 `usage` 文档里没有任何阶段键。
2. **源码守卫**：`the_domain_reports_carry_no_phase_timing` 扫描 `src/**` 与 `crates/*/src/**` 的
   `.rs`（>20 个文件），断言九个 harness 侧名字
   （`Stages`/`TopStage`/`NestedStage`/`window_micros`/`unattributed_micros`/`sink_visit`/`sink_encode`/
   `sink_write`/`first_result_micros`）**一个都不出现**在引擎源码里，并自检这些名字确实存在于
   harness 自身（守卫不是空转）。语料指纹面另有既有门禁：`cargo test --test p5_corpus_fingerprint` 与
   `p5_benchmark` 的源码守卫在本次改动前后都不变（本专项没有改 `tests/fixtures/**`、`fuzz/corpus/**`
   或任何产品源码）。

**仪器化开销与其边界**：probe 只统计它自己的站点，不改变任何发布的领域字段；它在 bcprov 4 worker
discard 档上值 +5.9%，因此 §2/§3 的**墙钟**结论一律以不挂 probe 的 run 为准，probe 只用于归因；
`counts`（D0 计数口）在同一档上值 +1.4%，落在噪声量级附近，故其读数只用于**计数**，不用于时间。

## 6. 变异：去掉一条观测/计数，相应用例变红（随后还原）

两次变异都在最终树状态前后各跑一次同一用例，证明这些用例真的在被观测的读数上失败，而不是
"看起来会失败"。变异只是临时修改，两次都已还原；还原后 `git status --porcelain` 只有本专项新增的
两个路径（`tests/p5_optimize_workloads.rs`、本 evidence 目录），产品源码零改动。

| # | 变异 | 用例 | 结果 |
| --- | --- | --- | --- |
| A | 删掉 `src/bulk/observation.rs::BulkProbe::delivery` 里的 `records_delivered` 自增（即交付计数口不再计数） | `the_probe_counts_every_delivered_record_and_window_call` | **FAILED**：`the port counted 0 deliveries and the sink was handed 233 records: one of the two is not observing the stream`；还原后 `ok`（11 个用例中 1 通过、10 过滤） |
| B | 删掉 harness `Measuring::account` 里 `encoded_bytes` 的累加（即编码档不再计编码字节） | `the_three_sink_modes_publish_the_same_result` | **FAILED**：`the encoding sink really serializes every record`；还原后 10 passed / 0 failed |

变异 A 作用在**产品侧观测口**上，变异 B 作用在**harness 自己的计数**上：两类读数都有一条能失败
的用例，而不是只有打印。
