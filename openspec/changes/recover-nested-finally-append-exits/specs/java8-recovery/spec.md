## ADDED Requirements

### Requirement: 嵌套 catch 与追加清理的 finally 恢复

系统 SHALL 对 Java 8 方法中已证明的嵌套具名 catch 和外层异常清理，以及具名 catch 与共用异常清理，恢复执行一次追加效果的 `try`/`catch`/`finally`。系统 MUST 保持正常、具名 catch 和外抛路径上被调用成员、参数、效果次数与异常完成顺序。

#### Scenario: 内层 catch 后执行普通语句再清理
- **WHEN** 内层 catch 的正常路径与 try 正常路径汇合、执行受外层异常范围保护的普通语句，然后两者执行同一追加清理；外层异常 handler 执行等价清理后重抛原异常
- **THEN** 系统输出内层 `try`/`catch`、汇合后的普通语句与外层唯一 `finally`，其 Java 8 完整类在正常、具名 catch 和外抛路径与原 class 的副作用顺序一致

#### Scenario: 内层 catch 后直接清理
- **WHEN** 内层 catch 两条正常路径直接汇入同一清理，异常 handler 执行等价清理并重抛原异常
- **THEN** 系统输出一次外层 `finally`，并保持具名 catch 与异常路径的清理次数和顺序

#### Scenario: try 和 catch 各有清理副本
- **WHEN** try 正常完成、具名 catch 正常完成各执行一份相同追加清理并汇入同一后续块，共用异常 handler 执行第三份后重抛原异常
- **THEN** 系统输出一个 `try`/`catch`/`finally` 和独立后续块，三条路径的清理各执行一次

### Requirement: 不等价副本与未闭合区域的原子拒绝

系统 MUST 在任一副本的接收者、字段、调用目标、常量参数、结果消费、异常表覆盖、正常出口或重抛值无法证明一致时，拒绝归并 `finally`，不得丢失物理行为或发布部分结构化正文。

#### Scenario: 改变任一追加副本
- **WHEN** 任一正常或异常清理副本写入不同目标、传入不同字符串，或者其调用结果另有可观察用途
- **THEN** 系统保留带物理位置的拒绝诊断，不输出声称三条路径共享的一份 `finally`

#### Scenario: 清理重新进入保护范围
- **WHEN** 异常范围扩到正常清理或 handler 清理，或者有额外入口/出口使清理次数不再为每条完成路径一次
- **THEN** 系统 MUST 拒绝归并，不把可能重复执行的清理改写为一个 `finally`

#### Scenario: 预算或取消中止
- **WHEN** 证明、区域恢复或源码构建耗尽预算，或者任务取消
- **THEN** 系统 MUST 停止且不发布半个 `finally`、不完整来源图或重复拥有的物理块

### Requirement: 来源完整性

系统 SHALL 对恢复的语句保留每个被消费的物理指令和异常行的可追溯来源，并确保每个物理控制流块只有一个结构所有者。

#### Scenario: 多份物理清理折叠为一份源码
- **WHEN** 两份或三份受证追加清理折叠为一个 `finally` 正文
- **THEN** 源码来源包含全部副本、正常跳转、handler 存储和重抛的物理位置，且后续块没有被提前吞入 `finally`
