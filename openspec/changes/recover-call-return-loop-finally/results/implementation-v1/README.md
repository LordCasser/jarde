# 调用返回循环finally实现与边界

私有proof针对两真实编译器同一有限形态：60 BCIs/opcodes、7 canonical/SSA blocks、8条完整边、[17,30)->88及[88,90)->88两row、header7/latch59/blocks{7,12,59}、全部66 SSA values/10 phis/读写slot/type/producer/consumer、每条effect与handler集合。候选admission有固定上限，随后每项证明charge及末尾poll保留实际StopReason；它不假定受保护Static (I)I callee纯，也不读取callee体。cleanup仍限定实际System.out/StringBuilder/fin:/append/toString/println符号，未放宽既有cleanup_sequence或NoCall规则。

三份cleanup30/59/90各核自身fresh NewSite、dup双未初始化alias、constructor初始化写回与两次append/toString/println的内部SSA依赖；dup第一个未使用alias按实际uses=[]检查，不能机械要求每个输出单消费者。saved call25->store28->load56->return58逐值单consumer，handler x phi为[x,Itself]且明确replaced_by x，caught phi分别来自(block23,throw25)与(block88,throw88)，save88->load116->throw118保留主异常身份。c0@25 may_throw/handlers[0]，全部cleanup操作handlers[]，清理异常替代pending completion。

Plan复用Finally/SavedReturn：lead12..17在try外，body17..30，normal cleanup30..56，owned12/23/59/88，join/header7；fallthrough cleanup59..85/transfer85及融合Tail119..125。BCI12没有exception edge，需一个仅返回bool的固定admission hint在循环scope内触发原guard::examine；完整paid proof成功后才进入既有finally_body额外return授权。handler只在成功后owned认领，child scope/coverage/visited rollback及Builder原子提交保持。hint-false不触发新examine；hint-true但最终拒绝的候选近邻可支付正常proof费用，不声称所有fallback费用不变。

真实targeted native第三轮5命令全0：private4、公库2、CLI1，共7 passed；13个重新真实分析的class反例包含副本x、调用实参、返回reload、尾步、分支、保护边界、异常存取、额外dup/pop consumer、单副本新常量及实际StringBuffer构造/append符号变化。晚AnalysisSteps与charge内取消都在末尾126停止、无证书且不退款；实际elapsed=0在本proof第一个charge0保留Interrupted并无新增AnalysisSteps，不是伪造错误或入口预取消。公共库/CLI检查完整类3方法、两return/一个finally/无fallback与全部60 BCI真实方法来源。

第一轮编译成功但1pass/1fail：caught def错误要求block23/bci23；保留compiled-guard与raw。修正到真实throw25后第二轮private2及CLI构建通过；公共v2实验随后只执行首CLI0就发现entry缺口、无javac/重建运行。首公共v1 runner因旧工作树路径在source前置检查停止、0目标命令。未将这些历史失败并入成功计数。

中间第三轮CLI b4a6b2ce…c9c851下完整独立类另10命令全0：两份1706B完整text原样重编，未改driver rebuilt-only编译后四次Java8/23均等217B oracle；runtime实际用timeout30s/kill-after5s包裹。父screen另24命令全0、6完整类重编成功、12个main匹配原oracle。FX保留6方法，finReturn structured，finContinue/finBreak仍有handler45的fallback（注释45/47/50/52）；固定main匹配不证明这两方法的异常出口完整。纯c0的次数/时点与try异常路径未由driver直接观察，仍需上述IR证明。

本组独审与全部原始流保留在持久cache CORE/call-return-finally-{first,second,third}-proof-native-v1、call-return-finally-intermediate-full-source-v{1,2,3}和effectful-finally-after-call-return-screen-v1，最终便携包在最终CLI/产品自身CI接受时封存。这些是中间源码恢复验证，不冒称最终exact-head自身CI；集成门仍执行中，任务3.1/3.2保持未勾。
