## ADDED Requirements

### Requirement: 已证嵌套条件链的整数尾返回

当一个 Java 8 `int` 方法的条件链在两个直接整数常量生产块合流，并由唯一 stack phi 交给唯一 `ireturn` 时，系统 SHALL 在完整 CFG 正常前驱、SSA 使用、区域所有权和来源均已证明的情况下恢复按原测试边求值的整数条件表达式。生成的完整类源码 MUST 能以 Java 8 重编，且 `java -Xverify:all` 运行返回值及条件求值次数 MUST 与原 class 一致。证明不完整时 MUST 原子拒绝受影响的方法，不得输出会改变值或缺少返回的可编译宣称。

#### Scenario: 嵌套布尔选择返回整数 1 或 2

- **WHEN** `nested(boolean,boolean,boolean)` 的 BCI 1/5/12 测试汇到 BCI 15/19 的两个 int 常量，再由 BCI 20 唯一返回，所有证明门闭合
- **THEN** Jarde SHALL 生成保留原物理分支次序的 `return` 条件值；原 class、固定 JADX、Jarde 的完整 Java 8 类源码 SHALL 重编并以 `java -Xverify:all` 运行 13 行一致

#### Scenario: 布尔 1/0 消费保持原门

- **WHEN** 既有短路值由 1/0 生产并用于布尔返回、字段、数组、调用或局部消费者
- **THEN** 系统 MUST 保持其已有布尔类型检查、短路简化和拒绝门，不得将整数返回路径的放宽套用到这些消费者

#### Scenario: 整数尾路径不完整

- **WHEN** 任一 producer 非直接 int 常量、phi 有额外输入或使用、consumer 不是唯一 `int ireturn`，或存在未闭合的正常/异常边、来源与预算停止
- **THEN** 系统 MUST 保留可定位物理 BCI 和未证报告，MUST NOT 发表不完整的嵌套条件值
