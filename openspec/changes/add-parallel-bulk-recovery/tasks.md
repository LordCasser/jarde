## 1. 固定边界与反例

- [ ] 1.1 冻结实现起点、构建配置、输入/方法清单和已有正确性缺口；复用三项归档及 T1–T4 关闭证据，登记独立 T5/CLI 文档预算等边界，在新增的真实普通 worker 栈上复跑深链进程门禁；verification 不把工作树或旧主线程结果冒充固定候选（B01/B10；A13–A18）。
- [x] 1.2 固定库请求/事件/汇总及 CLI JSONL schema，记录本设计允许的 API 变更与 effective defaults；用 schema/序列样例验证物理身份、无 Body、重复声明、共享证据引用、末尾无 Final、0/非法配置均有明确行为（B01/B07/B08）。证据：`src/bulk.rs` 的请求/事件/汇总类型 + `crates/jarde-cli/src/export.rs` 的 JSONL 六种 `kind`（`header`/`class_prepared`/`method`/`diagnostic`/`class_end`/`final`）；`crates/jarde-cli/tests/export_cli.rs` 10 项覆盖 record 形状、物理身份与成员 ordinal、无 Body 与未产出分桶、`--jobs 0|abc|1.5` 拒绝、`final` 缺失即非完成；effective defaults 固定为：CLI `export` 自带 16 项 counted 维度上限（`1<<40`）、2 小时墙钟与 `FactsCapacity(1<<14, 1<<27)`，全部经 header 发布（本地实测两个语料默认配置整包到达 `final`）。允许的 API 变更：`OVERRIDABLE_BUDGET_DIMENSIONS` 从 5 项扩到 15 个 counted 维度 + `elapsed_millis`（`tests/task_operations.rs` 相应断言为本次有意改变，未知维度/零值仍拒绝）。
- [ ] 1.3 建立小型可再生语料和测量计数：flat/nested STORED/DEFLATED、同字节多 origin、M 方法大类、损坏后缀、深表达式及慢 sink；旧逐方法直接/共享 store 臂保存真实总账，验证计数不作为耗时结论（B01/B02/B10）。

## 2. 同类准备与事实所有权

- [x] 2.1 提供可信 prepared class 视图及多值 raw name/descriptor 方法 locator，沿用唯一 reader；M 方法回归验证一次结构/定位准备，重复声明、多个 Code、错误 owner/offset 和不完整表不会进入快路径（B02；A07/A14/A18）。证据：`crates/jarde-reader/src/prepared.rs` + `tests/p1_prepared_class.rs`（12 项，含逐字段解码等价、多值 locator 与歧义、重复/缺失 Code、成员表停止、记录复核拒绝、clear/零容量后的 backing 生命周期）。
- [x] 2.2 将 CP/Header payload 和可信 digest 交接改为共享/借用，锁内只做短操作；通过 clone/hash/解析计数验证同一持有事实不按方法复制重建，clear/零容量/内容变化不影响活动引用或原失效规则（B02/B09；A15/A18）。证据：`crates/jarde-reader/src/facts_cache.rs`（`Arc` 共享句柄 + `FactsKey::from_trusted`）+ `tests/p5_shared_payload.rs`（6 项）；既有 `tests/p5_facts_cache.rs` 13/13 未改一字。
- [x] 2.3 让 driver 和按需同类 callee 消费 prepared 输入，复用原分析/恢复流水线；与独立恢复逐方法展开证据对照，验证仍保留版本/环境绑定、debug、异常顺序、source map，未需求 Body 不被解码（B02/B07；A12/A16/A17）。证据：`jarde_jvm::{analyze_prepared_method_ir, read_prepared_callees}` + `tests/p3_prepared_input.rs`（6 项：逐字段报告等价、usage 形状无额外 class header、文本逐字节相同、callee 两入口等价、归档类等价、成员表停走不发布逐方法结论）；`tests/p3_declaration_handoff.rs` 只加强未放宽（prepared pass 零 `read_own_definition`、零 `method_code_facts(`、零 `ClassHeaders`，一次 `method_code(` 与一次 `MethodBodies`）。偏差：单方法/查询入口仍走直接读取（保持既有按需计费形状），prepared 入口由批量路径消费。

## 3. 串行批量闭环

- [x] 3.1 增加有界物理遍历交接和 workers=1 的 recover_all，按 scope/container/entry/member ordinal 连续处理；验证不先收集全包 Header/IR，所有可读声明包括无 Body 与失败项均可对账（B01/B02）。证据：`src/bulk.rs` + `tests/bulk_recovery_serial.rs`（6 项：交付顺序 == 独立游标顺序 == 声明序、逐类 `ClassPrepared→Method*→ClassEnd`、`class_headers == 0` 而单方法入口 `>= 18`、`class_bytes` == 各类字节和、无 Body 与拒绝项各自处置、超大类带位置拒绝、explanation_only 是结果不是停止）。
- [ ] 3.2 增加类型化 sink 和共享准备引用，结果消费后释放 IR/局部事实；用全量与单项对照验证正文/来源/规则一致，零保留及超过 cache 容量的 sweep 不回读仍持有的事实（B02/B07/B09）。已证据部分：类型化 sink 与共享准备引用已实现（`RecoverySink` 六事件、`ClassPrepared` 只交接身份与读取证据、`BulkRecoveryReport` 不含逐方法结果）；**未验证**：交付后释放与"零保留/超容量 sweep 不回读"缺分配器级或紧容量级断言，留待超容量 sweep 与真实语料。
- [x] 3.3 实现遍历/执行/交付分离的最终汇总及局部失败继续；受控坏方法、坏类、未知后缀与 explanation_only 验证分母、计数及 aggregate execution 均诚实（B01/B04/B07）。证据：serial 的计数分桶（declared/delivered/not_executed 与 outcome 桶、被拒类方法数未知只降 partial）、aggregate 状态断言；总账口径由 `tests/p1_budget_ledger.rs`（13 项）固定。

## 4. 总预算与有界 worker

- [x] 4.1 在原 charge/poll 通路增加批量共享总账和方法局部额度，校验累计/高水位/elapsed 的归属规则；用 barrier 控制两个任务竞争最后额度，验证不超限、不重置、不为许可失败工作计费、entry usage 延续（B04；A14）。证据：`crates/jarde-reader/src/ledger.rs` + `tests/p1_budget_ledger.rs`（13 项：barrier 竞争最后额度、entry usage 折入且不重置、高水位取最大、elapsed 为操作墙钟、溢出拒绝、首个停止不被覆盖、局部额度只停该工作）。
- [x] 4.2 实现只存在于本次操作的 scoped worker 生命周期及明确并发配置；注入创建失败/worker 异常，验证停止派发、全部 join、无遗留线程，至少两个类真实重叠执行且不复制 snapshot（B03/B06）。证据：`tests/bulk_recovery_lifecycle.rs`（4 项：`ClassGate` rendezvous `peak>=2`、`WorkerWatch.high_water==2` 且 `alive()==0`、`fail_worker_at` → `Failed` + `bulk_worker_spawn_failed`、worker panic → `Failed` + `bulk_worker_panicked`）。
- [x] 4.3 实现最多 W 个类、每类一槽的有序背压，固定单项容量并据总窗口缩减实际 worker 数；慢首类/快后类/超大方法或 ClassPrepared 记录/慢 sink 回归验证进展、权重上限、jobs 改变不缩小单项可接受上限（B05）。证据：`tests/bulk_recovery_backpressure.rs`（慢首类下顺序稳定、`buffered_weight_high_water <= limit`、超限单项零文本 + 可定位 `(weight,limit)` + 1/4 worker 同 ceiling）。
- [x] 4.4 实现总取消、局部方法停止、sink 主动停止及等待唤醒；在准备/计算/满槽/交付各站点取消，验证不派新任务、协作退出、执行未交付仍计费且无法取得容量的单项不挂起（B04/B05/B06）。证据：`tests/bulk_recovery_cancel.rs`（6 项：预取消零事件、跨类取消保留已确认前缀、满槽等待被唤醒、交付回调内取消、`StopAt` → `Sink` stop、`Err` → `Failed` 且前缀 == 已确认条数）。
- [ ] 4.5 加入强制调度扰动与完整/中止对照，验证 1/N worker 的完整语义及排序一致、中止子集真实；保留旧串行完整 fingerprint，仅为新并行资源记录登记固定差异白名单及总账核对（B07；A15/A18）。已证据部分：1 vs 4 worker 的逐方法 fingerprint/key/outcome/text 与 summary 指纹（忽略 worker 数与 usage）一致；**未做**：旧串行完整 fingerprint 对照与固定差异白名单登记。

## 5. CLI 流式导出

- [x] 5.1 增加薄 export 命令及 jobs auto/N、显式 scope/policy、局部与总限制参数；真实 CLI 测试验证 effective config、非法配置、文件 create_new、一个进程一次库操作，无每方法进程或重复分析（B08）。证据：`crates/jarde-cli/src/export.rs` + `crates/jarde-cli/tests/export_cli.rs`（7 项：auto/1 单进程一次操作且 header 报 requested/effective、`--jobs 0|abc|1.5` → exit 2 且不建文件、已存在输出被拒且字节不变、`--roots` 数组与重复 `--root` 等价、方法记录 == 同进程库值）；`--roots @FILE` 数组形态支撑 54 个装载位置的真实 WAR（本地实测）。
- [x] 5.2 增加有界 JSONL 编码和逐记录交付确认；注入输出超额、部分写/flush 失败及取消，验证完整前缀、残缺末行处理、缺 Final 非完成、退出 2/4 与库 summary 一致，不能绕过额度补写成功（B05/B08）。证据：export_cli 的小额度（仅 header／半额／仅缺 final 三种）与遍历受损用例（exit 4、final 诚实、前缀完整、无残缺行被计交付）；`ulimit -f` 注入 I/O 失败 → exit 2 且前缀恰好一条完整记录；两条反例（去掉额度、为 final 绕过额度）均使测试变红后还原。**未覆盖**：部分写回滚分支在 macOS 不可证伪（写整块返回 EFBIG），保留为 ENOSPC 防线。
- [x] 5.3 更新能力清点/隔离测试：显式 bulk 可有 scheduler，普通 open/query/recover 仍无隐式批量；验证 reader/query 依赖隔离、单方法局部性和 CLI/库同源，不仅删除旧“无调度器”断言（B03/B08；A16/A17）。证据：`tests/p5_benchmark.rs` 的源码守卫改成"只有 `src/bulk.rs` 可含调度拼写"，并新增 `no_ordinary_entry_reaches_the_bulk_module_or_recover_all`（19 passed / 1 ignored）；反例实验：非 bulk 文件插入 `thread::scope` → 守卫失败，`src/bulk.rs` 内同样拼写 → 守卫通过（两次探针已还原）；单方法局部性与 reader/query 隔离继续由既有 `tests/p3_isolation.rs`、`tests/task_operations.rs` 断言。

- [x] 5.4 让整包导出真正持有操作内保留：实测 `jarde-cli export` 在真实 jar 上只为 29 个类就耗尽默认总账的 65,536 条 `archive_entries` 额度（`owner=methods`），因为调用方没有挂 store 时每个类的准备都重新解析容器目录，成本按"容器条目数 × 类数"增长——这使整包导出在默认配置下既跑不完也失去与 jadx 同工作集对照的意义。证据：`crates/jarde-cli/src/export.rs` 为这次操作附带 `FactsCapacity::new(1<<14, 1<<27)` 并在 header 发布；实测 `archive_entries` 从 65,517 → 2,569（bcprov，容器 2,569 条）与 65,470 → 944（s2-009，53 个目录各解析一次）；库级整包 sweep 到达 `final`、`stop=None`、2,430/7,200 类与 15,003/57,180 方法全交付；`export_cli` 新增宽容器与保留形状用例，反例（去掉 store 挂载）三条用例同时变红；`tests/p5_benchmark.rs` 的 cache 构造守卫只新增"显式批量适配器"一处例外并保留 needle 自检。仍未解决：CLI 默认总量维度不含 `ir_items`/`analysis_steps`（见 1.2 的默认策略修复）。

## 6. 性能、回归与交付

- [ ] 6.1 固定 A–E 消融臂、OS/cache 初态、输出范围、样本和预先声明的退化/RSS 判据；先检查仪表开/关开销，再运行 1/2/4/6 worker 至少十次独立交错样本，保存冷端到端/CPU/RSS/首次结果和工作计数原始数据（B10）。 已做的部分：3 次交错重复、bcprov 与 s2-009 两臂、jarde jobs 1/2/4 与 jadx 默认/`-j1`，原始样本 `/tmp/jarde-b10/raw.jsonl`，口径与解释见 verification §3；**未做**：≥10 次重复、A–E 全集、事先声明的退化/RSS 判据与 OS cache 初态。
- [ ] 6.2 在 bcprov 与 nested WAR 对账全部物理类/方法，分别记录 jadx 1/N 线程、JVM、构造器/内联/resources/输出单位差异；用真实完整总耗时作比较，不以 p50 或更少恢复得出追平/超过结论（B01/B07/B10）。 已做的部分：bcprov 与 s2-009 的完整物理类/方法对账（2,430/15,003 与 7,200/57,180 全部 declared=delivered，`traversal_complete=true`）与整包总耗时对照（见 verification §3）；**未做**：jadx 侧方法级 join 对账、构造器/内联/resources 差异逐项登记。
- [ ] 6.3 跑超容量单次 sweep、大类倾斜、慢 sink、取消和普通小单请求回归；记录吞吐、尾部、工作集和同步瓶颈，作显式默认配置裁决，未证实收益不填成功，功能交付与性能主张分别验收（B03–B10）。 已记录的瓶颈：并发从 1→2 有收益、2→4 无进一步收益，最可能是交付（编码+写出）单线程且产物 357 MB–2.38 GB 串行写出；见 verification §3。**未做**：超容量 sweep、大类倾斜、慢 sink 与默认配置裁决。
- [ ] 6.4 固定候选运行 fmt/clippy/workspace、受控 JDK 编译执行与 debug/release 子进程门禁；所有新增关键不变量有去掉修正会失败的反例证据，真实语料不作为 CI 的 /tmp 依赖（B01–B10；A07/A08/A13–A18）。
- [ ] 6.5 完成 verification、公共示例、支持矩阵和父专项引用，执行 OpenSpec strict/链接与契约保留检查；仅按实际证据勾选并在归档时同步主 specs，不把本次规划或跨工具输出不等价写成实现/性能完成（B01–B10）。

## 7. 单类源码快捷视图（用户追加）

- [x] 7.1 增加库级 `class_source` 操作与装配模块：复用现有类视图事实与逐方法恢复结果，装配类头/字段/方法签名与缩进体；无 Body、未产出/拒绝、仅解释、带停止诊断分别用稳定标记出现在成员位置，成员不被静默省略；用真实 fixture 验证文本包含类声明与至少一个方法体、括号与缩进成对、歧义名不执行（B11；A07/A13/A16）。证据：`src/class_source.rs` + `Engine::class_source` + `tests/class_source.rs`（12 项）+ `crates/jarde-cli/tests/class_source_cli.rs`（13 项）。
- [x] 7.2 增加 CLI `class-source` 子命令与 `--format text|json`：text 输出装配文本，json 输出库自身序列化；一次进程一个类视图，退出码沿用 0/1/2/3/4 且不改变既有五个命令行为；验证重复运行文本逐字节相同、部分成员失败退出 4（B11；B08；A18）。
- [x] 7.3 记录该视图的实际读取形状与限制：一次请求内类 Header 读取次数与成员列举次数不随成员数重复、取消保留已确认成员且 usage 非零、不解析 imports/resources；把结论写入 verification 与支持矩阵，不把装配文本写成可编译工程（B11；A14/A16）。证据：`Engine::class_source` 改为消费一次 prepared 类（`read_prepared_definition` + `analyze_prepared_method_ir` + 与 `recover_method` 同一条呈现链路），`class_headers` 从 1+N 收紧为常数 2（绑定读 + 准备读）、`class_bytes` 恒为类自身长度的 2 倍、`method_bodies` = 实际解码数；接线前后输出文本逐字节相同；`tests/class_source.rs`（16 项）与 `class_source_cli`（13 项）加强后全绿，反例（改回每成员独立读）使形状测试变红后还原。仍未解决：无 import/package 装配、无类型推断命名、接收者拼写归 `spell-the-instance-receiver-as-this`。

## Acceptance Map

| ID | 主题 | 主要外部判据 |
| --- | --- | --- |
| B01 | 范围/身份/分母 | 物理清单逐项对账、重复候选与未知后缀 |
| B02 | 可信类准备 | 一次准备、非 M 次全表扫描、逐方法事实/行为一致 |
| B03 | 并发与生命周期 | 实际重叠、并发上限、创建失败和返回后零遗留 |
| B04 | 共享总账/局部限制 | 竞争最后额度不超限、已有使用量延续、局部与全局停止 |
| B05 | 背压/内存进展 | 慢首类无死锁、窗口有界、单项超限可结束 |
| B06 | 取消与未交付工作 | 各等待点唤醒、全部 join、未交付工作仍可核账 |
| B07 | 内容与覆盖 | 完整逐方法语义/来源相同、三种覆盖独立、部分集合诚实 |
| B08 | CLI 交付 | 同源 JSONL、有效 jobs、完整记录确认、Final/退出状态 |
| B09 | 复用容量 | zero/over-capacity/clear 不破坏活动事实、消费后释放 |
| B10 | 全量性能 | 冷端到端真实总账、独立重复、同工作集/质量对账 |
| B11 | 单类源码视图 | 可读装配文本、成员诚实标记、逐字节确定性、按需读取形状 |
