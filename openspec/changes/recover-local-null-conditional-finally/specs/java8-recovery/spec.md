## ADDED Requirements

### Requirement: 局部可空引用门控的调用清理恢复为唯一 finally

系统 SHALL 在两行 any 异常表（正文行 + 自保护绑定行）、`[aconst_null, astore s]` lead、受保护正文内同一槽的赋值全集均有出处、两份"`aload s; ifnull exit; aload s; invoke ()V`"副本逐参数同形、保存返回与原 Throwable 身份均被证明时，输出唯一 `try/finally`：lead 呈现为局部声明与 `= null` 初始化，清理折叠为一份 `if (local != null) { local.close(); }`，两份副本的全部物理 BCI 均有来源映射。证明不完整时 MUST 保守拒绝，不得发布部分结构或改判为无条件清理。

#### Scenario: 正常与异常路径行为

- **WHEN** 固定形态类以原 class、固定 JADX Java-input、Jarde 三方 `javac --release 8` 重编并运行
- **THEN** 正常路径清理恰执行一次并返回正文值；正文抛错路径清理恰执行一次且原值重抛；正文赋值为 null 时跳过清理；清理自身抛错时新异常覆盖原异常；`java -Xverify:all` 全通过且逐路径一致

#### Scenario: 证明失败的安全拒绝

- **WHEN** 无 lead null 初始化、判空方向反转、两副本槽或调用目标不一致、正文区间外同槽赋值、副本文法增删，或保存/重抛身份改写
- **THEN** 系统 MUST 拒绝唯一 finally 投影，保留可定位的物理与 SSA 证据，不得把条件清理改判为无条件清理或用户 catch

#### Scenario: 与相邻证书互不误触

- **WHEN** 输入是 Test14 字段判空、Tf4 布尔标志或本形态的近邻变体
- **THEN** 各证书只命中自己的文法；其它形状保持既有恢复或拒绝行为不变

#### Scenario: 停止与来源

- **WHEN** 证明、区域构造或输出被取消或耗尽预算
- **THEN** 原子停止，无半成品；成功时 lead、正文、两份副本与保存/重抛的全部目标 BCI SHALL 在 source map 中可查
