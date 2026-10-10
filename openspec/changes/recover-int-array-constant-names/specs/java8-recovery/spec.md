## ADDED Requirements

### Requirement: Same-class integer names in direct array returns

系统 SHALL 在完整普通方法直接返回已恢复的一维int数组初始化器、且直接int字面量对应唯一同类static final int常量字段时，以不被该方法词法绑定遮蔽的字段名呈现该元素。系统 MUST 保持完整类的数组值、每次调用分配、求值顺序和运行输出，并保留全部物理成员及指令来源；相等数值不得被描述为原源码确实使用该符号的证明。

#### Scenario: Unique constant in a direct int array initializer
- **WHEN** 完整方法返回 `new int[] { 127, 129, 65535 }`，当前类唯一常量字段CONST_INT的值为65535且名称未遮蔽
- **THEN** 系统 SHALL 将对应元素呈现为CONST_INT，保持其它元素和成员，完整重编执行与原类逐字输出相同

#### Scenario: Exact token origins include repeated names
- **WHEN** 一个或多个直接数组元素被呈现为同类常量名，其物理指令位置与输出范围可以逐个唯一确定
- **THEN** 系统 MUST 为每个名称的确切token记录对应Field和该方法的指令来源，不得把整个return或数组范围冒充名称范围；无法唯一对应时保留原呈现

#### Scenario: Candidate ambiguity or lexical shadowing
- **WHEN** 候选字段表不完整、同值字段有歧义、名称被parameter/local/resource/catch等绑定遮蔽，或方法名称事实不完整
- **THEN** 系统 MUST 不提交受影响的数组名称替换及其来源记录，并保持数值与原运行语义

#### Scenario: Unsupported expression or competing body projection
- **WHEN** 返回形状为嵌套/非int数组、未完成store链、cast/算术元素，或当前正文已由其它呈现修改且不能证明仍是原恢复正文
- **THEN** 系统 MUST 保留不在本片准入范围内的元素及当前有效正文，不得扩大整数替换或覆盖已有派生呈现

#### Scenario: Stop and evidence independence
- **WHEN** 名称证明、范围恢复、输出或最终发布前遇预算停止/取消，或相同输入选择默认与完整证据
- **THEN** 系统 MUST 不发布半完成的方法或来源记录，准确传播停止；证据选择不得改变完整正文或常量准入，原switch名称恢复仍保持既有行为
