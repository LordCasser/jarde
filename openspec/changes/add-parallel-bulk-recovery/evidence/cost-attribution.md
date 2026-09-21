# Where a whole-scope run's time goes (2026-09-21, one machine, release build)

Produced by `examples/bulk_scope_sweep`, whose three sink modes pay exactly one more layer
each: `discard` counts records only, `encode` serializes every record with the same
`serde_json` the CLI uses and drops it, `write` appends the encoded line to a scratch file.
The counted work is identical in all three (the counters printed beside each row), so the
columns differ only in what the sink is asked to do.

| corpus / mode | jobs=1 | jobs=2 | jobs=4 |
| --- | --- | --- | --- |
| bcprov discard (2,430 classes, 14,495 bodies, `ir_items` 11,025,007) | 3.26 s | 3.50 s | 3.84 s |
| bcprov encode | 3.55 s | 3.60 s | 3.90 s |
| bcprov write | 4.78 s | 4.23 s | 4.95 s |
| s2-009 discard (53,247 bodies, `ir_items` 32,966,465) | 11.16 s | 11.21 s | 12.23 s |
| s2-009 encode | 12.93 s | 11.63 s | 12.75 s |
| CLI `export`, median of 10 interleaved runs (writes 357 MB / 2.37 GB) | 4.41 / 19.31 s | 4.44 / 15.71 s | 4.65 / 16.90 s |

Read alongside `raw-timing-interleaved.jsonl` (CLI, ten interleaved samples per
configuration) and `raw-timing-first-pass.jsonl` (the earlier three-sample pass).

What the table supports:

* **The parallel loss is inside the operation.** With a sink that only counts, four workers are
  slower than one on both corpora, and the counters are identical — so encoding, writing and the
  stream's backpressure are not what removes the gain. The earlier attribution to "single-threaded
  delivery" is refuted by these rows.
* **Encoding is cheap; writing is not.** Serializing the whole stream costs 0.3 s (bcprov) and
  1.5 s (s2-009); writing 357 MB / 2.37 GB to a file costs about another 1.5 s / 6 s and is the
  noisiest column.
* **A caller without a store pays for the walk.** The same s2-009 scope run through the example
  without a facts store did not finish inside a 30 s wall clock (its counters show 5.7-6.6M
  `archive_entries` instead of 10,358): a container's directory is re-parsed for classes read
  after the walk moved on. The CLI attaches one by default and the example now does too.

Not supported by this table: any statement about other machines, about page-cache state (never
controlled here), or about jadx. Single samples per cell.

## 第二轮：总账锁是热点，改成按维度原子后并行符号翻转

观测口：reader 的 `LedgerObserver`（每条 entry 计数、每 64 条采样 timing）+ root 侧 `BulkProbe`
（`test-support` 门控，经 `BulkRecoveryRequest::with_probe` 挂载；报告与流事件不含观测字段）。探针
构建相对纯构建约 +7–9%，因此墙钟结论一律取纯构建，探针只用于归因。

归因（release、空载、每语料先温一次 page cache、3 次中位数、`discard` 模式；两次运行的 entry 数
完全相同：bcprov 29,789,133、s2-009 81,504,368）：

| 语料 | jobs | ns/entry（waited+held） | 临界区合计（串行下界） | 等待合计（线程时间） | 窗口 wait 合计 | 墙钟 |
| --- | --- | --- | --- | --- | --- | --- |
| bcprov | 1 | 60 | 1.23 s | 0.57 s | 0 | 3.79 s |
| bcprov | 2 | 119 | 2.13 s | 1.41 s | 7.01 s | 4.08 s |
| bcprov | 4 | 180 | **2.91 s** | 2.41 s | 15.37 s | 4.47 s |
| s2-009 | 4 | 178 | **7.91 s** | 6.64 s | 47.02 s | 13.69 s |

即：`Budget::charge` 一次调用进两次临界区（checkpoint + charge），entry 自身耗时随 worker 数从
60 ns 涨到 180 ns（跨核搬同一行 cache），且这段是**串行**的；窗口与有序等待是下游症状。

优化后（同会话交错、纯构建、3 次中位数）：

| 语料（discard） | jobs=1 | jobs=2 | jobs=4 | jobs=8 |
| --- | --- | --- | --- | --- |
| bcprov before → after | 3.55 → 3.34 | 3.90 → **3.12** | 4.17 → **3.12** | 4.44 → 3.29 |
| s2-009 before → after | 11.48 → 10.89 | 11.43 → **9.72** | 12.63 → **9.94** | 13.35 → 10.61 |

总账自身 180 → 43 ns/entry（jobs=4），`held` 合计 2.91 → 1.29 s（bcprov）、7.91 → 3.45 s（s2-009），
`waited` 归零；entry 数、计费表、1/N 等价、排序、背压上界、取消与单账停止归属全部不变。jobs=8 比 4
慢 5–7%，说明继续加深窗口当前无益。

**下一瓶颈（未优化）**：窗口本身。优化后最大可观测等待是 `take_front`（协调器等最早类）约等于墙钟的
94%，`place_method` 等待 8.1 s / 25.7 s（线程时间），采样中 condvar 占 70–75%。它是"每活动类至多一个
待交付结果 + 最早类优先"的有序交付代价，不是可随手删除的开销；轮次延迟、最早类产量与窗口准入谁最终
定界尚未证实，需要换窗口设计（架构决策）才能再动。另有时钟读取占采样 6.8–7.3%（每条 entry 的 deadline
检查），只测到份额，未实现。
