## Why

本地fresh native在既有bulk生命周期门出现真实失败：ClassGate已同时容纳两个任务，但BulkWindow.concurrent_classes_high_water为1。run_class_task在gate/delay之后才进入既有执行计数区间，故测试观察区间和公开类任务计数区间不一致；这个独立问题阻塞typed foreach的完整验收。

## What Changes

- 把已有RAII执行计数guard放到run_class_task入口，使准备前等待、正常结束、早返回和panic都处在同一类任务执行区间。
- 保留旧失败、复用现有重叠/worker退出/取消/消费方门，保持所有阈值；单独提交此修复后重新开始两fresh native。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `analysis-contracts`: 明确bulk类任务执行计数覆盖任务整个调用区间，正常或异常离开都释放。

## Impact

生产变更只涉及src/bulk.rs现有Entered guard的位置，不新增队列、锁、计数器、线程、依赖或公有模型。已有test-support gate用于观测真实重叠，不改其算法/超时或降低high_water==2断言。stage前提为已有bulk worker/serial共用run_class_task。语法恢复算法和71/612账本计数不变；typed foreach规格保持独立，须重新绑定最终CLI并完成其余门。
