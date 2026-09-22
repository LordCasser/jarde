# O7（任务 2.7）：W6a 的 1/N worker、总账、结果窗口与取消；W6b 需求核对

## 1. 调查的问题与假设

**问题**：在**已经按类去重**（每类只准备一次）的前提下，1/N worker 的并行收益还有多少、它被什么限制、
取消是不是真的停；以及 W6b（跨请求并发/single-flight）有没有实际需求。

## 2. 测量形状

| 来源 | 形状 | 样本 |
| --- | --- | --- |
| 本轮新测 | W6a `discard` 1/2/4/6 worker，两语料，各 5 个独立进程（冷进程到流结束）；另加**取消臂** `stop=2`（sink 确认 2 条方法后回答 `Stop`）×5 | `investigation-*-raw.jsonl` |
| 本轮新测（探针） | 同批样本挂 `BulkProbe`（`test-support`）：窗口站点、总账六站点、worker 生命周期 | 样本的 `probe` 字段 |
| 引用 bulk 子 change | `verification.md` §3/§12/§13、`evidence/cost-attribution.md`（总账锁 60→43 ns/entry 的前后对照）；`tests/bulk_recovery_*.rs`（本轮真跑 38 passed / 0 failed） | 引用 |
| 引用 G0 | 10 样本的 W6a 表与三档 sink 消融 | 引用 |

## 3. 必要工作证据（p/s/h）

### 去重后的并行收益（中位数，n=5）

| workers | bcprov `request` ms | bcprov `window` ms | 外部 wall s | RSS MiB | fixture `request` ms | fixture `window` ms |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 3,705.1 | 3,964.1 | 4.34 | 120.3 | 13.84 | 14.95 |
| 2 | 2,773.4 | 3,035.0 | 3.41 | 124.0 | 11.37 | 12.52 |
| 4 | 2,357.4 | 2,616.7 | 2.90 | 128.7 | 9.66 | 10.81 |
| 6 | 2,118.8 | 2,379.2 | 2.63 | 160.6 | 10.70 | 11.85 |

- 1→2 −25%，2→4 −15%，4→6 −10%（bcprov `request`，逐级递减）；方法与结果在所有档位相同
  （15,003 方法 / 19,865 记录 / 分类桶 same），`classes_prepared` 恒为 2,430。
- **「去重后」的口径**：`classes_prepared` 2,430 对方法 15,003 = **每类一次准备服务 6.2 个方法**；
  bulk 路径不移动 demand 计数口（`counts.class_preparations=0`），所以去重的证据是 `classes_prepared` 与
  操作自己的四段账，而不是 D0 计数器。
- fixture 的时序**未证实**：14.95 → 10.81 → 11.85 不单调，量级与进程启动同阶（沿用 G0 的判定）。

### 谁在限制（探针，同一批样本；线程时间合计）

| workers | `take_front` 等待 | 占 `request` 段 | `take_task` 等待 | `place_method` 等待 | 总账六站点等待 | worker 忙/线程 |
| --- | --- | --- | --- | --- | --- | --- |
| 2 | 2,613 ms | 94% | 1,632 ms | 0 | 0 | 3.91 / 5.55 s |
| 4 | 2,121 ms | **90%** | 4,796 ms | 0 | 0 | 4.59 / 9.39 s |
| 6 | 1,802 ms | 85% | 7,361 ms | 0 | 0 | 5.28 / 12.65 s |

1 worker 时窗口站点**一次都没有被调用**（`take_front` calls = 0）：串行路径不经过窗口。
交付本身很便宜：`deliver_nanos` 87.5 ms、`sink_nanos` 83.9 ms（19,865 条，即 ≈4.4 µs/条），
最长单个类任务 142 ms。与 G0 的 10 样本读数同形（`take_front` ≈90% of request、ledger 等待 0）。

### 取消（`stop=2`，sink 在第 2 条方法后回答 `Stop`）

| 读数 | bcprov | fixture |
| --- | --- | --- |
| sink 确认为方法记录数 / 交付数 | 2 / 2 | 2 / 2 |
| 操作自己的 `methods_delivered` | 1 | 1 |
| `final_delivered` / `traversal_complete` | 0 / 0 | 0 / 0 |
| `classes_prepared` / `classes_seen` / `records` | 2 / 2 / 4 | 2 / 2 / 4 |
| `not_executed` / `methods_declared` | 0 / 4 | 5 / 11 |
| 外部 wall | 0.27 s | 0.01 s |

即：**真停**（不再派新类、没有终态事件、没有把停止记成完成），且 worker 已被 join（调用返回、进程退出）。
`delivered=1` 对 sink 确认 2 的原因是 `src/bulk.rs::publish` 只为回答 `Continue` 的记录计数：
回答 `Stop` 的那一条**已交付**但不是「继续确认」。**登记为观察，不在本轮改**（`src/bulk.rs` 是明确禁区）。

## 4. 理想上界

- **当前可达的部分**：bcprov 1→6 worker 的 `request` 段 3705 → 2119 ms（1.75×），
  上界受**有序窗口**而不是 CPU 数限制（w=4 时 90% 的请求段在等 `take_front`）。
- **不可达的部分（未证实）**：若窗口等待能完全消除，`request` 段的理论下界是
  `request − take_front等待` = 2,357 − 2,121 = 236 ms（w=4）——但这条**不能**读成可达收益：
  等待背后是「最早类优先」的交付顺序约束（`place_method` 与共享池已在 4.7 里归零），
  要动它需要换窗口设计（bulk §12 已把这条登记为**未裁定的架构决策**）。
- 与 G0 的 10 样本对比：形状一致（1→6 单调下降），绝对值受机器负载影响，**不作吞吐结论**。

## 5. 资源与语义代价

- 内存：RSS 120.3 → 160.6 MiB（1→6 worker，bcprov），fixture 7.4 → 8.7 MiB；
  窗口的权重上界是 `buffered_weight_high_water ≤ limit`（不是 RSS 声明），二级额度（每活动类预留 + 共享池）
  已在 bulk 4.7 落地。
- 语义：1/N 的逐方法身份/顺序/正文/分类/aggregate 一致，差异白名单只有 `elapsed_millis` 与其字节后果
  （bulk `parallel-diff-whitelist.md`）；总预算只有一个（`p1_budget_ledger` 13 项）。
- 未完成面：bulk 6.3（超容量、大类倾斜、慢 sink、默认裁决）与 6.1（≥10 次交错）仍**未完成**；
  本专项不把「主体已交付」读成性能门禁完成。

## 6. 负向实验与结果

- **负向 1（取消必须真停且不装完成）**：本轮新增门禁
  `the_stopped_export_delivers_its_confirmed_prefix_and_stops` 真跑通过：断言
  `stopped_by_sink=1`、`methods=methods_confirmed=2`、`final_delivered=0`、`traversal_complete=0`、
  `delivered < methods`、`delivered + not_executed < methods_declared`。
  把停止记成 Complete，或把被拒的记录也算成交付，任一都会变红。
- **负向 2（跨 worker 同结果）**：`the_worker_sequence_publishes_one_result`（1/2/4/6 指纹相同）与 bulk 的
  `bulk_recovery_workers`（1 vs N 逐方法指纹相同）**真跑通过**。
- **负向 3（顺序与背压的反例，引用）**：bulk 4.7 的反例 A（取消每类预留 → 最早类 0/8 条交付、被看门狗取消）
  与反例 B（池容量写死 → 派生矩阵用例变红）各在去除修正时失败；3.4 的反例（去掉任务槽里的容器句柄 →
  `archive_entries` 按类数增长到 50）同形。这些证明「顺序/背压/容器交接」是被观察的量。
- **负向 4（普通入口不得触达批量）**：`tests/p5_benchmark.rs::no_ordinary_entry_reaches_the_bulk_module_or_recover_all`
  （19 passed / 1 ignored）——把普通 open/query/recover 接到 `recover_all` 即变红。

## 7. 处置建议

- **类间并行（W6a）：已交付、已由 bulk 子 change 默认启用，本条不重复立项**；
  下一瓶颈（有序窗口）是 bulk 登记的未裁定架构决策，本专项只提供归因（90% 的请求段在等它）。
- **W6b（跨请求并发 / single-flight）：暂缓**，理由是**缺实际需求证据**（沿用 [g0-workloads.md](g0-workloads.md) §4 的登记：
  归档 P5 的裁决为 disabled、`docs/support-matrix.md` 记录普通入口不存在该路径、bulk 的非目标明列该候选）。
  触发条件不新编：出现「一个 snapshot 两个并发请求」的宿主场景且其重叠差异越过同机噪声带宽。
  W6a 不因它被阻断（本条已独立测量）。
- **不实现** single-flight、key 表或 async runtime 来占位；也不改 `src/bulk.rs` 的计数语义。

## 8. 未确认项

- 5 个样本**不满足** bulk §14 事先声明的 ≥10 次交错判据，因此「吞吐提升多少」「是否接近 jadx」都不成立；
  只有形状与方向（1→6 单调下降、区间不重叠：3,705 vs 2,119 ms 的中位数）。
- 尾延迟**未测**（10 个样本也不够，本轮只有 5 个）。
- `take_front` 等待的 90% 是**采样口径**（窗口自己的 `wait_timeout`），不是「整次调用变慢」；
  它与「窗口设计换取收益」的比例关系未证实。
- `methods_delivered` 只计 `Continue` 确认这一条**观察**未上报为缺陷，也未在本轮修改产品代码。
