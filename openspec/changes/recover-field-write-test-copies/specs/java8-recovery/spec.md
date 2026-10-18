## ADDED Requirements

### Requirement: A proved instance int field assignment test compares its original value

系统 SHALL 恢复已完整证明的this实例int字段赋值后直接整数比较：右值只求值一次，字段写入发生在右值正常完成后，比较读取复制的原值。输出 MUST 保留正常与异常路径的可观察效果顺序及全部物理来源，不以字段重新读取替代原值。

#### Scenario: A stateful call feeds the field write and test once
- **WHEN** 同块复制的int右值分别且唯一到达实例字段写入与直接比较，右值调用记录ledger或按调用序号抛出异常
- **THEN** 完整生成源码原样重编后调用次数、顺序、字段、比较结果与异常身份同原class，抛出时未完成的字段赋值不生效

#### Scenario: A volatile field write does not introduce another read
- **WHEN** 相同已证形态写入volatile int字段并比较复制的值
- **THEN** 生成比较读取该次保存的原值，不增加读取该字段的操作

#### Scenario: Unproved copies keep their refusal
- **WHEN** 存在额外或错误消费者、跨块值、缺失身份、不同接收者、非int字段或受保护异常范围
- **THEN** 恢复保留拒绝与来源，不把部分证明发布为完整源码

#### Scenario: Stopped recovery publishes no partial result
- **WHEN** 候选证明或输出遇到预算、取消或期限停止
- **THEN** 系统保留真实停止原因且不发布部分源码或map
