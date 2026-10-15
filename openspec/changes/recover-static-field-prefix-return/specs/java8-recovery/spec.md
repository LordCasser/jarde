# Spec Delta

## ADDED Requirements

### Requirement: A proved static int prefix update returns its new value

系统 SHALL 恢复可证明的本类非final、非volatile静态int字段加一后直接返回新值，在完整无异常处理单块内保持同一字段身份、一次读取、一次写回及相同新值的返回。输出 MUST 保持int溢出、求值顺序与物理来源，MUST NOT 以写回后再次读取字段代替原返回值。无法证明时遵循既有拒绝契约，实际预算或取消停止保持原执行语义。

#### Scenario: A static increment returns the computed sum

- **WHEN** 方法完整地读取已证明的本类可更新静态int字段，加一后将同一新值写回并直接返回，没有其它消费者、插入效果或异常范围
- **THEN** 输出 SHALL 呈现该次前置更新的新值返回，字段读取、写入和返回分别执行一次；初值-1及7分别返回并保存0及8，初值MAX_VALUE返回并保存MIN_VALUE，不得返回旧值、常量或另一次字段读取

#### Scenario: A saved return precedes the finally effect

- **WHEN** 完整FO的 `evalThenFin` 先调用上述 `bump` 并保存返回值，随后finally把字段置为100
- **THEN** 完整恢复源码 SHALL 保持原有return/finally顺序，初值7得到8/100，初值MAX_VALUE得到MIN_VALUE/100；固定main初值0的输出为1/100。MUST NOT 通过删除辅助方法、改原Java或oracle、仅编译方法片段来通过验收

#### Scenario: Field identity and value consumers must be proved

- **WHEN** 读写字段的物理声明、owner/name/descriptor/staticness或可更新性不能闭合，或写回与返回并非同一新加法值、存在额外alias消费者/插入效果/跨块或handler范围
- **THEN** 本能力 MUST NOT 将候选呈现为已证明的前置更新；受影响范围保留可定位的bytecode与物理来源，不能只比较显示字段名、猜测目标或认领部分链。其它既有规则能正确恢复的形态继续由其原契约处理

#### Scenario: Origins retain the read and the returned write

- **WHEN** 上述静态前置更新经公共库或JSON CLI恢复
- **THEN** 正文与source map SHALL 保留同一物理方法的字段读、常量、加法、复制、写回和return锚点；复制或算术不能因被折叠而丢失来源，字段producer与写入/返回consumer身份不得串用

#### Scenario: A stop commits no partial update

- **WHEN** 候选证明或发射期间观察到本次真实预算、取消或期限停止
- **THEN** 结果 MUST 保持既有停止原因、位置与真实usage，不发布部分前置更新正文或source map；不得将停止当普通缺证后继续后继构建，也不得用入口预取消代替证明期停止验证

#### Scenario: Existing updates keep their own returned values

- **WHEN** 同时运行既有实例字段前置/后置更新、静态postfix旧值消费者及无值字段单位更新
- **THEN** 原更新的返回值、求值次数、拒绝边界及物理来源 SHALL 保持；本片不得将旧值返回改成新值，也不得把减一、窄/宽型或不满足原条件的字段纳入本能力

#### Scenario: Whole source execution proves recovery

- **WHEN** 受控javac23 --release8与真实javac8的FO分别经最终冻结CLI恢复，并将每份完整生成Java原样重编后在Java8/23运行
- **THEN** 固定main与独立driver的非零/溢出/return-before-finally结果 SHALL 与对应原class逐项相同；rebuilt classpath不得混入原class/JAR，不允许改生成文本或删方法。CLI exit0、Complete或语法/质量平面不能替代实际重编与运行证据
