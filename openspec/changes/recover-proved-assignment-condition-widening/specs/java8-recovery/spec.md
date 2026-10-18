## ADDED Requirements

### Requirement: Proved assignment conditions permit int widening from narrow numeric values

当条件内局部赋值的值、执行路径、目标变量及其 `int` 声明均已证明，且右值有确切的 `byte`、`short` 或 `char` Java 类型时，系统 SHALL 接纳合法扩大转换，恢复原条件位置的一次赋值。系统 MUST 保留右值的值、求值次数、异常时序及赋值后变量的读取；MUST NOT 仅因共享 JVM 栈类别而认定源类型可转换。

#### Scenario: Fixed character-producing loop is completely recovered

- **WHEN** 固定完整 `TestLoopConditionInvoke` 的字符调用、局部赋值和循环早返路径均已证明
- **THEN** 系统 SHALL 输出完整结构化的 `test(C)Z` 与可重编译的完整类，保留一次条件调用、终止字符判断、命中返回与退出后位置恢复；MUST NOT 因合法 `char` 到 `int` 赋值而撤回整个方法

#### Scenario: Dynamic input preserves values and effects

- **WHEN** 独立完整控制输入覆盖立即命中、先不匹配再命中、非零起始位置遇终止符、终止符等于查找值、数组越界及字符65535
- **THEN** 完整生成类重编译后的返回、异常类别、位置、调用次数及事件顺序 SHALL 与同一原始输入一致，右值 MUST 在原条件位置只求值一次，MUST NOT 插入冗余强制转换或提前求值

#### Scenario: Exact assignments retain their existing behavior

- **WHEN** 已证明的右值类型与目标声明完全相同
- **THEN** 系统 SHALL 保留既有赋值准入、源文本和物理来源；新增扩大转换能力 MUST NOT 改变原精确类型路径

#### Scenario: Missing type or incompatible conversion remains refused

- **WHEN** 右值类型未知、boolean 与不同类型混用、引用与基本类型混用、引用类型不匹配、发生窄化，或局部复制/使用/路径证据不完整
- **THEN** 系统 MUST 保留原拒绝与物理来源，MUST NOT 将缺证或 JVM int 栈形当成扩大转换证明，也 MUST NOT 发布部分条件赋值

#### Scenario: Evidence and stops preserve atomic output

- **WHEN** 同一输入请求 default/all/essential 证据，或赋值构建遇到预算、取消、期限停止
- **THEN** 成功请求的代码决定 SHALL 一致，raw方法与source map SHALL 对应真实物理方法和BCI；真实停止 MUST 继续经原事务传播，MUST NOT 变成缺证后发布半完成源码或映射
