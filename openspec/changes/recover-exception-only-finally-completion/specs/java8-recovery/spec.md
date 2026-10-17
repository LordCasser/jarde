## ADDED Requirements

### Requirement: Recover proved exception-only finally completion

对于经完整证明的有界直线方法，如果 try 无法正常完成、以抛异常结束，且 finally 只含正常落穿的副作用，系统 SHALL 恢复完整 try/finally 源码，保留所有可抛操作、原始异常对象、cleanup 恰好一次与覆盖顺序。生成源码在支持的 Java 8 目标编译及重建运行中 MUST 与原 class 一致。

#### Scenario: Cleanup completes and preserves pending primary

- **WHEN** try 的副作用完成后抛出原始异常对象，finally 的副作用正常完成
- **THEN** 恢复源码 SHALL 先执行 try 副作用再执行一次 cleanup，并抛出同一个原始异常对象，不伪造 return 或正常完成路径

#### Scenario: A protected invocation throws before explicit throw

- **WHEN** try 内受保护的普通调用抛出已知 RuntimeException 或 Error，尚未执行后续显式 throw
- **THEN** cleanup SHALL 仍执行恰好一次；cleanup 正常完成时传播该调用抛出的同一对象，后续 try 操作不得执行

#### Scenario: Cleanup exception overrides the pending primary

- **WHEN** cleanup 的普通调用在原显式异常或 try 调用异常待传播时抛出已知 RuntimeException 或 Error
- **THEN** 最终传播的对象 SHALL 是该 cleanup 异常，cleanup 不得重入或重复执行

#### Scenario: Complete source and physical origins remain available

- **WHEN** 两真实编译器产生的支持输入通过公共分析与整类源码入口恢复
- **THEN** default/all evidence 下正文 SHALL 一致，所有原物理方法、指令来源与方法身份 MUST 保留；独立完整类的原样源码 SHALL 编译并在隔离重建 classpath 上与固定原始 oracle 一致

#### Scenario: Unproved completion retains an honest refusal

- **WHEN** 原始异常重抛身份、实际异常覆盖、完整出口或所有权无法证明，或 finally 自身包含显式非局部 return/throw
- **THEN** 系统 MUST 保留现有可定位拒绝及原始事实，不以普通 catch、虚构正常清理副本或删除原方法冒充该 finally 已恢复

#### Scenario: Analysis stops without publishing partial recovery

- **WHEN** 证明或呈现过程中触发 budget、cancellation 或 deadline stop
- **THEN** 系统 SHALL 保留真实停止原因和计量，且 MUST 不发布部分正文、来源映射或已提交的部分 ownership
