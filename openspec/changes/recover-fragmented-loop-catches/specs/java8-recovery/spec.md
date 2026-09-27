## ADDED Requirements

### Requirement: 循环内分段异常范围的嵌套具名 catch 准确恢复

当 Java 8 方法在同一循环内包含内层具名 catch、其 handler 的续边回到循环更新，以及由多条不相交异常表行共享的外层具名 catch 时，系统 SHALL 只在全部异常派发顺序、每条可抛指令的保护关系、handler 和续接的唯一所有权、局部值及物理来源闭合后输出结构化源码。系统 MUST 以原 class 的正常及异常路径效果和返回值为语义基准；证据不足时 MUST 保留可定位的整方法物理引用，不得把外层 catch 缩入某个条件分支或把内部 handler 当作普通循环入口。

#### Scenario: 固定完整 CF-18 类
- **WHEN** 输入为固定 `ExceptionRegionsAudit` Java 8 class，并恢复、重编和运行其完整源码
- **THEN** Jarde 完整类 SHALL 通过 `javac --release 8` 与 `java -Xverify:all`，输出与原 class 相同的 `124:115`，目标方法无 `@bytecode`，外层 `IllegalStateException` catch 包含负值分支后可能抛出异常的 `work(value)` 路径；固定 JADX 完整类的外抛/非零退出 MUST 作为对照记录而非期望值

#### Scenario: 缩小的同构入口
- **WHEN** 输入为固定 `HandlerLoopProbe` 的两级具名 catch 和分段外层 handler
- **THEN** 原/JADX/Jarde 完整类 SHALL 在 Java 8 重编、JVM 验证运行后都输出 `4:110`，且每条异常表行、两个 handler、循环更新及普通/异常续接只有一个来源归属

#### Scenario: 异常行和词法范围不能等价
- **WHEN** 共享 handler 的分段行在类型、派发优先级、可抛指令覆盖、异常进入边或同一词法 catch 的唯一性上不闭合
- **THEN** 系统 MUST 拒绝折叠，不得提前、遗漏或重复任一 handler 的效果，也不得伪造 Java `try` 保护范围

#### Scenario: 循环回接或局部值不能等价
- **WHEN** 某 handler 有额外普通入口、流向不同循环内部点或无法证明与普通路径在受证更新点唯一汇合，或 handler 参数/局部定义与后继读取不能闭合
- **THEN** 系统 MUST 保留安全引用，不得因放宽不可约诊断而发布不完整或错误的 try/loop

#### Scenario: 停止与既有异常形态
- **WHEN** 证明或输出遇到预算耗尽、取消，或输入是已验收的非分段 catch、finally、TWR 与真正交叉的异常范围
- **THEN** 停止 MUST 原子传播且不发布半份结构/来源；已验收结果及拒绝边界 SHALL 保持
