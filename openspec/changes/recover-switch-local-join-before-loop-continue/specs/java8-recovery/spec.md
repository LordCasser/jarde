## ADDED Requirements

### Requirement: 已证循环内 switch 的局部汇合与 continue 各有唯一归属

对于外层循环中的 Java 8 整数 switch，若至少两个 case 的正常路径在唯一 switch-local join 汇合，另一个 case 从自身路径精确到达该循环已证 `continue_target`，系统 SHALL 分别恢复 switch-local 完成和 loop continue；MUST 保证共享语句、循环更新及每个物理 block 的唯一归属，不得在终止的 continue 后输出 switch break。

#### Scenario: CF-13 的共享语句与循环更新

- **WHEN** case 0/default 到 BCI 55 的共享语句，case 1 经 BCI 49 直接到外层循环更新 BCI 58，且更新回到 loop header BCI 4
- **THEN** 完整 Jarde Java 8 类 SHALL 重编，`java -Xverify:all` runner SHALL 输出 `38`，共享语句只在未 continue 路径执行，更新每次迭代只执行一次；BCI 9/12、49、55、58 的来源和 Region owner MUST 唯一且对应正确语句

#### Scenario: continue 后不得追加 switch break

- **WHEN** 一个 switch arm 已被证明终止于当前循环的 `continue_target`
- **THEN** 该 arm SHALL 输出 loop `continue`，后续不得有不可达的 `break;`；其他到局部 join 的 case 可以正常结束 switch，并由 switch 后 continuation 执行共享语句

#### Scenario: 局部汇合或出口不唯一

- **WHEN** 存在嵌套 switch、额外入口、多个不可比较的汇合点、不能证明的跨 case/循环出口，或一个普通 loop 尾部路径只是在更新块自然汇合
- **THEN** 系统 MUST 保留现有引用/解释或既有安全恢复结果，不得把更新块猜成 case 的 continue，不得重复认领或丢弃物理 block

#### Scenario: 有界证明被中断

- **WHEN** 局部 join 或 case transfer 证明中耗尽预算或收到取消信号
- **THEN** 结果 MUST 是现有中止/不完整状态，不能发表只恢复部分 case 的完整源码
