## ADDED Requirements

### Requirement: Proved mixed short-circuit values may feed one Boolean array element store

当 Java 8 方法的有界无环短路测试图以真实 1/0 producer 汇入唯一栈 Phi，该 Phi 的唯一直接 use 是真实 `bastore` 的值操作数，且数组值可证明为 `[Z`，系统 SHALL 在 store 位置使用现有数组元素赋值语法恢复一次惰性 Boolean RHS。系统 SHALL 独立证明数组、下标和值三个操作数的 SSA 身份、一次性和实际求值顺序，并保留所有被认领指令的来源。不能仅凭 `bastore`、1/0 常量、可编译源码或 JADX 输出推断 Boolean 类型。任一物理 owner、正常/异常边、类型、操作数或预算证明不完整时 MUST 原子拒绝相关候选，不得发表部分赋值。

#### Scenario: One Boolean array store preserves three operands
- **WHEN** `array(nullArray)[index(pos)] = (a && b()) || c()` 编译为数组和下标先求值、短路 Phi 作为 BCI 29 `bastore` 的第三操作数
- **THEN** 完整 Java 8 类 SHALL 编译，并在八组布尔输入乘以正常/null/越界三种目标的 24 路径上，数组值、数组/下标/b/c 调用次数及异常类别 SHALL 与原 class 逐行相同

#### Scenario: Null or bounds failure follows RHS evaluation
- **WHEN** `array(nullArray)` 返回 null 或 `index(pos)` 给出越界位置，而 RHS 的 b/c 调用可观察
- **THEN** 恢复源码 SHALL 在适用 RHS 求值后才出现 `NullPointerException` 或 `ArrayIndexOutOfBoundsException`，数组和下标各求值一次，不得提前检查目标或重复 RHS

#### Scenario: Byte array shares the store opcode
- **WHEN** 目标只证明为 `[B` 或组件类型未知，即使真实指令为 `bastore` 且 producer 为 1/0
- **THEN** 系统 MUST 拒绝将其恢复为 `boolean[]` 赋值，并保留候选来源

#### Scenario: Ownership or operand evidence is incomplete
- **WHEN** 物理块有第二 owner、Phi 有第二直接 consumer、数组/下标值有无法证明的用途，或图有额外正常/异常边
- **THEN** 系统 MUST 保留原子 fallback，不得因已识别数组赋值语法而绕过 Region、SSA 或预算合同
