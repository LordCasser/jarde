## Why

循环continue局部汇合的实际root验收已让Region正确表达原始TestContinueInLoop，但公开whole-class测试仍Fallback：数组自减的dup2/iaload/isub/iastore未恢复。实际Java lib510/0，公开测试1/1，完整方法正文与失败留在 `/private/tmp/jarde-continue-ladder-focused-root-v2-20261019/raw/2.stdout.raw`。根因是现有Builder prove_array_update只准0x60/iadd，Array计划和发射固定+=；这与Region归属不同，拆为明确依赖，不通过放宽循环证明掩盖。

## What Changes

复用受证int[]复合赋值的精确数组/下标身份、dup2四输出唯一消费、old在左、RHS闭合与求值顺序，为非返回值语句增加精确0x64/isub；私有Array计划携带现有ArithmeticOp并发射现有AssignOp::Subtract。既有加法及返回值+=路径保持原门。没有新IR/pass/CFG/Region或依赖。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证int数组减法复合写恢复，未知/额外用途保留拒绝与真实停止。

## Impact

只改Builder现有CompoundAssignments/Array计划与发射，加最小真实IR测试。复用continue父片已冻结原件fixture，不新增class/JAR或Reader增量。最终完整八case/八API/JDK8和23/隔离VM/native/当前产品自身CI由两片共享同一最终产品与CLI收据，不重复执行冒充覆盖。父片未完成任务在本依赖实际接受后才能关闭。
