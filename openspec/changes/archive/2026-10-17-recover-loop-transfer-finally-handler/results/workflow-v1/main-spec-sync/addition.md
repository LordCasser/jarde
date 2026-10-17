
### Requirement: A proved loop transfer retains complete finally handler evidence

系统 SHALL 完整恢复有充分证据的int数组循环continue或break清理形：本轮正常转移与落穿均对同一int局部作同一更新，catch-all出口保存原异常、清理并重抛。输出 MUST 保持转移目标、每路径效果、真实保护范围及物理来源；不能仅因handler无正常前驱而省略未证明的异常语义。缺证保持原拒绝，实际停止保留原原因与用量，成功须经完整生成源码原样重编及隔离对照验收。

#### Scenario: Continue advances while break exits

- **WHEN** 固定两方法从c=0开始，遇x==3分别continue或break，其它轮次先c加x，离开try时均c加100
- **THEN** 输入[3,4] SHALL 分别返回204与100，[1,3,4]分别返回305与201，[3,3]分别返回200与100；MUST NOT 合并转移目标、遗漏清理或额外运行break后的轮次

#### Scenario: Ordinary iterations keep both local updates

- **WHEN** 固定方法输入为空、[1]或[1,2]且没有命中转移条件
- **THEN** 两方法 SHALL 分别返回0、101、203，并保持int溢出及原每轮本体更新与清理更新的顺序和次数；不得因去除fallback而丢失一次局部写回

#### Scenario: Protection starts after array reads

- **WHEN** 原数组长度和本轮元素读取位于try外，null在循环准备时抛出NullPointerException
- **THEN** 恢复 SHALL 保持该真实边界；不能把数组访问移入try、为try外异常执行清理，或把未发生的受保护异常声称为运行已覆盖

#### Scenario: The exceptional completion needs full evidence

- **WHEN** 系统认领catch-all保存/清理/重抛出口，或基于受保护操作不可抛异常而采用等价正常表示
- **THEN** 所有物理保护范围、入口、实际effects及异常值身份 SHALL 完整证明；有可达异常边时须保留原异常经过一次清理后重抛的语义。MUST NOT 仅凭没有正常前驱、没有运行触发或原handler只剩注释就宣称完整恢复

#### Scenario: Different effects or values remain unclaimed

- **WHEN** 副本局部/增量不同、保护范围或handler身份改变、异常保存/重载不是同值、存在额外消费者/正常handler入口/可能抛出的新操作
- **THEN** 本能力 MUST 不认领未证明部分并保留诊断、bytecode及物理来源；原named-catch与其它finally规则不得因此放宽，已有正确能力仍按其原契约处理

#### Scenario: Every physical operation keeps its origin

- **WHEN** 公共库或JSON CLI交付上述已证明方法
- **THEN** 方法 SHALL 为Java/Structured且无该handler的fallback，完整text及source map保留同一物理方法的try外读取、条件、两条正常清理、转移、handler保存/更新/重抛、循环更新与最终return的全部实际BCI；不能靠空语句或伪造来源覆盖未证明指令

#### Scenario: Real proof stops publish no ownership

- **WHEN** 候选已经开始真实证明，预算、取消或期限在证明或正文提交前被观察到
- **THEN** 系统 MUST 保留原原因、位置及已批准usage，不发布部分证书、块ownership或未提交正文/来源，不把停止降为普通缺证后继续；真实证明期预算/取消检查不能由入口预取消或伪造错误代替

#### Scenario: Complete source execution defines the bounded acceptance

- **WHEN** 两真实编译器原独立完整类经最终冻结CLI恢复，未改driver只对原样重编的完整生成类编译后在Java8/23运行
- **THEN** 固定多组continue/break/落穿/null/溢出观察 SHALL 逐字等原class oracle，rebuilt classpath不混原class/JAR，不改文本/driver/方法/oracle。原FX/FY/FO完整类与JADX对照须如实保留，运行不可观察的handler事实仍依赖独立静态证明；本片不代表父计划或任意JAR/WAR全部完成
