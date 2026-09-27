## ADDED Requirements

### Requirement: Proved ordered static field initializer projection for ordinary classes

对 Java 8 普通类源码请求，系统 SHALL 仅在同轮完整 `<clinit>()V` 候选证明全部运行时静态字段各有唯一、有序且可呈现的赋值，RHS 字段读取与副作用次序在声明处保持原样时，将整组赋值投影为对应字段声明初值并省去该 `<clinit>` 的源码正文。实例字段和构造器 SHALL 保持原有位置与语义；任何证明失败或停止 SHALL 不发布部分字段初值投影。

#### Scenario: Ordered dependency chain

- **WHEN** 普通类的唯一完整 `<clinit>` 依次写入 `trace`、`a`、`b`、`c`、`result`，每次 RHS 只读取已按原顺序初始化的同类字段，且没有额外顶层效果
- **THEN** 完整类源码以相同顺序在这五个静态字段声明处写初值，Java 8 重编及验证运行同原 class 一致

#### Scenario: Instance initialization must remain in constructor

- **WHEN** 普通类构造器先调用方法写实例状态，再以读取该状态的方法结果赋给实例字段
- **THEN** 该赋值 SHALL 仍在构造器内，不得被本静态组投影移动到实例字段声明

#### Scenario: Incomplete or reorder-unsafe group

- **WHEN** 静态字段写入缺失或重复、存在额外顶层效果、ConstantValue 阶段冲突、前向读取、异常边、缺失或歧义 `<clinit>`、成员事实不完整，或 RHS 字段身份不可证明
- **THEN** 系统 SHALL 保留原字段声明和静态块，不发布该组的任何前缀投影

#### Scenario: Budget stop

- **WHEN** 整组证明或表达式发射被预算或取消停止
- **THEN** 系统 SHALL 依既有停止规则报告，并保持最终源码中这组静态字段未被部分改写
