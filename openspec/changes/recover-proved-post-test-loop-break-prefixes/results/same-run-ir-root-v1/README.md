# 同轮 IR 与出口所有权定位

root实际执行真实Reader/IR探针，debug/no-debug均为23B Code、14指令、五canonical块0/2/8/13/21与六正常边。自然循环仅2/13，唯一latch13、ipdom21；8保留Push/Store/Goto→21，实际header ForHeader=None，首内部gap为ArmsDoNotMeet13。初值、循环增量和出口写入的SSA/Phi均保留。BCI/ValueId只说明本实例，不是生产匹配条件。

现有纯goto桥不能接纳赋值前缀；既有scope/gateway已能保留其语句，break目标应继续为最终出口21，不能改成前缀8。新片只证明单块独占所有权，不新增Region/CFG/pass，也不改LoopTarget元数据。

实际探针1 passed/0 failed/0 ignored，峰值target 167,409,253B，默认栈与guard/finally exactclean；临时测试移除、region源码精确恢复原SHA `19d21dd5f17f48018a5d427a62971b1e4754f1083df3a08cfcc91546705df371`、target absent。此结果是事实探针，不是生产修复验收。

归档807,880B，SHA `d841b5ba253bddafb72aebd97acf1bfddf895fe2a18bfcb6963fbf525206f5cc`，32 regular payload、八目录及inventory。root独立逐成员核bytes/SHA/mode及现场闭合。原/候选源码快照、raw流、停止与恢复收据、静态备忘、OpenSpec准备指令均保留；准备JSON不冒称执行结果。共享临时源已恢复，因此不要求那些临时外部pin仍在现场。见 `root-review.json`。
