# 生产路径对抗审查

## 结论

本次只读核对未发现删除 child `component == child_type` 结构早拒后绕过数组类型赋值检查的路径。改动只让满足既有结构闭合条件的 child array 候选进入 parent proof；是否能写成 Java initializer，仍由 Builder 在 parent 的确切 store BCI 上独立判定。

这份结论基于当前源码逐段检查。root 保存的 `root-focused-builder-v2` 记录显示 `direct_child_array_builder_uses_exact_store_type_proofs_and_stops_atomically` 与 `direct_child_array_chain_is_structural_and_commits_only_as_a_whole` 两个 focused unit tests 通过（2/2）。本审查没有运行测试。它不代表完整六类、双 compiler legs、全仓或 CI 已通过。当前 results 中较早的 `root-focused-integration-v1/result.json` 是 exit 101；其 stderr 记录缺少 `JAVA_8` / `DeclaringClass` 导入和 `&&[u32]` iterator 错误。没有在本次审查中把该旧失败算作产品语义结果，也没有据此声称集成验证通过。

## 结构闭合没有退化成赋值判定

`build.rs:13032-13047` 仍只从真实 `Operation::NewArray` 读取当前 parent 的 component。Child 分支以 `child_values.get(stored_value)` 按完整 SSA `ValueId` 找到候选，再要求 `child.consumer == store.bci()`（`build.rs:13215-13219`）。因此仅在相同或可赋值的 class 名下，不会把另一个数组候选当成该 store 的值；删除的类型相等条件不参与这项身份配对。

数组结构的逐层边界保持闭合。每个元素仍验证 `dup` 的唯一 reader 和两个不同输出、连续期望常量 index、实际 store operand 中数组与 index 的 ValueId、以及 store opcode/component 与 parent 实际 array component 的关系（`build.rs:13106-13203`）。Child 的全部 source（除其作为 consumer 的 parent store）仍必须位于当前 element 的 `[cursor + 2, store_pos)` 区间，且完整 source 集合随后作为 element dependencies 进入 `interval_is_expression`（`build.rs:13219-13248`, `build.rs:13422`）。这个共同 postlude 会拒绝区间里没有被表达式 proof 拥有的指令，不能因为 child 类型不同就吸收 interleaved effect 或未闭合 source。

Child 的 sole consumer 也有两层准确证据：child array reader 根据最终 retained `ValueId` 的 SSA `uses()` 要求一个且仅一个之后的 reader（`build.rs:13688` 起）；父 proof 再要求该 reader 的 store BCI 就是当前 `aastore`，并要求其中的 `stored_value` 恰好命中 child 的 `final_value`（`build.rs:13215-13219`）。构造元素另由 `verify_array_store` 将 `expected_array_store=(store, stored_value)` 传入。该 verifier 检查唯一 reader/writer BCI，并在 `array_store_consumes_site_value` 再比对真实 `aastore` 的 stored SSA ValueId（`init.rs:1160-1174`, `init.rs:2362`）。这些检查都不依赖 class 赋值关系。

## handler、深度、预算和共同提交

Parent 的 handler 基准仍来自分配点的 canonical effect。proof 最后遍历完整 `source_bcis`，对每个 source poll/计费；任何可能抛异常的 effect 都必须与分配点的 handler 列表完全相等（`build.rs:13089-13094`, `build.rs:13472-13484`）。由于 child source 被并入 parent 的 element dependencies 和最终 sources，child 中可能抛异常的指令也接受同一 ordinal 闭包检查。层级仍按 `child.depth + 1` 累积，并在超过 `MAX_VALUE_DEPTH / 2` 时拒绝（`build.rs:13219-13224`）。candidate 扫描、指令、reader use、source closure、site ownership 和提交边界继续使用现有 `Budget` poll/charge；Stop 通过 `?` 向上传播，不会被转换成成功候选。

共同提交仍在一个完整 parent/child chain 上执行。`ArrayInitializers::prove_with_composition` 先把每 block 的候选留在局部 maps；如果候选 consumer 是写入本次 constructed array 的 store，就不让 child 自己单独 commit。父候选随后闭合整条 `children` 链，先对数组 owned BCIs 和构造 Site owned BCIs 做重复/已提交冲突检查，再一并移入 `proved`；链不闭合、冲突或 Stop 都不会提交半条 chain（`build.rs:12735-12862`）。构造 Sites 展平后依赖同一 `claimed` 集合防止相互重叠。报告入口通过 `sites_after_array_composition` 对 pending map 执行一次 `take_pending_sites()`，之后由同一 allocation census 按 head `remove` 并移入 ordinary Sites（`init.rs:515-620`）。这条 handoff 不复制 pending map，也不要求新的消费者。

## Java 类型门仍按精确 store 运行

Builder 渲染 parent initializer 时把每个元素与它自己的 `store_bci` 一起送到 `array_initializer_element`（`build.rs:25348` 附近、`build.rs:26673-26735`）。Reference element 要么是 null、类型完全相同或 component 为 Object，要么必须通过 `initializer_reference_widens`；其 snapshot proof 同时精确匹配 store BCI、完整 source 类型和完整 target component 类型（`build.rs:29434-29459`）。因此结构闭合的 `Object[]` child 可以保留结构事实，但缺少 `Object[] → Base[]` 的赋值证明时，完整 Java body 仍被拒绝。

Facade 的 snapshot widening producer 也锚定实际 store。它只扫描当前方法的 `aastore`；从这个 BCI 的 SSA stack reads 按 stack depth 排出 arrayref、index、value，并分别从 arrayref descriptor 的 component 和实际 stored value type 得到 target/source。loader 必须与本次 runtime loader 一致，reference array rank 必须相等，primitive 或未知 descriptor 不产生 relation（`src/facade.rs:25065-25159`）。之后只在所选 snapshot header chain 能证明 leaf class widening 时发出含该 store BCI 和完整类型 spellings 的 proof（`src/facade.rs:25169-25213`）。Builder 使用 exact 三元组，不把某个 store 的 fact 借给其他 store。facade 预算停止时丢弃这组 widening facts，因而结果是类型拒绝/fallback，不是无证据放行（`src/facade.rs:25314-25324`）。

闭合类型辅助逻辑也没有放宽 primitive invariance 或任意 rank mismatch：`array_reference_widens` 仅覆盖 Java 数组自身的闭集关系；snapshot hierarchy path要求相同 rank；fallback leaf widening 也要求相同 rank；primitive component 不进入 reference-array leaf widening（`build.rs:29461` 之后及 `src/facade.rs:24988-25101`）。这支持“结构事实先闭合、Java 呈现单独拒绝”的边界，而不是将未知类型当成成功呈现。

## 剩余验证边界

Focused Builder v2 的两个测试实际通过，包含 exact `DerivedA[] → Base[] @24`、`DerivedB[] → Base[] @45` proof 正例，以及缺少 proof、错误 source、错误 target、错误 BCI 的正文 ragged 控制；结构测试还覆盖 Object child 结构闭合、错序、额外 reader、interleaved effect、child instruction scan Stop 与 parent chain 原子提交。现有测试并未在本次 focused unit 命令内执行 facade 从 snapshot class headers 生成这两条 widening facts 的端到端过程；该路径的依据在上面标为源码静态核对。完整 canonical family 和全仓/CI 尚未由这两条 focused tests 证明。
