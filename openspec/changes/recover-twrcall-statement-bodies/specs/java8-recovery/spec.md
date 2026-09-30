## ADDED Requirements

### Requirement: TWR 正文的调用语句恢复

系统 SHALL 在 try-with-resources 受保护正文中的语句为"非 void 调用后紧随丢弃其结果"（字节码为 `invoke` + `pop`，pop 唯一读值为该调用结果）时，将该语句随资源证书一起呈现为调用语句。void 调用与 return 的既有呈现 SHALL 逐字不变；调用结果被消费（接收、嵌套或续读）的正文 SHALL 保持既有拒绝。

#### Scenario: 调用语句体恢复

- **WHEN** 固定形态类（如 `try (T r = new T()) { r.toString(); }`）以原 class、固定 JADX、Jarde 三方 Java 8 重编运行
- **THEN** 正文调用语句完整呈现，正常与异常路径 `java -Xverify:all` 行为与原 class 一致

#### Scenario: 结果被消费保持拒绝

- **WHEN** 调用结果被局部接收、pop 读值非该调用结果、或调用与 pop 间有其它指令
- **THEN** 维持既有正文拒绝与诊断，不得呈现部分语句

#### Scenario: 对照组不变

- **WHEN** 正文为 void 调用或含 try 内 return
- **THEN** 与本变更前输出逐字一致
