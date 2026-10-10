## ADDED Requirements

### Requirement: Proved single-receiver int field multiplication is recoverable

当同次字段与值流证据证明非静态 `int` 字段在同一基本块内读取一次、执行准确整数乘法、写回同一位置，且读写 receiver 是一次求值产生的准确复制、旧值和乘积均没有额外消费者时，Java 8 恢复 SHALL 能输出一次 `*=` 语句。输出 MUST 保持 receiver 与 RHS 的求值次数、读写和异常顺序、整数溢出语义，以及全部物理字段与指令来源。字段同名或接收者呈现相同文本 MUST NOT 代替位置身份与效果证据。

#### Scenario: Nested int field multiplication

- **WHEN** 完整类族中的实例方法只执行 `this.a.f *= n` 后无值返回，`f` 是准确 int 字段，receiver、复制、读、乘、写及 RHS 依赖满足同次完整证明
- **THEN** 输出 SHALL 保留嵌套 receiver 并写出 `*=`，完整生成类族 SHALL 能原样 Java 8 重编并在验证运行中得到与原程序相同的字段结果

#### Scenario: Distinct explicit receiver reads remain distinct

- **WHEN** 显式字段赋值的读 receiver 和写 receiver 来自两个不同字段读取，而本次证据没有证明它们是同一次求值的复制
- **THEN** 本片 MUST NOT 将其合并为复合赋值，已有普通赋值恢复和每次读取的来源 SHALL 保留

#### Scenario: Update proof does not close

- **WHEN** 读写成员或 receiver 身份不一致、目标不是 int 实例字段、复制/旧值/乘积有额外消费者，或者依赖区间/指令顺序不满足证明
- **THEN** 本片 MUST NOT 发布乘法复合赋值，原投影或可定位拒绝 SHALL 保留所需生产者、效果和物理来源

#### Scenario: Evidence requests do not alter semantics

- **WHEN** 同一原始类族分别以默认与完整 evidence 请求恢复
- **THEN** 生成正文及每个物理方法的正文与来源 SHALL 恒同，额外证据记录与预算使用量可按请求不同

#### Scenario: Proof or emission stops

- **WHEN** 证明或发射遇到预算/取消停止
- **THEN** 系统 MUST 遵循现有 Stop 契约，不发布半条更新或半份来源，不将停止解释为成功的乘法恢复
