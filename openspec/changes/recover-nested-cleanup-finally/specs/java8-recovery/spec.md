## ADDED Requirements

### Requirement: 受证 finally 内层 IOException catch 的重复清理恢复

系统 SHALL 仅在异常表、两份清理、接收者/调用目标、handler 完成路径和全部物理指令归属均已证明时，把重复的清理副本恢复为唯一 `finally { try { ... } catch (IOException ...) { ... } }`。系统 MUST 保持清理受检异常被吞掉、未检异常覆盖先前异常的行为；证明不足时 MUST 保留安全拒绝，不得发布一半结构。

#### Scenario: 固定四行目标正常完成

- **WHEN** Test4 同 BCI/opcode/四行异常表的 Java 8 方法正常执行 `write(1)` 与清理
- **THEN** 原始、固定 JADX 和 Jarde 完整源码 SHALL 重编并通过 JVM 验证运行，Jarde 源码只写一份 finally 内层 `try/catch`，正常行为一致且全部物理 BCI 可追溯

#### Scenario: 正文或清理异常

- **WHEN** 同源级嵌套结构的可注入 control 遇到正文 `IOException`、清理 `IOException` 或清理 `RuntimeException`
- **THEN** 三方 SHALL 保持相同事件顺序和终局异常：正文异常在清理成功或受检异常被吞掉后继续传播，清理未检异常覆盖先前异常；control MUST 明示其字节码不同，不得据此宣称固定目标的异常路径已动态触发

#### Scenario: 异常表或副本不等价

- **WHEN** 任一 named/catch-all 行的范围或目标改变、清理调用目标/接收者/次序/次数改变、catch 参数被读取、原 Throwable 被替换，或有额外 CFG 边/未归属 BCI
- **THEN** 系统 MUST 拒绝唯一 finally 投影，并保留原物理事实和拒绝位置

#### Scenario: 预算与取消

- **WHEN** 证明、Region 或 Build 在预算耗尽、取消或不可呈现的 catch/局部上停止
- **THEN** 系统 MUST 原子回退，不得输出半份外层 finally、半份内层 catch 或丢失物理来源
