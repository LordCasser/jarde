# O5（任务 2.5）：facts 各层的复用距离、开销与超容量行为

## 1. 调查的问题与假设

**问题**：现有 facts store 在 W1/W2/W5 上真的被复用了多远、命中避免了哪些工作、查找/哈希/克隆/同步要花多少，
以及容量不足时当前策略是「拒绝」还是「淘汰」；更高层（method decode/X1/resolution/IR）该不该现在建。

## 2. 测量形状

| 来源 | 形状 | 样本 |
| --- | --- | --- |
| 本轮新测 | W5（整包 sweep + 同 store 上 10 次单方法往返）在 **5 个容量档**（`none` 0/0、`tiny` 1 entry/4 KiB、`e1`、`e4`、`roomy` 1<<14/1<<27）上跑两语料，各 5 个独立进程 | `investigation-*-raw.jsonl` |
| 引用 G0 | W5 roomy/tiny ×10；W1/W2 的 store 报告 | 引用并复测 |
| 引用门禁 | `tests/p5_facts_cache.rs`（13）、`tests/p5_shared_payload.rs`（6）、`tests/p5_container_lookup.rs`（29）、`tests/d2_prepared_handoff.rs`（9） | 本轮**真跑** |

## 3. 必要工作证据（p/s/h）

### 容量与复用距离（中位数，n=5；`x/y` = fixture / bcprov）

| capacity | `container_hits` | `container_misses` | `directory_parses`（往返累计） | `refused_capacity`(+bytes) | 往返 `sequence_micros` | 往返 `archive_entries` |
| --- | --- | --- | --- | --- | --- | --- |
| `roomy` | 392 / 17538 | 20 / 1 | 22 / 11 | 0 | 361 / **469** | 10 / **70** |
| `e4` | 392 / 17538 | 20 / 1 | 22 / 11 | 0 | 361 / 525 | 10 / 70 |
| `e1` | 208 / —（本轮 bcprov 未跑 e1） | 204 / — | 22 / — | 11 / — | 407 / — | 10 / — |
| `tiny` | 0 / 0 | 412 / 17539 | 42 / 31 | 19(+3 B) / 19(+2 B) | 422 / **24,971** | 130 / **51,450** |
| `none` | 0 / 0 | 412 / 17539 | 42 / 31 | 32 / 31 | 458 / **25,268** | 130 / **51,450** |

- **复用距离**：bcprov 上 `roomy` 与 `e4` 的**复用计数逐项相同**（`archive_entries` 70、`container_hits` 17538、
  `directory_parses` 11、`refused` 0；往返段 469 对 525 µs 的差落在 5 样本 spread 内）——10 次往返里命中的是
  **同一个类事实 + 同一个容器**（`entries=1, containers=1`），即本工作负载的复用距离是「同一请求流内的立即重复」，
  不是「跨很多类之后回头」。
  fixture 的 `e4` 也够用；只有 `e1` 开始出现部分拒绝（11 次 entry 拒绝、命中率 208/412）。
- **命中避免的工作**：同一 10 次往返，`roomy` 51,380 个 `archive_entries` 被省掉（51,450 → 70）、
  `directory_parses` 20 次（31 → 11）、往返段 25,268 → 469 µs（**54×**，区间不重叠）。
  fixture 同形但小得多（130 → 10 entries；422 → 361 µs，**1.17×**）：目录越贵，保留越值钱。
- **一次性 sweep 无复用可省**：四档的 sweep 段 3.70–3.72 s、分类桶与方法数完全相同——
  单次遍历里 `p = 0`。这条读数否决了「缓存改善整包导出」的说法（O4/O7 的加速来自并行与按类准备，不是缓存）。

### 命中路径自身要花多少（h 的上界，而不是分量）

`roomy` 往返段 10 次请求共 469 µs（bcprov，n=5），即**每次请求 ≤ 47 µs 是含一切的上界**
（lookup、key、锁、克隆、以及请求本身的分析/表现）。真正想要的 `h`（查找 + 哈希 + 克隆 + 同步各自的份额）
**没有单独测**：单请求路径不进共享总账（G0 §3 已写明这条路径没有 ledger 站点），store 的计数器只计事实，
不读时钟。可引用的两个结构事实：

- **可信摘要免重算**：`FactsKey::from_trusted` 用一次已验证读取发布的 digest，不再对同样的字节哈希
  （源码文档 + 门禁 `a_trusted_digest_answers_the_entry_the_bytes_wrote`）；
- **命中只做 `Arc::clone`**：载荷在锁内只克隆句柄、不复制内容
  （门禁 `the_store_hands_out_the_one_allocation_it_holds`、`a_live_handle_survives_a_clear`）。

## 4. 理想上界

- **可达上界**：bcprov 往返段最多可省 25,268 − 469 = **24.8 ms / 10 请求**（98%），代价是 `retained_bytes`
  3,685,071 B。这个上界只在「同一 store 上有立即重复访问」时可达。
- **不可达的部分**：单次 sweep（W5 的 `p=0`）与 W1/W2 的单次请求流**没有任何可省的重复**；
  把它们算进收益就是重复支付提前成本。
- **`p`**：W5 往返段占 W5 窗口的 469/16,562 = 2.8%（bcprov roomy）；但它不是端到端目标的一部分
  （sweep 才是），所以不写成「端到端 2.8%」。

## 5. 资源与语义代价 + 备选矩阵

| 维度 | 当前事实（可引用） | 备选与判断 |
| --- | --- | --- |
| **分层身份** | 容器层：`{snapshot, ContainerOrigin, schema}`，origin 单列而非 class name；CP/Header：`{content digest, length, policy}`；结构/Header 互不代答 | X1 需再加 consumer schema；resolution 需再加 symbol/source/loader/view/平台/依赖快照；IR/source 还要方法、规则与命名证据。**本轮不建**：没有这些层被复用的实测复用距离 |
| **完整发布** | 只有**完整不可变**的容器事实才会发布：`only_a_complete_container_is_published`、`a_cancelled_or_damaged_container_is_not_published` | 保持。拒绝/损坏/取消都不写入部分事实 |
| **byte admission** | `FactsCapacity { entries, retained_bytes }` 双界；权重按贡献核算，共享 backing 只计一次（归档 change 已实现） | 保持双界。`e1` vs `e4` 的读数说明 entry 界是**先满的那个**，所以两个界都必须留 |
| **淘汰** | **没有淘汰**：满则拒绝（`FactsCache::new` 文档 + `refused_capacity*` 计数）。`tiny`/`none` 档的 19/31 次拒绝证明它真的在拒绝 | LRU/二次访问准入**暂缓**：需要「热点淘汰」证据（跨请求的复用距离分布），本轮只有立即重复 |
| **直接回退** | 拒绝、失效、损坏都丢弃该项并在剩余预算内直接算：`the_zero_capacity_store_retains_nothing_and_changes_no_result`（本轮新增，真跑通过）、`a_discarded_entry_falls_back_under_the_same_budget`、`the_capacity_is_a_bound_and_a_refusal_is_not_a_hit` | 保持。回退不改结果，只改资源读数 |

内存/驻留：bcprov `roomy` 往返结束时 `entries=1, containers=1, retained_bytes=3,685,071`，
进程 RSS 中位数 124–128 MiB（整个 W5 进程含 sweep）；`none` 档 `retained_bytes=0`，RSS 同量级
——**驻留不是 RSS 的主导项**（sweep 的工作集才是），所以任何「省下 X MB RSS」的说法都还没有证据。

## 6. 负向实验与结果

本轮**真跑**（`cargo test --locked --features test-support`，全部 0 failed）：

| 反向场景 | 门禁 | 结果 |
| --- | --- | --- |
| 同名字节变了不能命中 | `p5_facts_cache::the_key_binds_content_policy_and_declaration`、`p5_container_lookup::a_changed_content_chain_or_declaration_misses` | ok |
| 同一 store 服务两个 snapshot 不能互相作答 | `p5_facts_cache::one_store_serves_two_snapshots_without_answering_either_with_the_other` | ok |
| 内容相同、origin 不同不合并 | `p5_facts_cache::content_is_shared_and_two_origins_are_never_merged` | ok |
| **依赖补齐**不能沿用旧的「不存在」 | `p5_facts_cache::a_provider_added_later_is_answered_by_a_fresh_search` | ok |
| Strict 不能拿 Forensic 的读作答 | `p5_facts_cache::a_strict_request_is_never_answered_with_a_forensic_read` | ok |
| 失败/停止/取消的构造不能被缓存 | `p5_facts_cache::a_stopped_read_stores_nothing_and_a_cancelled_request_is_never_answered` | ok |
| 满容量不是命中、活句柄不被作废 | `p5_facts_cache::the_capacity_is_a_bound_and_a_refusal_is_not_a_hit`、`p5_shared_payload::{a_full_store_refuses_without_invalidating_a_live_payload, a_store_with_room_for_nothing_retains_nothing_and_answers_no_wrong_value}`、`p5_container_lookup::a_capacity_refusal_does_not_rescan_the_request` | ok |
| 内容变化时容器链/声明必须 miss（真跑打印） | `flat JAR with 4 sibling archives: directories parsed 1, nested materialized 0` / 归档 change 的 `a_changed_content_chain_or_declaration_misses` | ok |

`p5_facts_cache` 13 / `p5_shared_payload` 6 / `p5_container_lookup` 29 / `d2_prepared_handoff` 9（全部 0 failed）。
**这些是本条最需要的反向场景**：命中必须绑定内容、策略、origin 与 snapshot，且「补齐依赖」不能命中旧的失败结论。

## 7. 处置建议

- **容器层与 CP/Header 层：保持现状，不重开**（归档 O1 与 `642e49f` 已交付，本轮复核无新缺口）。
- **更高层 store（method decode / X1 / resolution / IR / source）：建议暂缓**。
  重新进入条件（可观察）：出现一条「同一 store 上跨 ≥2 个请求复用同一方法/同一 resolution」的**实测复用距离**，
  且该产品的驻留量能在 `FactsCapacity` 的字节界内被证明（当前 2430 类的 CP/Header 层已占 8.7 MB）。
- **淘汰策略：暂缓**，维持「满则拒绝」。触发条件：实测复用距离分布显示热点集中，
  且第二访问准入能证明「不淘汰导致重复计算」的代价大于记账成本。
- **直接回退：保持为默认**；本条不引入任何新开关。
- 本条**不暂缓任何已确认需求**：它是一个「维持 + 等证据」的结论。

## 8. 未确认项

- **`h` 的分量**（查找/哈希/克隆/同步各自的耗时）未单独测；只有「每次请求 ≤ 47 µs」的总上界。
- **锁竞争**：单请求路径确实经过 store 的 `Mutex`，但没有探针；W6a 的共享总账等锁读数为 0（G0）**不能**替它作证。
- **真实宿主请求流的复用距离**：没有 W6b 需求（见 O7），所以「跨请求保留」的价值只在 W5 的立即重复上被测量。
- **`retained_bytes` 与 RSS 的换算**未建立；本文件不宣称任何 RSS 收益。
