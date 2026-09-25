## ADDED Requirements

### Requirement: A proved do-while body branch keeps its iteration semantics

唯一闩锁测试及循环体内分支的归属、汇合与跳转目标均已证明时，系统 SHALL 恢复可编译的 `do-while`，保持每轮体语句、条件求值及循环后的语句各自原有执行次数和顺序。内部分支的字节码形状不得仅因它位于循环头块就被当成第二个循环测试。

#### Scenario: A body branch skips to the latch

- **WHEN** 循环体内一条已证明的条件路径直接到达该循环唯一闩锁，另一条路径执行剩余体语句再到同一闩锁
- **THEN** 完整源码 SHALL 可编译，跳过路径不得执行被跳过的语句；允许写成等价的单臂 `if` 或 `continue`，每轮闩锁条件只求值一次

#### Scenario: A body branch exits this loop

- **WHEN** 循环体内一条已证明的条件路径经普通转移到达此层循环唯一出口，另一条路径执行剩余体语句及闩锁测试
- **THEN** 完整源码 SHALL 在该路径写出作用于此层循环的 `break`，不得在提前退出时执行剩余体语句或闩锁条件，出口之后的返回或语句只呈现一次

#### Scenario: A side-effecting latch stays at the latch

- **WHEN** 唯一闩锁条件读取一个每次调用都有可观察副作用的值生产者
- **THEN** 完整源码 SHALL 保留原 class 每次条件求值的调用次数、返回值及异常顺序，不得把条件移到体前或复制到提前退出路径

### Requirement: Unproved loop transfers remain explicit gaps

只有本层循环的唯一闩锁或出口可由同一正常流证明时，系统 SHALL 认领相应转移；不能证明的跳转、跨层退出或被其它结构截获的 `break` SHALL 留下包含真实分支、转移和必要生产者位置的拒绝证据，不得产生看似完整但执行路径不同的源码。

#### Scenario: A target is not the proved latch or exit

- **WHEN** 循环体一条转移的目标既不是该循环已证明的唯一闩锁，也不是已证明的此层唯一出口
- **THEN** 系统 SHALL 保留来源完整的拒绝，MUST NOT 把它猜成 `break`、`continue` 或 `return`

#### Scenario: Sources and bounded output agree

- **WHEN** 同一已证明的 `do-while` 分别请求默认与完整来源证据
- **THEN** 两种正文 SHALL 一致，分支、体语句、显式转移、闩锁及出口的真实 BCI SHALL 可追溯；预算或取消不足时 SHALL 经既有停止合同结束
