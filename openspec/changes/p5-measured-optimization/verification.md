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
