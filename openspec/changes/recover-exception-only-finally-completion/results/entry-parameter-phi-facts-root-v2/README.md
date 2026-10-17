# 冗余入口参数phi的实际首拒

完整367B新原Java类的canonical/SSA/operations实际probe证明：两Complete块、一catch-all、body invoke2/throw6、handler invoke10/rethrow14；Stack0 caught phi之外有Local0 phi，inputs[Value0,Value0]、uses空且replaced_by Value0，Value0正是Entry block0 Local0。原proof的真实首拒是instrumented20131的phis.len()!=1，不是cleanup或异常身份。

采用既有resolve/same，只放行上述有界、无活跃用途、所有输入相同入口槽值的已消去phi；不加solver，非平凡合流拒绝。新独立参数重赋值class作Complete反例另记输入/最终Rust验收。v1诊断因Operations无Debug编译失败保留；v2改打印逐操作事实后实际1probe通过。临时guard已恢复aabb4cb原SHA，source restore receipt在archive，默认栈/守卫/exact clean通过。此probe返回None，不冒称正向恢复已通过。
