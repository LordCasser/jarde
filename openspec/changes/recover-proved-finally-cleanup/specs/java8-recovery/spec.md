## ADDED Requirements

### Requirement: 已证明的非覆盖型 finally 清理

当 Java 8 方法的保护区间在正常返回与异常完成时均执行同一可证明等价的清理，且清理正文自身没有显式替代返回或抛错时，恢复层 SHALL 可生成 `try/finally` Java 结构。输出 MUST 保持每条可达路径上清理恰好一次、try 返回值的保存、原异常对象的重抛以及清理执行期间的异常优先级。仅有 catch-all 异常表项或相似指令序列不足以声称此结构；证明不足时 MUST 保守引用所有受影响指令和物理来源。

#### Scenario: Normal return keeps its saved value
- **WHEN** try 的返回值先求出，随后执行已证明的清理且清理正常完成
- **THEN** 生成的完整类 SHALL 在相同效果顺序下只执行一次清理，并返回清理前已求出的值；清理对局部或字段的写入不得使返回值被重新求出

#### Scenario: Try exception runs cleanup and keeps the same primary
- **WHEN** try 内的值计算或语句抛出异常，清理正常完成，异常路径原本重抛捕获的同一对象
- **THEN** 生成的完整类 SHALL 保留 try 中已发生的效果、执行清理一次并抛出同一异常对象

#### Scenario: Cleanup itself fails
- **WHEN** 清理调用在正常返回或 try 异常路径上抛出新异常
- **THEN** 生成的完整类 SHALL 让清理异常覆盖原返回或原异常，并且清理不得再执行第二次

#### Scenario: Catch-all without complete proof
- **WHEN** 清理副本不等价，保护区间遗漏或重复覆盖退出路径，异常路径未重抛同一异常，或清理含显式替代 return/throw
- **THEN** 恢复层 MUST 不将该区域输出为已证明的 `finally`，MUST 保留正常路径、异常处理器、可观察生产者及具体缺失证据的来源

#### Scenario: A catch-all range catches its own normal cleanup
- **WHEN** 合法 Java 8 class 的 catch-all 半开范围由 `[0,20)` 扩到 `[0,23)`，正常清理调用在 BCI 20 抛错后进入 BCI 25 的处理器并再次清理
- **THEN** 恢复层 MUST NOT 折叠为只执行一次清理的 `finally`；未能完整呈现等价 Java 结构时 MUST 保留指名 BCI 的拒绝与两份清理的物理来源

#### Scenario: Sources and bounded recovery
- **WHEN** 请求默认或完整来源，或清理证明/发射中遇到预算耗尽或取消
- **THEN** 两种来源模式 SHALL 有相同正文，并保留保护区间、正常与异常清理副本、保存的返回值和重抛指令的物理 BCI/成员来源；停止 MUST 遵循既有受限输出契约
