# CF07 computed-init for 呈现边界

这是独立呈现债务，未在 return-arm latch 来源片中实现。root 阅读本地 JADX TestLoopCondition5.java 的活动 Java 测试：恰一个 `for (`、一个 `return -1;`、两条 return。已接受的新 CF07 实际 jadx-output/default/sources/cf07/LoopCases.java 也使用 for，而 Jarde 仍以先赋初值的 while/If else递减表达；完整源码运行一致不替代上游 for 文本断言。

Jarde region.rs::for_header_candidate 的 preheader 末尾门为 Push(ConstantValue::Int)+Store（约11155-11178），该冻结输入初值 end-1 的物理序列是0 iload3/1 iconst1/2 isub/3 istore4，故第一道此门不满足。root 临时实际 Region 诊断证实 for_header=None。不能把“body尾不是Straight”误当 ForHeader 不成立的根因；它只说明 implicit_tail_latch_origin 未保留goto25的来源。

参考 JADX LoopRegionVisitor.checkForIndexedLoop 的 loopEnd、SSA phi/init/增量与循环内使用判据。后续应先诊断 Jarde 同一次 SSA preheader expression、init读者、header phi及 Builder初始化消费，再判断能否复用既有表达式证明扩大初值门；此处没有证明通用初值移动安全，也不新增机制或放宽门。来源片验收全18物理BCI时，CF07整单元仍保留本债务与其他未验形态。

## 后续静态审计：第二道独立门

root 继续核 for_header_candidate 和调用次序，并复核 Luna v2 私有审计：初值 producer 门之外，还扫描全部 SSA，拒绝 natural loop/preheader 以外的 induction-slot 读写。冻结 lastIndexOf 的 return19 中 iload4 正在 natural blocks[5,11,22] 以外；当前 prove_for_header 先于 loop_terminal_returns，故仅放宽 computed initializer 不足以覆盖这条候选。

已有 terminal-return 证书可供后续调查提前复用，仅为已证 return leaf 的读提供例外；不得扩大 natural blocks、容许外部写或更改所有权。还需独立核 region.rs::implicit_tail_latch_origin 的 direct return/latch If 分支目前仅 for_header=None；Builder 的初始化/update 消费与该来源限制是不同层。完整静态笔记见 computed-for-architecture-luna-v2。当前片仍无这类生产修改；证书在准确 frame context 的结果、扩大初值后下一道拒绝及 for 正文/map须另做真实 IR 与完整类验收。
