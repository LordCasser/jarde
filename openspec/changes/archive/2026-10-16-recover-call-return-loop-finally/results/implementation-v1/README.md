# 调用返回循环finally实现与边界

私有proof针对两真实编译器同一有限形态：60 BCIs/opcodes、7 canonical/SSA blocks、8条完整边、[17,30)->88及[88,90)->88两row、header7/latch59/blocks{7,12,59}、全部66 SSA values/10 phis/读写slot/type/producer/consumer、每条effect与handler集合。候选admission有固定上限，随后每项证明charge及末尾poll保留实际StopReason；它不假定受保护Static (I)I callee纯，也不读取callee体。cleanup仍限定实际System.out/StringBuilder/fin:/append/toString/println符号，未放宽既有cleanup_sequence或NoCall规则。

三份cleanup30/59/90各核自身fresh NewSite、dup双未初始化alias、constructor初始化写回与两次append/toString/println的内部SSA依赖；dup第一个未使用alias按实际uses=[]检查，不能机械要求每个输出单消费者。saved call25->store28->load56->return58逐值单consumer，handler x phi为[x,Itself]且明确replaced_by x，caught phi分别来自(block23,throw25)与(block88,throw88)，save88->load116->throw118保留主异常身份。c0@25 may_throw/handlers[0]，全部cleanup操作handlers[]，清理异常替代pending completion。

Plan复用Finally/SavedReturn：lead12..17在try外，body17..30，normal cleanup30..56，owned12/23/59/88，join/header7；fallthrough cleanup59..85/transfer85及融合Tail119..125。BCI12没有exception edge，需一个仅返回bool的固定admission hint在循环scope内触发原guard::examine；完整paid proof成功后才进入既有finally_body额外return授权。handler只在成功后owned认领，child scope/coverage/visited rollback及Builder原子提交保持。hint-false不触发新examine；hint-true但最终拒绝的候选近邻可支付正常proof费用，不声称所有fallback费用不变。

真实targeted native第三轮5命令全0：private4、公库2、CLI1，共7 passed；13个重新真实分析的class反例包含副本x、调用实参、返回reload、尾步、分支、保护边界、异常存取、额外dup/pop consumer、单副本新常量及实际StringBuffer构造/append符号变化。晚AnalysisSteps与charge内取消都在末尾126停止、无证书且不退款；实际elapsed=0在本proof第一个charge0保留Interrupted并无新增AnalysisSteps，不是伪造错误或入口预取消。公共库/CLI检查完整类3方法、两return/一个finally/无fallback与全部60 BCI真实方法来源。

第一轮编译成功但1pass/1fail：caught def错误要求block23/bci23；保留compiled-guard与raw。修正到真实throw25后第二轮private2及CLI构建通过；公共v2实验随后只执行首CLI0就发现entry缺口、无javac/重建运行。首公共v1 runner因旧工作树路径在source前置检查停止、0目标命令。未将这些历史失败并入成功计数。

中间第三轮CLI b4a6b2ce…c9c851下完整独立类另10命令全0：两份1706B完整text原样重编，未改driver rebuilt-only编译后四次Java8/23均等217B oracle；runtime实际用timeout30s/kill-after5s包裹。父screen另24命令全0、6完整类重编成功、12个main匹配原oracle。FX保留6方法，finReturn structured，finContinue/finBreak仍有handler45的fallback（注释45/47/50/52）；固定main匹配不证明这两方法的异常出口完整。纯c0的次数/时点与try异常路径未由driver直接观察，仍需上述IR证明。

最终补强断言后的native为9命令全0，连同前27命令完成fmt、CI同参数Clippy、reader/指纹、46项private finally、13个兼容target、原P5门、CLI三项、API examples、MSRV1.88及两版strict364/0，实际exact clean且target不存在。前27计110 passed/0 failed/8 ignored；补强的public2/CLI3与同名旧结果重叠，不累加成115。两收据4936行源码仅两test文件增加Java/Structured/empty fallback断言，全部production不变，冻结CLI均为60261992B/0555、SHA-256 b4a6b2ce2911165da5bb38f7a56a7d8c14d286936c97b4181e647cff7fc9c851。合并独审25a3a3ed186110af0cc60a42031dbdc4972aa34a8bcdcfc31ab7f61301a70546核对72份raw。

最终冻结CLI再次独立运行完整类10命令与父screen24命令，分别20/48份raw全0，源码及工具输入前后完全一致；两份1706B完整源码原样javac8/javac23重编、未改driver rebuilt-only编译、四次Java8/23运行217B逐字匹配。父6完整类重编0、12 main逐字匹配，FX finContinue/finBreak的handler45 fallback仍未关闭。最终合并独审48e1e2276c03c390990742dc73f7dd7841f9db0d504eb7bc6519e2289cf59e34通过12项检查，任务3.1已完成。

产品提交464543594b6fad00c67be016ffa5e21d18bc1591通过实际Git ls-tree与每行blob SHA1/mode将4936行最终native源码和冻结CLI绑定。该提交自身push CI38913819742已终态成功；根审953 checks/0 errors，独审2863 checks/0 errors，149 API observations/298 raw、4 jobs/52 steps、双seed各373 summaries/3670 passed/0 failed/98 ignored、160 primary每target-FQN唯一ok、6 JDK命令9项控制、单实际strict364均闭合。任务3.2完成，独审SHA-256 5f58b2413db431aa8475f46984e09fbd17592599b46f95369ff48e7ffc715a63；CLI/source/产品head绑定未变。最终便携包将保留首次proof失败、公共v1前置失败与v2 entry缺口，不能将这些历史失败写成重编或重建运行成功。
