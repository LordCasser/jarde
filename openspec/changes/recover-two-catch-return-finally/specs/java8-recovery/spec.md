## ADDED Requirements

### Requirement: 双具名 catch 与提前返回的四份清理恢复为唯一 finally

系统 SHALL 在 Java 8 方法的四条真实异常行、两个具名 catch、一个 catch 内保存后提前返回、四份相同调用型清理、所有正常与异常完成及来源均被证明时，输出可重编的唯一 `try/catch/finally`。正常及第一 catch SHALL 返回 `0`，第二 catch SHALL 返回 `1`，非具名异常 SHALL 在清理成功后原值重抛。清理自身抛错 SHALL 只执行该路径一次并覆盖先前返回或异常。证明不完整时 MUST 保守拒绝，不得删去或重复物理副作用。

#### Scenario: 三种正常完成

- **WHEN** 正文正常完成，抛出第一个具名异常，或抛出第二个具名异常
- **THEN** 清理各执行一次，结果依次为 `0`、`0`、`1`，源码仅含一个 finally 和两个具名 catch

#### Scenario: 未捕获异常与清理异常

- **WHEN** 正文抛出不匹配两个具名 catch 的异常，或任一路径的清理调用抛出异常
- **THEN** 清理成功时重抛原 Throwable；清理抛错时新异常覆盖此前结果，且不被同一个方法再次清理

#### Scenario: 四份副本或返回/异常身份不明

- **WHEN** 副本调用目标、参数或范围不一致，提前返回的保存值与加载值不一致，原 Throwable 被改写，异常行顺序/范围改变，或存在外部入口及额外出口
- **THEN** 系统 MUST 拒绝唯一 finally 投影，保留可定位的字节码/异常行证据

#### Scenario: 停止与来源

- **WHEN** 证明、区域构造或源码输出被取消或耗尽预算
- **THEN** 系统 MUST 原子停止，不得发布半个结构；成功时四份清理及全部目标 BCI SHALL 在 source map 中可查
