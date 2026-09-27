## ADDED Requirements

### Requirement: 共享 finally 中的拼接返回值在清理前求值

系统 SHALL 在三行共享异常处理与三份相同清理均已证明时，允许具名 catch 的保存返回值来自完整受证的 Java 8 字符串拼接，并输出可重编的 `try`/具名 `catch`/唯一 `finally`。拼接及其可能抛出的操作 SHALL 在该 catch 的保护范围内、在清理前且仅执行一次；正常返回、具名 catch 返回、清理异常覆盖与原异常重抛 MUST 与原 class 一致。系统 SHALL 为拼接、两份保存返回、三份清理及异常行保留物理来源。

#### Scenario: 固定拼接返回和三份字段清理
- **WHEN** 原 class 的正常路径保存字面量返回，具名 catch 路径先计算 `"caught:" + error.getMessage()` 再保存返回，两条路径及共用异常 handler 各有一份相同字段清理
- **THEN** 输出可重编的唯一 `finally`；正常路径与具名 catch 路径均只清理一次，返回值、计数及异常路径与原 class 一致

#### Scenario: 拼接生产者不完整或不唯一
- **WHEN** 返回值的生产者并非完整受证的字符串拼接、保存值被另一消费者使用、拼接的任一指令不在该具名 catch 的保护范围，或者求值被移到清理后
- **THEN** 系统 MUST 拒绝将该路径折叠为共享 `finally`，保留可定位的字节码拒绝证据

#### Scenario: 三行及异常优先级改变
- **WHEN** 异常行顺序、范围或目标变化，三份清理效果不同，或拼接抛错不再由原共用 handler 清理
- **THEN** 系统 MUST 拒绝输出这一共享 `finally`，不得吞掉、重复或提前执行清理

#### Scenario: 停止与输出原子性
- **WHEN** 证明、构建或输出过程中被取消或耗尽适用预算
- **THEN** 系统 MUST 返回停止状态，不得发布半个结构化方法或无来源的语句
