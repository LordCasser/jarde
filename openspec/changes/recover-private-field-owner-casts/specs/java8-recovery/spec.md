## ADDED Requirements

### Requirement: 已证明的 Java 8 父类字段写入保留效果

当输入明确证明 receiver 的父类关系与字段 symbolic owner，系统 SHALL 保留对该父类字段的赋值效果。对唯一、精确且只更新同一 private 字段的 Java 8 accessor，系统 MUST 保留调用处的父类实参转换、写入效果和返回值消费；证据不完整时不得静默省略字段更新或宣称源码可编译。

#### Scenario: 子类 receiver 写入已证明的父类字段
- **WHEN** `B extends A`，`B.set` 的 field reference 精确指向 `A` 的 `visible` 字段，且字段描述符与写入值一致
- **THEN** 恢复源码保留该 field owner 和单次写入；重编后反射读取 `visible` 得到输入值

#### Scenario: private 字段经唯一 javac accessor 更新
- **WHEN** Java 8 类族中 `B.set` 唯一调用 `A.access$002(A, boolean)boolean`，该 helper 只更新 `A.hidden` 并返回写入值，且调用方丢弃该结果
- **THEN** 恢复源码保留一次 private 字段更新、已证明的 `B` 到 `A` 实参转换和结果消费；完整类族重编并验证后反射读取 `hidden` 得到输入值

#### Scenario: owner、继承关系或 accessor 不能证明
- **WHEN** 字段 owner/descriptor 与声明不符、`B extends A` 关系缺失、accessor 多目标，或 accessor 更新了其他字段/含额外效果
- **THEN** 系统拒绝该窄恢复，不重绑定字段 owner，不静默删除更新，并保留拒绝位置的来源引用
