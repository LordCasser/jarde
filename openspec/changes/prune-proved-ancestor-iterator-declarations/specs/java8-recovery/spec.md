## ADDED Requirements

### Requirement: A committed iterator foreach removes only its wholly consumed ancestor declaration

系统 SHALL 在已成功恢复的 iterator foreach 完整消费某个局部变量、并且唯一无初值声明位于该循环可见的真实祖先作用域时，清除这条失用迭代器声明。系统 MUST 保持循环的全部效果、元素绑定、其他声明及后续使用关系；单个 SSA 值被消费不能代替整个局部变量失用的证据。缺证时 MUST 保留已有可靠循环和声明。

#### Scenario: Nested iterator has an unused ancestor declaration

- **WHEN** 完整方法中内层 iterator foreach 已证明，迭代器只在该循环的创建、条件与 next 访问使用，唯一祖先空声明确实只服务于该循环
- **THEN** 方法恢复文本及完整类源码 SHALL 不再包含该空声明，嵌套 foreach 及其他正文保持，零次迭代、异常、状态和事件顺序 SHALL 与原 class 一致

#### Scenario: Another value of the same local remains observable

- **WHEN** 同一个局部变量在循环外、另一个异常分支或循环前后还有读写，即使当前 iterator 值的 uses 已经闭合
- **THEN** 系统 MUST 保留该变量所需声明和所有其他读写，不把不同 SSA 值误当成不同声明，也不能为此撤回已经可靠的 foreach

#### Scenario: Slot reuse and sibling declarations remain distinct

- **WHEN** 相同槽号承载另一个已区分的局部变量，或存在同名兄弟分支声明、有初值声明、同来源锚点的其他节点或重复匹配
- **THEN** 系统 MUST 按实际变量、节点形态与词法范围区分；证据不唯一时保留相关声明，不删除另一个变量或分支的声明

#### Scenario: A speculative foreach is discarded

- **WHEN** 子循环候选成功后，其所在分支或整方法被既有恢复逻辑回滚、拒绝或替换为 bytecode quote
- **THEN** 该候选 MUST 不能授权删除最终输出中仍需保留的声明；只有最终实际存在的受证循环能够消费相应空声明

#### Scenario: Origins and output planes remain truthful

- **WHEN** 一条受证祖先空声明被清理，并以 default、all 或显式包含 SourceMap 的证据请求恢复同一输入
- **THEN** 各请求的代码决定 SHALL 一致，被消费指令来源仍可在对应循环找到；raw 方法 map SHALL 对应新 raw 方法文本并保持全部物理指令和合法 UTF8 区间，不能把旧坐标或方法 map 冒称为完整类投影 map

#### Scenario: Cleanup encounters a real execution stop

- **WHEN** 变量用途核验、最终作用域匹配或清理遇到预算、取消或期限停止
- **THEN** 系统 MUST 沿既有停止契约报告，不能转成普通缺证并继续发布半完成方法；成功发布的其他内容继续遵循既有操作停止和输出承诺
