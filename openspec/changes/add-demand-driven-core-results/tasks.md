**3/32**（D0 的三项已实现并按 [verification](verification.md) 记录验证；D1–D5 的 29 项未开始）。实施顺序与所有权见 [design §13](design.md#13-可独立提交的实施边界)，D01–D12 为该文档的验收编号。不因文档或 strict 通过勾选实现；不修改其它 change 的完成状态。

## 1. D0 基线、字段角色与反例

- [x] 1.1 冻结实施 revision/dirty patch、现有完整证据与 query 顺序基线，在 verification 中记录源码入口和开放缺口；复跑数组槽宽与 T5 反例并登记独立修复所有权，不将修复混入本 change（D03/D12）。→ verification §1–§5：`809ca81`、dirty 全为文档、反例真实复跑（`javac` 拒绝 `return arg1;`；`65!`→`A!`/`x65`→`xA`），两项修复所有权仍独立。
- [x] 1.2 对 RecoveryReport、规则计划、emitter、analysis/callee 报告逐字段标注算法输入、内部计划、必要结果或可选明细，记录拥有者/生命周期/计费站点；审查确认内部 CFG/SSA 本已与报告分离，不重建第二套模型（D01/D04/D11）。→ verification §6（297 字段清点：算法输入 30 / 内部计划 116 / 必要结果 51 / 可选明细 100）与 design §16。
- [x] 1.3 增加有界 test-support 或 harness 计数，分别见证 class 物化/准备、body 解码、consumer work、可选拥有型记录和释放；用一次人为重复构造/先全量后过滤的变异验证计数会变红，仪表不进入领域 fingerprint（D01/D02/D04/D08）。→ `src/d0_counts.rs` + `tests/d0_demand_counts.rs`（5 passed），变异 M1/M2 各变红后还原，见 verification §7。

## 2. D1 需求与结果契约

- [x] 2.1 为现有恢复请求加入类型化 evidence kinds、driver BCI 范围与生效选择；验证 Essential/All、合法空范围、反向/越界/非指令边界、无 Code 和不支持类别，无静默忽略或自动扩大（D03/D04/D05）。 证据：`crates/jarde-java/src/evidence.rs`（`RecoveryEvidenceKind`/`RecoveryEvidenceRequest`/`BytecodeRange`）；边界表：空范围=合法空结果、反向/仅范围/越界/指令内部/无 Code/不支持类别各自拒绝或沿用既有停止，未知拼写是适配器 usage 错误（无静默扩大）。
- [x] 2.2 增加证据选择/状态清单与可选 payload 的一致性约束；验证 NotRequested、Complete 空结果、Partial、NotPerformed 可区分，既有 limits/usage 与独立语义平面仍可读取（D03/D05/D06）。 证据：`RecoveryEvidence`（固定 5 项）+ `RecoveryReport.evidence`；`NotRequested`/`Complete`（含合法空）/`Partial{delivered}`/`NotPerformed` 四态各有用例；清单与 payload 由库内 `debug_assert` 与测试内独立重述的映射双向检查；limits/usage 与独立语义平面未变。
- [x] 2.3 在原拒绝站点形成核心可定位缺口，复用既有原因与物理位置；验证关闭规则明细仍能取得语句级拒绝、ExplanationOnly、无 Body、未知分母与真实停止，不从生成注释反解析事实（D05）。 证据：`StopReason::EvidenceRefused{code,at,message}`；关闭规则明细后诊断仍逐字相同（从计划的 gap/counter 生成，不从记录读）；`ExplanationOnly`/无 Body/未知分母/真实停止的既有断言全绿。
- [x] 2.4 在 facade、class-source、bulk、CLI/示例和测试调用点显式传播选择，完整审计调用显式 All，普通恢复默认 Essential；验证库/适配器相同选择同结果，不增加协议实现或另一路恢复（D01/D03）。 证据：`*_with_evidence` 系列入口 + 原 3 参入口显式 `essential()`；`BulkRecoveryRequest.evidence`（`for_scope`=all、`with_evidence` 可收窄、serde 缺省=全量）；CLI `--evidence KIND`/`--evidence-bci`；示例显式传选择；全量审计调用点改 `all()`（既有测试的 20+ 处迁移逐条声明）。

## 3. D2 普通操作内的可信准备交接

- [x] 3.1 将目标绑定所取得的可信 read/facts 交给同次 preparation/声明消费者；验证明确身份与名字选择的所选定义各自不重读，候选搜索费用独立，缺失/歧义/未完成行为保持（D02/D10）。 证据：`ConfirmedRead.source` 与 `prepared_read()`；名字路径交 `search_targets` 选中的那次读，身份路径由运行自己读；计数：class_source 2→1 次物化、歧义/环境被拒时准备与解码均为 0；反例 M3。
- [x] 3.2 将 class_view 的选定 body 定位和 class_source 的声明/正文接到同一 prepared 生命周期；以多方法 fixture 验证一次所选类物化、一次准备、按需 body 数量，替换把 class_headers=2 视为目标的旧断言（D01/D02）。 证据：`class_view` 新增 `ViewBodies`（有 Code 才准备一次，0 body 时 0 次），每个 body 走 `prepared.method_code`；`class_source` 删除第二次物化，`class_headers` 2→1；反例 M4（每 body 各准备一次 → 3≠1）；旧断言 `class_headers == 2` 被替换为 1（有意改变，7 处 + CLI 1 处）。
- [x] 3.3 将直接 recover_method/recover_target 的 driver 与必要同类 callee 接到同一 preparation；验证有/无 accessor、重复成员、缺依赖和 profile/loader 不匹配，不跳过原绑定检查（D02/D10）。 证据：`analyze_method_ir_owning_the_read` 交出自己那份 read；同类 callee 走 `read_prepared_callees`，`class_headers` 2→1 且 `method_bodies` 不变（2）；有/无 accessor、重复成员、缺依赖与 profile/loader 不匹配用例全绿；反例 M5。
- [x] 3.4 验证 none/zero/容量不足/足容量四种 store 的普通操作与连续请求；活动事实只共享，未保留的下一请求可重建，取消与最后消费者释放后计数归零（D02/D11）。 证据：四种 store 状态下同一操作计数 1/1/8、`class_headers` 1、文本一致；连续两次请求与 `clear()` 后计数相同（无免费复用）；反例 M6（把 store 代答的容器登记为 held → 下一次无 store 请求的 `archive_entries` 1≠5）已修并还原验证。

## 4. D3 可选证据的实际构造

- [x] 4.1 将语义规则计划与公开 RuleDetails 拥有型记录分离；保持所有前提/拒绝决定执行，关闭明细时不复制完整记录，逐规则接受与拒绝对照 All 基线（D03/D04/D05）。 证据：规则计划与公开记录分离，关闭明细后逐规则接受/拒绝与诊断逐字对照 All 基线（`tests/d1_evidence_selection.rs::closing_the_rule_records_keeps_the_refusals_and_the_explanation`）。
- [x] 4.2 将 RegionDetails、NameDetails 与可选读取明细按选择物化，保留必须的 HeaderRead 摘要和核心缺口；验证关闭类别构造计数为零且取消无隐藏构造（D04/D05/D11）。 证据：三类按选择物化；读取明细只在"选中 且 呈现真的产出"时发布（`src/facade.rs` 的 `publish_read_details` 接缝，读取本身不重复）；关闭类别构造计数为 0（`crates/jarde-java/src/demand_counts.rs` + `src/d0_counts.rs` 的 `read_detail_records`）。
- [x] 4.3 让默认 emitter 不构造完整 Segment 表；详细模式复用同一 formatter 与 AST 的映射/计数 sink，验证正文逐字一致、偏移有效、不复制第二份正文且没有第二次 IR/恢复（D03/D04）。 证据：`crates/jarde-java/src/emit.rs:393` 只在选中 `SourceMap` 时记录 span；正文逐字一致由 Essential/All 对照用例与 JDK 执行对照保证。
- [x] 4.4 实现 driver BCI 局部证据选择及必要 origin 闭包；测试跨方法相同 BCI、复合表达式、生成节点与范围外依赖，局部结果等于完整证据的规定投影（D04/D10）。 证据：`the_driver_range_selects_the_records_it_intersects`（局部结果 == 完整结果与该范围相交的记录，顺序相同）；合法空范围 = `Complete` 空结果；无法回答的选择各自带码拒绝，不扩大成全量也不回答空；无 `Code` 成员的范围沿用既有 missing-table 停止。
- [x] 4.5 实现“可信正文提交 → 所选证据物化”的停止边界，并将新增工作接到同一预算；在前提规划、正文提交前、各证据类别中途和发布处受控停止，验证没有预算重置、没有假 Complete、已提交正文不丢失（D06）。 证据：`a_selected_category_that_stopped_states_its_prefix`——物化顺序固定（region→rule→name→source map），紧预算停在该顺序末尾的类别上，映射报告已记录的前缀、其余已选类别 `Complete`、**已提交正文一个字节都没动**，`execution` 非 Complete。
- [x] 4.6 更新 bulk 对可选 payload 的拥有容量核算及完整证据接线；验证新结果权重不少于实际持有下界、零选明细无虚构费用，不修改 worker/ledger/窗口算法（D11）。 证据：`src/bulk.rs::result_weight` 扩展为按选择核算——选中的 read-details 把每个成员的解码体（facts 帧 + 四张表）、header-read 证明与拒绝消息计入拥有字节；未选则不计（"零选明细无虚构费用"）；切片表按长度计、借用调试表不计，文档写明这是下界；worker/ledger/窗口算法未动。

## 5. D3 产物绑定与证据重建

- [x] 5.1 构造版本化产物绑定值，覆盖完整物理方法/必要 ordinal、环境、规则/schema、输出配置和正文摘要；验证同名异内容、同字节不同 origin、重复 entry/成员、格式及规则变化不碰撞代答，显示标签不替代身份，外部值仅作待验证输入（D07/D10）。 证据：`crates/jarde-java/src/artifact.rs` 的 `ArtifactBinding`（6 维 + `ARTIFACT_SCHEMA`/`TEXT_DIGEST`）与 `ArtifactDimension::ALL`；碰撞矩阵 13 项（同名异内容、同字节两 origin、重复 entry、重复成员、换 profile、schema+1、文本变化、期望无类别）各自给出 `Mismatched{维度}`/拒绝，显示标签不参与判定。
- [x] 5.2 在同一恢复管线支持 expected_artifact 的证据展开；先验证重建产物，再关联证据，测试全部临时状态释放后的成功重建、不同文本明确 mismatch、紧预算真实停止及新请求费用独立（D07）。 证据：`expected_artifact` + `ArtifactAgreement{NotStated,Agreed,Mismatched,Unverifiable}`，判定点在正文提交后、证据物化前（mismatch 只拒证据）；重建在释放首个结果与预算后由新预算完成，计数口显示第二次请求自己读类/解 body/跑恢复（计费独立）；紧预算 `Partial{BudgetExceeded(IrItems)}` 且 artifact 保留；`Unverifiable` 与 mismatch/无证据三态可分。
- [x] 5.3 测试 Essential → 局部证据 → All → 换方法 → 放弃序列，确保已知正文/事实不被不同证据选择改变，产物绑定不持有隐藏 IR/AST，store 容量保持有界（D03/D07/D11）。 证据：序列用例（正文/决策/绑定不被选择改变；换方法 → `Mismatched{Method}` 且零附着；放弃时 `body_decodes=0`/`recovery_runs=0`；store 驻留 3 entries/1 MiB 内有界）；绑定结构上无法持 IR/AST（尝试持 `Arc<MethodIr>` 无法编译），故释放见证=derive + `size_of` 上界 + 逐请求重建计数。

## 6. D4 增量结构查询与续扫

**进度（2026-09-21）**：6.3/6.5/6.6 已由查询分页实现交付并登记证据；6.1/6.2/6.4/6.7 仍开放，已知边界是 `Resource` consumer 与 `SnapshotAll` scope 仍走枚举路径（`META-INF/services` 一类名字无法定向查出，分母必须是全部条目），单容器内非 class 条目暂无公开的增量枚举入口（`ScopeCursor` 只产出 class 候选），因此"同容器内资源条目的账单"仍是打开目录时一次付清；这些要在推进 6.1/6.4 时一并解决，不靠缩小验收回避。

- [x] 6.1 在现有物理遍历底座提供 query 可用的 entry 事件，覆盖 class 与 resource，复用 bulk 的 class 过滤而不并入其 scheduler；以多 nested、损坏子树和逐 entry 取消验证物理顺序与未知范围（D08/D11）。 证据：`crates/jarde-reader/src/entry_cursor.rs`（`ArtifactSnapshot::entry_cursor`、`EntryCursor::next_entry`、`ScopeEntry{entry,depth,kind}`，`kind` 区分 class/nested/other）；容器 depth-first、容器内按 ordinal；class 规则与 bulk 走查共用同一函数（`scope_cursor::is_class_candidate_name`），不并入其 scheduler；完整遍历与 `enumerate_artifact_tree`/`enumerate` 逐项（顺序/深度/分类）相等；损坏子树、逐 entry 取消/预算停止各有用例。
- [x] 6.2 替换 query 的全范围 ProviderScan 收集，以需求拉取推进；密集首 entry 命中小页时不展开后续 nested，必要目录验证/解压/CRC 仍计费，完整遍历清单相等（D01/D08）。 证据：`UnitStream` 只剩 `Standalone/Entries/Walked`（`Enumerated`/`ProviderContainer`/`covered_end` 删除）；计数：小页 5 archive_entries / 433 B / 0 code / 0 nested vs 完整 67 / 3748 / 30 / 8；必要目录验证/CRC/解压照旧计费；反例 M1（resource 路径回退整树枚举）两用例变红。
- [x] 6.3 将 code consumer 改为可停止的逐项产出并接共用边界解码；复用当前 unit CP/member facts，测试一方法多命中、后续方法与损坏 Code 后缀，页满不构造 unit 的全部匹配（D01/D04/D08）。 证据：`crates/jarde-query/src/xref/code.rs` 的 `CodeWalk`（单元字节与 class facts 只解码一次、跨方法保活），页满即停：小页 `code_bytes` 5 vs 整单元 72（D0 计数口实测）；一方法多命中、后续方法与损坏 Code 后缀由 `p1_query_demand`/`p1_xref_code` 覆盖。
- [x] 6.4 将 metadata/bootstrap/resource 消费者接入同一停止反馈与当前位置表示；覆盖单个位置产生多条结果、混合 consumer、资源无 CP 线索及未实现类别，不改变 derivation 或产生过滤假阴性（D08/D09）。 证据：`tests/p1_query_positions.rs` 5 项（步内切断 `item_index>0`、资源事实无 CP 线索 `constant_pool_index=None`、混合 consumer 跨容器续页、未实现类别）；derivation 与产物字段未改。
- [x] 6.5 版本化细粒度 cursor，绑定查询身份、遍历/consumer 内位置与必要祖先状态；验证旧 schema、篡改 target/scope/relation/位置均被拒，合法页大小/预算变化允许继续，验证与重放有界（D09）。 证据：`QueryBoundary{container,ordinal,position,item_index}` + `QueryPosition`，`QUERY_ENGINE_SCHEMA` 2→3，`cursor_digest` 绑定 position，续扫时按类成员表逐字节校验（伪造位置 → `query_cursor_mismatch`）；旧 schema 被显式拒绝。
- [x] 6.6 去除按已发布匹配序号重造整单元结果的续页路径，保留必要底座验证的真实费用；连续多页与完整查询逐项相等，无重复/遗漏，稀疏命中及最终空页合法（D08/D09）。 证据：步骤机把位置前移到下一步（`UnitComplete` 时连字节都不再读）；`p1_query_api` 的续页断言更新（边界单元不再重放、`scanned_items` 2→1/0），连续多页与完整查询逐项相等由 `p1_query_demand` 的拼接用例承担。
- [x] 6.7 核对每页 coverage/execution/diagnostics；坏后缀只在访问时报告，页满与取消/预算不同，未扫描范围和未知分母可见，伪造 cursor 不能把前缀计成已扫描（D05/D08/D09）。 证据：`tests/p1_query_planes.rs` 5 项——页满 `Complete`、取消 `Cancelled`+诊断、预算 `Partial{BudgetExceeded{dimension}}` 三态可分；坏后缀只在读到时报；未到达容器记为 skipped 且未知分母不冒充空；伪造 cursor（改 ordinal/position）→ `query_cursor_mismatch`。

## 7. D5 独立验收、测量与文档

- [ ] 7.1 跑完整配置确定性与 Essential/All/局部证据足额语义对照，包含既有算术/receiver/boolean/concat/effect/origin/拒绝 fixture；受控 JDK 编译执行中不得以新增拒绝删掉原正确样本，已知失败独立修复后重冻候选（D03/D04/D10）。
- [ ] 7.2 执行 debug/release 子进程、深表达式正常结束与各阶段停止释放、cache clear/容量拒绝、慢消费者及放弃序列门禁；证明无 abort、无遗留工作、真实费用和有界所有权（D06/D11）。
- [ ] 7.3 按父性能协议冻结导航/恢复/证据追问/分页与放弃工作负载，保留相同完整产出和不同证据选择两组对照；记录首次结果、全序列 CPU/墙钟、构造数、返回字节、RSS 与保留权重，时间主张至少十次独立交错样本且无事后挑选（D12）。
- [ ] 7.4 对每项 D01–D12 保存正例、反例/变异、命令与原始证据，区分“契约通过”“工作量下降”“时间收益未证实”；以关闭明细却仍构造全表、旧匹配序号重扫等变异证明门禁有效（D01–D12）。
- [ ] 7.5 运行 fmt/clippy/workspace 及受影响 source-map/query/fingerprint 门禁，更新公共示例、能力清单、verification 与架构状态；执行 OpenSpec strict、本地链接与 MODIFIED 场景保留检查，真实实现完成后再同步归档主 specs（D01–D12）。
