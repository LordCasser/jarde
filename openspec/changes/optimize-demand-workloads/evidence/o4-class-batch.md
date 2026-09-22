# O4（任务 2.4）：原顺序与「按类准备、逐方法交付」的比较

## 1. 调查的问题与假设

**问题**：把「每个方法一次普通请求」换成 bulk 子 change 的「按类准备、逐方法交付」，
在 W2/W3/W5 上省下的是哪一部分、总量/顺序/背压门禁是否仍然成立，以及冷端到端是否真的包含准备与输出。

## 2. 测量形状

| 来源 | 形状 | 样本 |
| --- | --- | --- |
| 本轮新测 | W3 两臂：同一 8 个类、同一 36（fixture 77）个 body，同一 store，一臂每类一请求、一臂每方法一请求 | `investigation-*-raw.jsonl`，n=5 |
| 引用 G0 | W3 既有三样本（整类一次 / 逐方法 / 固定集合）×10 样本；W6a 1/2/4/6 worker ×10 | 引用 |
| 引用 bulk 子 change | `tests/bulk_recovery_*.rs`（8 个目标，**本轮真跑 38 passed / 0 failed**）、`evidence/cost-attribution.md`、`verification.md` §3/§12/§13 | 引用 |

## 3. 必要工作证据（p/s/h）

### W3：同一目标集合、同一进程内的直接对照（左 bcprov，右 fixture）

| 口径 | class-major（按类准备） | method-major（原顺序） |
| --- | --- | --- |
| 类物化 | 8 / 8 | 36 / 77 |
| 类准备 | 4 / 8 | 36 / 77 |
| body 解码 | 36 / 77 | 36 / 77 |
| `class_headers` 合计 | 8 / 8 | 36 / 77 |
| `request` 段 | **600 / 851 µs** | **2206 / 4150 µs** |
| 逐请求中位数 | 23 / 31 µs | 56 / 42 µs |

整类一次（W3a）在 bcprov 上是 1 物化 + 1 准备 + 3 解码（`request` 1245 µs，窗口 289.5 ms 由
`prepare` 的调用方发现成本主导）；逐方法（W3b）同一批 3 个 body 是 **3 物化 + 0 准备 + 3 recovery**
（`request` 703 µs）。两臂的**结果**由新增门禁
`the_two_delivery_shapes_answer_the_same_bodies` 钉成相等（逐成员解码事实摘要），**真跑通过**。

### W6a（bulk 的按类准备、逐方法交付，冷进程）

| workers | `request` ms | `window` ms | 外部 wall s | RSS MiB | `classes_prepared` |
| --- | --- | --- | --- | --- | --- |
| 1 | 3,705.1 | 3,964.1 | 4.34 | 120.3 | 2,430（fixture 24） |
| 2 | 2,773.4 | 3,035.0 | 3.41 | 124.0 | 2,430 |
| 4 | 2,357.4 | 2,616.7 | 2.90 | 128.7 | 2,430 |
| 6 | 2,118.8 | 2,379.2 | 2.63 | 160.6 | 2,430 |

（本轮 W6a 的 `request`/`window`/wall/RSS 中位数，n=5；窗口 = `open` 2.7 ms + `prepare` 256.2 ms + `request`。
fixture 同形：`request` 13.84 / 11.37 / 9.66 / 10.70 ms。）

`classes_prepared` 恒为 2,430 而方法 15,003：**一次准备服务 6.2 个方法**（fixture 24 类 / 183 方法 = 7.6）。
fixture 同形：`request` 13.84 / 11.37 / 9.66 / 10.70 ms，1→6 worker。方法与结果总数在所有档位相同
（15,003 / 19,865 记录，分类 same）——即「交付形状」不改变结果。

### 冷端到端确实包含准备与输出

W6a 的 `window` = `open`（2.7 ms）+ `prepare`（256.1–258.3 ms 的调用方发现：load root + 类声明清单）
+ `request`（3,705 ms，其内嵌 `sink_encode`/`sink_write` 是**嵌套**读数）。**不**等于只测库调用：
prepare 的 256 ms 属于工作流而不属于请求，两者在样本里分列（G0 的账本规则）。

### 总量 / 顺序 / 背压门禁（引用，本轮真跑）

```text
bulk_recovery_backpressure 7 passed | bulk_recovery_cancel 7 passed | bulk_recovery_delivery 3 passed
bulk_recovery_handover 2 passed | bulk_recovery_lifecycle 4 passed | bulk_recovery_retention 2 passed
bulk_recovery_serial 7 passed | bulk_recovery_workers 6 passed        (0 failed)
```

- 总额度：一个 batch 一个共享总账（`p1_budget_ledger` 13 项），方法局部上限与整包总量分离（4.6）。
- 顺序：`bulk_recovery_serial` 断言交付顺序 == 独立游标顺序 == 声明序；1 vs N 的逐方法指纹由
  `bulk_recovery_workers` 断言相同（白名单只有 `elapsed_millis` 及其字节后果）。
- 背压：`buffered_weight_high_water ≤ limit`、单项容量与 jobs 无关、二级额度（每活动类预留 + 共享池）。
- 失败语义：单个方法的拒绝不伪装成整批成功（`outcome_*` 桶 + `methods_not_executed`）。

## 4. 理想上界

- **同一目标集合上的直接对照**（不受机器负载影响）：`request` 段 2206 → 600 µs（bcprov，3.7×）、
  4150 → 851 µs（fixture，4.9×）；类物化 36 → 8、准备 36 → 4。**成立**。
- **端到端比例 `p`**：W6a 与 W3 不是同一工作负载（整包 vs 8 类），不能相除；
  在 W6a 内部，`classes_prepared`=2,430 说明「按类准备」已经落在默认全量导出路径上，
  再想从「减少类级重复」里拿收益已经没有对象。**未证实**（也没有可测对象）。
- 一个 batch 的总额度：当前 bulk 路径的 `counts.class_preparations=0`（它不走 demand 计数口），
  所以本条只能用 `classes_prepared`/`methods` 与操作自己的四段账（entry/discovery/method/delivery）。

## 5. 资源与语义代价

- 驻留/RSS：W6a 1→6 worker 的 RSS 中位数 120.3 → 160.6 MiB（fixture 7.4 → 8.7 MiB）；
  窗口的权重按**拥有容量**核算（4.3 复核已改），`buffered_weight_high_water ≤ limit`，不是 RSS 声明。
- 语义代价（design §7 的硬约束，引用门禁持有）：一个 batch 有总额度；不得用逐方法新预算突破；
  失败方法不伪装整批成功；只恢复显式物理范围内的声明、不把依赖加入目标、不把方法产物包称为完整类源码；
  普通按需入口不隐式扩大范围（`tests/p5_benchmark.rs` 的源码守卫：只有 `src/bulk.rs` 可含调度拼写）。
- 未完成面：bulk 自己的 6.3（超容量 sweep、大类倾斜、慢 sink、默认裁决）**仍未完成**，
  4.5/6.1 为部分完成；本专项**不**把「主体已交付」读成生命周期与性能门禁全部完成。

## 6. 负向实验与结果

- **负向 1（分组不改变结果）**：新增门禁 `the_two_delivery_shapes_answer_the_same_bodies` 真跑通过；
  变异方向：任一族群的 per-member 解码事实被分组影响即变红（例如 class-major 少解一个 body）。
- **负向 2（同一份产出的跨 worker 一致性）**：`the_worker_sequence_publishes_one_result`（W6a）与 bulk 的
  `bulk_recovery_workers` 都断言 1/2/4/6 或 1/N 的指纹相同——**真跑通过**。
- **负向 3（顺序/背压反例，引用）**：`bulk_recovery_backpressure` 的慢首类 + 快后类用例在「去掉每类预留」时
  变红（bulk verification §13 的反例 A 记录了「最早类 0/8 条交付、被看门狗取消」）；池容量写死时派生矩阵用例变红。
  这些反例的作用是证明「顺序与背压」是被观察的，不是被声明的。
- **负向 4（不得扩大范围）**：`p5_benchmark.rs` 的
  `no_ordinary_entry_reaches_the_bulk_module_or_recover_all`（19 passed / 1 ignored）——普通入口一旦触到批量即变红。

## 7. 处置建议

- **按类准备、逐方法交付：已交付并已由 bulk 子 change 拥有，本条不重复立项**。
- **残余可做的事只在 bulk 自己的未完成面**（6.3 的四类退化形状与默认裁决），属该子 change 的范围。
- **不改**普通按需入口去隐式批量（门禁已钉住）。
- 本条**不暂缓**：它没有待实施项。

## 8. 未确认项

- W3 两臂的「同一 store」使第二条臂（method-major）享受第一条臂的保留，因此**两臂的绝对成本都偏乐观**；
  但两臂的**工作量计数**（物化/准备/解码）与「请求数比例」不受此影响。
- bulk 的逐方法交付成本（`sink`/`deliver` 的 87.5 / 83.9 ms 每 19,865 条）与 O8 的编码成本混在同一段里，
  两者的精确边界在 O8 处理。
- 超容量 sweep、大类倾斜、慢 sink 三类退化形状**本轮未测**（归 bulk 6.3）。
