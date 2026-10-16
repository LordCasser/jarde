# Spec Delta

## ADDED Requirements

### Requirement: A proved loop-local finally preserves its pending return and next iteration

系统 SHALL 恢复有完整证据的int数组循环内条件try：命中条件时先保存int返回值，未命中时继续，finally在每条离开try的路径上对同一int局部作相同更新。输出 MUST 保持求值顺序、每路径一次清理、循环更新与真实异常范围及来源；缺证保留原拒绝，实际执行停止保留原原因与用量。成功 MUST 经完整生成源码原样重编及隔离执行对照验收。

#### Scenario: A pending return retains the value before local cleanup

- **WHEN** 固定noCall形状从c=0开始，每轮x==3时取得c+7、finally令c加100
- **THEN** 输出 SHALL 让[3]返回7、[1,3]返回107、[1,2,3]返回207、[3,1]和[3,3]返回7；MUST NOT 在清理后重新计算c+7、漏掉return或继续执行后续轮次

#### Scenario: Fallthrough performs cleanup and loop update exactly once

- **WHEN** 输入为空、[1]、[1,2]或[1,2,1]，没有触发提前返回
- **THEN** 输出 SHALL 分别返回0、100、200、300，保持每个有效轮次一次清理及一次索引更新，不丢失方法最终return、不重复或提前清理

#### Scenario: Array reads keep their actual exception boundary

- **WHEN** 原方法在进入try前读取数组长度与本轮元素，null输入于循环准备处抛NullPointerException
- **THEN** 恢复 SHALL 保持读取在try外并传播原异常；MUST NOT 扩大保护范围、把数组读取移入finally或为假定的受保护异常执行额外清理

#### Scenario: Incompatible copies or value flow remain unclaimed

- **WHEN** 清理副本的局部或增量不同、待返回值有额外消费者、存在未解释的写入/入口/异常范围，或清理所读的实际值流无法完整证明
- **THEN** 本规则 MUST 不认领未证明结构，保留bytecode、拒绝原因和物理来源；同slot或显示名称相同不能代替值流证据，不能普遍放宽其它finally规则

#### Scenario: Physical origins include the lead and every completion

- **WHEN** 公共库或JSON CLI交付该已证明循环finally
- **THEN** source map SHALL 保留同一物理方法的try外数组读取、受保护条件、返回值保存与消费、三个清理副本、handler绑定/重抛以及循环更新的实际BCI；折叠副本不得丢失异常表事实或原始分析

#### Scenario: A late proof stop commits no partial ownership

- **WHEN** 候选已经开始实际证明，预算、取消或期限在证明或正文提交前首次被观察到
- **THEN** 系统 MUST 保留实际原因、位置和已批准用量，不发布部分证书、块ownership或未提交正文/来源，不将停止当普通缺证继续；预算与取消须有真实证明期检查验证

#### Scenario: Whole source and the unchanged driver prove the bounded slice

- **WHEN** 真实javac8及javac23 --release8的独立NoCallFinally完整类经最终冻结CLI恢复
- **THEN** 两份完整text SHALL 原样重编，未改driver只对rebuilt类编译后在Java8/23运行，十组固定观察逐字匹配84字节原oracle；不得混用原class/JAR、改生成文本、删方法或改oracle。运行观察不直接证明待返回之后不可观察的局部清理次数，该结论还须独立结构和值流证明

#### Scenario: The broader finally parent retains unresolved cases

- **WHEN** 本片独立完整类通过验收，并重新观察原FY.noCall
- **THEN** 原FY/FX所有方法及整类重编结果 SHALL 继续完整记录，finReturn及其它未证形状保留真实缺口，不把局部成功当父计划全部完成；既有FO、LooplessFinally与其它finally控制保持原行为和计费契约
