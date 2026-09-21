# verification — add-parallel-bulk-recovery（实施中，2026-09-21）

本文件按任务逐条记录**实际执行过**的证据；未执行、未证实的部分单独列出，不勾选、不换算成"已完成"。
所有命令在 `/Users/lordcasser/workspace/projects/jarde`（工作树，未提交）执行；语料为 `openspec/benchmark-protocol.md` 登记的 vulhub 12 个 artifact（本轮 12 项字节数与 sha256[:16] 全部复核一致）。

## 1. 全量门禁（最终状态）

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 无差异 |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 0 warning |
| `cargo test --workspace --all-targets --all-features --locked` | **1435 passed / 0 failed / 7 ignored**（89 个 test target） |
| `cargo test --test p5_benchmark --locked` | 19 passed / 1 ignored（源码守卫 + 能力清点） |

## 2. 逐任务证据

| 任务 | 状态 | 证据（文件 + 真实运行） |
| --- | --- | --- |
| 1.2 契约与 effective defaults | 已实现已验证 | `src/bulk.rs` 类型面；`crates/jarde-cli/src/export.rs` JSONL；`export_cli` 10 项；默认配置整包实测（见 §3） |
| 2.1 prepared class + 多值 locator | 已实现已验证 | `crates/jarde-reader/src/prepared.rs`；`tests/p1_prepared_class.rs` 12 项（解码逐字段等价、定位器保留全部重复 ordinal、重复/缺失 Code、成员表停止、记录复核拒绝、clear/零容量后 backing 有效） |
| 2.2 共享 payload | 已实现已验证 | `crates/jarde-reader/src/facts_cache.rs`（`Arc` 共享 + `FactsKey::from_trusted`）；`tests/p5_shared_payload.rs` 6 项；`tests/p5_facts_cache.rs` 13/13 未改一字 |
| 2.3 driver/callee 消费 prepared | 已实现已验证 | `jarde_jvm::{analyze_prepared_method_ir, read_prepared_callees}`；`tests/p3_prepared_input.rs` 6 项；`tests/p3_declaration_handoff.rs` 加强未放宽 |
| 3.1 有界遍历 + 串行闭环 | 已实现已验证 | `src/bulk.rs`；`tests/bulk_recovery_serial.rs` 6 项 |
| 3.2 交付后释放 / 超容量 sweep | **部分** | 类型面不含逐方法结果、事件按产生顺序交付；缺分配器级或紧容量级断言 |
| 3.3 汇总与局部失败继续 | 已实现已验证 | `tests/bulk_recovery_serial.rs` 计数分桶与 aggregate 断言；`tests/p1_budget_ledger.rs` 13 项 |
| 4.1 共享总账 + 局部额度 | 已实现已验证 | `crates/jarde-reader/src/ledger.rs`；`tests/p1_budget_ledger.rs` 13 项（含 barrier 抢最后额度） |
| 4.2 scoped worker 生命周期 | 已实现已验证 | `tests/bulk_recovery_lifecycle.rs` 4 项（rendezvous 重叠、`WorkerWatch` 零遗留、创建失败与 panic → Failed） |
| 4.3 有序背压 | 已实现已验证 | `tests/bulk_recovery_backpressure.rs` 2 项（慢首类、权重上限、超限单项局部停止、1/4 worker 同 ceiling） |
| 4.4 取消与唤醒 | 已实现已验证 | `tests/bulk_recovery_cancel.rs` 6 项（预取消、跨类取消、满槽唤醒、交付内取消、sink Stop、sink Err） |
| 4.5 调度扰动与旧串行对照 | **部分** | 1 vs 4 worker 逐方法 fingerprint/key/text 与 summary 指纹一致；未登记旧串行完整 fingerprint 与固定差异白名单 |
| 4.6 sink 接入同一总账 | **未实现** | CLI 侧自持输出额度（两本账各自有界）；`RecoverySink` 拿不到 ledger 许可 |
| 5.1 export 命令 | 已实现已验证 | `crates/jarde-cli/src/export.rs`；`export_cli` 10 项 |
| 5.2 有界 JSONL 与交付确认 | 已实现已验证 | 小额度三态、遍历受损、`ulimit -f` 注入 I/O 失败；部分写回滚分支在 macOS 不可证伪（如实记录） |
| 5.3 能力清点/隔离 | 已实现已验证 | 守卫改为"只有 `src/bulk.rs` 可含调度拼写" + 新增 `no_ordinary_entry_reaches_the_bulk_module_or_recover_all`；两次探针反例 |
| 5.4 操作内保留 | 已实现已验证 | `EXPORT_FACTS = FactsCapacity::new(1<<14, 1<<27)`；`archive_entries` 65,517 → 2,569（bcprov）与 65,470 → 944（s2-009）；反例（去掉挂载）三条用例变红 |
| 7.1/7.2 单类源码视图与 CLI | 已实现已验证 | `src/class_source.rs`、`tests/class_source.rs` 16 项、`crates/jarde-cli/tests/class_source_cli.rs` 13 项 |
| 7.3 单类视图读取形状 | 已实现已验证 | 改为消费一次 prepared 类：`class_headers` 常数 2、`class_bytes` = 类长×2、文本逐字节不变；反例使形状测试变红 |
| 6.1–6.3 性能 | **首轮实测，未满足协议** | 见 §3：3 次交错重复、2 个 artifact，不足协议要求的 ≥10 次与 12 artifact 全集 |
| 6.4 候选固化与 JDK 门禁 | **部分** | fmt/clippy/workspace 已跑；受控 JDK 编译执行对照本轮未重跑（见 §5） |
| 6.5 verification/文档同步 | 进行中 | 本文件；支持矩阵与父专项引用待同步 |

## 3. 性能首轮实测（配对、同机、交错、每配置 3 次）

命令：jadx 1.5.6 `-d <tmp> --no-res`（默认 6 线程与 `-j 1`）；jarde `export --jobs 1|2|4 --scope artifact_tree`（默认配置，含操作内保留）。

| 语料 | jadx 默认 | jadx `-j1` | jarde jobs1 | jarde jobs2 | jarde jobs4 | 产物（jarde） |
| --- | --- | --- | --- | --- | --- | --- |
| bcprov（2,430 类 / 15,003 方法） | **2.77 s** | 4.26 s | 4.74 s | **4.09 s** | 4.31 s | 357 MB JSONL |
| s2-009（7,200 类 / 57,180 方法） | **5.22 s** | 10.12 s | 17.35 s | **13.68 s** | 15.48 s | 2,383 MB JSONL |

原始样本：`/tmp/jarde-b10/raw.jsonl`（每行 `tool/artifact/config/wall_s/exit/output_bytes`）。

**可读出来的结论（按同口径）**

- 同并发对照（jadx `-j1` vs jarde `jobs1`）：bcprov **1.11×**、s2-009 **1.71×**。相对本 change 的起点（逐方法 sweep 单线程 2.65 / 5.10 ms/体、整包 40.2 s / 296.5 s），整包导出已从"慢 12–46×"进入同一量级。
- 对 jadx 默认并发：bcprov **1.48×**、s2-009 **2.62×**（jarde 最优配置）。
- **并行收益有上限**：jobs 从 1 增到 2 有收益（s2-009 −21%、bcprov −14%），2→4 无进一步收益。最可能原因是**交付是单线程的**（编码 + 写出按记录串行，产物 357 MB–2.38 GB），且每个类窗口内的可并行工作被背压限住。这是本轮记录的第一个瓶颈，属任务 6.3 的"显式默认配置裁决"输入。
- **产出单位不同**：jadx 写整类 `.java`（语料上 22.5–82.3 MB），jarde 写逐方法 JSONL（含 source map 与四个平面，357 MB / 2.38 GB）。逐字节比较不成立；方法集合对账才是可比口径。
- 退出码语义不同：jarde `export` 在"遍历完整、全部方法已交付、但有方法未产出语句"时为 4（aggregate partial，bcprov 3 个、s2-009 318 个 `not_produced`）；jadx 同类情形为 3（finished with errors）。
- 未控制 OS page cache 初态（本轮无丢缓存能力），机器非空载时的负载未记录；因此本表只作**首轮观察**，不是协议化结论，不用于宣称"追平/超过"。

## 4. 正确性与可读性对照（本轮实做）

- 库级等价：prepared 与直接路径逐字段/逐字节相同（`tests/p3_prepared_input.rs`）；1 与 4 worker 逐方法文本/身份/顺序相同（`tests/bulk_recovery_workers.rs`）；单类装配改读取形状前后文本逐字节相同（`tests/class_source.rs`）。
- 命令级同源：`export_cli` 把真实 JSONL 与"进程内 `Engine::recover_all` + 记录 sink"逐条比较（除 elapsed 整条相等）。
- 跨命令一致性（本轮手工）：同一 fixture 上 `export` 的逐方法语句在 `class-source` 装配文本中逐字出现（3/3）。
- 可读性并排样本与 jadx 差异记录在 `/tmp/jarde-b10/comparison-notes.md`（bcprov 两个类 + S2-009 一个类）。

## 5. 未执行 / 未证实（不得读作完成）

1. **未**用本轮 `export` 输出重跑受控 JDK 编译执行对照（benchmark session 的 harness 读取旧 sweep 记录形状，未适配本轮的 JSONL 记录）。
2. **未**跑协议要求的 ≥10 次独立交错重复与 12 artifact 全集；**未**声明任何加速结论。
3. **未**验证"交付后释放 IR/局部事实"（任务 3.2）与旧串行完整 fingerprint 对照（任务 4.5）。
4. **未**实现 sink 的输出额度接入同一总账（任务 4.6）。
5. 机器负载与 page cache 初态未记录（`benchmark-review.md` 已把该缺口列为协议问题）。

## 6. 由本轮发现并立项/排期的问题

| 问题 | 处理 |
| --- | --- |
| 实例接收者被写成 `this_`/`arg0`（可读性缺陷） | 新变更 `openspec/changes/spell-the-instance-receiver-as-this`（0/7，含证据与反例要求） |
| 整包导出每类重解析容器目录 | 任务 5.4 已修（保留 store） |
| 整包导出缺少足够的默认总量维度 | 任务 1.2 已修（16 项可覆盖 + CLI 自有默认） |
| 并行收益受单线程交付限制 | 记录为任务 6.3 的裁决输入（未改实现） |
| `ClassMemberFacts::method_count` 在字段表截断时为 0，coverage 显示 "0 of 0" | 未决（锚点 `crates/jarde-reader/src/classfile.rs::class_member_facts`、`src/facade.rs::member_coverage`） |
| 单方法入口对同一 definition 读两次（driver + callee） | 未决（既有断言 `class_headers == 2` 钉住；批量路径已用 prepared 修好） |
