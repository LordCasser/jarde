# Compose constructed reference array elements：当前 proof seam 审计

日期：2026-10-09。只读复核当前 checkout 与 OpenSpec；未运行 Cargo、Git、rustfmt、Java、JADX 或全量 apply，未勾选 tasks。仓库根目录没有 AGENTS.md；按本 task 提供的项目约束执行。OpenSpec CLI 状态为 spec-driven、ready、0/8；其 contextFiles 正是 proposal、design、spec/java8-recovery、tasks。

## task 1.2：调用关系、现有回归与费用

report.rs 当前在 construction census 前先调用 ArrayInitializers::prove，再调用 init::sites（当前约 9375–9386 行）。顺序有实际语义依赖：init::verify 需要读 ArrayInitializers 的 inline_char_argument_bcis 和 inline_argument_chain_bcis，以证明 Java 8 String(char[])、constructor varargs 和嵌套数组参数。不能把完整 init::sites 提前。

ArrayInitializers::prove 按 block 倒序建立局部 candidates/candidate_values，再根据 consumer/child chain closure、ownership排重后才提交 allocations、aliases、owned、element_sources。child array可能已在本 block 候选中完整证明，却尚未进入最终 self.allocations/aliases。现有两个 accessor 只读最终映射；候选 construction verifier 需要一个覆盖“已提交计划 + 本 block 已闭合候选”的短生命周期只读视图。不得 clone完整 ArrayInitializers，也不得先提交parent或全方法预扫描。

构造器相关当前实现：
- init::sites 做 method-wide allocation census，产生 Site 与 AllocationCandidate；无 Budget参数，census及普通 verify 都无逐读取计费。
- init::verify 私有，接 ConstructionFacts。它递归验证 nested constructor，检查constructor区间副作用、参数 dependency、array accessor和handler coverage；本身没有 Budget。
- Site 保存 head/dup/constructor/arguments/owned/instance/expression。instance 是 BCI集合。is_the_instance 仅以 SSA Value Definition 的 BCI membership 判断；这不足以证明一个确切 stored ValueId 就是构造完成的 instance。
- verify 起点现在只判断 operations.get(dup.bci()) == Operation::Duplicate，没有把 opcode 0x59 纳入该结构事实的条件。受限 array candidate路径必须额外核对真正 opcode 0x59，以及准确的 reads/writes instance-copy关系；不要顺便扩大普通 new 规则范围。
- outside_readers 对每个Site按BCI排除produced_by后扫描所有 SSA blocks/instructions及reads；没有 Budget。candidate新增调用要对每条实际instruction/read/use/replacement遍历逐次 poll/IrItems charge并带当前BCI返回StopReason，不能只按Site.expression长度估价。
- array proof 已共享 report传来的 &mut Budget。effects表逐 instruction poll+IrItems charge；candidate allocation、array duplicate/index/store、consumer、single_use、dependency closure与interval也有实际收费点。ArrayInitializers::prove经 report 的 stopped路径保留具体 Budget/Cancelled/Interrupted原因。证明读取费用不是后续 Site::materialize 的NewRecord费用。
- 现有 Sites/verify全路径计费属于既有债务；当前 design明确普通路径避免意外扩大计费。新增 composed candidate verification应使用正在运行的同一个array proof Budget；普通new census以Unmetered分支运行。Site单次移交不重复verify，其费用已在composition候选处支付。
- Builder已有 Sites::site_of/site_producing → new_expr，以及array_initializer_element按 element ValueId/store配对渲染的路径，不需要新AST。不过site_producing也使用上述BCI粗判，成功结构必须留下可让Builder精确连接 stored ValueId与Site的事实。
- 五个 wrapper primitive argument conversions是byte/short/long/float/double，Integer无需显式primitive cast；它们在boxedDirect里是独立未覆盖控制，不允许借本项放宽Operation::PrimitiveConversion。
- Site读数组参数的回归在 init.rs：only_the_complete_direct_java8_string_char_array_is_embedded；nested_argument_sites_present_the_complete_inner_construction；three_layers_present_and_a_deeper_run_keeps_its_outermost_refusal；varargs_inline_array_argument_sites_present_the_complete_chain；incomplete_or_double_purpose_array_chains_stay_refused；a_handler_boundary_inside_the_array_chain_keeps_the_refusal；the_array_chain_certificate_requires_the_argument_consumer_interval。这些helper先证明ArrayInitializers，再调用当前未计费sites，证明既有功能与顺序，不能作为sites已计费的证据。
- 费用测试 inline_char_array_proof_obeys_shared_budget_and_cancellation 用IrItems=1及预取消确认数组proof产生带BCI的StopReason且无证书。新candidate扫描要加入同一共享预算并沿该路径停止。

## 推荐受限接口与事务点（请root确认后再改产品）

### Child-array只读事实

在 build.rs 的 ArrayInitializers::prove block候选上下文中建立轻量borrowed view：委托既有已提交ArrayInitializers，叠加当前block的 candidates与candidate_values；它只回答 init::verify 已使用的两个查询，即精确String(char[])间隔与inline argument array chain。local candidate只有在自身结构/consumer/depth已闭合后才能被view查询。实现形式优先复用这两个现有accessor的逻辑；可加只含此类方法的内部只读接口，或让accessor接受candidate resolver。不能复制全计划、复制扫描或让未闭合parent满足自己参数依赖。

report在ArrayInitializers::prove前已持有ssa/operations/fields/chains/reserved/java_release/member_targets/MethodFacts/code和共享Budget。向ArrayInitializers::prove传一个内部借用的构造组合上下文，或以等价窄参数传入这些既有事实，不新建pass/service。候选证明在array proof的原位按需调用init受限verifier；保持所有其它ArrayInitializers调用的无composition行为。

### Site身份与paired consumer

在已有paired aastore发现点，调用类似 init::verify_array_store(head, index, block, ty, facts-view, (store_bci, stored_value), budget) 的私有到crate内API。它复用现有verify的constructor/arguments/handler证明，但outside readers必须精确只包含该store，并验证这个store的真实ValueId就是构造后的instance输出；Site.instance的BCI membership不能替代。额外读者、第二store、pop、invoke、wrong instance均Refusal而不是成功。

Site消费者身份应在SSA值层完成：检查new输出→dup实际输出→invokespecial receiver及完成对象的SSA replacement/copy链，stored value必须与最终instance值一致。对dup additionally核对opcode 0x59和对应reads/writes，不能只看Operation::Duplicate。已有SsaValue::replaced_by、SsaValue::uses、instruction reads/writes、stack_operands和SsaTable::block是局部身份资料入口；不增加通用alias analysis。候选Site可以记录精确 finished ValueId，或ArrayInitializer保留存值→Site head绑定，以供后续Builder使用；选择可避免BCI重新猜身份的最小方式。

### 内部Site use与outer array use

普通 expression_values_have_single_use 对element_dependencies里的所有stack outputs要求uses.len()==1；直接把完整Site.expression当普通producer会被new→dup、copy→constructor等Site内部use拒绝。只对verifier已证明的Site instance内部边作局部豁免；外部finished ValueId必须唯一读于配对store。构造参数producer的use closure保持有效，不可因为BCI属于Site.expression就跳过其所有use。Site作为一个构造element atom时，完整Site.expression填满array dup/index到paired store之间的physical interval；已有ascending indices、array value单用、final consumer和parent-child closure保持原样。

### 单次移交与ownership原子性

Candidate在prove_array_initializer内通过受限verify后，将Site临时放到prove调用局部candidate_sites map（key为array allocation BCI）；不要加入当前可clone的ArrayInitializer，避免clone Site。最终parent/child array chain闭合时，在同一个ownership核对中组合所有node.owned与每个Site.owned：检查对既有committed ownership、node之间、Site之间以及node/Site相互冲突。只要一个节点或Site失败，整条候选链不提交任何array ownership或construction Site。成功时array节点进入原有ArrayInitializers，Site move进pending transfer map。source/provenance集合可以重叠，statement owner必须唯一。

随后init::sites签名对arrays做可变借用，开始时把pending Site map move到局部map，而不是clone。现有allocation census仍扫描一次；遇到已绑定head时将Site move进Sites并标对应AllocationCandidate.verified=true，不再调用普通verify。剩余allocations走既有普通路径；每个构造allocation仍只有一个verified flag和一个NewRecord。移交是结构证据传递，不重新计proof费用。

结构和呈现是两个平面：后半元素结构失败时，candidate/父链零结构提交，保留所有raw来源；后半element仅在Builder类型兼容失败时，已闭合结构记录可保留，但必须拒绝完整正文和完整initializer，不产生前半Java或悬空独立construction。这个分离已由当前design/spec校正。

### Meter与StopReason

init::verify不能伪称已有预算。为candidate path提供共享meter（如VerifierCost::Shared(&mut Budget) / Unmetered）：前者每处理instruction/read edge/use/replacement、nested recursive candidate、child-array query真实扫描都poll并IrItems charge，记录确切at；后者供原普通construction census保持当前费用。nested recursion复用同一Budget。任何Budget/Cancelled/Interrupted直接向ArrayInitializers::prove的Result传播为StopReason，不能转换成Refusal，也不可以只按Site.expression.len收费。站点materialize继续走原有证据输出预算。

## 历史审查报告的更正说明

历史审查报告 /private/tmp/em18-next-plan-adversarial-review.md 第5项曾把“第二元素不兼容”也要求为Site/array零提交；这已被当前design/spec明确修正。本次报告以当前文档为准：后半结构失败要零结构提交；后半Builder类型拒绝可以保留真实闭合结构与Site/来源，但完整正文必须拒绝。历史报告未修改，仅记录此范围更正，避免把旧结论带入实施。

## 文件/符号锚点

锚点由当前checkout的rg与源码复核，不使用旧Atlas行号：
- report.rs 9375–9386：ArrayInitializers::prove → init::sites；所需构造上下文已在调用方。
- init.rs：Site约68；ConstructionFacts约124；Sites accessor约251；sites约332；verify约435；数组参数查询约631/656；outside_readers约1641；renders_its_reads约1674；Site materialize位于Sites实现。
- build.rs：ArrayInitializer约12532；ArrayInitializers约12556；现有child accessors约12567/12625；prove约12663；prove_array_initializer约12785；generic element collector调用约13040；handler/effect closure约13110；single-use约13253；array reader约13328；collect_expression_bcis约14232；dependency/use closure约14304/14347；interval_is_expression约14388；new_expr约25772。
- stop.rs：charge约115、poll约140。
