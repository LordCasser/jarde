# 下一片架构审计：二维int数组复合更新

当前五源CLI已fresh验证P02_multianewarray两腿：原程序6\n；候选完整类compile1，lambda helper fallback。不是capture故障，也不是旧README描述的silent打印0。新baseline保留原class/真实JDK/完整生成类与失败流，尚无本片freshJADX对照。

物理helper lambda$sum$0([[ILjava/lang/Integer;)V为aload0/0/aaload2/0/dup2@4/iaload5/aload1/intValue7/iadd10/iastore11/return12。frame.rs:676把aaload输出定为普通Ref，未声明具体行数组类型；decode.rs:423将其表示为ArrayElementLoad，只有0x59映为Duplicate，dup2保留Other。array_of_value已有从精确[[I原数组递归降一rank的读取；collect_expression_bcis已允许ArrayElementLoad。AST已有IndexAssign，不需新nested-index实体。

prove_array_update开头先在store的dup2产物array_store上要求array_of_value==int[]，这依赖copy的frame保留完整数组名。直接int[]局部复制可成立；aaload得到行的Unknown引用经过Other dup2，现有array_of_value无法追过copy。后续现有证明已经查准确dup2 opcode/四个各异ValueId及read/store对应、输入原array精确int[]、唯一uses、read/add/store次序、左值完整prefix和rhs bounded interval。应先核实这条前置type gate是否冗余；候选方案是在四副本准确源身份闭合后，用原row的现成类型事实证明array target，保持所有原owner/effect/budget条件。不能为猜测Object补全局类型推断，也不能泛化Duplicate去接纳所有dup2。

此为源代码支持的候选定位，尚未实施、未声称移除一个检查就已经成功。下一spec应固定两完整原class、历史拒绝、当前JADX源码/test路径和fresh对照；额外控制要验证旧int[]更新不退化、读写不同row/index不误折叠、row/index/RHS各求值一次、null/越界在RHS前失败及RHS改写外层row时仍写原行对象。停止/取消与来源用现有compound proof/Builder通路检查，不凭helper row predicate替代完整生产与语义证据。

实现不得改lambda/SAM gate、数组初始化、所有dup opcode或平台type rows。旧capture change的multianewarray快照断言需精准升级为“capture/sum等原区域保持、helper改为structured”，历史baseline/fixed文件保留不重写。先写独立OpenSpec、由Luna实施、root双JDK全类对照与完整24腿/确切CI验收。BigDecimal当前产品CI37989319644仍在运行，当前五源暂冻结。
