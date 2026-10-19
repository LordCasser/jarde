## Context

见proposal。完整原件基线46实际命令：原件和JADX各四VM/八观察相同，Jarde八API目标Fallback、16整类编译失败、无Jarde目标运行。14条BCI虽都有注释来源，并非可执行恢复。

root已实际运行同轮Reader/IR探针1/0并精确恢复源码：debug/no-debug均为23B Code、五块0/2/8/13/21，正常边0→2、2→8/13、8→21、13→2/21。自然循环仅2/13，唯一latch13、ipdom21；slot2循环Phi V4=[V9,V15]、出口Phi V7=[V14,V15]，分别保留初值3、出口写0与iinc结果。实际header-tested ForHeader=None，首次内部gap就是ArmsDoNotMeet13。地址和值ID只用于说明实例，生产不得匹配它们。

现有loop_region的body_branch、latch_tested_loop的两道出口检查都只承认loop_exit_bridge的单goto；8含Push/Store/Goto，不能通过。effectful_dual_exit_loop要求三个自然循环块与纯回跳，本例两个块不匹配。现有Frame.loop_body已把单后继8→最终出口21加入候选scope，region_at的transfer gateway分支会保留8内语句；break只在21解释。scope候选不是所有权证明。

## Goals / Non-Goals

**Goals:** 证明普通后测循环的一个独占单块出口语句前缀，交给原If臂遍历，原latch前缀与测试仍各一次，最终return保持共享一次。

**Non-Goals:** 不支持多块前缀链、任意跨层break、异常/cleanup/jsr/clone/switch出口，不放宽纯goto桥或三块双出口旧证书，不扩大自然循环集合，不新增Frame字段、Region/AST/IR种类、pass、全局solver、跨候选索引或ForHeader机制。不做值重写，也不按常量0或本类名/BCI/slot匹配。

## Decisions

1. **补局部所有权证书。** 只在实际单latch后测形成立、旧纯goto桥不匹配时读一个在自然集合外的普通直线块；它必须含保留的语句且以唯一goto/goto_w到同一latch最终出口。source属于当前自然循环，prefix、header、latch和最终出口身份互异，prefix未被占有，parent scope若存在必须已允许它。逐项核同轮Code/SSA块完整、非clone、准确唯一正常入边来自该source、准确唯一正常出边到该最终出口，无额外入口、异常/call/未知边或角色重叠；不能以normal-flow可达或loop_transfer_sources候选代替物理闭合。
2. **保持旧语义边界。** loop_exit_bridge继续只承认无效果单goto。新增窄证书用于body_branch和latch_tested_loop一致的形状判断，不单独删一处拒绝。现有scope/普通If递归负责prefix语句，LoopTarget.break_target始终是最终出口，不能提前改成前缀。exits集合本身不代表break，已有gateway区别足够；不重写该元数据。多块链留作后续有证需求。
3. **原遍历和表示足够。** 受证时走现有latch-tested LoopForm::DoWhile、Straight、If和LoopBreak；不复制prefix或共享return，不把赋值移到循环外，也不让break执行latch自增/测试。旧pure bridge优先级和正常header/for规则保持。现有Builder负责普通赋值与Phi展示；若出现独立Builder缺口应另列依赖，不能扩大此片。
4. **预算和状态是证明的一部分。** 新节点/指令/边/所有权扫描逐项charge并poll，沿现有Result/Stop契约传播，不将Stop转None或退款。证明先于占有且只读，不增加事务机制；若实现引入已修改状态后仍可能失败的试探，只复用既有checkpoint并恢复visited和unclosed tail。最终结构/唯一owner/全覆盖检查仍为权威。
5. **参考算法而非移植框架。** 已读本地JADX LoopRegionMaker.insertLoopBreak：沿退出路径找break位置、检查跨循环与控制流。采用其保留break前指令和精确出口的分工，Jarde用既有所有权证书表达，不移植改图/属性系统。标准库与现有Runtime足够，无新增依赖；原件许可保留LICENSE/NOTICE，没有复制JADX生产代码。
6. **分层验收。** 解析、Java8校验、Runtime选择、完整JVM验证、源码恢复和有限运行分别报告。固定完整outer/child源码与两profile不裁剪；class/plainJAR×default/all共八API，目标structured、身份/全文/maps一致，所有物理来源有效且缺口逐列。最终JDK8/23整类重编及隔离VM比较k=9返回5、k=10返回0、异常类别/stdout；保留空cwd、parentless、CodeSource和错根拒绝。两case不能区分等价while，不强制某种do文本，也不造溢出用例。原件/JADX仅在精确绑定下复用。
7. **边界验证要击中证书。** 真实IR正例保留prefix owner/原赋值和最终break身份；至少一项真实可解析图变体打破唯一入边或准确终点，证明新证书拒绝，而非假定其它既有规则也必须拒绝。控制若仅parser/IR级，不冒称verifier有效或运行通过。实际测预算停止/预取消；公开API核不发布半个正文或map。

## Risks / Trade-offs

- [把前缀误作break目标，跳过写入或重复执行] → 保持最终出口与gateway区别，核全部物理owner、k=10返回0及来源。
- [只改一处纯桥检查，后续仍拒绝或scope越界] → 同一窄证书贯穿形状准入与body前置检查，保留parent scope和全边闭合。
- [从实例扩张到通用出口框架] → 单块普通正常流，拒绝多块/共享/非normal边；当前无新表示或机制依据。
- [把诊断成功算修复成功] → 1/0只是事实探针，16整类失败仍是基线，最终源码与自身CI未接受前不关闭实现任务。

## Migration Plan

先逐字归档原始基线和同轮IR/精确恢复收据，Luna私有最小实现与实图tests；root逐差异审查，再集成fixture/Reader真实库存/指纹和公开tests。受guard定向与完整三方对照通过后，同一最终产品执行一次fresh workspace和自身CI四jobs/独立官方ZIP/双seed及继承具名门，绑定Git/CLI/源码，最后主spec同步、strict、账本/handoff与clean-main。Cargo仍5GiB余量、target1GiB、默认栈、每秒guard/finally exactclean；期间无共享写/Gitmutation/network/manualJDK或CLI。root独占共享/Git/工具链；Luna只读共享、写私有tmp、Python-B。无已证损坏的通用桥或循环机制，不混入旧预算扫描等独立债务。
