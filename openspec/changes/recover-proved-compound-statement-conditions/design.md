## Context

动机见 proposal。已应用但未整体验收的 continuation 候选在真实 Switch2 上证明 dispatch0 的直接 target164 为唯一续接；随后普通 region 恢复在78拒绝。关键物理边为 `98→111/118`、`111→118/131`、`118→124/136`、`124→131/136`、`131→136`。131是共同 payload，136是物理 return。候选 CLI SHA `fd2d526f3e684d13179d344cf7394bbfeefb391d541ea55d01dafd81ab3d0054` 的完整类对照仍记录 Switch2 编译失败；这些失败必须保留。

本地 JADX HEAD 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。`IfRegionMaker.mergeNestedIfNodes`、`isInversionNeeded`、`canMerge`、`mergeIfInfo`、`canSelectNext` 先沿 then/else 路径递归合并、对齐另一个出口、必要时反转，再创建一个 IfRegion。参考算法思想，不移植其通过等价 return 或重复路径省略物理 block 的规则；本项目必须保留独立来源与 canonical 归属。此前笔记出现的末位e是root笔误，不是另一冻结版本。

现有 `Region::If` 只有一个branch；Loop多测试仅覆盖同质逻辑。`TwoExitReturn`/`ShortCircuitValue`的生产者及消费者前提不适用于语句payload，且现有builder通过clone展开shared successor会复制测试表达式。现有 `test_expr`、依赖/类型核验、`short_circuit_not`、`logical_expression`及普通If AST足以构造已合并的表达式，无需新AST或表达式解释器。

## Goals / Non-Goals

**Goals:** 先证明有限纯测试闭包，再按原边顺序归约成每个测试一次的AND/OR/反转树，最后恢复两个普通语句出口。保留现有frame/case/loop边界、完整来源、预算与原子性。

**Non-Goals:** 不新增通用布尔代数优化器、BDD/SAT依赖或第二套普通region walker；不复制一般DAG成ternary树，不增加临时布尔局部/labels，不改已有TwoExitReturn/ShortCircuitValue范围或成本，不迁移所有旧If形状，不支持纯transfer gateway或条件内赋值内联的新形态。相关独立债务留待以后处理。

## Decisions

1. **合并条件与语句归属分层。** 在普通双后继分支入口、走arms之前尝试窄条件证明。仅收集真实解码为比较的测试，最多沿用既有24测试上限；第一个测试可以有既有普通If允许的lead，内部测试必须由现有SSA dependency/effect规则证明全部指令用于其条件。遇到内部独立语句时将其视为语句出口，不跨越它继续收集。两个不同的出口必须完整覆盖所有路径；少于两个测试或多于两个出口拒绝候选。禁止将return/payload本身吞作测试。

2. **canonical闭合先于短路归约。** 用真实边计算有限遍历/拓扑关系，不以BCI大小证明无环或拓扑。每个测试须有精确解码fallthrough/taken身份，完整canonical出边恰为两个互异Normal successor，内部入边恰来自已发现测试且互不重复；root的入口沿现有frame/前缀允许路径。边界外节点、已visited的内部节点、clone-path改变、loop header、未准入case entry、异常/子例程边拒绝。边界可作为出口，不能作为内部测试。扫描逐项实际计费并poll。无环、入口、scope与归约全成功前不提交visited。

3. **采用出口对齐归约，避免表达式展开。** 每个测试摘要保存当前条件树、两个canonical出口和其唯一测试集合。沿原then/else路径选下一摘要；必要时仅反转条件和交换出口。沿then合并时要求另一个else出口精确相同，组成AND；沿else合并时要求另一个then出口精确相同，组成OR。复杂路径先有界递归合并子摘要，再检验出口对齐；任何合并要求测试集合不相交，内部共享节点所有真实前驱已被当前组合覆盖。归约结束后根摘要必须覆盖全部测试、两个真实出口，每个测试叶子恰一次，未消耗共享关系或歧义结果均拒绝。实际图可先合并98/反转111为 `(t98 && !t111)`，再合并118/124为 `(t118 && t124)`，沿第一个摘要else合成OR。方向只由decoder/`test_expr`约定确定，不能写死BCI公式。

4. **最小必要结构是复合条件的证明载体。** 增加内部CompoundIf结构，其prefix、两个普通arm与join沿用If语义；另保留实际tests/test_edges及按索引引用这些测试的AND/OR/反转树。单测试旧If保持原表示和行为。新结构必要在于一个branch字段无法持有四测试共同的条件与来源；不增加第二套Region图或公开请求/IR/Frame schema。所有Region汇总、block归属、path/guard/来源/递归消费者必须显式处理新结构；只适用单测试的专项形状继续拒绝新结构，不能忽略额外测试。普通arm仍由现有walker恢复，payload131只属于其arm，join136外置并由包围结构消费一次。

4a. **共享void return属于包围arm的有限接续。** 实际root诊断已确认Switch2外层block71（physical branch75）的arm是单个If78（physical branch95），then为已证明CompoundIf98/111/118/124、else为空，两个结构的join均为136；136为单条void return，外层frame boundary为164。保留既有boolean-return/Phi消费者的窄证明。只为此已证明statement条件形状复用terminal continuation步骤：outer If及其CompoundIf的join必须均为fresh next；支持的所有arm仍为已闭合普通region，所有prefix/test/arm物理owner唯一且在原frame/path/scope内。完整扫描canonical边，next的入边必须全部为这些owner中准确的Normal来源，拒绝重复、外部或非Normal入边。next必须是单一物理`return`(0xb1)，无successor/leaving edge且不等于frame边界/case入口。既有walker只可新增该一个return块；所有失败和Stop恢复ownership/其他walker状态，预算保留。return接到外层arm，不进入条件、被复制或跨到164；不支持任意return值/嵌套语句图，也不为测试扩展公开CFG API。

5. **builder复用表达式与拒绝路径。** 在deferred/suppression分析前先验证所有测试SSA依赖、Java boolean类型和已证明条件树，再构造一份条件表达式；沿树引用每个测试一次，使用现有 `test_expr`、`short_circuit_not`、`logical_expression`。反转浮点条件保留显式Not，不能用关系运算互补破坏NaN语义。first-test lead仍先输出，其他测试不生成独立语句。条件及两个arms全可消费时才输出现有StmtKind::If；失败保留完整拥有范围的fallback，不留下已省略的生产者或部分map。不复制既有boolean-return/Phi消费者证明，也不借此次任务通用化它们。

6. **独立证据与组合交付。** continuation和此变更分别记录tasks/验证，可在同一root候选中串行集成并用同一个冻结CLI组合验收。Switch2旧完整失败不能改成通过；只有新真实整类编译、状态矩阵执行与准确来源闭合后才完成对应正例。FT2及其他九fixture作为对照；JDK8 helper major55造成的环境不可用单列，不作为产品回归或通过。必要时另建经过真实编译的兼容helper对照，不改原check、不修改冻结证据。

## Reuse, Maintenance, and Licensing

标准库集合与现有canonical CFG/SSA足以完成24测试范围的出口对齐合并。图库不提供本项目完整边、clone/frame归属、Stop和来源契约；BDD/SAT也不自动保持副作用顺序，因此不增加依赖。仅参考本地JADX算法，不复制其Java实现；维护仍由jarde-java负责，依赖和第三方许可义务不变。读/解码、Java8方言准入、测试JDK选择和源码恢复保持分层；解析成功、恢复、重编、执行分别记录。

## Risks / Trade-offs

- 共享测试被展开或重排 → 合并前保存canonical身份，最终每个test叶子一次；用调用记录和抛异常位置作实际对照。
- 错误跨过payload/外部入口或case边界 → 完整canonical闭合和SSA独立指令核验，候选失败前不commit。
- 新结构被旧专项当成单分支 → 审计全部Region exhaustive和If-only消费者；不能证明则继续窄拒绝。
- rootlead被误省略、budget被吞成一般refusal → 沿用现有lead/deferred和Stop传播；在新增证明与builder分别实测。
- 有限DAG不能无重复归约 → 保留明确拒绝；该机制不宣称覆盖任意布尔控制流。

## Validation and Delivery

root独占Git/Cargo/rustfmt/JDK/JADX/CLI，Luna只准备private候选。用户已批准机器最低5GiB余量、本仓target1GiB上限，一秒守卫进程组中止、每轮完成cargo clean。运行真实Switch2、独立有副作用和NaN条件完整源码对照、负向ownership/edge/scope边界、预算/取消原子性及适用控制流回归。来源以真实class digest/方法/ordinal/javap BCI和UTF8范围校验；严禁用文本golden或仅Structured标志替代行为验证。产品自身CI接受后才记录干净交付，不增加未接受的71单元完成数。
