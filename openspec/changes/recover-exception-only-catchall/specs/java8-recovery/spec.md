## ADDED Requirements

### Requirement: Exception-only catch-all recovery

系统 SHALL 仅在单一 catch-all 异常行完整保护仅以抛出结束的正文、handler 对原异常执行可呈现的清理并重抛同一对象、且所有入口出口及异常优先级均可证明时，把它恢复成行为等价的 Java `try`/`catch (Throwable)`。没有上述闭合证据时 MUST 保守拒绝，不得把清理误写为可能漏执行或重复执行的 `finally`/catch。

#### Scenario: 固定仅异常完成的清理
- **WHEN** 原 class 的受保护体只能抛出 `IllegalStateException`，单一 catch-all handler 增加计数并重抛同一 Throwable
- **THEN** 输出一个可重编的 `catch (Throwable)`，完整类 `java -Xverify:all` 的异常类型、消息、计数与原 class 一致，所有物理指令及异常行保有来源

#### Scenario: 出现正常完成路径
- **WHEN** 相同外形的受保护体另有 `return`、可达正常出口或跳过 handler 的路径
- **THEN** 系统 MUST 拒绝 exception-only 证书，不能输出仅在异常时执行的清理来代替正常路径的清理

#### Scenario: 处理器及范围不闭合
- **WHEN** catch-all 行未覆盖应受保护的抛出、扩围到自己的清理、与其它异常行竞争，或有正常边从外部进入 handler
- **THEN** 系统 MUST 拒绝该证书，并保留可定位的字节码/异常行证据

#### Scenario: 原异常未被原值重抛
- **WHEN** handler 分支到不同清理、抛出新异常，或重抛值不能证明是入口捕获对象
- **THEN** 系统 MUST 拒绝将其作为该单一 `catch (Throwable)` 形态发布

#### Scenario: 停止与预算
- **WHEN** 构建、区域遍历或输出期间被取消或耗尽适用预算
- **THEN** 系统 MUST 返回停止状态，不得发布半个 catch、丢失清理效果或伪造来源
