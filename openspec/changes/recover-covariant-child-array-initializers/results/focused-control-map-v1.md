# 当前 focused 控制覆盖图

本记录只读核对了现有实现与测试，没有运行 Cargo、Git、Java，也没有改产品、测试、fixture 或规划文件。重点是区分三类现有证据：array 候选结构证明、构造 Site 的 array composition、Builder 的元素类型呈现。它们目前不是同一个闭环。

## 可复用的真实测试

| 测试 | 当前直接证明 | 不证明的部分 |
|---|---|---|
| `init::tests::composed_reference_array_commits_exact_constructed_values_once`（`crates/jarde-java/src/init.rs`） | 用冻结 `p3-constructed-reference-array-elements-v1` 的 `sequence`、`collections` 两个方法，经 `ArrayInitializers::prove_with_composition` 再经 `sites_after_array_composition`，检查构造对象作为 reference-array 元素时的 Site、constructor output 和 Collection 的 child-array identity atom。 | 不是“数组元素本身是一个 child array、再被父数组消费”的 ownership 证明；也没有验证父 child-array candidate 的索引、consumer 或类型闭包。这里的 `single_use_atoms` 是已验证对象 Site/嵌套 varargs child 的身份事实。 |
| `init::tests::composed_nested_site_ownership_and_store_binding_are_exact` | `NestedControls.nested` 的实际 SSA `aastore` operands；锁定 outer/inner allocation BCI 6/10 与 store BCI 25。正确 stored `ValueId` 成功，替换成 array/index operand、错误 head 或错误 block position 均拒绝；outer/inner 两个 pending Site 各移交一次、owned 不相交，instance 未互相扁平化。 | 嵌套的是 `new Outer(new Inner(...))` 里的构造 Site，不是 nested array；不能替代 child-array parent/child chain 的结构控制。 |
| `init::tests::nested_composition_budget_can_stop_inside_recursive_constructor_proof` | 绕过 array candidate census，直接对上述真实 block/store 调 `verify_array_store`；充足预算证明成功，`analysis_steps=1` 在 outer 扫描后进入 inner argument scan，并精确 Stop 于 BCI 14。 | 这是递归 constructor proof 的预算控制，不是 nested-array 结构扫描的停点。 |
| `init::tests::fresh_array_index_byte_variants_do_not_commit_partial_candidates` | 从实际 frozen sequence class 派生重复/倒序 index Code 变体；重分析后 array candidate 不存在、pending Site 为空；同时保留首个单独元素 store 可证明的 counterproof。 | 控制对象是 reference-array 内的 constructed object chain，不是父 child-array 的两个层级。倒序首元素的断言也不应被描述成已走到后续 store。 |
| `init::tests::extra_stack_reader_byte_variant_reaches_exact_site_value_guard` | 从真实 class 派生额外 stack reader；SSA 明确分析 duplicated/completed/stored aliases 与实际 reader，candidate 与 pending Site 均不提交。 | 仍然是 object Site 与其 array `aastore` reader，不是 child array 的 retained array reader。 |
| `init::tests::handler_range_byte_variant_reaches_array_effect_closure` | 两个 frozen javac legs 的 classfile exception-table 变体，校验改动确实到达 array effect closure；在 handler 边界跨越时拒绝且不提交。 | 保护 object-array composition 的 exception-region 闭包，不直接保护 nested child-array effects。 |
| `init::tests::array_argument_conversion_is_accepted_but_unrelated_conversion_aborts_the_candidate` | 正向证明第二个 `Long` constructor 的真实 i2l 参数依赖；派生的无关 i2l 不属于参数闭包时，完整 array candidate 与 pending Site 均不提交。 | 该变化验证构造对象参数的 effects/dependency closure，不等同于在 child-array-parent interval 插入 effect 的控制。 |
| `build.rs` 类型 helper：`initializer_reference_widening_reuses_scalar_and_equal_rank_array_facts`、`initializer_snapshot_widening_requires_the_exact_store_and_full_type_pair` | 对 scalar/equal-rank reference-array widening 的闭表、release、rank、方向，以及 snapshot proof 的精确 store BCI、source、target 进行纯 helper 断言。 | 没有实际 SSA child-array 候选抵达 Builder；不能证明任一 array store 的 ValueId/BCI 与类型 proof 实际配对。 |
| `tests/p3_heterogeneous_array_initializers.rs::direct_new_family_pins_recovered_constructor_methods_and_remaining_controls` | direct family 六个 class/JDK leg 中，已有的 object-constructor direct positive/refusal 记录；当前 `numberGridDirect`、`collectionGridDirect` 明确是完整 body fallback，且对应 `news` 为空。 | 当前 direct child-array grids 仍未恢复，故其已有 fallback 不是 child-array 正向证明。 |
| `tests/p3_heterogeneous_array_initializers.rs::factory_families_are_complete_mapped_and_match_both_frozen_jdk_legs` | factory family 的完整 class-source、recompile、`-Xverify:all` 和双流对照；其中是完整 family 的端到端验收入口。 | factory 不等于本片 direct child-array 三 grid；不能拿 factory 成功充作 child-array 结构或 Builder 类型证明。 |

实现位置可直接对照 `crates/jarde-java/src/build.rs::prove_array_initializer`（约 13019 起）：它按物理 block 中的 `dup; index; value; aastore` 扫描，先将 `stored_value` 与 child candidate 或构造 Site 配对；child 分支当前比较 `component` 与 child array 的实际类型，并要求 child sources 落在当前 element interval。随后公共 postlude 校验 `interval_is_expression`，最后对 source/effects 比较 handler set 并提交 candidate 链与构造 Sites。构造 Site 的 composition 则在 `crates/jarde-java/src/init.rs::verify_array_store` 和 `sites_after_array_composition`，两者不应与 child-array 的 parent/child chain 混为一个测试目标。

## 预算和停止证据的边界

- `nested_composition_budget_can_stop_inside_recursive_constructor_proof` 是当前唯一清楚定位到递归 inner scan 的预算控制，但它走的是 constructor Site。
- `composed_reference_array_commits_exact_constructed_values_once` 的 `analysis_steps=1` 仅断言 array+construction proof 会 Stop；它没有断言停在 child-array structural scan 的具体 BCI。
- `tests/p3_heterogeneous_array_initializers.rs::class_source_budget_and_cancellation_report_their_exact_stops` 分别把 `output_bytes=0` 与整体 class-source analysis budget 用于公共 API，核对返回的 partial report 与 stop dimension。前者是报告输出停止，不是 nested structure 停止。
- 同一 integration 文件的 `class_source` helper 用默认 `task_budget(&[])` 生成完整报告；所有 normal factory/direct 调用完成后，class-source 入口在结束处对 `report.text` 应用 `class_source` 预算并记录本次 stop（见 `crates/jarde-java/src/report.rs` 的 class-source 组装/终止流程）。这不能证明 inner child verifier 在特定结构检查处 Stop，也不能证明 Builder 正文 renderer 的预算/取消边界。
- 当前代码/测试没有一个清晰、专属的“已进入 nested child-array proof 后，在 child instruction scan 中精确 Stop”控制，也没有“已通过 structural child proof 后，Builder 在 child type presentation/render 位置 Stop”的控制。不要用 factory 的 `output_budget` 或报告末尾 `class_source` stop 填补这两个空缺。

## Builder store/type proof 的已知闭环

`array_initializer_element` 在 `build.rs` 根据 `stored_value` 渲染出的 `Expr` 的 `presented` 类型与 component 类型做兼容判断；引用分支只接受 null、精确类型、Object、`initializer_reference_widens(...)`。该函数把 snapshot widening 限定到 `proof.bci == store_bci && proof.source == presented && proof.target == required`。`NewArray` 呈现分支还要求 `element_sites[element_index]` 对应 Site 已提交，且 `site.finished_value == Some(value)`，失败时正文报错并走 fallback。

因此 helper 测试已经钉住类型 fact 的 BCI/source/target 精确性；`composed_nested_site_ownership_and_store_binding_are_exact` 钉住实际 constructor stored ValueId 的 store binding。但目前没有 actual child-array 变体同时证明：child candidate 与 parent `aastore` 的结构/ownership 已闭合，之后由于真实 Builder component type mismatch 而拒绝完整正文。`child-control-construction-plan-v1.md` 提出的 `ownGridDirect` 第二层变体可补这一真实桥接；必须先证明 child 与 parent store identity/interval 都通过，再断言 Builder fallback，不能只看报告 fallback。

## 最少新增 focused 测试建议

1. 结构/ownership 控制放在 `crates/jarde-java/src/build.rs` 的现有 `ArrayInitializers` 私有测试区，贴近 `prove_array_initializer`。一条真实 `ownGridDirect` SSA 正例应断言 outer/inner array allocation、child store 与 parent store 的 exact operands/BCI、children 链和 ownership 无重复，以及最终 candidate/pending 转移一次。复用现有 direct fixture，不增加产品 helper/type。
2. 同一测试区复用该 classfile Code-span 派生控制，覆盖父 index 顺序、child retained array 的额外 reader、parent interval 的独立 effect、depth 及 child proof 内精确 `AnalysisSteps`/预取消；每个结构拒绝均断言 parent 与 child 均无半提交。只在真实 counterproof 可独立证明时记录其范围。结构闭包的测试应读内部 candidate/site，不以最终整方法 fallback 代替拒绝点。
3. Builder 的真实类型负控放在 `tests/p3_heterogeneous_array_initializers.rs`，复用 `DIRECT_JAVAC8/23`、class-source/archive/body 路径与计划里的 Object[] child patch。先证明相同 child-parent `aastore` 结构仍能提交，再断言完整 body fallback/准确类型拒绝。已有纯 helper 用例继续保护 exact store/type pair、primitive invariance 和 rank，不另复制类型表。
4. 预算要分开：build.rs focused direct proof 预算在实际 child instruction scan 上 Stop（测试确认完整预算先成功且 stop BCI 落于 nested child scan）；Builder/body 停止若要覆盖，应在真实完整正文入口单独构造对应预算并确认 stop/content，不以 `output_bytes` 或 scan census 替代。

这些是测试位置和证据边界建议，不是已执行结果；按 OpenSpec 仍由 root 对抗验收后认定覆盖。
