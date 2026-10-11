## ADDED Requirements

### Requirement: A proved computed initializer retains a counted for presentation

系统 SHALL 将已经安全恢复的一次性计算初始化纳入具有唯一计数更新的 `for` 呈现，保持初始化、每轮条件、正文、返回与更新的执行位置、次数和异常次序，以及完整物理来源。无法证明时 MUST 保留可靠的既有呈现或解释，不能为了出现 `for` 重复表达式或丢失控制流。

#### Scenario: Reverse search starts with a computed index

- **WHEN** 冻结 `lastIndexOf([IIII)I` 将 `end - 1` 写入唯一循环变量，循环内比较与递减已证明，命中时只在唯一返回叶读取该变量
- **THEN** 完整源码包含一个 `for`，初始化为原来的减法，命中返回索引、未命中返回 `-1`；原完整类与重编源码在相同输入的结果一致，方法所有物理 BCI 包括回跳 `25→5` 均有准确来源

#### Scenario: An initializer has observable effects

- **WHEN** 初始化依赖调用、可能抛异常的读取、共享值或生产者与存储之间的独立副作用
- **THEN** 系统 SHALL 复用已证明的一次求值和原语句位置；独立调用、异常及共享值的观察结果与原 class 相同，证明不足时不发布会重排或重复求值的 for 呈现

#### Scenario: A local is also accessed outside the proved loop

- **WHEN** 循环变量在精确证明的命中返回叶中仅被读取
- **THEN** 该读取不会单独阻止 for 呈现，且返回叶依然只属于一个正文位置

#### Scenario: A loop or outside access is ambiguous

- **WHEN** 循环变量在其他循环外位置被读取或写入，返回叶存在额外前驱，更新或回边不唯一，边界值在循环内改变，或者初始化不能保持作用域和异常上下文
- **THEN** 系统 MUST 拒绝新的 for 投影，保留完整可靠的控制流、物理报告与拒绝边界

#### Scenario: Evidence modes and the complete source agree

- **WHEN** 同一冻结输入以 default/all 证据配置恢复，并使用固定原/JADX/Jarde 完整类及同一 runner 双 JDK 对照
- **THEN** 两个 Jarde 配置的正文、身份与来源一致，完整源码原样重编运行与原 class 的 exit/stdout/stderr 一致；JADX 仅提供独立对照，不替代原 class 的行为真值

#### Scenario: The counted-loop proof is stopped

- **WHEN** 证明、初始化消费或来源发布触发预算不足或取消
- **THEN** 系统 SHALL 传播准确既有 Stop 状态，MUST 不发布部分正文、部分 source map 或未完成循环产物
