当前实施状态：**0/22**。O1 的独立子交付 `bound-container-lookup` 已归档（8/8，`b22ea04`），但下列任务还要求专项工作负载、归因、原始样本及调查处置，不能仅凭子 change 归档勾选。`preserve-task-operation-stops` 已在 `8586356` 完成；后续恢复修正与开放缺口见[当前复测](../../completion-review.md)，其实现继续独立于本专项。

2026-09-21：全量多线程需求已确认；[add-parallel-bulk-recovery](../add-parallel-bulk-recovery/tasks.md) 唯一拥有 O2 恢复类准备、O4 bulk、O7 类间并行及必需的 O5/O8，实现主体已提交，各项实现与独立复核持续推进，状态以子项 tasks/verification 为准。两份任务表各自计数；完成子项规划不勾选本专项调查，也不等于父专项或实现完成。

当前顺序：独立关闭数组槽宽与 T5 语义缺口；[add-demand-driven-core-results](../add-demand-driven-core-results/tasks.md) 负责普通 prepared 交接、需求/证据产品和增量查询；bulk 原子项继续生命周期及全量门禁。父专项负责 W1–W6 的归因、连续/放弃序列与策略准入，不重复实现。具体阶段与所有权见 design §15。

## 1. G0 固定基线与测量入口

- [x] 1.1 冻结 baseline revision、构建参数及 fixture 摘要，复核 design 的源码观察；登记 `85828c4`、`8586356` 的历史行为边界，按当前完成复核及独立归档更新正确性修正、开放反例和 clone/toString 待核实项。正式候选按声明工作负载验收，区分仍存在、已修复和未确认项，验证未混入其它 change 的行为差异（A13/A14/A18）。 证据：`evidence/g0-baseline.md`——基线 `ba2076a`、rustc/cargo 1.98.1、`--release --locked`；fixture 由 24 个已提交 `.class` 内存构造（15824 B、blake3 `145f7903…`、24 类/183 方法记录），清单 sha256[:16] `3decfd21…`；真实语料 bcprov 2,903,072 B/`5329ddef…`（S2-009 因 root 绑定 snapshot 身份未纳入，理由已写）。design §1 逐行复核：**仍存在 6 条**（含 `container_candidates`/`container_record`、`ScanUnit::read_unit`、`class_view`/`body_result`、`method_code_facts`、`FactsCache` 接口、open/read_verified、`write_success` 长度固定点）、**已改变 2 条**（`tree_candidates` 不存在；"全量收集后分页"已被可停止扫描取代，须在 2.3 重测）、**未确认**：单方法剩余重读量与 clone/toString 历史记录。`85828c4..HEAD` 75 个提交（28 触及产品代码）逐条归类为行为/工作量/计数。
- [x] 1.2 扩展现有 P5 harness 的 W1–W5 固定工作负载与已明确的 W6a 全量 1/2/4/6 worker 序列，另登记 W6b 跨请求并发/合并的宿主需求或缺失理由；验证启动/open/准备/请求/输出与序列总成本可独立复现，不依赖临时 benchmark 目录。 证据：`tests/p5_optimize_workloads.rs`（W1–W5 固定工作负载 + W6a 的 1/2/4/6 worker × discard/encode/write）与 `evidence/g0-workloads.md`、`run-baseline.py`；四段（open/prepare/request/output）同钟互斥、重叠即 panic，启动=外部 wall−进程内 window，嵌套段只带 `parent`；fixture 内存构造、write 档落 `CARGO_TARGET_TMPDIR` 并显式拒绝 `/tmp`。**W6b 登记为暂缓**（无宿主并发需求证据，触发条件沿用归档 P5 §2.1 与 `docs/support-matrix.md`），harness 词表排除 `w6b` 且有用例钉住该拒绝。
- [x] 1.3 增加需要的工作计数及外部互斥阶段计时，补同结果的丢弃 sink / JSON 编码到计数 sink / 编码写文件消融与锁/有序等待观测；分开使用量、cache 驻留和进程 RSS；用仪表开/关及嵌套阶段样例验证无重复归因，阶段时间不进入领域报告 fingerprint（A15）。 证据：`evidence/g0-instrumentation.md`——三档消融（bcprov 1 worker 的 request 段 discard 3,844 / encode 4,157 / write 4,311 ms；`sink_encode` 307 ms、`sink_write` 145 ms）；总账六站点 `waited_calls=0`，有序等待 `take_front` 12,837 次/2.216 s（≈request 段 90%）、`take_task` 40,796 次/5.03 s（线程时间）；usage / store 驻留（`FactsReport`）/ 进程 RSS 三平面分列；仪表对照：挂 probe +5.9%（区间不重叠）、只带计数口 +1.4%（噪声内）、feature-off 为基线，三者 `domain` 指纹相同；9 个 harness needle 在 `src/**`+`crates/*/src/**` 零命中，`p5_corpus_fingerprint` 未变。
- [x] 1.4 为每组登记主指标、退化/内存界限、样本量与统计方法，保存至少 10 个独立基线样本及原始数据；验证可按同一配置重跑，尾延迟证据不足明确标记而非推广经验 p95。 证据：`evidence/g0-metrics.md` + 6 份 JSONL（fixture 20 配置×10、bcprov 18×10，另有两轮对照，failures 0）；重跑契约读数（fixture 70、bcprov 218 个比较项含 `domain` 指纹）0 处不同，仅时长与内嵌 elapsed 位数引起的 B/数十 B 字节差；结论三层分列（契约全绿；工作量给出冷/热 170 对 62 µs、批量准备 1 对逐方法 0、往返 26.1 ms 对 0.53 ms ≈49×、391.2 MB 编码 +307 ms 与写盘 +145 ms；时间层凡落在 spread 内一律记未证实，10 样本不推广 p95）。

## 2. O1–O8 分项调查

- [ ] 2.1 O1：复核已归档 `bound-container-lookup` 的定向访问、目录/backing 与跨请求保留证据，不重复实现；补足 B/D/C/W/F 在 W1/W2/W5 的归因和处置，验证 sibling/深度/重复名/容量场景齐全。历史 B 缺失或与当前语义不齐时明确不可归因，不用旧数字替代新样本（A07/A08/A15/A16）。
- [ ] 2.2 O2：测量 W1–W4 的 class 物化、结构解析、方法定位和 driver/callee 重读；先把已实现 prepared 能力在单方法/明确选定方法集合上的复用作为候选，不借此解码其它 Body；复核 642e49f 已共享的 CP/Header，当前重点是 bind/prepare/driver/callee 的剩余重读，不重做旧 deep clone 修正；跟踪 bulk 子项的恢复类准备与共享 ownership，普通入口与查询消费者的实施跟踪 demand-driven 子项，验证分步候选的独立计数、内存代价和歧义/策略约束，不重复实现。
- [ ] 2.3 O3：测量查询首屏、boundary 重扫和 unit 临时结果，按 demand-driven 子项测量 consumer lazy 执行、候选过滤、可停止 sink/续扫；以小页密集命中与坏后缀说明行为差异，验证拟修改的 cursor/coverage/diagnostic 契约被明确列出（A01/A14/A17）。
- [ ] 2.4 O4：比较 W2/W3/W5 原顺序与 bulk 子项的按类准备/逐方法交付，复核总预算、输出顺序及背压门禁；验证冷端到端包含准备和输出，不把主体已交付等同于所有生命周期与性能门禁完成。
- [ ] 2.5 O5：调查各 facts 层的复用距离、命中避免的工作、hash/clone/同步开销及超容量行为；提交分层身份、完整发布、byte admission/淘汰备选和直接回退矩阵，验证内容变化/依赖补齐不会误命中（A15/A18）。
- [ ] 2.6 O6：在固定导航和放弃序列上评估预热/预取，或记录缺少实际需求的暂缓依据；提交准备成本、未使用比例、前台延迟、争用和独立预算/取消模型，验证“提前支付”不被计成总计算节省。
- [ ] 2.7 O7：跟踪 W6a bulk 子项的 1/N worker、共享总账、结果窗口和取消实验，测量去重后的并行收益；W6b single-flight 单独核对实际需求、产品 key/依赖、订阅者独立预算及取消/失败/最后需求撤销，缺少需求只暂缓该候选，不阻断已确认的类间并行（A14/A15）。
- [ ] 2.8 O8：测量摘要/复制/分配/输出对相应总时长的贡献；区分所需分析与实际序列化的字段，跟踪 demand-driven 子项的默认必要结果及显式证据展开，分别度量必要分析、明细实际构造和序列化，始终保留身份、content/quality/coverage/execution 和真实拒绝/停止；比较增量摘要、可信 digest 传递、backing 共享、编码改进和 mmap 的约束；提交选择/否决依据，验证不可变 snapshot、校验和最终输出字节契约均保留。

## 3. G1/G2 准入与最小实验

- [ ] 3.1 汇总 O1–O8 的必要工作证据、`p/s/h` 或缺失项、理想上界、资源和语义代价，逐项作准入/否决/暂缓决定；用 5% 占比示例核对上界计算，并验证重叠收益未重复相加。
- [ ] 3.2 为每个准入项登记独立实施 change、具体 capability delta、直接路径及回退责任；验证 O1 仍由已有 change 唯一负责，其它项没有绕过协议/预算设计，规划未完成的子项不开始生产实现。
- [ ] 3.3 按准入项进行可撤销单因素实验，隔离算法、容器/CP 产品、batch/并发和预热状态；保存端到端、准备和容量退化对照，验证组合结果未误归给单项。未准入项以有证据的决定完成调查，不创建空实现。
- [ ] 3.4 根据实验决定实施或撤销，并复核所需库的现有能力、维护、许可及边界缺口；验证新依赖有明确必要性，收益未证实或资源超限的原型没有变成默认策略。

## 4. G3 实施跟踪、验收与回退

- [ ] 4.1 跟踪每个已准入子 change 的实际任务和验证记录，收集 O1 及后续独立交付的固定候选产物；验证仅引用真实通过的任务，仍未完成的子项保持待办，不在本任务中打包实施全部方案。
- [ ] 4.2 对已交付策略执行足额跨路径语义对照及紧预算/受控取消实验；既有串行及 bulk workers=1 的同初态完整对照仅删除 elapsed_millis，新并行路径按其显式 spec 核对完整语义和独立资源总账；验证不删除 origin/顺序/语义 diagnostics/coverage/规则/source map 来掩盖调度差异（A13/A14/A15/A18）。
- [ ] 4.3 演练适用的 cache 关闭/失效/容量拒绝、串行降级、预热取消、共享生产者失败和发布回退；验证剩余预算不重置、已发布前缀不重复、等待者不挂起，并明确不适用场景及原因。
- [ ] 4.4 对实际被准入的组合执行 W1–W5、已声明的 W6a 及适用的 W6b 回归，核对冷/热/一次性扫描、首屏/总量、内存/尾延迟和预算退化；按预先标准作显式启用或保持关闭决定，性能差异落在噪声内时标记未证实。

## 5. G4 重排与专项交付

- [ ] 5.1 每个阶段后重测剩余耗时占比与工作次数，更新 O1–O8 的排名、重叠关系及下一轮/停止决定；验证没有沿用旧热点占比，也没有因已有专项而自动实施剩余候选。
- [ ] 5.2 汇总可复现命令、原始样本、分项调查结论、子 change 状态、收益/代价及回退记录；运行适用代码检查与 `openspec validate --all --strict --no-interactive`，确认所有调查有处置、已准入实现无隐藏未完成项，并分别报告调查完成和产品交付范围。
