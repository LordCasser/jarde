## ADDED Requirements

### Requirement: null 局部 lead 的直体 finally 恢复

系统 SHALL 在单行 catch-all 异常表、lead 恰为 `[aconst_null, astore s]` 且在语句边界完整结束、直体正文内同槽赋值、两份无条件同目标调用清理副本的实参均读 lead 槽 s 的合并值流、保存返回为区间内构造、原 Throwable 身份均被证明时，输出唯一 `try/finally`：lead 呈现为局部声明与 `= null` 初始化，清理折叠为一份调用，返回值在正文末呈现为原构造表达式。证明不完整时 MUST 保守拒绝，不得发布部分结构。

#### Scenario: 正常与异常路径行为

- **WHEN** 固定形态类以原 class、固定 JADX Java-input、Jarde 三方 `javac --release 8` 重编并运行
- **THEN** 正常路径清理恰执行一次并返回构造值；正文抛错路径清理恰执行一次且原值重抛；正文赋值为 null 时清理调用仍执行一次且不因 null 抛错；清理自身抛错时新异常覆盖原异常；`java -Xverify:all` 全通过且逐路径一致

#### Scenario: 证明失败的安全拒绝

- **WHEN** lead 非 null 常量初始化或多于两指令、两副本调用目标或实参槽不一致、实参不读 lead 槽、保存返回身份断裂，或副本文法增删
- **THEN** 系统 MUST 拒绝唯一 finally 投影，保留可定位的物理与 SSA 证据；字段赋值 lead 与条件清理家族（Tf1/Tf4）的既有行为不变

#### Scenario: 停止与来源

- **WHEN** 证明、区域构造或输出被取消或耗尽预算
- **THEN** 原子停止，无半成品；成功时 lead、正文、两份副本与保存/重抛的全部目标 BCI SHALL 在 source map 中可查
