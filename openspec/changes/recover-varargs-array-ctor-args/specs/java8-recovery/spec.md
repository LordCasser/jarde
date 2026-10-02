## ADDED Requirements

### Requirement: 构造实参位的 varargs 内联数组可呈现

系统 SHALL 在外层构造的实参位出现完整内联匿名数组存储链（`anewarray` 起始、元素生产与既有 varargs 内联数组判据同源、区间连续、值单用途为该实参）时，证明该链并将其呈现为外层实参位的 `new T[]{…}` 数组初始器。裸调用位、嵌套构造实参位与普通调用实参位 SHALL 逐字不变；链不完整、双用途或跨块 SHALL 保持既有拒绝。

#### Scenario: 集合拷贝构造接 varargs 工厂恢复

- **WHEN** `new ArrayList<Integer>(Arrays.asList(1, 2, 3))` 及其下游消费链三方 Java 8 重编运行
- **THEN** 构造与数组初始器实参完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 既有位与负例不变

- **WHEN** 输入为裸 varargs 调用、嵌套构造实参、普通调用实参，或链中插语句/数组双用途
- **THEN** 前三者与本变更前逐字一致；后者保持既有拒绝
