## ADDED Requirements

### Requirement: A proved short-circuit field chain preserves every decision and its one write

Java 8 恢复结果遇到同极性、至少三次条件判断共用一个布尔值生产者，并将汇合值一次写入静态 `Z` 字段时，若每条路径、值来源、异常边界、字段身份与唯一消费均有证据，SHALL 输出可用 Java 8 模式重编且与原 class 的字段值、右侧调用次数及异常顺序相同的正文。未证明时 MUST 保留全部判断、两个值生产者、字段写入和同块后缀的来源缺口，MUST NOT 把缺少写入的可编译文本标为完整恢复。目标字段是 `Z` 本身不证明任意 JVM 整数等于布尔字面量。

#### Scenario: A three-condition OR chain skips later operands

- **WHEN** Java 8 编译的 `result = extra || left || rhs()` 以三个条件出口共享 true 生产者，`rhs()` 增加可观察调用计数并返回可切换的值
- **THEN** 重编的完整类在 `extra=true`、`extra=false,left=true`、前两项都 false 且 `rhs=true`、前两项都 false 且 `rhs=false` 四条路径上 SHALL 与原 class 的字段值和调用次数逐项一致；前两条路径 MUST 不调用 `rhs()`，字段赋值只出现一次

#### Scenario: A producer or consumer has an unproved extra use

- **WHEN** 同类短路链的共享生产者还有外部入口、Phi 还有第二消费者、某个判断带有无法原位呈现的额外效果，或目标字段身份无法确认
- **THEN** 报告 MUST 拒绝链的结构化字段赋值，同时在引用及 source map 中保留所有参与条件、生产者、`putstatic` 与后缀 BCI，不得把重访消费块称作真实循环而遗漏生产者

#### Scenario: Existing two-condition and exception boundaries survive

- **WHEN** 输入是已证明的两次判断 `&&`/`||` 字段写入，或在 `try` 保护区内带有实际异常边的短路写入
- **THEN** 前两种 SHALL 保持原有短路求值与唯一字段赋值；异常边未证明时 MUST 保持可见的整体拒绝及 catch 来源，不能因链识别扩大而发射没有异常路径证明的赋值
