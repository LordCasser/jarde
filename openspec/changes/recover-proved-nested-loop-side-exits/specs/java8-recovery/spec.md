## ADDED Requirements

### Requirement: Proved nested loop exit paths remain in their Java body

系统 SHALL 恢复能够完整证明归属的嵌套循环退出路径，使其前缀、内层循环、早返回与准确break保持原class的执行位置、次数和目标。退出路径即使不走本循环回边，也 MUST 不被截成空分支后作为完整结构发布；无法证明时 MUST 保留准确拒绝和完整物理来源。

#### Scenario: A nested loop ends by breaking its containing loop

- **WHEN** 一个循环正文分支通过独占前缀进入内层循环，内层正常结束后通过独占跳转退出本循环，且该路径的全部入口、出口和所有者均可证明
- **THEN** 系统 SHALL 输出完整内层循环及指向正确包含循环的break；原固定TestNestedLoops4对应完整源码重编运行同原返回12，外层条件后的语句只执行其原有次数，不丢弃退出分支或重复正文

#### Scenario: The nested body returns before reaching its break

- **WHEN** 内层循环的一条独占比较分支终止方法，而其他路径保持循环或到达准确break
- **THEN** 输出 SHALL 保留该return及其准确求值点；可到达该分支的受控真实输入重编执行与原class一致，不能以固定方法中未运行到该分支的正常结果替代验收

#### Scenario: A prefixed loop arm meets an enclosing condition

- **WHEN** 外层条件的一臂直接到达join，另一臂含独占前缀和上述完整循环，所有正常出口都到达同一join，其他已证明路径终止方法
- **THEN** 系统 SHALL 完整呈现该条件臂，并将join及其后续语句放在条件之外一次；循环或return路径不得被当作空臂，也不得在两个位置认领同一物理块

#### Scenario: Evidence and complete source describe every physical instruction

- **WHEN** 同一冻结输入以default/all恢复并输出完整类源码
- **THEN** 两配置 SHALL 保持正文与源码映射一致，每条物理指令含正确owner/name/descriptor与合法UTF8来源范围，循环gateway/回边/return均可追溯；完整类原样双JDK重编运行同原，不以局部片段或原class运行代替生成源码验收

#### Scenario: A side path has an unproved entry or exit

- **WHEN** 退出路径有额外正常或异常入口、额外未知出口、不可约或跨路径嵌套、共享return/gateway、错误break目标或重复所有者
- **THEN** 系统 MUST 拒绝新增结构准入，保留完整相关物理范围与准确原因；不能仅扩大可达scope、改变自然环集合或忽略额外边获得Structured结果

#### Scenario: The nested exit proof is stopped

- **WHEN** 退出路径证明或其正文/外层接续消费触发预算或取消
- **THEN** 系统 SHALL 返回准确Stop及其位置与用量，MUST 不发布半个正文、来源映射或未完成循环所有权；失败候选不得改变同请求后续拒绝结果
