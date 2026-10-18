## Why

Seq1循环头恢复的真实定向测试已通过原始完整方法，但完整副作用补充在第二个条件的`dup_x1; putfield; …; if_icmp`处失败。右值被复制后同时用于字段写入和比较；既有字段复制证明只覆盖字段链及读改写接收者，不能把此处当普通dup或用重新读取字段代替原值。

## What Changes

- 为同块、单次右值求值、唯一字段写入和直接整数比较的精确复制形态补充局部证明，保留字段写入时机、比较原值及物理来源。
- 复用现有字段复制计划和保存局部，不新增公共AST、Operation、pass或通用栈置换机制。
- 真实Reader正反例与完整原/JADX/Jarde双编译器、双VM对照验收，保留首次失败。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 精确证明的实例int字段赋值后直接比较可恢复，比较读取复制的原值。

## Impact

范围限于jarde-java的FieldCopies及其现有builder消费者、测试和证据。依赖既有field符号/类型证明；Seq1完整循环验收依赖本片，本片不改变region所有权。前置真实失败为`/private/tmp/jarde-seq1-focused-root-v1-20261018`，原始测试1通过、补充和依赖其正例的Stop测试2失败；无新CLI。非目标为任意接收者、category2、数组写、跨块/短路赋值表达式、异常handler与通用dup_x1解释器。71/612不增计。
