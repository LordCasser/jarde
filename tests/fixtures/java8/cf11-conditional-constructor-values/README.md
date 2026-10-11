# CF11 条件构造值

三个实际 javac23 original Java8 class，未编辑字节；SHA256/大小/原始来源见 pins.json。固定 SequentialLoops2 保留 upstream lowercases={'a'} 的原始行为，不修测试方法的数组越界。完整源码/JADX/Jarde双JDK重编运行在 recover-proved-constructor-conditional-values/results 的真实 capture；本目录只安装 canonical fixture，Runner 的19行控制 oracle 及固定10行 oracle 保存在完整证据中。

Positive 覆盖单臂/双臂构造、调用实参、静态/实例字段。Refusals 是边界 probe，并非整个类必须拒绝：localBoundary 可由 stack Phi→Store 恢复，independentBranchEffect 保持普通 structured if；shared output、nested Phi 与 handler 的实际拒绝单列。不借同源隔离结果冒称两个生成类 combined 成功。
