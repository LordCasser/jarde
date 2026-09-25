## ADDED Requirements

### Requirement: 已证明的同步块双返回出口

当 Java 8 同步块的一个受保护条件分支有两条正常返回路径，且每条路径在返回前退出同一监视器、所有可达抛错路径由同一个正确退出监视器并重抛原异常的 handler 处理时，恢复层 SHALL 可把分支与各自的返回写在同一个 `synchronized` 块内。一个 `monitorexit` 数量或相似 opcode 片段不足以认领；证明不完整时 MUST 保守引用所有相关块及来源。

#### Scenario: Either arm returns its own value
- **WHEN** 条件分别进入两个求值与 `monitorexit; ireturn` 路径
- **THEN** 输出完整类 SHALL 在相同效果顺序下只求值被选臂一次，返回该臂的值，并在返回前释放原监视器

#### Scenario: Value producer throws inside one arm
- **WHEN** 被选臂的返回值生产者抛出异常
- **THEN** 输出完整类 SHALL 保留该生产者已发生的效果、只释放原监视器一次，并抛出相同异常对象

#### Scenario: Incomplete or mismatched protection
- **WHEN** 任一臂缺少退出、退出不同锁、保护区间没有覆盖可抛错指令、handler 不重抛原对象或存在额外可达出口
- **THEN** 本层 MUST 不输出已证明同步块，MUST 在拒绝来源中保留分支、各臂值生产者、所有监视器操作和 handler

#### Scenario: Sources and stopping
- **WHEN** 请求默认或完整来源，或证明/发射达到正文、来源预算或取消
- **THEN** 两种来源模式 SHALL 有相同正文，真实 BCI/成员来源涵盖进入、分支、两臂值/退出/返回、异常退出与重抛；停止遵循既有有界结果
