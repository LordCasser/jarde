# G0 1.4：度量口径、10 次独立基线样本与三层结论

## 0. 口径声明（先说限制）

- 设计 §2 已**先于**本轮测量固定了每组要分开的指标与"至少 10 个独立重复、交错执行、不用 10 次
  的经验 p95"这一方法；本文件把它们落到具体样本上，**没有**在看到样本后调整任何主指标、统计方法
  或边界。各表的"界限"一列是**事后登记的宽松上限**（用于捕获爆炸式退化与内存失控），不是事前接受
  阈值；本轮不是一次优化实验，因此没有可被事后放宽的收益门槛。
- 机器**不空载**：每个 campaign 开始时记录 `os.getloadavg()`——fixture `[3.98, 4.47, 4.85]`、
  bcprov `[5.14, 5.18, 5.19]`（macOS 26.6.2 / arm64）。因此本文件的时间层只用**中位数与极值形状**，
  凡落进样本 spread 的差异一律记"未证实"，不写成功结论。
- OS page cache 未受控；fixture 的进程启动里含测试 harness 自己的初始化（见 §3 的启动列）。
- 所有样本一个不丢：`run-baseline.py` 只在进程失败时写 `failure` 行，摘要里单独计数。

## 1. 组、主指标、界限、样本量与统计方法

| 组 | 主指标（读哪个字段） | 退化/内存界限（事后登记的宽松上限） | 样本量与统计方法 |
| --- | --- | --- | --- |
| W1 | `first_result_micros`；`open`/`prepare`/`request`/`output` 四段 | 同 snapshot 的第二次请求不得与冷请求同量级；fixture RSS ≤ 16 MiB、bcprov ≤ 64 MiB | 每配置 **10** 个独立进程，中位数 + min/max（极值一并给出） |
| W2 | `sequence_micros` 与逐请求 `per_request_micros`；`class_headers_total` | 序列总长 ≤ 8×单请求；`class_headers_total == classes`；fixture RSS ≤ 16 MiB、bcprov ≤ 64 MiB | 同上 |
| W3 | `method_bodies_total`、`class_headers_total`、`counts.class_preparations`/`class_materializations` | 一批 body 的 `class_preparations ≤ 1`；逐方法请求的 `class_materializations == 方法数`；fixture RSS ≤ 16 MiB、bcprov ≤ 64 MiB | 同上 |
| W4 | 逐页 `page_micros`、`pages`、`items`、`scanned_items`、`has_more` | 小页与大页的 `items` 集合相同；`has_more==0` 才算耗尽；页数触 `PAGE_CAP=4096` 必须登记；fixture RSS ≤ 16 MiB、bcprov ≤ 96 MiB | 同上 |
| W5 | sweep 的 `window_micros` + 分类；round trip 的 `sequence_micros` 与 `cache.{hits,container_hits,directory_parses,refused_*}` | tiny 与 roomy 的 sweep stream 指纹与方法数相同；`refused_*` 只在 tiny 出现；fixture RSS ≤ 16 MiB、bcprov ≤ 256 MiB | 同上 |
| W6a | 冷进程到流结束：外部 `wall_seconds` 与进程内 `window_micros`；`first_result_micros`；`encoded_bytes`/`written_bytes`；`outcome_*` 分类 | 1/2/4/6 worker × 3 sink 的 stream 指纹、方法数、分类、`traversal_complete`、`final_delivered` 全相同；fixture RSS ≤ 16 MiB、bcprov ≤ 256 MiB | 每 (worker, sink) 配置 **10** 个独立进程，中位数 + min/max |

统计方法全表一致：**每个配置 10 个独立进程样本、按配置交错的顺序采集、报中位数与 min/max**。
不做别的事后统计，不报 p95（理由见 §5）。

## 2. 原始数据（保存位置与规模）

| 文件 | 行 | 样本 | 内容 |
| --- | --- | --- | --- |
| [`baseline-fixture-raw.jsonl`](baseline-fixture-raw.jsonl) | 200 | 290 | fixture（仓库内确定性 ZIP）20 个配置 × 10，**最终树构建** |
| [`baseline-bcprov-raw.jsonl`](baseline-bcprov-raw.jsonl) | 180 | 270 | bcprov 18 个配置 × 10，**最终树构建** |
| [`baseline-fixture-raw-first-run.jsonl`](baseline-fixture-raw-first-run.jsonl) | 200 | 290 | 同 20 个配置的**另一次重跑**（较早构建；该次还没有 outcome 分类字段，`status` 文本还是带引号的 `Debug` 拼写） |
| [`baseline-bcprov-raw-first-run.jsonl`](baseline-bcprov-raw-first-run.jsonl) | 180 | 270 | 同 18 个配置的另一次重跑（较早构建） |
| [`baseline-fixture-feature-off-raw.jsonl`](baseline-fixture-feature-off-raw.jsonl) | 40 | 40 | 同 4 个配置，**不启用 `test-support`** 的构建（仪表对照与"同配置重跑"） |
| [`baseline-bcprov-feature-off-raw.jsonl`](baseline-bcprov-feature-off-raw.jsonl) | 40 | 40 | 同上，bcprov |

行内含：`wall_seconds`、`max_rss_bytes`、`cpu_seconds`、`exit_code` 与**原样**的样本 JSON
（`phases`/`nested`/`numbers`/`usage`/`bulk`/`counts`/`cache`/`probe`/`domain`）。摘要文件
`baseline-*-summary.txt` 是脚本当时打印的表，不是第二份数据。

## 3. 摘要（中位数，方括号为 min/max；`n=10`）

> 本节的中位数取自**第一次**运行（`baseline-*-raw-first-run.jsonl`）；第二次运行（§2 里引用的
> `baseline-*-raw.jsonl`，最终树构建）的同表读数落在 §6 的比值区间内（中位比值 0.975（fixture）/
> 0.988（bcprov）），而且两轮的契约读数完全相同。两份都在本目录，读者可以自己重算两遍。

### fixture（24 类 / 183 方法 / 2.84 MB 编码量）

| 样本 | window ms | wall ms | RSS MiB | 主读数 |
| --- | --- | --- | --- | --- |
| `w1-cold-method` | 2.16 [2.00, 3.22] | 8.8 | 6.0 | `first_result` 169.5 µs；open 19 µs / prepare 890 µs / request 170 µs / output 93 µs |
| `w1-second-request` | 0.12 [0.11, 0.17] | ” | ” | `first_result` 62.5 µs |
| `w1-second-snapshot` | 1.77 [1.71, 1.88] | ” | ” | 第二个 snapshot 的 open 13.5 µs / prepare 791 µs |
| `w2-navigation-sequence` | 2.83 [2.66, 2.89] | 7.3 | 5.0 | `sequence` 213.5 µs / 8 请求，`class_headers_total` 8 |
| `w3-all-bodies-one-request` | 2.18 [2.06, 2.24] | 8.6 | 7.0 | `counts`: materializations 1、preparations 1、body_decodes 8 |
| `w3-per-method-requests` | 1.16 [1.09, 1.22] | ” | ” | `sequence` 720.5 µs / 8 请求；materializations 8、preparations 0、recovery_runs 8 |
| `w3-fixed-set-across-classes` | 0.93 [0.92, 0.99] | ” | ” | `sequence` 518 µs / 8 类各一方法 |
| `w4-small-page` | 2.48 [2.46, 2.58] | 7.8 | 5.2 | 24 items / 13 页（`max_items=2`，到耗尽） |
| `w4-large-page` | 0.42 [0.41, 0.48] | ” | ” | 24 items / 1 页（`max_items=64`） |
| `w4-multi-consumer` | 0.39 [0.39, 0.42] | ” | ” | 4 consumer，items 与单 consumer 相同 |
| `w4-abandon-after-one-page` | 0.03 [0.02, 0.03] | ” | ” | 1 页 2 items 后丢弃 cursor |
| `w5-sweep` (roomy / tiny) | 14.25 / 15.23 | 21.4 / 22.0 | 7.9 | 183 方法、233 记录、24 类；tiny 的 `refused_capacity`>0 |
| `w5-round-trip` (roomy / tiny) | 1.52 / 1.65 | ” | ” | `sequence` 417 / 480.5 µs / 10 请求 |
| `w6a-export` discard 1/2/4/6 | 16.65 / 13.91 / 12.79 / 12.70 | 22.2 / 19.4 / 18.2 / 18.1 | 7.4–8.8 | `first_result` 208→560 µs |
| `w6a-export` encode 1/2/4/6 | 18.90 / 14.42 / 13.42 / 13.56 | 24.2 / 19.8 / 18.8 / 19.0 | 7.4–8.8 | `encoded_bytes` ≈2,836,71x（10 个样本极差 9 B） |
| `w6a-export` write 1/2/4/6 | 20.61 / 15.09 / 14.44 / 14.81 | 26.7 / 21.0 / 20.3 / 20.6 | 7.4–8.8 | `written_bytes` ≈2,836,94x（极差 8 B） |

### bcprov（2,430 类 / 15,003 方法 / 391.2 MB 编码量）

| 样本 | window ms | wall ms | RSS MiB | 主读数 |
| --- | --- | --- | --- | --- |
| `w1-cold-method` | 289.00 [285.50, 298.87] | 583.8 | 25.1 | open 2.87 / **prepare 267.6** / request 1.42 / output 0.10 ms |
| `w1-second-request` | 0.12 [0.11, 0.16] | ” | ” | request 0.07 ms（同 snapshot、同 store） |
| `w1-second-snapshot` | 287.23 [283.86, 293.12] | ” | ” | 第二个 snapshot 的 prepare 266.0 ms |
| `w2-navigation-sequence` | 288.58 [286.42, 299.80] | 295.1 | 19.8 | `sequence` 1.37 ms / 8 请求 |
| `w3-all-bodies-one-request` | 289.68 [283.96, 296.94] | 297.3 | 21.5 | `counts`: materializations 1、preparations 1、body_decodes 3 |
| `w3-per-method-requests` | 0.92 [0.89, 1.05] | ” | ” | `sequence` 726.5 µs / 3 请求；materializations **3**、preparations 0、recovery_runs 3 |
| `w3-fixed-set-across-classes` | 0.97 [0.95, 1.05] | ” | ” | `sequence` 752.5 µs / 8 类各一方法 |
| `w4-small-page` | 841.02 [821.87, 877.83] | 1407.1 | 66.2 | 1,094 items / **548** 页（`max_items=2`） |
| `w4-large-page` | 274.43 [268.30, 281.39] | ” | ” | 1,094 items / 18 页（`max_items=64`） |
| `w4-multi-consumer` | 273.24 [267.93, 278.30] | ” | ” | 4 consumer，1,094 items（与单 consumer 相同） |
| `w4-abandon-after-one-page` | 0.05 [0.05, 0.06] | ” | ” | 1 页 2 items |
| `w5-sweep` (roomy / tiny) | 3148.86 [3028.68, 3245.47] / 3137.01 [3031.46, 3260.13] | 3244.9 / 3253.9 | 122.2 / 123.6 | 15,003 方法、19,865 记录；分类见 §4 |
| `w5-round-trip` (roomy / tiny) | 17.50 [17.18, 18.20] / 43.40 [42.37, 45.83] | ” | ” | `sequence` 526 µs（roomy）vs 26,104 µs（tiny）/ 10 请求 |
| `w6a-export` discard 1/2/4/6 | 4113.65 / 3192.98 / 2739.61 / 2413.88 | 4192 / 3269 / 2815 / 2489 | 122–160 | `first_result` 2.54→2.73 ms |
| `w6a-export` encode 1/2/4/6 | 4434.70 / 3200.20 / 2786.28 / 2496.47 | 4512 / 3277 / 2860 / 2572 | 125–161 | `encoded_bytes` ≈391,181,1xx（极差 475 B） |
| `w6a-export` write 1 / 4 | 4579.61 / 2887.01 | 4672 / 2986 | 127 / 132 | `written_bytes` ≈391,201,0xx（极差 415 B） |

**启动（外部 wall − 进程内 window 之和）**：fixture 的 W1 进程 ≈ 4.7 ms（其中绝大部分是测试
harness 自己的初始化，不是产品度量）；bcprov 的 W1 进程 ≈ 6.3 ms，而那一个进程里的三段 window
合计 577.5 ms。**这一列不与其它工具或其它 harness 的启动比较**，它只是把残差写出来。

## 4. 三层结论

**契约层（跨配置必须相同，全部成立）**

- W6a 全 12 个 fixture 配置与 10 个 bcprov 配置：stream 指纹（顺序 + 身份 + disposition + 正文 +
  每记录语义指纹）、`methods`/`methods_declared`、`outcome_*` 分类、`traversal_complete=1`、
  `final_delivered=1` 完全相同；`the_worker_sequence_publishes_one_result` 与
  `the_three_sink_modes_publish_the_same_result` 把其中 fixture 的部分钉成用例。
- bcprov 的分类恒为 produced 12,045 / explanation_only 2,447 / no_body 508 / not_produced 3 /
  refused 0 / oversized 0，状态 `partial`；fixture 为 `complete`。**这是运行自己的读数**：不能把
  `partial` 读成缺陷，也不能把 `complete` 读成覆盖更广。
- W4 两种页大小的 items 集合相同（1,094 / 1,094；fixture 24 / 24），多 consumer 与单 consumer 的
  items 相同。
- W5 两种 store 容量下 sweep 的 stream 指纹与方法数相同；round trip 的**结果**指纹相同（去掉了
  每个 `elapsed_millis` 与 `usage` 记录）。
- 仪器化不改变任何一条：§[g0-instrumentation.md](g0-instrumentation.md) §4 的三种构建/开关给出同一 `domain`。

**工作量层（计与量，不是时间）**

- W3（bcprov）：一个类的整批 body 一次请求 = `class_materializations 1` + `class_preparations 1` +
  `body_decodes 3`；**逐方法请求 3 次 = materializations 3 + preparations 0 + recovery_runs 3**。
  即同一类的字节在普通单方法入口里按请求重复物化，而批量入口只物化一次——这是 O2/2.2 要跟踪的
  剩余重读的第一条计数证据（fixture 同形：1 + 1 + 8 对 8 + 0 + 8）。
- W1（bcprov）：同一 snapshot 上第二次单方法请求的 request 段 0.07 ms，冷请求 1.42 ms；`prepare`
  （类声明清单）267.6 ms 是两者共有的、打开后一次性支付的发现成本——它**不属于**任何单方法请求。
- W5（bcprov）容量消融：tiny（1 entry / 4 KiB）与 roomy 的 sweep 段相同（3,137 / 3,149 ms，spread
  互相覆盖），差别出现在**复用**上：round trip 10 次请求 26,104 µs（tiny）对 526 µs（roomy），
  `directory_parses` 31 对 11，`container_hits` 0 对 17,538。容量不足的代价是"重复解析目录"，
  不是"整次扫描更慢"。
- W6a 三档（bcprov，request 段）：丢弃 3,844→2,144 ms（1→6 worker）、编码再加 313→84 ms 的
  `sink_encode`、写盘再加 145→180 ms 的 `sink_write`；`sink_visit` 只有 310–338 µs。编码与写盘都
  不改变结果，只是把成本叠在交付上。
- W4（bcprov）：同样 1,094 items，`max_items=2` 需 548 页、request 段 566 ms；`max_items=64` 需
  18 页、274 ms。小页的**续扫代价**可测，且不是线性于 items（更多次 cursor 重放）。
- 锁/有序等待（见 [g0-instrumentation.md](g0-instrumentation.md) §3）：总账六站点等待为 **0**；
  最大可观测等待是协调器的 `take_front`（bcprov 4 worker 下 2.216 s ≈ request 段的 90%）。

**时间层（只报形状，凡落进 spread 记未证实）**

| 主张 | 观测 | 判定 |
| --- | --- | --- |
| W6a 并行改善吞吐（bcprov discard） | 1→4 worker：4,113.65 → 2,739.61 ms，极值区间不重叠（4,003–4,231 vs 2,691–2,788） | **方向成立**（形状）；绝对值受机器负载影响，不作吞吐结论 |
| 6 worker 相比 4 worker | 2,413.88 vs 2,739.61 ms（区间不重叠） | 方向成立；未见 jobs=8 那样的回退（本轮未测 8） |
| 编码占 request 段 | +8.4%（w=1）→ +3.9%（w=6） | 观测值；未做显著性检验，作"形状" |
| 写盘再叠加 | +4.2%（w=1）/~+6.8%（w=4） | 同上（write 档样本噪声最大：4,486–5,213 ms） |
| W5 tiny vs roomy 的 round trip | 26.1 ms vs 0.53 ms（≈49×），区间不重叠 | 数量级差异成立；**不**外推为其它语料的加速比 |
| 仪表化开销 | 挂 probe +5.9%（bcprov 4 worker discard）；只带计数口 +1.4% | 与 bulk 子项记录的量级一致；故时间结论一律取不挂 probe 的 run |
| W2/W3 的序列时间、W1 冷请求时间 | fixture 0.2–0.7 ms、bcprov 0.7–1.4 ms | **未证实**（远小于同机噪声带宽与进程启动开销） |
| fixture 上的 1/2/4/6 worker 差异 | 16.65→12.70 ms（discard） | **未证实**（fixture 太小，固定在 10 ms 量级的开销与机器负载同级） |

## 5. 尾延迟：**未证实**

本轮每组 10 个样本，**不足以**给出任何 p95/p99：10 个样本的经验 95 分位没有可用的置信区间，
且机器 load average 4–5、page cache 未受控。因此本文件**不报**尾延迟数字，也不把 `max` 当尾延迟
证据（`max` 只是 10 个样本里的最大值，已在表中给出）。要对尾延迟作主张，须按 design §2 另设足够
样本量并重新声明口径。

## 6. 重跑与复现

**同配置重跑（整个 campaign 各跑两遍）**：两次的**契约读数**逐项相同——29 个 fixture 配置与 27 个
bcprov 配置的 `domain` 指纹各只有 1 个值且两轮相同；两轮都有的契约字段（fixture 70 个、bcprov 218 个
比较项，含 `methods`/`records`/`items`/`outcome_*`/`traversal_complete`/`final_delivered`）**0 处不同**。
不同的只有两类读数：**时长**（中位数比值 0.83–1.00（fixture）/ 0.96–1.09（bcprov），即同机噪声），
以及**长度类字节计数**（`encoded_bytes`/`written_bytes`/`returned_bytes` 差几字节到几十字节）——后者是
因为每条记录/每个报告的 JSON 里内嵌该次运行的 `elapsed_millis`，位数变化即字节数变化，与 bulk 子项
记录的白名单是同一条（`elapsed_millis` 及其直接的字节后果；长度不属语义指纹）。

同一配置在 feature-on 与 feature-off 两个构建下也各重跑了 `w2`/`w3`/`w6a` 四个配置（各 10 样本），
两轮的 `domain` 指纹与计数完全一致，见 §2 的两个 `*-feature-off-*` 文件。

```text
# 一个样本（一个进程）
cargo test --release --test p5_optimize_workloads --features test-support --no-run --message-format=json   # 取 executable
JARDE_OPTIMIZE_WORKLOAD=w6a JARDE_OPTIMIZE_WORKERS=4 JARDE_OPTIMIZE_SINK=discard \
  <executable> --exact measure_workload --ignored --nocapture

# 整轮基线（脚本自己构建并取可执行文件）
python3 evidence/run-baseline.py --tag fixture --repeats 10 --features test-support \
    --out baseline-fixture-raw.jsonl --run w1: --run w2: --run w3: --run w4: \
    --run "w5:capacity=tiny:workers=2:sink=discard" --run "w5:capacity=roomy:workers=2:sink=discard" \
    --run "w6a:workers=1:sink=discard" ... --run "w6a:workers=6:sink=write"

python3 evidence/run-baseline.py --tag bcprov --repeats 10 --features test-support \
    --out baseline-bcprov-raw.jsonl --run "w1:artifact=<bcprov 路径>" ...   # 全清单见 summary 文件
```

## 7. 与其它证据的关系（不重复认领）

- bcprov 的整包并行、窗口与总账归因的**实现**证据在
  [add-parallel-bulk-recovery/evidence](../../add-parallel-bulk-recovery/evidence/)（其中
  `cost-attribution.md` 的 jobs 表与本文件的 W6a 表是**两次独立的测量**：不同 harness、不同会话、
  同机不同负载，数字不可互相"对齐"只可互相印证形状）。
- 需求驱动的证据选择与按需路径在本轮为 W3/W4 的对照提供背景，其实现在
  [add-demand-driven-core-results](../../add-demand-driven-core-results/)。
- 跨工具（jadx）测量仍是 `openspec/benchmark-protocol.md` 的事，本轮没有跑，也不得从本文件推出。
