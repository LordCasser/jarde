## ADDED Requirements

### Requirement: Bounded compound statement conditions

系统 SHALL 在单入口有限条件图可被证明为保序短路条件且每个物理测试只出现一次时，恢复一个复合条件及其普通语句分支。所有路径 MUST 被已证明的两个出口覆盖，真实边方向、条件读取及副作用的执行次数与顺序 MUST 保留。输出 MUST NOT 通过复制测试、payload、return 或后续语句来消除共享节点；无法满足这些条件时 SHALL 有界拒绝。

#### Scenario: Shared condition suffix before a statement

- **WHEN** 两个条件路径共享后续测试，整体可保序归约为 `(a && b) || (c && d)`，成功路径进入一个共同 payload，失败路径进入共同续接
- **THEN** 输出一个复合条件，四个物理测试各一次，payload 与续接各一次，并且完整生成类编译和执行结果与原 class 一致

#### Scenario: One shared void return after a nested compound statement

- **WHEN** 已证明普通If包围受支持CompoundIf，二者的共同续接是同一路径/frame内fresh单一void return，所有入边准确来自该arm的唯一物理owner，且没有异常、重复、外部入口或跨case边
- **THEN** 将该return消费一次并保留在包围arm内，条件和payload不拥有它，不穿过外层frame边界；无法完整证明或发生Stop时不遗留新增owner或部分产物

#### Scenario: Conditional effects and floating comparison

- **WHEN** 支持范围内的测试含可观察调用、字段读取、可能抛异常的操作或浮点比较
- **THEN** 每条路径上的调用与读取序列、异常发生位置及 NaN 分支行为与原 class 相同，不因条件反转或合并而提前、重复或省略求值

#### Scenario: Scope or physical edge cannot be proved

- **WHEN** 候选包含未证明的外部入口、重复或非普通控制流边、循环、clone-path 变化、跨 case/frame/scope 边界、独立中间语句或不可安全归约的共享条件
- **THEN** 保留可靠拒绝及完整物理来源，不输出宣称结构化的部分复合条件，也不遗留已消费却未呈现的 block

### Requirement: Compound condition origins and atomic stops

复合条件及其分支 SHALL 绑定准确的物理方法、成员序号与原始指令来源，条件测试、payload 与续接的归属 MUST 可独立核验。新增分析与构造 SHALL 使用请求现有预算和取消 token，Stop MUST 保留真实原因、维度、位置及实际 usage，并保持部分源码和 map 原子缺失。

#### Scenario: Actual origins after nested switch recovery

- **WHEN** 已证明的 switch 续接包含受支持复合条件，生成完整方法及类
- **THEN** 所有真实测试与 transfer/terminal 指令均有准确物理来源，条件、case body、共享续接没有重复 block owner，default/all 配置的正文与 map 一致

#### Scenario: Stop during a new proof or emission

- **WHEN** 新增条件扫描、归约、arm 恢复或表达式构造实际触发预算或取消 Stop
- **THEN** 返回原始 Stop 及准确位置和 usage，不提高预算、不将其改成一般拒绝，且没有部分 binding、正文或 source map
