## ADDED Requirements

### Requirement: 共用正常汇合点的共享 finally 清理
系统 SHALL 在 Java 8 class 的一个具名 catch 和共用 catch-all 完整证明三份相同的当前实例布尔字段赋值，并证明 try/catch 的正常完成均在清理后进入同一个后续块时，输出唯一的 `finally` 赋值并继续恢复后续块。系统 MUST 保持原 class 在正常完成、具名 catch 和异常重抛路径上的清理效果、异常顺序和可追溯物理来源。

#### Scenario: 三份清理相同且两条正常路径汇合
- **WHEN** try 正常完成、具名 catch 正常完成和共用 catch-all 各执行一次相同的 `this.f = true`，前两条路径各经唯一 goto 汇入同一个 `return this.f`，异常 handler 保留原 throwable 重抛
- **THEN** 系统输出一个 `try`/`catch`/`finally` 和其后的 `return this.f`；完整类源码可按 Java 8 重编、通过验证，并在正常和具名 catch 路径与原 class 的返回值及字段值一致

#### Scenario: 任一清理副本的接收者、字段或值不同
- **WHEN** 三份副本中任一份改变 receiver、字段身份或写入布尔值，或清理值有额外消费者
- **THEN** 系统 MUST 拒绝把它们合并为一个 `finally`，保留带物理 BCI 的拒绝诊断，不发布部分结构化语句

#### Scenario: 异常范围或正常续接不闭合
- **WHEN** 具名或 catch-all 行的保护范围、优先顺序、handler、清理后的正常跳转或共用汇合点不能完整证明
- **THEN** 系统 MUST 保守拒绝整个共享 finally 候选，不能丢失异常路径或把后续返回吞进 `finally`

#### Scenario: 资源限制与取消
- **WHEN** 候选证明或源码构建耗尽预算，或者任务被取消
- **THEN** 系统 MUST 停止且不发布部分 `try`/`catch`/`finally` 源码或不完整来源图
