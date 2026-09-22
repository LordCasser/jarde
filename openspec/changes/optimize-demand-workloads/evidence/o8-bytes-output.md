# O8（任务 2.8）：摘要、复制、分配与输出对总时长的贡献

## 1. 调查的问题与假设

**问题**：在 W1 的 open/输出与 W6a 的交付上，摘要（hash）、复制/分配与最终编码各占多少；
哪些是「必要分析」、哪些是「实际序列化的字段」；候选（增量摘要、可信摘要传递、backing 共享、编码改进、mmap）各自受什么约束。

## 2. 测量形状

| 来源 | 形状 | 样本 |
| --- | --- | --- |
| 本轮新测 | W1 四段账（`open`/`prepare`/`request`/`output`）逐样本分列；W6a 三档 sink 消融（discard / encode / write，w=1 与 w=4） | `investigation-*-raw.jsonl`、`investigation-bcprov-sinks-raw.jsonl`（n=5 / n=3） |
| 引用 G0 | 10 样本的 W6a 三档消融、进程 RSS、`elapsed_millis` 唯一可丢字段的口径 | 引用 |
| 引用 `add-demand-driven-core-results` | `evidence/measurement-7-3.md`：essential 与 all 两种证据选择的 payload/RSS | 引用 |

## 3. 必要工作证据（p/s/h）

### W1：四段（中位数，n=5；单位 µs）

| 语料 | window | open（读入 + hash） | prepare（调用方发现） | request | output（调用方序列化） | 未归因余量 |
| --- | --- | --- | --- | --- | --- | --- |
| fixture（15.8 KB 输入） | 1,967 | 18 | 813 | 137 | **85** | 914（46%） |
| bcprov（2.9 MB 输入） | 276,664 | 2,681 | 256,334 | 1,415 | 97 | 16,137（5.8%） |

- **摘要/读入**（`open`）：2.9 MB 用 2,681 µs ≈ **1.08 GB/s**（读 + blake3 + 目录校验），占窗口 1.0%。
- **输出**：fixture 上调用方把一个 179 B 的文档序列化花了 85 µs——**占 `request` 的 62%**（小请求形态）；
  bcprov 上同样是 97 µs，但只占 request 的 6.9%。所以「输出占比」完全取决于请求本身多大，不能给一个全局数。
- **未归因余量在 fixture 上是最大单项（46%）**：那是调用方自己的目标发现（`targets()` 每次 `class_view` 建表），
  不是产品阶段；它登记在 `unattributed_micros` 里而不是塞进某个阶段。

### W6a：三档 sink 消融（同一会话，bcprov，n=3）

| sink (w=4) | `request` ms | 相对 discard | `window` ms | 嵌套 `sink_encode` | 嵌套 `sink_write` | 字节 |
| --- | --- | --- | --- | --- | --- | --- |
| discard | 2,497.98 | — | 2,770.19 | 0 | 0 | 0 |
| encode | 2,567.68 | **+69.7 ms（+2.8%）** | 2,840.94 | 331,994 µs（线程时间） | 0 | 391,181,028 B |
| write | 2,660.33 | +92.6 ms（对 encode +3.6%） | 2,931.91 | 333,840 µs | 218,521 µs | 写出 391,200,904 B |

fixture 同形（n=5，w=2）：discard `request` 11,235 µs 对 encode 11,556 µs（+2.9%），`sink_encode` 2,672 µs，
编码 2,836,709 B。`sink_visit` 只有 3–4 µs（计数回调本身不计成本）。
**读法**：编码占请求段 2.8–2.9%（w=4/w=2），写盘再叠加 ~3.6%；嵌套读数（33 万 µs 的线程时间）不能直接除以墙钟。

### 所需分析 vs 实际序列化的字段（引用）

`add-demand-driven-core-results` 的 7.3 表：`nav-class`/`nav-decl`/`expand`/`page-*` 两种选择下 returned 字节**完全相同**，
只有 `recover-one`（9,156 → 10,896 B）与 `sweep`（RSS +8.2 MiB）随选择增加而变；
即「必要结果」与「可选明细」在 payload 上是可分的，且默认选择不构造可选明细。
本轮的 `w1-class-source`（文本 2,876 / 3,017 B）是同一形状的另一读数。

## 4. 理想上界

- **编码**：完全消除 391.18 MB 的编码在 w=4 上最多省 **69.7 ms，占请求段 2.8%**；
  写盘再省 92.6 ms（3.6%）。这就是「编码改进」这个方向的天花板——**小于 5%**。
- **摘要/读入**：`open` 的 2,681 µs 是「必须读这些字节」的下界（校验和内容身份都要求读完），
  可省的只有「重复哈希同一份内容」，而这一条已由 `FactsKey::from_trusted` 交付；剩余上界 0（未发现重复哈希）。
- **CLI 的长度固定点（推导，不是直接测量）**：`crates/jarde-cli/src/main.rs::write_success` 在
  `MAX_RESPONSE_LENGTH_ITERATIONS = 20` 内反复序列化整份响应直到 `response_bytes` 稳定（至少 2 次：
  一次计数、一次写出）。按本轮实测的编码速率（391.18 MB / 331,994 µs ≈ **1.18 GB/s**），
  一份 1 MB 的单文档响应每多一次遍历约 0.85 ms。**本轮的 bulk 路径不受它影响**（JSONL 流各自按记录确认交付）。
- 一次性导出的端到端里，这三项加起来仍远小于并行/窗口那一项（O7）。

## 5. 资源与语义代价 + 备选矩阵

| 备选 | 约束 | 判断（依据） |
| --- | --- | --- |
| **增量摘要**（边读边算、不重读） | 需要把摘要绑定到每次读取的内容身份并处理失败重试 | **否决**：当前 `open` 本来就在一次读取里算完（2,681 µs / 2.9 MB），没有第二次遍历可省；引入增量状态只会增加失败路径 |
| **可信 digest 传递** | digest 只能来自一次已验证读取，禁止相信外部伪造摘要 | **保持已交付**（`FactsKey::from_trusted` + 门禁 `a_trusted_digest_answers_the_entry_the_bytes_wrote`） |
| **backing 共享**（借用父区间代替 owned） | 与 `Arc` 生命周期、共享权重核算、STORED 校验纠缠 | **否决（沿用归档裁定）**：O1 已换成「每容器一份不可变 owned backing」，共享权重只计一次 |
| **编码改进** | 不改输出字节契约；`serde_json` 是现有能力 | **否决/暂缓**：上限 2.8%（bulk 路径）；CLI 的单文档多次遍历只有推导值（0.85 ms/MB），**未直接测量** → 记为「需要大输出单文档工作负载才能判」 |
| **mmap / 文件 backing** | 需要 `unsafe` 或新依赖；且与「不可变 snapshot + 全内容摘要」冲突 | **否决**：本轮禁用 `unsafe` 与新 crate；内存目标与不可变快照无法同时满足（design §11） |
| **不改**：CRC/size/metadata 检查、raw bytes、输出计数含分隔符 | 契约 | 保持；见 §6 的负向证据 |

## 6. 负向实验与结果

- **负向 1（三档同结果，真跑）**：`the_three_sink_modes_publish_the_same_result` 断言三档的流指纹
  （顺序 + 身份 + disposition + 正文 + 每记录语义指纹）相同、`written_bytes == scratch_file_bytes`、
  discard 的 `encoded_bytes == 0` 且两个嵌套读数为 0、encode 只付编码不付写盘。**真跑通过**
  （整文件 14 passed / 0 failed / 1 ignored）。它同时钉住「输出字节契约」：任何编码改进都不得改变流。
- **负向 2（内容变化必须改变摘要与答案，真跑）**：`p5_facts_cache::the_key_binds_content_policy_and_declaration`
  与 `one_store_serves_two_snapshots_without_answering_either_with_the_other` 通过——同名不同内容不会命中；
  `p5_corpus_fingerprint` 未改（本专项没有改 `tests/fixtures/**`）。
- **负向 3（`elapsed_millis` 是唯一可丢字段）**：harness 的 `strip_elapsed` 断言它至少移除 1 个字段，
  且 G0 §6 的两轮重跑在 70/218 个契约比较项上 0 处不同——即长度类字节差（几百 B）来自内嵌耗时位数，
  不是内容变化。任何「用 lossy 字符串换字节」的方案会在这里变红。
- **负向 4（不发布半个成功 JSON，引用）**：`crates/jarde-cli/tests/export_cli.rs` 的缺失 `final`、部分写、
  取消三类用例（bulk 5.2 记录 `ulimit -f` 注入 → exit 2 且前缀恰好一条完整记录）。

## 7. 处置建议

- **编码改进：否决**（上限 2.8% 的请求段；默认路径没有大输出单文档）。
- **CLI 长度固定点：暂缓**，触发条件 = 出现一份**大输出单文档**的真实命令（≥1 MB 响应），
  届时直接用同一探针测「序列化次数 × 字节」；当前只有推导值，不作为准入依据。
- **增量摘要 / mmap：否决**（无可省的第二次遍历；禁用 `unsafe` 与新依赖）。
- **可信摘要传递与 owned immutable bytes：保持为安全路径**，不引入新的自引用或全局 intern。
- **不暂缓**已确认的交付语义（流式边界、终态记录、输出预算在提交前约束）。

## 8. 未确认项

- CLI 长度固定点的**直接测量缺失**（只有按实测编码速率推导的 0.85 ms/MB）。
- 分配次数（allocator 级）**没有测量**：没有计数探针，RSS 只给三个平面之一（O1/O5 已分列）。
- `output` 段只覆盖调用方对**返回文档**的序列化；W6a 的编码在 `request` 内（嵌套读数），两者不可相加。
- fixture 上 46% 的未归因余量属于调用方的目标发现，**不是**产品阶段；任何「引擎开销」的说法不得引用它。
