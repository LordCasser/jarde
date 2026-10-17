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

#### Scenario: 混合出口共享循环尾
- **WHEN** 完整SL的mixed方法包含switch break、loop continue、method return与default累积，独立driver传入`{1,2,4,3}`
- **THEN** 输出 SHALL 为124，continue路径不得执行循环尾，break/default路径各执行一次循环尾，return后不再执行尾；局部join与continue目标保持不同身份，物理块不得重复认领

#### Scenario: 不可证明时维持拒绝
- **WHEN** 出口归属或局部 def-use 无法在同一词法结构内闭合
- **THEN** 系统 MUST 保持整方法响亮拒绝与可定位诊断；晚预算、取消或期限停止不得发布部分正文或source map，保持原停止原因与已发生usage

#### Scenario: 无局部join的循环回跳来源
- **WHEN** SN的case-return形已证明一个arm返回、另一arm回到当前循环，局部join与postdom均为空
- **THEN** 系统 SHALL 保留该arm的精确continue目标，goto57/60等物理回跳BCI须有可定位source map，不能仅因输出行为正确而省略来源；沿用既有loop transfer表示
