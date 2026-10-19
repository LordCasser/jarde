## Why

固定 JADX 原始 TestContinueInLoop 的完整基线已确认差距：Jarde 八种配置均拒绝目标方法，16次完整源码编译失败；JADX 可编译结果的32次观察与原件一致。真实 IR 进一步确认 for 更新证明已成功，首次内部失败是共享块被两条分支重复进入，因此应补循环内的局部汇合证明，而不是再造循环恢复机制。

## What Changes

- 在既有 Region 条件/循环恢复中，证明一个分支直接 continue、其余分支先汇合再执行条件 break 的窄形态；保留原始执行顺序和一次更新。
- 用真实同轮图关闭局部汇合、所有边和物理块所有权，保留现有拒绝、停止和来源契约。
- 保存原始 debug/no-debug、class/plain JAR 对照与反例，按有限用例验证数组、副作用字段、异常和输出。

阶段前提：完整基线已归档；真实 IR 已运行并恢复临时源码，证据归档完成后才勾选。Luna 仅私有实现，root 审查和集成；上一布尔片产品的自身CI源码采集关闭前不集成本片生产代码、fixture或指纹。

非目标：不放松所有 arms-do-not-meet/循环覆盖门，不新增全局后支配/类型求解器、pass、CFG或Region种类，不复制共享尾块，不处理任意深度分支/异常清理/多重更新或推进整个CF09计数。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加受证循环分支局部汇合及直接 continue 绕过后续 break 的行为契约。

## Impact

主要涉及 jarde-java 私有 Region 局部汇合和转移证明，以及真实 IR/公开整类回归、fixture、Reader实际库存和指纹。复用现有 Runtime、预算和标准库，无新依赖、公共协议或宿主耦合。许可保留，原件/JADX证据精确复用，不自动下载目标依赖或调用外层JUnit。
