## ADDED Requirements

### Requirement: 可空资源 finally 保持非空关闭与保存返回值

系统 SHALL 仅在受保护的资源赋值、正常与异常两份非空清理、保存的字符串返回值、原 Throwable 重抛及全部异常覆盖可完整证明时，将固定双行布局恢复为可重编的 `try/finally`。输出 MUST 保持同一资源值的词法绑定、Scanner 求值顺序、非空关闭次数、异常身份和每个物理字节码的来源；证据不足时 MUST 安全拒绝。

#### Scenario: 资源存在

- **WHEN** 固定 Test9 同布局方法找到资源并正常读取文本
- **THEN** 恢复源码 SHALL 返回同一文本并关闭该输入流恰好一次；固定原 class、原 Java 8 源码与恢复源码的行为 SHALL 一致

#### Scenario: 资源缺失或正文抛错

- **WHEN** 资源查找返回 `null`，或取得非空资源后正文构造/读取抛错
- **THEN** 恢复源码 MUST 保持原异常；资源为 null 时不调用 `close()`，资源非空时调用一次，且不得把初始化用的 null 与后续赋值拆成两个不相关的源码变量

#### Scenario: 清理抛错

- **WHEN** 正常返回或异常传播期间的 `close()` 抛出异常
- **THEN** 恢复源码 MUST 让清理异常覆盖原返回值或先前异常，不得吞掉、重复执行或借 handler 自保护行重新进入清理

#### Scenario: 反例、来源和停止

- **WHEN** 清理副本的目标/接收者或 null 判断对象不同、保存返回值或重抛 Throwable 被改写、异常范围/入口变化、物理来源缺失，或预算耗尽/请求取消
- **THEN** 系统 MUST 不发布部分 `try/finally`，保留未证字节码和异常行来源；停止 MUST 不交付半成品源码与来源映射
