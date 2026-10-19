## ADDED Requirements

### Requirement: Proved post-test loop break prefixes preserve their statements

系统 SHALL 在普通后测循环的单块出口前缀被独占路径证明闭合时恢复可执行的结构化源码，保留break前语句与最终退出位置；未证明的出口 MUST NOT 通过本规则被伪造为安全break。

#### Scenario: Exit assignment executes before break
- **WHEN** 完整输入在一次迭代内先判断条件，满足时写局部值后跳出，否则继续体尾更新及循环测试，且出口前缀证明闭合
- **THEN** 恢复源码 SHALL 保持写入发生在break前且恰好一次，break路径 MUST NOT 执行体尾更新或后测条件，共享返回 SHALL 保留原值

#### Scenario: Complete source behavior agrees across metadata profiles
- **WHEN** 同一完整原件以debug/no-debug、class/plainJAR与default/all请求恢复，并用固定有限输入重编运行
- **THEN** 结构化结论、方法库存、身份和有效物理来源 SHALL 一致，证据选择 MUST NOT 改变同一输入全文或maps；运行结果、异常类别和stdout SHALL 与原件相同，不要求某种等价循环拼写

#### Scenario: Unproved prefix ownership refuses this rule
- **WHEN** 前缀共享入口、终点错误、跨越父scope、存在重复owner、未知克隆或未闭合的非正常转移
- **THEN** 系统 MUST NOT 通过此规则跳过或复制前缀效果，也 MUST NOT 静默将它当成无效果的跳转桥

#### Scenario: Proof interruption preserves atomic reporting
- **WHEN** 新证明遇到真实预算耗尽或取消
- **THEN** 系统 SHALL 保留准确停止原因与用量，MUST NOT 发布未完成的正文、声明或来源，也 MUST NOT 把停止降为成功或退款后继续
