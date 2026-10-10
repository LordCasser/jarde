## ADDED Requirements

### Requirement: Proved discarded call results keep physical sources

系统 SHALL 在普通调用语句已由同次真实字节码证明为返回值丢弃时，将对应pop保留为完整调用语句跨度的derived物理来源。系统 MUST 保持原调用primary、正文、求值次序与准确方法身份，MUST 不为未证对应关系制造来源。

#### Scenario: A successful call is immediately discarded

- **WHEN** 同block中非void调用的准确结果只被紧邻pop消费且不写局部，普通调用语句成功恢复
- **THEN** pop查询返回该调用语句的完整跨度，原调用primary与所有既有来源不变，default/all正文与来源一致

#### Scenario: A discard pairing is not proved

- **WHEN** pop来自另一值、存在额外reader或中间操作，或指令为pop2，未形成上述对应
- **THEN** 系统保持既有恢复或明确拒绝及物理来源，MUST 不把未证指令附到无关调用语句

#### Scenario: A refused call already quotes the discard

- **WHEN** 准确调用呈现被拒绝且既有quote包含对应pop
- **THEN** 系统保持原quote和来源，不另造成功调用或重复求值

#### Scenario: Origin publication stops

- **WHEN** 追加来源或发布正文遇到预算不足或取消
- **THEN** 系统传播既有Stop，MUST 不发布部分正文或来源映射
