# 下一窄片：CF07 非空 If arm 汇合来源

本片 For 来源产品冻结后只进行此有界架构审计，没有实施下一片。依据新 CLI CF07 独立接受 cf07-candidate-acceptance-root-v2.json，双 JDK/default-all 的 counted(II)I 仅缺 goto20→27；准确 raw javap、完整源码和行为已归档，不靠旧 CLI 推测失败。lastIndexOf 的 goto25→5 保留独立范围。

root 读核 build.rs 普通 Region::If→StmtKind::If 分支（17441起）：已分别构造两arm，仅对空arm的单指令Transfer补derived来源；join字段当前被模式的..忽略。已有Region::If保存准确join，Builder拥有canonical/ssa/operations与budget，Stmt/Emitter现有OriginSet足以承载完整If范围。循环gateway_origins归属完整LoopStmt，不能把if内部20混入该字段。已有Switch直arm tail到join的来源补充（17767起）可参考局部位置，但其any normal edge检查不宜直接复制为新的充分证明；下一片应显式要求全canonical outgoing唯一且为Normal到本If join，并拒绝非goto/goto_w、错误target/异常/多边。

参考本地JADX IfRegionMaker.findOutBlock（230-279）：先dom frontier交集，再union合适候选，最后path cross。当前Jarde已经恢复If和while，问题是隐藏transfer的物理来源，没有证据要求替换join算法或新增Region/IR/Frame/pass。局部处理在既有Builder If origin构造中复用OriginSet即可，不能据BCI覆盖缺口随意加来源。

下一片先用独立诊断确认实际Region::If的join=27、then tail Straight canonical17、terminal@20、两个arm唯一owner，以及全canonical边inventory。现有all-evidence报告只给顶层loop blocks(6/11/17/23/27)，不暴露嵌套If join，因此上述嵌套结构仍为未动态验证的实施前提。若实际尾部是Sequence，只核最后直接child；不反向穿过nested If/loop/return，不泛化全arm扫描。严格按现有budget charge/poll传播Stop，无新事务/回滚实体。

以后才立独立OpenSpec：唯一明确目标是完整If span派生@20、保留condition@14和外while@30与已有OriginSet；真实goto target/额外边/异常边/错误末尾/Stop反例，加双JDK原/JADX/Jarde完整类原raw与所有BCI独立验收。不得把静态审计当实现成功，不宣称lastIndexOf25或CF07整单元完成。Luna准备稿cf07-if-join-next-architecture-luna-v1.md中的具体runtime形状假设保持未验证标签。
