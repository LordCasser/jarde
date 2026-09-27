## ADDED Requirements

### Requirement: 共享 catch-all 的具名 catch/finally 恰执行一次清理

当一个 Java 8 方法的同一具名 catch 行优先于 try 范围的 catch-all，具名 catch 正文另有 catch-all 行指向同一异常 handler，且两个正常出口与异常 handler 各持一份经 SSA/效果证明等价的清理副本时，系统 SHALL 仅在所有异常边、半开范围、返回值快照、原异常重抛和物理来源闭合后投影一份源级 `finally`。每条完成路径 MUST 与原 class 一样清理恰一次；证据不完整时 MUST 保留原物理引用和停止/拒绝状态。

#### Scenario: 正常返回与具名 catch 返回共享一次清理
- **WHEN** 最小 `handled(false)` 走正常返回，`handled(true)` 经 `IllegalArgumentException` catch 返回，二者及共用异常 handler 的副本均满足完整证书
- **THEN** Jarde 的完整 Java 8 类源码重编、`java -Xverify:all` 后，两条路径的返回文本与清理次数均与原 class 一致且清理次数各为 1；固定 JADX 结果单独记录，不因其可能在正常路径重复清理而改变 Jarde 语义

#### Scenario: 扩围异常行或竞争副本不得折叠
- **WHEN** 任一 catch-all 覆盖自己的正常清理、handler/cleanup 多一个入口或目标、三个副本的效果/实参不等、saved return/原异常 SSA 关系不闭合，或预算/取消中断
- **THEN** 系统 MUST 不发布单份 `finally`，不得省略或重复执行清理，并保留对应物理 BCI/异常行的拒绝或停止来源

#### Scenario: 既有单出口 finally 与命名 catch 不回归
- **WHEN** 输入是已受证的单正常出口 finally、普通具名 catch 或覆盖型 finally 负例
- **THEN** 既有成功或拒绝边界 SHALL 保持，不因本多出口证书扩大其声明范围
