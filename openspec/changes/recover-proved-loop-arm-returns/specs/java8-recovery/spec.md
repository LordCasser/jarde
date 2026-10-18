## ADDED Requirements

### Requirement: A proved conditional loop arm preserves its terminal returns

系统 SHALL 将条件臂中已证明闭合的普通循环及其终止return叶恢复为结构化Java，保留条件前缀、正常汇合、循环更新及方法返回各自的执行位置。恢复 MUST 保持完整物理方法和来源，不得把终止return当作普通循环出口、遗漏语句，或仅因固定输入未走到返回分支而删除它。

#### Scenario: The unchanged nested-loop source keeps both returns
- **WHEN** 完整原始TestNestedLoops5的debug/no-debug输入经standalone或archive-child、default或all配置恢复
- **THEN** 目标方法结构化，内层早返回与正常尾部返回均保留，不凭空增加continue，完整三个方法均可观察且所有物理BCI有同方法来源

#### Scenario: An early return skips only the remaining effects
- **WHEN** 参数化有限嵌套循环在内层第一次或先完成外层尾部后执行早返回，或正常完成多次迭代
- **THEN** 返回值、返回前效果、被跳过的尾部效果及同实例累计状态逐字等于对应原class的独立JVM观察

#### Scenario: A method exit remains distinct from the ordinary join
- **WHEN** 循环臂只有一个真实普通汇合出口并含已证明的终止return叶
- **THEN** 返回路径终止当前方法，普通完成路径准确回到条件汇合块，循环更新转移仍指向原循环并保留其物理来源

#### Scenario: Unproved ownership cannot authorize the arm
- **WHEN** 返回叶共享入口、缺失或错误的返回证据、额外普通出口、异常边、clone身份、重复所有者或latch来源使该循环臂不能完整证明
- **THEN** 该候选保持拒绝，不将不闭合结构作为完整Java或完整覆盖发布；已由其他独立证明正确恢复的输入不被强制全局降级

#### Scenario: Real proof stops are retained
- **WHEN** 新增证明工作中预算耗尽、取消或期限到达
- **THEN** 消耗与原停止原因如实保留，停止不被当作无模式继续尝试，不发布部分正文或来源映射

#### Scenario: Full source and physical identity define acceptance
- **WHEN** 对目标和补充完整源码进行原/JADX/Jarde比较
- **THEN** 原样保留全部方法和生成源码，以真实编译产物、隔离CodeSource及可用的双JVM运行证明语义，逐一核验default/all、物理方法身份和UTF8来源范围；依赖版本造成的工具边界、编译失败与未执行运行分别记录
