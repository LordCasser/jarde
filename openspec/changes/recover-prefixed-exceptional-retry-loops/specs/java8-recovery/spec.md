## ADDED Requirements

### Requirement: Prefixed exceptional retry loop recovery
系统 SHALL 在可完整证明的单具名 catch 重试嵌套循环中恢复语义等价的结构化 Java，区分外层循环入口、带前缀的内层重试入口、保护区起点及外层退出。恢复 SHALL 保持前缀执行次数、正常与异常续接、局部值与效果顺序，不依赖源码命名或固定字节码位置。

#### Scenario: Original nested loop with catch retry
- **WHEN** 输入为具名 catch 回到内层前缀、try 正常完成回到外层入口、条件分支退出外层的完整合法类，且所有路径与归属可证明
- **THEN** 目标方法 SHALL 结构化恢复，保持类的全部方法，完整源码原样重编后的返回、异常和效果 SHALL 与原类一致。

#### Scenario: Renamed and effectful prefix equivalent
- **WHEN** 同构合法类改变名称、常量或前缀效果，且仍满足相同边界与完整证明
- **THEN** 恢复 SHALL 保留每次重试及外层迭代的真实效果次数与顺序，不因命名或位置变化误拒或错码。

### Requirement: Exceptional retry ownership and source integrity
系统 SHALL 仅在具名异常表、实际可抛点、参数绑定、正常与异常入口、退出及物理身份均闭合时恢复此形；每个物理块 SHALL 只归属一次，所有已恢复指令 SHALL 有合法同方法来源，非局部转移 SHALL 保持真实目标和来源。

#### Scenario: Unsupported competing entry or handler row
- **WHEN** 候选存在无法证明的额外入口、共享或交叉行、错误 handler 回跳、错误正常续接、带效果退出 gateway 或 clone 身份错配
- **THEN** 系统 SHALL 保留明确的未恢复结果，不发布推测的结构化正文或借用其他路径的来源。

#### Scenario: Hidden transfer sources remain attributable
- **WHEN** 已恢复嵌套循环以标签 break、continue 或隐式循环尾呈现字节码转移
- **THEN** 来源 SHALL 覆盖真实物理转移指令及全部恢复指令，不重复归属 handler 或方法尾部。

### Requirement: Exceptional retry proof stop preservation
新增证明和构建 SHALL 受现有预算、取消及期限限制；停止 SHALL 保持原停止原因并且不发布部分候选正文、块归属或映射。

#### Scenario: Stop inside candidate after partial construction
- **WHEN** 新候选已开始证明或构建后实际触发预算、取消或期限停止
- **THEN** 系统 SHALL 返回对应停止状态并保持未发布的候选状态完整回滚，不把停止降为形状不匹配。
