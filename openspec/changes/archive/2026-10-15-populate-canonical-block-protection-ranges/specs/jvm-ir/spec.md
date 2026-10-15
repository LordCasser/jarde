# Spec Delta

## ADDED Requirements

### Requirement: Canonical block protection reflects physical member ranges

完整 canonical 图 SHALL 对每个 block 发布按物理异常表声明顺序去重的保护 ordinal：当且仅当该节点至少一个真实原始成员块内有指令位于该半开保护范围。融合节点 MUST 合并成员的范围事实，不能把非连续块之间的包络空隙算作成员；clone 的事实保持其原始块、调用路径与物理方法身份。该字段不承诺整个融合节点均受保护。

#### Scenario: Protected nonthrowing member remains visible

- **WHEN** 一个原始块包含受某异常保护范围覆盖的非抛出指令，而同一记录的实际异常边只来自其他 throwing block
- **THEN** 本块的公开保护 ordinal 包含该记录，handler-row 与 throw-site 事实仍仅按其原有异常边契约发布，不因本字段扩大异常输入

#### Scenario: Fused members cover only part of the node

- **WHEN** 融合节点的一部分真实成员受保护，其他成员不在范围内
- **THEN** 保护 ordinal 为真实成员范围事实的有序去重并集，物理原始块和 origin 保持；不得解释为整个节点的全覆盖

#### Scenario: Forward fusion skips a protected gap

- **WHEN** 前向跳转融合两个非连续块，其包络内的另一个原始块受保护，但融合成员都不受该记录保护
- **THEN** 融合节点不发布该 ordinal，被跳过的块仍按自身真实范围发布，不由包络推导覆盖

#### Scenario: Boundaries overlapping rows and clones preserve identity

- **WHEN** 保护范围端点等于块边界、多条记录重叠，或同一原始块在多个 live/dead jsr 调用路径下克隆
- **THEN** 使用半开边界核实际成员，多个 ordinal 按声明顺序保留一次；各 clone 仅描述自己的真实成员与路径，不能被折叠为一份物理控制流证据

### Requirement: Canonical protection growth obeys normalization budgets

系统 SHALL 在新增范围扫描、ordinal 存储及保留副本增长前按现有 analysis/storage 预算计费；既有载荷移动可复用已计费存储。预算不足或取消时 MUST 按既有 normalization 终止契约保留原因和最后有效阶段，不发布带部分保护事实的完整 canonical 图；无异常记录的图不因本功能增加无用扫描或 ordinal 存储。

#### Scenario: Protection storage or work cannot be admitted

- **WHEN** 真实保护 ordinal、融合成员或装配处理的剩余 IrItems/AnalysisSteps 不足，或处理中收到取消
- **THEN** 在相应增长前停止，保持预算维度或取消原因，不把不完整范围事实发布为成功图
