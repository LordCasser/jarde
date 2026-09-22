# O3（任务 2.3）：查询首屏、boundary 重扫与 unit 临时结果

## 1. 调查的问题与假设

**问题**：分页改成「可停止」之后，首屏到底花了多少、续页有没有重扫、以及一个页的粒度是不是「unit 级」——
即一个只取 1 条的小页是否仍支付它开始的整个 unit。

## 2. 测量形状

| 来源 | 形状 | 样本 |
| --- | --- | --- |
| 本轮新测 | W4 六臂：`max_items=2` 续到耗尽（548 页 bcprov / 13 页 fixture）、`max_items=64`、4 consumer、取一页即丢弃 cursor、以及**损坏后缀**三臂，各 5 个独立进程 | `investigation-*-raw.jsonl` |
| 引用 G0 的「已改变」项 | 同一已声明工作负载在 `ba2076a` 上的 W4 读数 | 引用并复测（见 §3 注） |
| 引用 `add-demand-driven-core-results` / P1 | `tests/p1_query_demand.rs`（6 passed）、`tests/d0_demand_counts.rs` 的 unit 粒度读数 | 本轮真跑 |

新增字段：W4 每臂现在带 `usage`（整段序列的**同一本预算**）与逐页 `scanned_series`（每页结束时
`coverage.scanned_items`），所以「重扫」可以按**计数**而不是只按时长看。

## 3. 必要工作证据（p/s/h）

### 首屏与页成本（中位数，n=5）

| 臂 | fixture | bcprov |
| --- | --- | --- |
| 只取一页（`max_items=2`，随后丢弃 cursor） | `request` 18 µs（逐页中位 18） | `request` 42 µs（逐页中位 40） |
| 小页续到耗尽 | 13 页，`request` 978 µs，逐页中位 74 µs（max 137） | **548 页**，`request` 538,530 µs，逐页中位 764 µs（max 9,920） |
| 大页 `max_items=64` | 1 页，334 µs（逐页 333 µs） | 18 页，260,526 µs（逐页中位 9,234 µs） |
| items（两种页大小必须相同） | 24 / 24 | 1,094 / 1,094 |

> G0 把这一行标成「已改变」并留下「须在 2.3 按当前代码复测」。本轮复测给出同一形状：
> bcprov 548 页 / 538.5 ms 对 18 页 / 260.5 ms（G0：548 / 566 ms 对 18 / 274 ms）。**该观察在新代码上仍然成立。**

### boundary 重扫：**条目级为 0**

`scanned_series` 之和 vs 已发布 items（每臂 n=5，两语料一致）：

| 臂 | 页数 | Σ`scanned_items` | items | 重扫 |
| --- | --- | --- | --- | --- |
| 小页续到耗尽 | 13 / 548 | 24 / 1,094 | 24 / 1,094 | **0 / 0** |
| 大页 | 1 / 18 | 24 / 1,094 | 24 / 1,094 | 0 / 0 |

`scanned_items` 的口径把「续页重放的前缀」也算作已扫描（`crates/jarde-query/src/xref/mod.rs` 的
`ScanUnit`：重放项「counts as scanned but is neither published nor charged again」），所以这个和**能**看出重放；
本工作负载上它一次都没有发生：cursor 精确停在上一页停下的位置。

### unit 临时结果（引用 D0 门禁原文）

```text
pool probe full: items=2 scanned_items=2 archive_entries=5 read_bytes=723
pool probe page: items=1 scanned_items=1 archive_entries=3 read_bytes=532 has_more=true
```

一个只发布 1 条的页支付了**它开始的整个 unit**：3 个 archive entry、532 B 读，而整次查询是 5 / 723。
即当前的停止粒度是 **unit（一个类的条目）**，不是 item。这是 O3 候选「细粒度可重放续扫位置」的直接证据。

### 续页的时间代价（bcprov，字节工作量相同）

| 臂 | `entry_bytes` | `read_bytes` | `result_items` 计费 | `request` |
| --- | --- | --- | --- | --- |
| 小页 548 页 | 5,004,475 | 2,203,840 | **3,663** | 538,530 µs |
| 大页 18 页 | 5,004,475 | 2,203,840 | 1,094 | 260,526 µs |

字节层的工作**逐项相同**，item 级重扫为 0，所以 +278.0 ms 的差只能来自 530 个额外的**页边界**本身
（≈0.55 ms/边界），以及 `result_items` 多计费的 2,569 条。**口径限制**：W4 的四条臂在同一个进程、同一个 store 上按
小页→大页→多 consumer→放弃的顺序运行，第二条臂起享受前一条的保留；因此「小页 230 / 大页 34 个
`archive_entries`」（fixture）这类差是**顺序混淆**，不能读成分页代价。上面这张表之所以可读，
是因为两条臂的字节数**相等**——更大的臂本应因保留而更省，仍然相等，说明差值不是保留带来的。

## 4. 理想上界

- **首屏**：bcprov 一页 42 µs 对整段 260.5 ms（大页）／538.5 ms（小页），即首屏已经比「取完全部」早三个数量级；
  这里几乎没有可删的等待——上界是「把 unit 级停止变成 item 级」能省的部分，见下条。
- **unit → item 粒度**：引用读数给出每页最多可省 `532 − 该页真实需要的字节`（1 条命中时为 3→1 个 entry 量级），
  但本轮**没有**在真实语料上测出「一个 unit 内命中密度」的分布，所以只给 fixture 上的单点（3/532 对 5/723），
  **不给百分比**。
- **续页边界**：bcprov 上 530 个额外边界 = 278.0 ms，占 W6a 1-worker `request` 段 3,705 ms 的 **7.5%**。
  这是「调用方坚持 2 条一页」时的上界，不是默认配置的收益（默认没人在用 2 条一页）。

## 5. 资源与语义代价

- 驻留：W4 大页在 bcprov 上把 store 填到 `entries=2430, retained_bytes=8,687,490`（5 样本中位数），
  进程 RSS 66.7 MiB；小页同量级（69.3 MiB 一类的读数落在同组）。
- **契约风险（本条最重）**：cursor identity、coverage 与 diagnostics 三者联动。
  本轮观测到两条必须写进 delta 的行为：①**损坏条目会终止整次走查**，`has_more` 因此保持 1（不是 0）直到
  `issue` 消失；②早停的页**不**为它没读到的后缀报诊断（第一页 0 条诊断，走到底 1 条 `entry_integrity`）。
  这两条正是「未扫描后缀如实保留为未覆盖/未决范围」的现成形态，任何「细粒度停止」改动不得把它们变成
  「看起来完整」或「空 items 即无命中」。
- 计费：`page.has_more` 不得自动记成预算耗尽/取消；本轮的损坏臂里 `status` 与 `execution` 仍由引擎自己陈述，未被改写成 Complete。

## 6. 负向实验与结果

- **负向 1（坏后缀，真跑通过）**：同一归档、同一查询、三个停止点。
  `w4-damaged-first-page`：2 items、0 诊断、`has_more=1`、`scanned_items=2`（`request` 28 / 36 µs）；
  `w4-damaged-full-page`：4 items、**1 条 `entry_integrity`**、`has_more=1`、`scanned_items=4`（62 / 65 µs）；
  `w4-damaged-continued-to-exhaustion`：4 页、4 items、2 条诊断、`has_more=1`、
  `items_equal_full_page=1`（与整页臂**同一** items 指纹与诊断码）。
  对照：完好 fixture 的大页 24 items、0 诊断、`has_more=0`、小页续到耗尽同样 `has_more=0`。
  门禁 `a_page_that_stops_before_a_damaged_suffix_states_what_it_covered` 把这些钉成断言并**真跑通过**。
  变异方向：把「页满」或「损坏」记成 Complete（`has_more=0`）即变红。
- **负向 2（页大小不改变 items）**：`the_pages_cover_the_scan_and_the_page_size_changes_no_item` 继续通过
  （小页 13 页与大页 1 页的 items 指纹相同；4 consumer 与单 consumer 相同）。
- **负向 3（续页不重复发布/计费）**：引用 `p1_query_demand.rs` 的
  `the_continuation_splices_back_to_the_unpaged_scan`、`a_page_that_ends_at_a_member_boundary_does_not_decode_it_again`、
  `a_replayed_prefix_is_never_published_twice`、`a_small_page_stops_before_the_work_behind_it`、
  `a_small_resource_page_stops_before_the_containers_behind_it`、`the_whole_resource_traversal_pays_for_every_container_it_reaches`
  ——**本轮真跑 6 passed / 0 failed**。

## 7. 处置建议

- **「item 级可停止 sink」**：证据支持（unit 级停止 + 0 重扫 + 一页只发布 1 条仍付整个 unit），
  但**先把 cursor/coverage/diagnostic 契约 delta 写成子 spec**（design §6 已列为前置）；建议准入**最小实验**，
  不是直接实现。
- **「boundary 重扫」：本轮判定为不存在**（条目级 0），不立项；若将来出现，用 `scanned_series` 重测。
- **「页边界本身的开销」**（0.55 ms/边界，仅在小页时可见）：不建议为它单独立项；
  它随调用方的页大小选择出现，属于使用方式而不是默认路径。
- **不暂缓**：本条无「缺需求」类暂缓项。

## 8. 未确认项

- W4 四臂共享一个进程与 store，**冷/热顺序混淆**没有在本轮消除（fixture 的 `archive_entries` 差即受此污染）；
  要得到干净的分页开销对照需要「每臂一个进程」的配置，本轮没有为它改工作负载词表。
- 真实语料上「一个 unit 内的命中密度分布」**未测**，所以 unit→item 的收益只有 fixture 单点。
- 未测：并发多个 cursor 同时续扫同一 snapshot 时的开销与顺序（属 W6b，见 O7）。
- 尾延迟：10 个样本量级的任何 p95 都不成立（沿用 G0 口径），本轮 5 样本更不成立。
