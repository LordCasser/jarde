# P5 验证记录

本文件记录 `p5-measured-optimization` 各片的**实际验证**：命令、数字、证伪与如实边界。与 P2/P3/P4 的记录同口径——**只有实际跑过并复核过的才写在这里**，未做的写在「未完成」里。

**P5 的进入条件**（`design.md`）：「以**被选目标阶段**的结果契约和真实基线稳定为进入条件」——P2 29/29、分层 7/7、P3 12/12、P4 10/10 均已归档，**条件满足**。

## 2026-09-20 1.1：语料 fingerprint 与 A01–A18 覆盖核对（提交 `6d08124`）

### 语料 fingerprint 的形状与落点

**落点判断**：fingerprint 属**实现/测试的语料清单** → 写进仓库（`tests/fixtures/corpus-fingerprint.json` + `tests/p5_corpus_fingerprint.rs` + `tests/fixtures/README.md` 新增一节）；**A01–A18 的判定属验收权威陈述** → **未写进 `openspec/**`**，完整清单见本文档下表。

**形状**：数据文件 `tests/fixtures/corpus-fingerprint.json`（schema `jarde-corpus-fingerprint/1`），顶层键 `schema / purpose / regenerate / scope / dimensions / acceptance_rows / known_gaps / files`。
**`files`：93 条**（`tests/fixtures` 77 + `fuzz/corpus` 16），每条 `path / bytes / blake3`；与 `git ls-files tests/fixtures fuzz/corpus` 的范围内集合**逐条相等（93 = 93，双向无差）**。
**范围规则**（可审查，写在 `scope` 里）：根 `tests/fixtures`、`fuzz/corpus`；排除目录名 `target/out/artifacts/__pycache__`（`out` 是编译残留位）；排除扩展名 `md`（**README 是关于语料的记录，不是输入**——否则改个错别字就动 fingerprint）与 `py`；排除清单自身。
**按范围收录而非按名单**：新 fixture 落地即被覆盖，不必等谁想起来加。

### 先探明：既有语料锚定到了什么程度（**避免重造**）

| 既有载体 | 今天记录了什么 | 是否已构成 fingerprint 的一部分 |
| --- | --- | --- |
| `tests/fixtures/*/README.md` 的 SHA-256 表 | 每个**编译产物**的编译器版本、精确命令、目标 dialect、字节数、SHA-256 | **部分是，但没有任何测试校验它**：全仓库 `Cargo.toml`/`Cargo.lock` **没有 sha2/sha256 实现**（grep 零命中），Rust 侧算不出这些摘要，**表是散文**；`.java` 源、README 自身、golden JSON 均被漏掉 |
| `p1-golden`(5) / `p2-golden`(27) / `p4-golden`(13) 的 blake3 回放 | 内存构造的 fixture 字节摘要，由 `*_golden.rs` 重建后断言 | **是**（45 条摘要）——本片**只索引、不重复**（`Carrier::Generator` 指向 test 文件 + builder 名，`Carrier::Golden` 指向记录摘要的文档，摘要断言留在原处） |
| `fuzz/corpus/**` | 提交的对抗/退化种子；既有用例**只有 10 条尺寸**且需 nightly | 原来是「尺寸 + 事实」而非摘要；本片为 **16 条**加 blake3 |
| `p3_execution_comparison.rs`（`#[ignore]`） | 编译执行对照 | 是消费方，不是锚 |

**结论**：此前**没有**统一的、可一条命令校验的语料固定机制。本片补的正是这个缺口。

### 八个维度（任务原文：版本/编译器/打包/身份/字节码/恢复/退化/对抗）

| 维度 | 今天由什么承担 | 本片补了什么 | 状态（carriers） |
| --- | --- | --- | --- |
| 版本 | `historical/ecj-4.6.1/v45–v52`（45.3–52.0，8 个 ECJ 产物）+ `p4-modern/{v8,v16,v17}`（52/60/61 javac）+ `p4-golden/version-boundaries.json`（72 / 非法 minor）+ `p4_feature_registry.rs::with_version` | 8 个 ECJ 与 3 个 javac 产物的 blake3；版本改写字节锚定到既有 golden | pinned（13） |
| 编译器 | ECJ 4.6.1 `-source 1.3`（历史）与 javac 23.0.1；**编译器本身不入库**（生成期输入） | **源文件**的 blake3 + 三个 `record` 载体（README 存在性；其版本/命令叙述**不入摘要**） | pinned（10） |
| 打包 | 提交的 `fuzz/corpus/artifact_tree/*`（minimal-jar/nested.jar/multi-release.jar/damaged-entry.jar/truncated-jar）与 `query/*`；`p1-golden/{nested,multi-release}.json`；WAR/Boot/ZIP64 **只在 builder 里** | 种子归档与两个 golden 的 blake3 | **partial（11）**：WAR/Boot/ZIP64 无摘要 |
| 身份 | `p1-golden/nested.json`（同一 jar 字节在两个 `WEB-INF/lib` 条目，一 STORED 一 DEFLATED）+ 两个 builder 的同字节多序号/多 origin 形状 | golden 与嵌套种子的 blake3 | **partial（4）**：builder 的重复形状无摘要 |
| 字节码 | ECJ v52、B2 lambda、签名文法四类、R2 两类、P4 concat、`JvmBytecodeOracle.java`、`p1-golden/code.json`、`p1_xref_code.rs::fixture` | 全部样本 + oracle 源 + 两个 builder 的索引 | pinned（10） |
| 恢复 | `p3-local-rewrite`、`p3-scope`（含 `v8-debug`）、`p3-handlers`、`p3-corpus`（5 个 flag 矩阵 + 缺失依赖）、`historical/v52`、`p2-golden/{legacy-clone,historical}.json`、`fuzz/corpus/method_analysis/*` | 17 条（**本片最大一维**） | pinned（17） |
| 退化 | 提交的截断/损坏种子、`p2-golden/resource-boundary.json`、缺失依赖样本（stub 以**源码**入库、故意的 class 不入库） | 种子 + golden + builder 索引 | pinned（10） |
| 对抗 | `p4-golden/illegal-modern.json`、`p2-golden/wide-switch.json`、fuzz 种子的 wide/switch/重叠 handler、两个 proptest 套件、fuzz targets | 种子与 golden 的 blake3 | **partial（9）**：proptest 用例无摘要 |

### 缺什么（`known_gaps` 五条，**如实，未凑**）

① WAR/Boot/ZIP64 树只存在于 builder；② STORED/DEFLATED 与多 origin 只在既有 golden 命名的形状上被固定；③ 两个 proptest 套件的用例**逐次生成**（CI 固定 `PROPTEST_RNG_SEED` 使之**可复现**，不是**已固定**）；④ JDK 25 oracle 的**输出**来自运行它的 JDK，**仓库不固定 JDK**；⑤ `p3_execution_comparison` 的 wrapper 由机器上的 `javac` 编译，**编译器未固定**。
**这五条都不是靠改一个文件能补上的**（要么给 builder 加 golden，要么属既有边界），故**如实列出而不造语料**。

### 校验与证伪（**父级独立复核**）

父级复跑：`cargo test --test p5_corpus_fingerprint --locked` = **5 passed / 0 failed / 1 ignored**（0.03 s，无 JDK、无网络、不写文件）；JSON 结构核对 = **93 files / 8 dimensions / 18 acceptance_rows / 5 known_gaps**。
**父级独立证伪**：翻转 `p4-modern/v16/Marker.class` 的**一个字节** → 校验 FAILED 并打印两个摘要（`recorded blake3 809575fc… / 340 bytes, now blake3 11711f88… / 340 bytes`）；还原后 `sha256sum -c` OK、校验复绿。
**实现者另证伪一组**：新增一个清单不知道的文件 → `unlisted: … is in the corpus but not in the manifest` FAILED；删除后复绿。
失败消息同时给出「若是有意改动 → 跑哪条命令再生成；若不是 → 修语料，**不要重新记录来让它变绿**」。

### A01–A18 逐行覆盖清单（**本片核心产出**）

判定只依据**实跑的测试名 + 既有 verification 锚点**；**未跑的检查不写成已通过**；P5 自身的两行**不声称已通过**。

| ID | 当前证据（用例 + 关键断言） | 判定 | 缺哪半 / 阶段 |
| --- | --- | --- | --- |
| **A01** | `p1_xref_code.rs::unused_constant_pool_entries_are_candidates_but_never_calls`（同一 fixture：CP 命中是 candidate、X1 consumer 为 0）；`p1_xref_metadata.rs::metadata_only_types_hit_only_their_requested_category`；golden `code_golden_replays_the_unused_pool_contrast_and_the_call_coordinates` | **现已通过** | — |
| **A02** | `p1_xref_code.rs::invocation_item_keeps_the_complete_symbol_and_its_byte_coordinates`（完整 descriptor + BCI + opcode + CP index + span）；`historical_classfile_corpus.rs::historical_ecj_headers_and_finally_bytecode_are_stable`；oracle 交叉 `jvm_bytecode_oracle.rs::jdk25_instruction_boundaries_match_public_bytecode_inspection` | **现已通过**；oracle 一条**本机未运行**（环境缺口，**not verified**） | 缺：本机 JDK 25 oracle 重跑（环境，非代码） |
| **A03** | `p1_xref_metadata.rs::a_compiled_record_component_annotation_is_found_at_its_own_position`、`a_compiled_code_type_annotation_is_found_without_decoding_the_body`、`a_compiled_generic_return_signature_is_read`、`a_compiled_throws_signature_reports_only_its_class_type`、`generic_signatures_follow_their_own_grammar`；样本 `r2-annotation-positions/{v17,v8}` | **现已通过** | — |
| **A04** | `p1_xref_bootstrap.rs::a_real_metafactory_site_is_linkable_and_traces_the_implementation_handle`、`an_implementation_handle_is_never_reported_as_a_call`、`a_custom_bootstrap_claims_no_final_target`；golden `bootstrap_golden_replays_the_compiled_lambda`；P3 供 lambda 恢复 | **现已通过**（P1 结构 + P3 恢复两半） | **「现代恢复」增量未发生**（与 P4 3.4 一致）：`jarde-java` 对 `ModernFacts`/`modern_facts`/`ModernOrigin`/`OutputLevelStatus` **零引用**；触发条件＝恢复层开始消费 `ModernFacts` |
| **A05** | `p1_xref_bootstrap.rs::nested_condy_reports_both_levels_and_one_fact_per_shared_subgraph`、`every_use_site_keeps_its_own_path_to_a_shared_deeper_node`、`a_cycle_is_reported_instead_of_followed`、`a_condy_chain_past_the_derived_bound_stops_with_a_diagnostic`；P4 `p4_modern_facts.rs::a_shared_subgraph_is_expanded_once…`、`a_cycle_is_recorded_as_facts_instead_of_being_followed`、`the_{edge,depth,node_and_step}_budget…` | **现已通过**（P1 + P4 逐条对上） | — |
| **A06** | `p1_multi_release.rs::a06_selects_root_11_and_17_and_marks_the_other_variants`、`a06_without_v17_falls_back_to_v11_and_without_manifest_to_base`、`manifest_evidence_states_and_path_diagnostics`；golden `multi_release_golden_replays_three_views_and_one_physical_query`；P4 `p4_runtime_matrix.rs::three_profiles_select_different_entries_and_keep_every_physical_entry`、`a_manifest_condition_that_hides_versioned_entries_is_diagnosed` | **现已通过** | — |
| **A07** | `p1_artifact_tree.rs::duplicate_nested_entries_derive_distinct_child_and_inner_identities`、`boot_and_war_library_names_require_exact_prefix_case_direct_path_and_lowercase_jar`、`war_and_ordinary_nested_layout_are_physical_evidence_only`；P2 `p2_resolution.rs::a_byte_equal_duplicate_is_ambiguous_not_ordinal`、`the_same_bytes_at_two_origins_are_not_merged`；P4 深度 `p4_runtime_matrix.rs::a_name_in_two_war_roots_reports_both_origins_and_the_order_the_declaration_gives` | **现已通过** | P4 有意边界仍在：具名模块顺序、兄弟 loader 并存、`WEB-INF/classes/` 下 MR 目录未做 |
| **A08** | `p1_artifact_tree.rs::boot_tree_is_explicit_and_nested_entry_is_replayable_without_intermediate_output`、`nested_replay_preserves_depth_entry_count_and_cancellation_errors_without_result_items`；`p1_xref_code.rs::zip_scan_reads_the_class_entry_and_never_other_entries`、`a_truncated_class_candidate_fails_with_its_entry_origin`；golden `nested_golden_replays_stored_and_deflated_origins` | **现已通过** | — |
| **A09** | P1：`historical_classfile_corpus.rs::historical_ecj_headers_and_finally_bytecode_are_stable`（v45–v52）；P2：`p2_return_address.rs::the_historical_jsr_finally_completes_the_call_context_pass`、`a_return_address_the_bytes_do_not_prove_stops_the_stage_before_it_completes`、`a_live_call_site_behind_a_mid_block_jsr_is_still_refused`；`p2_canonical.rs::the_historical_finally_normalizes_with_one_clone_per_call_site`、`an_exact_clone_budget_completes_and_one_less_falls_back`；golden `p2_golden.rs::the_historical_corpus_replays_from_45_to_52`、`the_legacy_clone_entry_bills_two_call_sites`；P3：`p3_method_ir.rs`/`p3_isolation.rs` 读 **v45** 走恢复路径、`p3_guard.rs::a_finally_copy_is_refused_and_the_refusal_says_which_code_it_copies` | **现已通过**（P2 范围，P3 恢复另验收） | **观察（不抬高为缺口）**：P3 的编译执行对照读 v52 与 `p3-*` 样本、**不含 45–48**；45–48 的 P3 侧证据是 IR/隔离路径 |
| **A10** | `p2_frame.rs::the_frame_phase_completes_a_body_without_any_debug_table`、`an_exhausted_item_budget_publishes_no_frame_table`、`an_exhausted_item_budget_inside_the_names_phase_publishes_no_names`；`p2_ssa.rs::the_ssa_phase_names_a_real_body_and_completes_the_pipeline`；`p2_golden.rs::the_input_side_premises_of_the_entries_hold`（missing-debug）；P3 命名 `p3_scope.rs::without_debug_evidence_a_reused_slot_stays_one_variable`、`the_debug_table_is_read_by_the_same_run_that_decoded_the_body`；flag 矩阵 `p3_execution_comparison.rs::the_corpus_is_read_the_same_way_by_every_legal_flag_set` | **现已通过**（**且不宣称 verifier 通过**：`the_matrix_never_claims_verification`、`requested_verification_and_debug_never_claim_a_complete_schema`） | — |
| **A11** | `p2_declaration_refs.rs::a_sub_call_site_resolves_to_the_base_declaration`、`a_candidate_that_resolves_to_another_declaration_is_not_an_item`、`an_unconsumed_pool_entry_is_not_a_candidate`；`p2_members.rs`（63 条）；`p2_resolution.rs::a_damaged_candidate_fails_at_its_origin_without_falling_back`、`a_listing_that_stops_early_never_reads_as_missing`；P4 深度 `p4_x2_states.rs::a_symbolic_owner_resolves_to_the_declaration_and_a_range_extends_it_to_its_candidates`、`a_missing_dependency_is_stated_by_name_and_never_as_the_negative_answer` | **现已通过**（P2 + P4 深度扩展） | P4 边界：`DeclarationRefReport` 无结构化 `unresolved_dependencies` 平面；歧义/环与「名字不存在」共用 `UnresolvedDependency` |
| **A12** | `p3_accessor_edges.rs::the_two_original_edges_survive_a_recovery_run_field_by_field`（同一次运行呈现直接字段表达式，且 `Engine::query` 两条原始边逐字段不变）、`the_anchors_name_the_member_each_field_access_is_in_and_only_named_bodies_are_read`、`a_call_to_another_classs_same_named_member_is_not_read_from_this_class` | **现已通过**（P3 公开入口验收） | **P4 无增量（如实）**：concat 只落事实平面（`a_real_concat_site_is_a_reader_fact_and_joins_to_the_bci_of_its_instruction`），**不产生文本**；触发条件写在 `docs/support-matrix.md` |
| **A13** | `p1_xref_code.rs::an_undecodable_member_is_reported_without_hiding_the_other_members`；`p2_cfg.rs::a_member_that_declares_no_body_is_a_fact_and_not_a_failed_pass`、`a_class_whose_header_cannot_be_read_fabricates_no_method_result`、`two_members_of_one_class_are_answered_about_themselves`；`p2_members.rs::a_missing_member_keeps_the_reference_and_fabricates_nothing`；四平面分离 `p2_contracts.rs::result_planes_are_reported_side_by_side_and_never_inferred`；golden `p2_golden.rs::the_stopped_inputs_stay_published_and_explained` | **现已通过** | — |
| **A14** | `p1_query_bounds.rs::the_within_bound_rejects_every_dimension_over_its_limit`、`result_budget_publishes_the_reliable_prefix_of_a_high_fanout_entry`、`cancellation_at_the_high_fanout_unit_boundary_publishes_nothing`；`p1_artifact_tree.rs::budget_and_precancellation_return_non_complete_reliable_prefixes`；`p1_multi_release.rs::budget_exhaustion_is_partial_and_never_fakes_completion`、`cancellation_stops_before_any_faked_selection`；golden `p2_golden.rs::the_resource_boundary_entries_stop_where_their_budget_says`；**P4 3.3 对抗重跑无回归**（bomb 4 / condy 7 / 缺失依赖 4 / 不可约 CFG 1 / 取消压力 40 / 非法现代 6） | **现已通过**（P2 范围 + P4 回归） | P5 侧增量：3.1 的 optimized/direct coverage 与资源差分（未做） |
| **A15** | 今天只有**输入侧**：`p1_xref_golden.rs`(5 replay)、`p2_golden.rs`(9 条)、`p4_golden.rs::every_entry_replays_its_fixture_digest_and_planes` —— 固定 snapshot/query/view 并比较**完整序列化报告**（唯一归一化是删 `elapsed_millis`） | **未通过（P5 自身，未开始）** | 缺**运行侧**：冷/热/关缓存三条路径的完整结果一致 + 完整运行语义 fingerprint。仓库今天**不存在任何 cache/index/并行实现**（`cache` 仅出现在 `crates/jarde-jvm/src/{ssa.rs,method_ir.rs}` 的**文档注释**，且都是「不缓存」的说明）→ 属 **P5 1.3 + 2.x + 3.1** |
| **A16** | P2：`p2_closure.rs::a_request_for_a_class_that_has_a_body_reads_no_code_byte`、`a_read_is_recorded_for_the_definition_whose_bytes_were_read`、`a_budget_stop_keeps_every_record_inside_the_charged_attempts`；`p2_entry_counts.rs::one_method_analysis_attempts_one_body_however_many_the_class_declares`；P3：`p3_recovery_entry.rs::one_recovery_request_reads_one_body_and_presents_that_member`、`p3_accessor_edges.rs::the_anchors_name_the_member_each_field_access_is_in_and_only_named_bodies_are_read`、`p3_isolation.rs::a_physical_query_constructs_none_of_the_recovery_layer_and_the_recovery_run_constructs_it` | **现已通过**（P2 + P3 重做） | P5 测量侧：局部 vs 全范围的物化类/方法/字节范围报告（1.3）未做 |
| **A17** | `p2_entry_counts.rs::the_x1_consumer_scan_constructs_no_ir`、`the_x0_pool_probe_constructs_no_ir_and_decodes_no_body`；运行期隔离 `p3_isolation.rs`（query 的 IR/边/步/克隆**全 0**、header/body 0；recover 同批非 0、header 1/body 1）；源码守卫 `p2_contracts.rs::physical_entry_modules_do_not_reference_the_recovery_layer`、`the_a17_guard_detects_rewritten_references_and_added_files`；分层实跑 **0/0/0** | **现已通过**（构造计数代理 + 运行期隔离 + 源码守卫 + cargo 分层） | ① P4 记录的守卫**部分盲区仍在**：受守卫集是 X1 物理入口面（`query.rs` + `xref/**` + 已加入的 `plugin.rs`），其余 4 个模块（`jarde-reader/src/{release_registry,modern,runtime_matrix}.rs`、`jarde-jvm/src/reflection.rs`）在守卫之外，但**其依赖边被 cargo 在解析期拒绝**——与 acceptance.md「守卫是补充而非替代」一致。② A17 仍是**预算代理**而非逐 pass 构造计数器（P2 记录） |
| **A18** | `p1_query_api.rs::a18_open_snapshot_stays_stable_and_a_reopened_one_rejects_old_cursors`、`cursor_mismatches_bind_snapshot_view_relation_and_schema`、`cursor_for_another_target_is_rejected_before_the_scan`、`cancellation_is_never_reported_as_complete`；`p1_query_bounds.rs::cancellation_at_the_high_fanout_unit_boundary_publishes_nothing`；P1 3.3 的 fuzz 门禁 | **部分通过**：P0/P1 半（固定字节源、游标与快照绑定、变化即中止）已通过 | 缺 **P5 那一半**：跨快照 token/缓存隔离的回归——**无 cache/并行实现故无法对照** → 属 **P5 2.x（cache key 含 snapshot/view/platform/registry）+ 3.1/3.2** |

### 汇总

- **现已通过**：A01、A02（除本机未跑的 oracle）、A03、A04、A05、A06、A07、A08、A09、A10、A11、A12、A13、A14、A16、A17——共 **16 行**；
- **未通过**：**A15**（P5 自身，未开始）；
- **部分通过**：**A18**（P1 半已通过，P5 的缓存/并行半未做）；
- **两处如实标注的边界**（不是缺口，是**不虚报**）：A04/A12 的「**现代恢复**」增量**未发生**（与 P4 3.4 一致，触发条件已写明）。

### 与 1.2 的接口

**引用对象是数据文件，不是 Rust 常量**：`tests/fixtures/corpus-fingerprint.json`（schema `jarde-corpus-fingerprint/1`）。1.2 用 `serde_json` 直接读：
- **按维度选语料**：`dimensions["packaging"].carriers[]` / `dimensions["recovery"].carriers[]`（`kind` ∈ `file|golden|generator|builder|record`）；
- **按验收行选语料**：`acceptance_rows["A15"].corpus[]`；
- **benchmark 报告应记录它读到的摘要**（`measured-execution` spec 要求「记录输入 fixture fingerprint」）：建议记 `schema` + 每个 fixture 的 `files[].blake3`，或直接记 `blake3(corpus-fingerprint.json)` 作为「语料版本」单一标识（**文件自身不能含自己的摘要**，故由消费者算）。
**可挂的 gate**：`cargo test --test p5_corpus_fingerprint --locked`（0.03 s、无 JDK、无网络、不写文件）；有意改语料时才跑 `-- --ignored regenerate_corpus_fingerprint`。

### 证据

全量 **1082 passed / 0 failed / 4 ignored**（基线 1077/0/3 → **+5 passed、+1 ignored**；新增的正是本片 5 条校验 + 1 条 `#[ignore]` 再生成器；**无既有用例改红或改绿**）；`p5_corpus_fingerprint` 5 passed / 1 ignored；fmt 与 clippy 1.98.1 干净；**MSRV `+1.88.0` check 通过**；`openspec validate --all --strict` **16 passed**（P4 归档后数量）；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；锁文件两条 exit 0。**被修正的既有断言：0 条**（测试侧唯一改动是 `tests/fixtures/README.md` **新增**一节）。

### 未完成（如实）

**属后续片**：1.2 的冷/热与三路 benchmark（本片只固定输入）；1.3 的结果 fingerprint/稳定排序/coverage 对照 → **A15**；2.x 的 cache key 与失效/回退 → **A18 的缓存半**；3.1–3.4 的差分与门槛。
**本片如实列出的 `known_gaps` 五条**（见上）**未闭合**。
**前序遗留（本片未处理）**：CLI 出口缺失；X3 若干分支与 plugin `Skipped{Error}`/`Cancelled` 无 fixture；`MethodParameters` 未读；类级事实不在载荷；A17 源码守卫的 4 个模块盲区（依赖边由 cargo 兜底）。

## 2026-09-20 1.2 + 1.3：direct 基线、结果 fingerprint 与 A15/A16（提交 `e7f509f`）

### 落点与 cache/parallel 的处理选择（**判断**）

**先探明**：仓库**没有任何** benchmark 设施——`benches/` 目录 0 个、`criterion`/`[[bench]]`/`harness = false` 在所有 `Cargo.toml` **0 命中**、无手工计时工具。故本片**新建**。

**落点** `tests/p5_benchmark.rs`（测试内 harness，7 条常开 + 1 条 `#[ignore]` 重复测量）。理由：① 常开的确定性/排序/coverage 对照必须与测量走**同一套请求构造与同一套 fingerprint 函数**，拆到 `benches/` 会出现两个权威；② `harness = false` 的 bench 目标会被 `cargo test --all-targets` 一并执行，把 200 次重复塞进既有门禁；③ 本仓库对昂贵/选项式工作的既有约定就是 `#[ignore]` 集成测试（`p3_execution_comparison`、1.1 的再生成器）。**未触及 `Cargo.toml`**。

**cache/parallel 选「甲」**：**只建 `direct` harness，cache/parallel 如实记为 `absent`**，**不**跑「关缓存/开缓存」两条同代码路径。
**理由**：`design.md` 决策 5 要的是 `optimized vs direct` 的**两条真实路径**；给同一段代码贴两个标签，会把「结果相同」读成「缓存已验证」——**正是 design.md Risk 与 A15 备注警告的那种「测量形状的主张」**。
**预留位是代码而非输出**：`compare(&RunRecord, &RunRecord)`/`Verdicts` 对**任意两次运行**通用，第二条路径出现当天即由同一函数测量与对照；context 里 `cache`/`concurrency` 写明「absent，P5 2.2 / 2.x 拥有」。

### 指标五项与内存代理的局限

| 指标 | 来源 |
| --- | --- |
| 耗时 | harness `Instant`（µs）**+** 报告自身的 `usage.elapsed_millis` |
| 读取字节 | `input_bytes`/`entry_bytes`/`read_bytes`/`output_bytes` |
| 物化范围 | `archive_entries`/`class_headers`/`method_bodies`/`analysis_steps`/`normalization_clones` **+** 发布的 `coverage`（各维 `state`/`scanned`/`skipped`） |
| 内存代理 | `class_bytes`/`attribute_bytes`/`code_bytes`（已物化的 class-file 内容）+ `ir_items`/`ir_edges`（逐单位分配的派生存储）+ `result_items` |
| 取消状态 | `ExecutionReport` 的终止状态 + `diagnostics`（既有语义，未新增表达） |

**代理的局限（写进代码文档）**：① **不是 RSS**；② 是 `UsageSnapshot` 的**累计计费计数**，无 allocator 开销/碎片，除 `nested_depth`/`dependency_depth` 外**无高水位**，无法区分峰值与总量，且计数**包含已释放的临时对象**；③ 一次运行完全可以在不 charge 这些维度的前提下分配内存。故这些维度上的 delta 只能支持「**这条路径派生/持有的工作单位更多**」，**不能**支持「这条路径用了更多内存」。**未**引入 allocator 计数器或新依赖。

**耗时为何用 harness 时钟**：报告时钟是**整毫秒**，本语料下每次重复**都是 0**，承载不了分布；`elapsed_millis` 仍是 fingerprint 归一化的那一个字段。

### 实测基线（`REPEATS = 200`／行；同一台机器、单线程构建、`RUST_TEST_THREADS=1`）

**耗时**（µs；`first` = 进程内首次）：两次完整测量（相隔数分钟、同一代码）

| 行 | run A: min / median / p90 / max / first | run B: min / median / p90 / max / first |
| --- | --- | --- |
| `full-range-xref/minimal-jar` | 122 / **131** / 154 / 274 / 274 | 127 / **141** / 157 / 472 / 472 |
| `full-range-xref/v52-class` | 129 / **139** / 157 / 200 / 141 | 132 / **147** / 160 / 244 / 160 |
| `single-member/v52-class` | 120 / **134** / 171 / 537 / 537 | 123 / **141** / 188 / 1825 / 1825 |
| `cancelled/…minimal-jar` | 19 / **21** / 22 / 54 / 54 | 19 / **22** / 23 / 71 / 57 |

**固定量**（两次测量**逐字相同**，已 diff 证明）：

| 行 | 读取字节 | 物化范围 | 内存代理 | 状态 |
| --- | --- | --- | --- | --- |
| `full-range-xref/minimal-jar` | input=659 entry=627 read=627 out=0 | entries=13 headers=**0** bodies=**0** steps=0 | class=555 attr=174 code=22 ir_items=**0** ir_edges=**0** items=4 | complete |
| `full-range-xref/v52-class` | input=303 read=**909** out=909 | headers=0 bodies=0 | class=**909** attr=588 code=48 items=1 | complete |
| `single-member/v52-class` | input=303 read=**303** out=418 | headers=**1** bodies=**1** steps=29 | class=303 attr=120 code=4 ir_items=86 ir_edges=5 | complete |
| `cancelled/…minimal-jar` | input=659，其余 0 | 全 0 | 全 0 | **cancelled** |

**可比性声明（必须随数字一起读）**：均为**同一台机器、同一进程池、单线程构建**的重复测量；语料是提交的小样本（659 B 归档 / 303 B class），数字刻画的是 direct 路径在该规模下的**形状**，**不可外推**；**不给任何加速倍数、P95、吞吐或目标**；两次测量中位数漂移 **+5%~+8%**、前后半差最多 ~12%、**进程内首次运行是 1.3×~13× 中位数**——即**低于约 10% 的差异在本机这套 harness 上不可区分**，**跨环境比较一律不做**。

### 结果 fingerprint 与稳定排序

**形状**：对**完整运行**的序列化文档（query 行 = 整个 `QueryReport`；局部行 = 整个 `RecoveredMethod`）**递归删除 `elapsed_millis`**（**唯一**归一化，与 `p2_golden`/`p1_xref_golden` 同口径），按 `serde_json` 渲染后取 **blake3**。`Published` 同时保留原始文档，使「归一化到底做了什么」**可被质问**。

**确定性证据（父级独立复现）**：① 8 次进程内重复 × 3 行：fingerprint/计费/状态/顺序**全等**；② **三个独立进程**的 marker **逐字相同**（`full-range-xref/minimal-jar blake3:7603cd8c…`、`full-range-xref/v52-class blake3:1a255003…`、`single-member/v52-class blake3:27354fa2…`）——**这一条才把「确定」与「只是本进程的 HashMap 顺序稳定」区分开**（seed 按进程变化）；③ 两次完整测量之间所有计费/coverage/诊断/fingerprint 行**逐字相等**（仅时间戳变）。
**稳定排序基线**：同进程 8 次 + 跨进程的 item identity 序列（source+consumer+operation+target+derivation）逐字相同；对照报告把顺序要求定为**相等**而非「同一集合的另一种稳定序」，因为决策 4 说并行只改调度、不改发布顺序。**今天没有并行，所以这是「顺序基线现在是确定的」的证据，不是并行正确性的证明。**

### 父级独立证伪（**修正了一次无效变异**）

- **第一次尝试无效**：在 `fingerprint_of` 里对文档**无条件插入** `elapsed_millis` → **7 条全绿**。原因是该变异对**比较双方施加同一变换**，故对断言是 no-op——**是我的变异写错，不是断言弱**。**如实记录，不计入证据。**
- **正确变异**：跳过 `strip_elapsed`（归零 `removed`）→ **2 红**，与实现者自报一致：`the_result_fingerprint_ignores_the_wall_clock_and_nothing_else` 与 `the_direct_row_fingerprints_are_printed_for_a_cross_process_comparison`，后者给出的正是**同进程两次同输入运行的不同摘要**（`8cb7f902…` vs `bc6adb02…`，局部行的墙钟偶尔跨 1 ms）。
- **值得记录的一点**：`repeated_direct_runs_publish_the_same_result_fingerprint` 在两次变异下**都保持绿**——本语料下报告时钟几乎恒为 0，**「只重复比对」不足以照出这个缺陷**，这正是 stamp 测试存在的理由。
- **实现者第二组**：局部行记录 `class_headers += 1` → **恰好 1 红**（`the_local_and_the_full_range_runs_are_compared_field_by_field`，`left: 2 / right: 1`），其余 6 绿。副本校验通过。

### A15 / A16 的如实判定

**A15：仍未通过。** 今天能验的是**重复那一半**：同一 direct 请求 8 次重复 + 3 个独立进程，完整结果（含 evidence/coverage/representation/diagnostics/usage）在去掉墙钟后一致 → **verified**。
**不能验的是缓存一致性**：仓库**不存在任何 cache/index**（`cache` 只出现在 `crates/jarde-jvm/src/{ssa.rs,method_ir.rs}` 的「不缓存」文档注释），**没有第二条路径**可以「关缓存/开缓存」，spec 的 `Cold and warm comparison` 第 2 个条件（「以**开启缓存**执行」）**无法满足**。故 **A15 判定维持 1.1 的「未通过」**，负责片为 **2.x + 3.1**。
顺带：`冷/热` 今天只能测「进程内首次 vs 重复」的**耗时**差，其**语义**差被断言为 **0**（首次运行的 fingerprint 与后续每次相同）。

**A16：不重做。** 本片只在 benchmark 层面**记录**物化范围：局部行 `class_headers=1`、`method_bodies=1`（该类声明 **3** 个有 body 的成员，前提由 harness 自己的 header 读取得出）、`read_bytes=303`、`coverage=complete_within_schema scanned=2`；同一 303 B 语料上的全范围行 `read_bytes=909`、`class_bytes=909`、`attribute_bytes=588`、`code_bytes=48`。这就是「**局部 vs 全范围**」在**相同语料**上的对照数字。

### 与 1.1 fingerprint 的实际接法

运行时读 `tests/fixtures/corpus-fingerprint.json` → 断言 `schema == "jarde-corpus-fingerprint/1"` → 用 `files[]` 逐条校验 subject 的 **blake3 + 字节数**（**跑的就是校验过的那些字节**）→ 用 `acceptance_rows["A15"].corpus[]`（`minimal-jar`）与 `dimensions["recovery"].carriers[]`（v52）断言「该 subject 仍被归类在那一行/那一维下」→ 每行 context 记录 **`blake3(corpus-fingerprint.json)` = `5b56783f4cbdaa6f4d5b…`** 作为**语料版本**（文件不能含自身摘要，由消费者算）。**改语料 → benchmark 红**，而不是静默换基线。

### 对 3.1 可比性有直接影响的**四项实测事实**（本片发现，建议 3.1 先读）

① 局部行 `Engine::recover_method` 的**发布报告少报请求计费**（`output_bytes +115`、`ir_items +2`、`analysis_steps +5` 在报告之外被 charge）——资源门槛若读 `analysis.execution.usage` 会**低估局部路径**；
② **X1 全范围行的 `class_headers`/`method_bodies` 恒 0**，却物化了 588–606 `class_bytes`（X1 走自己的读取计费，**不走 P2 的 header/body demand 路径**）——**「读了几次 header」在 X1 路径与 P2/recovery 路径之间不可直接对照**，资源门槛必须**两侧取同一条计费路径**（本片 `compare()` 统一取**请求总计费** `budget.usage()`）；
③ `result_items` 是**计费量而非页大小**（归档行 `result_items=4` 而实际发布 1 个 item）；
④ 本机中位数复现性 **~10%**，进程内首次运行可达中位数 **13×**。

### 证据

全量 **1089 passed / 0 failed / 5 ignored**（基线 1082/0/4 → **+7 passed、+1 ignored** = 新文件的 7 常开 + 1 `#[ignore]`；**无既有用例改红或改绿**）；`p5_benchmark` 7 passed / 1 ignored；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` **16 passed**；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；锁文件两条 exit 0 且 **`Cargo.toml`/`Cargo.lock` 无 diff（未新增依赖）**。**被修正的既有断言：0 条**。
**CI**：`e7f509f` → 见下。

### 未完成（如实）

**A15 的缓存半**属 2.2（cache/index）+ 3.1（optimized/direct 差分）——今天无第二条路径可测；**cache/parallel 的实测量与对照**属 2.x（本片只预留 `compare()` 机制与 context 字段）；**取消压力**（并发/解压/IR 内取消）属 3.2（本片只覆盖「入口前取消」的确定性变体）；**ZIP bomb/condy 图/不可约 CFG/缺失依赖语料上的优化回归**属 3.2。1.1 的 `known_gaps` 五条**未闭合**，本片未动。

## 2026-09-20 2.1 + 2.2：两项「基于测量」的决策（提交 `170ea20`）

### 2.1：多查询合并 / 细粒度并行 / single-flight —— **决策：保持 disabled**

**依据（全部来自 1.2/1.3 的实测，或由本片新增守卫机械核对）**：
- **收益证据不存在**：语料 659 B 归档 / 303 B class；direct 中位 **131–147 µs**；本机中位复现性 **~10%**（两次测量漂移 +5%~+8%、前后半最多 ~12%），**进程内首次可达 13× 中位**。低于该带宽的调度收益在本机**不可区分**。
- **没有第二路径可比**：仓库无 cache/index/并行实现——现由**源码守卫** `the_engine_has_no_cache_index_or_scheduler_to_extend` **机械核对**（而非一次性 grep）。
- **行级资源形状也不支持**：局部行 1 body（303 B、29 steps、86 IR items，类声明 3 个 body）；两条全范围行 **`class_headers=0`/`method_bodies=0`** 却物化 555 与 909 class bytes；worker 的**固定成本（spawn/交接/有序合并）在本 harness 里从未测过**。

**触发条件（写进记录，未编造阈值）**：(a) 某个入口或 benchmark 行能**对一个 snapshot 跑两个请求** → 用本 harness 量「重叠对 vs 顺序对」，**触发条件是「该行存在」而非某个收益数值**；(b) 若出现一行其资源报告里**每单位份额占首位的负载** → 候选差异须先越过**同机同 harness 的重复带宽**（今天 ~10%）；**「多大算值得默认开启」未定**（决策 5）。
**ceiling 与 upgrade path 同记录**：共享请求只改调度、不得重置预算、取消订阅者状态相互隔离；并行需 `compare()` 的顺序**相等**与 3.2 的压力语料。

**A14 的如实判定**：今天成立且被验的是「入口前取消 → `cancelled`、零 items、诊断非空」（既有 `a_cancelled_direct_run_is_never_published_as_complete`），加本片新补 **`a_cancelled_request_does_not_reach_a_later_one_over_the_same_bytes`**——取消是**每请求**的，取消后的下一个同字节请求**重新发布基线 fingerprint**（`7603cd8c…`）。**这是 single-flight 那条规则今天唯一能验的形状**：会「取消这批字节的工作」而不是「取消这个订阅者」的实现会先在这里红。
**single-flight：无对象可验**——没有任何共享请求，spec 的「一个订阅者取消而另一个继续等待」**没有主体**，**不假装验过**。
**A15 判定维持「未通过」**（缓存半）；A18 的缓存/并行半同理未动。

### 2.2：cache/index 范围与 key 维度 —— **决策：保持 disabled；key 以「记录」形状定义**

**理由三条（本仓库的事实而非偏好）**：① key **没有消费者**——没人读 key，形状只能靠猜；② **两维今天根本没有身份**：IR 与 recovery **没有任何 version 常量/字段**（全 grep 无 schema/version 标识），实现 key 意味着**先凭空造出它要 hash 的版本**，正是 Non-Goal 拒绝预设的东西；③ key 只能靠 **cached vs direct 对照**验证（A15 的缓存半），今天钉住的是猜测，且**会被后人误读为「已验证」**。

**key 维度记录**（`KEY_DIMENSIONS`，**10 条** = 决策 2 的 9 条 + `facts-cache` 决议层单列的 `dependency-snapshot`）：每条带 `name`/`layers`（`cp-header`/`x1`/`resolution`/`ir-source`）/`source`/`carrier`。`Carrier::Present` **锚定到源码 token**（`ArtifactSnapshot`、`PhysicalView`、`RuntimeProfile`、`HIGHEST_REGISTERED_MAJOR`、`QUERY_ENGINE_SCHEMA`、`providers`、`PASSES`、`Limits`），`Absent{why, closed_by}` 用于 IR 与 recovery。
**测试钉住**：决策 2 的九条**各恰好一次**、层名合法、**Present 的 token 必须在引擎源码中存在**、**Absent 必须写清 why 与谁关闭**。

**A01 的验证结论：已覆盖，不重做** —— `p1_xref_code.rs::unused_constant_pool_entries_are_candidates_but_never_calls`（X0 候选带 CP index、`consumer: None`、无 BCI/opcode；同一 target 的 `mentions_symbol` **零 items**、`scanned_items=0`），配套 `p1_xref_metadata.rs::metadata_only_types_hit_only_their_requested_category`、`p2_declaration_refs.rs::an_unconsumed_pool_entry_is_not_a_candidate`。
**cache 语境下补了一条**：`the_reference_path_publishes_no_pool_candidate_as_an_item`——参考路径两条 X1 行的 derivations **只有 `structural_consumer`**（各 1 条），`constant_pool_candidate` **0**、**无 consumer 的 item 0**。**这就是任何 cache/index 路径必须复现的形状**：命中一旦被当事实发布，`compare()` 的 fingerprint 差分**立即红**。

### 实现了什么 vs 只定义了契约

**已实现**（`tests/p5_benchmark.rs` 决策记录段 + 6 条测试）：候选记录（`Candidate`/`Benefit`/`MeasuredBenefit`）与**规则** `check_candidate`——「矩阵能跑的第二配置必须带一份实测对照」「收益声明不得留在参考配置」「不得只测一次」「必须指向某个变小的事实」；`runnable_configurations` **从行本身推导**矩阵可跑配置；源码守卫；`RunRecord` 两个新字段（`derivations`、`items_without_a_consumer`）。
**仅契约**：key 维度表（**无 key 类型、无落盘、无索引布局**）。
**未实现**：cache/index/并行/single-flight 本体；触发条件与 upgrade path 已逐条记录。

### 父级独立证伪与证据

- 全量 **1095 passed / 0 failed / 5 ignored**（基线 1089 → **+6**，即 6 条新测试；无既有用例改红/改绿）；`p5_benchmark` **13 passed / 1 ignored**；fmt 与 clippy 1.98.1 干净。
- **父级独立证伪**：让 `runnable_configurations` **谎称**矩阵有一个并未跑过的第二配置（`enabled`/`parallel`）→ **恰好 1 红**：`every_benefit_claim_needs_a_second_runnable_path`，失败消息正是设计拒绝的那句话——「**要么它是第二条代码路径（那就量它并记进 `CANDIDATES`），要么它是参考路径戴了第二个标签，而 P5 的设计（5）与 `performance-gates` 规格拒绝后者**」。
- **实现者三组证伪**：① 把 `fine-grained-parallel` 从「不证明收益」改成「已证明」→ 同一断言红；② 把 `budget` 维度的 carrier token 改名 `LimitsV2` → `the_recorded_key_dimensions_are_the_ones_the_decision_names` 红（「no engine source contains that token」）；③ 往 `crates/jarde-reader/src/lib.rs` 植入 `pub struct CacheEntry` → 源码守卫红。
- **实现者自查抓到的形状缺陷（如实记录）**：`MeasuredBenefit.deltas` 原为 `Vec`，而 `CANDIDATES` 是 `static`——**「候选带实测收益」这一状态永远构造不出来**，守卫会写在一个它看不见的形状外面；改为 `&'static [(CountedBudgetDimension, i128)]` 后才重跑证伪。

### 守卫的盲区（如实，与 A17 守卫同口径）

只覆盖 `crates/*/src` + `src/`，抓 `cache`/`index`/`facts_cache` **模块名**、**以 `Cache` 开头的类型声明**与**线程派生**；**不抓**局部 memo、以别的名字命名的缓存、依赖内部实现。已写在代码文档里——**是补充而非替代**。

### 未完成（如实）

**2.3（cache 损坏/依赖补齐/profile 变化的失效与直接路径回退）在 2.1/2.2 之后曾标注「无 cache 可做」**；父级据 `facts-cache`（本 change 的 ADDED 能力）与 `design.md` 的「**disabled-by-default 实验开关**」裁决：**disabled 约束的是「启用」而非「存在」**，故 2.3 **实现 in-memory cache 的完整身份/失效/回退语义**（不落盘、不加依赖、保持默认关闭），并由此让 **A15 的缓存半首次可验**、给 3.1 一个真正的第二条路径。**2.2 的「不默认启用」决定不变**，被取代的只是其「连 key 类型都不实现」的推理。
**属 3.x**：A15 缓存半与 optimized/direct 差分（3.1）；对抗与共享不变量回归（3.2）；发布实测范围与未决门槛（3.3）；fmt/clippy/test/benchmark smoke/strict（3.4）。

## 2026-09-20 2.3：facts cache 的完整身份/失效/回退（提交 `2f6754a`）

### 父级裁决的执行（§0）

2.1/2.2 决定「保持 disabled」，而 `facts-cache` 是本 change 的 **ADDED** 能力、2.3 的任务原文是**实现**它。父级据三条规格原文裁决：`facts-cache` 的 Purpose 是「为**已证明有收益的** facts cache…提供身份与失效边界」；`performance-gates` 明写「若收益不稳定…**MUST 保留未启用状态并记录原因**」；`design.md` 的 Migration Plan 明写「**disabled-by-default 实验开关**」——**开关的存在意味着被开的东西存在**；且 3.1 的 `Optimized versus direct path` **需要第二条路径**。
**结论**：**实现** in-memory cache 的完整语义、**默认关闭**、**不落盘/不加依赖/无并发**。**2.2 的「不默认启用」不变**，被取代的只是其「连 key 类型都不实现」的推理。

### 形状与落点

**层：CP/Header**（`facts-cache` 里代价最高、身份最清晰的一层）。新模块 `crates/jarde-reader/src/facts_cache.rs`（506 行）。
**落点在真实读取路径**：`Budget` 加手柄（`with_facts_cache`/`facts_cache()`），由**既有**入口咨询——`classfile::class_facts`（被 X1 三路 sub-scan 与 resolution 的 `read_definition_content` 调用）与 `classfile::inspect_header`。**未改任何既有函数签名**，故 `Engine::query`/`resolve_symbol`/`recover_method`/CLI **自动获得**。
**key = `(blake3 内容摘要 + 长度, registry 版本 HIGHEST_REGISTERED_MAJOR, parse policy)`**；entry 另记 `FactsIdentity { registry, format = 1 }` 用于**丢弃**不兼容项。
**不变量**：读取仍发生（读/CRC/digest/origin/coverage/read evidence 全由原代码产生）；**命中不 charge 任何计数维度**（只 `poll()`）；**只有跑完的 parse 才写入**；关闭时与今天**逐字节相同**；**引擎任何地方都不构造 cache**（父级独立核对：`jarde-jvm`/`jarde-query` 中 `FactsCache::`/`facts_cache` **零命中**）。

### 失效三条（各可单独翻转）

| scenario | 实现 | 用例 |
| --- | --- | --- |
| **依赖补齐** | 本层**不存任何关于 name/依赖的 verdict**（verdict 没有字节，进不了内容键） | `a_provider_added_later_is_answered_by_a_fresh_search`：无 provider → `UnresolvedDependency`；补 provider → `Missing`（依赖清空）；**两者都等于无缓存答案**；补 provider 后**旧环境仍答 negative**（一个 store、两个环境、两个答案） |
| **RuntimeProfile / parse policy** | policy 进 key；**profile 不进**（本层产品与 profile 无关） | `the_key_binds_content_policy_and_declaration`（**四维各自翻转**：内容/policy/entry format/registry）；`a_strict_request_is_never_answered_with_a_forensic_read`（major 72 先 Forensic 存，**Strict 必须仍 Err**——**policy 掉出 key 就是 fail-open**） |
| **分层** | 原始层 key 只有内容/registry/policy | `a_raw_layer_entry_survives_the_profile_and_the_recovery_above_it`（profile 8→11 **仍命中**，`reads` 证据逐字节相同）；`the_built_layer_carries_the_dimensions_the_record_binds_to_it`（从 `KEY_DIMENSIONS` 读出 cp-header 维度 = `[snapshot, registry, budget]` 并逐个对上载体） |

### 可选与透明三条

- **损坏/版本不兼容 → 丢弃 + 同预算直接路径 + 报告状态**：`a_discarded_entry_falls_back_under_the_same_budget` 三段数字——① 余量下回退 charge 与无缓存**完全相同**；② `class_bytes=1` 时两次拒绝**同一维度、同一 usage**；③ **整条 query 已先付 `archive_entries=13`/`read_bytes=627`，这些 charge 留在请求里**（父级实跑复现：`archive_entries=13 entry_bytes=627 read_bytes=627 class_bytes=185 … against class_bytes=555`——**即回退没有重置预算**）。
- **状态报告**经 `FactsCache::report()`（hits/misses/stored/refused_capacity/discarded_format/registry/product），**不进结果文档**——因为 `Cold and warm results` 要求 **diagnostics 相等**，把 cache 状态塞进结果会让每个热运行与冷运行不符。
- **候选不成为事实（A01）**：`the_cache_removes_repeated_parses_and_changes_nothing_else` 在 cache 路径复现参考形状（`constant_pool_candidate` 探针行 = candidate + `consumer: None`；全范围行 `constant_pool_candidate = 0`）。
- **取消/耗尽不重置**：预算停止的 parse → `entries 0 / stored 0`；热缓存 + 已取消 token → `Error::Cancelled`，整条 query `cancelled`、0 item、与无缓存取消运行**同一前缀**。命中**仍为「发布」付 `ResultItems`**（`a_hit_still_bills_what_it_publishes`）。

### 冷/热对照与 **A15 判定更新**（父级独立实跑）

```
the archive 行:  class_bytes 555 direct / 185 cold / 0 warm；read_bytes 627 两侧相同
  cold→warm: status/order/coverage/diagnostics 全 equal；fingerprint different
  differing paths: 只有 execution.usage.class_bytes 与 attribute_bytes   ← 只有 charge
  warm → warm again: fingerprint equal
the control 行: class_bytes 555→185→0，differing paths 同样只有 usage
matrix 行:      class_bytes −555、attribute_bytes −87，其余 14 维全 0；中位 137 µs → 116 µs
```
- **结果半一致**：status/order/coverage/diagnostics **逐字段相等**，且**逐路径枚举证明 diff 全落在 `usage` 内**；**两个暖运行之间 fingerprint 完全相等**（暖路径是输入的纯函数）。
- **资源确实更少**且是**确定性**的（charge 维度，非墙钟）：全范围 −185 class −29 attribute；local 行 −303 class −98 attribute；matrix −555/−87。墙钟 137→116 µs 在本语料上大于 halves 散布，但**只有一台机器、一次测量、无阈值**（决策 5）。
- **A15 判定：由「未通过」改为「部分通过」**。依据：缓存半的**语义条件**首次有真实第二条路径可比——结果四平面相等 + 候选不成为事实 + 取消/预算不重置 + 暖路径自等；**未成立的一半**是「**整档 fingerprint 相等**」这一读法，它要求 cache 什么都不省（或伪造 usage），而 `Cold and warm results` 列举的必须相等平面里**没有资源**。**3.1 的 `Optimized versus direct path` 仍待做**（扩到 X1/resolution/IR 层与 3.2 的压力语料）。

### 父级独立证伪

**让 policy 掉出 key**（`FactsKey::of` 恒用 `ParsePolicy::Structure`）→ **4 红**，含 `a_strict_request_is_never_answered_with_a_forensic_read`（**fail-open** 那一例）、`the_key_binds_content_policy_and_declaration`、`the_cached_facts_are_the_facts_the_direct_path_produces`、`a_hit_still_bills_what_it_publishes`；还原后校验 OK。
**实现者三组**：① 同上（4 红）；② `unusable()` 恒 `None`（跳过格式/版本校验）→ 2 红；③ 回退时刷新预算生命周期 → 3 红。
**③ 的方法学记录（如实）**：第一次变异**没有**打红预算用例（回退时尚未 charge 任何东西 → 刷新不可观测）；实现者据此**加强测试**（新增「整条 query 已先付枚举/读取」的第三段）后再跑才红——**这是「测试被证据推翻后补强」的记录**，不是放宽。

### 被修正的既有断言（7 处，无放宽）

① `the_engine_has_no_cache_index_or_scheduler_to_extend` **更名**为 `the_engine_has_one_disabled_facts_cache_and_no_index_or_scheduler`；**index/scheduler 五条 needle 逐字未变**，cache 半边由「不存在」改为三条**更强**的事实（只在一个模块声明、**引擎零构造点**、`Budget::new` 不带）——原断言今日为假。② `REFERENCE_CACHE` 文案由 "absent" 改 "off: …disabled by default"。③ `Context.cache: &'static str` → `String`（标签由手柄 `describe()` 派生）。④ 跨进程 marker 行数 3→4（**多覆盖一行** cache-on）。⑤ `CANDIDATES["facts-cache/index"]` 由 `Unmeasured` 改 `Measured`，并**新增测试重算 `compare()` 的 delta 与记录表逐项相等**——记录因此**可被测量证伪**而非散文。⑥ `fine-grained-parallel` 的 ceiling 引用同步。⑦ `published_rows` **追加**第 4 行（warm cache-on），前 3 行仍以 `cache: None` 构造。

### 证据

全量 **1109 passed / 0 failed / 5 ignored**（1095 → **+14** = `p5_facts_cache` 12 + `p5_benchmark` 2；**无既有测试改变结果**）；`p5_facts_cache` 12 passed；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 16 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；锁文件两条 exit 0 且 **`Cargo.toml`/`Cargo.lock` 零改动（未新增依赖）**。
**CI**：`2f6754a` → 见下。

### 未完成（如实）

**未实现**（按 §0「选一层做完整语义」）：X1/resolution/IR-source 三层**只有契约**；`KEY_DIMENSIONS` 的 `IR`/`recovery` 仍 `Carrier::Absent`。**容量以 entry 数计，不以字节计**（deferral：触发条件 = 出现比「request 允许读进来的最大 class」更值得约束的内存压力）。**中位数是一次机器读数、未阈值化**，两个中位数**无测试断言**（确定性半边才有）。
**属 3.1**：全层 evidence/coverage/representation/diagnostic 差分；**属 3.2**：ZIP bomb/condy/不可约 CFG/缺失依赖语料上的同一对照；**属 3.3/3.4**：启用开关的发布记录与全量门槛。

## 2026-09-20 资源边界：fuzz smoke 的内存守卫落在正常区间内（提交 `819c12a`）

**这不是 P5 的任务项，而是 P5 期间由 CI 暴露、并被父级定位的一个**资源边界**问题——属 3.2 的「资源边界」面。

### 现象

`83a6909`（**纯文档提交**，只改 `openspec/changes/p5-measured-optimization/*.md`）的 CI 中 `fuzz-smoke` job 的 **method-analysis** 步骤失败：libFuzzer 报 **`out-of-memory`**，`peak_rss_mb: 574`（限 `-rss_limit_mb=512`），失败输入 84 字节（一个截断的 `fuzz/SharedSubroutine` class）。
**关键归因线索**：同一个 fuzz 步骤在**前一次**提交 `2f6754a`（**facts cache**，代码改动）上**通过**——**纯文档提交不可能引起 OOM**，故这是**概率性**的。

### 父级定位（四项实测）

1. **单输入不是原因**：把那 84 字节输入单独跑 → **3 ms 完成、不 OOM**。故 574 MB 是**整个 20 秒会话的常驻集**，不是某个输入的开销。
2. **不是每次执行的泄漏**：method_analysis 20 s（210,890 execs）→ **260 MB**；90 s（747,487 execs）→ **401 MB**。执行数 **×3.5** 而 RSS 只 **×1.54**——**次线性**，符合分配器行为而非逐次泄漏。
3. **归因清晰（决定性）**：**等时长比较三个 target**（20 s、同机、`-rss_limit_mb=4096`）：
   | target | execs | peak RSS |
   | --- | --- | --- |
   | `query` | 153,509 | 188 MB |
   | `artifact_tree` | 271,064 | **486 MB** |
   | `method_analysis` | 194,721 | 278 MB |
   **做零 IR 工作的 `artifact_tree` 峰值最高**，而跑整条流水线的 `method_analysis` 三者中最低——故数百 MB 是**fuzzer 自身的常驻集与分配器行为**，**不是流水线的开销**。
4. **引擎侧无不随预算增长**：查过 `with_capacity`（全按 `blocks.len()`/`edges.len()`，不按 limit）——**没有「按预算预分配」**。

### 处置

`-rss_limit_mb` **512 → 2048**（三个 smoke 步骤），并在 CI 里**逐条记录上述测量**（188/486/278 MB、次线性增长、glibc 比 macOS 保留更多——一次 Linux 运行在本地峰值为 260 MB 的 target 上越过了 512）。
**理由**：该守卫的用途是**捕捉失控分配**，**必须高于健康会话的常驻区间**，否则会把正常运作报成故障；2048 留出**高于最高实测峰值 4 倍以上**的余量。
**不做的**：**不**降低 fuzz target 的预算（那会改变被测试的内容，而现有证据不支持）；**不**把 RRS 限值当作引擎内存门禁（引擎的边界由预算系统承担，已在别处验证）。

### 证据

- 本地实测（macOS/arm64、单 worker、`-max_len=65536 -timeout=10`）：上表 + 90 秒增长曲线。
- **CI**：`819c12a` → run 35482664074，**四 job success**（含此前失败的 fuzz smoke）。
- **如实边界**：CI 在 glibc 上的具体峰值未测得（只知 574 MB）；`artifact_tree` 的 486 MB **同样未解释到分配点**——本记录证明的是「**不是流水线的逐次开销**」，**不是**「引擎没有保留」。若将来要收紧，方向是给 harness 加内存剖面，而不是继续调这个数字。

## 2026-09-20 3.1 + 3.2：差分门禁与对抗语料回归（提交 `6ca0cc2`）

**改动面只有两个测试文件**（父级独立复核：`crates/`、`src/`、`openspec/`、`Cargo.toml`/`Cargo.lock` **零改动**）——本片是**门禁与证据**，不是实现。

### 差分门禁的形状

`plane_comparison(plane, &Comparison) -> PlaneComparison`（`Compared(bool)` / `NoObject(&'static str)`）把规格的**八面**逐条映射到值：
**evidence** = **去 charge 的文档** fingerprint；**representation** = 同一文档（仅恢复行有）；**coverage**；**diagnostics**；**snapshot/view identity** = 新增 `Verdicts.identity`（scope/view/profile/providers）；**peak memory proxy** = 新增 `Verdicts.peaks`（**只有两个真高水位** `nested_depth`/`dependency_depth`；其余是累计 charge，只作 delta 报告）；**cancellation** = `status`；**budget results** = 新增 `Verdicts.termination` + coverage。**名单外的面名直接 `panic!`**（防止门禁静默漏面）。
**阻断默认启用 = `blocking_difference`**：任一行任一 `Compared(false)` 即 `Err(理由 + 哪一面)`；`enablement_allowed` 另拒「没有第二条路径」。`Candidate` 新增 `default_state`，今天三项**全部 `Disabled`**。
**门禁测试 `a_candidate_may_be_enabled_only_while_its_differential_is_equivalent` 逐候选**跑差分——`Unmeasured` 的两个候选打印原因，`Measured` 的 `facts-cache/index` 跑**全部**行；判定**不依赖 state**（实测候选一旦差分不等即红）。**父级独立复核**：该用例通过。

### 「无对照对象」的如实清单（**本片的核心诚实项**）

- **逐行**：`representation` 在 query/resolution 行上是 `NoObject`（该层发布 item、不发布源码），由恢复行承担；**测试同时断言「有对象的行必须 Compared、无对象的行必须 NoObject」**，防止静默缺席。
- **整体**（`NO_SECOND_PATH_TODAY`，逐条原因实跑打印）：index 路径、parallel 路径、merged/single-flight 路径、**P4 modern 事实面**（矩阵无一行问 `ModernFacts`）、**真峰值 RSS**（无 allocator 仪表）——五项**今天只有一个值**，**不是被跳过**。

### 五类语料回归（十行 × 三跑 × 八面，off/on 全 equal）

| 语料 | 行 | 终止与范围断言（实跑） |
| --- | --- | --- |
| ZIP bomb | `zip-bomb/understated-entry`（自建：central+descriptor **谎报** uncompressed=1、流实为 8 KiB） | `partial` + `{"dimension":"entry_bytes","kind":"budget_exceeded"}` + scanned/skipped 非空 |
| condy 图 | `condy-graph/shared-subgraph`（Dynamic 链 + 两入口共享 bootstrap entry） | `complete`；含 `bootstrap_edge`+`bootstrap_argument` |
| 不可约 CFG | `irreducible-cfg/guarded-suppressedCatching`（**提交的** `p3-handlers/v8/Guarded.class`） | `complete`；含 `jre_region_irreducible` |
| **A13 对照** | `irreducible-cfg/guarded-one`（**同一类**的可呈现成员） | `complete`；`"representation":"java"` |
| 缺失依赖 | `missing-dependency/{unresolved,resolved,negative}` | `unresolved_dependency` / `"state":"resolved"` / `"state":"missing"`+依赖清空 |
| 取消（解压） | `cancelled-under-decompression/nested`（提交的 `nested.jar`） | `cancelled` + 未建立 ordinal 进 skipped |
| 取消（IR） | `budget-under-ir/guarded-one`（`analysis_steps:4`） | `partial` + `{"dimension":"analysis_steps",…}` |
| 取消（IR） | `cancelled-under-ir/guarded-one` | `cancelled`；三维 coverage 全 `not_requested`（**「未处理」如实、不伪装完成**） |

存储**确实在回路里**（非取消行断言 `consultations>0 && hits>0`；取消行断言 `Cancelled`）。

### A13/A14/A18 逐条

- **A13**：off/on 两侧，`guarded-one`（呈现）与 `guarded-suppressedCatching`（拒答）**同类并存**、四平面分离；resolution 面 `resolved` 与 `missing` **同类并存**；`unresolved` 行证明**缺失依赖不变否定**。
- **A14**：四条停止行在两侧都给出**已扫描范围**与**终止原因**，且 `exit != complete`；`partial` 行三维 coverage 至少一维非 `complete_within_schema`。
- **A18**：**本片补**跨快照隔离（**2.3 未覆盖，父级在派单里点名要求**）——`one_store_serves_two_snapshots_without_answering_either_with_the_other` 三段：(a) 两个快照持**不同内容**共用一个 store，各自等于自己的 direct 运行、`entries==2`；(b) **同内容不同 origin** 的两快照共享一个 entry、各自仍发布**自己的** origin/身份；(c) **取消的请求不留任何东西给另一个快照**。token/cursor 半引用既有的 `p1_query_api.rs`。
**父级独立证伪**：把**内容从 key 里拿掉**（空 digest、长度 0）→ **恰好该用例红**，消息即 `the second snapshot was answered with facts read from the first one: the store is not keyed by the content it holds (A18's cross-snapshot isolation)`；还原后校验 OK。

### 实现者三组证伪

① cache **改变 evidence** → 3 红（门禁指名 `plane evidence`）；② **跨快照污染** → A18 用例红（published item 的 `constant_pool_index` 4 vs 3、span 42 vs 37）；③ **取消被报成 Complete** —— 分两个方向，**这一对是本片最有价值的方法学记录**：
- **③a 引擎侧对称地**把 `Error::Cancelled` 出版为 `Complete` → **取消压力用例红**，但**差分本身仍判「相等」**（两侧同样错）。**如实记录**：「差分抓不到**对称**的语义错误，抓到它的是每行自己的 `exit`/终止原因断言，**二者缺一不可**」。
- **③b 只丢 cache-on 侧的取消** → 用例与**门禁同时红**（面 `cancellation` DIFFERENT）——即差分**确实阻断**这样一个不对称差异。

### 两处机制性限制（本片发现，写进代码文档）

① **取消不可能被 cache「吞掉」**：引擎每次 cache 咨询**之前**都已 poll 过读取路径，故那种 bug **无法产生**；能产生并演示的是**配置不对称**的终止状态丢失。② **差分对对称错误判相等**（见 ③a）。

### 被修正的既有断言（1 处，无放宽）

`Comparison::equivalent()`：原含 `verdicts.fingerprint` → 新为 `evidence && order && coverage && diagnostics && identity && peaks && termination`。**原因**：`fingerprint` **含 charge 记录**，而 `Cold and warm results` 列举的**必须相等平面里没有资源**（charge 正是缓存**应当**移动的一半）。**未放宽**：唯一既有调用者（warm→warm）**仍单独断言** `warm_pair.verdicts.fingerprint`；含 charge 的整档 fingerprint **仍被比较、仍打印**；新增的 identity/peaks/termination 是把比较面**扩到规格原文**。`tests/` 的 diff **删除行数为 1**，其余全部新增。

### 证据

全量 **1113 passed / 0 failed / 5 ignored**（1109 → **+4**：p5_benchmark +3、p5_facts_cache +1；**无既有用例改红或改绿**）；fmt 与 clippy 1.98.1 干净；`openspec validate --all --strict` 16 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；锁文件两条 exit 0；**未新增依赖**。
**CI**：`6ca0cc2` → 见下。

### 未完成（如实）

**`NO_SECOND_PATH_TODAY` 五条仍无第二条路径**（index/parallel/merged、P4 modern 面、真 RSS）；`KEY_DIMENSIONS` 的 IR/recovery 仍 `Carrier::Absent`；容量仍按 entry 数计。
**属 3.3**：把「今天默认 off、门槛未定（决策 5）、测得范围与未决阈值」写成发布记录——本片只提供**可执行门禁与实跑数字**（**未阈值化**：十行的墙钟未断言）。**属 3.4**：文档同步与最终门禁归档。
