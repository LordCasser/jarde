# 合并后负边界回归与修复（recover-preceded-statement-catches）

基线为主线 `86f2e740`（含本变更全部已合入切片）。全仓 `cargo test --workspace --tests --locked --no-fail-fast` 暴露 8 个失败（任务原列 7 个；`task_operations.rs` 的 `the_presentation_does_not_re_read_the_text` 与 `an_explanation_only_result_and_a_stop_are_presented_in_order` 同为 `PostfixHandlerBoundary.update` fixture，未在原清单单列）。取证结论：**7 个负边界/呈现回归由 `f9da424c` 的候选门槛引入；2 个语料账本回归与本门槛无关，由同变更已验收的 `e7b722e0`（任务 2.2 concat 值流归属）引入**。两者分别修复与重测如下。

## 一、门槛回归（f9da424c）与修复

`f9da424c` 在 `guard.rs::resources` 候选循环加入"前置指令非 `Operation::Store` 则 `continue`"，无条件交回 catches。该 skip 同时吞掉了**行起点不在语句边界**的形状——这些行的行范围劈开一条语句或吞掉初始化，不是任何源语句能拥有的子句边界：

| 失败测试 | 行形状 | `before` | 行起点栈深 | 修复后行为 |
| --- | --- | --- | --- | --- |
| `a_range_beginning_inside_the_field_assignment_is_not_a_plain_catch` | 行起点移入 `field = 7`（`bipush 7; putstatic`）中间 | `bipush`（Push） | 1 | 保持检查，`jre_guard_resource_init` 拒绝，文本无 `catch (` |
| `a_handler_range_that_swallows_the_initialisation_is_refused` | 行 `[6,9)` 扩为 `[5,9)`，盖住资源自身 `astore` | `open()` 调用（非 void Invoke） | 1 | 保持检查，`jre_guard_resource_init` 拒绝 |
| `exception_handler_boundary_inside_postfix_chain_is_refused_with_real_bcis` | 行 `[7,12)` 起于 `dup2` 后的 old-value 链中间 | `dup2`（Duplicate） | 2 | 保持检查，`jre_guard_resource_init` + `jre_region_uncovered_blocks`，ExplanationOnly，无同一 BCI 双重呈现 |
| `explanation_only_means_no_emitted_statement` | 同上（同 fixture） | 同上 | 同上 | ExplanationOnly 恢复 |
| `an_explanation_only_result_and_a_stop_are_presented_in_order`、`the_presentation_does_not_re_read_the_text` | 同上（同 fixture，task_operations 面） | 同上 | 同上 | 恢复 |

**修复**（`crates/jarde-java/src/guard.rs`，commit `e0089c89`）：skip 加两个既有判据族条件，均不发明新机制——

1. `statement_ends(facts, before)`：`before` 必须是**语句终止型**操作——void 返回的 `Invoke`（`parse_method` 返回类型为 `None`，覆盖 `helper();` 与 `println(...);`）、`Field` 写（`completed_field_assignment` 所读语句的终点）、`Return`（`areturn` 结束 return 语句，是 FinallyOnce `handled` 的 catch 子句边界）。分支、monitor enter、栈重排、值生产者不是语句终点（`synchronized` 自身的 `monitorenter`/`goto`、`dup2`、`bipush` 由此全部留在检查路径）。
2. `statement_boundary(facts, before, row.start_bci)`：行起点处块内操作数栈深度为 0 且块入口无栈槽——`completed_field_assignment` 对自身赋值所做的同一完成证明，推广到任意语句；行范围因此不劈开任何语句、不吞掉任何初始化。

非边界或非语句终点的行不 skip，落入既有完整证明路径并以 `Unproven::ResourceInit` 等原因拒绝——与 `f9da424c` 之前的行为一致（`twr` 对非 Store `before` 在 `initialisation` 处必败，不存在"交回 catches"以外的第三种结局）。

## 二、语料账本回归（e7b722e0，与本门槛无关）与重测

`the_fixed_shape_bills_the_counts_the_corpus_pins`、`the_old_per_method_arms_keep_their_ledger_and_the_same_text` 失配为 `ir_items` 单维：`many-method-class` 20331→20277（−54），两臂 33784→33730（各 −54）；其余全部维度、全部行、成员文本与分类不变。

逐提交二分（`d88ed16d` 过、`f9da424c` **过**、`e7b722e0` 败）证明本失配由 `e7b722e0` 引入，非本门槛。因果链：**`e7b722e0` 值流归属 → head 58 链由拒绝变为 presented → `jre_concat_split` 拒绝诊断及其证据记录消失 → 物化记录计费 −54 `ir_items`**。机制（CLI 逐成员取证）：`e7b722e0` 的值流归属使 `Guarded.suppressedCatching` 循环链（head 58）不再以 `jre_concat_split` 拒绝——旧检查以 catch 子句入口块的首个 `toString`（BCI 29，非本链消费者）错误拒绝该链；新检查限定 `bci > head` 且经接收者有界回溯归属，链 58 无跨块消费点，按任务 2.2 "否则不以此码拒绝"交回 walk，诊断由 `jre_concat_split` warning 变为 `jre_concat_chains` "2 presented, 0 refused"，证据面物化记录随之 −53（全类 −53/−54）。成员整体保持 `Quoted`（`p3_execution_comparison` 的 `jre_region_uncovered_blocks` 期望不变）——成员文本与分类不变，移动的只是诊断面的记录计数，即已验收裁决下计费基线的相应漂移。

**不可反向修复**：恢复旧的无条件 split 拒绝会回退已验收正例——实测 `FinallyOnce.main` 第三链由 `error.getMessage() + ":" + count()` 退化为引用的 `StringBuilder` 链，违反任务不变量（"不得回退已验收正例……FinallyOnce 全类……N2b `jre_concat_split`@32"）；且记录计数是已验收裁决的直接函数，不存在保裁决恢复计数的机制。按本文件既定流程（"regenerate a pin after an intended shape change"）以 `record_the_billing_table` 与两臂自身 reader 重测，仅动三个常量的 `ir_items`（−54），注释按先例记录成因与测量方式（独立提交，便于整体放弃时回退）。

## 三、门禁（全部真实执行）

- `cargo test --workspace --tests --locked --no-fail-fast`：**全绿**（两段执行：修复后仅余 p5 两例；重测后全绿。`bulk_recovery_delivery` 未触发 flake）。
- 正例逐项回归：`p3_preceded_catches` 11/11（M5 家族、`a_getstatic_statement_before_a_named_catch_presents_the_catch`、`field_write_statements_before_a_named_catch_stay_presented`、`the_whole_m3_main_recovers`、`the_original_finally_once_main_recovers_with_its_three_chains`、`a_store_prefix_still_degrades_to_the_resource_refusal`、`m1_and_m2_presentations_are_unchanged_verbatim`、split 归属两例、TWR 多资源证书、预算/取消原子性）；`p3_guard` 13/13（含 `a_handler_range_that_swallows_the_initialisation_is_refused`、`a_single_resource_is_declared_in_the_header_and_closed_by_the_compiler`、`three_resources_are_declared_in_the_order_whose_closes_run_backwards`、`withCatch` 拒绝）；guard.rs verdict 级 `a_non_store_statement_prefix_is_not_examined_as_a_resource_header`（M5.main/P2.run `NotGuarded`）与 `a_two_resource_header_keeps_its_certificate`；`p3_catch_after_field_assignment`、`p3_content`、`p3_postfix_handler_boundary`、`task_operations` 全绿；N1/N2b 由 p3_preceded_catches 的 `a_cross_block_builder_keeps_its_split_refusal_at_its_own_to_string` 与执行对照面覆盖；Test2/dt14/CustomInit 位于全绿 workspace 内。
- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings -A clippy::too_many_arguments -A clippy::cloned_ref_to_slice_refs -A clippy::collapsible_if -A clippy::type_complexity -A clippy::len_zero -A clippy::needless_option_as_deref`：错误站点集与基线**逐条相同**（22 站点，仅 guard.rs 行号因新增函数平移 5154→5214、6819→6879；任务所述 23 站点在本地 1.98 下实测为 22，站点集与基线零差异），零新增。
- `openspec validate --all --strict`：218 passed, 0 failed。

## 四、遗留

- 任务原清单未列的 `the_presentation_does_not_re_read_the_text`（task_operations）随同 fixture 修复，无独立处理。
- 语料账本重测提交（tests/p5_bulk_corpus.rs）独立成 commit；若上游判定账本冻结优先于已验收 concat 归属，整体 revert 该 commit 即回到"门槛已修、账本两例红"的状态，门槛修复本身不受影响。

## 五、CF-15 交叉切片对"store 前置"负边界的翻转（2026-09-30）

本文件第一节冻结的负边界中，**store 前置降级**一项（`a_store_prefix_still_degrades_to_the_resource_refusal`，N1/P3StorePrefix）已被后继切片 [`recover-named-row-crossing-locals`](../../recover-named-row-crossing-locals/proposal.md) **翻转为正例**：该测试现为 `a_store_prefix_before_a_named_catch_presents_the_catch`，两个形状完整恢复为普通 `try`/`catch`。结构论证（C4.twrNamed 与 P5TwoResources 字节码实证）：

1. 具名 catch 包装整个语句时，编译器让该行的保护范围**覆盖资源初始化**——`twrNamed` 的 `IllegalStateException` 行跨 `[0, 36)`，起点在 `new C4()` 之前，故"紧跟完成 store 的行起点"不可能是包装行的起点；
2. lowering 自身的行在两侧都携带资源：行尾是该槽的 normal close（`aload; invokeclose` 或 `ifnull` 分支型），handler 是 close/suppress/rethrow 序列——两处皆无 close 的具名行不是任何 lowering 层级；
3. N1/P3StorePrefix 的行恰是"具名 + 行尾无 close + handler 无 close + 起点为完成 store 的语句边界"，结构上只能是用户 `catch`。

本文件第一节的其余负边界**逐项保持**：catch-all 行降级（CF-15 的 X1Catchall，`jre_guard_handler` 一字不动）、行劈开语句/吞初始化/交叠呈现（`statement_ends`/`statement_boundary` 判据原样保留，CF-15 门槛判别是并列的第五种回答，不触及它们）。旧测试期望的保留理由（当时"never spelled as a user catch"的口径）由 CF-15 的 change 文档与[复放记录](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/results/crossing/crossing-replay.md) §3.1 承接。

