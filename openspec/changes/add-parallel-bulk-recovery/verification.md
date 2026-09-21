# verification — add-parallel-bulk-recovery（实施中，2026-09-21）

本文件按任务逐条记录**实际执行过**的证据；未执行、未证实的部分单独列出，不勾选、不换算成"已完成"。
以下 §1–6 为实施时记录，当时在 `/Users/lordcasser/workspace/projects/jarde` 的未提交工作树执行；提交后的独立 review 冻结在 `e06fe14`，见 §7–8。实施时语料为 `openspec/benchmark-protocol.md` 登记的 vulhub 12 个 artifact（本轮 12 项字节数与 sha256[:16] 全部复核一致）。

## 1. 实施时全量门禁（历史记录，非本次复跑）

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
| 3.1 有界遍历 + 串行闭环 | **复核重新打开**，见 R1/R4 | `src/bulk.rs`；`tests/bulk_recovery_serial.rs` 6 项 |
| 3.2 交付后释放 / 超容量 sweep | **存在反例**，见 R2/R5 | 类型面不含逐方法结果、事件按产生顺序交付；缺分配器级或紧容量级断言 |
| 3.3 汇总与局部失败继续 | 已实现已验证 | `tests/bulk_recovery_serial.rs` 计数分桶与 aggregate 断言；`tests/p1_budget_ledger.rs` 13 项 |
| 4.1 共享总账 + 局部额度 | 已实现已验证 | `crates/jarde-reader/src/ledger.rs`；`tests/p1_budget_ledger.rs` 13 项（含 barrier 抢最后额度） |
| 4.2 scoped worker 生命周期 | 已实现已验证 | `tests/bulk_recovery_lifecycle.rs` 4 项（rendezvous 重叠、`WorkerWatch` 零遗留、创建失败与 panic → Failed） |
| 4.3 有序背压 | **复核重新打开**，见 R1/R7 | `tests/bulk_recovery_backpressure.rs` 2 项（慢首类、权重上限、超限单项局部停止、1/4 worker 同 ceiling） |
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

原始样本原引用：`/tmp/jarde-b10/raw.jsonl`（每行 `tool/artifact/config/wall_s/exit/output_bytes`）。**提交后复核：该目录不存在，仓库亦无原始样本/运行脚本；上表仅保留为历史汇总，不能独立重算统计量。**

**可读出来的结论（按同口径）**

- 同并发对照（jadx `-j1` vs jarde `jobs1`）：bcprov **1.11×**、s2-009 **1.71×**。相对本 change 的起点（逐方法 sweep 单线程 2.65 / 5.10 ms/体、整包 40.2 s / 296.5 s），整包导出已从"慢 12–46×"进入同一量级。
- 对 jadx 默认并发：bcprov **1.48×**、s2-009 **2.62×**（jarde 最优配置）。
- **并行收益有上限**：jobs 从 1 增到 2 有收益（s2-009 −21%、bcprov −14%），2→4 无进一步收益。**交付串行**是源码事实，产物为 357 MB–2.38 GB；它是否主导耗时尚无阶段计时证明。总账锁竞争、类倾斜和有序等待也未排除，串行路径另有已复现的固定等待（R1）。这些是任务 6.3 的待测假设，不能称已定位唯一瓶颈。
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
| 整包导出每类重解析容器目录 | 任务 5.4 只修 CLI 足额 cache 路径；crate 的活动事实交接仍存在 R2 |
| 整包导出缺少足够的默认总量维度 | 任务 1.2 已修（16 项可覆盖 + CLI 自有默认） |
| 并行收益在 2→4 时消失 | 归因未证实；先修 R1，再按 design 的消融/阶段观测定位 |
| `ClassMemberFacts::method_count` 在字段表截断时为 0，coverage 显示 "0 of 0" | 未决（锚点 `crates/jarde-reader/src/classfile.rs::class_member_facts`、`src/facade.rs::member_coverage`） |
| 单方法入口对同一 definition 读两次（driver + callee） | 未决（既有断言 `class_headers == 2` 钉住；批量路径已用 prepared 修好） |


## 7. 独立 review：e06fe14（2026-09-21）

**裁决：本 change 未完成，不归档。** 已有 prepared/worker/CLI 主体及原测试证据成立；它们没有覆盖下列反例。重新打开 tasks 3.1、4.3，补回遗漏的 4.6，checklist 从 17/26 改为 **15/27**。这些是新反例触发的状态修订，不表示原测试失败。本轮未改生产代码，不把 review 记录当作修复提交。

| ID / 级别 | 固定候选上的证据 | 根因、修复与验收 |
| --- | --- | --- |
| R1 / P1 | 4 类 / 18 方法，300 ms 总 deadline：jobs1 实测 515 ms，18 条已交付但 `final=false / partial / ElapsedMillis`；jobs2 为 14 ms、jobs4 为 8 ms，均 Complete。无限 deadline 的 jobs1 仍约 508–511 ms | `src/bulk.rs:2186` serial 调用 `dispatch_class` 后不 pop/finish slot；`:2293` drain 为第一个永远没有 ending 的旧 slot 等待 500 ms。slot 元数据还随已完成类数增长。完成时立即退役，验证实际槽高水位及正常结束没有无工作等待；不能仅删除 timeout 掩盖遗漏生命周期 |
| R2 / P1 | 同一 4-entry flat fixture、18 方法：无 store / 零容量 / 1 byte 容量均计 `archive_entries=92`；足额 store 为 4；none 的 discovery=20、methods=72，retained 为 4/0；正文/方法数均完整 | `ScopeClass` 仅 locator；`src/bulk.rs:1553` 重新 `snapshot.prepared_class`，后者再 `container_facts`。游标现有 Arc 没有交接给任务；`engine.rs:1347` 前的 prepared loader 绑定仍经 `providers.rs::candidates_at` 寻址目录，每方法再付 4 entries。保持可信绑定/CRC 校验，让准备及绑定查询消费现有活动事实；none/zero/over-capacity/clear 与 nested DEFLATED 都不得重建仍持有的目录/backing |
| R3 / P1 | CLI limit=165147，写文件 164628 bytes，加已计费正文 4112，总计 168740，仍 exit0/Complete；单独无限额度输出 164647 bytes，报告只记 output_bytes=4112。库 Final `result_items=31`，返回总账为32（同一次 retained run） | `export.rs:350` 快照 allowance 为第二本账，sink 无计费许可；`bulk.rs:2902` 前后先取 summary，再为 Final charge。共同总账先准入，明确编码实际工作/确认交付边界，Final 自身费用也入终态；不能给停止报告偷偷另开余额 |
| R4 / P2 | ScopeCursor 刚创建、只产出第一类，都返回 CompleteWithinSchema；已取消的 standalone cursor 第一次仍 `Ok(Some(...))` | `scope_cursor.rs` 的 complete 初始 true、仅损坏时变 false，standalone 提前 return 未 poll。区分尚未耗尽和已穷尽；所有首次产出遵守取消。当前 bulk 外层 traversal_done/poll 屏蔽部分影响，但公开 reader API 会误导渐进消费者 |
| R5 / P1 | `max_class_bytes=1`，4 个 deflated class 最终全拒绝，仍物化并计 discovery.entry_bytes=1813、read_bytes=1156，prepared=0 | `bulk.rs:1443` 先 read_class，再比较长度；上限只挡 parser，未挡完整解压分配。verified record 的 class 长度先准入，物化时继续 enforcing bound；超大类拒绝不能先支付其全部内存。snapshot 自身 backing 与 class 新物化分开核算 |
| R6 / P2 | CLI header 的 method_limits 也是全部 counted `1<<40` 与 elapsed 7200000；无独立局部配置 | `export.rs:279` 将整包 budget.limits 原样作为 method_limits；单项 2 MiB retained ceiling 又只在构造完后检查。这是资源策略缺口，不能把形式有限等同可用工作集。分别声明/调节总量与方法额度，真实大方法压力下验证局部停止且其它方法继续 |
| R7 / P2 | `bulk.rs:1730` 仅 `text.len + 64*records`；不统计拥有 Vec/String capacity，亦未计入未序列化 RecoveryFacts | 与 design §5 的容量口径不一致；现有高水位测试只证明代理分数不越限。按现有契约核算归属容量/Arc 去重并验证消费后释放；RSS 继续独立测量，不新建通用 allocator |

另一个明确的优化空间（R8，尚无耗时归因）：`jarde-jvm/src/engine.rs:1387` 对每方法执行 `facts.constant_pool.clone()`，pool 类型为 `Vec<CpEntryFacts>`；`providers.rs:863` 的 `facts.clone()` 亦为完整 ClassFacts 深拷贝。reader 缓存命中的 Arc 共享没有贯穿 JVM 消费链。M 个方法、K 项 CP 的工作形状仍包含 O(M×K) 的复制；在父专项 O2/O5 比较 immutable Arc/借用传递，保持每方法可变 IR 独立。源码能证明重复复制，不能据此声称它占总耗时多少。

上述耗时是本机 **debug 小 fixture 的反例观测**，不作为 jarde/jadx 吞吐比较，也不声称 jobs4 通常比 jobs2 快。R1 的固定等待路径与编译优化无关。R2 证明重复工作，不将 92/4 当作耗时倍数。

本次独立复跑：

```sh
cargo test --features test-support --locked --test bulk_recovery_serial --test bulk_recovery_workers --test bulk_recovery_lifecycle --test bulk_recovery_backpressure --test bulk_recovery_cancel --test p1_scope_cursor --test p1_prepared_class --test p1_budget_ledger --test p5_shared_payload --test p3_prepared_input --test class_source
cargo test -p jarde-cli --locked --test export_cli --test class_source_cli
```

第一条 **83 passed / 0 failed**；CLI **23 passed / 0 failed**，合计 **106 passed / 0 failed**；`cargo fmt --all -- --check` 和 `git diff --check` 通过，`openspec validate --all --strict --no-interactive` 为 **21 passed / 0 failed**。全 workspace 1435 的数字仍属 §1 历史记录，未冒充本次重新执行。当前四个提交没有可取得的远端 CI run；当前 SHA 的 GitHub checks 查询返回 No commit found，不把本地提交等同 CI 成功。

后续路线以父专项 design §15 为准：先修以上生命周期/总账/活动事实缺陷，再复用 prepared 服务单点/选定方法、实现按需结果/证据投影与查询续扫，最后测量并行热点。历史 T5 等恢复行为缺口仍独立处理；新增 bulk 未代替其行为门禁。原始 benchmark 缺失、受控 JDK 执行对照、worker 默认栈深链、旧串行 fingerprint 及正式 B10 仍阻止关闭。

## 8. 小型复跑探针（保留源码，不依赖历史临时文件）

下面源码使用仓库已提交的 `tests/bulk_support.rs` 生成 fixture；保存为 `tests/review_e06fe14_probe.rs`，先创建 `/tmp/jarde-review-e06fe14`，执行 `cargo test --test review_e06fe14_probe --locked -- --nocapture`。它只打印当前行为，不以现状代替修复后的正确断言。复核后只删除本探针；生产测试修正时应把对应反例归入各自现有 target。

```rust
mod bulk_support;
use bulk_support::*;
use jarde::*;

#[test]
fn audit_current_bulk_boundaries() {
    let (snapshot, _) = open(flat_fixture());
    let mut setup = Budget::new(limits());
    let roots = container_roots(&snapshot, &mut setup, &FLAT_PREFIXES);
    std::fs::write("/tmp/jarde-review-e06fe14/flat.jar", flat_fixture()).unwrap();
    std::fs::write("/tmp/jarde-review-e06fe14/roots.json", serde_json::to_vec(&roots).unwrap()).unwrap();
    let env = environment(&snapshot, tree_scope(), roots);
    let mut cursor = snapshot.scope_cursor(&tree_scope()).unwrap();
    println!("CURSOR fresh={:?}", cursor.coverage_state());
    cursor.next_class(&mut setup).unwrap().unwrap();
    println!("CURSOR after_one={:?}", cursor.coverage_state());
    for (label, capacity) in [("none", None), ("zero", Some(FactsCapacity::none())), ("tiny", Some(FactsCapacity::new(1, 1))), ("retained", Some(FactsCapacity::new(1024, 1 << 24)))] {
        let mut budget = match capacity {
            Some(capacity) => Budget::new(limits()).with_facts_cache(FactsCache::new(FactsIdentity::current(), capacity)),
            None => Budget::new(limits()),
        };
        let mut sink = Recorder::new();
        let report = Engine::new().recover_all(std::slice::from_ref(&snapshot), &request(env.clone(), 1), &mut budget, &mut sink).unwrap();
        println!("CACHE {label}: entries={} read={} classes={} methods={} status={}", report.usage.archive_entries, report.usage.read_bytes, report.summary.classes_prepared, report.summary.methods_delivered, report.summary.status());
        println!("OWNERS {label} discovery_entries={} method_entries={}", report.discovery_usage.archive_entries, report.method_usage.archive_entries);
        for event in &sink.events {
            if let Recorded::Final(event) = event {
                println!("FINAL {label}: {:?}", event.summary.execution);
            }
        }
        println!("RETURN {label}: {:?}", report.summary.execution);
    }
    let mut budget = Budget::new(limits()).with_facts_cache(FactsCache::new(FactsIdentity::current(), FactsCapacity::new(1024, 1 << 24)));
    let mut sink = Recorder::new();
    let req = request(env.clone(), 1).with_capacities(1, DEFAULT_MAX_RESULT_WEIGHT, DEFAULT_MAX_BUFFERED_RESULT_WEIGHT);
    let report = Engine::new().recover_all(std::slice::from_ref(&snapshot), &req, &mut budget, &mut sink).unwrap();
    println!("CLASS_CAP cap=1 entry_bytes={} read_bytes={} refused={} prepared={}", report.discovery_usage.entry_bytes, report.discovery_usage.read_bytes, report.summary.classes_refused, report.summary.classes_prepared);
    for workers in [1, 2, 4] {
        let mut total = limits();
        total.elapsed_millis = 300;
        let mut budget = Budget::new(total).with_facts_cache(FactsCache::new(FactsIdentity::current(), FactsCapacity::new(1024, 1 << 24)));
        let mut sink = Recorder::new();
        let report = Engine::new().recover_all(std::slice::from_ref(&snapshot), &request(env.clone(), workers), &mut budget, &mut sink).unwrap();
        println!("DEADLINE workers={workers} elapsed={} delivered={} final={} status={} stop={:?}", report.usage.elapsed_millis, report.summary.methods_delivered, report.final_delivered, report.summary.status(), report.stop);
    }
    let (single, _) = open(SCOPE.to_vec());
    let mut cursor = single.scope_cursor(&PhysicalScope::SnapshotAll).unwrap();
    let mut cancelled = Budget::new(limits());
    cancelled.cancellation_token().cancel();
    println!("CANCELLED_SINGLE yielded={:?}", cursor.next_class(&mut cancelled).map(|v| v.is_some()));
}
```

CLI 预算反例：`cargo build -p jarde-cli --locked` 后，用探针生成的 flat.jar 先执行一次 `export --jobs 2` 写到一个不存在的新路径；记完整文件字节数 N，再以 `--budget output_bytes=N+500` 写到另一个新路径。比较文件字节数与末条 `summary.execution.usage.output_bytes` 之和，当前仍能大于上限并 Complete。不要把文中确切文件长度固定成跨环境断言（elapsed 字段会改变位数）；正式回归应读取本次的字节数并独立核账。

## 8. 独立 review（`e06fe14`）的处置

review 的 R1–R8 全部有落点，逐条给出**修复位置与验证**；未闭合的留在原任务里，不勾选。

| ID | 处置 | 证据 |
| --- | --- | --- |
| R1 串行槽泄漏（500 ms 空等） | 已修 | serial 分支不再开槽；`BulkWindow.window_slots_high_water`；`tests/bulk_recovery_serial.rs`/`workers.rs` 钉住 jobs=1 在"实测工作+200ms"deadline 内 complete+final+18/18 且槽高水位 0，jobs=2 为 1..=2。反例：恢复旧行为 → 506–711 ms partial/无 final/`slots 4` |
| R2 容器事实依赖缓存准入 | 已修 | 活动句柄（`ContainerFactsHandle` + snapshot 的弱索引）由游标与类读登记：同一 4 条目 fixture 在 none/零容量/1 byte/足额四种配置下 `archive_entries` 都是 **4**（此前 92）；`tests/p1_active_facts.rs` 7 项。反例：删掉活句柄分支 → 92 复现 |
| R3 两本账 + Final 少计 | 已修 | `DeliveryAccount` 进 sink、CLI 自持额度删除、终态费用入总账；`tests/bulk_recovery_delivery.rs` 3 项把两层读数钉成等式并断言 `written <= allowance`；真实语料 limit 65,536 → `consumed = 写出 + 正文`、无 final、非零退出 |
| R4 游标提前报完整 / standalone 首项漏取消 | 已修 | `coverage_state()` 需 `exhausted && complete`；`advance()` 顶部先 `poll()`；`tests/p1_scope_cursor.rs` 新增 4 项（只取一项不得 Complete、取完才 Complete、预取消 standalone/zip 零候选、中途取消保留前缀）。反例：退回旧逻辑 → 3 项变红 |
| R5 `max_class_bytes` 晚检查 | 已修（reader 侧入口就绪，bulk 已有等价拒绝） | reader 新增 `prepared_class_within`/`prepared_root_class_within`：物化前按目录记录拒绝、读取中按剩余额度分块；bulk 侧当前用"读取预算 = min(调用方, max_class_bytes)"达到同样可观察结果（1 byte 上限时 `entry_bytes == 0 && read_bytes == 0`），反例：改回读后检查 → 1,813 bytes 复现。**接线到新入口**是待办的清理项（不改可观察行为） |
| R6 方法局部额度继承整包 ceiling | 已修 | `BulkLimits` 分离 `total` 与 `method`，CLI 取任务单请求默认；header/summary 同时发布两者；用例：局部 512 B 只让部分方法以 `StopReason::Budget{output_bytes}` 停止、`report.stop == None`、其余方法照常交付。反例：改回继承 → 变红 |
| R7 权重模型与 design §5 口径不一致 | 已修 | `result_weight` 按拥有容量核算（含 capacity、source map、各表、facts；写明 `Arc` 去重与口径外项）；最大结果权重 2,063 → 7,311；新用例要求 `weight >= owned`，旧公式必然违反。真实语料 bcprov：总和 51.4M → 138.6M，最大项 1.05M → 1.42M（占 2 MiB 上限 68% vs 此前显示的 50%） |
| R8 CP/ClassFacts 深拷贝（O(M×K)） | 已修 | `MethodIr` 持有 `Arc<ClassFacts>`、driver/FrameDeclaration/绑定路径全部按句柄传递；`tests/p1_active_facts.rs` 断言同类多方法的池切片**指针同一**（`ptr::eq`），18 次绑定只剩指针拷贝。反例：改回 `Arc::new(facts.clone())` → 指针同一性与源码守卫双红 |

## 9. 性能测量（`e06fe14` 之后，同机、release、空载）

原始样本：`evidence/raw-timing-interleaved.jsonl`（CLI，10 次交错/配置）、`evidence/raw-timing-first-pass.jsonl`（首轮 3 次）、归因表 `evidence/cost-attribution.md`。

- CLI `export` 中位数（10 次交错）：bcprov 4.41 / 4.44 / 4.65 / 4.88 s（jobs 1/2/4/6），s2-009 19.31 / 15.71 / 16.90 / 17.91 s。
- 归因（三档 sink，计数完全相同）：bcprov discard 3.26 / 3.50 / 3.84 s、encode 3.55 / 3.60 / 3.90 s、write 4.78 / 4.23 / 4.95 s；s2-009 discard 11.16 / 11.21 / 12.23 s、encode 12.93 / 11.63 / 12.75 s。
- 结论：**并行收益在操作内部就消失**（丢弃 sink 时 4 worker 不快于 1 worker），JSON 编码 0.3–1.5 s，写 357 MB / 2.37 GB 另占约 1.5 s / 6 s；"单线程交付是瓶颈"的旧假设被这组数据证伪。
- **热点已定位并修复**（`evidence/cost-attribution.md` 第二轮）：总账一把锁 + 每次 charge 两次临界区，entry 耗时随 worker 数 60 → 180 ns 且串行（bcprov 4 worker 临界区 2.91 s、s2-009 7.91 s）。改成按维度 cache-line 对齐的原子准入（CAS 仍是唯一许可、无预借、无本地累积，entry 数与计费表逐项不变）后：bcprov 4.17 → 3.12 s、s2-009 12.63 → 9.94 s（discard，3 次中位数），总账 43 ns/entry，**并行符号翻转**（1 worker 不再最快）。
- **下一瓶颈（未优化，架构决策级）**：窗口的有序交付——`take_front` 等待约占墙钟 94%，condvar 采样 70–75%；jobs=8 比 4 慢 5–7%，加深窗口无益。轮次延迟/最早类产量/窗口准入谁最终定界仍未证实。
- 方法集对账（6.2）：原脚本输出 bcprov 双向差 0、s2-009 仅 jarde 多 10（6 个 `<init>` + 4 个）。**该脚本丢弃 origin/package，不能证明物理一一覆盖。** 后续直接类声明清点仍确认 S2-009 的完整类名签名并集比历史 jadx 清单多 10；它们来自另一物理版本，不能等同“物理仅多 10 个”。精确分母及局限见 §11。

未做的仍在原任务：3.2 的重开项已闭、4.5 的旧串行 fingerprint 对照与差异白名单、6.1 的 A–E 全集与事先声明判据、6.3 的默认配置裁决、6.4 的候选固化与 JDK 门禁记录。

## 10. 测量本身的口径限制

本页所有耗时都是**一台机器、未控制 OS page cache、未记录同机负载**的单次或少量样本，只用来说明形状与归因方向，不构成吞吐结论，也不用于 jadx 对比；跨工具结论见 `evidence/comparison-notes.md`（那里写明两边产出单位不同）。


## 11. S2-009 同名多 origin 复核

本次直接读取 `S2-009.war` 及 nested JAR 的 class 声明表（只读 CP/this_class/method 表，未运行方法恢复），与已有 jadx join 文档的完整类名/name/descriptor 集合比较。输入 SHA-256：`dda30ca7a2587391e95bfc0b868726818311b1521dd15f68fd5feefd9ca1fc7c`。该清点不是恢复质量或运行时装载结果证明。

| 口径 | 数量 |
| --- | ---: |
| 物理 class 条目 | 7,200 |
| 不同声明类名 | 7,171 |
| 多物理来源的同名类组 | 29 |
| 物理方法声明（含无 Body 与构造器） | 57,180 |
| 完整 this_class + name + descriptor 的并集 | 56,892 |
| 历史 jadx 名称签名集合 | 56,882 |
| 物理记录超过名称签名并集 | 288 |
| 名称签名并集仅 jarde 有 | 10 |
| 名称签名集合仅 jadx 有 | 0 |
| 本次 class 声明读取失败 | 0 |

10 个额外签名全部位于 `WEB-INF/lib/commons-logging-api-1.1.jar`，对应的 7 个类也存在于 `WEB-INF/lib/commons-logging-1.1.1.jar`，两侧同名类的 SHA-256 **均不相同**。后者这 7 个类的方法签名集合与历史 jadx 清单逐类相等；前者含下列 6 个构造器与 4 个名为 `access$0` 的方法差异。因此应登记为“另一物理版本独有的名称签名”；不能删除它们以对齐数量，也不能计作 jarde 多恢复了 10 个正确方法。

以下类名前缀均为 `org/apache/commons/logging/`：

| 类名后缀 | 仅在该并集一侧出现的方法签名 |
| --- | --- |
| `LogFactory` | `access$0(Ljava/lang/String;)V` |
| `LogFactory$2` | `<init>(Ljava/lang/ClassLoader;Ljava/lang/String;)V` |
| `impl/SimpleLog` | `access$0()Ljava/lang/ClassLoader;` |
| `impl/WeakHashtable$1` | `<init>(Ljava/util/Enumeration;)V` |
| `impl/WeakHashtable$Entry` | `<init>(Lorg/apache/commons/logging/impl/WeakHashtable$2;Ljava/lang/Object;Ljava/lang/Object;)V` |
| `impl/WeakHashtable$Referenced` | `<init>(Lorg/apache/commons/logging/impl/WeakHashtable$2;Ljava/lang/Object;)V`；`<init>(Lorg/apache/commons/logging/impl/WeakHashtable$2;Ljava/lang/Object;Ljava/lang/ref/ReferenceQueue;)V`；`access$0(Lorg/apache/commons/logging/impl/WeakHashtable$Referenced;)Ljava/lang/Object;` |
| `impl/WeakHashtable$WeakKey` | `<init>(Lorg/apache/commons/logging/impl/WeakHashtable$2;Ljava/lang/Object;Ljava/lang/ref/ReferenceQueue;Lorg/apache/commons/logging/impl/WeakHashtable$Referenced;)V`；`access$0(Lorg/apache/commons/logging/impl/WeakHashtable$WeakKey;)Lorg/apache/commons/logging/impl/WeakHashtable$Referenced;` |

**对账器缺陷与证据限制。** `evidence/join.py:52–59` 只取 entry raw_name，丢弃 container 链及 ordinal；`:71–73` 又只保留 basename。例：`a/Foo.m()V` 与 `b/Foo.m()V` 会被判相同；两个 nested jar 内的同一路径、同签名会被 set 合并。descriptor 即使含引用类型也不能补回 owner/origin。本例用完整声明名重算后仍得到 10，但这不使旧算法可靠。

已读取的历史输入 `/tmp/jarde-bench/candidate-8586356/timing/jadx_join_struts2__s2-009__S2-009_war.json` 的 `dex` 为 **空列表**。方法集合吻合支持版本选择的解释，但缺少 jadx 实际来源选择的原始证据，不能宣布严格的物理 join 完成。其原始资产与来源映射仍需由正式 harness 保存，不能依赖该临时路径关闭 6.2。标准清点应复用 reader 的声明读取，不另建产品解析器。

产品裁决：物理枚举/export 保留所有定义；MCP 名字查询可按完整声明名展示候选摘要，按需展开 origin/字节身份差异；已有 loader/roots/profile 能唯一选择时显示该选择及其它来源，不能证明的关系保留未决。显式选定 origin 也不能绕过该环境的绑定校验；纯物理字节码检查与环境相关恢复各守其契约。相同内容最多复用不含来源/环境的解析事实，不合并身份，不默认共享受绑定影响的恢复结论。

## 11. 固定候选与基线（任务 1.1）

**候选提交**：以本轮最后一个实现提交为准（本文件随该提交一起提交，SHA 见提交记录；工作树态一律不当作候选）。**构建**：`cargo build --release -p jarde-cli --locked`；**验证**：`cargo test --workspace --all-targets --all-features --locked`（当前 1516 passed / 0 failed）+ 显式 `#[ignore]` 门禁（JDK 对照 3 passed、两条 release 深链门禁 2 passed）。

**构建与工具**：rustc/cargo 1.98.1（Homebrew）· macOS 26.6.2 arm64 · 12 核 / 24 GiB · release 构建用于本页性能读数 · OpenJDK 23.0.1（受控编译执行对照）· jadx 1.5.6（跨工具对照）。

**输入与方法清单**（vulhub 12 artifact；本轮逐项复核字节数与 sha256[:16] 与协议一致）：

| 语料 | 类 | 方法声明 | 无 Body | produced | explanation_only | not_produced |
| --- | --- | --- | --- | --- | --- | --- |
| bcprov-jdk15on-152.jar | 2,430 | 15,003 | 508 | 12,041 | 2,451 | 3 |
| S2-009.war（含 53 容器） | 7,200 | 57,180 | 3,635 | 44,501 | 8,726 | 318 |

方法集对账（6.2）：jadx 侧成员集与 jarde 的声明集 bcprov 双向差 0；s2-009 上 jadx 缺 0、jarde 多 10（6 个 `<init>` + 4），是**同名多物理 origin** 的物理盈余（`two-origins` 语料可复现同一行为：第二个 origin 的成员为 `not_produced`，class_end `Failed`，聚合 partial）而不是缺失。

**已有正确性缺口（不是本轮引入，也不由本轮修复）**：

- `org/bouncycastle/jce/X509LDAPCertStoreParameters.equal`：自 `8807fa5` 起由"有产出（带 fallback/循环区段诊断）"变为 `jre_recursion_bound` 拒绝（递归上界的代价）；
- `ClassMemberFacts::method_count` 在成员走查停在字段表时为 0（消费者若把它读成"声明了 N 个方法"会错，见未决项）；
- 单方法入口对同一 definition 读两次（driver + callee；既有 `class_headers == 2` 断言钉住）；
- 生成型语料（`tests/p5_bulk_corpus.rs` 的 6 类）不入 `corpus-fingerprint.json`，钉住它们的是该文件的计费表。

**局部类型 / concat 修正的实际状态**：均已独立归档——`2026-09-21-unify-local-type-decisions`、`2026-09-21-re-express-string-concatenation`（后者附四 job 绿的 CI 记录）。本 change 不代修、不重开。

**T1 进程门禁（普通 worker 栈）**：debug `a_deep_concatenation_chain_answers_in_a_subprocess`、release `the_deep_chain_answers_in_the_optimized_build`，以及本 change 新增的批量版 `a_deep_concatenation_chain_is_presented_by_a_bulk_worker`（debug，随 workspace 跑）与 `a_deep_chain_reaches_a_bulk_worker_in_the_optimized_build`（release，显式 `#[ignore]` 门禁）。批量版用 `--jobs 2` 把类任务放到库自己用**默认 stack_size** 创建的工作线程上；128 KiB 栈探针会以 `jarde-bulk-0 has overflowed its stack` abort，证明门禁针对的是 worker 栈而不是主线程。

**不冒充**：本页出现的所有读数都来自 release 二进制在**本机**的运行，未控制 OS page cache、未记录同机负载；跨工具比较写在 `evidence/comparison-notes.md` 并声明两边产出单位不同。

## 12. 默认配置裁决（任务 6.3 的当前结论，未完成项明列）

**已裁定的部分（有实测支撑）**

| 决策 | 值 | 依据 |
| --- | --- | --- |
| CLI `export` 的总量默认 | 16 个 counted 维度 = `1<<40`，墙钟 2 h | 默认配置下两个真实语料整包到达 `final`（bcprov 2,430 类/15,003 方法、s2-009 7,200 类/57,180 方法） |
| 方法局部上限 | 任务单请求默认（16 MiB 输出 / 30 s 等），**不继承**整包 ceiling | 局部 512 B 时只有部分方法以 `StopReason::Budget{output_bytes}` 停止、`report.stop == None`、其余方法照常交付 |
| 操作内保留 | 调用方 store 容量 `1<<14` answers / 128 MiB；CLI 默认附带，库不自行发明 | 无 store 时同一 s2-009 scope 在 30 s 墙钟内只推进到 5.7M `archive_entries`（应为 10,358）；附带后目录各解析一次 |
| worker 默认 | `--jobs auto` = `available_parallelism()` 再按窗口缩减，两个值都发布 | 头部记录；窗口不足时缩减而不缩小单项容量 |

**未裁定（需要架构决策，不在本轮）**：窗口的有序交付是当前第一瓶颈（`take_front` 等待≈墙钟 94%，采样 condvar 70–75%，jobs=8 比 4 慢 5–7%）。要动它需要换窗口设计（例如按类分批发布或允许更深的批次窗口），属结构性决策，不以"再调参数"收尾。此处如实记录为未完成，不勾选 6.3。

**本轮新增的相关证据**：活动容器随任务交接后，同一 3 容器/8 条目/4 类 fixture 在每个 worker 数与三种 store 配置下 `archive_entries` 恒为 8（此前按类数增长到 50）；查询分页的"页满即停"与 descriptor 统一（数组一槽）不改变导出路径的计费口径，但会改变同语料的方法文本（接收者与数组参数拼写），因此任何跨版本耗时对照必须固定候选。

**T5 与其它独立边界的登记**：append 参数转换（T5）已独立立项并完成（`make-required-conversions-explicit`，其 verification 记录执行对照）；数组槽宽归 `fix-descriptor-slot-facts`；接收者拼写归 `spell-the-instance-receiver-as-this`。本 change 不把它们的修复算作自己的成果，只在其影响本 change 读数时记录（例如同语料文本变化导致跨版本耗时对照必须固定候选）。

**CLI 文档预算边界**：`export` 的流额度与库总账已在 D2/4.6 后统一（同一本账、终态记录自身计入）；`recover`/`class-source` 等文档命令仍按既有 `output_bytes` 语义拒绝超限文档。CLI 深链门的"紧输出上限"不再 pin 具体停止种类（旧标定所依赖的冗余读已被 D2 移除），精确形状由库层 `p3_concat_conversion` 承担——这是登记在案的口径变化，不是放宽验收。

**深链进程门禁（普通 worker 栈）复跑**：debug `a_deep_concatenation_chain_is_presented_by_a_bulk_worker`（随 workspace 跑）与 release `the_deep_chain_reaches_a_bulk_worker_in_the_optimized_build`（`--jobs 2` 把类任务放到库自建、未设 `stack_size` 的线程上）在本轮最终候选上通过；128 KiB 栈探针会以 `jarde-bulk-0` 溢出 abort，证明该门禁针对 worker 栈。

## 13. 窗口二级额度的前后对照与默认裁决（任务 4.7 / 6.3）

**机制**：`BulkLimits.shared_result_pool_weight` = 声明总窗口 − 生效 worker 数 × 单项容量（`saturating_sub`）；每个活动类先用自己的**预留**（第一条结果 `charge = 0`，永不等待），其余结果进共享池（`charge = max(weight, 1)`），池满才等。池为 0 时规则本身就让每个活动类回到"一个待交付结果"，无需特例分支。

**探针口径（线程时间合计，每配置 1 样本）**：

| 语料 | jobs | `place_method` 首版 → 现在 | `take_front` 首版 → 现在 |
| --- | --- | --- | --- |
| bcprov | 4 | 8.00 s → **0** | 3.10 s → 2.12 s |
| bcprov | 8 | 20.63 s → **0** | 3.10 s → 1.56 s |
| s2-009 | 4 | 25.47 s → **0** | 9.41 s → 6.41 s |
| s2-009 | 8 | 67.53 s → **0** | 9.48 s → 4.96 s |

**纯构建墙钟（3 次中位数，交错；测量期机器非空载，load 6.6→13.8，作形状证据）**：bcprov jobs=2 3.04→2.53 s、jobs=4 3.09→2.10 s、jobs=8 3.23→1.76 s；s2-009 jobs=2 9.41→8.20 s、jobs=4 9.78→6.85 s、jobs=8 10.49→6.04 s；jobs=1 两语料持平（串行不经过窗口）。每次运行的交付计数完全相同（records/methods 全同），时间差不是少做了工作。

**6.3 默认配置裁决（本次可裁定的部分）**：有序窗口的二级额度**已确立为默认行为**——它按设计推导（不是开关），池容量随声明总窗口与生效 worker 数发布，池为 0 时与首版等价。两个真实语料上并行首次呈现单调收益（jobs=8 最快），因此"并行路径默认启用"有了非计时依赖：计数器与顺序断言在 1/N 下逐项一致（`bulk_recovery_workers`、`p1_budget_ledger`、`bulk_recovery_delivery`）。

**仍未裁定**：是否对外宣称"接近/超过 jadx"仍需 `optimize-demand-workloads` 的协议化测量（≥10 次独立交错、控制 page cache 初态与负载）；本页数字只用于形状与归因。

## 14. 6.3 的最终测量判据（事先声明）

**样本**：bcprov 与 S2-009 各一，`export` 与 `examples/bulk_scope_sweep`（discard/encode/write 三档）两种入口；`--jobs 1/2/4/8` 与 `auto`。

**口径**：每格 ≥10 次**独立交错**样本（配置交替执行），报中位数与极值；每次运行必须记录 ①退出码与 `status`、②交付计数（`records`/`methods`/`files` 口径见 `evidence/*.jsonl`）、③机器负载与 page cache 初态（是否先温一次）。**交付计数不同即该次样本作废**——不准用"少做了工作"换时间。

**分列结论**（不在同一段里混写）：①功能交付（退出码/final/计数/语义对照）②工作量（计费维度与目录解析计数）③时间收益（仅在 ≥10 交错且负载可记录时声明；落在噪声内记为未证实）。

**默认裁决的三条硬要求**：`--jobs auto` 的取值在两个语料上都必须落在"≥2 worker 且快于 jobs=1"的区间；退化形状（1 MiB 输出额度、零容量 store、超大类）在默认配置下必须是"可读前缀 + 非完成 + 非零退出"；`final` 缺失一律不当作完成。

**明确不做的**：不把本项目的任何数字写成与 jadx 的吞吐比较——那需要同工作集、同质量口径与同一台机器上的协议化对照，属 `optimize-demand-workloads` 的 O7，不在此处顺手宣布。
