## ADDED Requirements

### Requirement: 已提交局部的表达式内多读呈现

当一条表达式的多个操作数读取同一**已提交局部声明**（单 store 已作为声明语句呈现、SSA 单定义）时，系统 SHALL 按源码形态以该局部名多次呈现，方法行为完整。

#### Scenario: 单表达式四次读取主锚
- **WHEN** 输入为固定 `NI`（`n.make(3).v + n.make(2).outerTag() + externalMake(n,1)… + (… == n)`，`javac --release 8`）的 class 并恢复
- **THEN** 表达式 SHALL 完整呈现且剥离编译后 `-Xverify:all` 输出与原一致（`3/10/5/true`）

#### Scenario: 阈值下限对照零回退
- **WHEN** 输入为 `NJ`（2 消费者）
- **THEN** 渲染 SHALL 逐字节不变

#### Scenario: 栈携带与循环携带形仍拒
- **WHEN** 生产者是无声明的栈携带 saved 值，或为循环携带引用（增强 for 协议多读）
- **THEN** 拒绝文本 SHALL 逐字保持
