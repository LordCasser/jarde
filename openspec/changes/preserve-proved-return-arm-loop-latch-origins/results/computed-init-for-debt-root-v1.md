# CF07 computed-init for 呈现边界

这是独立呈现债务，未在 return-arm latch 来源片中实现。root 阅读本地 JADX TestLoopCondition5.java 的活动 Java 测试：恰一个 `for (`、一个 `return -1;`、两条 return。已接受的新 CF07 实际 jadx-output/default/sources/cf07/LoopCases.java 也使用 for，而 Jarde 仍以先赋初值的 while/If else递减表达；完整源码运行一致不替代上游 for 文本断言。

Jarde region.rs::for_header_candidate 的 preheader 末尾门为 Push(ConstantValue::Int)+Store（约11155-11178），该冻结输入初值 end-1 的物理序列是0 iload3/1 iconst1/2 isub/3 istore4，故第一道此门不满足。root 临时实际 Region 诊断证实 for_header=None。不能把“body尾不是Straight”误当 ForHeader 不成立的根因；它只说明 implicit_tail_latch_origin 未保留goto25的来源。

参考 JADX LoopRegionVisitor.checkForIndexedLoop 的 loopEnd、SSA phi/init/增量与循环内使用判据。后续应先诊断 Jarde 同一次 SSA preheader expression、init读者、header phi及 Builder初始化消费，再判断能否复用既有表达式证明扩大初值门；此处没有证明通用初值移动安全，也不新增机制或放宽门。来源片验收全18物理BCI时，CF07整单元仍保留本债务与其他未验形态。
