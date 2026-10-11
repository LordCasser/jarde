# 复合条件与 switch continuation 边界审计

范围：只读对照 `recover-proved-compound-statement-conditions` tasks 2.4/2.5 与 `recover-proved-switch-continuations` tasks 2.6/2.7、当前 source test 和归档 raw。没有运行工具链、Git 或 CLI，也没有修改仓库。审计基于当前 `region.rs` SHA-256 `65f6ef29dae25e4901a97b4d1b74d08b3c26856587817f5417a97de0930d512b` 和 `build.rs` SHA-256 `34c8d100ba8dcae10b0c1561874ca082312ab49a0f6c3662cb2357d3da1d586d`。

## 复合条件 2.4：拒绝边界

| 规格边界 | 当前覆盖与拒绝层级 | 证据 / 尚缺 |
|---|---|---|
| 外部入边、重复入边、非 Normal 入边 | `real_switch2_direct_target_join_proves_candidate_witnesses` 先从真实 TestSwitch2 Reader 图取得 proof 与 full rows，再给 `compound_canonical_edges_proved` 加一条外部→payload、重复 test predecessor、外部异常→join row，断言该生产共用的 canonical closure validator 返回 `false`。这是**validator-level 定向负例**：实际节点/基准 rows 来自 Reader，只在提供给同一 production validator 的 row iterator 中加入对抗边。OpenSpec 要求必要负例，并未要求每种 row 变体都必须由 classfile mutation 产生；此 seam 适合覆盖 canonical validator 的拒绝判定。它仍然不证明完整 Walker traversal、region integration 或 public fallback 的行为。 | `region.rs:17286-17293`, `17530-17609`；shared void-return 外部/重复/异常入边也仅直接调用 switch2 candidate helper：`18256-18281`。需将这项 validator-level 覆盖和完整 Walker/integration/public outcome 覆盖分开计数，不能因 rows 为 seam 输入就否认 validator 负例，也不能把它们外推成更高层已覆盖。 |
| frame / case | 真实 Switch2 compound proof 直接给 frame boundary、case entry 和错误 join，期待 `None`；case 例同时核查 visited、tail、catch 与 fragment 状态不变。共享 void-return continuation 正向接在 boundary 164、return 136；负例覆盖 arm frame scope 不含 return、outer frame 的 case_entries 含 return，并检查 walker/run 不变。属于**实际 proof/helper refusal**，不是公共报告 fallback 断言。 | `region.rs:18003-18084`, `17665-17766`。return continuation 的生产代码还检查 `outer_frame.scope`，但测试未单独让 outer scope 排除 return：`region.rs:4532-4553`。 |
| cycle / irreducible / 不可无重复归约 DAG | `real_compound_non_tree_conditions_refuse_without_walker_state_leaks` 用三份真实 class 输入，构建真实 Reader canonical/SSA，并调用 compound proof。JDK8/JDK23 的 class verifier 接受三个变体，但 Rust raw `observed-lib-v5` 在 irreducible 分支查找 pathless BCI 83 时失败；root 记录实际 IDs 为 0/29/40/62/51/74、unreachable 为空。修订后当前测试把该变体 physical 83 定位到 canonical interval 74，并保留 cycle/no-repeat 的精确 83 块身份，但**修订后的 source 还没有对应新 Rust raw**。旧 raw 顺序意味着单入口 cycle 分支在该 panic 前已执行；不可约分支 proof 尚未触达，no-repeat 分支没有运行。 | `region.rs:19018-19080`, `19210-19265`, `19273-19360`; cycle README `results/cycle-negative-validation-root-v1/README.md` 与 `observed-lib-v5/1.stderr.raw`。可计为 class 验证，不可计为 v5 Rust proof refusal / 无泄漏已通过。 |
| clone-path | 当前 `real_legacy_clone_target_with_different_path_refuses_switch_certificate` 用真实 jsr clone IDs，但调用的是 `prove_switch_fallthroughs`，不覆盖 compound proof。compound clone 新 fixture 的双 JDK `-Xverify:all` 通过；归档 `clone-negative-validation-root-v1/README.md` 明确写 Reader→compound proof Rust 负向待执行。 | `region.rs:18698-19014`; clone validation README；没有当前 compound clone Rust proof 或 public fallback evidence。 |
| 内部独立语句 / effect / store | production proof 会拒绝不在 condition dependency 闭包内的内部指令/store：`region.rs:7947-7979`；Builder 有条件依赖/写回约束。独立 `iinc`/`istore` fixture 的整类 Jarde default/all 为 ExplanationOnly，README 明确不能声称新增 Builder purity guard 已触达；生成类未恢复。相近 build tests 是短路值/assignment 场景，不是新 CompoundIf 普通语句 proof。 | `region.rs:7947-7979`; `build.rs:22435-22530`; `results/independent-effect-boundary-root-v1/README.md`。仍缺真实内部 effect/store 变体到完整 proof/Builder 的明确拒绝路径证据。 |
| public fallback / 无部分消费 | 对实际基线 Switch2 recovery 检查 structured/no fallback、完整 block ownership；但 above cycle/edge/clone/effect negative cases没有统一断言其 public `RecoveryContent`、fallback reason/source map，或“所有物理 block 都仍有唯一可靠呈现”。直接 proof/helper 的 `None` 只证明未产出 proof。 | 正向 Switch2 whole recovery：`region.rs:18086-18108`, `18174-18206`。tasks 2.4 仍未勾选：`tasks.md:11`。 |

整体判断：2.4 目前没有闭环。已真实执行的 compound closure 三种人工边变体是有价值的 shared-validator 负向；不能将它们扩写成真实 CFG 外部/重复/异常边 proof。最新 cycle v5 改正了输入的 canonical 身份假设，但其 post-fix Rust run 缺失；clone 和内部独立语句也仍待实际 Reader→proof/Builder 拒绝。现有失败路径多数停在 `Option::None` 的私有 proof seam，尚未证明对应 malformed class 最终落到何种公开 fallback。

## 复合条件 2.5：Stop 精度与原子性

- **compound proof early budget / pre-cancel：已精确 pin，但仅在扫描开始前。** `region.rs:17939-18000` 精确断言零步 Stop 为 `AnalysisSteps`, `written=0`, `limit=0`, `at=root_branch_bci`；pre-cancel Stop `at=root_branch_bci` 且 usage 0、visited 原样。这证明入口 poll/charge 传播，不能代表在 DAG 扫描/归约中途取消。
- **compound proof late budget：位置和 usage 仍是宽断言。** `region.rs:17905-17937` 用成功 usage−1 制造 late Stop，reason/dimension/written/limit 精确，但 `at: Some(_)`、usage 只要求 `0 < usage <= limit`。spec 要求真实位置和实际 usage，需从 post-fix/当前实际测试 raw 固定 stop BCI 与 usage；不可按 `limit` 推算 usage，因为失败 charge 可能一次请求多步且 charge 不消费。
- **compound integration late budget：位置和 usage同样未精确 pin。** `region.rs:17850-17904` 证明预算在 arm walk 后的 closure 过程中止、所有 Walker 状态恢复；`at: Some(_)` 与 usage 范围不足以 pin 停止地点/消耗量。取消在此 integration 无对应断言。
- **Builder budget：unit seam 已精确并有实际 raw。** `build::tests::compound_builder_proof_and_emission_stops_restore_the_trial`：preflight full 52 / limit 51 Stop `AnalysisSteps@59`, usage 51；emission full 68 / limit 67 Stop `@59`, usage 67；trial state 恢复。源：`build.rs:36898-36915`, `37069-37129`, `37168-37250`。实际结果在 `results/edge-negative-validation-root-v1/observed-lib-v2/1.stderr.raw`，完整库该轮 348/0，README 记明 v4 精准 pin 通过。
- **Builder cancel：仅在 preflight 完成后、调用 emitter 前同步取消。** 源注释明确不声称异步中途取消；仅要求 `Cancelled { at: Some(at) }` 且 at 属于 owned，没有精确 BCI 或取消后的 usage 断言：`build.rs:37252-37304`。可证明 emitter 入口响应 cancellation 和状态回滚，不能证明 emit 执行若干步骤后响应。
- **public API Stop：整体原子性有真实通过记录，但 source/Stop phase 粒度有限。** archived `public-stop-passed-v2` raw 有 8/0/0，实际 compound whole-recovery late Stop 是 `AnalysisSteps`, limit/usage 516, `written=633`, BCI 7，并断言 no binding/text/map；见 `builder-stop-validation-root-v1/README.md` 和该目录 `1.stdout.raw`。它纠正了早期错误假设“written 必须为0”。预取消 public 测试仅检查 cancelled 与无产物，没有精确位置/usage。永久测试位于 `crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs:999-1077`。root 对后续 public 修订实跑为 `2/8/2`，对应的 Stop 组 8 tests 通过；原早期编译失败raw保留，不代表当前文件失败。永久用例断言真实 stop BCI 属于物理指令、usage不超过 limit、written 等于 output_bytes、NotProduced 且 binding/text/map为空。

因此 2.5 中 Builder budget 的 reason/dimension/position/usage/state pins 已闭合；proof late、integration late、Builder cancel 和 public pre-cancel仍不足以逐项满足“实际 Stop reason、dimension、location、usage”的规范。任务行仍未勾选：`tasks.md:12`。

## Switch continuation 2.6：边界矩阵

| 规格边界 | 覆盖情况 | 证据 / 缺口 |
|---|---|---|
| ambiguous candidate | 真实 Switch2 测试证明唯一候选的正例和 candidate helper 的 source witness；没有 0 个 / 多个 join 候选枚举的反例。直接测试固定传 `join_node`，绕过 candidate discovery/uniqueness resolver。 | `region.rs:17286-17344`, `17436-17529`; tasks 2.2 要求零/多候选拒绝，2.6 仍显式列 ambiguity。 |
| external/duplicate/non-Normal incoming | Switch2 candidate helper 对 outsider→join、duplicate source、exception→join rows 返回 None；compound closure validator 另测 outsider→payload/duplicate test predecessor/non-Normal join。均由真实 canonical ID 生成 row variants，但不是完整 Reader graph 变体。FT2 continuation 测试移除实际 switch owner 96，保留真实 canonical rows，使真实入边在 owner 集外，`continue_switch_arm` 拒绝且 visited 不变。 | `region.rs:18256-18281`; FT2 `region.rs:16278-16322`; 根实际 `focused-validation-root-v3/observed` 和 edge-negative lib raw 为正向结果证据。 |
| shared non-join block / overlap | FT2 和 Switch2 正向检查 join/tail 外置一次；现有 switch overlap/case tests可证明旧 coverage 拒绝，但没有专门的 candidate join source 共享一个“不是 join”的内部 block、且触发新 continuation ownership refusal 的变体。 | FT2 `region.rs:16167-16221`; `tests/p3_switch_forward_join.rs:60-92` 正例只检查 BCI40 return once。任务2.6仍待负例。 |
| invalid case crossing / cycle / nested switch | `tests/p3_switch_forward_join.rs:108-125` 用 cross-case goto 与回边保证旧 forward-join public 输出保持 quoted；FT2/new Switch2 正向 coverage覆盖正常 case 边界。没有证据表明这些输入走到了新 tail/candidate consumer 的特定 guard；无针对 nested-switch candidate route 的新 continuation负向用例。 | `tests/p3_switch_forward_join.rs:108-125`; region内部已有 nested switch/loop的一般拒绝路径不等于新 helper专项覆盖。 |
| unknown terminal | 旧 `terminal_case_unknown_terminal_fact_refuses_certificate` 修改 Operation alias 后，证明旧 switch certificate 对终端 unknown fact 返回 None；不是新增 candidate join/terminal 路径的 class graph 变体。 | `region.rs:18627-18692`; 对新 continuation独立候选而言覆盖仍间接。 |
| path/clone | FT2 新 tail route 检查 physical path/scope，真实 legacy clone seam 则只验证旧 switch proof clone target mismatch；对 Switch2 candidate/current continuation 的 clone-path mismatch没有公开 Reader 实测。 | `region.rs:18698-19014`; clone Rust compound proof pending，continuation task要求也未勾选。 |
| partial source / fallback | FT2真实正例检查 structured、无 fallback、tail一次和source origins；cross-case/backedge public测试检查保留 bytecode quote且不单独输出join return。Switch2 full class正向已通过完整 CLI comparison，但不是负例。 | `tests/p3_switch_forward_join.rs:60-124`; `verification-root.md:21-29`。无对上述候选歧义/新异常/重叠变体的完整 public outcome检查。 |

2.6 所列歧义、共享非 join block、unknown/cycle/nested candidate scope仍有缺口。已有 FT2 cross-case/backedge、owner-removal拒绝和旧 unknown terminal test不能自动计成新 candidate shape 的 proof boundary；任务状态仍未勾选：continuation `tasks.md:14`。

## Switch continuation 2.7：Stop 观察

- **Switch2 candidate proof：精确入口 Stop 已测。** `region.rs:17345-17390` 对真实类 candidate helper 用 29 步边界精确断言 `AnalysisSteps@5`, usage=29；pre-cancel 精确 `Cancelled@5`, usage=0。对应 `real_switch2_direct_target_join_proves_candidate_witnesses` 的 helper，不是完整 method recovery 中途的 Stop。
- **FT2 tail continuation：精确 next 零预算 Stop 已测。** `region.rs:16224-16322` 先实际 walk switch arm 到 next=175，再以零预算调用 consumer，断言 `AnalysisSteps@175`, usage=0 和 visited unchanged。它核实 continuation `next` 位置，但停止发生在 tail 处理前，未覆盖已扫描若干 tail node/edge 后的 late Stop 或 cancel。
- **Switch2 compound/void-return continuation：late Stop 精度不足。** `region.rs:17768-17848` 预算−1时仅断言 `at: Some(_)`、usage `>0 && <= limit`; cancel 测试只断言 `Cancelled { at: Some(_) }`，不核对 usage。应先从新 raw 观察确定实际 `next`/terminal BCI 与 admitted usage，再把这两项精确写入断言；不要从 frame boundary 164 推断 stop BCI。该 continuation 与 switch2 candidate proof的精确 `@5` 是不同 phase。
- **适用范围/原子性：** continuation late budget/cancel 均核对 visited、tail、catch、fragment 与 run 没有部分改动；公开 FT2 zero-step 也保留原 owner。现有 `tests/p3_switch_forward_join.rs:127-159` 则只要求 incomplete/partial report，没有 reason/dimension/location/usage/source-map 精度，不能单独满足 2.7。

所以 2.7 有精确 candidate entry 与 FT2 continuation-start pins，但没有 late tail Stop 与 cancellation的精确位置/usage；完整 public matrix和回归验收仍待 root运行。任务仍未勾选：continuation `tasks.md:15`。

## 优先补证据（不降低 OpenSpec）

1. 对 compound proof 的 `limit = complete_steps - 1` 和 integration 的 `full_integration_steps - 1` 用一次受控实际执行观察 `StopReason.at` 与实际 usage，再固定精确 Stop 四元组；同样观察 Switch2 shared-return continuation 的 late budget/cancel位置与usage。
2. 先完成 cycle-v5 修订后的 Rust real-class test。其结果必须分别明确 single-entry cycle、irreducible、no-repeat 三个 fixture 哪些实际到达完整 proof、返回 `None` 且无 ownership 泄漏；不能将旧 v5 的 348/1 raw当作修订通过。
3. clone 变体、内部独立 effect/store、candidate ambiguity、共享非 join block、nested switch 与 unknown terminal，应在新机制的真实 Reader/class seam中区分“入口直接 guard”“完整 proof拒绝”“只在公开 fallback 表现为quoted”。未观察到具体层级前不记录已命中。
4. 最后对负例至少核可靠 source/fallback/owner outcome；纯 proof `None` 本身不证明 public output 没有部分消费。


## v2 边界说明

`compound_canonical_edges_proved` 的 edge-row tests 是被测 production validator 的合法单元级 seam：canonical IDs/基准图是真实 Reader 输出，变异只在调用该 validator 时追加/重复 row。spec 没要求外部、重复和非Normal三者各自都必须由合法 classfile 变体制造；这三项可记为 validator 判定已覆盖。它们不能等价成 `compound_statement_condition` 全 DFS、`compound_statement_region` arm closure，或公开 fallback/原子呈现已覆盖。

永久 public late-stop 测试现位于 `crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs`，v2根运行 2/8/2 通过；旧审计的“仅归档、无永久测试”表述已更正。公开测试的实现性断言是物理 BCI membership 和 usage 上界，actual raw 观察到具体 `@7`, usage 516, written 633；具体 BCI/usage 仍未作为源码常量 pin。
