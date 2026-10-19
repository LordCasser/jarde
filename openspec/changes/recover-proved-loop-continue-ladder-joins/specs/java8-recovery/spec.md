## ADDED Requirements

### Requirement: Proved loop-local joins preserve direct continue and conditional break

系统 SHALL 在循环更新与分支完成路径均被证明时，恢复普通循环内正常分支的局部汇合，并保留绕过该汇合的直接 continue 和后续条件 break；无法闭合的路径 MUST NOT 通过本规则伪造为可执行的结构化源码。

#### Scenario: Equality continues before the later break test
- **WHEN** 完整输入的循环在小于/大于比较后分别修改数组、相等时直接继续，只有前两种情况才执行后续条件 break，且所有路径证明闭合
- **THEN** 恢复源码 SHALL 保持相等路径绕过后续break测试、其余路径先修改再测试；每条继续路径 SHALL 执行一次循环更新，break路径 MUST NOT 执行该更新

#### Scenario: Post-loop effects and exceptional paths remain faithful
- **WHEN** 输入还含循环后的字段更新，并以有限用例触发正常遍历、continue后break、空数组、null数组和可达整数环绕
- **THEN** 完整源码重编后的数组、字段、异常类别及stdout SHALL 与原件相同，异常发生前后的副作用边界 MUST NOT 改变

#### Scenario: Metadata and evidence selection preserve the conclusion
- **WHEN** 相同方法分别以debug/no-debug以及class/plain JAR的default/all请求恢复
- **THEN** 目标方法 SHALL 获得相同的结构化恢复结论；证据选择 MUST NOT 改变同一输入的Java全文、方法身份或来源映射，来源anchor MUST 指向正确物理方法的有效指令

#### Scenario: Unproved routing preserves refusal
- **WHEN** 候选分支含无法证明的额外入口、旁路、更新跳过、重复所有权、未知克隆或异常转移
- **THEN** 系统 MUST NOT 依此规则把它伪造为局部汇合或continue，MUST NOT 静默丢弃任何路径或效果

#### Scenario: Proof stops retain atomic publication
- **WHEN** 新证明遇到真实预算耗尽或取消
- **THEN** 系统 SHALL 保留准确停止原因与实际用量，MUST NOT 发布未完成的声明、分支正文或来源映射，也 MUST NOT 将停止降为成功
