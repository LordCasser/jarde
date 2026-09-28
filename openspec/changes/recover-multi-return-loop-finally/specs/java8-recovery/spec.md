## ADDED Requirements

### Requirement: 循环正文的双返回 finally 保持完成值与异常身份

系统 SHALL 仅在两个保存的返回值、正常可达正文循环、三份同一可观察清理调用、三行异常覆盖以及原 Throwable 的重抛关系均有完整物理证据时，将该布局恢复为可重编的 `try/finally`。输出 MUST 保持两处返回值的求值先于清理、循环内操作顺序、清理次数、异常覆盖和全部字节码来源。证据不足时 MUST 安全拒绝，不能按指令相似性删除清理副本。

#### Scenario: 前置无清理路径与提前返回

- **WHEN** 固定 Test5 同布局方法在进入受保护范围前因 `c == null` 返回，或 `first()` 为 false 并保存 `null` 返回
- **THEN** 恢复源码 SHALL 分别执行零次和一次 `close()`，返回 `null`，且清理仅包围原受保护的两条路径

#### Scenario: 正文循环与列表返回

- **WHEN** 正文循环执行一次或多次，`load` 与 `toNext` 正常完成
- **THEN** 恢复源码 SHALL 按原顺序添加元素、保存列表结果、执行一次 `close()` 再返回同一列表；原 class、固定 JADX 与恢复源码的可观察行为 SHALL 一致

#### Scenario: 正文或清理抛错

- **WHEN** `first`、`load` 或 `toNext` 抛错，或正常/异常完成期间的 `close()` 抛错
- **THEN** 恢复源码 MUST 在受保护的异常路径运行一次清理，保持原 Throwable 身份；清理自身抛错 MUST 覆盖待返回值或先前异常，且不能因 handler 的自保护行重复清理

#### Scenario: 不完整证明与停止

- **WHEN** 任一保存值/重载值关系、清理接收者或调用目标、循环入口/出口、异常表范围、自保护范围、重抛身份或物理块来源与证书不符，或分析预算耗尽/请求取消
- **THEN** 系统 MUST 不发布部分 `try/finally`，保留未证字节码和物理异常行的来源；停止 MUST 不交付半成品源码或来源映射
