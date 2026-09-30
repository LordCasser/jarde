## ADDED Requirements

### Requirement: 正文置真局部标志门控的字段更新清理恢复为唯一 finally

系统 SHALL 在两行 any 异常表（正文行 + 自保护绑定行）、两指令 `false` 初始化 lead、受保护正文恰一次置真同一局部布尔 slot、保存返回与原 Throwable 身份、两份逐参数同形的"`iload` 条件 + 同字段读改写"副本均被证明时，输出唯一 `try/finally`：lead 呈现为布尔声明与 `false` 初始化，清理折叠为一份 `if (!flag) { field 复合更新; }`，两份副本的全部物理 BCI 均有来源映射。证明不完整时 MUST 保守拒绝，不得发布部分结构或改判为无条件清理。

#### Scenario: 正常与异常路径行为

- **WHEN** 固定形态类以原 class、固定 JADX Java-input、Jarde 三方 `javac --release 8` 重编并运行
- **THEN** 正常路径 `result` 净变化为 +1 且返回 `call()` 值；正文抛错路径执行一次 `result -= 2` 并原值重抛；清理抛错时新异常覆盖原异常，`java -Xverify:all` 全通过且逐路径一致

#### Scenario: 证明失败的安全拒绝

- **WHEN** 无置真写、两次置真或写后含其它语句、条件反转、两副本常量/字段/操作不一致、lead 非 `false` 常量初始化，或副本文法增删指令
- **THEN** 系统 MUST 拒绝唯一 finally 投影，保留可定位的物理与 SSA 证据，不得把条件清理改判为无条件清理或用户 catch

#### Scenario: 停止与来源

- **WHEN** 证明、区域构造或输出被取消或耗尽预算
- **THEN** 原子停止，无半成品；成功时 lead、正文、两份副本与保存/重抛的全部目标 BCI SHALL 在 source map 中可查
