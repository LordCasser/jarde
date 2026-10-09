# 构造元素数组组合的对抗架构审查

## 范围与结论

只读检查当前 `crates/jarde-java/src/init.rs`、`build.rs`、`report.rs`，聚焦 candidate 联合提交、constructor 参数中的 child array、pending Site 单次移交、`single_use_atoms` 与预算停止传播。未运行 Cargo、Git、rustfmt、Java，也未改产品、tests 或 planning。

在所审范围内，未确认会把半个构造 Site 发布、重复消费 Site、或在递归构造元素的预算停止后继续发布 partial plan 的实现缺陷。设计要求的“父 candidate 成功后共同提交”适用于父数组的 initializer/candidate-owned Sites；constructor 参数中的数组若先作为独立 root candidate 完成结构证明，不是父数组 child 链的一部分。这里的“独立”只指 child array 自身的 initializer 结构事实；不代表 enclosing constructor 已恢复成 Java，也不代表完整正文已成功。下面列出具体提交顺序与边界，便于 root 对真实控制继续核对。

## Joint commit 与 constructor child-array

`ArrayInitializers::prove_with_composition`（`build.rs` 约 12702 行起）按 block 逆序收集局部 `candidates`、`candidate_values`、`candidate_sites`。`prove_array_initializer` 在完整扫描数组元素时可借用已证明 child facts；constructor Site 及其嵌套 Sites先留在 `candidate_sites`。在 candidate 提交阶段，代码先查 child-chain closure、数组 ownership 冲突、Site ownership 冲突；只有全部通过后才取出该 chain 的 Sites，再写入 `proved.allocations`/aliases/ownership，最后 `commit_site`。任一拒绝会跳过该候选；预算/cancel 通过 `?` 使整个 proof 返回 `Err`，局部 `proved` 不会交给 report。

constructor 参数里的匿名数组与“元素直接存入另一新数组”的 `children` 链不同：参数数组的 consumer 是构造参数依赖中的调用或构造操作，不会因 `ArrayStore` consumer 被自动并入父数组 child-chain。它可先独立成为已闭合的数组结构事实。父元素构造验证随后通过 `ChildArrayFacts::inline_argument_chain_bcis_metered`（`build.rs` 约 13020 行起）读取该事实，并将参数依赖纳入 Site expression 的组成验证。若更外层数组之后结构失败，先证明的参数数组仍有自己的完整 allocation/store/consumer 闭包；这只能说明 child array initializer 结构存在，不能推出 enclosing constructor 可独立恢复。若该 constructor 的唯一 reader 是未闭合父数组的 `aastore`，普通 Site census 不会授权该 store：普通 verifier 未收到 `(store BCI, stored ValueId)` 组合许可，`aastore` 也不是一般构造表达式的可呈现 consumer，因此该 constructor Site 仍会拒绝，完整正文仍不能据此恢复。独立 child array 的结构事实不会使父数组或外层构造表达式半提交。

这里有一个需保留为验收边界的区别：第二元素结构失败应阻止该父数组和其 candidate-local Site 提交；不能由此要求撤回原本独立闭合的 child array 结构事实。至于完整 Java 恢复，仍取决于 enclosing constructor 是否有可呈现的唯一 consumer；若唯一 consumer 是未闭合父数组的 `aastore`，body 应保持拒绝，不得把 child array 的结构接受写成 Java 恢复。若未来规格把“父失败时参数 child initializer 的结构事实也必须零提交”解释为字面事务原子性，应先由 planning 明确语义，再改实现；当前代码没有证据支持这种更强约束。

## Site 移交与 census

`report.rs` 在 `ArrayInitializers::prove_with_composition` 后只调用一次 `init::sites_after_array_composition`。该函数通过 `take_pending_sites()` 的 `mem::take` 移出 map，再进入 `sites_with_pending_array_composition` 的同一 allocation census。匹配 allocation head 的 pending Site 被 remove、标记 verified、加入普通 `Sites`；未匹配的 allocation 按原 verifier 处理，census 最后按 BCI 排序。builder 消费同一 `Sites`，没有第二个产品级 registry 或再次构造扫描。

该 census 对 reserved/concat-owned head 会在 pending map remove 之前跳过；但当前可达性上，`verify_array_store` 在生成 Site 前就以 `reserved` 和 `chains.owns(head)` 拒绝该 head，故目前没有可由该路径产生、随后被静默遗弃的 pending Site。若未来增加新的 pending Site 生产者，此不变量应加断言或显式记录冲突；本次不是已确认缺陷。

## `single_use_atoms` 豁免范围

组合处在 `build.rs` 约 13380 行：对 Site expression 依赖调用 `expression_values_have_single_use_except(..., &site.single_use_atoms, ...)`，随后仍调用 `dependency_uses_stay_within_metered`。`single_use_atoms` 的生成在 `init.rs` 约 1290 行，仅含 Site `produced_by`、递归 nested Site 的同类集合、已经单独证明的 embedded/inline array BCIs。它不是对所有 constructor expression BCI 的通用豁免。

这些来源各自有既有闭包证明：构造 receiver/完成值由 outside-reader 与精确 expected `(store BCI, stored ValueId)` 检查；nested Site 递归运行同一 verifier；embedded/inline array 来自闭合 child-array proof。第二个 containment pass 仍遍历依赖 producer 的全部 SSA uses，要求每个 use 落在当前 block 的 element dependencies 或 paired terminal store 内。因此多出的外部 reader不会仅因 BCI 被豁免而穿过。豁免粒度是 BCI 而非 ValueId，单独看略宽；但在目前允许的这些操作（构造 scaffolding 与已证明 initializer sources）上，前置 proof 加 use-containment封住了可观察副作用重放。暂未找到可具体触发的越界接受形，故不将其登记成 bug。

## 共享 meter 与停止传播

`VerifyMeter::charge`（`init.rs` 约 208 行）在有 Budget 时每次先 `stop::poll`，再对指定维度 `stop::charge`。构造 Site 的 `verify_array_store` 以同一个 candidate Budget 创建 metered verifier；递归 `verify_metered` 的 `Stop` 与普通 refusal 分开传播。数组 proof 的外层 block/candidate扫描、effects快照、Site conflict/ownership检查及 commit准备也有 poll/charge；在任一阶段停止均由 `?` 返回，使局部 plan 不被调用方接收。existing report路径因此不会把“停止在后续元素”误作“成功提交前半数组”。

`sites_after_array_composition` 本身没有 Budget 参数，但它只把已证明 pending Sites 移入普通 census，并对其余 allocation 复用既有未计量 `init::sites` verifier；这属于普通 construction pass 的存量计费边界，不是本次 composition 新增的递归 Site proof 漏 poll。若将来要求整个 census 也响应同一预算，应作为计费范围变更独立讨论，不能把它描述成当前组合 budget Stop 被吞掉。

## root 后续核验建议

1. 用一条 constructor 参数含 inline child array、父数组第二元素故意结构失败的 focused control，分别检查 child array 是否作为独立表达式有效、父数组/pending Site 是否未提交；断言语义按上文边界区分。
2. 对候选停在内层 `verify_metered` 的预算阈值，确认函数返回 `StopReason::Budget` 且调用方拿不到任何 `ArrayInitializers`；再对外层 Site ownership冲突拒绝确认没有pending提交。
3. 保留 exact store/stored ValueId、duplicate reader 与错误 stored value 控制，验证 BCI 级豁免未绕过 `dependency_uses_stay_within_metered`。

这些是建议审计断言，不是本报告已运行的测试结果。
