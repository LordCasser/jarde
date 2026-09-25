## ADDED Requirements

### Requirement: A proved shared-true short-circuit value writes its static field exactly once

Java 8 源恢复遇到两次条件分支共用 true 值生产者、false 值另由一条分支产生，并在汇合后写入静态 `Z` 字段时，若所有路径、值来源、字段身份和唯一消费均可证明，SHALL 生成可重编且与原 class 字段值、短路求值次数及异常顺序相同的 Java 正文。右侧条件 MUST 只在左侧没有直接产生 true 时求值，`putstatic` MUST 恰好表现一次。证明不充分时 MUST 引用分支、生产者、字段写入及同块后续指令，MUST NOT 把缺写入的可编译文本标为完整等价恢复。

#### Scenario: A true left operand skips the right call

- **WHEN** Java 8 编译的 `result = left || rhs()` 以共享 true 生产者汇合，`rhs()` 递增可观察计数
- **THEN** 已证明的重编源码在 `left=true` 时 SHALL 写 `result=true` 且调用计数保持 0，在 `left=false` 时 SHALL 写 `result=true` 且调用计数为 1；字段赋值只出现一次，原 class、重编 class 的两行运行输出相同

#### Scenario: An unproved shared-true value remains a complete gap

- **WHEN** 同形图形有额外入口、非唯一汇合值或第二消费者，或者某个条件值含无法原位呈现的额外效果
- **THEN** 输出 MUST 保留所有参与分支、两臂生产者、原字段写入及消费者后缀的 BCI 和来源，质量 MUST NOT 为完整结构化 Java；不能发射一个只含部分 `if` 或省略写入的等价声明

#### Scenario: A static boolean field does not erase raw integer evidence

- **WHEN** 汇合两臂供给 JVM 可接受但不是精确 1/0 的整数，而目标字段描述符是 `Z`
- **THEN** 恢复 MUST 保留整数最低位存储语义或保守拒绝该折叠，MUST NOT 因字段类型把任意整数生产者改写成布尔字面量
