## ADDED Requirements

### Requirement: Proved legacy clone coverage

系统 SHALL 在返回地址和调用上下文已证明、版本允许 legacy 子程序的输入上保留每个调用路径内的合法控制流；不能仅因内部块缺少直接调用或返回入边而拒绝规范化。克隆 MUST 由实际进入其上下文的路径覆盖，物理来源与调用身份保持分离；无法证明覆盖时不得发布 CanonicalCFG，原始事实和原因仍保留。新增分析工作与派生存储 SHALL 在请求预算内完成，预算或取消停止不得变成正常 fallback 成功，verification 不得被提升为完整验证。

#### Scenario: Shared subroutine contains conditional branches

- **WHEN** 两个调用点进入同一已证明子程序，子程序含多个普通条件分支块和合法返回
- **THEN** 每个调用路径保留独立克隆，内部块可经同上下文真实普通路径被覆盖，CanonicalCFG 不因缺少直接调用入边被拒绝，来源仍指向原物理指令

#### Scenario: Nested return reaches an outer continuation

- **WHEN** 外层子程序调用内层，内层合法返回外层的 continuation，外层再经普通控制流继续
- **THEN** 内层返回只覆盖匹配外层调用路径的 continuation 和其实际后继，不合并两个外层调用点的身份

#### Scenario: Exception flow stays inside a proved context

- **WHEN** 已证明子程序的异常路径进入同调用上下文的 handler
- **THEN** handler 与正常块一样接受真实上下文路径覆盖，异常表顺序、保护范围和物理来源保持一致

#### Scenario: An explicitly dead call context is preserved

- **WHEN** 合法输入含规范化保留的、方法入口不可达的调用点及其子程序
- **THEN** 保留该调用点的独立克隆，覆盖校验不得把其错误标成 live，也不得因入口不可达而删除已有产物

#### Scenario: A clone has no rooted context path

- **WHEN** 克隆只形成自循环、从匹配入口断开、依赖未覆盖克隆的返回，或只能经不匹配调用路径的普通或调用返回边进入
- **THEN** 覆盖不可证明，规范化拒绝发布 CanonicalCFG；同路径其他块存在一个调用入口不足以证明该克隆

#### Scenario: Coverage checking stops before publication

- **WHEN** 克隆覆盖分析的新增存储或工作耗尽预算，或请求在校验中收到取消
- **THEN** 保留真实预算或取消原因与实际 usage，不发布半完成 CanonicalCFG，不执行其后 Frame/SSA，也不声称输入已经完成 verifier
