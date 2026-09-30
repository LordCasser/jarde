## ADDED Requirements

### Requirement: 分段两行表与早返回副本的 null-lead finally 恢复

系统 SHALL 在两行同 handler 的 any 异常表（无自保护行）、`[aconst_null, astore s]` lead、受保护正文含条件流与同槽赋值、段间隙恰为"值保存 + 副本 + areturn"的早返回块、三份"`aload s` + 同目标调用"副本逐参数同形且实参槽即 s、两个保存返回（null 字面量与正文生产值）身份均被证明时，输出唯一 `try/finally`：条件正文与早返回完整呈现（`return null;` 保留为正文内语句），清理折叠为一份调用，三份副本的全部物理 BCI 均有来源映射。证明不完整时 MUST 保守拒绝，不得发布部分结构或合并两个返回。

#### Scenario: 五路径行为

- **WHEN** 固定形态类以原 class、固定 JADX Java-input、Jarde 三方 `javac --release 8` 重编并运行
- **THEN** 已缓存路径跳过读入仍清理一次并返回转换值；未缓存路径读入、写字段、清理一次；`validate()==false` 路径提前返回 null 且清理恰一次；正文抛错路径清理一次并原值重抛；清理抛错时新异常覆盖；`java -Xverify:all` 全通过且逐路径一致

#### Scenario: 证明失败的安全拒绝

- **WHEN** 间隙含额外语句、三副本不同形或实参不读 lead 槽、保存返回身份改写、lead 非 null 初始化、行表出现自保护行，或条件跳转被改写
- **THEN** 系统 MUST 拒绝唯一 finally 投影，保留可定位的物理与 SSA 证据；Test13 分段、Test5 多返回与条件/直体家族（Tf1/Tf2/Tf4）的既有行为不变

#### Scenario: 停止与来源

- **WHEN** 证明、区域构造或输出被取消或耗尽预算
- **THEN** 原子停止，无半成品；成功时 lead、条件正文、早返回块、三份副本与两个返回的全部目标 BCI SHALL 在 source map 中可查
