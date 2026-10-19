## Context

动机见 proposal；已接受原始基线在 openspec/evidence/java-syntax-2026-10-19/cf09-testcontinueinloop-original-root-v1。原件67B Code、44指令，两方法和public f:I。46实际命令保留全部失败；原件和JADX各32有限观察相同，Jarde八API整方法fallback/16完整编译失败，无Jarde目标运行。

root真实同轮IR探针1/0通过：debug/no-debug均为可约自然循环header2，成员[2,8,19,29,42,35,50]、唯一latch50、counter slot3/header Phi V6=[V22,V34]。实际 header_tested_loop 的 ForHeader 已成功(init1/update50)，不是归纳变量识别缺口。branch8/29全局ipdom56在循环外；branch42全局ipdom也是56，既有exit-bridge局部规则正确改选50。首次内部gap为重复进入42，然后发布branch8 ArmsDoNotMeet、header2 LoopShape。临时测试和两个test-only打印已逐字恢复，target absent，峰值170241062B。现有编译JRE_JOIN_PROBE与完整基线除usage外全文JSON一致，只有额外stderr。所有数字仅说明实例，生产不得匹配固定地址/类名/slot。

## Goals / Non-Goals

**Goals:** 让两个普通分支在自己的共享后续块汇合；另一个已证continue直接进入同一for更新，跳过该共享后续中的条件break；全物理块各一次、更新/字段副作用顺序不变。

**Non-Goals:** 本片只扩展已证for更新和单层else-if的普通正常流；不泛化深层/多循环出口/任意SSA、异常/cleanup、jsr clone、多个更新；不新增pass、全局图求解器、CFG/Region种类或第二套循环恢复，不复制共享块，不复用switch_continue等不同语义字段。

## Decisions

1. **复用现有局部证明。** 优先扩展 ladder_join/有界arm routes、现有LoopContinue及Frame循环目标。缺的是相对本循环完成边界的局部join证明；全局postdom不能独自表示一个分支abrupt continue的if。只有现有候选不存在或落在本循环/当前局部边界之外时尝试新窄准入，保留已有更具体的bridge/nested-side-exit/fragmented规则优先级。禁止直接删ArmsDoNotMeet、重入、覆盖检查；也不因路径最终可达latch就把它当continue。
2. **先证明再占有。** 在walk任一arm前，依据同轮canonical normal edges、真实ForHeader更新、scope和domination，找唯一首个共享正常后续块；两个正常分支必须到该点，允许绕过它的路径只能是该loop已证continue。逐边关闭入口/出口、无共享前缀交叉、无新循环/clone/未知非normal边，限制从分支至候选最多一层内部分支。候选不等于header/update/出口，所有进入候选的前驱属于本分支支配的前向区域；未证路径保留旧拒绝。复用已有路由结构；只有现有返回值无法携带必要证据时才增最小私有记录，不建立跨候选索引。
3. **保留continue的完成语义。** 新局部join下，直接进入已证update的那条条件边/受证纯transfer必须成为该loop的LoopContinue，不能变成空arm，也不能先走共享break块。合法呈现可以重排为else中的提前continue后decrement；只能改变等价结构，不能重排实际读数组/赋值/条件求值。正常两arm共享后续一次，条件break仍走既有规则；update50及其backedge由原ForHeader拥有一次，出口后的f++保持一次。具体选点与显式continue必须来自同一个闭合证明，不能靠遍历先后或BCI相邻猜测。
4. **保持事务与预算。** 所有新候选/节点/边/所有权扫描逐项charge并poll；复用现有Stop和visited/checkpoint，不退款、不把停止当缺证。候选失败不留下visited/未关闭tail等状态污染，不发布半个if/循环或来源；最终全覆盖/唯一物理owner仍为权威。当前旧ladder边界/terminal-return等独立债务不混入本片。
5. **JADX参考边界。** 参考其IfRegionMaker选择局部out、LoopRegionMaker对额外exit与end前驱标break/continue，再由LoopRegionVisitor认定for更新的分工。Jarde已有这些Region叶和ForHeader，无需复制标注/改图机制；只采用“正常汇合与abrupt完成分开证明”的算法思想。原始fixture保留JADX LICENSE/NOTICE，未复制生产算法代码；Rust标准库、现有Runtime足够，无新依赖。
6. **验收分层。** Reader物理读取、Java8版本校验、Runtime选择、完整JVM校验、源码恢复与所选运行等价分别报告。原始全outer/child不裁剪；class/plainJAR×debug/no-debug×default/all八API，目标structured且可重编；最终JDK8/23完整源码运行原八有限用例，比较数组/f/异常类别/stdout，保留空cwd/parentless/CodeSource/错根。源/CLI精确绑定时复用原件和JADX，不重跑。逐列所有来源缺口，不能用Evidence Complete冒称全BCI。反例应验证完整边闭合而非镜像实现，至少额外入口/未知旁路/绕过update/clone或异常边中的适用实图变体；再加实际证明预算停止和公共预取消。
7. **集成与依赖。** Luna只读共享和写私有tmp；root独占共享/Git/工具链，上一产品cc1bf自身CI采集闭合后才集成新code/fixtures/fingerprint。Cargo继续5GiB余量、target1GiB、默认栈、每秒guard/finally exactclean；期间不共享写、Gitmutation、network、manualJDK/CLI。许可和真实失败全部保留，不下载依赖，不调用外层JUnit。

## Risks / Trade-offs

- [把continue路径错误汇入条件break，或漏掉break前后的update] → hit equality后下一元素再break的原八case与实际数组/f共同判定；不只看返回/编译。
- [共享块重复拥有或候选试探污染后续] → 提前证明localjoin与边，反例变更真实图，保留所有权与覆盖门及真实Stop。
- [为了本例扩张成通用完成分析] → 限制一层普通for/scope；未知场景留给后续固定原件，不增全局solver。
- [Atlas陈旧导致读错代码] → scoped发现后以当前文件rg定位和SHA为准；不运行全库index。

## Migration Plan

先归档基线和真实IR，再由Luna私有最小实现/真实图tests与公开fixtures并行准备。root逐差异审查后集成、定向构建、完整三方对照、最终fresh工作区及产品自身CI双seed/具名门、源码Git/CLI绑定，最后主spec同步/strict/账本/handoff与clean-main。原件、失败和历史归档不覆写。
