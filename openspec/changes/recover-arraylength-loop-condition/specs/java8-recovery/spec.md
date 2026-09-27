## ADDED Requirements

### Requirement: Array length consumed by a loop condition remains in that condition

当 Java 8 方法的数组长度读取能够被证明只在循环测试中作为该测试的值使用，系统 SHALL 将其按原执行次序呈现在条件表达式中，并保留原索引更新和循环后的控制流。长度读取可能抛出异常，因此系统 MUST NOT 将它移出测试、重复求值、删除或把不符合增强 for 条件的循环改写为 foreach。缺少完整值来源、消费、控制流或正文证据时，系统 MUST 保留带来源的保守回退。

#### Scenario: Step-two indexed loop
- **WHEN** `for (int i = 0; i < values.length; i += 2)` 在循环体读取 `values[i]` 并在循环后返回累计值
- **THEN** 原 class、固定 JADX 与 Jarde 的完整 Java 8 源码 SHALL 可重编，`-Xverify:all` 运行 SHALL 逐行一致，且 Jarde SHALL 保留非 foreach 的步长与返回值

#### Scenario: Length read is not exclusively the condition value
- **WHEN** 数组长度读取在测试块中未被终端分支消费、被复用，或测试块还含无法按原位置呈现的效果
- **THEN** 系统 MUST 拒绝该循环的结构化投影，并保留相应物理来源与拒绝原因
