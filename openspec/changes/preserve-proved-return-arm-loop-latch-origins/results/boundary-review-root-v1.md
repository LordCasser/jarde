# root真实边界与证明停止复核

三个Java源码原样javac23/target8编译（7命令），完整class字节/源/javap/旧CLI raw保留。新的同次IR诊断实际1/0/0，临时region测试执行后还原全部52产品/测试/class pins。私有patch的hunk计数错误导致root v1 apply-check失败（未执行Rust），root v2按已审加入代码重建准确diff；两次失败/修正原件保留。

- ReturnLatchEdges：canonical return block9以及latch block11各含真实Exception→20；自然单latch11。实际Region为Try→Loop→If6(joinNone)，then空、else Try，gateway为空；uncovered[17,9]。不是单block双Straight候选，不能说已通过异常末尾proof门或已完整恢复该Try。
- NestedLatch：实际外环header0/latch22、内环header11/latch16，Region外环LoopShape fallback，uncovered[28,22]。不会递归搜取内层回边；不虚称进入新tail helper。
- MultipleLatch：真实header0自然latches9和19；末尾If6另侧嵌套If14，返回17/更新19，gateway为空。fresh真实Walker调用现有helper实际Ok(None)、analysis_steps2；拒绝在单Straight arm形状检查处，不能说命中了自然唯一latch检查。

永久同次CF07真实IR测试 actual1/0/0：fresh proof budgets0在16停止、2在25的latch计费停止、3在25完整canonical-edge计费停止；IR完成后取消在16停止；充分预算返回Some25，真实恢复Loop的gateway为[25]。另外公开网关测试late-budget及cancel均无部分text/map。root未伪造Region/CFG，也未对mutated class执行运行。

错误target25→28消去循环的旧源码映射缺口单列；iinc代替末尾goto、return改athrow的真实物理字节负例实际passed，全部15gateway通过。尚无同时满足直接返回/latch两arm的不可归约输入，不把一般irreducible fixture计作本片专属门禁；现有证明的is_irreducible检查仍保留。
