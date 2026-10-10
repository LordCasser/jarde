## ADDED Requirements

### Requirement: A proved nonempty if arm retains its join transfer source

系统 SHALL 在已恢复 If 的非空分支末尾存在准确、唯一指向该 If 汇合点的无条件跳转时，将该物理跳转作为完整 If 语句的派生来源保留。来源 MUST 绑定原class字节、物理方法name/descriptor与准确BCI，保持已有正文及来源，不得归到外层循环或分支中普通赋值。

#### Scenario: A nonempty conditional arm closes at its proved join
- **WHEN** counted(II)I 在已恢复循环内的非空then arm通过物理goto@20→27到该If准确汇合点，完整边与唯一arm所有权均已证明
- **THEN** @20以derived来源覆盖完整If语句，condition@14与外层while@30及所有旧来源保留，全部物理BCI可准确查询

#### Scenario: A source addition does not change complete executable source
- **WHEN** 双JDK原class分别以default与all证据profile恢复完整类
- **THEN** 两profile正文与map恒同，完整生成类原样重编运行的exit/stdout/stderr与同JDK原class逐字一致

#### Scenario: An unproved terminal edge gains no invented origin
- **WHEN** 缺少准确汇合点，transfer并非该arm的最后已持有goto，或者出边有错误target、多边、异常或所有权冲突
- **THEN** 系统不得为该transfer添加此If的derived来源，既有结构恢复与保守拒绝按原规则处理

#### Scenario: Other source owners remain independent
- **WHEN** 已有空arm来源、for/while latch来源和嵌套else回跳与本If汇合来源同时存在或缺失
- **THEN** 既有来源保持原归属，本片不凭运行成功宣称未证明的嵌套else回跳已覆盖

#### Scenario: A stopped origin proof publishes no partial artifact
- **WHEN** 新汇合来源证明遇到预算不足或取消
- **THEN** 系统传播既有Stop，MUST 不发布部分正文或部分source map
