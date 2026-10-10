## ADDED Requirements

### Requirement: Proved one-armed loop sequences are recoverable

当一个单臂条件区域的同次控制流与效果证据证明：非空直线前缀只进入该臂独占的普通自然循环，循环唯一正常出口直接到外层条件汇合点、或经该臂独占的直线尾部到汇合点，且循环主体已满足既有恢复前提时，系统 SHALL 恢复完整条件区域内的前缀、循环与尾部。恢复 MUST 保持条件极性、语句位置、各正常与异常路径上的求值次数、物理 block 唯一归属及可追踪来源；MUST NOT 因 walker 分段产生 continuation 而丢弃整个已证结构。

#### Scenario: Conditional iterator setup and loop

- **WHEN** `countEmpty(List<String>)` 先将计数置零，在非 null 条件内建立 iterator 并循环处理元素，循环后在条件外返回计数，所有控制流和主体前提均闭合
- **THEN** 输出 SHALL 保留条件内的 setup 与完整循环，完整生成类 SHALL 能原样重编并验证运行，null、空集合、混合元素、重复调用及 null 元素行为 MUST 与原 class 相同

#### Scenario: Plain while prefix and reversed predicate

- **WHEN** `prefixWhile` 或 `takenArm` 在单臂条件内先初始化循环局部变量，再执行普通 while，循环唯一出口到条件后的返回
- **THEN** 两种条件极性的输出 SHALL 保持初始化与循环都在原臂内，e=true/false、n=0/1/4 的完整类行为 MUST 与原 class 相同

#### Scenario: Straight tail after the loop

- **WHEN** `loopAndTail` 在同臂循环后还执行一次 `sum += 100`，该尾部无其他入口且只到外层 join
- **THEN** 输出 SHALL 将尾部保留在原条件内并置于循环之后，零轮和多轮情况下各执行一次，跳过条件时不执行

#### Scenario: Ownership or exit is not proved

- **WHEN** prefix/header/body/tail 存在外部入口、循环出口不一致、已被其他区域持有、越过父 scope/共享边界，或 continuation 实为外层循环的回边/continue 目标
- **THEN** 系统 MUST 保留准确拒绝与完整引用来源，MUST NOT 重新进入该循环、丢弃同臂尾部或发表重复拥有物理 block 的结构

#### Scenario: Existing statements and origins remain observable

- **WHEN** 同一完整类以默认与完整 evidence 请求恢复
- **THEN** 输出正文及每个物理方法的正文与来源 SHALL 一致；已有成功的无前缀循环 SHALL 保持其语义，任何已知来源缺口 MUST 作为缺口记录，不得以 structured 标志代替来源验收

#### Scenario: Budget or cancellation interrupts continuation

- **WHEN** 分支续接、边/所有权证明或发射受预算、取消或递归停止约束中断
- **THEN** 系统 MUST 遵守既有 Stop 契约，不发表部分正文或来源，也不将停止改为普通结构失败或完整成功
