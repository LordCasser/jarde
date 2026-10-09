# EM-18 构造元素组合计划：对抗审查

审查范围：compose-constructed-reference-array-elements 四份 OpenSpec 文档、当前 init.rs / build.rs / report.rs，以及已冻结 direct family 源码。此处只读；没有运行构建、Java工具或修改仓库文件。

## 结论

计划方向成立：需要在已有 fresh-array candidate 内，把完整 construction proof 与该 candidate 的确切 aastore 合并；不能单独把 Allocate 加进 collect_expression_bcis，也不能仅凭 init::Sites 已有 construction site 就推定数组存储合法。arrays → sites 顺序有真实依赖，不能整体前移。

但 design 尚未闭合“合并后的两个计划怎样唯一发布”：init::verify 是私有函数，返回 Site；当前 ArrayInitializers::prove 先产出数组计划，而 init::sites 随后独立重扫并建 Sites。需要一个小的 candidate-scoped seam：数组 candidate 请求受限 construction proof，把唯一允许的 outside reader 指定为该具体 aastore，暂存 construction Site，只在整个数组闭合后随 candidate 一起进入既有事实；后续普通 sites census 必须读取或跳过此已提交 site，避免重复验证或重复 ownership。若共同提交实际意味着跨两轮事后协调，请先明确接口和失败回滚点。不能提前全量构造扫描，也不能接受普通 store construction site 再追认数组合法。

## 可复用事实与边界

- init::Site（init.rs 当前约 68 行）已保留 head/dup/constructor/arguments/owned/instance/expression。instance 是 allocation、dup、constructor call 和已证明尾部复制的 BCI 身份集合；expression 是同 block 从 new 到 invokespecial 的完整物理区间。它比另建 allocation registry 或 AST 节点更适合作为 array element 的闭合依赖。
- init::verify（约 435 行）已验证紧邻 new; dup、精确 constructor owner/receiver、参数值在 dup 和调用之间产生、参数依赖闭合、唯一 outside reader、唯一 Java 写入点。普通规则通过 outside_readers + renders_its_reads 拒绝 aastore，正是缺少 candidate 专属消费点许可；不要普遍允许 Operation::ArrayStore。建议给 verifier 加内部受限的允许 reader 参数，钉为 (store BCI, stored ValueId/copy identity)，而不是通用 store 开关。
- ArrayInitializer.elements 已将 (ValueId, store BCI) 成对保存；prove_array_initializer 从 stack_operands(store) 精确拆出 array/index/stored value，验证 index、数组值和 store opcode，并调用 collect_expression_bcis、expression_values_have_single_use、dependency_uses_stay_within、interval_is_expression 闭合元素。
- 只对 candidate-local Allocate element，以完整 Site.expression 代替普通 collect_expression_bcis 结果，并要求该 BCIs 集合正好闭合 dup/index 与对应 aastore 之间的物理区间；仍需检查 store 读到的值正是这个 site 的构造后实例，不能仅检查 BCI 次序、类型相同或单 reader。Site.instance 与 Site.expression 已有身份/范围材料。普通 producer 路径保持原样。
- ArrayInitializers::prove（约 12663 行）逆序处理同 block allocation，先建 child candidate，再从 consumer 链闭合和排重后提交 owned、element_sources、aliases 与 allocations；这是原子提交的已有模型。可借用 candidate 暂存/最终 commit，不要对 proved 全局计划先插半份 Site。首元素 Site 暂存、后续元素失败时，必须保留整个失败构造区间来源。
- effect 检查入口已有：array candidate 将 source_bcis 组织为 scaffold、element dependencies 和 consumer，逐 BCI charge/poll，再对 may-throw effect 比较 handlers() 与 array allocation coverage。constructor allocation、参数依赖、invokespecial 和 store 都必须进入同一个闭包检查；不能只比较构造调用和 store 两端。init::verify 对 nested/array/dynamic/concat 已有 handler coverage gate，但普通 Site 不总做该比较，因此 array candidate 的 effect 闭合仍必要。
- report.rs 当前明确先 ArrayInitializers::prove(...) 再 init::sites(...)（约 9375–9386 行）。verify 还依赖 arrays 来识别 constructor 参数里的已证 char[]、varargs/nested array chain；把 sites 整体前移会令此输入不存在。局部 verifier 若要当前 candidate 的数组参数证明，可读取逆序处理中已闭合的 child candidates/既有 ArrayInitializers 视图；不要新建第二次全方法扫描。
- Builder 已有 Sites::site_of → new_expr 渲染路线（build.rs 约 25772 行起），可在 initializer value 位置产出 new X(args)，不需要新 AST。要确保 ownership 分层互不冲突：array site 负责 anewarray/dup/index/aastore/consumer scaffolding，construction site 负责其 new/dup/init；argument BCIs 在 provenance 中但不能重复作为 statement owner。失败不能留下独立可 render 的 Site。
- 预算已按 candidate 指令、闭包指令、use 扫描 poll/charge。复用 verify 时，新闭包/use walks 也必须收费；不得为找 constructor 先无预算全扫。不能把 Site.expression.len() 当作唯一廉价 charge 来代替它真正执行的 SSA 读取。

## 必须钉住的反例

1. Stored instance 身份：正确 index 的 aastore 消费 alias、另一个对象或构造前 dup 值；必须拒绝。要求证明存值 ValueId 经 SSA use-def/copy规则是该 Site 的构造结果。仅 head < store 或单 reader 不够。
2. 唯一消费：outside_readers 目前覆盖 produced values 的真实 SSA readers，renders_its_reads 不接 aastore。临时许可必须确认所有 readers 正好一个且 BCI 就是 paired store；第二 astore、invoke、pop、field 或其他 array store 都拒绝。加同一 allocation 被 aastore+invoke/第二 aastore 消费的真实负例。
3. 参数/元素效果：verify 已拒绝 constructor 间不属于物理参数 dependency 的 invoke/decoded operation；array 侧要求 exact interval/dependency equality。新组合必须包含构造参数 producer BCIs，保留顺序，不允许在存储点重复求值或重复渲染。
4. exception region 与 handler：构造内每个 may-throw instruction、嵌套数组参数、invokespecial、paired aastore 都按 array allocation 的 handler set 比较；同 block 只是必要非充分条件。
5. 后续元素失败：首个 construction 可证明而第二个不兼容、错 index 或 handler mismatch 时，array 与首个 Site 都不提交；所有相关 producer BCI 仍在 refusal/source 记录中。
6. 既有 constructor 内数组参数：回归覆盖 inline_char_argument_bcis、inline_argument_chain_bcis 和 child array candidates。组合层不能破坏这些事实或改动 standalone consumer 边界。

## 计划措辞与范围

- 新 capability 是 fresh reference-array initializer，须明确不改变既有赋值兼容判据。赋值兼容与 construction identity 是串联的独立证据：类型通过不等于 store 属于该 construction；construction 成功也不等于其 class 可赋给 component。
- Scenario“异构构造元素与同输入子类”边界不清，建议改为“同快照直接/间接子类和接口实现”；至少覆盖 DerivedA→Mid→Base 与 Base implements LocalInterface，并保留每个 store 的 source/component witness。
- design 的“store value 为 site 完整 instance”是核心，但 Site.instance 是 BCI 集合，不是 ValueId。接口必须显式带 ValueId，或由 verifier 内按 SSA use-def 校验 store operand；BCI membership不能替代 operand equality。
- “derived provenance可引用构造BCI，但不得竞争statement ownership”要定义 Site.expression、array element_sources、Site.owned 的关系。要求唯一 owner、所有 BCI 有 provenance，不要求 provenance集合彼此不重叠。
- mark 调用能观察顺序，但现有 direct source 的不少构造参数是 mark → 静态工厂。可加简单的 new C(mark(...)) direct argument；两元素 mark 次序与运行输出可对照原件。
- handler/block负例必须真实覆盖 array allocation、argument invocation、constructor、store 之间的异常覆盖差异；同一基本块不保证 exception table coverage相同。

## direct fixture 与 PrimitiveConversion 独立控制

完整 direct source 已检查：openspec/changes/recover-heterogeneous-array-init/evidence/heterogeneous-array-initializers-v3-pathfix/direct/Main.java 及 Base.java、Mid.java、DerivedA.java、DerivedB.java、LocalInterface.java。Main 包含 sequenceDirect、collectionDirect、throwableDirect、numberGridDirect、collectionGridDirect、ownTwoHopDirect、ownInterfaceDirect、ownGridDirect 和 boxedDirect；mark 递增 trace，可观察 class 和调用次序。DerivedA → Mid → Base → LocalInterface 区分多跳class和接口证据；也有嵌套数组。这里只核实源代码，没有重放编译或运行。

boxedDirect 的六项是 new Byte((byte) mark(...))、new Short((short) ...)、new Integer(...)、new Long((long) ...)、new Float((float) ...)、new Double((double) ...)；其中五个显式 primitive conversions（byte 项也显式）属于 Operation::PrimitiveConversion / Builder primitive-cast路径，不是 reference-array component widening。它把 Number component、wrapper construction 和 primitive conversion 混在一起，不能作为纯 construction-reference-composition成功的单独判据。应留为独立 control：组合改动不能改变其既有行为；若该方法失败，primitive cast 单独归档，不记作 construction identity/store failure。新成功主线用 StringBuilder、ArrayList、exception 和自有 subclass 的引用元素，避免 primitive cast 混杂。

## 最小实现契约

数组候选遇到存值由 Allocate 产生的 aastore 时，调用仅 pub(crate) 的 candidate-specific verifier。它复用 init::verify 构造、参数、reader与 effect逻辑，但唯一 reader期望钉为当前 (store BCI, stored ValueId)。成功只暂存 Site 和闭合 expression BCIs；数组 proof 再按配对 ValueId/index/store、完整物理区间、handler/effects、component witness 完成闭合。只有整个数组 candidate 闭合与排重后，array scaffold 和暂存 Site 才成为既有事实，随后 init::sites 跳过已提交的 exact allocation并保留其他职责。失败时不留下可独立 render 的 Site，保留原拒绝及相关来源。

如果无法在不复制 ArrayInitializers 或提前构造全表的情况下提供这个接口，则“candidate 内复用”和“共同提交”尚未闭环；先补设计接缝再实现，不要扩大 Allocate producer白名单。
