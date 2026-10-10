# 非空 If arm 汇合来源：root 接续入口

当前2/7，四规划文档已生成，尚未实施产品/新CLI；内部Region诊断已实际通过并精确还原。精确基线引用上一For来源片results/cf07-candidate-acceptance-root-v2.json：root已实跑独立接受新CLI29命令、119 inventory members与10完整类腿，双JDK/default-all稳定，counted(II)I准确只缺goto20→27，lastIndexOf([IIII)I只缺goto25→5。不把该scoped接受当全BCI完成。

架构审计见前片results/cf07-if-join-next-architecture-root-v1.md与Luna草案。既有If.join、Builder的canonical/ssa/operations/budget、OriginSet和完整If emitter足够承载本片。Loop.gateway_origins不能复用为@20归属；不要照搬Switch的any Normal edge判据。公开RecoveryReport仅暴露扁平RegionRecord；外部MethodIr可核canonical/SSA，却不能读实际嵌套If树，因为region::recover为pub(crate)。不新增生产hook：前片确切CI接受后使用已授权的临时内部测试诊断，root实际运行并还原，核源码pins与无遗留修改。物理instruction@20与canonical源块17不是同一个身份。

下一步先完成前片d714a6bcc自身CI与干净主线，再做1.2诊断；未验证join/末尾/ownership前不派发生产实现。实际分支形状如果不满足限定边界则更新设计，不建立通用来源补洞或新Frame。机器5GiB余量/本仓target1GiB实时中止及完成清理按用户授权继续。71/612和CF07整单元计数不变。

内部诊断准备已root全文读审，但尚未应用或执行：results/internal-diagnostic-luna-v2.patch区分canonical branch leader11/physical branch14，并读取当前For冻结class；v1未执行稿保留。root wrapper改正输出归属本change，覆盖历史全lib摘要为精确单测试1/0/0，记录自身SHA及实际环境剥离；git apply --check实际成功仅证明补丁可应用，不能关闭1.2。准备差异与SHA见results/internal-diagnostic-preparation-root-v1.json。

根复核现有所有权边界：Region::blocks的If/Sequence展开保留重复，region::recover在完整树交给Builder前用overlapping_owner预算化拒绝并改为whole quote；Builder::arm在传播Stop前恢复stmts/declared。正式实现应利用这一既有不变量，不新增方法归属表或二次图分析。Atlas的重名blocks调用关系仅作导航，SSA::blocks与Region::blocks须以实际源码类型区分；动态join证明仍未运行。

15:59 UTC任务1.2已root实际接受：results/internal-diagnostic-acceptance-root-v1.json，真实test精确1/0/0；同一次JVM IR下outer header6/If canonical branch11与instruction14/join27/then Straight17两指令iinc17+goto20，全canonical outgoing唯一Normal17→27、then/else/method计数1/0/1。第一次test编译缺FieldCopies限定与4处Vec引用层级错误，真实失败raw/v1代码保持；root v3临时patch/v2runner修正后成功。还原region SHA79056610e41111c3f6b4e3c38d56923a3f1e05dbd83ee0fa259dbed0902f93a5、50pins恒同，临时test不遗留；cargo clean实际341files/144.8MiB，target/fuzz target均无。下一步可派发2.1 private最小实现，不用diagnostic通过代替新CLI/完整类/自身CI验收。
