# Spec Delta

## ADDED Requirements

### Requirement: A proved conditional return and fallthrough share one finally

系统 SHALL 恢复可证明的无循环条件try，其一个正常出口先保存int返回值、另一个正常出口落穿，两个出口与唯一catch-all异常出口各执行同一条直线副作用清理。输出 MUST 保持值求得位置、每路径一次清理、原异常或清理异常的优先级与物理来源；缺证保持既有拒绝，真实执行停止保持原原因和用量。成功 MUST 以完整生成源码原样重编及隔离运行对照验收。

#### Scenario: Return and fallthrough keep separate completions

- **WHEN** 固定条件体对空数组或[2]正常落穿，对[3]在try内取得返回字段的值，并在两条出口各执行同一清理
- **THEN** 恢复 SHALL 保持落穿后返回-1及条件成立时返回已取得的3000；每次清理一次，MUST NOT 把正常落穿改为条件返回或丢失方法后继

#### Scenario: The return value is captured before cleanup

- **WHEN** 未改driver把返回字段置3000，清理中原有println的受控实现把该字段改为99
- **THEN** 原类与完整重建类 SHALL 都返回3000且字段变99、清理次数为1；MUST NOT 在清理后重新读取字段来取得返回值

#### Scenario: An exception runs cleanup once and keeps its identity

- **WHEN** null输入在受保护数组读取处抛NullPointerException，清理正常完成
- **THEN** 系统 SHALL 保持一次清理及原异常的传播，不把该路径转为返回值、不生成吞异常的catch，也不重执行抛异常的producer

#### Scenario: A cleanup exception replaces the pending completion

- **WHEN** 固定driver令原println调用抛专用清理异常，分别覆盖待返回3000与受保护NullPointerException两条路径
- **THEN** 两条路径 SHALL 都只执行一次清理并传播专用清理异常；MUST NOT 让自身handler再次捕获清理操作、重复执行清理、恢复原返回或优先抛原NullPointerException

#### Scenario: Copies and physical ownership are complete

- **WHEN** 三份清理经证明折叠为一个finally
- **THEN** 正文与来源 SHALL 保留受保护条件、值保存/返回、两条正常清理与异常保存/重抛的实际方法及BCI；没有被证明的入口、额外消费者、跨范围效果或副本差异 MUST 阻止本规则认领。其它规则已正确恢复的形态保持原契约

#### Scenario: Proof work preserves actual stops

- **WHEN** 新候选证明已开始，真实预算、取消或期限在该证明或正文提交前被观察到
- **THEN** 恢复 MUST 保留原停止原因、位置与已准许用量，不发布部分证书、ownership或未提交正文/来源，不把停止当普通缺证继续后继恢复；真实证明收费中的预算/取消验证不能由入口预取消或伪造错误代替

#### Scenario: Full source and independent driver prove recovery

- **WHEN** 真实javac8及javac23 --release8生成的独立完整类分别经最终CLI恢复
- **THEN** 每份完整report.text SHALL 原样重编，未改driver仅对rebuilt类编译后在Java8/23运行，六组固定观察逐字匹配原类oracle；classpath不得混原class/JAR，不得修生成文本或删方法，CLI exit0或结构平面不替代执行证据

#### Scenario: The parent finally work keeps its remaining gaps

- **WHEN** 本片验收独立完整类及原FY.loopless方法
- **THEN** 原完整FY与FX的所有方法 SHALL 继续如实报告并另存整类重编结果；只恢复loopless不得被标为noCall/finReturn或整个父计划已完成。既有FO、直线/条件清理及其它finally控制保持原契约，不通过删除失败方法、调整oracle或改旧计费表完成本片
