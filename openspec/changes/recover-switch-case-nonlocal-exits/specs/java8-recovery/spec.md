## ADDED Requirements

### Requirement: 循环内 switch case 非局部出口的准确恢复

当 Java 8 方法的循环内 switch 在 case 体内包含 `return` 或带标签 `break` 到外层循环时，系统 SHALL 只在出口归属、局部值 def-use 与效果顺序全部可证时恢复相应出口结构，重编运行后与原 class 行为一致。

#### Scenario: case 内 return 锚点
- **WHEN** 输入为固定 `SN`（retInSwitchNoTail/retInSwitchTail）Java 8 class 并恢复、重编、运行
- **THEN** 两形 SHALL 恢复为 case 内 return，完整类通过 `javac --release 8` 与 `java -Xverify:all`，`{1,3}` 输出 `101`/`111` 与原一致

#### Scenario: case 内带标签 break 锚点
- **WHEN** 输入为固定 `SN.labBreak` 与判别驱动 `{9,1,5}`
- **THEN** 恢复 SHALL 保留（或等价重建）外层出口，重编运行输出 `10`；jadx 的单迭代重构（输出 9）是行为错码反例

#### Scenario: 局部出口对照零回退
- **WHEN** 输入为 `SM` 五对照形（break/continue/混用/空尾/if 版）
- **THEN** 既有恢复 SHALL 逐字节不变

#### Scenario: 不可证明时维持拒绝
- **WHEN** 出口归属或局部 def-use 无法在同一词法结构内闭合
- **THEN** 系统 MUST 保持整方法响亮拒绝与可定位诊断
