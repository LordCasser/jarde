# Nested int array compound controls

完整 NestedIntUpdates.java 与固定外部 Runner.java 由 root 使用真实 Corretto8/OpenJDK23，source/target8、g:none、空 CP/SP 共同编译。v8/v23 仅复制原目标完整class；Runner 用于原始与恢复目标的同一行为harness，不称作反编译产物，也不借原class重编candidate。source、工具、原Code/descriptor和10条oracle原双流见 OpenSpec results/controls-v1/manifest.json；canonical复制身份见 canonical-inputs-v1.json。

覆盖二维/三维/一维更新，row/index/RHS调用一次及异常次序，RHS替换行仍写原行，以及不同读取/写入行的普通赋值。全部源和成员参与每次candidate编译；不得剥离失败成员。原始oracle每腿exit0、stderr空，stdout保存在对应 javac*.original.stdout。
