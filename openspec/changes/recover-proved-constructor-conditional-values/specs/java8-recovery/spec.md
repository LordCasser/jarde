## ADDED Requirements

### Requirement: Proved conditional expressions retain branch-local constructions

系统 SHALL 在构造身份、唯一消费者、完整条件分支及实参求值关系均获证时，恢复条件表达式中的分支局部对象构造；构造仅在对应分支被选择时执行，原对象身份、实参次序、异常路径和物理来源 MUST 保持。无法证明时 MUST 保留原字节码效果及拒绝原因，不以复制构造或强制类型转换掩盖缺口。

#### Scenario: Fixed sequential loops return a constructed conditional value

- **WHEN** 固定TestSequentialLoops2在两循环之后，通过条件值选择原字符串或按已生成字符数组构造新字符串
- **THEN** 完整生成类 SHALL 可重编译并与原双JDK案例的返回值、对象是否重用及异常相同，尾部构造和条件分支的全部物理来源完整

#### Scenario: Two conditional arms create distinct objects

- **WHEN** 两个独立构造分别位于条件两臂，且各构造结果只作为该条件值的一次输入并由唯一消费者读取
- **THEN** 恢复 SHALL 保持两臂各自构造身份，并仅执行被选择一臂的构造和实参效果一次

#### Scenario: Ownership or full conditional proof is incomplete

- **WHEN** 构造或合并结果共享、多次读取，合并由循环携带或串联，分支存在未纳入表达式的独立效果，或者异常边无法获证
- **THEN** 恢复 MUST 拒绝无法证明的条件表达式呈现，保留完整效果和物理来源，不额外复制对象或移动求值

### Requirement: Conditional construction presentation agrees across reports

系统 SHALL 仅将实际提交源码中唯一匹配当前物理方法及构造位置的对象构造报告为已呈现。构造 Site 候选已识别但后续源码证明失败时，摘要、拒绝诊断及被选择的NewRecord MUST 一致报告未呈现，并保留已知构造位置；其他方法的同BCI节点、方法引用和重复节点 MUST NOT 充当成功构造的证据。

#### Scenario: Exactly one committed construction

- **WHEN** 最终源码含当前构造身份对应的唯一真实new表达式
- **THEN** 摘要 SHALL 计为presented，诊断不生成该构造的拒绝，被选择的NewRecord呈现状态为true

#### Scenario: Candidate does not become a unique source construction

- **WHEN** 构造候选后续被回退，匹配源码构造不存在、重复，或仅有其他物理方法同BCI的节点
- **THEN** 摘要 SHALL 计为未呈现，诊断说明未提交唯一构造，被选择的NewRecord呈现状态为false

#### Scenario: Evidence selection and driver ranges

- **WHEN** 同一方法使用default、all、无RuleDetails选择或带BCI范围的选择
- **THEN** 正文、来源及构造呈现判定 SHALL 相同；选择仅决定可选记录交付，范围仅过滤记录，不改变全方法摘要及拒绝诊断

### Requirement: Conditional construction recovery obeys existing stops

系统 SHALL 将条件构造所有权及最终呈现判断纳入既有预算与取消约束；停止 MUST 沿既有公开报告传播，不返回当前方法的部分正文、来源或半完成规则记录。

#### Scenario: Late budget stop during conditional construction work

- **WHEN** 所有权、表达式构建或最终呈现判断达到实际预算限制
- **THEN** 公开报告 SHALL 保留停止维度、实际usage及物理位置，无当前方法部分产物，原先完成的其他成员按既有契约保留

#### Scenario: Cancellation before recovery

- **WHEN** 请求在恢复前已取消
- **THEN** 该工作 SHALL 不开始，并使用既有NotPerformed/取消报告，而非构造成功或普通refusal
