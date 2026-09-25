## ADDED Requirements

### Requirement: 已证明左值的后置递增旧值恢复

当 Java 8 方法中实例 `int` 字段或 `int[]` 元素的后置递增，及其返回旧值的数据流均可完整证明时，恢复层 SHALL 生成含该左值后置 `++` 的 Java 表达式。生成的完整类 MUST 返回写入前的值、写入递增后的值，并保持接收者、数组及下标的单次求值、异常与副作用顺序。任一条件不成立时 MUST 保守拒绝整条受影响链及其可观察生产者，不得输出丢失写入、重复求值或把新值当旧值的正文。

#### Scenario: Effectful field receiver returns the old value
- **WHEN** 一个返回 `int` 的方法把可观察调用所得对象的同一 `int` 字段后置递增，并立即返回复制出的旧值
- **THEN** 完整生成类 SHALL 可编译执行，接收者调用恰好一次，返回旧字段值，字段写入新值；溢出时仍遵守 `int` 回绕

#### Scenario: Effectful array and index return the old value
- **WHEN** 一个返回 `int` 的方法对有可观察求值的 `int[]` 数组和下标指向的同一元素后置递增，并立即返回复制出的旧值
- **THEN** 完整生成类 SHALL 可编译执行，数组和下标各求值一次，返回旧元素值，写入新值

#### Scenario: Exception before the update
- **WHEN** 字段接收者为 null，或数组接收者为 null、下标越界
- **THEN** 生成类 SHALL 在与原 class 相同的求值阶段抛出相同类别的异常，已经发生的可观察调用次数与顺序相同，写入不得发生

#### Scenario: Exception handler boundary inside the candidate chain
- **WHEN** 合法 class 的异常表只保护后置更新链的一部分，例如数组和下标调用在保护区外、元素读取和写入在保护区内
- **THEN** 恢复层 MUST NOT 把整链投影为处于单一 `try` 范围的后置表达式；若较早的 Region 阶段已拒绝该方法，报告 SHALL 保留实际拒绝区域的物理来源，且不得声称已恢复其异常边界。Region 拒绝时逐指令来源缺口另案处理

#### Scenario: Unproved identity or extra consumer
- **WHEN** 读取与写入不是同一字段/数组元素，返回值并非后置复制出的旧值，或任一复制值另有消费者
- **THEN** 恢复层 MUST 保留有关读取、复制、算术、写入、返回及可观察生产者的拒绝范围和物理来源，MUST NOT 将该链误报为一个后置递增表达式

#### Scenario: Sources and bounded output
- **WHEN** 调用方请求默认或完整来源，或递增恢复在正文/来源预算与取消处停止
- **THEN** 两种来源模式 SHALL 呈现相同正文，并保留实际接收者、数组、下标、读取、复制、加法、写入及返回的 BCI/成员来源；停止 MUST 遵守既有受限输出契约，不交付不完整但声称成功的更新
