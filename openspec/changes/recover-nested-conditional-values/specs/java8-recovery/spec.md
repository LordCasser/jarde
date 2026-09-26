## ADDED Requirements

### Requirement: Proven nested conditional stack values recover as one expression

当一个有界的嵌套条件树的所有可达叶子各自产生一个值，且这些值恰好流向同一栈汇合值及一个已支持的消费者时，系统 SHALL 将整棵树恢复为 Java 嵌套条件表达式。恢复 SHALL 依据真实控制流、值来源、静态类型和消费位置；不能证明整棵树时 MUST 保留完整来源并走保守回退。

#### Scenario: Three leaves and a final return

- **WHEN** 外层条件的一个分支继续测试内层条件，三个叶子分别产生同型值，最终由同一个 return 消费
- **THEN** 系统 SHALL 输出可重编译的嵌套条件值；外层测试先于内层测试，且内层测试只在对应外层分支执行，三个叶子中恰有选中的一个求值一次

#### Scenario: Deeper tree and other supported consumers

- **WHEN** 多层嵌套条件区域在既有深度与预算内获得完整值流证明，最终汇合值由已支持的局部赋值、算术、调用实参或字段写入消费
- **THEN** 系统 SHALL 在消费位置保持原有求值顺序、目标类型与调用目标，不重复执行叶子生产者；不能证明类型或目标时 MUST 拒绝该恢复

#### Scenario: Effects, exceptions and origin

- **WHEN** 外层/内层测试及叶子分别记录效果或抛出可区分异常
- **THEN** 重编译输出的返回值、效果顺序、调用次数及异常类别和消息 SHALL 与原 class 一致；完整证据 SHALL 能定位测试、叶子、跳转与消费者的真实字节码来源

#### Scenario: Unproved tree remains conservative

- **WHEN** 条件树存在外部跳入、循环或异常处理器边、叶子之外的不可折叠语句、不能唯一映射的汇合输入、多个独立消费者，或预算/取消阻止完整证明
- **THEN** 系统 MUST NOT 发布部分恢复的嵌套条件值，也 MUST NOT 把任何未证明的多前驱汇合猜成 `?:`；若无另一条已证明的恢复路径，SHALL 保留相关测试、生产者、跳转与消费者的来源及既有拒绝/停止状态
