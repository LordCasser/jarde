## ADDED Requirements

### Requirement: 已证单出口循环臂可以在内层 if 的局部 join 续接

当内层 `if` 的一个臂是完整且只有一个正常出口的循环、另一臂是直线区域，且两个臂的实际出口和 canonical 入边都唯一汇于同一局部 join 时，系统 SHALL 在外层 arm 内继续走访该 join 到父边界的已证直线尾部；MUST 保持各 canonical block 唯一归属、来源完整及执行顺序。

#### Scenario: 内层循环后仍有外层 arm 语句

- **WHEN** `LoopIfJoin.run(ZZ)I` 的 BCI 6 内层 `if` 一臂在 BCI 10–12 跳到 26，另一臂是 BCI 15–23 的单出口循环，BCI 26 执行 `value += 10` 后再到外层 BCI 34
- **THEN** Jarde SHALL 恢复完整的嵌套 `if`、循环和仅执行一次的后续加法，且原/JADX/Jarde 完整 Java 8 类的 `-Xverify:all` 输出 MUST 均为 `4 / 13 / 11`

#### Scenario: 不能证明唯一循环出口或 join

- **WHEN** 循环臂有额外 break/return、异常或子程序边，join 有第三个入口，循环出口与直线臂目标不同，或尾部已被另一 Region 认领
- **THEN** 系统 MUST 保留保守引用/解释，不能丢弃尾部、重复拥有 block 或把局部 slot 拒绝直接当作放宽结构的理由

#### Scenario: 有界分析中断

- **WHEN** 出口、入边或尾部所有权证明受预算或取消中断
- **THEN** 系统 MUST 返回不完整/中止状态，不得发表只含内层循环而缺少外层后续语句的完整方法
