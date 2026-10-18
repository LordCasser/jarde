## ADDED Requirements

### Requirement: A closed single-exit loop keeps statements before its test

系统 SHALL 恢复有完整结构证据的单入口、单出口自然循环：每轮在循环头执行语句、随后判断是否退出，其余循环体为返回该头部的直线链。输出 MUST 保持头部语句在首次、每次回边及最终退出判断中的执行次数和顺序，正常退出后继续原后继，不把语句移至循环外或仅在条件成立后执行。

#### Scenario: Two unchanged sequential loops retain their continuation
- **WHEN** 完整原始TestSequentialLoops经debug/no-debug、standalone/archive-child及default/all配置恢复
- **THEN** 两个循环及其间物理转移、最终return均保留，完整方法可原样重编，有限正常和整数溢出输入与原class返回相同结果

#### Scenario: Header calls preserve effects and exceptions
- **WHEN** 头部调用记录次数与顺序，在首次、回边后或第二循环最终判断时正常返回或抛出原异常
- **THEN** 生成源码的返回值、调用ledger、异常身份、异常前可观察字段及同实例后续累计状态与原class相同；抛出时未完成的赋值不得提前生效

#### Scenario: State-dependent tests can exit without entering the body
- **WHEN** 已证明的头部调用根据状态使退出条件在首次判断或某次回边后成立
- **THEN** 头部效果仍执行一次，跳过的循环体无效果，循环后的后继按原路径执行；原始纯算术路径不可达的边界不得冒称被原输入覆盖

#### Scenario: Unclosed owners cannot justify a loop
- **WHEN** 存在额外入口/出口、非普通canonical边、错误clone身份、未闭合回边、重复owner、缺失物理分支或SSA证据，或候选依赖未证明的异常/嵌套归属
- **THEN** 本候选拒绝并保留可靠降级表示，已由原有独立证明支持的结构继续遵循其原契约，不把未知边丢弃以取得覆盖

#### Scenario: Pure test loops retain their established contract
- **WHEN** 原有条件表达式证明足以恢复while、for或do-while
- **THEN** 头部语句消费者不得放宽纯条件约束或改变已有正确投影，不能通过把条件中未证明的效果当作纯值隐藏

#### Scenario: Proof work retains real execution stops
- **WHEN** 新增闭合扫描或输出过程中预算耗尽、取消或期限到达
- **THEN** 原停止原因及真实用量保持，不被当作无模式继续尝试，不发布部分正文、证书或source map

#### Scenario: Whole-source comparison defines acceptance
- **WHEN** 比较原始、JADX与Jarde生成的完整类
- **THEN** 保留全部方法、字段、物理身份、同方法全BCI与UTF8来源，生成Java原样双编译器重编；成功才用隔离classpath/CodeSource双VM对真实原oracle验收，失败与工具边界如实单列，不删除失败成员或修补生成源码
