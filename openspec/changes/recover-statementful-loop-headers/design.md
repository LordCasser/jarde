## Context

见proposal。当前2de80主线仅关闭e70产品文档。Seq1旧/当前八API完整非usage JSON恒同；原始42B/25BCI、无handler、两方法无字段。实际临时诊断v2确认：header2自然BCI{2,17}、latch17、header→14退出，14 goto23；header23自然{23,34}、latch34、退出40。首循环默认Frame下dual-exit不适用，header消费者因Store5不满足StatementFree返回fallback并终止后续walk。

第一头部local3 phi@2(value6)来自原前缀value13与回边value28；Store5将add的value16写local4为value17，Load7和body使用此值。第二头部local3 phi@23(value10)来自value6/30，local4 phi11来自value17/25；其dup26/store27可能已有表达式消费者支持，实际修后核定，不能在首拒前声称第二循环已经成功恢复。

JADX冻结输出使用两个while(true)。本地LoopRegionMaker.makeEndlessLoop以单出口break及RegionStack限制out；只借鉴保持头部语句位置的算法。Jarde既有LoopEndless、If.test_effects、LoopBreak和Straight已能表达同一语义，不移植JADX源代码、不新增许可或依赖。Atlas此次返回陈旧结构行号，以当前文件和实际Reader日志为准。

## Goals / Non-Goals

**Goals:** 闭合线性单出口循环头语句，保留每次求值、正常后继、顺序循环的局部绑定和物理转移来源；沿现有结构到AST的职责边界实现。

**Non-Goals:** 不新增公共Region/AST、条件赋值算法、通用effect solver、Frame字段或全局pass；不扩大异常表/非局部transfer/复杂循环体证明，不改变StatementFree含义，不以源码同形美化为验收条件。

## Decisions

1. 在原header条件证明因本头部StatementFree失败处尝试局部消费者，失败仍保留原reason。纯条件路径与原dual-exit路径保持原顺序；不对正常成功候选增加新扫描。只处理无active边界/scope/if-arm/loop-target/try/finally/switch/monitor归属、无handler的顶层形状，保留现有其它证明。
2. 自同次view取自然集和唯一latch，核header比较的真实两个successor（一个在环内、一个唯一外部出口）、物理比较目标/arity与SSA。其余自然节点按唯一normal successor形成覆盖全部节点的有限直线链并最终goto同一header；自然集、Frame可见集、未visited及完整clone ID一致。不可只比较BCI或依赖view省略了异常边后的结果。
3. 全canonical入射行一次有预算/轮询扫描，拒绝重复行、任何非Normal入射、外部→非header入口、外部→header多入口、非证明出口或额外内部边。先核自然集的唯一header→outside与内部链/latch→header；outside若为唯一header前驱、唯一普通后继、同clone的一条物理goto且SSA恰为该transfer，纳入唯一可选出口gateway。最终owners为natural∪该gateway，普通出口相应为header→outside或gateway→后继；后继须不在owners。核entry/出口/内部边的精确集合与物理offset，不吸收带效果或共享gateway；method-entry无前缀形态若实现不支持则拒绝，不默补入口事实。
4. 直接组合现有LoopEndless：body为header的If（退出臂LoopBreak，继续臂空，join为链入口）随后Straight链；header由If拥有、latch由链拥有，tests为空，gateway_origins只记录准确latch goto。无gateway时break来源是物理条件branch，Loop.exit/next为outside；有gateway时退出臂为Sequence[Straight(gateway), LoopBreak(source=其goto)]，该块只由此臂拥有，Loop.exit/next为其后继。原14因此有实际break来源，20仍是latch origin。root读build.rs的instruction分支已确认纯Transfer返回Ok且不出语句，单独把14留给后继Straight必丢map，故需上述精确吸收；不是运行猜测。Builder::If已有test_effects按原序写头部语句，再求test_expr；现有局部绑定/类型证明仍生效，失败可靠fallback。全部事实与owner核完且最后轮询成功后一次visited.extend，不做试走、不需要回滚事务。
5. 不直接把header作为loop_body_sequence起点套现有Frame：boundary=header会在入口停，boundary=exit可能使一臂回到header与外部join不一致；为此扩展普通If/join或新增Frame语义会扩大当前问题。也不放宽纯条件门、不扩写现有复杂dual-exit专证。实现可用最小私有函数隔离上述形状，但不新建证书/抽象层。
6. 所有新增集合构造/链遍历/SSA与canonical扫描按规模计费并轮询，保留Stop，重复源码扫描避免；默认stack、磁盘余量5GiB、target上限1GiB、每秒守卫、finally exact clean。禁止为通过本片改旧P5预算或计量断言，真实新费用须先解释触发范围。

7. root对抗审读发现仅核branch_offset不能排除opcode错配：分支须同时绑定原始opcode/effective_opcode/SSA末指令、宽度及既有decode::comparison结果；latch/gateway同样绑定真实goto家族/宽度与SSA。复用既有comparison解码只放宽crate内可见性，不复制条件sense映射；二分查找费用按实际调用上界2*blocks+8计算。内部错opcode变形是proof-context边界，不冒称合法JVM输入。

补充首次真实测试暴露独立字段复制缺口，见依赖`recover-field-write-test-copies`；原始整方法已通过，补充和Stop前置仍失败。该依赖通过既有FieldCopies单store saved形态及一个同phi Boolean缓存重建解决，未改简fixture或扩大region proof。首次失败11成员冻结在该依赖results，后续完整矩阵单独接受。

第三轮真实定向4内部门全过、原始与公开Stop通过，补充已Structured但缺goto32来源。此处canonical/SSA把goto32与后续phase2语句合并，不能将整个外部块吸入break。对非纯gateway外部块，仅当第一指令是无read/write的真实goto、raw/effective/SSA及宽度一致、物理目标等于同块下一SSA指令且下一指令非transfer时，记录该纯前缀BCI到既有Loop.gateway_origins；完整块保持外部Straight所有权。额外入口/非Normal入射在同一canonical census拒绝，已visited外部块拒绝；不处理多goto前缀，不挪动phase2。费用沿既有二分查找上界计入。第四轮保留缓存语义失败，第六轮671通过出现及完整三方源码矩阵已实际验收，确认该来源修复正确；第三轮失败仍独立保留，不改写为通过。

真实边界输入的双JVM验证与Reader观察：extraBodyExit自然集{2,16,24}有额外body出口；multipleLatches自然集{2,16,22,28}有两个latch，两者直接消费者均拒绝且不claim。dual-entry不形成natural候选；jsr BCI23有真实path[2]/[5]但视图不暴露natural loop。跨分析同BCI错path仅为proof-context负例，不能据此宣称whole jsr-loop支持；此既有架构范围继续单列，不混入本片。

## Risks / Trade-offs

第二次完整workspace seed实际在command75发现两个真实拒绝边界回归：`recover_loop_test_copy_store`的参数目标和MultiCopy均被statementful路径接管，Builder把未证明copy/test引用为注释，却保留Endless while及部分效果。旧完整拒绝文本不得改为此输出。结构闭合不能替代表达式证书；新路径只在StatementFree失败后工作，更必须区分可顺序写出的语句和未获证书的复制。

修正沿既有职责边界：只将现有`build::LocalAssignmentShape`及`local_assignment_at`提升到crate内可见，不增加公共类型或重复SSA算法；新helper在claim前遍历头部`Operation::Duplicate`，须同时满足region既有`unobservable_store_dance_part`或`store_dance_part`准入，以及Builder既有完整物理/SSA/source-store-test唯一用处证明（测试值来自同头部terminal comparison真实栈操作数）。后者补足region粗粒度准入不检查全部单用事实的局限，参数槽边界仍由既有region准入保留。所有额外遍历和每次证明中的头部、entry槽与effect表扫描预先按真实上界计费并轮询，Stop继续传播且不claim。该新消费者不为其它未证明的dup家族新增恢复；已有纯条件路径不受影响。不能粗拒所有Store或Other：独立字段dup_x1证书和头部赋值/调用仍是正例。

验收新增同次真实Reader负控，两编译器完整合法类的参数目标/多复制形状直接拒绝且visited不变，既有公开完整快照原样通过；原Seq1、效果补充、volatile及LeadingTick四组完整正向源码与来源重新绑定最终CLI。若生成源码与已接受矩阵逐字相同，可明确引用该同一Java字节的双编译/双VM证据，不能冒称重新运行或用质量标签替代真实来源与语义。旧两次failed seed和取消CI独立保留，最终两个fresh seed和新产品自身CI仍必须实际通过。

- [头部赋值被抬出循环或重复求值] → 原始溢出/回边用例和带计数、顺序ledger、指定调用号抛出、状态相关返回的完整补充类对照；纯算术原例不能覆盖的路径单列。
- [局部c/z跨两循环声明不完整] → 全方法完整原样重编、SSA/身份/所有物理BCI核验；不新增表达式规则掩盖缺失证明。
- [goto20或已吸收gateway14丢来源] → 公开SourceMap-only/all全BCI/UTF8，精确owner次数和canonical对抗变形。
- [候选失败污染claim或吞停止] → 先只读证明再一次提交；正例扫描中预算中止及取消/期限门，不发布部分source/map。
- [补充helper自身为其它已知缺口] → 完整基线先识别，保留非目标失败；不删除成员或临时修生成源码，独立债务与当前范围分开报告。

## Migration Plan

冻结基线/实际诊断及补充oracle；root审读Luna私有补丁与测试后集成，先定向和完整源码矩阵，再广门/两个fresh seed/自身CI。完整证据后更新handoff并提交推送，所有历史失败原样保留。本片不改变71/612整体计数。
