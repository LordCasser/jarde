## ADDED Requirements

### Requirement: 函数表达式作为构造实参

对于参数在 allocation 与构造调用之间产生、构造实例有唯一可呈现消费者、函数表达式的 bootstrap/SAM/handle/capture/类型适配均获证明且整个表达式处于相同异常处理范围的输入，系统 SHALL 呈现完整构造表达式和完整方法。函数值的创建、捕获、构造器其他参数及函数体调用的次数与次序 MUST 保持；任意 bootstrap 不得仅因 opcode 为 invokedynamic 获得函数语法。

#### Scenario: 双编译腿的构造函数表达式族
- **WHEN** 输入为冻结的 Runnable 无捕获/捕获、Comparator boxed 参数、Callable boxed 返回、IntUnaryOperator 原始参数/返回、静态方法引用以及带 String 第二参数的 Thread 构造族，分别由真实 javac8 与 javac23 --release8 产生
- **THEN** 各方法 SHALL 完整呈现为含函数表达式的构造；全类文本剥离注释后 SHALL 在两条编译腿上编译，外部驱动的返回值及副作用 SHALL 与原 class 相同；不得用空方法或局部文本替代验收

#### Scenario: 捕获与其他构造参数的求值次序
- **WHEN** 第二参数的 name() 在构造时记录 trace=1，函数体在以后 run() 时记录 trace=13
- **THEN** 恢复后的创建阶段 SHALL 只记录 1，调用阶段 SHALL 记录 13，构造器名和 Runnable 重载 SHALL 与物理 descriptor 相符

#### Scenario: 跨异常处理边界的构造实参
- **WHEN** allocation、动态参数、构造调用或唯一消费者的异常处理范围不一致
- **THEN** 构造 SHALL 拒绝并保留相关 bytecode 与 origin；不得将动态参数或捕获移动到另一 handler 的覆盖范围

#### Scenario: 未证明与无关动态调用
- **WHEN** 动态调用不在构造参数的物理值依赖链中，或其 bootstrap/适配/capture 未获既有函数恢复证明
- **THEN** 相关区间 SHALL 可靠拒绝，完整保留 effect 与来源；不得输出猜测的 lambda 或有缺失参数的构造表达式

#### Scenario: 既有保守边界
- **WHEN** 输入为既有 unconsumed-construction-invokes 反例、弃置含动态参数的构造、或可空绑定方法引用
- **THEN** 未证明部分 SHALL 保持拒绝；普通调用、nested-new、array/concat 构造族与 legacy LG 的其他方法 SHALL 不退化
