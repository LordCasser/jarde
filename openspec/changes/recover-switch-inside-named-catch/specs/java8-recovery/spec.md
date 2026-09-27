## ADDED Requirements

### Requirement: 已证明的 switch 外层具名 catch 恢复

系统 SHALL 在 Java 8 方法的具名异常行完整保护整数 switch 的 dispatch 和各 case 正文、且正常出口经唯一无效果 transfer 到后续语句时，输出语义等价的 `try { switch (...) { ... } } catch (T e) { ... }`。系统 MUST 保留字段前缀、每个 case 的调用次数、handler 派发顺序和后续返回。

#### Scenario: 完整保护的三臂 switch
- **WHEN** 一个已证明的字段赋值位于具名保护范围前，同一范围完整覆盖 switch 的三个 case，所有正常出口汇到范围外唯一 transfer，catch handler 也流向同一后续返回
- **THEN** 系统输出字段赋值、外层 try/switch/catch 与后续 return，完整 Java 8 类在正常 case、default 和异常路径与原 class 的结果及效果顺序一致

### Requirement: 资源候选与异常范围的保守边界

系统 MUST 仅在前缀确为完整且栈闭合的字段赋值时，将该前缀排除为 TWR 资源头；真实或未证明的资源头不得被降格成具名 catch。异常范围未覆盖整个 switch 时，系统 MUST 保留原物理边界而不得输出声称覆盖所有 case 的外层 catch。

#### Scenario: 真正的资源头或不完整字段赋值
- **WHEN** 保护范围前有 local resource `Store`、Java 9 resource copy，或者字段赋值的语句/栈消费无法证明闭合
- **THEN** 系统沿用现有 TWR 证明/拒绝或保守回退，不因本变更跳过 guarded rule 而输出普通 catch

#### Scenario: 仅部分 case 受保护
- **WHEN** 具名异常行只覆盖一个或部分 case 调用，而不完整覆盖 dispatch 与全部 case 正文
- **THEN** 系统不把该行提升为覆盖整个 switch 的 catch，保留可追溯的拒绝或低级结构

### Requirement: 出口 transfer 的唯一归属与来源

系统 SHALL 仅在 transfer 是异常范围 exclusive end 的完整单指令块、唯一后继为已证明的 try 后续位置、所有普通前驱由该 try 正文拥有且无 handler 竞争入口时认领它。每个物理块和指令 MUST 只有一个结构所有者或带位置的安全拒绝，预算/取消或 Builder 失败不得发布半份恢复结果。

#### Scenario: 唯一普通出口
- **WHEN** 所有 switch arm 经唯一纯 goto 块到达 try 后 return
- **THEN** 系统在源码词法顺序中省略该 goto 文本，同时将其物理 BCI 记入来源，且 switch arm、catch handler、goto、return 均只归属一次

#### Scenario: 出口图不闭合
- **WHEN** 该块有额外前驱、非唯一后继、异常 handler 入口、额外效果或跳向不同 continuation
- **THEN** 系统不吸收它，不把未覆盖块静默省略，并保持原子 fallback/stop
