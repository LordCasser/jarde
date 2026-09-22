# O2（任务 2.2）：class 物化、结构解析、方法定位与 driver/callee 的剩余重读

## 1. 调查的问题与假设

**问题**：在 D2 已把普通入口收敛到「一次选中定义只准备一次」之后，W1–W4 里**还剩多少**同一 class 的重复物化/准备，
它们的内存代价是什么，以及 `class_source` 的 bind + prepare 双读取是否已关闭？

## 2. 测量形状

| 来源 | 形状 | 样本 |
| --- | --- | --- |
| 引用 `add-demand-driven-core-results` | `tests/d2_prepared_handoff.rs`、`tests/d0_demand_counts.rs`（真跑，见 §6） | 9 + 5 passed |
| 引用 G0 | `evidence/g0-metrics.md` 的 W3 计数行（10 样本） | 引用 |
| 本轮新测 | fixture 与 bcprov 的 W1（冷 / 第二次 / 第二个 snapshot / `class_source`）、W2、W3 的 `class-major` 与 `method-major` 两臂，各 5 个独立进程 | `investigation-*-raw.jsonl` |

新测的键：`class_materializations`（一次被选中定义的受信读取）、`class_preparations`（一次 `PreparedClass::prepare`）、
`body_decodes`、`recovery_runs`，加 `usage.class_headers/method_bodies/class_bytes` 与 `cache.retained_bytes`。

## 3. 必要工作证据（p/s/h）

### W1（fixture / bcprov，中位数 n=5，`request` 段 µs）

| 样本 | 物化 | 准备 | 解码 | recovery | request | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| `w1-cold-method` | 1 / 1 | 0 / 0 | 1 / 1 | 1 / 1 | 137 / 1415 | 方法体不命名同类成员 ⇒ 不求 callee、不准备 |
| `w1-second-request` | 1 / 1 | 0 / 0 | 1 / 1 | 1 / 1 | 57 / 63 | **同 snapshot 同请求：读与准备不共享**，只有容器事实命中 |
| `w1-second-snapshot` | 1 / 1 | 0 / 0 | 1 / 1 | 1 / 1 | 57 / 1329 | 同 library、另一 snapshot，行为与冷路径一致 |
| `w1-class-source`（名字入口） | 0 / 0 | **1 / 1** | 8 / 3 | 8 / 3 | 551 / 2593 | 8（bcprov 3）个成员共用**一次**准备与**一次**类头 |

第二、三行的「物化仍为 1」是 O2 的**剩余重读第一条**：同一定义在同一 store 上的下一次请求会重新做一次受信读取
（CRC/长度/span 全部重做），保留的只有容器目录/backing 与（内容 key 相同的）结构事实。

### W2（8 个类连续导航 + 1 次控制重问）

`class_materializations` **9**、`class_preparations` 0、`body_decodes` 0、`recovery_runs` 0（fixture，n=5）；
逐请求 `class_headers` 恒为 1。即：导航面每个类付一次读取、零解码、零准备，
**控制重问仍是一次物化**，没有从 store 拿回「已读过的定义」。

> **本轮 harness 修正登记（两处，均只影响读数口径，不改产品语义）**
>
> 1. **W2 的计数基线**：原先取在请求段**之后**，两次快照之差因而恒为 0。本轮把基线移到请求段之前，
>    上面的 9 是修正后的读数；只影响 W2 的 `counts` 文档，不影响任何已发布的 G0 读数
>    （G0 的 W2 指标是 `sequence_micros`/`per_request_micros`/`class_headers_total`）。
> 2. **`the_instrumentation_changes_no_domain_report` 的长度字段**：该门禁原先用**逐字节相等**比较
>    `returned_bytes`，而返回文档的长度内嵌该次运行的 `elapsed_millis`（G0 已把这条写进「长度不属语义指纹」）。
>    在全 workspace 并行负载下它因此可能偶发变红（本轮观察到一次）。现在除 `returned_bytes` 外**逐字段
>    精确相等**，该字段允许 ≤64 B 的差值并给出理由；变异验证：把 `sequence_micros` 加进比较集合 →
>    该用例以 `the counted work differs between the two instrumentations` 失败（随后还原）。


### W3 两臂（同 8 个类、同 36（fixture 77）个 body，同一 store，先 class-major 后 method-major）

| 口径 | class-major（每类一请求） | method-major（每方法一请求） | 比 |
| --- | --- | --- | --- |
| 物化 | 8 / 8 | 36 / 77 | 4.5× / 9.6× |
| 准备 | 4 / 8 | 36 / 77 | 9× / 9.6× |
| 解码 | 36 / 77 | 36 / 77 | 1×（同一结果） |
| `request` 段 | 600 µs / 851 µs | 2206 µs / 4150 µs | 3.7× / 4.9× |
| 逐请求中位数 | 23 µs / 31 µs | 56 µs / 42 µs | — |

（左 bcprov、右 fixture。）**重复量**：method-major 在 8 个定义上做 36 次读取与 36 次准备，
其中 28 次读取、32 次准备（fixture 69/69）是对**同一次的重复**。

### driver/callee 与 `class_source`

- **driver/callee 重读：本轮未观测到。** 普通入口把绑定/自己的读取交给同一个表现层
  （`recover_bound_method` / `recover_own_read`），只有「绑定没有读过这个类」时才走
  `CalleeClass::None → read_named_callees`。引用门禁原文：
  `recover_method (no callees): materializations=1 preparations=0 body_decodes=1 recovery_runs=1`。
- **`class_source` 的 bind + prepare 双读取：已关闭**（D2）。引用门禁原文：
  `class_source: materializations=1 preparations=1 body_decodes=8`，且身份/名字两条绑定路径都断言
  「一个类头 + 一次准备 + 同一份文本」。
- **缺哪个计数**：`class_materializations` 只钩在**身份路径**的读取上；名字搜索的候选读取是搜索自己的成本，
  不计数（产品侧文档已声明，测试 doc 也写明）。所以名字入口（我的 `w1-class-source`）读到 0/1，
  而身份入口（引用门禁）读到 1/1。要量名字路径的读取必须看 `usage.class_bytes`/`class_headers`。

## 4. 理想上界

- 以 W3 两臂为同一目标集合上的**直接对照**：把「每方法一请求」换成「按类准备、逐方法交付」，
  类级工作从 36 读 + 36 准备降到 8 读 + 4 准备，`request` 段 2206 → 600 µs（bcprov，区间不重叠）。
  这是**同一进程同一 store 内的相对读数**，因此不受机器负载影响：**这条上界成立**。
- 若在操作内复用「同一次已选定的读取」（让第二次请求不重做受信读取）：W1 第二次请求 57/63 µs 里含 1 次物化，
  W2 的 9 次物化里有 1 次是控制重问。可删的部分**至多是每次请求一次读取成本**；
  本轮**没有单独测出**这一次读取的微秒数（它与请求其余部分同段），所以只写计数上界，不给百分比。
- `p`（占端到端比例）：W1 冷请求 1.415 ms 相对同进程窗口的 prepare 256.3 ms 很小，但两者不同源，不做相除。**未证实**。

## 5. 资源与语义代价

- 驻留：bcprov W3 两臂结束后 `entries=4, containers=1, retained_bytes=3,691,187 B`；进程 RSS 中位数 21.7 MiB
  （W3）、26.1 MiB（W1）。fixture 同形为 44,507 B / 7.5 MiB。
- **要共享结构就必须驻留结构**：把「已读过的定义 → prepared」提升为跨请求事实，等于把当前只有 4 个 entry 的结构层
  扩到「访问过的每个类」。W5 的 roomy sweep 已经是 2430 entry / 8,687,490 B 的量级（CP/Header 层），
  若同一目标集合被 prepared 层再记一遍，驻留会叠加而不是共享。
- 语义代价：方法 key 必须用 raw name/descriptor 并保留重复声明歧义（design §5）；不同 snapshot/物理来源不能混淆；
  Strict/Forensic 不能互相代答；共享 code/bootstrap 解码时只保留必要 use-site 摘要，
  不能为省扫描而驻留全部方法 IR。若准入，后一条与本轮 W3 的 `owned_records`（fixture 1058、bcprov 644）直接冲突，须单独定量。

## 6. 负向实验与结果

本轮**真跑**（`--features test-support`）：

- `cargo test --test d2_prepared_handoff -- --nocapture` → 9 passed / 0 failed，打印
  `class_view (three bodies): materializations=1 preparations=1 body_decodes=3 | class_headers=1 method_bodies=3 class_bytes=1064`、
  `None/Zero/TooSmall/Roomy: materializations=1 preparations=1 body_decodes=8`（**四档 store 状态做同样的工作**）、
  `with a store: archive_entries=3 entry_bytes=532 | without: archive_entries=5 entry_bytes=532`。
- `cargo test --test d0_demand_counts -- --nocapture` → 5 passed / 0 failed。
- **负向 1（「缓存等于免费」必红）**：`the_four_store_states_do_the_same_work_and_no_request_is_free` 与
  `a_store_never_makes_the_next_storeless_request_cheaper`——把「命中」当「请求变便宜」的实现会在这里失败。
- **负向 2（两臂同结果）**：新增门禁 `the_two_delivery_shapes_answer_the_same_bodies`
  （`tests/p5_optimize_workloads.rs`）比较两臂的逐成员解码事实摘要，**真跑通过**
  （整文件 14 passed / 0 failed / 1 ignored，0.4 s）。变异方向：臂的摘要一旦含 `coverage`/`usage` 这类「分组读数」即变红。
- **负向 3（未请求就不解码）**：`a_member_only_read_decodes_no_body_and_builds_no_record` 断言
  `list_members` 与「无 body 的 `class_view`」都 `body_decodes=0`（打印行同上），即「按需」不是「顺手全解」。

## 7. 处置建议

- **「按类准备、逐方法交付」：已由 bulk 子 change 实现（O4 引用其门禁），本条不再立项。**
- **「操作内复用同一次已选定读取」：建议进入 §3 最小实验**（单因素、可撤销），
  直接路径清楚：同一 snapshot + 同一物理定义 + 未失活的读取句柄；失败即回到当前读取。
  **必要前提**：先测出「一次物化」自己的成本占比，否则无法与 O5 的 CP/Header 层收益区分。
- **`class_source` / driver-callee 重读：判定已关闭，不立项**（引用门禁持有）。
- **不建议**把整个 `MethodIr` 提升为长期全局缓存（design §5 已列为非目标）。

## 8. 未确认项

- 「一次受信读取」自身耗时**未单独测**，所以 §4 只有计数上界、没有百分比。
- `query` 消费者路径（`ScanUnit::read_unit`）是否重复物化同一 unit：W4 的 `class_materializations` 恒为 0
  （query 不移动 demand 计数口），本条**未测**，见 O3。
- 名字绑定路径的读取成本只在 `usage.class_bytes` 上可见，本轮未建立与身份路径可比的口径。
- `owned_records` 驻留与本条候选的冲突量**未定量**。
