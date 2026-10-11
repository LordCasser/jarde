## Context

动机见 proposal。固定上游适配整类和同一次 Reader→canonical→SSA→NormalFlowView→recover 的实际证据已提交于 evidence/java-syntax-2026-10-11/cf11-testnestedloops4-{baseline,same-ir}-root-v1。15节点/19正常边/40物理指令；中间 natural loop 为17/23/56，退出分支23→28经内层31/39/47、return45或gateway53→62。原 scope 截断28，临时 body 为不完整 If(join=None)，外层10接续最终拒绝。arm boundary index5准确指向BCI62；loop_arm_join_source首个closed_body拒绝为真实Region形状与源码推导，未独立插桩该helper。

JADX LoopRegionMaker 先以多出口与 edge-specific break处理Java正文，再由LoopRegionVisitor转for；本仓参考问题分解，不复制其可变CFG或宽泛frontier例外。当前Atlas持久索引行号过期，架构审查已以实际源码和真实IR为准。基础Reader/SSA/normal-flow事实正确，本片不修改其图或natural集合。

## Goals / Non-Goals

**Goals:** 完整恢复一个计数/条件循环正文中的单入口退出分支：准确前缀→一层fresh可约内嵌循环→本循环break gateway，并允许内嵌循环的已证terminal return；随后在外层条件中一次呈现整个prefixed loop arm、一次呈现外层continuation。所有物理指令来源、求值顺序、循环次数和return/break目标正确。

**Non-Goals:** 通用多出口CFG恢复、任意深度退出子图、复制共享正文、异常/handler退出路径、跨canonical path、改变natural membership、增加公开IR/Frame/pass/依赖、强制三个for。Builder现有表达式和值消费不作无关重构；CF16 finally和其他独立债务后排。

## Decisions

1. **先证归属，再扩scope。** 在header_tested_loop的既有body/expected构造附近，以当前循环准确normal exit和outer frame为界，识别一个natural-body比较分支的外部后继；另一后继必须属于本循环正常迭代路径。退出分支限为独占straight prefix进入一个fresh、可约natural loop，再由独占单goto gateway到本循环normal exit。内层除已证明的return leaf外只有该正常出口，所有内部/外部边完整核准。成员来自已存在natural facts，前缀采用已有exact_straight_chain_edges等判据；不从可达集合猜所有者、不新建通用DFS恢复器。若以最小私有证明记录携带owners/端点，限于该次claim，不加Walker长期并行缓存。
2. **准确嵌套和边界。** 退出子图的prefix/child natural/gateway必须不在本循环natural set、互不重叠、同canonical path、在准确outer scope内且未被认领。子图外入边只能是原分支→prefix，child header只有prefix及自身合法backedge入口；gateway前驱/单goto/目标完整核对，Return叶复用loop_terminal_returns的唯一比较前驱、无canonical出边与decoded Return判据。返回叶可由outer loop已有terminal证书覆盖，但不得借当前尚未扩展的body scope凭空获证。任何exception edge、额外exit/entry、irreducible或不明child嵌套均拒绝。natural set保持不变，退出路径是Java正文owner，不伪装为迭代块。
3. **共用证明结果与原恢复入口。** 将准确新增owner加入本循环body frame和expected coverage，再让普通Region walker消费现有Loop/If/Sequence/LoopBreak/Straight。从child正常exit恢复gateway，LoopBreak准确指向中间循环，Return仍是方法终止。先后顺序必须保持prefix、child、break；LoopBreak本身无blocks，物理gateway仍由其Straight或现有来源证书持有。恢复后要求实际Region blocks+精确来源所有者与proved expected一致，每个owner一次；不能把空Straight当完整外部arm，不能用covers只检查natural部分就接受漏正文。
4. **外层接续消费完整结构。** 保留loop_arm_join_source原单出口路径；新增形态只能通过上项证明的完整owners/端点接受。核本循环header正常exit与gateway出口都到同一arm join，已证return终点离开方法；原canonical natural边23→28是已归属组件入口，不再是不明loop出口。核全入/出边，不把owners==natural改成任意超集，也不把唯一exit_source改成任意多个。若需要多个join sources，沿已有two_level_loop_join_sources的具体source列表方式消费，保留唯一source调用者的约束，不以随便取一个source绕过tail前驱检查。外层straight prefix单入口、join只在arm外恢复一次；无join If/nested Loop仅在其全部边和transfer/terminal已被准确证明时接受，不递归放开任意结构。
5. **失败原子性与计费。** 纯shape/edge证明不认领owners；正文或parent-arm trial保存并恢复原visited/unclosed_tail/catch_joins/fragmented等实际会变的状态，失败/Stop不遗留半个loop。沿现有charge/poll为节点、canonical边检查、必要存储和trial工作计费；同次证明/消费共享已有结果，避免靠不计费的重复扫描。诊断测试需在真实IR内找到证明阶段实际Stop，核reason/dimension/usage/BCI和公开空artifact；预取消单独记录，不冒称中途取消。
6. **可观测验收优先。** 固定原方法12只能证明正常结果，不能证明动态不可达的return0。另构造保留该嵌套break形态、由独立returnThreshold控制内层return条件的真实Java类，覆盖未命中/命中/跳过inner/zero inner迭代及调用或计数的可观察顺序；原class为行为真值。新源码完整原样双JDK编译运行，JADX单独对照，失败如实记录。形状反例可用同次真实IR的局部边扰动，但明确不是JVM可验证class；必要公共反例使用真实编译fixture。

标准库BTreeSet、现有Runtime/IR/预算及判据足够，无外部库缺口，不需要复制JADX算法代码；若实际实现提出新通用机制或扩大形态，先回到设计审查，不能顺手混入。本片新增的是私有局部证明分支及消费，不建立第二套恢复架构。

## Risks / Trade-offs

- 只修外层closed_body会发布漏掉内层的结构 → scope与完整expected先闭合，parent最后验收。
- outer scope包含退出路径不等于middle已拥有它 → 全canonical边、唯一入口、端点与actual owners共同获证。
- physical gateway可能被LoopBreak吞掉 → 全物理BCI及准确UTF8范围/方法身份验收，保留gateway owner。
- 正常返回12掩盖早return丢失 → 独立threshold受控原类实际执行早return和计数。
- 较宽结构影响旧prefixed/return-arm/foreach → 同版本永久回归和fresh workspace，必要计费pin仅按实测修正。

## Migration Plan

前置产品自身CI和干净主线已关闭；只在main工作，不新建worktree。Luna私有候选，root串行审查、应用和工具链实跑；5GiB free/target1GiB一秒守卫，完成clean。固定上游源/工具/class/CLI pins和历史失败保留。永久回归、真实Stop、完整双JDK三方源码、本地workspace/同CI Clippy/strict、实际产品自身CI全部接受后才能关闭任务并更新handoff/ledger；不关闭整CF11或71单元。
