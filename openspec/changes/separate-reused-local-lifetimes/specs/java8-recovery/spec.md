## ADDED Requirements

### Requirement: Proven disjoint lifetimes of one JVM local slot remain distinct in Java source

当同一 JVM 局部槽先后存放类型不兼容的两个值，而完整的定义与用途证明表明它们属于不相交的源码生命周期时，Java 8 恢复结果 SHALL 为两段分别写出类型正确的局部变量、名称和声明。恢复 MUST 保留每条原指令的来源、原 class 的值/副作用/异常行为及已证明的循环形态。

#### Scenario: Array loop temporary followed by integer local with debug information

- **WHEN** javac 将数组增强 `for` 的合成 `int[]` 别名放在槽 2，循环后又将有 LVT 名称的 `int first` 放在同槽，且两段没有相交的定义/用途
- **THEN** 完整类源码 SHALL 把数组别名与 `first` 写为不同类型的局部，后者保持可证明的原名；`javac --release 8` 重编后的 runner 在正常、空数组和 null 输入上 SHALL 与原 class 的值及异常相同

#### Scenario: Same class without debug information

- **WHEN** 同一字节码的 LVT 被移除，但类型不兼容、用途不相交和控制流次序仍由 class 自身证明
- **THEN** 完整类源码 SHALL 使用稳定且互不冲突的合成名称区分两段，并经 Java 8 重编及执行对照；不得仅因缺失 LVT 就把数组与整数写成同一变量

### Requirement: Unproved slot separation cannot publish a contradictory declaration

一个槽的定义/用途若通过非平凡跨段 phi、从后段返回前段的回边、异常区域或未识别的访问相连，系统 MUST NOT 把它们仅凭 BCI 大小或不同帧类型拆为两个可执行局部；若保留一个变量又会把已证明不兼容的引用与原始类型写入同一 Java 声明，完整源码 SHALL 报告无法证明的区域，不得声称该区域已恢复为可编译方法。前段内部的平凡自 phi 与互斥分支各自完整的值链可以通过独立证明。相同类型的普通连续赋值仍 SHALL 维持既有语义及稳定输出。

#### Scenario: Connected or ambiguous writes

- **WHEN** 同槽两个写入跨越一个可能读取两者的 phi、从后段回到前段访问点的循环回边或异常后继，或者某次访问无法归入唯一生命周期
- **THEN** 恢复 SHALL 拒绝该分段并保留可查来源/未恢复标记，不得发布一个把 `int` 赋给 `int[]` 的已恢复正文，也不得借调试表名字猜测执行路径

#### Scenario: Budget and evidence selection

- **WHEN** 生命周期证明期间预算耗尽或请求被取消，或对相同已证明输入分别请求 essential/all 证据
- **THEN** 停止请求 SHALL 沿用现有停止契约而不发布半份分段；两个成功请求的 Java 正文 SHALL 相同，每个变量的写入、读取与被投影循环的原 BCI SHALL 可追溯
